//! Descriptor-bound per-user marketplace trust enrollment and snapshots.

use std::{
    fs::File,
    io::{Read as _, Write as _},
    os::fd::OwnedFd,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use omarchygs_game_cartridge::CatalogPublicKey;
use omarchygs_marketplace_trust::{
    ChannelEgressError, ChannelOrigin, GuardedChannelClient, MAX_TRUST_CHANNEL_BYTES,
    MarketplaceKeyStatus, MarketplaceTrust, PublicChannelBootstrap, catalog_key_sha256,
    verify_marketplace_trust_bytes, verify_marketplace_trust_bytes_at_rest,
    verify_trust_transition,
};
use rand_core::{OsRng, RngCore as _};
use rustix::{
    fs::{
        AtFlags, FileType, FlockOperation, Mode, OFlags, RenameFlags, fchmod, flock, fstat, fsync,
        mkdirat, open, openat, renameat_with, unlinkat,
    },
    process::geteuid,
};
use serde::Serialize;

use crate::{CompanionError, Result};

const TRUST_DIRECTORY: &str = "marketplace-trust";
const TRUST_FILE: &str = "channel.signed.json";
const TRUST_LOCK: &str = ".channel.lock";

#[derive(Clone)]
pub enum ClientMarketplaceTrust {
    None,
    Manual(Arc<CatalogPublicKey>),
    Channel(Arc<ClientTrustStore>),
}

#[derive(Clone)]
pub enum ClientTrustSnapshot {
    Manual(Arc<CatalogPublicKey>),
    Channel(Arc<MarketplaceTrust>),
}

impl ClientTrustSnapshot {
    pub fn authorize_key(&self, key: &CatalogPublicKey, snapshot_version: u64) -> Result<()> {
        if snapshot_version == 0 {
            return Err(CompanionError::MarketplaceUntrusted);
        }
        match self {
            Self::Manual(expected) if expected.as_ref() == key => Ok(()),
            Self::Manual(_) => Err(CompanionError::MarketplaceUntrusted),
            Self::Channel(trust) => {
                trust
                    .validate_now(now_unix()?)
                    .map_err(|_| CompanionError::MarketplaceUntrusted)?;
                trust
                    .authorize_key(key, snapshot_version)
                    .map(|_| ())
                    .map_err(|_| CompanionError::MarketplaceUntrusted)
            }
        }
    }

    pub fn key_by_fingerprint(
        &self,
        fingerprint: &str,
        snapshot_version: u64,
    ) -> Result<CatalogPublicKey> {
        match self {
            Self::Manual(key)
                if catalog_key_sha256(key).map_err(|_| CompanionError::MarketplaceUntrusted)?
                    == fingerprint =>
            {
                self.authorize_key(key, snapshot_version)?;
                Ok((**key).clone())
            }
            Self::Manual(_) => Err(CompanionError::MarketplaceUntrusted),
            Self::Channel(trust) => {
                trust
                    .validate_now(now_unix()?)
                    .map_err(|_| CompanionError::MarketplaceUntrusted)?;
                trust
                    .key_by_fingerprint(fingerprint, snapshot_version)
                    .cloned()
                    .map_err(|_| CompanionError::MarketplaceUntrusted)
            }
        }
    }

    pub fn authorize_current_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<()> {
        if snapshot_version == 0 {
            return Err(CompanionError::MarketplaceUntrusted);
        }
        match self {
            Self::Manual(expected) if expected.as_ref() == key => Ok(()),
            Self::Manual(_) => Err(CompanionError::MarketplaceUntrusted),
            Self::Channel(trust) => {
                trust
                    .validate_now(now_unix()?)
                    .map_err(|_| CompanionError::MarketplaceUntrusted)?;
                trust
                    .authorize_new_snapshot(key, snapshot_version)
                    .map_err(|_| CompanionError::MarketplaceUntrusted)
            }
        }
    }

    pub fn packages(&self) -> &[omarchygs_marketplace_trust::ClientPackageArtifact] {
        match self {
            Self::Manual(_) => &[],
            Self::Channel(trust) => &trust.payload().packages,
        }
    }

    pub fn key_status(&self, fingerprint: &str, snapshot_version: u64) -> &'static str {
        match self {
            Self::Manual(key) => {
                if snapshot_version > 0
                    && catalog_key_sha256(key).ok().as_deref() == Some(fingerprint)
                {
                    "active"
                } else {
                    "unknown"
                }
            }
            Self::Channel(trust) => {
                let Ok(now) = now_unix() else {
                    return "expired";
                };
                if trust.validate_now(now).is_err() {
                    return "expired";
                }
                let Some(record) = trust
                    .payload()
                    .keys
                    .iter()
                    .find(|record| record.key_sha256 == fingerprint)
                else {
                    return "unknown";
                };
                if snapshot_version < record.first_snapshot_version
                    || record
                        .last_snapshot_version
                        .is_some_and(|last| snapshot_version > last)
                {
                    return "unknown";
                }
                match record.status {
                    MarketplaceKeyStatus::Active => "active",
                    MarketplaceKeyStatus::Retired => "retired",
                    MarketplaceKeyStatus::Revoked => "revoked",
                }
            }
        }
    }
}

