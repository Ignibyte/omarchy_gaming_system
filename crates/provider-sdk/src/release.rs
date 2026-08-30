//! Deterministic SDK export and signed release provenance.

use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer as _, SigningKey, Verifier as _, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{
    ProviderError, Result,
    model::{is_identifier, is_sha256_hex},
    protocol::{ProviderCompatibility, sha256_hex},
};

/// First deterministic Provider SDK export format.
pub const PROVIDER_SDK_VERSION: u32 = 1;
/// Public SDK package name.
pub const PROVIDER_SDK_PACKAGE: &str = "omarchygs-provider-sdk";
/// Public SDK package version.
pub const PROVIDER_SDK_PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

const LOCK_PATH: &str = "sdk-lock.json";
const RELEASE_PATH: &str = "sdk-release.json";
const RELEASE_DOMAIN: &[u8] = b"omarchygs-provider-sdk-release-v1\0";
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_DEPTH: usize = 5;
const MAX_INVENTORY_ENTRIES: usize = 64;
const MAX_INVENTORY_PATH_BYTES: usize = 4 * 1024;

const STATIC_FILES: &[(&str, &[u8])] = &[
    ("Cargo.toml", include_bytes!("../sdk/v1/Cargo.toml.txt")),
    ("README.md", include_bytes!("../sdk/v1/README.md")),
    ("LICENSES.md", include_bytes!("../sdk/v1/LICENSES.md")),
    ("src/lib.rs", include_bytes!("lib.rs")),
    ("src/model.rs", include_bytes!("model.rs")),
    ("src/protocol.rs", include_bytes!("protocol.rs")),
    ("src/release.rs", include_bytes!("release.rs")),
    (
        "sdk/v1/Cargo.toml.txt",
        include_bytes!("../sdk/v1/Cargo.toml.txt"),
    ),
    ("sdk/v1/README.md", include_bytes!("../sdk/v1/README.md")),
    (
        "sdk/v1/LICENSES.md",
        include_bytes!("../sdk/v1/LICENSES.md"),
    ),
    (
        "sdk/v1/schemas/compatibility-offer.schema.json",
        include_bytes!("../sdk/v1/schemas/compatibility-offer.schema.json"),
    ),
    (
        "sdk/v1/schemas/compatibility-selection.schema.json",
        include_bytes!("../sdk/v1/schemas/compatibility-selection.schema.json"),
    ),
    (
        "sdk/v1/schemas/provider-grant.schema.json",
        include_bytes!("../sdk/v1/schemas/provider-grant.schema.json"),
    ),
    (
        "sdk/v1/schemas/provider-message.schema.json",
        include_bytes!("../sdk/v1/schemas/provider-message.schema.json"),
    ),
    (
        "sdk/v1/schemas/sdk-lock.schema.json",
        include_bytes!("../sdk/v1/schemas/sdk-lock.schema.json"),
    ),
    (
        "sdk/v1/schemas/sdk-release.schema.json",
        include_bytes!("../sdk/v1/schemas/sdk-release.schema.json"),
    ),
    (
        "sdk/v1/fixtures/compatibility-offer.json",
        include_bytes!("../sdk/v1/fixtures/compatibility-offer.json"),
    ),
    (
        "sdk/v1/fixtures/compatibility-selection.json",
        include_bytes!("../sdk/v1/fixtures/compatibility-selection.json"),
    ),
    (
        "sdk/v1/fixtures/reject-downgrade-offer.json",
        include_bytes!("../sdk/v1/fixtures/reject-downgrade-offer.json"),
    ),
];

/// One exact file pinned by the SDK lock.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderSdkFilePin {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

/// Canonical identity of every compiled-owned SDK artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderSdkLock {
    pub format: String,
    pub sdk_version: u32,
    pub package: String,
    pub package_version: String,
    pub compatibility: ProviderCompatibility,
    pub files: Vec<ProviderSdkFilePin>,
}

