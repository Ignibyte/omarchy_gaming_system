//! Root-authenticated package metadata and non-executable local staging.

use std::{
    cmp::Ordering,
    fs::File,
    io::Read as _,
    os::fd::OwnedFd,
    path::{Path, PathBuf},
    sync::Mutex,
};

use omarchygs_marketplace_trust::{
    ChannelEgressError, ChannelOrigin, ClientPackageArtifact, GuardedChannelClient,
    PublicChannelBootstrap,
};
use rand_core::{OsRng, RngCore as _};
use rustix::{
    fs::{
        AtFlags, Dir, FileType, FlockOperation, Mode, OFlags, RenameFlags, fchmod, flock, fstat,
        fsync, mkdirat, open, openat, renameat_with, unlinkat,
    },
    process::geteuid,
};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

use crate::{ClientMarketplaceTrust, CompanionError, Result};

const UPDATES_DIRECTORY: &str = "updates";
const UPDATES_LOCK: &str = ".updates.lock";
const MAX_STAGED_PACKAGE_FILES: usize = 8;
const MAX_STAGED_PACKAGE_BYTES: u64 = 512 * 1024 * 1024;

pub struct ClientPackageChannel {
    bootstrap: PublicChannelBootstrap,
    updates: OwnedFd,
    lock_file: OwnedFd,
    process_lock: Mutex<()>,
    download_lock: tokio::sync::Mutex<()>,
    updates_path: PathBuf,
}