impl ClientMarketplaceTrust {
    pub fn snapshot(&self) -> Result<ClientTrustSnapshot> {
        match self {
            Self::None => Err(CompanionError::MarketplaceUntrusted),
            Self::Manual(key) => Ok(ClientTrustSnapshot::Manual(key.clone())),
            Self::Channel(store) => store.snapshot(),
        }
    }

    pub fn status(&self) -> Result<TrustStatus> {
        match self {
            Self::None => Ok(TrustStatus::none()),
            Self::Manual(key) => TrustStatus::manual(key),
            Self::Channel(store) => store.status(),
        }
    }

    pub fn inventory_snapshot(&self) -> Result<Option<ClientTrustSnapshot>> {
        match self {
            Self::None => Ok(None),
            Self::Manual(key) => Ok(Some(ClientTrustSnapshot::Manual(key.clone()))),
            Self::Channel(store) => store.inventory_snapshot(),
        }
    }

    pub async fn synchronize(&self) -> Result<TrustStatus> {
        match self {
            Self::Channel(store) => {
                store.synchronize().await?;
                store.status()
            }
            _ => Err(CompanionError::InvalidInput),
        }
    }

    pub fn channel_store(&self) -> Option<&Arc<ClientTrustStore>> {
        match self {
            Self::Channel(store) => Some(store),
            _ => None,
        }
    }
}

pub struct ClientTrustStore {
    root_path: PathBuf,
    bootstrap: PublicChannelBootstrap,
    directory: OwnedFd,
    lock_file: OwnedFd,
    process_lock: Mutex<()>,
    current: RwLock<Option<Arc<MarketplaceTrust>>>,
}

