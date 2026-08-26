use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    archive::sha256_hex,
    error::{CartridgeError, Result},
    io::read_bounded_regular_file,
    validate::canonical_json,
};

pub const SDK_VERSION: u32 = 1;
pub const PRESENTATION_PROTOCOL_VERSION: u32 = 1;
pub const CARTRIDGE_TOOL_NAME: &str = "omarchygs-cartridge";
pub const PREVIEW_TOOL_NAME: &str = "omarchygs-cartridge-preview";
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

const SDK_LOCK_PATH: &str = "sdk-lock.json";
const MAX_SDK_FILE_BYTES: u64 = 512 * 1024;
const STATIC_FILES: &[(&str, &[u8])] = &[
    ("README.md", include_bytes!("../sdk/v1/README.md")),
    (
        "schemas/cartridge-manifest.schema.json",
        include_bytes!("../sdk/v1/schemas/cartridge-manifest.schema.json"),
    ),
    (
        "schemas/presentation.schema.json",
        include_bytes!("../sdk/v1/schemas/presentation.schema.json"),
    ),
    (
        "schemas/view-schema.schema.json",
        include_bytes!("../sdk/v1/schemas/view-schema.schema.json"),
    ),
    (
        "schemas/release-attestation.schema.json",
        include_bytes!("../sdk/v1/schemas/release-attestation.schema.json"),
    ),
    (
        "schemas/catalog-policy.schema.json",
        include_bytes!("../sdk/v1/schemas/catalog-policy.schema.json"),
    ),
    (
        "schemas/sdk-lock.schema.json",
        include_bytes!("../sdk/v1/schemas/sdk-lock.schema.json"),
    ),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolPin {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SdkFilePin {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SdkLifecycle {
    pub current: Vec<u32>,
    pub deprecated: Vec<u32>,
    pub retired: Vec<u32>,
    pub deprecation_new_release: String,
    pub retirement_new_release: String,
    pub active_session_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SdkLock {
    pub format: String,
    pub sdk_version: u32,
    pub presentation_protocol_version: u32,
    pub cartridge_tool: ToolPin,
    pub preview_tool: ToolPin,
    pub lifecycle: SdkLifecycle,
    pub files: Vec<SdkFilePin>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SdkIdentity {
    pub sdk_version: u32,
    pub presentation_protocol_version: u32,
    pub lock_sha256: String,
    pub cartridge_tool: ToolPin,
    pub preview_tool: ToolPin,
}

pub fn export_sdk(output: &Path) -> Result<SdkIdentity> {
    require_empty_directory(output)?;
    let schema_dir = output.join("schemas");
    fs::create_dir(&schema_dir)?;
    for (relative, bytes) in STATIC_FILES {
        write_new_read_only(&output.join(relative), bytes)?;
    }
    let lock = expected_lock();
    let lock_bytes = canonical_json(&lock)?;
    write_new_read_only(&output.join(SDK_LOCK_PATH), &lock_bytes)?;
    File::open(&schema_dir)?.sync_all()?;
    File::open(output)?.sync_all()?;
    Ok(identity_from_lock(&lock, &lock_bytes))
}

pub fn verify_sdk_directory(root: &Path) -> Result<SdkIdentity> {
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    let actual_paths = collect_exact_paths(root)?;
    let mut expected_paths = STATIC_FILES
        .iter()
        .map(|(path, _)| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    expected_paths.insert(SDK_LOCK_PATH.to_owned());
    if actual_paths != expected_paths {
        return Err(CartridgeError::InvalidSdk);
    }
    for (relative, expected) in STATIC_FILES {
        let actual = read_bounded_regular_file(&root.join(relative), MAX_SDK_FILE_BYTES)?;
        if actual != *expected {
            return Err(CartridgeError::InvalidSdk);
        }
        if relative.ends_with(".json") {
            let _: serde_json::Value = serde_json::from_slice(&actual)?;
        }
    }
    let lock_bytes = read_bounded_regular_file(&root.join(SDK_LOCK_PATH), MAX_SDK_FILE_BYTES)?;
    let lock: SdkLock = serde_json::from_slice(&lock_bytes)?;
    if canonical_json(&lock)? != lock_bytes || lock != expected_lock() {
        return Err(CartridgeError::InvalidSdk);
    }
    Ok(identity_from_lock(&lock, &lock_bytes))
}

/// Identity of the exact SDK contract embedded in this production verifier.
///
/// A server-side marketplace consumer uses this instead of trusting an SDK
/// directory supplied by the marketplace or cartridge publisher.
pub fn supported_sdk_identity() -> Result<SdkIdentity> {
    let lock = expected_lock();
    let lock_bytes = canonical_json(&lock)?;
    Ok(identity_from_lock(&lock, &lock_bytes))
}

fn expected_lock() -> SdkLock {
    SdkLock {
        format: "omarchygs.cartridge-sdk-lock/v1".to_owned(),
        sdk_version: SDK_VERSION,
        presentation_protocol_version: PRESENTATION_PROTOCOL_VERSION,
        cartridge_tool: ToolPin {
            name: CARTRIDGE_TOOL_NAME.to_owned(),
            version: TOOL_VERSION.to_owned(),
        },
        preview_tool: ToolPin {
            name: PREVIEW_TOOL_NAME.to_owned(),
            version: TOOL_VERSION.to_owned(),
        },
        lifecycle: SdkLifecycle {
            current: vec![SDK_VERSION],
            deprecated: Vec::new(),
            retired: Vec::new(),
            deprecation_new_release: "allow_with_warning".to_owned(),
            retirement_new_release: "deny".to_owned(),
            active_session_policy: "signed_catalog_policy".to_owned(),
        },
        files: STATIC_FILES
            .iter()
            .map(|(path, bytes)| SdkFilePin {
                path: (*path).to_owned(),
                bytes: bytes.len() as u64,
                sha256: sha256_hex(bytes),
            })
            .collect(),
    }
}

fn identity_from_lock(lock: &SdkLock, lock_bytes: &[u8]) -> SdkIdentity {
    SdkIdentity {
        sdk_version: lock.sdk_version,
        presentation_protocol_version: lock.presentation_protocol_version,
        lock_sha256: sha256_hex(lock_bytes),
        cartridge_tool: lock.cartridge_tool.clone(),
        preview_tool: lock.preview_tool.clone(),
    }
}

fn require_empty_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    if fs::read_dir(path)?.next().transpose()?.is_some() {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    Ok(())
}

fn collect_exact_paths(root: &Path) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| CartridgeError::InvalidSdk)?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(CartridgeError::UnsafeFilesystemPath);
        }
        if name == "schemas" && metadata.file_type().is_dir() {
            for schema in fs::read_dir(entry.path())? {
                let schema = schema?;
                let schema_name = schema
                    .file_name()
                    .into_string()
                    .map_err(|_| CartridgeError::InvalidSdk)?;
                let schema_metadata = fs::symlink_metadata(schema.path())?;
                if !schema_metadata.file_type().is_file()
                    || schema_metadata.file_type().is_symlink()
                {
                    return Err(CartridgeError::UnsafeFilesystemPath);
                }
                paths.insert(format!("schemas/{schema_name}"));
            }
        } else if metadata.file_type().is_file() {
            paths.insert(name);
        } else {
            return Err(CartridgeError::InvalidSdk);
        }
    }
    Ok(paths)
}

fn write_new_read_only(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o444);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