impl ClientPackageChannel {
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
        match mkdirat(&root_fd, UPDATES_DIRECTORY, Mode::from_bits_truncate(0o700)) {
            Ok(()) | Err(rustix::io::Errno::EXIST) => {}
            Err(_) => return Err(CompanionError::Cache),
        }
        let updates = openat(
            &root_fd,
            UPDATES_DIRECTORY,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_private_directory(&updates)?;
        let lock_file = openat(
            &updates,
            UPDATES_LOCK,
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_private_file(&lock_file)?;
        flock(&lock_file, FlockOperation::LockExclusive).map_err(|_| CompanionError::Cache)?;
        let measured = measure_staging(&updates, None).map(|_| ());
        if flock(&lock_file, FlockOperation::Unlock).is_err() {
            return Err(CompanionError::Cache);
        }
        measured?;
        fsync(&updates).map_err(|_| CompanionError::Cache)?;
        fsync(&root_fd).map_err(|_| CompanionError::Cache)?;
        Ok(Self {
            bootstrap,
            updates,
            lock_file,
            process_lock: Mutex::new(()),
            download_lock: tokio::sync::Mutex::new(()),
            updates_path: root.join(UPDATES_DIRECTORY),
        })
    }

    pub fn status(&self, trust: &ClientMarketplaceTrust) -> Result<ClientPackageStatus> {
        let snapshot = trust.snapshot()?;
        let mut available = snapshot
            .packages()
            .iter()
            .filter(|artifact| self.matches_running_package(artifact))
            .filter(|artifact| {
                compare_versions(
                    &artifact.package_version,
                    &self.bootstrap.installed_package_version,
                ) == Ordering::Greater
            })
            .cloned()
            .collect::<Vec<_>>();
        available.sort_by(|left, right| {
            compare_versions(&left.package_version, &right.package_version)
                .then_with(|| left.sha256.cmp(&right.sha256))
        });
        Ok(ClientPackageStatus {
            format: "omarchygs.client-package-status/v1",
            platform: self.bootstrap.platform.clone(),
            architecture: self.bootstrap.architecture.clone(),
            installed_package_version: self.bootstrap.installed_package_version.clone(),
            available,
        })
    }

    pub async fn stage(
        &self,
        trust: &ClientMarketplaceTrust,
        sha256: &str,
    ) -> Result<StagedPackage> {
        let _download = self.download_lock.lock().await;
        let initial = trust.snapshot()?;
        let artifact = initial
            .packages()
            .iter()
            .find(|artifact| {
                artifact.sha256 == sha256
                    && self.matches_running_package(artifact)
                    && compare_versions(
                        &artifact.package_version,
                        &self.bootstrap.installed_package_version,
                    ) == Ordering::Greater
            })
            .cloned()
            .ok_or(CompanionError::InvalidInput)?;
        let target = staged_name(&artifact.sha256);
        if self.existing_is_exact(&target, &artifact)? {
            self.require_current_artifact(trust, &artifact)?;
            return Ok(self.staged(&artifact, target));
        }
        let origin = ChannelOrigin::parse(&self.bootstrap.channel_origin)
            .map_err(|_| CompanionError::MarketplaceUntrusted)?;
        let client = GuardedChannelClient::production(origin)
            .await
            .map_err(map_channel_error)?;
        let mut random = [0_u8; 16];
        OsRng.fill_bytes(&mut random);
        let suffix = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let temporary = format!(".{}.{suffix}.tmp", artifact.sha256);
        let fd = openat(
            &self.updates,
            temporary.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_bits_truncate(0o600),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_private_file(&fd)?;
        let std_file = File::from(fd);
        let mut output = tokio::fs::File::from_std(std_file);
        let download = client
            .download_exact(
                &artifact.relative_path,
                "application/vnd.archlinux.package",
                artifact.bytes,
                &artifact.sha256,
                &mut output,
            )
            .await;
        if let Err(error) = download {
            drop(output);
            let _ = unlinkat(&self.updates, temporary.as_str(), AtFlags::empty());
            return Err(map_channel_error(error));
        }
        if output.sync_all().await.is_err() {
            drop(output);
            let _ = unlinkat(&self.updates, temporary.as_str(), AtFlags::empty());
            return Err(CompanionError::Cache);
        }
        let std_file = output.into_std().await;
        let _process = match self.process_lock.lock() {
            Ok(lock) => lock,
            Err(_) => {
                let _ = unlinkat(&self.updates, temporary.as_str(), AtFlags::empty());
                return Err(CompanionError::Cache);
            }
        };
        if flock(&self.lock_file, FlockOperation::LockExclusive).is_err() {
            let _ = unlinkat(&self.updates, temporary.as_str(), AtFlags::empty());
            return Err(CompanionError::Cache);
        }
        let publish = (|| {
            fchmod(&std_file, Mode::from_bits_truncate(0o600))
                .map_err(|_| CompanionError::Cache)?;
            std_file.sync_all().map_err(|_| CompanionError::Cache)?;
            drop(std_file);
            if self.existing_is_exact(&target, &artifact)? {
                unlinkat(&self.updates, temporary.as_str(), AtFlags::empty())
                    .map_err(|_| CompanionError::Cache)?;
                self.require_current_artifact(trust, &artifact)?;
                return Ok(self.staged(&artifact, target.clone()));
            }
            ensure_capacity(&self.updates, artifact.bytes, &temporary)?;
            self.require_current_artifact(trust, &artifact)?;
            match renameat_with(
                &self.updates,
                temporary.as_str(),
                &self.updates,
                target.as_str(),
                RenameFlags::NOREPLACE,
            ) {
                Ok(()) => {}
                Err(rustix::io::Errno::EXIST) => {
                    unlinkat(&self.updates, temporary.as_str(), AtFlags::empty())
                        .map_err(|_| CompanionError::Cache)?;
                    if !self.existing_is_exact(&target, &artifact)? {
                        return Err(CompanionError::Cache);
                    }
                }
                Err(_) => return Err(CompanionError::Cache),
            }
            fsync(&self.updates).map_err(|_| CompanionError::Cache)?;
            Ok(self.staged(&artifact, target.clone()))
        })();
        if flock(&self.lock_file, FlockOperation::Unlock).is_err() {
            let _ = unlinkat(&self.updates, temporary.as_str(), AtFlags::empty());
            return Err(CompanionError::Cache);
        }
        match publish {
            Ok(staged) => Ok(staged),
            Err(error) => {
                let _ = unlinkat(&self.updates, temporary.as_str(), AtFlags::empty());
                Err(error)
            }
        }
    }

    fn matches_running_package(&self, artifact: &ClientPackageArtifact) -> bool {
        artifact.platform == self.bootstrap.platform
            && artifact.architecture == self.bootstrap.architecture
    }

    fn require_current_artifact(
        &self,
        trust: &ClientMarketplaceTrust,
        artifact: &ClientPackageArtifact,
    ) -> Result<()> {
        if trust
            .snapshot()?
            .packages()
            .iter()
            .any(|candidate| candidate == artifact)
        {
            Ok(())
        } else {
            Err(CompanionError::MarketplaceUntrusted)
        }
    }

    fn existing_is_exact(&self, name: &str, artifact: &ClientPackageArtifact) -> Result<bool> {
        let fd = match openat(
            &self.updates,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) => return Ok(false),
            Err(_) => return Err(CompanionError::Cache),
        };
        validate_private_file(&fd)?;
        let metadata = fstat(&fd).map_err(|_| CompanionError::Cache)?;
        if metadata.st_size < 0 || metadata.st_size as u64 != artifact.bytes {
            return Err(CompanionError::Cache);
        }
        let mut file = File::from(fd);
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|_| CompanionError::Cache)?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        let actual = digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if actual != artifact.sha256 {
            return Err(CompanionError::Cache);
        }
        Ok(true)
    }