impl ClientTrustStore {
    pub fn open(root: &Path, bootstrap: PublicChannelBootstrap) -> Result<Self> {
        bootstrap
            .validate()
            .map_err(|_| CompanionError::MarketplaceUntrusted)?;
        let root_fd = open(
            root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_private_directory(&root_fd)?;
        match mkdirat(&root_fd, TRUST_DIRECTORY, Mode::from_bits_truncate(0o700)) {
            Ok(()) | Err(rustix::io::Errno::EXIST) => {}
            Err(_) => return Err(CompanionError::Cache),
        }
        let directory = openat(
            &root_fd,
            TRUST_DIRECTORY,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_private_directory(&directory)?;
        let lock_file = openat(
            &directory,
            TRUST_LOCK,
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_private_file(&lock_file)?;
        fsync(&directory).map_err(|_| CompanionError::Cache)?;
        fsync(&root_fd).map_err(|_| CompanionError::Cache)?;
        let current = read_current(&directory, &bootstrap)?.map(Arc::new);
        Ok(Self {
            root_path: root.to_path_buf(),
            bootstrap,
            directory,
            lock_file,
            process_lock: Mutex::new(()),
            current: RwLock::new(current),
        })
    }

    pub fn bootstrap(&self) -> &PublicChannelBootstrap {
        &self.bootstrap
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn snapshot(&self) -> Result<ClientTrustSnapshot> {
        let trust = self
            .reconcile_current()?
            .ok_or(CompanionError::MarketplaceUntrusted)?;
        trust
            .validate_now(now_unix()?)
            .map_err(|_| CompanionError::MarketplaceUntrusted)?;
        Ok(ClientTrustSnapshot::Channel(trust))
    }

    fn inventory_snapshot(&self) -> Result<Option<ClientTrustSnapshot>> {
        Ok(self.reconcile_current()?.map(ClientTrustSnapshot::Channel))
    }

    pub fn status(&self) -> Result<TrustStatus> {
        let current = self.reconcile_current()?;
        TrustStatus::channel(&self.bootstrap, current.as_deref(), now_unix()?)
    }

    fn reconcile_current(&self) -> Result<Option<Arc<MarketplaceTrust>>> {
        let _process = self
            .process_lock
            .lock()
            .map_err(|_| CompanionError::Cache)?;
        flock(&self.lock_file, FlockOperation::LockExclusive).map_err(|_| CompanionError::Cache)?;
        let result = (|| {
            let persisted = read_current(&self.directory, &self.bootstrap)?;
            let cached = self
                .current
                .read()
                .map_err(|_| CompanionError::Cache)?
                .clone();
            let reconciled = match (cached.as_ref(), persisted) {
                (Some(previous), Some(candidate))
                    if candidate.payload().bundle_version == previous.payload().bundle_version =>
                {
                    if candidate.signed_bytes() != previous.signed_bytes() {
                        return Err(CompanionError::MarketplaceUntrusted);
                    }
                    return Ok(cached);
                }
                (Some(previous), Some(candidate)) => {
                    verify_trust_transition(previous, &candidate)
                        .map_err(|_| CompanionError::MarketplaceUntrusted)?;
                    Some(Arc::new(candidate))
                }
                (None, Some(candidate)) => Some(Arc::new(candidate)),
                (Some(_), None) => return Err(CompanionError::MarketplaceUntrusted),
                (None, None) => None,
            };
            *self.current.write().map_err(|_| CompanionError::Cache)? = reconciled.clone();
            Ok(reconciled)
        })();
        if flock(&self.lock_file, FlockOperation::Unlock).is_err() {
            return Err(CompanionError::Cache);
        }
        result
    }

    pub async fn synchronize(&self) -> Result<()> {
        let origin = ChannelOrigin::parse(&self.bootstrap.channel_origin)
            .map_err(|_| CompanionError::MarketplaceUntrusted)?;
        let client = GuardedChannelClient::production(origin)
            .await
            .map_err(map_channel_error)?;
        let bytes = client
            .get_bytes(
                &self.bootstrap.manifest_path,
                MAX_TRUST_CHANNEL_BYTES,
                "application/json",
            )
            .await
            .map_err(map_channel_error)?;
        let candidate = verify_marketplace_trust_bytes(
            &bytes,
            &self.bootstrap.root,
            &self.bootstrap.channel_id,
            &self.bootstrap.channel_origin,
            now_unix()?,
        )
        .map_err(|_| CompanionError::Rejected)?;
        self.publish(candidate)
    }

    fn publish(&self, candidate: MarketplaceTrust) -> Result<()> {
        self.bootstrap
            .authorize_trust(&candidate)
            .map_err(|_| CompanionError::MarketplaceUntrusted)?;
        let _process = self
            .process_lock
            .lock()
            .map_err(|_| CompanionError::Cache)?;
        flock(&self.lock_file, FlockOperation::LockExclusive).map_err(|_| CompanionError::Cache)?;
        let result = (|| {
            // A newly installed package may raise its enrollment floor above
            // the currently usable bundle. The old authenticated bytes still
            // remain continuity evidence and must constrain the replacement.
            let persisted = read_persisted_current(&self.directory, &self.bootstrap)?;
            if let Some(previous) = persisted.as_ref() {
                if candidate.payload().bundle_version == previous.payload().bundle_version {
                    if candidate.signed_bytes() != previous.signed_bytes() {
                        return Err(CompanionError::MarketplaceUntrusted);
                    }
                    *self.current.write().map_err(|_| CompanionError::Cache)? =
                        Some(Arc::new(candidate));
                    return Ok(());
                }
                verify_trust_transition(previous, &candidate)
                    .map_err(|_| CompanionError::MarketplaceUntrusted)?;
            }
            write_current(&self.directory, candidate.signed_bytes())?;
            *self.current.write().map_err(|_| CompanionError::Cache)? = Some(Arc::new(candidate));
            Ok(())
        })();
        if flock(&self.lock_file, FlockOperation::Unlock).is_err() {
            return Err(CompanionError::Cache);
        }
        result
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustStatus {
    pub format: &'static str,
    pub mode: &'static str,
    pub state: &'static str,
    pub enrolled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_unix: Option<u64>,
    pub keys: Vec<TrustKeyStatus>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustKeyStatus {
    pub key_sha256: String,
    pub status: &'static str,
    pub first_snapshot_version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot_version: Option<u64>,
}

impl TrustStatus {
    fn none() -> Self {
        Self {
            format: "omarchygs.client-trust-status/v1",
            mode: "none",
            state: "unavailable",
            enrolled: false,
            channel_id: None,
            channel_name: None,
            channel_origin: None,
            bundle_version: None,
            expires_at_unix: None,
            keys: Vec::new(),
        }
    }

    fn manual(key: &CatalogPublicKey) -> Result<Self> {
        Ok(Self {
            format: "omarchygs.client-trust-status/v1",
            mode: "manual",
            state: "current",
            enrolled: true,
            channel_id: None,
            channel_name: None,
            channel_origin: None,
            bundle_version: None,
            expires_at_unix: None,
            keys: vec![TrustKeyStatus {
                key_sha256: catalog_key_sha256(key)
                    .map_err(|_| CompanionError::MarketplaceUntrusted)?,
                status: "active",
                first_snapshot_version: 1,
                last_snapshot_version: None,
            }],
        })
    }

    fn channel(
        bootstrap: &PublicChannelBootstrap,
        trust: Option<&MarketplaceTrust>,
        now: u64,
    ) -> Result<Self> {
        let Some(trust) = trust else {
            return Ok(Self {
                format: "omarchygs.client-trust-status/v1",
                mode: "channel",
                state: "unenrolled",
                enrolled: false,
                channel_id: Some(bootstrap.channel_id.clone()),
                channel_name: None,
                channel_origin: Some(bootstrap.channel_origin.clone()),
                bundle_version: None,
                expires_at_unix: None,
                keys: Vec::new(),
            });
        };
        let state = if trust.validate_now(now).is_ok() {
            "current"
        } else {
            "expired"
        };
        Ok(Self {
            format: "omarchygs.client-trust-status/v1",
            mode: "channel",
            state,
            enrolled: true,
            channel_id: Some(trust.payload().channel_id.clone()),
            channel_name: Some(trust.payload().channel_name.clone()),
            channel_origin: Some(trust.payload().channel_origin.clone()),
            bundle_version: Some(trust.payload().bundle_version),
            expires_at_unix: Some(trust.payload().expires_at_unix),
            keys: trust
                .payload()
                .keys
                .iter()
                .map(|key| TrustKeyStatus {
                    key_sha256: key.key_sha256.clone(),
                    status: match key.status {
                        MarketplaceKeyStatus::Active => "active",
                        MarketplaceKeyStatus::Retired => "retired",
                        MarketplaceKeyStatus::Revoked => "revoked",
                    },
                    first_snapshot_version: key.first_snapshot_version,
                    last_snapshot_version: key.last_snapshot_version,
                })
                .collect(),
        })
    }
}

fn read_current(
    directory: &OwnedFd,
    bootstrap: &PublicChannelBootstrap,
) -> Result<Option<MarketplaceTrust>> {
    let Some(trust) = read_persisted_current(directory, bootstrap)? else {
        return Ok(None);
    };
    if bootstrap.authorize_trust(&trust).is_err() {
        return Ok(None);
    }
    Ok(Some(trust))
}

fn read_persisted_current(
    directory: &OwnedFd,
    bootstrap: &PublicChannelBootstrap,
) -> Result<Option<MarketplaceTrust>> {
    let file = match openat(
        directory,
        TRUST_FILE,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(file) => file,
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(_) => return Err(CompanionError::Cache),
    };
    validate_private_file(&file)?;
    let metadata = fstat(&file).map_err(|_| CompanionError::Cache)?;
    if metadata.st_size == 0 || metadata.st_size as usize > MAX_TRUST_CHANNEL_BYTES {
        return Err(CompanionError::Cache);
    }
    let mut bytes = Vec::with_capacity(metadata.st_size as usize);
    File::from(file)
        .take(MAX_TRUST_CHANNEL_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CompanionError::Cache)?;
    if bytes.len() > MAX_TRUST_CHANNEL_BYTES {
        return Err(CompanionError::Cache);
    }
    let trust = verify_marketplace_trust_bytes_at_rest(
        &bytes,
        &bootstrap.root,
        &bootstrap.channel_id,
        &bootstrap.channel_origin,
    )
    .map_err(|_| CompanionError::MarketplaceUntrusted)?;
    Ok(Some(trust))
}

fn write_current(directory: &OwnedFd, bytes: &[u8]) -> Result<()> {
    let mut random = [0_u8; 16];
    OsRng.fill_bytes(&mut random);
    let suffix = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let temporary = format!(".channel-{suffix}.tmp");
    let file = openat(
        directory,
        temporary.as_str(),
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::from_bits_truncate(0o600),
    )
    .map_err(|_| CompanionError::Cache)?;
    let result = (|| {
        validate_private_file(&file)?;
        let mut file = File::from(file);
        file.write_all(bytes).map_err(|_| CompanionError::Cache)?;
        file.sync_all().map_err(|_| CompanionError::Cache)?;
        fchmod(&file, Mode::from_bits_truncate(0o600)).map_err(|_| CompanionError::Cache)?;
        drop(file);
        match renameat_with(
            directory,
            temporary.as_str(),
            directory,
            TRUST_FILE,
            RenameFlags::empty(),
        ) {
            Ok(()) => fsync(directory).map_err(|_| CompanionError::Cache),
            Err(_) => Err(CompanionError::Cache),
        }
    })();
    if result.is_err() {
        let _ = unlinkat(directory, temporary.as_str(), AtFlags::empty());
    }
    result
}

fn validate_private_directory(fd: &OwnedFd) -> Result<()> {
    let metadata = fstat(fd).map_err(|_| CompanionError::Cache)?;
    if FileType::from_raw_mode(metadata.st_mode) != FileType::Directory
        || metadata.st_uid != geteuid().as_raw()
        || metadata.st_mode & 0o777 != 0o700
    {
        Err(CompanionError::Cache)
    } else {
        Ok(())
    }
}

fn validate_private_file(fd: &OwnedFd) -> Result<()> {
    let metadata = fstat(fd).map_err(|_| CompanionError::Cache)?;
    if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile
        || metadata.st_uid != geteuid().as_raw()
        || metadata.st_nlink != 1
        || metadata.st_mode & 0o077 != 0
    {
        Err(CompanionError::Cache)
    } else {
        Ok(())
    }
}

fn now_unix() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| CompanionError::MarketplaceUntrusted)
}

fn map_channel_error(error: ChannelEgressError) -> CompanionError {
    match error {
        ChannelEgressError::Unavailable => CompanionError::Unavailable,
        ChannelEgressError::InvalidInput
        | ChannelEgressError::Denied
        | ChannelEgressError::Rejected => CompanionError::Rejected,
        ChannelEgressError::Internal => CompanionError::Cache,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt as _};

    use omarchygs_game_cartridge::generate_catalog_keypair;
    use omarchygs_marketplace_trust::{
        MarketplaceTrustKey, MarketplaceTrustPayload, generate_trust_root_keypair,
        sign_marketplace_trust, signed_trust_bytes,
    };

    use super::*;

    #[test]
    fn persisted_rotation_and_terminal_revocation_survive_restart() {
        let now = now_unix().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let root_path = temp.path().join("cache");
        fs::create_dir(&root_path).unwrap();
        fs::set_permissions(&root_path, fs::Permissions::from_mode(0o700)).unwrap();
        let (root_private, root_public) =
            generate_trust_root_keypair("root-1", "official").unwrap();
        let (_, first) = generate_catalog_keypair("catalog-1", "official-marketplace").unwrap();
        let (_, second) = generate_catalog_keypair("catalog-2", "official-marketplace").unwrap();
        let bootstrap = PublicChannelBootstrap {
            format: "omarchygs.public-channel-bootstrap/v1".to_owned(),
            channel_id: "official".to_owned(),
            channel_origin: "https://packages.example.test/v1/".to_owned(),
            manifest_path: "trust.signed.json".to_owned(),
            minimum_bundle_version: 1,
            minimum_current_snapshot_version: 1,
            platform: "arch-linux".to_owned(),
            architecture: "x86_64".to_owned(),
            installed_package_version: "0.1.0-1".to_owned(),
            root: root_public.clone(),
        };
        let initial_payload = payload(now, first.clone());
        let initial = verified(&initial_payload, &root_private, &root_public, now);
        let store = ClientTrustStore::open(&root_path, bootstrap.clone()).unwrap();
        let peer = ClientTrustStore::open(&root_path, bootstrap.clone()).unwrap();
        store.publish(initial.clone()).unwrap();
        assert_eq!(store.status().unwrap().state, "current");
        assert_eq!(peer.status().unwrap().bundle_version, Some(1));

        let mut rotated_payload = initial_payload.clone();
        rotated_payload.bundle_version = 2;
        rotated_payload.current_snapshot_version = 6;
        rotated_payload.keys[0].status = MarketplaceKeyStatus::Retired;
        rotated_payload.keys[0].last_snapshot_version = Some(5);
        rotated_payload.keys.push(MarketplaceTrustKey {
            key_sha256: catalog_key_sha256(&second).unwrap(),
            key: second.clone(),
            status: MarketplaceKeyStatus::Active,
            first_snapshot_version: 6,
            last_snapshot_version: None,
        });
        let rotated = verified(&rotated_payload, &root_private, &root_public, now);
        store.publish(rotated).unwrap();
        let peer_rotated = peer.snapshot().unwrap();
        assert!(peer_rotated.authorize_key(&first, 5).is_ok());
        assert!(
            peer_rotated.authorize_current_key(&first, 5).is_err(),
            "retired evidence keys must not authenticate current lifecycle policy"
        );
        assert!(peer_rotated.authorize_current_key(&second, 5).is_err());
        peer_rotated
            .authorize_current_key(&second, 6)
            .expect("the active key authenticates only the declared current snapshot");
        assert!(peer_rotated.authorize_current_key(&second, 7).is_err());

        let mut revoked_payload = rotated_payload;
        revoked_payload.bundle_version = 3;
        revoked_payload.keys[0].status = MarketplaceKeyStatus::Revoked;
        let revoked = verified(&revoked_payload, &root_private, &root_public, now);
        store.publish(revoked.clone()).unwrap();
        let peer_snapshot = peer.snapshot().unwrap();
        assert_eq!(
            peer_snapshot.key_status(&catalog_key_sha256(&first).unwrap(), 5),
            "revoked"
        );
        assert!(
            peer_snapshot.authorize_key(&first, 5).is_err(),
            "an already-open peer must reject a key revoked by another process"
        );
        assert!(
            store.publish(initial.clone()).is_err(),
            "rollback must preserve revocation"
        );
        drop(store);

        let reopened = ClientTrustStore::open(&root_path, bootstrap).unwrap();
        let snapshot = reopened.snapshot().unwrap();
        assert_eq!(
            snapshot.key_status(&catalog_key_sha256(&first).unwrap(), 5),
            "revoked"
        );

        let floor_root = temp.path().join("fresh-cache");
        fs::create_dir(&floor_root).unwrap();
        fs::set_permissions(&floor_root, fs::Permissions::from_mode(0o700)).unwrap();
        let mut floor_bootstrap = reopened.bootstrap().clone();
        floor_bootstrap.minimum_bundle_version = 3;
        floor_bootstrap.minimum_current_snapshot_version = 6;
        let fresh = ClientTrustStore::open(&floor_root, floor_bootstrap).unwrap();
        assert!(
            fresh.publish(initial).is_err(),
            "fresh enrollment must reject a bundle below the packaged floor"
        );
        fresh
            .publish(revoked.clone())
            .expect("the exact packaged trust floor should enroll");

        let continuity_root = temp.path().join("continuity-cache");
        fs::create_dir(&continuity_root).unwrap();
        fs::set_permissions(&continuity_root, fs::Permissions::from_mode(0o700)).unwrap();
        let continuity =
            ClientTrustStore::open(&continuity_root, reopened.bootstrap().clone()).unwrap();
        continuity.publish(revoked).unwrap();
        drop(continuity);

        let mut advanced_floor = reopened.bootstrap().clone();
        advanced_floor.minimum_bundle_version = 4;
        advanced_floor.minimum_current_snapshot_version = 6;
        let advanced = ClientTrustStore::open(&continuity_root, advanced_floor).unwrap();
        assert_eq!(advanced.status().unwrap().state, "unenrolled");
        let mut revived_payload = revoked_payload.clone();
        revived_payload.bundle_version = 4;
        revived_payload.keys[0].status = MarketplaceKeyStatus::Retired;
        let revived = verified(&revived_payload, &root_private, &root_public, now);
        assert!(
            advanced.publish(revived).is_err(),
            "a floor advance must retain terminal revocation history"
        );
        let mut continued_payload = revoked_payload;
        continued_payload.bundle_version = 4;
        let continued = verified(&continued_payload, &root_private, &root_public, now);
        advanced
            .publish(continued)
            .expect("a continuity-preserving floor advance should enroll");
    }

    fn payload(now: u64, key: CatalogPublicKey) -> MarketplaceTrustPayload {
        MarketplaceTrustPayload {
            format: "omarchygs.marketplace-trust-channel/v2".to_owned(),
            channel_id: "official".to_owned(),
            channel_name: "Official OmarchyGS".to_owned(),
            channel_origin: "https://packages.example.test/v1/".to_owned(),
            marketplace_origin: "https://market.example.test/v1/".to_owned(),
            marketplace_authority_id: "official-marketplace".to_owned(),
            bundle_version: 1,
            current_snapshot_version: 1,
            not_before_unix: now - 10,
            expires_at_unix: now + 3600,
            keys: vec![MarketplaceTrustKey {
                key_sha256: catalog_key_sha256(&key).unwrap(),
                key,
                status: MarketplaceKeyStatus::Active,
                first_snapshot_version: 1,
                last_snapshot_version: None,
            }],
            packages: Vec::new(),
        }
    }

    fn verified(
        payload: &MarketplaceTrustPayload,
        private: &omarchygs_marketplace_trust::TrustRootPrivateKey,
        public: &omarchygs_marketplace_trust::TrustRootPublicKey,
        now: u64,
    ) -> MarketplaceTrust {
        let signed = sign_marketplace_trust(payload, private).unwrap();
        verify_marketplace_trust_bytes(
            &signed_trust_bytes(&signed).unwrap(),
            public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .unwrap()
    }
}