/// Authenticated SDK release provenance payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderSdkReleasePayload {
    pub format: String,
    pub authority: String,
    pub key_id: String,
    pub sdk_version: u32,
    pub lock_sha256: String,
    pub source_revision: String,
    pub builder_sha256: String,
}

/// Domain-separated signed SDK release envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedProviderSdkRelease {
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

/// Verified identity returned by export and verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderSdkIdentity {
    pub sdk_version: u32,
    pub compatibility: ProviderCompatibility,
    pub lock_sha256: String,
    pub release_sha256: String,
    pub source_revision: String,
    pub builder_sha256: String,
}

/// Local SDK release signer. Secret seed material is never serialized.
pub struct ProviderSdkReleaseSigner {
    authority: String,
    key_id: String,
    signing_key: SigningKey,
}

impl ProviderSdkReleaseSigner {
    /// Construct one project-controlled release signer from local secret bytes.
    pub fn new(authority: &str, key_id: &str, signing_seed: [u8; 32]) -> Result<Self> {
        if !is_identifier(authority, 3, 64, b"._-") || !is_identifier(key_id, 3, 64, b"._-") {
            return Err(ProviderError::InvalidInput);
        }
        Ok(Self {
            authority: authority.to_owned(),
            key_id: key_id.to_owned(),
            signing_key: SigningKey::from_bytes(&signing_seed),
        })
    }

    /// Public key used to verify an exported release envelope.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

/// Export one exact SDK into an existing empty directory and sign its identity.
pub fn export_sdk(
    output: &Path,
    signer: &ProviderSdkReleaseSigner,
    source_revision: &str,
    builder_sha256: &str,
) -> Result<ProviderSdkIdentity> {
    validate_provenance(source_revision, builder_sha256)?;
    require_empty_directory(output)?;
    for (relative, bytes) in STATIC_FILES {
        write_new_read_only(&output.join(relative), bytes)?;
    }
    let lock = expected_lock();
    let lock_bytes = canonical_json(&lock)?;
    write_new_read_only(&output.join(LOCK_PATH), &lock_bytes)?;
    let lock_sha256 = sha256_hex(&lock_bytes);
    let payload = ProviderSdkReleasePayload {
        format: "omarchygs.provider-sdk-release-payload/v1".to_owned(),
        authority: signer.authority.clone(),
        key_id: signer.key_id.clone(),
        sdk_version: PROVIDER_SDK_VERSION,
        lock_sha256: lock_sha256.clone(),
        source_revision: source_revision.to_owned(),
        builder_sha256: builder_sha256.to_owned(),
    };
    let payload_bytes = canonical_json(&payload)?;
    let mut signed = Vec::with_capacity(RELEASE_DOMAIN.len() + payload_bytes.len());
    signed.extend_from_slice(RELEASE_DOMAIN);
    signed.extend_from_slice(&payload_bytes);
    let envelope = SignedProviderSdkRelease {
        key_id: signer.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signer.signing_key.sign(&signed).to_bytes()),
    };
    let release_bytes = canonical_json(&envelope)?;
    write_new_read_only(&output.join(RELEASE_PATH), &release_bytes)?;
    sync_directories(output)?;
    Ok(identity(&lock, &payload, &release_bytes))
}

