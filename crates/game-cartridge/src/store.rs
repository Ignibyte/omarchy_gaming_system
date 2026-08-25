use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use rand_core::{OsRng, RngCore};

use crate::{
    archive::sha256_hex,
    contract::{
        ActivationRecord, HostProfile, MAX_ARCHIVE_BYTES, MAX_JSON_BYTES, RevocationRecord,
        VerifiedCartridge,
    },
    error::{CartridgeError, Result},
    io::read_bounded_regular_file,
    keys::{PublisherPublicKey, valid_identifier},
    validate::canonical_json,
    verify_archive, verify_archive_bytes,
};

pub fn install_cartridge(
    archive_path: &Path,
    key: &PublisherPublicKey,
    host: &HostProfile,
    store_root: &Path,
) -> Result<ActivationRecord> {
    let verified = verify_archive(archive_path, key, host)?;
    if !verified.compatibility.compatible {
        return Err(CartridgeError::Incompatible);
    }
    prepare_store(store_root)?;
    if is_revoked(store_root, &verified.archive_sha256)? {
        return Err(CartridgeError::Revoked);
    }

    let blob = blob_path(store_root, &verified.archive_sha256);
    match fs::symlink_metadata(&blob) {
        Ok(_) => {
            let existing = read_bounded_regular_file(&blob, MAX_ARCHIVE_BYTES as u64)?;
            if existing != verified.archive_bytes {
                return Err(CartridgeError::InvalidActivation);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            atomic_write(&blob, &verified.archive_bytes)?;
        }
        Err(error) => return Err(error.into()),
    }

    let activation = ActivationRecord {
        format_version: 1,
        game_key: verified.manifest.game_key,
        cartridge_version: verified.manifest.cartridge_version,
        archive_sha256: verified.archive_sha256,
        signed_identity_sha256: verified.signed_identity_sha256,
    };
    let path = activation_path(store_root, &activation.game_key)?;
    atomic_write(&path, &canonical_json(&activation)?)?;
    Ok(activation)
}

pub fn revoke_cartridge(store_root: &Path, archive_sha256: &str, reason: &str) -> Result<()> {
    if !valid_sha256(archive_sha256)
        || reason.is_empty()
        || reason.chars().count() > 512
        || reason.chars().any(char::is_control)
    {
        return Err(CartridgeError::InvalidActivation);
    }
    prepare_store(store_root)?;
    let record = RevocationRecord {
        format_version: 1,
        archive_sha256: archive_sha256.to_owned(),
        reason: reason.to_owned(),
    };
    atomic_write(
        &revoked_path(store_root, archive_sha256),
        &canonical_json(&record)?,
    )
}

pub fn resolve_active_cartridge(
    store_root: &Path,
    game_key: &str,
    key: &PublisherPublicKey,
    host: &HostProfile,
) -> Result<VerifiedCartridge> {
    validate_store(store_root)?;
    let activation_file = activation_path(store_root, game_key)?;
    let activation_bytes = read_bounded_regular_file(&activation_file, MAX_JSON_BYTES as u64)?;
    let activation: ActivationRecord = serde_json::from_slice(&activation_bytes)?;
    if activation.format_version != 1
        || activation.game_key != game_key
        || !valid_sha256(&activation.archive_sha256)
        || !valid_sha256(&activation.signed_identity_sha256)
        || canonical_json(&activation)? != activation_bytes
    {
        return Err(CartridgeError::InvalidActivation);
    }
    if is_revoked(store_root, &activation.archive_sha256)? {
        return Err(CartridgeError::Revoked);
    }
    let blob = blob_path(store_root, &activation.archive_sha256);
    let bytes = read_bounded_regular_file(&blob, MAX_ARCHIVE_BYTES as u64)?;
    if sha256_hex(&bytes) != activation.archive_sha256 {
        return Err(CartridgeError::InvalidActivation);
    }
    let verified = verify_archive_bytes(&bytes, key, host)?;
    if !verified.compatibility.compatible
        || verified.manifest.game_key != game_key
        || verified.manifest.cartridge_version != activation.cartridge_version
        || verified.signed_identity_sha256 != activation.signed_identity_sha256
    {
        return Err(CartridgeError::InvalidActivation);
    }
    Ok(verified)
}

fn prepare_store(root: &Path) -> Result<()> {
    create_or_validate_directory(root)?;
    create_or_validate_directory(&root.join("blobs"))?;
    create_or_validate_directory(&root.join("blobs/sha256"))?;
    create_or_validate_directory(&root.join("active"))?;
    create_or_validate_directory(&root.join("revoked"))?;
    Ok(())
}

fn validate_store(root: &Path) -> Result<()> {
    ensure_directory(root)?;
    ensure_directory(&root.join("blobs"))?;
    ensure_directory(&root.join("blobs/sha256"))?;
    ensure_directory(&root.join("active"))?;
    ensure_directory(&root.join("revoked"))?;
    Ok(())
}

fn create_or_validate_directory(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            Ok(())
        }
        Ok(_) => Err(CartridgeError::UnsafeFilesystemPath),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path)?;
            ensure_directory(path)
        }
        Err(error) => Err(error.into()),
    }
}

fn ensure_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        Ok(())
    } else {
        Err(CartridgeError::UnsafeFilesystemPath)
    }
}

fn activation_path(root: &Path, game_key: &str) -> Result<PathBuf> {
    if !valid_identifier(game_key) {
        return Err(CartridgeError::InvalidActivation);
    }
    Ok(root.join("active").join(format!("{game_key}.json")))
}

fn blob_path(root: &Path, digest: &str) -> PathBuf {
    root.join("blobs/sha256").join(format!("{digest}.ogsc"))
}

fn revoked_path(root: &Path, digest: &str) -> PathBuf {
    root.join("revoked").join(format!("{digest}.json"))
}

fn is_revoked(root: &Path, digest: &str) -> Result<bool> {
    let path = revoked_path(root, digest);
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            let bytes = read_bounded_regular_file(&path, MAX_JSON_BYTES as u64)?;
            let record: RevocationRecord = serde_json::from_slice(&bytes)?;
            if record.format_version != 1
                || record.archive_sha256 != digest
                || record.reason.is_empty()
                || record.reason.chars().count() > 512
                || record.reason.chars().any(char::is_control)
                || canonical_json(&record)? != bytes
            {
                return Err(CartridgeError::InvalidActivation);
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or(CartridgeError::UnsafeFilesystemPath)?;
    ensure_directory(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(CartridgeError::UnsafeFilesystemPath)?;
    let mut random = [0u8; 8];
    OsRng.fill_bytes(&mut random);
    let suffix = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let temporary = parent.join(format!(".{file_name}.tmp-{suffix}"));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        set_read_only(&file)?;
        drop(file);
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(unix)]
fn set_read_only(file: &File) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o444))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_read_only(file: &File) -> Result<()> {
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions)?;
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
