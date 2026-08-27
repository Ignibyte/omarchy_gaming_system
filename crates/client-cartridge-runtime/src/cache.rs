use std::{
    fs::{self, File},
    io::{Read as _, Write as _},
    os::{
        fd::OwnedFd,
        unix::{fs::MetadataExt as _, fs::PermissionsExt as _},
    },
    path::Path,
    sync::Mutex,
};

use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CatalogPublicKey, CatalogStatus, LifecycleUse, PublisherPublicKey,
    SecureCartridgeStore, SecureResolution, VerifiedAcquisition, rich_2d_host_profile,
};
use rand_core::{OsRng, RngCore as _};
use rustix::{
    fs::{
        AtFlags, FileType, FlockOperation, Mode, OFlags, RenameFlags, fchmod, flock, fstat, fsync,
        mkdirat, open, openat, renameat, renameat_with, unlinkat,
    },
    process::{Uid, geteuid},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

use crate::{
    CompanionError, Result,
    remote::{CatalogRelease, selected_origin},
};

const PROFILE_FORMAT: &str = "omarchygs.client-profile-mounts/v1";
const MOUNT_FORMAT: &str = "omarchygs.client-cartridge-mount/v1";
const MAX_PROFILE_BYTES: u64 = 256 * 1024;
const MAX_PUBLISHER_KEY_BYTES: u64 = 64 * 1024;
const MAX_MOUNTS: usize = 128;

pub struct ClientCartridgeCache {
    _root: OwnedFd,
    profiles: OwnedFd,
    publisher_keys: OwnedFd,
    policies: OwnedFd,
    lock_file: OwnedFd,
    process_lock: Mutex<()>,
    content: SecureCartridgeStore,
}

impl ClientCartridgeCache {
    pub fn open(root: &Path) -> Result<Self> {
        create_private_root(root)?;
        let owner = geteuid();
        let root_fd = open(
            root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_directory(&root_fd, owner)?;
        let profiles = open_or_create_directory(&root_fd, "profiles", owner)?;
        let publisher_keys = open_or_create_directory(&root_fd, "publisher-keys", owner)?;
        let policies = open_or_create_directory(&root_fd, "policies", owner)?;
        open_or_create_directory(&root_fd, "content", owner)?;
        let lock_file = openat(
            &root_fd,
            ".mounts.lock",
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_regular_file(&lock_file, owner)?;
        let content = SecureCartridgeStore::open_existing(&root.join("content"))
            .map_err(|_| CompanionError::Cache)?;
        fsync(&root_fd).map_err(|_| CompanionError::Cache)?;
        Ok(Self {
            _root: root_fd,
            profiles,
            publisher_keys,
            policies,
            lock_file,
            process_lock: Mutex::new(()),
            content,
        })
    }

    pub fn mounts_all(&self, server_id: Uuid) -> Result<Vec<MountRecord>> {
        self.with_mount_lock(|| Ok(self.read_profile(server_id)?.mounts))
    }

    pub fn mounts(
        &self,
        server_id: Uuid,
        trusted_marketplace_key: &CatalogPublicKey,
    ) -> Result<Vec<MountRecord>> {
        let expected = marketplace_key_sha256(trusted_marketplace_key)?;
        let mounts = self.mounts_all(server_id)?;
        if mounts
            .iter()
            .all(|mount| mount.marketplace_key_sha256 == expected)
        {
            Ok(mounts)
        } else {
            Err(CompanionError::MarketplaceUntrusted)
        }
    }

    pub fn install(
        &self,
        verified: &VerifiedAcquisition,
        mount: MountRecord,
    ) -> Result<MountRecord> {
        self.install_for_use(verified, mount, LifecycleUse::NewLaunch)
    }

    pub(crate) fn install_session(
        &self,
        verified: &VerifiedAcquisition,
        mount: MountRecord,
    ) -> Result<MountRecord> {
        self.install_for_use(verified, mount, LifecycleUse::ActiveSession)
    }

    fn install_for_use(
        &self,
        verified: &VerifiedAcquisition,
        mount: MountRecord,
        use_kind: LifecycleUse,
    ) -> Result<MountRecord> {
        mount.validate()?;
        let evidence_key_sha256 = marketplace_key_sha256(verified.marketplace_key())?;
        let policy_key_sha256 = marketplace_key_sha256(verified.policy_marketplace_key())?;
        if mount.marketplace_key_sha256 != evidence_key_sha256
            || mount.policy_marketplace_key_sha256.as_deref() != Some(policy_key_sha256.as_str())
            || mount.policy_snapshot_version != Some(verified.policy_snapshot_version())
        {
            return Err(CompanionError::Rejected);
        }
        let staged = self
            .content
            .stage_reviewed_release_for_use(
                verified.release(),
                verified.policy_bytes(),
                verified.policy_marketplace_key(),
                use_kind,
            )
            .map_err(|_| CompanionError::Cache)?;
        if !staged.installed || staged.release.archive_sha256 != mount.archive_sha256 {
            return Err(CompanionError::AdmissionChanged);
        }
        let server_id = exact_uuid(&mount.server_id)?;
        self.with_mount_lock(|| {
            let mut profile = self.read_profile(server_id)?;
            if profile
                .mounts
                .iter()
                .any(|existing| existing.server_origin != mount.server_origin)
            {
                return Err(CompanionError::Rejected);
            }
            self.write_publisher_key(&mount.archive_sha256, &verified.entry().publisher_key)?;
            self.write_policy(
                &mount.archive_sha256,
                &policy_key_sha256,
                verified.policy_bytes(),
            )?;
            profile.mounts.retain(|existing| {
                existing.game_key != mount.game_key
                    || existing.archive_sha256 != mount.archive_sha256
                    || existing.admission_revision != mount.admission_revision
            });
            profile.mounts.push(mount.clone());
            profile.mounts.sort_by(mount_order);
            if profile.mounts.len() > MAX_MOUNTS {
                return Err(CompanionError::Cache);
            }
            self.write_profile_all(server_id, &profile)?;
            Ok(mount)
        })
    }

    pub fn remove_exact(
        &self,
        server_id: Uuid,
        game_key: &str,
        digest: &str,
        admission_revision: Option<u64>,
    ) -> Result<bool> {
        if !valid_identifier(game_key)
            || !valid_sha256(digest)
            || admission_revision.is_some_and(|revision| revision == 0)
        {
            return Err(CompanionError::InvalidInput);
        }
        self.with_mount_lock(|| {
            let mut profile = self.read_profile(server_id)?;
            let before = profile.mounts.len();
            profile.mounts.retain(|mount| {
                mount.game_key != game_key
                    || mount.archive_sha256 != digest
                    || admission_revision
                        .is_some_and(|revision| mount.admission_revision != revision)
            });
            if profile.mounts.len() == before {
                return Ok(false);
            }
            self.write_profile_all(server_id, &profile)?;
            Ok(true)
        })
    }

    pub fn remove(
        &self,
        server_id: Uuid,
        game_key: &str,
        digest: &str,
        admission_revision: Option<u64>,
        trusted_marketplace_key: &CatalogPublicKey,
    ) -> Result<bool> {
        let expected = marketplace_key_sha256(trusted_marketplace_key)?;
        if !self
            .mounts_all(server_id)?
            .iter()
            .all(|mount| mount.marketplace_key_sha256 == expected)
        {
            return Err(CompanionError::MarketplaceUntrusted);
        }
        self.remove_exact(server_id, game_key, digest, admission_revision)
    }

    pub(crate) fn resolve_mounted(
        &self,
        server_origin: &str,
        server_id: Uuid,
        game_key: &str,
        archive_sha256: &str,
        admission_revision: u64,
        trust: &impl CacheTrust,
    ) -> Result<SecureResolution> {
        let server_origin = canonical_origin(server_origin)?;
        if !valid_identifier(game_key) || !valid_sha256(archive_sha256) || admission_revision == 0 {
            return Err(CompanionError::InvalidInput);
        }
        let (mount, publisher_key, policy_bytes) = self.with_mount_lock(|| {
            let mount = self
                .read_profile(server_id)?
                .mounts
                .into_iter()
                .find(|mount| {
                    mount.server_origin == server_origin
                        && mount.game_key == game_key
                        && mount.archive_sha256 == archive_sha256
                        && mount.admission_revision == admission_revision
                })
                .ok_or(CompanionError::MountMissing)?;
            let publisher_key = self.read_publisher_key(archive_sha256)?;
            if publisher_key.publisher_id != mount.publisher_id {
                return Err(CompanionError::Cache);
            }
            let policy_fingerprint = mount
                .policy_marketplace_key_sha256
                .as_deref()
                .unwrap_or(&mount.marketplace_key_sha256);
            let policy_bytes = read_optional_regular_file(
                &self.policies,
                &policy_name(archive_sha256, policy_fingerprint),
                omarchygs_game_cartridge::MAX_JSON_BYTES as u64,
            )?;
            Ok((mount, publisher_key, policy_bytes))
        })?;
        let trust = trust.snapshot();
        trust.key_by_fingerprint(&mount.marketplace_key_sha256, mount.snapshot_version)?;
        let policy_fingerprint = mount
            .policy_marketplace_key_sha256
            .as_deref()
            .unwrap_or(&mount.marketplace_key_sha256);
        let policy_snapshot_version = mount
            .policy_snapshot_version
            .unwrap_or(mount.snapshot_version);
        let policy_key = trust.key_by_fingerprint(policy_fingerprint, policy_snapshot_version)?;
        trust.authorize_current_key(&policy_key, policy_snapshot_version)?;
        let resolution = if let Some(policy_bytes) = policy_bytes {
            self.content.resolve_exact(
                game_key,
                archive_sha256,
                &publisher_key,
                &rich_2d_host_profile(),
                &policy_bytes,
                &policy_key,
                LifecycleUse::ActiveSession,
            )
        } else if policy_fingerprint == mount.marketplace_key_sha256 {
            self.content.resolve_cached_exact(
                game_key,
                archive_sha256,
                &publisher_key,
                &rich_2d_host_profile(),
                &policy_key,
                LifecycleUse::ActiveSession,
            )
        } else {
            return Err(CompanionError::Cache);
        }
        .map_err(|_| CompanionError::AdmissionChanged)?;
        let activation = resolution.activation();
        if activation.publisher_id != mount.publisher_id
            || activation.cartridge_version != mount.cartridge_version
            || activation.archive_sha256 != mount.archive_sha256
            || activation.signed_identity_sha256 != mount.signed_identity_sha256
            || resolution.cartridge().manifest().rules_version != mount.rules_version
        {
            return Err(CompanionError::Cache);
        }
        Ok(resolution)
    }

    fn with_mount_lock<T>(&self, operation: impl FnOnce() -> Result<T>) -> Result<T> {
        let _process = self
            .process_lock
            .lock()
            .map_err(|_| CompanionError::Cache)?;
        flock(&self.lock_file, FlockOperation::LockExclusive).map_err(|_| CompanionError::Cache)?;
        let result = operation();
        if flock(&self.lock_file, FlockOperation::Unlock).is_err() {
            return Err(CompanionError::Cache);
        }
        result
    }

    fn read_profile(&self, server_id: Uuid) -> Result<ProfileDocument> {
        let name = profile_name(server_id);
        let file = match openat(
            &self.profiles,
            name.as_str(),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(file) => file,
            Err(rustix::io::Errno::NOENT) => return Ok(ProfileDocument::empty()),
            Err(_) => return Err(CompanionError::Cache),
        };
        validate_regular_file(&file, geteuid())?;
        let metadata = fstat(&file).map_err(|_| CompanionError::Cache)?;
        if metadata.st_size == 0 || metadata.st_size as u64 > MAX_PROFILE_BYTES {
            return Err(CompanionError::Cache);
        }
        let mut bytes = Vec::with_capacity(metadata.st_size as usize);
        File::from(file)
            .take(MAX_PROFILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| CompanionError::Cache)?;
        if bytes.len() as u64 > MAX_PROFILE_BYTES {
            return Err(CompanionError::Cache);
        }
        let profile: ProfileDocument =
            serde_json::from_slice(&bytes).map_err(|_| CompanionError::Cache)?;
        profile.validate(server_id)?;
        Ok(profile)
    }

    #[cfg(test)]
    fn write_profile(
        &self,
        server_id: Uuid,
        profile: &ProfileDocument,
        trusted_key_sha256: &str,
    ) -> Result<()> {
        if !valid_sha256(trusted_key_sha256)
            || !profile
                .mounts
                .iter()
                .all(|mount| mount.marketplace_key_sha256 == trusted_key_sha256)
        {
            return Err(CompanionError::Cache);
        }
        self.write_profile_all(server_id, profile)
    }

    fn write_profile_all(&self, server_id: Uuid, profile: &ProfileDocument) -> Result<()> {
        profile.validate(server_id)?;
        let bytes = serde_json::to_vec(profile).map_err(|_| CompanionError::Cache)?;
        if bytes.is_empty() || bytes.len() as u64 > MAX_PROFILE_BYTES {
            return Err(CompanionError::Cache);
        }
        let target = profile_name(server_id);
        let temporary = temporary_name();
        let fd = openat(
            &self.profiles,
            temporary.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(|_| CompanionError::Cache)?;
        let result = (|| {
            validate_regular_file(&fd, geteuid())?;
            let mut file = File::from(fd);
            file.write_all(&bytes).map_err(|_| CompanionError::Cache)?;
            file.sync_all().map_err(|_| CompanionError::Cache)?;
            fchmod(&file, Mode::from_bits_truncate(0o400)).map_err(|_| CompanionError::Cache)?;
            file.sync_all().map_err(|_| CompanionError::Cache)?;
            renameat(
                &self.profiles,
                temporary.as_str(),
                &self.profiles,
                target.as_str(),
            )
            .map_err(|_| CompanionError::Cache)?;
            fsync(&self.profiles).map_err(|_| CompanionError::Cache)
        })();
        if result.is_err() {
            let _ = unlinkat(&self.profiles, temporary.as_str(), AtFlags::empty());
        }
        result
    }

    fn read_publisher_key(&self, archive_sha256: &str) -> Result<PublisherPublicKey> {
        if !valid_sha256(archive_sha256) {
            return Err(CompanionError::InvalidInput);
        }
        let bytes = read_regular_file(
            &self.publisher_keys,
            &format!("{archive_sha256}.json"),
            MAX_PUBLISHER_KEY_BYTES,
        )?;
        let key: PublisherPublicKey =
            serde_json::from_slice(&bytes).map_err(|_| CompanionError::Cache)?;
        if serde_json::to_vec(&key).map_err(|_| CompanionError::Cache)? != bytes {
            return Err(CompanionError::Cache);
        }
        Ok(key)
    }

    fn write_publisher_key(&self, archive_sha256: &str, key: &PublisherPublicKey) -> Result<()> {
        if !valid_sha256(archive_sha256) {
            return Err(CompanionError::InvalidInput);
        }
        let bytes = serde_json::to_vec(key).map_err(|_| CompanionError::Cache)?;
        if bytes.is_empty() || bytes.len() as u64 > MAX_PUBLISHER_KEY_BYTES {
            return Err(CompanionError::Cache);
        }
        let target = format!("{archive_sha256}.json");
        if let Some(existing) =
            read_optional_regular_file(&self.publisher_keys, &target, MAX_PUBLISHER_KEY_BYTES)?
        {
            return if existing == bytes {
                Ok(())
            } else {
                Err(CompanionError::Cache)
            };
        }
        let temporary = temporary_name();
        let fd = openat(
            &self.publisher_keys,
            temporary.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(|_| CompanionError::Cache)?;
        let result = (|| {
            validate_regular_file(&fd, geteuid())?;
            let mut file = File::from(fd);
            file.write_all(&bytes).map_err(|_| CompanionError::Cache)?;
            file.sync_all().map_err(|_| CompanionError::Cache)?;
            fchmod(&file, Mode::from_bits_truncate(0o400)).map_err(|_| CompanionError::Cache)?;
            drop(file);
            match renameat_with(
                &self.publisher_keys,
                temporary.as_str(),
                &self.publisher_keys,
                target.as_str(),
                RenameFlags::NOREPLACE,
            ) {
                Ok(()) => {}
                Err(rustix::io::Errno::EXIST) => {
                    unlinkat(&self.publisher_keys, temporary.as_str(), AtFlags::empty())
                        .map_err(|_| CompanionError::Cache)?;
                    if read_regular_file(&self.publisher_keys, &target, MAX_PUBLISHER_KEY_BYTES)?
                        != bytes
                    {
                        return Err(CompanionError::Cache);
                    }
                }
                Err(_) => return Err(CompanionError::Cache),
            }
            fsync(&self.publisher_keys).map_err(|_| CompanionError::Cache)
        })();
        if result.is_err() {
            let _ = unlinkat(&self.publisher_keys, temporary.as_str(), AtFlags::empty());
        }
        result
    }

    fn write_policy(&self, archive_sha256: &str, key_sha256: &str, bytes: &[u8]) -> Result<()> {
        if !valid_sha256(archive_sha256)
            || !valid_sha256(key_sha256)
            || bytes.is_empty()
            || bytes.len() > omarchygs_game_cartridge::MAX_JSON_BYTES
        {
            return Err(CompanionError::Cache);
        }
        let target = policy_name(archive_sha256, key_sha256);
        let temporary = temporary_name();
        let fd = openat(
            &self.policies,
            temporary.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(|_| CompanionError::Cache)?;
        let result = (|| {
            validate_regular_file(&fd, geteuid())?;
            let mut file = File::from(fd);
            file.write_all(bytes).map_err(|_| CompanionError::Cache)?;
            file.sync_all().map_err(|_| CompanionError::Cache)?;
            fchmod(&file, Mode::from_bits_truncate(0o400)).map_err(|_| CompanionError::Cache)?;
            file.sync_all().map_err(|_| CompanionError::Cache)?;
            renameat(
                &self.policies,
                temporary.as_str(),
                &self.policies,
                target.as_str(),
            )
            .map_err(|_| CompanionError::Cache)?;
            fsync(&self.policies).map_err(|_| CompanionError::Cache)
        })();
        if result.is_err() {
            let _ = unlinkat(&self.policies, temporary.as_str(), AtFlags::empty());
        }
        result
    }
}

pub(crate) trait CacheTrust {
    fn snapshot(&self) -> crate::ClientTrustSnapshot;
}

impl CacheTrust for CatalogPublicKey {
    fn snapshot(&self) -> crate::ClientTrustSnapshot {
        crate::ClientTrustSnapshot::Manual(std::sync::Arc::new(self.clone()))
    }
}

impl CacheTrust for crate::ClientTrustSnapshot {
    fn snapshot(&self) -> crate::ClientTrustSnapshot {
        self.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MountRecord {
    pub format: String,
    pub server_id: String,
    pub server_origin: String,
    pub game_key: String,
    pub publisher_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub display_name: String,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub marketplace_key_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_marketplace_key_sha256: Option<String>,
    pub marketplace_id: String,
    pub marketplace_name: String,
    pub reviewed_by: String,
    pub review_summary: String,
    pub snapshot_version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_snapshot_version: Option<u64>,
    pub policy_version: u64,
    pub lifecycle_status: String,
    pub admission_revision: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl MountRecord {
    pub(crate) fn from_verified(
        server_origin: String,
        server_id: Uuid,
        selected: &CatalogRelease,
        verified: &VerifiedAcquisition,
    ) -> Result<Self> {
        let snapshot = verified.snapshot();
        let entry = verified.entry();
        let policy = verified.policy();
        if snapshot.authority_id != selected.marketplace.marketplace_id
            || snapshot.marketplace_name != selected.marketplace.marketplace_name
            || entry.reviewed_by != selected.marketplace.reviewed_by
            || entry.review_summary != selected.marketplace.review_summary
            || policy.policy_version != selected.marketplace.policy_version
            || lifecycle_name(policy.status) != selected.marketplace.lifecycle_status
        {
            return Err(CompanionError::Rejected);
        }
        let mount = Self {
            format: MOUNT_FORMAT.to_owned(),
            server_id: server_id.to_string(),
            server_origin,
            game_key: selected.game_key.clone(),
            publisher_id: selected.publisher_id.clone(),
            rules_version: selected.rules_version,
            cartridge_version: selected.cartridge_version,
            display_name: selected.display_name.clone(),
            archive_sha256: selected.archive_sha256.clone(),
            signed_identity_sha256: selected.signed_identity_sha256.clone(),
            marketplace_key_sha256: marketplace_key_sha256(verified.marketplace_key())?,
            policy_marketplace_key_sha256: Some(marketplace_key_sha256(
                verified.policy_marketplace_key(),
            )?),
            marketplace_id: selected.marketplace.marketplace_id.clone(),
            marketplace_name: selected.marketplace.marketplace_name.clone(),
            reviewed_by: selected.marketplace.reviewed_by.clone(),
            review_summary: selected.marketplace.review_summary.clone(),
            snapshot_version: snapshot.snapshot_version,
            policy_snapshot_version: Some(verified.policy_snapshot_version()),
            policy_version: selected.marketplace.policy_version,
            lifecycle_status: selected.marketplace.lifecycle_status.clone(),
            admission_revision: selected.server_admission.revision,
            trust_status: None,
            warning: selected.warning.clone(),
        };
        mount.validate()?;
        Ok(mount)
    }

    pub(crate) fn from_session_verified(
        server_origin: String,
        server_id: Uuid,
        admission: &AcquisitionServerAdmission,
        verified: &VerifiedAcquisition,
    ) -> Result<Self> {
        let release = verified.release();
        let manifest = release.cartridge().manifest();
        let payload = release.payload();
        let snapshot = verified.snapshot();
        let entry = verified.entry();
        let policy = verified.policy();
        if admission.server_id != server_id.to_string()
            || admission.game_key != manifest.game_key
            || admission.publisher_id != manifest.publisher_id
            || admission.rules_version != manifest.rules_version
            || admission.cartridge_version != manifest.cartridge_version
            || admission.archive_sha256 != release.cartridge().archive_sha256()
            || admission.signed_identity_sha256 != release.cartridge().signed_identity_sha256()
            || admission.game_key != payload.game_key
            || admission.publisher_id != payload.publisher_id
            || admission.rules_version != payload.rules_version
            || admission.cartridge_version != payload.cartridge_version
            || admission.archive_sha256 != payload.archive_sha256
            || admission.signed_identity_sha256 != payload.signed_identity_sha256
            || admission.admission_revision == 0
            || snapshot.authority_id != verified.marketplace_key().authority_id
            || !matches!(
                policy.status,
                CatalogStatus::Active | CatalogStatus::Deprecated | CatalogStatus::Retired
            )
        {
            return Err(CompanionError::Rejected);
        }
        let mount = Self {
            format: MOUNT_FORMAT.to_owned(),
            server_id: server_id.to_string(),
            server_origin,
            game_key: admission.game_key.clone(),
            publisher_id: admission.publisher_id.clone(),
            rules_version: admission.rules_version,
            cartridge_version: admission.cartridge_version,
            display_name: manifest.display_name.clone(),
            archive_sha256: admission.archive_sha256.clone(),
            signed_identity_sha256: admission.signed_identity_sha256.clone(),
            marketplace_key_sha256: marketplace_key_sha256(verified.marketplace_key())?,
            policy_marketplace_key_sha256: Some(marketplace_key_sha256(
                verified.policy_marketplace_key(),
            )?),
            marketplace_id: snapshot.authority_id.clone(),
            marketplace_name: snapshot.marketplace_name.clone(),
            reviewed_by: entry.reviewed_by.clone(),
            review_summary: entry.review_summary.clone(),
            snapshot_version: snapshot.snapshot_version,
            policy_snapshot_version: Some(verified.policy_snapshot_version()),
            policy_version: policy.policy_version,
            lifecycle_status: lifecycle_name(policy.status).to_owned(),
            admission_revision: admission.admission_revision,
            trust_status: None,
            warning: (policy.status == CatalogStatus::Deprecated).then(|| policy.reason.clone()),
        };
        mount.validate()?;
        Ok(mount)
    }

    fn validate(&self) -> Result<()> {
        if self.format != MOUNT_FORMAT
            || exact_uuid(&self.server_id).is_err()
            || self.server_origin.len() > 512
            || canonical_origin(&self.server_origin).is_err()
            || !valid_identifier(&self.game_key)
            || !valid_identifier(&self.publisher_id)
            || self.rules_version == 0
            || self.cartridge_version == 0
            || !valid_text(&self.display_name, 128)
            || !valid_sha256(&self.archive_sha256)
            || !valid_sha256(&self.signed_identity_sha256)
            || !valid_sha256(&self.marketplace_key_sha256)
            || self
                .policy_marketplace_key_sha256
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
            || (self.policy_marketplace_key_sha256.is_none()
                != self.policy_snapshot_version.is_none())
            || !valid_identifier(&self.marketplace_id)
            || !valid_text(&self.marketplace_name, 128)
            || !valid_identifier(&self.reviewed_by)
            || !valid_text(&self.review_summary, 512)
            || self.snapshot_version == 0
            || self
                .policy_snapshot_version
                .is_some_and(|version| version == 0)
            || self.policy_version == 0
            || !matches!(
                self.lifecycle_status.as_str(),
                "active" | "deprecated" | "retired"
            )
            || self.admission_revision == 0
            || self.trust_status.as_ref().is_some_and(|status| {
                !matches!(
                    status.as_str(),
                    "trusted" | "retired" | "revoked" | "expired" | "unknown"
                )
            })
            || self
                .warning
                .as_ref()
                .is_some_and(|value| !valid_text(value, 512))
        {
            Err(CompanionError::Cache)
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDocument {
    format: String,
    mounts: Vec<MountRecord>,
}

impl ProfileDocument {
    fn empty() -> Self {
        Self {
            format: PROFILE_FORMAT.to_owned(),
            mounts: Vec::new(),
        }
    }

    fn validate(&self, server_id: Uuid) -> Result<()> {
        let expected_server_id = server_id.to_string();
        let expected_origin = self
            .mounts
            .first()
            .map(|mount| mount.server_origin.as_str());
        if self.format != PROFILE_FORMAT
            || self.mounts.len() > MAX_MOUNTS
            || !self.mounts.iter().all(|mount| {
                mount.server_id == expected_server_id
                    && Some(mount.server_origin.as_str()) == expected_origin
                    && mount.validate().is_ok()
            })
            || !self
                .mounts
                .windows(2)
                .all(|pair| mount_identity(&pair[0]) < mount_identity(&pair[1]))
        {
            Err(CompanionError::Cache)
        } else {
            Ok(())
        }
    }
}

fn mount_identity(mount: &MountRecord) -> (&str, &str, u64) {
    (
        mount.game_key.as_str(),
        mount.archive_sha256.as_str(),
        mount.admission_revision,
    )
}

fn mount_order(left: &MountRecord, right: &MountRecord) -> std::cmp::Ordering {
    mount_identity(left).cmp(&mount_identity(right))
}

fn canonical_origin(value: &str) -> Result<String> {
    let origin = selected_origin(value)?.origin().ascii_serialization();
    if origin == value {
        Ok(origin)
    } else {
        Err(CompanionError::InvalidInput)
    }
}

fn create_private_root(root: &Path) -> Result<()> {
    if !root.is_absolute() {
        return Err(CompanionError::Cache);
    }
    match fs::create_dir(root) {
        Ok(()) => fs::set_permissions(root, fs::Permissions::from_mode(0o700))
            .map_err(|_| CompanionError::Cache)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(CompanionError::Cache),
    }
    let metadata = fs::symlink_metadata(root).map_err(|_| CompanionError::Cache)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(CompanionError::Cache);
    }
    Ok(())
}

fn open_or_create_directory(parent: &OwnedFd, name: &str, owner: Uid) -> Result<OwnedFd> {
    match mkdirat(parent, name, Mode::from_bits_truncate(0o700)) {
        Ok(()) | Err(rustix::io::Errno::EXIST) => {}
        Err(_) => return Err(CompanionError::Cache),
    }
    let directory = openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|_| CompanionError::Cache)?;
    validate_directory(&directory, owner)?;
    Ok(directory)
}

fn validate_directory(fd: &OwnedFd, owner: Uid) -> Result<()> {
    let metadata = fstat(fd).map_err(|_| CompanionError::Cache)?;
    if FileType::from_raw_mode(metadata.st_mode) != FileType::Directory
        || metadata.st_uid != owner.as_raw()
        || metadata.st_mode & 0o777 != 0o700
    {
        Err(CompanionError::Cache)
    } else {
        Ok(())
    }
}

fn validate_regular_file(fd: &impl std::os::fd::AsFd, owner: Uid) -> Result<()> {
    let metadata = fstat(fd).map_err(|_| CompanionError::Cache)?;
    if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile
        || metadata.st_uid != owner.as_raw()
        || metadata.st_nlink != 1
        || metadata.st_mode & 0o077 != 0
    {
        Err(CompanionError::Cache)
    } else {
        Ok(())
    }
}

fn read_regular_file(parent: &OwnedFd, name: &str, maximum: u64) -> Result<Vec<u8>> {
    read_optional_regular_file(parent, name, maximum)?.ok_or(CompanionError::Cache)
}

fn read_optional_regular_file(
    parent: &OwnedFd,
    name: &str,
    maximum: u64,
) -> Result<Option<Vec<u8>>> {
    let fd = match openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(_) => return Err(CompanionError::Cache),
    };
    validate_regular_file(&fd, geteuid())?;
    let metadata = fstat(&fd).map_err(|_| CompanionError::Cache)?;
    if metadata.st_size == 0 || metadata.st_size as u64 > maximum {
        return Err(CompanionError::Cache);
    }
    let mut bytes = Vec::with_capacity(metadata.st_size as usize);
    File::from(fd)
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CompanionError::Cache)?;
    if bytes.len() as u64 > maximum {
        return Err(CompanionError::Cache);
    }
    Ok(Some(bytes))
}

fn profile_name(server_id: Uuid) -> String {
    format!("{server_id}.json")
}

fn policy_name(archive_sha256: &str, key_sha256: &str) -> String {
    format!("{archive_sha256}.{key_sha256}.signed.json")
}

fn temporary_name() -> String {
    let mut random = [0_u8; 16];
    OsRng.fill_bytes(&mut random);
    let suffix = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!(".profile-{suffix}.tmp")
}

fn exact_uuid(value: &str) -> Result<Uuid> {
    let parsed = Uuid::try_parse(value).map_err(|_| CompanionError::InvalidInput)?;
    if parsed.is_nil() || parsed.to_string() != value {
        Err(CompanionError::InvalidInput)
    } else {
        Ok(parsed)
    }
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn marketplace_key_sha256(key: &CatalogPublicKey) -> Result<String> {
    let bytes = serde_json::to_vec(key).map_err(|_| CompanionError::Rejected)?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= maximum
        && !value.chars().any(char::is_control)
}

fn lifecycle_name(status: omarchygs_game_cartridge::CatalogStatus) -> &'static str {
    use omarchygs_game_cartridge::CatalogStatus;
    match status {
        CatalogStatus::Active => "active",
        CatalogStatus::Deprecated => "deprecated",
        CatalogStatus::Suspended => "suspended",
        CatalogStatus::Revoked => "revoked",
        CatalogStatus::Retired => "retired",
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::symlink};

    use omarchygs_game_cartridge::generate_catalog_keypair;

    use super::*;

    #[test]
    fn profile_mounts_are_private_exact_and_server_isolated() {
        let temp = tempfile::tempdir().expect("temp should create");
        let root = temp.path().join("cache");
        let cache = ClientCartridgeCache::open(&root).expect("cache should open");
        let (_, trusted_marketplace_key) =
            generate_catalog_keypair("marketplace-primary-v1", "marketplace")
                .expect("key should generate");
        let trusted_key_sha256 =
            marketplace_key_sha256(&trusted_marketplace_key).expect("key should hash");
        let first_server = Uuid::from_u128(1);
        let second_server = Uuid::from_u128(2);
        assert!(
            cache
                .mounts(first_server, &trusted_marketplace_key)
                .expect("mounts should read")
                .is_empty()
        );

        let first = mount(first_server, "game-one", 'a', &trusted_key_sha256);
        let mut second = mount(first_server, "game-one", 'b', &trusted_key_sha256);
        second.admission_revision = 2;
        let wrong_server = mount(second_server, "game-one", 'a', &trusted_key_sha256);
        assert!(
            cache
                .write_profile(
                    first_server,
                    &ProfileDocument {
                        format: PROFILE_FORMAT.to_owned(),
                        mounts: vec![wrong_server],
                    },
                    &trusted_key_sha256,
                )
                .is_err(),
            "a UUID-named profile must contain only mounts for that exact server"
        );
        let mut wrong_origin = second.clone();
        wrong_origin.server_origin = "https://other.example.test".to_owned();
        assert!(
            cache
                .write_profile(
                    first_server,
                    &ProfileDocument {
                        format: PROFILE_FORMAT.to_owned(),
                        mounts: vec![first.clone(), wrong_origin],
                    },
                    &trusted_key_sha256,
                )
                .is_err(),
            "one server UUID profile cannot mix canonical origins"
        );
        cache
            .write_profile(
                first_server,
                &ProfileDocument {
                    format: PROFILE_FORMAT.to_owned(),
                    mounts: vec![first.clone(), second.clone()],
                },
                &trusted_key_sha256,
            )
            .expect("profile should write");
        assert_eq!(
            fs::metadata(root.join("profiles/00000000-0000-0000-0000-000000000001.json"))
                .expect("profile metadata should read")
                .permissions()
                .mode()
                & 0o777,
            0o400
        );
        assert_eq!(
            cache
                .mounts(first_server, &trusted_marketplace_key)
                .expect("mounts should read"),
            vec![first.clone(), second]
        );
        assert!(matches!(
            cache.resolve_mounted(
                "https://other.example.test",
                first_server,
                &first.game_key,
                &first.archive_sha256,
                first.admission_revision,
                &trusted_marketplace_key,
            ),
            Err(CompanionError::MountMissing)
        ));
        assert!(
            cache
                .mounts(second_server, &trusted_marketplace_key)
                .expect("mounts should read")
                .is_empty()
        );
        assert!(
            cache
                .remove(
                    first_server,
                    &first.game_key,
                    &first.archive_sha256,
                    Some(first.admission_revision),
                    &trusted_marketplace_key,
                )
                .expect("exact remove should work")
        );
        assert_eq!(
            cache
                .mounts(first_server, &trusted_marketplace_key)
                .expect("mounts should read")
                .len(),
            1
        );
        assert!(
            !cache
                .remove(
                    first_server,
                    &first.game_key,
                    &first.archive_sha256,
                    Some(first.admission_revision),
                    &trusted_marketplace_key,
                )
                .expect("missing remove should be idempotent")
        );

        let (_, substituted_marketplace_key) =
            generate_catalog_keypair("marketplace-primary-v1", "marketplace")
                .expect("substituted key should generate");
        assert!(
            cache
                .mounts(first_server, &substituted_marketplace_key)
                .is_err(),
            "a cached marketplace-vetted mount must stay bound to its original trust key"
        );
    }

    #[test]
    fn cache_rejects_symlinked_or_public_roots() {
        let temp = tempfile::tempdir().expect("temp should create");
        let actual = temp.path().join("actual");
        fs::create_dir(&actual).expect("actual should create");
        fs::set_permissions(&actual, fs::Permissions::from_mode(0o700)).expect("mode should set");
        let linked = temp.path().join("linked");
        symlink(&actual, &linked).expect("symlink should create");
        assert!(ClientCartridgeCache::open(&linked).is_err());

        let public = temp.path().join("public");
        fs::create_dir(&public).expect("public should create");
        fs::set_permissions(&public, fs::Permissions::from_mode(0o755)).expect("mode should set");
        assert!(ClientCartridgeCache::open(&public).is_err());
    }

    #[test]
    fn each_mount_operation_releases_the_cross_process_lock() {
        let temp = tempfile::tempdir().expect("temp should create");
        let root = temp.path().join("cache");
        let first = ClientCartridgeCache::open(&root).expect("first cache should open");
        let second = ClientCartridgeCache::open(&root).expect("second cache should open");
        let (_, trusted_marketplace_key) =
            generate_catalog_keypair("marketplace-primary-v1", "marketplace")
                .expect("key should generate");

        first
            .mounts(Uuid::from_u128(1), &trusted_marketplace_key)
            .expect("mount lookup should finish");
        flock(&second.lock_file, FlockOperation::NonBlockingLockExclusive)
            .expect("mount lookup must release the file lock");
        flock(&second.lock_file, FlockOperation::Unlock).expect("test lock should release");
    }

    fn mount(
        server_id: Uuid,
        game_key: &str,
        digest_character: char,
        marketplace_key_sha256: &str,
    ) -> MountRecord {
        MountRecord {
            format: MOUNT_FORMAT.to_owned(),
            server_id: server_id.to_string(),
            server_origin: "https://games.example.test".to_owned(),
            game_key: game_key.to_owned(),
            publisher_id: "publisher".to_owned(),
            rules_version: 1,
            cartridge_version: 1,
            display_name: "Test Game".to_owned(),
            archive_sha256: digest_character.to_string().repeat(64),
            signed_identity_sha256: "c".repeat(64),
            marketplace_key_sha256: marketplace_key_sha256.to_owned(),
            policy_marketplace_key_sha256: None,
            marketplace_id: "marketplace".to_owned(),
            marketplace_name: "Test Marketplace".to_owned(),
            reviewed_by: "review-team".to_owned(),
            review_summary: "Reviewed exact release.".to_owned(),
            snapshot_version: 1,
            policy_snapshot_version: None,
            policy_version: 1,
            lifecycle_status: "active".to_owned(),
            admission_revision: 1,
            trust_status: None,
            warning: None,
        }
    }
}