/// Verify exact exported bytes, provenance, and project release signature.
pub fn verify_sdk_directory(
    root: &Path,
    verifying_key: &VerifyingKey,
    expected_authority: &str,
    expected_key_id: &str,
) -> Result<ProviderSdkIdentity> {
    if !is_identifier(expected_authority, 3, 64, b"._-")
        || !is_identifier(expected_key_id, 3, 64, b"._-")
    {
        return Err(ProviderError::InvalidInput);
    }
    require_directory(root)?;
    let mut expected_files = STATIC_FILES
        .iter()
        .map(|(path, _)| PathBuf::from(path))
        .collect::<BTreeSet<_>>();
    expected_files.insert(PathBuf::from(LOCK_PATH));
    expected_files.insert(PathBuf::from(RELEASE_PATH));
    let expected_directories = expected_parent_directories(&expected_files);
    let (actual_files, actual_directories) =
        collect_paths(root, &expected_files, &expected_directories)?;
    if actual_files != expected_files || actual_directories != expected_directories {
        return Err(ProviderError::ProtocolRejected);
    }
    for (relative, expected) in STATIC_FILES {
        let actual = read_bounded_regular_file(&root.join(relative))?;
        if actual != *expected {
            return Err(ProviderError::ProtocolRejected);
        }
        if relative.ends_with(".json") {
            let _: serde_json::Value =
                serde_json::from_slice(&actual).map_err(|_| ProviderError::ProtocolRejected)?;
        }
    }
    let lock_bytes = read_bounded_regular_file(&root.join(LOCK_PATH))?;
    let lock: ProviderSdkLock =
        serde_json::from_slice(&lock_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    if canonical_json(&lock)? != lock_bytes || lock != expected_lock() {
        return Err(ProviderError::ProtocolRejected);
    }
    let release_bytes = read_bounded_regular_file(&root.join(RELEASE_PATH))?;
    let envelope: SignedProviderSdkRelease =
        serde_json::from_slice(&release_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    if canonical_json(&envelope)? != release_bytes
        || envelope.key_id != expected_key_id
        || envelope.payload.len() > 4_096
        || envelope.signature.len() > 128
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(&envelope.payload)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&envelope.signature)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    let mut signed = Vec::with_capacity(RELEASE_DOMAIN.len() + payload_bytes.len());
    signed.extend_from_slice(RELEASE_DOMAIN);
    signed.extend_from_slice(&payload_bytes);
    verifying_key
        .verify(&signed, &signature)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let payload: ProviderSdkReleasePayload =
        serde_json::from_slice(&payload_bytes).map_err(|_| ProviderError::ProtocolRejected)?;
    validate_provenance(&payload.source_revision, &payload.builder_sha256)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if canonical_json(&payload)? != payload_bytes
        || payload.format != "omarchygs.provider-sdk-release-payload/v1"
        || payload.authority != expected_authority
        || payload.key_id != expected_key_id
        || payload.sdk_version != PROVIDER_SDK_VERSION
        || payload.lock_sha256 != sha256_hex(&lock_bytes)
    {
        return Err(ProviderError::ProtocolRejected);
    }
    Ok(identity(&lock, &payload, &release_bytes))
}

fn expected_lock() -> ProviderSdkLock {
    ProviderSdkLock {
        format: "omarchygs.provider-sdk-lock/v1".to_owned(),
        sdk_version: PROVIDER_SDK_VERSION,
        package: PROVIDER_SDK_PACKAGE.to_owned(),
        package_version: PROVIDER_SDK_PACKAGE_VERSION.to_owned(),
        compatibility: ProviderCompatibility::current(),
        files: STATIC_FILES
            .iter()
            .map(|(path, bytes)| ProviderSdkFilePin {
                path: (*path).to_owned(),
                bytes: bytes.len() as u64,
                sha256: sha256_hex(bytes),
            })
            .collect(),
    }
}

fn identity(
    lock: &ProviderSdkLock,
    payload: &ProviderSdkReleasePayload,
    release_bytes: &[u8],
) -> ProviderSdkIdentity {
    ProviderSdkIdentity {
        sdk_version: lock.sdk_version,
        compatibility: lock.compatibility.clone(),
        lock_sha256: payload.lock_sha256.clone(),
        release_sha256: sha256_hex(release_bytes),
        source_revision: payload.source_revision.clone(),
        builder_sha256: payload.builder_sha256.clone(),
    }
}

fn validate_provenance(source_revision: &str, builder_sha256: &str) -> Result<()> {
    if source_revision.len() != 40
        || !source_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || !is_sha256_hex(builder_sha256)
    {
        Err(ProviderError::InvalidInput)
    } else {
        Ok(())
    }
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| ProviderError::Internal)
}