    fn staged(&self, artifact: &ClientPackageArtifact, name: String) -> StagedPackage {
        let path = self.updates_path.join(name);
        let path_text = path.to_string_lossy().into_owned();
        StagedPackage {
            format: "omarchygs.staged-client-package/v1",
            package_version: artifact.package_version.clone(),
            bytes: artifact.bytes,
            sha256: artifact.sha256.clone(),
            source_revision: artifact.source_revision.clone(),
            source_sha256: artifact.source_sha256.clone(),
            build_provenance_sha256: artifact.build_provenance_sha256.clone(),
            staged_path: path_text.clone(),
            install_command: format!("sudo pacman -U -- {}", shell_quote(&path_text)),
        }
    }
}

fn ensure_capacity(updates: &OwnedFd, additional: u64, current_temporary: &str) -> Result<()> {
    let (files, bytes) = measure_staging(updates, Some(current_temporary))?;
    if files
        .checked_add(1)
        .is_none_or(|files| files > MAX_STAGED_PACKAGE_FILES)
        || bytes
            .checked_add(additional)
            .is_none_or(|bytes| bytes > MAX_STAGED_PACKAGE_BYTES)
    {
        Err(CompanionError::Cache)
    } else {
        Ok(())
    }
}

fn measure_staging(updates: &OwnedFd, excluded_name: Option<&str>) -> Result<(usize, u64)> {
    let directory = Dir::read_from(updates).map_err(|_| CompanionError::Cache)?;
    let mut files = 0_usize;
    let mut bytes = 0_u64;
    for entry in directory {
        let entry = entry.map_err(|_| CompanionError::Cache)?;
        let name = entry
            .file_name()
            .to_str()
            .map_err(|_| CompanionError::Cache)?;
        if matches!(name, "." | ".." | UPDATES_LOCK) {
            continue;
        }
        if !valid_staged_name(name) && !valid_temporary_name(name) {
            return Err(CompanionError::Cache);
        }
        let file = openat(
            updates,
            entry.file_name(),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| CompanionError::Cache)?;
        validate_private_file(&file)?;
        let metadata = fstat(&file).map_err(|_| CompanionError::Cache)?;
        if metadata.st_size <= 0
            || metadata.st_size as u64 > omarchygs_marketplace_trust::MAX_PACKAGE_BYTES
        {
            return Err(CompanionError::Cache);
        }
        drop(file);
        if excluded_name == Some(name) {
            continue;
        }
        files = files.checked_add(1).ok_or(CompanionError::Cache)?;
        bytes = bytes
            .checked_add(metadata.st_size as u64)
            .ok_or(CompanionError::Cache)?;
        if files > MAX_STAGED_PACKAGE_FILES || bytes > MAX_STAGED_PACKAGE_BYTES {
            return Err(CompanionError::Cache);
        }
    }
    Ok((files, bytes))
}

fn valid_staged_name(value: &str) -> bool {
    value.strip_suffix(".pkg.tar.zst").is_some_and(valid_sha256)
}

