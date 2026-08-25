use serde::{Deserialize, Serialize};

use crate::{
    ActiveSessionDecision, CatalogPolicy, CatalogPublicKey, HostProfile, LifecycleDecision,
    LifecycleUse, NewLaunchDecision, PublisherPublicKey, SdkIdentity, VerifiedCartridge,
    VerifiedRelease,
    archive::sha256_hex,
    ensure_allowed,
    error::{CartridgeError, Result},
    keys::valid_identifier,
    lifecycle::{lifecycle_decision, verify_catalog_policy_bytes, verify_catalog_policy_signature},
    release::verify_release_components,
    validate::canonical_json,
};

const MAX_RELEASE_RECORD_BYTES: u64 = 512 * 1024;
const MAX_POLICY_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecureActivationRecord {
    pub format_version: u32,
    pub game_key: String,
    pub publisher_id: String,
    pub cartridge_version: u32,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub release_attestation_sha256: String,
    pub conformance_sha256: String,
    pub sdk: SdkIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecureImportReport {
    pub report_format: String,
    pub ok: bool,
    pub installed: bool,
    pub activation: SecureActivationRecord,
    pub decision: LifecycleDecision,
    pub descriptor_relative: bool,
    pub authoritative_policy_verified: bool,
    pub provider_contacted: bool,
    pub database_required: bool,
    pub platform_credentials_read: bool,
}

#[derive(Debug, Clone)]
pub struct SecureResolution {
    cartridge: VerifiedCartridge,
    activation: SecureActivationRecord,
    policy: CatalogPolicy,
    decision: LifecycleDecision,
}

impl SecureResolution {
    pub fn cartridge(&self) -> &VerifiedCartridge {
        &self.cartridge
    }

    pub fn activation(&self) -> &SecureActivationRecord {
        &self.activation
    }

    pub fn policy(&self) -> &CatalogPolicy {
        &self.policy
    }

    pub fn decision(&self) -> &LifecycleDecision {
        &self.decision
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use std::{
        fs::File,
        io::{Read, Write},
        os::fd::OwnedFd,
        path::Path,
    };

    use rand_core::{OsRng, RngCore};
    use rustix::{
        fs::{
            AtFlags, FileType, FlockOperation, Mode, OFlags, RenameFlags, fchmod, flock, fstat,
            fsync, mkdirat, open, openat, renameat, renameat_with, unlinkat,
        },
        io::Errno,
        process::{Uid, geteuid},
    };

    use super::*;

    pub struct SecureCartridgeStore {
        owner: Uid,
        _root: OwnedFd,
        blobs: OwnedFd,
        active: OwnedFd,
        releases: OwnedFd,
        conformance: OwnedFd,
        policies: OwnedFd,
    }

    impl SecureCartridgeStore {
        pub fn open_existing(root: &Path) -> Result<Self> {
            let owner = geteuid();
            let root = open(
                root,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(io_error)?;
            validate_secure_directory(&root, owner)?;
            let blobs_parent = open_or_create_directory(&root, "blobs", owner)?;
            let blobs = open_or_create_directory(&blobs_parent, "sha256", owner)?;
            let active = open_or_create_directory(&root, "active", owner)?;
            let release_parent = open_or_create_directory(&root, "releases", owner)?;
            let releases = open_or_create_directory(&release_parent, "sha256", owner)?;
            let conformance_parent = open_or_create_directory(&root, "conformance", owner)?;
            let conformance = open_or_create_directory(&conformance_parent, "sha256", owner)?;
            let policies = open_or_create_directory(&root, "policies", owner)?;
            fsync(&root).map_err(io_error)?;
            Ok(Self {
                owner,
                _root: root,
                blobs,
                active,
                releases,
                conformance,
                policies,
            })
        }

        pub fn import_release(
            &self,
            release: &VerifiedRelease,
            signed_policy_bytes: &[u8],
            catalog_key: &CatalogPublicKey,
        ) -> Result<SecureImportReport> {
            let policy = verify_catalog_policy_bytes(signed_policy_bytes, catalog_key, release)?;
            let decision = lifecycle_decision(policy.status);
            self.cache_policy(&policy, signed_policy_bytes, catalog_key)?;
            ensure_allowed(&decision, LifecycleUse::NewLaunch)?;

            let digest = release.payload().archive_sha256.as_str();
            write_immutable(
                &self.blobs,
                &format!("{digest}.ogsc"),
                release.cartridge().archive_bytes(),
            )?;
            write_immutable(
                &self.releases,
                &format!("{digest}.signed.json"),
                release.attestation_bytes(),
            )?;
            write_immutable(
                &self.conformance,
                &format!("{digest}.json"),
                release.conformance_bytes(),
            )?;
            let activation = SecureActivationRecord {
                format_version: 1,
                game_key: release.payload().game_key.clone(),
                publisher_id: release.payload().publisher_id.clone(),
                cartridge_version: release.payload().cartridge_version,
                archive_sha256: release.payload().archive_sha256.clone(),
                signed_identity_sha256: release.payload().signed_identity_sha256.clone(),
                release_attestation_sha256: sha256_hex(release.attestation_bytes()),
                conformance_sha256: sha256_hex(release.conformance_bytes()),
                sdk: release.sdk().clone(),
            };
            atomic_replace(
                &self.active,
                &format!("{}.json", activation.game_key),
                &canonical_json(&activation)?,
            )?;
            Ok(SecureImportReport {
                report_format: "omarchygs.cartridge.secure-import/v1".to_owned(),
                ok: true,
                installed: true,
                activation,
                decision,
                descriptor_relative: true,
                authoritative_policy_verified: true,
                provider_contacted: false,
                database_required: false,
                platform_credentials_read: false,
            })
        }

        pub fn resolve_active(
            &self,
            game_key: &str,
            publisher_key: &PublisherPublicKey,
            host: &HostProfile,
            signed_policy_bytes: &[u8],
            catalog_key: &CatalogPublicKey,
            use_kind: LifecycleUse,
        ) -> Result<SecureResolution> {
            if !valid_identifier(game_key) {
                return Err(CartridgeError::InvalidActivation);
            }
            let activation_bytes = read_at(
                &self.active,
                &format!("{game_key}.json"),
                crate::MAX_JSON_BYTES as u64,
            )?;
            let activation: SecureActivationRecord = serde_json::from_slice(&activation_bytes)?;
            if canonical_json(&activation)? != activation_bytes
                || !valid_activation(&activation, game_key)
            {
                return Err(CartridgeError::InvalidActivation);
            }
            let policy = verify_catalog_policy_signature(signed_policy_bytes, catalog_key)?;
            if policy.game_key != activation.game_key
                || policy.publisher_id != activation.publisher_id
                || policy.archive_sha256 != activation.archive_sha256
            {
                return Err(CartridgeError::InvalidCatalogPolicy);
            }
            self.cache_policy(&policy, signed_policy_bytes, catalog_key)?;
            let decision = lifecycle_decision(policy.status);
            ensure_allowed(&decision, use_kind)?;

            let digest = activation.archive_sha256.as_str();
            let archive = read_at(
                &self.blobs,
                &format!("{digest}.ogsc"),
                crate::MAX_ARCHIVE_BYTES as u64,
            )?;
            if sha256_hex(&archive) != activation.archive_sha256 {
                return Err(CartridgeError::InvalidActivation);
            }
            let conformance = read_at(
                &self.conformance,
                &format!("{digest}.json"),
                MAX_RELEASE_RECORD_BYTES,
            )?;
            let attestation = read_at(
                &self.releases,
                &format!("{digest}.signed.json"),
                MAX_RELEASE_RECORD_BYTES,
            )?;
            if sha256_hex(&conformance) != activation.conformance_sha256
                || sha256_hex(&attestation) != activation.release_attestation_sha256
            {
                return Err(CartridgeError::InvalidActivation);
            }
            let release = verify_release_components(
                &archive,
                &conformance,
                &attestation,
                publisher_key,
                &activation.sdk,
                host,
            )?;
            if release.payload().game_key != activation.game_key
                || release.payload().publisher_id != activation.publisher_id
                || release.payload().cartridge_version != activation.cartridge_version
                || release.payload().archive_sha256 != activation.archive_sha256
                || release.payload().signed_identity_sha256 != activation.signed_identity_sha256
            {
                return Err(CartridgeError::InvalidActivation);
            }
            Ok(SecureResolution {
                cartridge: release.cartridge().clone(),
                activation,
                policy,
                decision,
            })
        }

        fn cache_policy(
            &self,
            policy: &CatalogPolicy,
            bytes: &[u8],
            catalog_key: &CatalogPublicKey,
        ) -> Result<()> {
            if bytes.len() as u64 > MAX_POLICY_BYTES {
                return Err(CartridgeError::LimitExceeded);
            }
            let policy_lock = openat(
                &self.policies,
                ".",
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(io_error)?;
            validate_secure_directory(&policy_lock, self.owner)?;
            flock(&policy_lock, FlockOperation::LockExclusive).map_err(io_error)?;
            let name = format!("{}.signed.json", policy.archive_sha256);
            if let Some(existing_bytes) = read_optional_at(&self.policies, &name, MAX_POLICY_BYTES)?
            {
                let existing = verify_catalog_policy_signature(&existing_bytes, catalog_key)?;
                if existing.archive_sha256 != policy.archive_sha256
                    || existing.game_key != policy.game_key
                    || existing.publisher_id != policy.publisher_id
                    || existing.policy_version > policy.policy_version
                    || (existing.policy_version == policy.policy_version && existing_bytes != bytes)
                {
                    return Err(CartridgeError::InvalidCatalogPolicy);
                }
                if existing_bytes == bytes {
                    return Ok(());
                }
            }
            atomic_replace(&self.policies, &name, bytes)
        }
    }

    fn open_or_create_directory(parent: &OwnedFd, name: &str, owner: Uid) -> Result<OwnedFd> {
        match mkdirat(parent, name, Mode::from_bits_truncate(0o700)) {
            Ok(()) | Err(Errno::EXIST) => {}
            Err(error) => return Err(io_error(error)),
        }
        let directory = openat(
            parent,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io_error)?;
        validate_secure_directory(&directory, owner)?;
        Ok(directory)
    }

    fn validate_secure_directory(directory: &OwnedFd, owner: Uid) -> Result<()> {
        let metadata = fstat(directory).map_err(io_error)?;
        let mode = Mode::from_raw_mode(metadata.st_mode);
        if FileType::from_raw_mode(metadata.st_mode) != FileType::Directory
            || metadata.st_uid != owner.as_raw()
            || mode.intersects(Mode::WGRP | Mode::WOTH)
        {
            return Err(CartridgeError::UnsafeFilesystemPath);
        }
        Ok(())
    }

    fn read_at(parent: &OwnedFd, name: &str, max_bytes: u64) -> Result<Vec<u8>> {
        let fd = openat(
            parent,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io_error)?;
        let file = File::from(fd);
        let metadata = file.metadata()?;
        if !metadata.file_type().is_file() || metadata.len() > max_bytes {
            return Err(CartridgeError::UnsafeFilesystemPath);
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take(max_bytes + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > max_bytes {
            return Err(CartridgeError::LimitExceeded);
        }
        Ok(bytes)
    }

    fn read_optional_at(parent: &OwnedFd, name: &str, max_bytes: u64) -> Result<Option<Vec<u8>>> {
        match openat(
            parent,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => {
                let file = File::from(fd);
                let metadata = file.metadata()?;
                if !metadata.file_type().is_file() || metadata.len() > max_bytes {
                    return Err(CartridgeError::UnsafeFilesystemPath);
                }
                let mut bytes = Vec::with_capacity(metadata.len() as usize);
                file.take(max_bytes + 1).read_to_end(&mut bytes)?;
                if bytes.len() as u64 > max_bytes {
                    return Err(CartridgeError::LimitExceeded);
                }
                Ok(Some(bytes))
            }
            Err(Errno::NOENT) => Ok(None),
            Err(error) => Err(io_error(error)),
        }
    }

    fn write_immutable(parent: &OwnedFd, name: &str, bytes: &[u8]) -> Result<()> {
        if let Some(existing) = read_optional_at(parent, name, bytes.len() as u64 + 1)? {
            return if existing == bytes {
                Ok(())
            } else {
                Err(CartridgeError::InvalidActivation)
            };
        }
        atomic_write(parent, name, bytes, false)
    }

    fn atomic_replace(parent: &OwnedFd, name: &str, bytes: &[u8]) -> Result<()> {
        atomic_write(parent, name, bytes, true)
    }

    fn atomic_write(parent: &OwnedFd, name: &str, bytes: &[u8], replace: bool) -> Result<()> {
        let mut random = [0u8; 12];
        OsRng.fill_bytes(&mut random);
        let suffix = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let temporary = format!(".{name}.tmp-{suffix}");
        let fd = openat(
            parent,
            temporary.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(io_error)?;
        let result = (|| -> Result<()> {
            let mut file = File::from(fd);
            file.write_all(bytes)?;
            file.sync_all()?;
            fchmod(&file, Mode::from_bits_truncate(0o444)).map_err(io_error)?;
            drop(file);
            if replace {
                renameat(parent, temporary.as_str(), parent, name).map_err(io_error)?;
            } else {
                match renameat_with(
                    parent,
                    temporary.as_str(),
                    parent,
                    name,
                    RenameFlags::NOREPLACE,
                ) {
                    Ok(()) => {}
                    Err(Errno::EXIST) => {
                        unlinkat(parent, temporary.as_str(), AtFlags::empty()).map_err(io_error)?;
                        let existing = read_at(parent, name, bytes.len() as u64 + 1)?;
                        if existing != bytes {
                            return Err(CartridgeError::InvalidActivation);
                        }
                    }
                    Err(error) => return Err(io_error(error)),
                }
            }
            fsync(parent).map_err(io_error)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = unlinkat(parent, temporary.as_str(), AtFlags::empty());
        }
        result
    }

    fn io_error(error: Errno) -> CartridgeError {
        CartridgeError::Io(error.into())
    }

    fn valid_activation(activation: &SecureActivationRecord, game_key: &str) -> bool {
        activation.format_version == 1
            && activation.game_key == game_key
            && valid_identifier(&activation.game_key)
            && valid_identifier(&activation.publisher_id)
            && activation.cartridge_version > 0
            && valid_sha256(&activation.archive_sha256)
            && valid_sha256(&activation.signed_identity_sha256)
            && valid_sha256(&activation.release_attestation_sha256)
            && valid_sha256(&activation.conformance_sha256)
    }

    fn valid_sha256(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn secure_directory_validation_rejects_an_unexpected_owner() {
            let root = tempfile::tempdir().unwrap();
            let directory = open(
                root.path(),
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .unwrap();
            let current = geteuid().as_raw();
            let unexpected = if current == 0 { 1 } else { 0 };

            assert!(matches!(
                validate_secure_directory(&directory, Uid::from_raw(unexpected)),
                Err(CartridgeError::UnsafeFilesystemPath)
            ));
            assert!(validate_secure_directory(&directory, geteuid()).is_ok());
        }
    }
}

#[cfg(target_os = "linux")]
pub use platform::SecureCartridgeStore;

#[cfg(not(target_os = "linux"))]
pub struct SecureCartridgeStore;

#[cfg(not(target_os = "linux"))]
impl SecureCartridgeStore {
    pub fn open_existing(_root: &std::path::Path) -> Result<Self> {
        Err(CartridgeError::UnsupportedSecureStore)
    }
}

#[allow(dead_code)]
fn _decision_contract(decision: &LifecycleDecision) -> bool {
    matches!(
        (decision.new_launch, decision.active_session),
        (NewLaunchDecision::Allow, ActiveSessionDecision::Continue)
            | (
                NewLaunchDecision::AllowWithWarning,
                ActiveSessionDecision::Continue
            )
            | (NewLaunchDecision::Deny, ActiveSessionDecision::Suspend)
            | (NewLaunchDecision::Deny, ActiveSessionDecision::Terminate)
            | (NewLaunchDecision::Deny, ActiveSessionDecision::Continue)
    )
}
