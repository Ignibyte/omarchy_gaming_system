use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer as _, SigningKey, Verifier as _, VerifyingKey};
use omarchygs_provider_sdk::{
    ProviderError, Result,
    protocol::{ProviderCompatibility, sha256_hex},
};
use serde::{Deserialize, Serialize};

const DOMAIN: &[u8] = b"omarchygs-provider-developer-kit-v1\0";
const LOCK_PATH: &str = "developer-kit-lock.json";
const RELEASE_PATH: &str = "developer-kit-release.json";
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const PACKAGES: [&str; 3] = [
    "omarchygs-provider-sdk-0.1.0.crate",
    "omarchygs-provider-starter-0.1.0.crate",
    "omarchygs-provider-conformance-0.1.0.crate",
];
const STATIC_FILES: &[(&str, &[u8])] = &[
    ("README.md", include_bytes!("../kit/v1/README.md")),
    ("LICENSES.md", include_bytes!("../kit/v1/LICENSES.md")),
    ("faults.json", include_bytes!("../kit/v1/faults.json")),
    (
        "schemas/config.schema.json",
        include_bytes!("../kit/v1/config.schema.json"),
    ),
    (
        "schemas/receipt.schema.json",
        include_bytes!("../kit/v1/receipt.schema.json"),
    ),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FilePin {
    path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DeveloperKitLock {
    format: String,
    sdk_version: u32,
    compatibility: ProviderCompatibility,
    files: Vec<FilePin>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReleasePayload {
    format: String,
    authority: String,
    key_id: String,
    lock_sha256: String,
    source_revision: String,
    builder_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SignedRelease {
    key_id: String,
    payload: String,
    signature: String,
}

/// Stable identity of an exported and verified developer kit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperKitIdentity {
    pub lock_sha256: String,
    pub release_sha256: String,
    pub source_revision: String,
    pub builder_sha256: String,
}

/// Local signer for an exact preview kit. Secret bytes are never serialized.
pub struct DeveloperKitReleaseSigner {
    authority: String,
    key_id: String,
    signing_key: SigningKey,
}

impl DeveloperKitReleaseSigner {
    pub fn new(authority: &str, key_id: &str, seed: [u8; 32]) -> Result<Self> {
        if !identifier(authority) || !identifier(key_id) {
            return Err(ProviderError::InvalidInput);
        }
        Ok(Self {
            authority: authority.to_owned(),
            key_id: key_id.to_owned(),
            signing_key: SigningKey::from_bytes(&seed),
        })
    }

    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

/// Export exact Cargo archives and compiled-owned public kit material into an
/// existing empty directory.
pub fn export_developer_kit(
    output: &Path,
    sdk_archive: &Path,
    starter_archive: &Path,
    conformance_archive: &Path,
    signer: &DeveloperKitReleaseSigner,
    source_revision: &str,
    builder_sha256: &str,
) -> Result<DeveloperKitIdentity> {
    validate_provenance(source_revision, builder_sha256)?;
    require_empty_directory(output)?;
    let archive_paths = [sdk_archive, starter_archive, conformance_archive];
    let mut contents = BTreeMap::new();
    for (name, path) in PACKAGES.into_iter().zip(archive_paths) {
        contents.insert(format!("packages/{name}"), read_regular(path)?);
    }
    for (path, bytes) in STATIC_FILES {
        contents.insert((*path).to_owned(), bytes.to_vec());
    }
    for (path, bytes) in &contents {
        write_new(&output.join(path), bytes)?;
    }
    let lock = DeveloperKitLock {
        format: "omarchygs.provider-developer-kit-lock/v1".to_owned(),
        sdk_version: 1,
        compatibility: ProviderCompatibility::current(),
        files: contents
            .iter()
            .map(|(path, bytes)| FilePin {
                path: path.clone(),
                bytes: bytes.len() as u64,
                sha256: sha256_hex(bytes),
            })
            .collect(),
    };
    let lock_bytes = canonical(&lock)?;
    write_new(&output.join(LOCK_PATH), &lock_bytes)?;
    let payload = ReleasePayload {
        format: "omarchygs.provider-developer-kit-release-payload/v1".to_owned(),
        authority: signer.authority.clone(),
        key_id: signer.key_id.clone(),
        lock_sha256: sha256_hex(&lock_bytes),
        source_revision: source_revision.to_owned(),
        builder_sha256: builder_sha256.to_owned(),
    };
    let payload_bytes = canonical(&payload)?;
    let mut signed = DOMAIN.to_vec();
    signed.extend_from_slice(&payload_bytes);
    let release = SignedRelease {
        key_id: signer.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signer.signing_key.sign(&signed).to_bytes()),
    };
    let release_bytes = canonical(&release)?;
    write_new(&output.join(RELEASE_PATH), &release_bytes)?;
    sync_directories(output)?;
    Ok(identity(&payload, &release_bytes))
}

/// Verify exact inventory, hashes, canonical JSON, provenance, and signature.
pub fn verify_developer_kit(
    root: &Path,
    key: &VerifyingKey,
    authority: &str,
    key_id: &str,
) -> Result<DeveloperKitIdentity> {
    if !identifier(authority) || !identifier(key_id) {
        return Err(ProviderError::InvalidInput);
    }
    let lock_bytes = read_regular(&root.join(LOCK_PATH))?;
    let lock: DeveloperKitLock =
        serde_json::from_slice(&lock_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    if canonical(&lock)? != lock_bytes
        || lock.format != "omarchygs.provider-developer-kit-lock/v1"
        || lock.sdk_version != 1
        || lock.compatibility != ProviderCompatibility::current()
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let expected = lock
        .files
        .iter()
        .map(|pin| PathBuf::from(&pin.path))
        .chain([PathBuf::from(LOCK_PATH), PathBuf::from(RELEASE_PATH)])
        .collect::<BTreeSet<_>>();
    if collect_files(root)? != expected {
        return Err(ProviderError::ProtocolRejected);
    }
    for pin in &lock.files {
        let bytes = read_regular(&root.join(&pin.path))?;
        if bytes.len() as u64 != pin.bytes || sha256_hex(&bytes) != pin.sha256 {
            return Err(ProviderError::ProtocolRejected);
        }
    }
    let release_bytes = read_regular(&root.join(RELEASE_PATH))?;
    let release: SignedRelease =
        serde_json::from_slice(&release_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    if canonical(&release)? != release_bytes || release.key_id != key_id {
        return Err(ProviderError::ProtocolRejected);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(&release.payload)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let signature = Signature::from_slice(
        &URL_SAFE_NO_PAD
            .decode(&release.signature)
            .map_err(|_| ProviderError::ProtocolRejected)?,
    )
    .map_err(|_| ProviderError::ProtocolRejected)?;
    let mut signed = DOMAIN.to_vec();
    signed.extend_from_slice(&payload_bytes);
    key.verify(&signed, &signature)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let payload: ReleasePayload =
        serde_json::from_slice(&payload_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    validate_provenance(&payload.source_revision, &payload.builder_sha256)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if canonical(&payload)? != payload_bytes
        || payload.format != "omarchygs.provider-developer-kit-release-payload/v1"
        || payload.authority != authority
        || payload.key_id != key_id
        || payload.lock_sha256 != sha256_hex(&lock_bytes)
    {
        return Err(ProviderError::ProtocolRejected);
    }
    Ok(identity(&payload, &release_bytes))
}

fn identity(payload: &ReleasePayload, release: &[u8]) -> DeveloperKitIdentity {
    DeveloperKitIdentity {
        lock_sha256: payload.lock_sha256.clone(),
        release_sha256: sha256_hex(release),
        source_revision: payload.source_revision.clone(),
        builder_sha256: payload.builder_sha256.clone(),
    }
}

fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| ProviderError::Internal)
}

fn identifier(value: &str) -> bool {
    (3..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn validate_provenance(source_revision: &str, builder_sha256: &str) -> Result<()> {
    let lowercase_hex = |value: &str, len| {
        value.len() == len
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if lowercase_hex(source_revision, 40) && lowercase_hex(builder_sha256, 64) {
        Ok(())
    } else {
        Err(ProviderError::InvalidInput)
    }
}

fn require_empty_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ProviderError::InvalidInput)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(ProviderError::InvalidInput);
    }
    if fs::read_dir(path)
        .map_err(|_| ProviderError::Internal)?
        .next()
        .is_some()
    {
        return Err(ProviderError::InvalidInput);
    }
    Ok(())
}

fn read_regular(path: &Path) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ProviderError::ProtocolRejected)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > MAX_FILE_BYTES
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(path)
        .map_err(|_| ProviderError::ProtocolRejected)?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if bytes.len() as u64 == metadata.len() {
        Ok(bytes)
    } else {
        Err(ProviderError::ProtocolRejected)
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| ProviderError::Internal)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o444);
    }
    let mut file = options.open(path).map_err(|_| ProviderError::Internal)?;
    file.write_all(bytes).map_err(|_| ProviderError::Internal)?;
    file.sync_all().map_err(|_| ProviderError::Internal)
}

fn collect_files(root: &Path) -> Result<BTreeSet<PathBuf>> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|_| ProviderError::ProtocolRejected)? {
            let entry = entry.map_err(|_| ProviderError::ProtocolRejected)?;
            let metadata = entry
                .metadata()
                .map_err(|_| ProviderError::ProtocolRejected)?;
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                files.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .map_err(|_| ProviderError::ProtocolRejected)?
                        .to_path_buf(),
                );
            } else {
                return Err(ProviderError::ProtocolRejected);
            }
        }
    }
    Ok(files)
}