fn valid_temporary_name(value: &str) -> bool {
    let Some(value) = value.strip_prefix('.') else {
        return false;
    };
    let Some((digest, suffix)) = value.split_once('.') else {
        return false;
    };
    let Some(random) = suffix.strip_suffix(".tmp") else {
        return false;
    };
    valid_sha256(digest)
        && random.len() == 32
        && random
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientPackageStatus {
    pub format: &'static str,
    pub platform: String,
    pub architecture: String,
    pub installed_package_version: String,
    pub available: Vec<ClientPackageArtifact>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StagedPackage {
    pub format: &'static str,
    pub package_version: String,
    pub bytes: u64,
    pub sha256: String,
    pub source_revision: String,
    pub source_sha256: String,
    pub build_provenance_sha256: String,
    pub staged_path: String,
    pub install_command: String,
}

fn staged_name(sha256: &str) -> String {
    format!("{sha256}.pkg.tar.zst")
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    version_tokens(left).cmp(&version_tokens(right))
}

fn version_tokens(value: &str) -> Vec<VersionToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut numeric = None;
    for character in value.chars() {
        let is_numeric = character.is_ascii_digit();
        if numeric.is_some_and(|prior| prior != is_numeric) {
            tokens.push(version_token(&current, numeric.unwrap_or(false)));
            current.clear();
        }
        numeric = Some(is_numeric);
        current.push(character);
    }
    if !current.is_empty() {
        tokens.push(version_token(&current, numeric.unwrap_or(false)));
    }
    tokens
}

fn version_token(value: &str, numeric: bool) -> VersionToken {
    if numeric {
        let normalized = value.trim_start_matches('0');
        let normalized = if normalized.is_empty() {
            "0"
        } else {
            normalized
        };
        VersionToken::Number(normalized.len(), normalized.to_owned())
    } else {
        VersionToken::Text(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum VersionToken {
    Number(usize, String),
    Text(String),
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

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt as _};

    use omarchygs_marketplace_trust::generate_trust_root_keypair;

    use super::*;

    #[test]
    fn package_versions_compare_numeric_runs() {
        assert_eq!(compare_versions("0.10.0-1", "0.9.0-9"), Ordering::Greater);
        assert_eq!(compare_versions("1.0.0-2", "1.0.0-10"), Ordering::Less);
        assert_eq!(shell_quote("/tmp/player's.pkg"), "'/tmp/player'\\''s.pkg'");
    }

    #[test]
    fn staging_inventory_is_private_bounded_and_restart_validated() {
        let temp = tempfile::tempdir().expect("temp should create");
        let root = temp.path().join("cache");
        fs::create_dir(&root).expect("cache should create");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("cache mode should restrict");
        let channel = ClientPackageChannel::open(&root, bootstrap()).expect("channel should open");
        for index in 1..=MAX_STAGED_PACKAGE_FILES {
            let path = root
                .join(UPDATES_DIRECTORY)
                .join(format!("{:064x}.pkg.tar.zst", index));
            fs::write(&path, [index as u8]).expect("staged fixture should write");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .expect("staged mode should restrict");
        }
        assert!(
            ensure_capacity(
                &channel.updates,
                1,
                &format!(".{}.{}.tmp", "f".repeat(64), "e".repeat(32)),
            )
            .is_err(),
            "a ninth staged object must exceed the bounded inventory"
        );
        drop(channel);
        ClientPackageChannel::open(&root, bootstrap())
            .expect("bounded staged inventory should survive restart");

        let hostile_root = temp.path().join("hostile-cache");
        fs::create_dir(&hostile_root).expect("hostile cache should create");
        fs::set_permissions(&hostile_root, fs::Permissions::from_mode(0o700))
            .expect("hostile cache mode should restrict");
        fs::create_dir(hostile_root.join(UPDATES_DIRECTORY)).expect("updates should create");
        fs::set_permissions(
            hostile_root.join(UPDATES_DIRECTORY),
            fs::Permissions::from_mode(0o700),
        )
        .expect("updates mode should restrict");
        std::os::unix::fs::symlink(
            "/etc/passwd",
            hostile_root
                .join(UPDATES_DIRECTORY)
                .join(format!("{}.pkg.tar.zst", "a".repeat(64))),
        )
        .expect("hostile symlink should create");
        assert!(ClientPackageChannel::open(&hostile_root, bootstrap()).is_err());
    }

    fn bootstrap() -> PublicChannelBootstrap {
        let (_, root) =
            generate_trust_root_keypair("package-root", "official").expect("root should generate");
        PublicChannelBootstrap {
            format: "omarchygs.public-channel-bootstrap/v1".to_owned(),
            channel_id: "official".to_owned(),
            channel_origin: "https://packages.example.test/v1/".to_owned(),
            manifest_path: "trust.signed.json".to_owned(),
            minimum_bundle_version: 1,
            minimum_current_snapshot_version: 1,
            platform: "arch-linux".to_owned(),
            architecture: "x86_64".to_owned(),
            installed_package_version: "0.1.0-1".to_owned(),
            root,
        }
    }
}