fn require_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ProviderError::Internal)?;
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        Ok(())
    } else {
        Err(ProviderError::ProtocolRejected)
    }
}

fn require_empty_directory(path: &Path) -> Result<()> {
    require_directory(path)?;
    if fs::read_dir(path)
        .map_err(|_| ProviderError::Internal)?
        .next()
        .transpose()
        .map_err(|_| ProviderError::Internal)?
        .is_none()
    {
        Ok(())
    } else {
        Err(ProviderError::InvalidInput)
    }
}

fn write_new_read_only(path: &Path, bytes: &[u8]) -> Result<()> {
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

fn read_bounded_regular_file(path: &Path) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ProviderError::ProtocolRejected)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_FILE_BYTES
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let file = File::open(path).map_err(|_| ProviderError::ProtocolRejected)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(ProviderError::ProtocolRejected);
    }
    Ok(bytes)
}

fn expected_parent_directories(files: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
    let mut directories = BTreeSet::new();
    for file in files {
        let mut parent = file.parent();
        while let Some(directory) = parent {
            if directory.as_os_str().is_empty() {
                break;
            }
            directories.insert(directory.to_path_buf());
            parent = directory.parent();
        }
    }
    directories
}

fn collect_paths(
    root: &Path,
    expected_files: &BTreeSet<PathBuf>,
    expected_directories: &BTreeSet<PathBuf>,
) -> Result<(BTreeSet<PathBuf>, BTreeSet<PathBuf>)> {
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    let mut pending = vec![(root.to_path_buf(), 0_usize)];
    let mut entry_count = 0_usize;
    let mut path_bytes = 0_usize;
    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DEPTH {
            return Err(ProviderError::ProtocolRejected);
        }
        for entry in fs::read_dir(&directory).map_err(|_| ProviderError::ProtocolRejected)? {
            let entry = entry.map_err(|_| ProviderError::ProtocolRejected)?;
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|_| ProviderError::ProtocolRejected)?
                .to_path_buf();
            entry_count = entry_count
                .checked_add(1)
                .ok_or(ProviderError::ProtocolRejected)?;
            path_bytes = path_bytes
                .checked_add(relative.as_os_str().len())
                .ok_or(ProviderError::ProtocolRejected)?;
            if entry_count > MAX_INVENTORY_ENTRIES || path_bytes > MAX_INVENTORY_PATH_BYTES {
                return Err(ProviderError::ProtocolRejected);
            }
            let metadata =
                fs::symlink_metadata(&path).map_err(|_| ProviderError::ProtocolRejected)?;
            if metadata.file_type().is_symlink() {
                return Err(ProviderError::ProtocolRejected);
            }
            if metadata.file_type().is_dir() {
                if !expected_directories.contains(&relative)
                    || !directories.insert(relative.clone())
                {
                    return Err(ProviderError::ProtocolRejected);
                }
                pending.push((path, depth + 1));
            } else if metadata.file_type().is_file() {
                if !expected_files.contains(&relative) || !files.insert(relative) {
                    return Err(ProviderError::ProtocolRejected);
                }
            } else {
                return Err(ProviderError::ProtocolRejected);
            }
        }
    }
    Ok((files, directories))
}

fn sync_directories(root: &Path) -> Result<()> {
    let mut directories = BTreeSet::<PathBuf>::new();
    directories.insert(root.to_path_buf());
    for (relative, _) in STATIC_FILES {
        let mut parent = root.join(relative).parent().map(Path::to_path_buf);
        while let Some(path) = parent {
            if !path.starts_with(root) || !directories.insert(path.clone()) || path == root {
                break;
            }
            parent = path.parent().map(Path::to_path_buf);
        }
    }
    for directory in directories.iter().rev() {
        File::open(directory)
            .and_then(|file| file.sync_all())
            .map_err(|_| ProviderError::Internal)?;
    }
    Ok(())
}