fn sync_directories(root: &Path) -> Result<()> {
    let mut directories = vec![
        root.to_path_buf(),
        root.join("packages"),
        root.join("schemas"),
    ];
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        File::open(directory)
            .map_err(|_| ProviderError::Internal)?
            .sync_all()
            .map_err(|_| ProviderError::Internal)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_export_and_exact_inventory() {
        let temp = tempfile::tempdir().expect("temp");
        let archives = [
            temp.path().join("sdk"),
            temp.path().join("starter"),
            temp.path().join("conf"),
        ];
        for (index, path) in archives.iter().enumerate() {
            fs::write(path, vec![u8::try_from(index + 1).expect("byte"); 64]).expect("archive");
        }
        let signer =
            DeveloperKitReleaseSigner::new("omarchygs", "kit-release-1", [7; 32]).expect("signer");
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).expect("first");
        fs::create_dir(&second).expect("second");
        let revision = "a".repeat(40);
        let builder = "b".repeat(64);
        let one = export_developer_kit(
            &first,
            &archives[0],
            &archives[1],
            &archives[2],
            &signer,
            &revision,
            &builder,
        )
        .expect("export");
        let two = export_developer_kit(
            &second,
            &archives[0],
            &archives[1],
            &archives[2],
            &signer,
            &revision,
            &builder,
        )
        .expect("export");
        assert_eq!(one, two);
        assert_eq!(
            verify_developer_kit(
                &first,
                &signer.verifying_key(),
                "omarchygs",
                "kit-release-1"
            )
            .expect("verify"),
            one
        );
        assert_eq!(
            collect_files(&first).expect("files"),
            collect_files(&second).expect("files")
        );
        for path in collect_files(&first).expect("files") {
            assert_eq!(
                fs::read(first.join(&path)).expect("first file"),
                fs::read(second.join(path)).expect("second file")
            );
        }
    }
}
