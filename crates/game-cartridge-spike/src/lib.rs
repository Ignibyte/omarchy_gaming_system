//! Isolated contracts for the Ticket 014 Game Cartridge architecture spike.
//!
//! This crate is deliberately outside the product workspace. It proves a
//! package, grant, provider, and presentation boundary without creating a
//! production remote-game API.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use rand_core::OsRng;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

pub const INTEGRITY_FILE: &str = "integrity.signed.json";
pub const PLATFORM_KEY_ID: &str = "platform-proof-v1";
pub const PROVIDER_KEY_ID: &str = "provider-proof-v1";
pub const PUBLISHER_KEY_ID: &str = "publisher-proof-v1";
pub const PLATFORM_ISSUER: &str = "omarchygs-spike";
pub const MAX_ENVELOPE_BYTES: usize = 128 * 1024;
pub const MAX_PROVIDER_BODY_BYTES: usize = 128 * 1024;
pub const MAX_CARTRIDGE_FILES: usize = 32;
pub const MAX_CARTRIDGE_ENTRIES: usize = 64;
pub const MAX_CARTRIDGE_DEPTH: usize = 1;
pub const MAX_CARTRIDGE_FILE_BYTES: u64 = 256 * 1024;
pub const MAX_CARTRIDGE_TOTAL_BYTES: u64 = 1024 * 1024;
pub const MAX_PRESENTATION_NODES: usize = 128;
pub const MAX_VIEW_BYTES: usize = 64 * 1024;
pub const MAX_GRANT_LIFETIME_SECONDS: i64 = 60;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum SpikeError {
    #[error("invalid key material")]
    InvalidKey,
    #[error("invalid signed envelope")]
    InvalidEnvelope,
    #[error("invalid cartridge")]
    InvalidCartridge,
    #[error("cartridge limit exceeded")]
    CartridgeLimit,
    #[error("invalid manifest")]
    InvalidManifest,
    #[error("invalid presentation")]
    InvalidPresentation,
    #[error("invalid view model")]
    InvalidView,
    #[error("invalid provider grant")]
    InvalidGrant,
    #[error("invalid provider message")]
    InvalidProviderMessage,
    #[error("invalid provider endpoint")]
    InvalidProviderEndpoint,
    #[error("I/O failure")]
    Io(#[from] std::io::Error),
    #[error("JSON failure")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedEnvelope {
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CartridgeManifest {
    pub format_version: u32,
    pub game_key: String,
    pub publisher_id: String,
    pub provider_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub sdk_min_version: u32,
    pub sdk_max_version: u32,
    pub display_name: String,
    pub entry_screen: String,
    pub required_capabilities: Vec<String>,
    pub optional_capabilities: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Presentation {
    pub format_version: u32,
    pub screens: Vec<Screen>,
    pub actions: Vec<ActionDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Screen {
    pub id: String,
    pub title: String,
    pub nodes: Vec<PresentationNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PresentationNode {
    Terminal {
        id: String,
        text_binding: String,
        accessible_label: String,
    },
    Grid {
        id: String,
        rows: u8,
        columns: u8,
        cells_binding: String,
        action: String,
        accessible_label: String,
    },
    Status {
        id: String,
        text_binding: String,
        accessible_label: String,
    },
}

impl PresentationNode {
    fn id(&self) -> &str {
        match self {
            Self::Terminal { id, .. } | Self::Grid { id, .. } | Self::Status { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActionDefinition {
    pub id: String,
    pub payload_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ViewModel {
    pub headline: String,
    pub board: Vec<String>,
    pub turn: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileDigest {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IntegrityIndex {
    pub format_version: u32,
    pub files: Vec<FileDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCartridge {
    pub manifest: CartridgeManifest,
    pub presentation: Presentation,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderGrant {
    pub issuer: String,
    pub audience: String,
    pub subject: String,
    pub provider_id: String,
    pub game_key: String,
    pub game_version: u32,
    pub cartridge_digest: String,
    pub platform_session_id: Uuid,
    pub issued_at: i64,
    pub expires_at: i64,
    pub token_id: Uuid,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderLaunchRequest {
    pub grant: SignedEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderCommandRequest {
    pub grant: SignedEnvelope,
    pub idempotency_key: Uuid,
    pub expected_revision: u64,
    pub action_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMessageKind {
    Launch,
    CommandResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderMessage {
    pub kind: ProviderMessageKind,
    pub provider_id: String,
    pub game_key: String,
    pub game_version: u32,
    pub cartridge_digest: String,
    pub platform_session_id: Uuid,
    pub provider_session_id: Uuid,
    pub event_id: Uuid,
    pub revision: u64,
    pub view: ViewModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProofResponse {
    pub status: String,
    pub title: String,
    pub detail: String,
    pub revision: u64,
    pub pairwise_subject_verified: bool,
    pub cartridge_digest: String,
    pub idempotent_replay: bool,
    pub duplicate_event_rejected: bool,
    pub raw_persona_disclosed: bool,
    pub device_token_disclosed: bool,
    pub database_access_disclosed: bool,
    pub presentation: Presentation,
    pub view: ViewModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ErrorDocument {
    pub code: String,
}

pub struct GrantExpectation<'a> {
    pub provider_id: &'a str,
    pub game_key: &'a str,
    pub game_version: u32,
    pub cartridge_digest: &'a str,
    pub platform_session_id: Uuid,
    pub required_scope: &'a str,
}

pub struct MessageExpectation<'a> {
    pub kind: ProviderMessageKind,
    pub provider_id: &'a str,
    pub game_key: &'a str,
    pub game_version: u32,
    pub cartridge_digest: &'a str,
    pub platform_session_id: Uuid,
}

pub fn now_unix_seconds() -> Result<i64, SpikeError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SpikeError::InvalidGrant)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| SpikeError::InvalidGrant)
}

pub fn generate_key_pair(private_path: &Path, public_path: &Path) -> Result<(), SpikeError> {
    let signing_key = SigningKey::generate(&mut OsRng);
    write_private_key(private_path, &signing_key.to_bytes())?;
    fs::write(
        public_path,
        format!(
            "{}\n",
            URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes())
        ),
    )?;
    Ok(())
}

#[cfg(unix)]
fn write_private_key(path: &Path, bytes: &[u8; 32]) -> Result<(), SpikeError> {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path)?;
    use std::io::Write as _;
    file.write_all(URL_SAFE_NO_PAD.encode(bytes).as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

#[cfg(not(unix))]
fn write_private_key(path: &Path, bytes: &[u8; 32]) -> Result<(), SpikeError> {
    fs::write(path, format!("{}\n", URL_SAFE_NO_PAD.encode(bytes)))?;
    Ok(())
}

pub fn load_signing_key(path: &Path) -> Result<SigningKey, SpikeError> {
    let bytes = read_key_bytes(path)?;
    Ok(SigningKey::from_bytes(&bytes))
}

pub fn load_verifying_key(path: &Path) -> Result<VerifyingKey, SpikeError> {
    let bytes = read_key_bytes(path)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| SpikeError::InvalidKey)
}

fn read_key_bytes(path: &Path) -> Result<[u8; 32], SpikeError> {
    let encoded = fs::read_to_string(path)?;
    let decoded = URL_SAFE_NO_PAD
        .decode(encoded.trim().as_bytes())
        .map_err(|_| SpikeError::InvalidKey)?;
    decoded.try_into().map_err(|_| SpikeError::InvalidKey)
}

pub fn sign_envelope<T: Serialize>(
    value: &T,
    key_id: &str,
    signing_key: &SigningKey,
) -> Result<SignedEnvelope, SpikeError> {
    validate_key_id(key_id)?;
    let payload = serde_json::to_vec(value)?;
    if payload.len() > MAX_ENVELOPE_BYTES {
        return Err(SpikeError::InvalidEnvelope);
    }
    let message = envelope_message(key_id, &payload);
    let signature = signing_key.sign(&message);
    Ok(SignedEnvelope {
        key_id: key_id.to_owned(),
        payload: URL_SAFE_NO_PAD.encode(payload),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

pub fn verify_envelope<T: DeserializeOwned>(
    envelope: &SignedEnvelope,
    expected_key_id: &str,
    verifying_key: &VerifyingKey,
) -> Result<T, SpikeError> {
    let (value, _) = verify_envelope_with_payload(envelope, expected_key_id, verifying_key)?;
    Ok(value)
}

fn verify_envelope_with_payload<T: DeserializeOwned>(
    envelope: &SignedEnvelope,
    expected_key_id: &str,
    verifying_key: &VerifyingKey,
) -> Result<(T, Vec<u8>), SpikeError> {
    if envelope.key_id != expected_key_id {
        return Err(SpikeError::InvalidEnvelope);
    }
    validate_key_id(&envelope.key_id)?;
    let payload = URL_SAFE_NO_PAD
        .decode(envelope.payload.as_bytes())
        .map_err(|_| SpikeError::InvalidEnvelope)?;
    if payload.len() > MAX_ENVELOPE_BYTES {
        return Err(SpikeError::InvalidEnvelope);
    }
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(envelope.signature.as_bytes())
        .map_err(|_| SpikeError::InvalidEnvelope)?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|_| SpikeError::InvalidEnvelope)?;
    verifying_key
        .verify(&envelope_message(&envelope.key_id, &payload), &signature)
        .map_err(|_| SpikeError::InvalidEnvelope)?;
    let value = serde_json::from_slice(&payload).map_err(|_| SpikeError::InvalidEnvelope)?;
    Ok((value, payload))
}

fn envelope_message(key_id: &str, payload: &[u8]) -> Vec<u8> {
    let mut message = b"omarchygs-spike-envelope-v1\0".to_vec();
    message.extend_from_slice(key_id.as_bytes());
    message.push(0);
    message.extend_from_slice(payload);
    message
}

fn validate_key_id(value: &str) -> Result<(), SpikeError> {
    if is_canonical_identifier(value, 3, 64) {
        Ok(())
    } else {
        Err(SpikeError::InvalidKey)
    }
}

pub fn sign_cartridge(
    directory: &Path,
    key_id: &str,
    signing_key: &SigningKey,
) -> Result<String, SpikeError> {
    let index = build_integrity_index(directory)?;
    let envelope = sign_envelope(&index, key_id, signing_key)?;
    let bytes = serde_json::to_vec_pretty(&envelope)?;
    fs::write(directory.join(INTEGRITY_FILE), bytes)?;
    Ok(cartridge_digest(&envelope.payload))
}

pub fn verify_cartridge(
    directory: &Path,
    expected_key_id: &str,
    verifying_key: &VerifyingKey,
) -> Result<VerifiedCartridge, SpikeError> {
    let envelope_bytes = read_limited(&directory.join(INTEGRITY_FILE), MAX_ENVELOPE_BYTES as u64)?;
    let envelope: SignedEnvelope =
        serde_json::from_slice(&envelope_bytes).map_err(|_| SpikeError::InvalidCartridge)?;
    let (index, _): (IntegrityIndex, Vec<u8>) =
        verify_envelope_with_payload(&envelope, expected_key_id, verifying_key)
            .map_err(|_| SpikeError::InvalidCartridge)?;
    if index.format_version != 1
        || index.files.is_empty()
        || index.files.len() > MAX_CARTRIDGE_FILES
        || index
            .files
            .windows(2)
            .any(|pair| pair[0].path >= pair[1].path)
    {
        return Err(SpikeError::InvalidCartridge);
    }
    let (actual, contents) = read_cartridge_files(directory)?;
    if actual != index {
        return Err(SpikeError::InvalidCartridge);
    }

    let manifest_bytes = contents
        .get("manifest.json")
        .ok_or(SpikeError::InvalidManifest)?;
    let manifest: CartridgeManifest =
        serde_json::from_slice(manifest_bytes).map_err(|_| SpikeError::InvalidManifest)?;
    validate_manifest(&manifest)?;
    let presentation_bytes = contents
        .get("presentation.json")
        .ok_or(SpikeError::InvalidPresentation)?;
    let presentation: Presentation =
        serde_json::from_slice(presentation_bytes).map_err(|_| SpikeError::InvalidPresentation)?;
    validate_presentation(&manifest, &presentation)?;

    Ok(VerifiedCartridge {
        manifest,
        presentation,
        digest: cartridge_digest(&envelope.payload),
    })
}

pub fn build_integrity_index(directory: &Path) -> Result<IntegrityIndex, SpikeError> {
    let (index, _) = read_cartridge_files(directory)?;
    Ok(index)
}

fn read_cartridge_files(
    directory: &Path,
) -> Result<(IntegrityIndex, BTreeMap<String, Vec<u8>>), SpikeError> {
    let metadata = fs::symlink_metadata(directory)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(SpikeError::InvalidCartridge);
    }

    let mut paths = Vec::new();
    let mut entries = 0_usize;
    collect_cartridge_paths(directory, directory, &mut paths, &mut entries, 0)?;
    paths.sort();
    if paths.is_empty() || paths.len() > MAX_CARTRIDGE_FILES {
        return Err(SpikeError::CartridgeLimit);
    }

    let mut total = 0_u64;
    let mut files = Vec::with_capacity(paths.len());
    let mut contents = BTreeMap::new();
    for (relative, absolute) in paths {
        let metadata = fs::symlink_metadata(&absolute)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(SpikeError::InvalidCartridge);
        }
        if metadata.len() > MAX_CARTRIDGE_FILE_BYTES {
            return Err(SpikeError::CartridgeLimit);
        }
        let bytes = read_limited(&absolute, MAX_CARTRIDGE_FILE_BYTES)?;
        let size = u64::try_from(bytes.len()).map_err(|_| SpikeError::CartridgeLimit)?;
        total = total.checked_add(size).ok_or(SpikeError::CartridgeLimit)?;
        if total > MAX_CARTRIDGE_TOTAL_BYTES {
            return Err(SpikeError::CartridgeLimit);
        }
        files.push(FileDigest {
            path: relative.clone(),
            size,
            sha256: URL_SAFE_NO_PAD.encode(Sha256::digest(&bytes)),
        });
        contents.insert(relative, bytes);
    }
    Ok((
        IntegrityIndex {
            format_version: 1,
            files,
        },
        contents,
    ))
}

fn collect_cartridge_paths(
    root: &Path,
    current: &Path,
    output: &mut Vec<(String, PathBuf)>,
    entries: &mut usize,
    depth: usize,
) -> Result<(), SpikeError> {
    for entry in fs::read_dir(current)? {
        *entries = entries.checked_add(1).ok_or(SpikeError::CartridgeLimit)?;
        if *entries > MAX_CARTRIDGE_ENTRIES {
            return Err(SpikeError::CartridgeLimit);
        }
        let entry = entry?;
        let absolute = entry.path();
        let metadata = fs::symlink_metadata(&absolute)?;
        if metadata.file_type().is_symlink() {
            return Err(SpikeError::InvalidCartridge);
        }
        if metadata.is_dir() {
            if depth >= MAX_CARTRIDGE_DEPTH {
                return Err(SpikeError::CartridgeLimit);
            }
            let relative = safe_relative_path(
                absolute
                    .strip_prefix(root)
                    .map_err(|_| SpikeError::InvalidCartridge)?,
            )?;
            if !matches!(relative.as_str(), "assets" | "locales" | "schemas") {
                return Err(SpikeError::InvalidCartridge);
            }
            collect_cartridge_paths(root, &absolute, output, entries, depth + 1)?;
        } else if metadata.is_file() {
            let relative_path = absolute
                .strip_prefix(root)
                .map_err(|_| SpikeError::InvalidCartridge)?;
            let relative = safe_relative_path(relative_path)?;
            if relative != INTEGRITY_FILE {
                validate_cartridge_member(&relative)?;
                output.push((relative, absolute));
            }
        } else {
            return Err(SpikeError::InvalidCartridge);
        }
    }
    Ok(())
}

fn validate_cartridge_member(path: &str) -> Result<(), SpikeError> {
    let allowed = matches!(path, "manifest.json" | "presentation.json")
        || path
            .strip_prefix("schemas/")
            .is_some_and(|name| !name.contains('/') && name.ends_with(".schema.json"))
        || path
            .strip_prefix("locales/")
            .is_some_and(|name| !name.contains('/') && name.ends_with(".json"))
        || path.strip_prefix("assets/").is_some_and(|name| {
            !name.contains('/')
                && [".png", ".qoi", ".ogg", ".wav"]
                    .iter()
                    .any(|extension| name.ends_with(extension))
        });
    if allowed {
        Ok(())
    } else {
        Err(SpikeError::InvalidCartridge)
    }
}

fn safe_relative_path(path: &Path) -> Result<String, SpikeError> {
    if path.is_absolute() {
        return Err(SpikeError::InvalidCartridge);
    }
    let mut segments = Vec::new();
    for component in path.components() {
        let Component::Normal(segment) = component else {
            return Err(SpikeError::InvalidCartridge);
        };
        let segment = segment.to_str().ok_or(SpikeError::InvalidCartridge)?;
        if segment.is_empty()
            || segment.len() > 96
            || segment.starts_with('.')
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(SpikeError::InvalidCartridge);
        }
        segments.push(segment);
    }
    if segments.is_empty() {
        Err(SpikeError::InvalidCartridge)
    } else {
        Ok(segments.join("/"))
    }
}

fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, SpikeError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit {
        return Err(SpikeError::CartridgeLimit);
    }
    let file = fs::File::open(path)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        Err(SpikeError::CartridgeLimit)
    } else {
        Ok(bytes)
    }
}

fn cartridge_digest(encoded_payload: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(encoded_payload.as_bytes()))
}

pub fn validate_manifest(manifest: &CartridgeManifest) -> Result<(), SpikeError> {
    let supported_capabilities = BTreeSet::from([
        "core.grid",
        "core.status",
        "core.terminal",
        "rich2d.animation",
    ]);
    let supported_permissions = BTreeSet::from(["game.command", "persona.display_name"]);
    let required = manifest
        .required_capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let optional = manifest
        .optional_capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let permissions = manifest
        .permissions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let display_name_is_valid = !manifest.display_name.is_empty()
        && manifest.display_name.chars().count() <= 64
        && manifest
            .display_name
            .chars()
            .all(|character| !character.is_control());

    if manifest.format_version != 1
        || !is_canonical_identifier(&manifest.game_key, 3, 32)
        || !is_canonical_identifier(&manifest.publisher_id, 3, 64)
        || !is_canonical_identifier(&manifest.provider_id, 3, 64)
        || manifest.rules_version == 0
        || manifest.cartridge_version == 0
        || manifest.sdk_min_version == 0
        || manifest.sdk_min_version > manifest.sdk_max_version
        || !display_name_is_valid
        || !is_canonical_identifier(&manifest.entry_screen, 1, 48)
        || required.len() != manifest.required_capabilities.len()
        || optional.len() != manifest.optional_capabilities.len()
        || permissions.len() != manifest.permissions.len()
        || !required.is_disjoint(&optional)
        || !required.is_subset(&supported_capabilities)
        || !optional.is_subset(&supported_capabilities)
        || !permissions.is_subset(&supported_permissions)
        || !required.contains("core.terminal")
        || !permissions.contains("game.command")
    {
        Err(SpikeError::InvalidManifest)
    } else {
        Ok(())
    }
}

pub fn validate_presentation(
    manifest: &CartridgeManifest,
    presentation: &Presentation,
) -> Result<(), SpikeError> {
    if presentation.format_version != 1
        || presentation.screens.is_empty()
        || presentation.screens.len() > 8
        || presentation.actions.is_empty()
        || presentation.actions.len() > 32
    {
        return Err(SpikeError::InvalidPresentation);
    }
    let action_ids = presentation
        .actions
        .iter()
        .map(|action| action.id.as_str())
        .collect::<BTreeSet<_>>();
    if action_ids.len() != presentation.actions.len()
        || presentation.actions.iter().any(|action| {
            !is_canonical_identifier(&action.id, 1, 48)
                || action.payload_fields.len() > 16
                || action
                    .payload_fields
                    .iter()
                    .any(|field| !is_canonical_identifier(field, 1, 48))
        })
    {
        return Err(SpikeError::InvalidPresentation);
    }

    let mut screen_ids = HashSet::new();
    let mut node_count = 0_usize;
    for screen in &presentation.screens {
        if !is_canonical_identifier(&screen.id, 1, 48)
            || !screen_ids.insert(screen.id.as_str())
            || screen.title.is_empty()
            || screen.title.chars().count() > 64
            || screen.title.chars().any(char::is_control)
            || screen.nodes.is_empty()
        {
            return Err(SpikeError::InvalidPresentation);
        }
        let mut node_ids = HashSet::new();
        for node in &screen.nodes {
            node_count += 1;
            if node_count > MAX_PRESENTATION_NODES
                || !is_canonical_identifier(node.id(), 1, 48)
                || !node_ids.insert(node.id())
                || !validate_node(node, &action_ids)
            {
                return Err(SpikeError::InvalidPresentation);
            }
        }
    }
    if !screen_ids.contains(manifest.entry_screen.as_str()) {
        return Err(SpikeError::InvalidPresentation);
    }
    Ok(())
}

fn validate_node(node: &PresentationNode, actions: &BTreeSet<&str>) -> bool {
    match node {
        PresentationNode::Terminal {
            text_binding,
            accessible_label,
            ..
        }
        | PresentationNode::Status {
            text_binding,
            accessible_label,
            ..
        } => {
            is_canonical_identifier(text_binding, 1, 48) && valid_accessible_label(accessible_label)
        }
        PresentationNode::Grid {
            rows,
            columns,
            cells_binding,
            action,
            accessible_label,
            ..
        } => {
            (1..=16).contains(rows)
                && (1..=16).contains(columns)
                && is_canonical_identifier(cells_binding, 1, 48)
                && actions.contains(action.as_str())
                && valid_accessible_label(accessible_label)
        }
    }
}

fn valid_accessible_label(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= 96
        && value.chars().all(|character| !character.is_control())
}

pub fn validate_view_model(view: &ViewModel) -> Result<(), SpikeError> {
    let serialized = serde_json::to_vec(view)?;
    if serialized.len() > MAX_VIEW_BYTES
        || view.headline.is_empty()
        || view.headline.chars().count() > 256
        || view.headline.chars().any(char::is_control)
        || view.board.is_empty()
        || view.board.len() > 256
        || view.board.iter().any(|cell| {
            cell.chars().count() > 4 || cell.chars().any(|character| character.is_control())
        })
        || view.status.is_empty()
        || view.status.chars().count() > 64
        || view.status.chars().any(char::is_control)
    {
        Err(SpikeError::InvalidView)
    } else {
        Ok(())
    }
}

pub fn pairwise_subject(
    secret: &[u8],
    provider_id: &str,
    game_key: &str,
    persona_id: Uuid,
) -> Result<String, SpikeError> {
    if secret.len() < 32
        || !is_canonical_identifier(provider_id, 3, 64)
        || !is_canonical_identifier(game_key, 3, 32)
    {
        return Err(SpikeError::InvalidGrant);
    }
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| SpikeError::InvalidGrant)?;
    mac.update(b"omarchygs-pairwise-subject-v1\0");
    mac.update(provider_id.as_bytes());
    mac.update(&[0]);
    mac.update(game_key.as_bytes());
    mac.update(&[0]);
    mac.update(persona_id.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

pub fn validate_grant(
    grant: &ProviderGrant,
    expected: &GrantExpectation<'_>,
    now: i64,
) -> Result<(), SpikeError> {
    let scopes = grant
        .scopes
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if grant.issuer != PLATFORM_ISSUER
        || grant.audience != expected.provider_id
        || grant.provider_id != expected.provider_id
        || grant.game_key != expected.game_key
        || grant.game_version != expected.game_version
        || grant.cartridge_digest != expected.cartridge_digest
        || grant.platform_session_id != expected.platform_session_id
        || grant.subject.len() != 43
        || grant.issued_at > now + 5
        || grant.expires_at <= now
        || grant.expires_at <= grant.issued_at
        || grant.expires_at - grant.issued_at > MAX_GRANT_LIFETIME_SECONDS
        || scopes.len() != grant.scopes.len()
        || scopes.len() != 1
        || !scopes.contains(expected.required_scope)
        || grant
            .scopes
            .iter()
            .any(|scope| !matches!(scope.as_str(), "game.launch" | "game.command"))
    {
        Err(SpikeError::InvalidGrant)
    } else {
        Ok(())
    }
}

pub fn validate_provider_message(
    message: &ProviderMessage,
    expected: &MessageExpectation<'_>,
) -> Result<(), SpikeError> {
    validate_view_model(&message.view)?;
    if message.kind != expected.kind
        || message.provider_id != expected.provider_id
        || message.game_key != expected.game_key
        || message.game_version != expected.game_version
        || message.cartridge_digest != expected.cartridge_digest
        || message.platform_session_id != expected.platform_session_id
        || message.provider_session_id.is_nil()
        || message.event_id.is_nil()
    {
        Err(SpikeError::InvalidProviderMessage)
    } else {
        Ok(())
    }
}

pub fn validate_spike_provider_endpoint(raw: &str) -> Result<Url, SpikeError> {
    let url = Url::parse(raw).map_err(|_| SpikeError::InvalidProviderEndpoint)?;
    let host_is_loopback = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if url.scheme() != "http"
        || !host_is_loopback
        || url.port().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        Err(SpikeError::InvalidProviderEndpoint)
    } else {
        Ok(url)
    }
}

fn is_canonical_identifier(value: &str, minimum: usize, maximum: usize) -> bool {
    let bytes = value.as_bytes();
    (minimum..=maximum).contains(&bytes.len())
        && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn signing_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn fixture_manifest() -> CartridgeManifest {
        CartridgeManifest {
            format_version: 1,
            game_key: "retro-grid".to_owned(),
            publisher_id: "ignibyte".to_owned(),
            provider_id: "fixture-provider".to_owned(),
            rules_version: 1,
            cartridge_version: 1,
            sdk_min_version: 1,
            sdk_max_version: 1,
            display_name: "Retro Grid".to_owned(),
            entry_screen: "main".to_owned(),
            required_capabilities: vec!["core.grid".to_owned(), "core.terminal".to_owned()],
            optional_capabilities: vec!["rich2d.animation".to_owned()],
            permissions: vec!["game.command".to_owned(), "persona.display_name".to_owned()],
        }
    }

    fn fixture_presentation() -> Presentation {
        Presentation {
            format_version: 1,
            screens: vec![Screen {
                id: "main".to_owned(),
                title: "RETRO GRID".to_owned(),
                nodes: vec![
                    PresentationNode::Terminal {
                        id: "headline".to_owned(),
                        text_binding: "headline".to_owned(),
                        accessible_label: "Game headline".to_owned(),
                    },
                    PresentationNode::Grid {
                        id: "board".to_owned(),
                        rows: 3,
                        columns: 3,
                        cells_binding: "board".to_owned(),
                        action: "advance".to_owned(),
                        accessible_label: "Game board".to_owned(),
                    },
                ],
            }],
            actions: vec![ActionDefinition {
                id: "advance".to_owned(),
                payload_fields: Vec::new(),
            }],
        }
    }

    fn write_fixture(directory: &Path) {
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec_pretty(&fixture_manifest()).expect("manifest serializes"),
        )
        .expect("manifest writes");
        fs::write(
            directory.join("presentation.json"),
            serde_json::to_vec_pretty(&fixture_presentation()).expect("presentation serializes"),
        )
        .expect("presentation writes");
    }

    #[test]
    fn signed_envelopes_reject_tampering_and_wrong_keys() {
        let key = signing_key(7);
        let wrong = signing_key(8);
        let value = ViewModel {
            headline: "Ready".to_owned(),
            board: vec![".".to_owned(); 9],
            turn: 0,
            status: "active".to_owned(),
        };
        let envelope = sign_envelope(&value, PLATFORM_KEY_ID, &key).expect("envelope signs");
        let verified: ViewModel = verify_envelope(&envelope, PLATFORM_KEY_ID, &key.verifying_key())
            .expect("matching envelope verifies");
        assert_eq!(verified, value);
        assert!(
            verify_envelope::<ViewModel>(&envelope, PLATFORM_KEY_ID, &wrong.verifying_key())
                .is_err()
        );
        let mut tampered = envelope;
        tampered.payload.push('A');
        assert!(
            verify_envelope::<ViewModel>(&tampered, PLATFORM_KEY_ID, &key.verifying_key()).is_err()
        );
    }

    #[test]
    fn cartridge_integrity_is_exact_and_executable_free() {
        let directory = TempDir::new().expect("temp directory creates");
        write_fixture(directory.path());
        let key = signing_key(11);
        let digest =
            sign_cartridge(directory.path(), PUBLISHER_KEY_ID, &key).expect("cartridge signs");
        let verified = verify_cartridge(directory.path(), PUBLISHER_KEY_ID, &key.verifying_key())
            .expect("cartridge verifies");
        assert_eq!(verified.digest, digest);
        assert_eq!(verified.manifest, fixture_manifest());
        assert_eq!(verified.presentation, fixture_presentation());

        fs::write(directory.path().join("presentation.json"), b"{}").expect("fixture tampers");
        assert!(
            verify_cartridge(directory.path(), PUBLISHER_KEY_ID, &key.verifying_key()).is_err()
        );

        for executable_name in ["game.qml", "script.js", "native.so", "redirect.url"] {
            let executable_directory = TempDir::new().expect("temp directory creates");
            write_fixture(executable_directory.path());
            fs::write(
                executable_directory.path().join(executable_name),
                b"untrusted",
            )
            .expect("executable fixture writes");
            assert!(matches!(
                sign_cartridge(executable_directory.path(), PUBLISHER_KEY_ID, &key),
                Err(SpikeError::InvalidCartridge)
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn cartridge_rejects_links_and_oversized_content() {
        use std::os::unix::fs::symlink;

        let directory = TempDir::new().expect("temp directory creates");
        write_fixture(directory.path());
        symlink("manifest.json", directory.path().join("alias.json"))
            .expect("fixture symlink creates");
        assert!(matches!(
            build_integrity_index(directory.path()),
            Err(SpikeError::InvalidCartridge)
        ));

        fs::remove_file(directory.path().join("alias.json")).expect("fixture symlink removes");
        fs::create_dir(directory.path().join("assets")).expect("asset directory creates");
        fs::write(
            directory.path().join("assets/large.png"),
            vec![0_u8; MAX_CARTRIDGE_FILE_BYTES as usize + 1],
        )
        .expect("oversized fixture writes");
        assert!(matches!(
            build_integrity_index(directory.path()),
            Err(SpikeError::CartridgeLimit)
        ));

        fs::remove_file(directory.path().join("assets/large.png"))
            .expect("oversized fixture removes");
        fs::create_dir(directory.path().join("assets/nested"))
            .expect("nested asset directory creates");
        assert!(matches!(
            build_integrity_index(directory.path()),
            Err(SpikeError::CartridgeLimit)
        ));
    }

    #[test]
    fn manifest_and_presentation_are_capability_bounded() {
        let manifest = fixture_manifest();
        let presentation = fixture_presentation();
        validate_manifest(&manifest).expect("fixture manifest validates");
        validate_presentation(&manifest, &presentation).expect("fixture presentation validates");

        let mut executable = manifest.clone();
        executable.required_capabilities.push("qml.eval".to_owned());
        assert!(validate_manifest(&executable).is_err());

        let mut unknown_action = presentation;
        if let PresentationNode::Grid { action, .. } = &mut unknown_action.screens[0].nodes[1] {
            *action = "shell".to_owned();
        }
        assert!(validate_presentation(&manifest, &unknown_action).is_err());
    }

    #[test]
    fn grants_are_pairwise_short_lived_scoped_and_audience_bound() {
        let persona = Uuid::new_v4();
        let first = pairwise_subject(&[3_u8; 32], "fixture-provider", "retro-grid", persona)
            .expect("pairwise subject derives");
        let second = pairwise_subject(&[3_u8; 32], "other-provider", "retro-grid", persona)
            .expect("second pairwise subject derives");
        assert_ne!(first, second);
        assert!(!first.contains(&persona.to_string()));

        let session = Uuid::new_v4();
        let now = 1_000;
        let grant = ProviderGrant {
            issuer: PLATFORM_ISSUER.to_owned(),
            audience: "fixture-provider".to_owned(),
            subject: first,
            provider_id: "fixture-provider".to_owned(),
            game_key: "retro-grid".to_owned(),
            game_version: 1,
            cartridge_digest: "digest".to_owned(),
            platform_session_id: session,
            issued_at: now,
            expires_at: now + 30,
            token_id: Uuid::new_v4(),
            scopes: vec!["game.launch".to_owned()],
        };
        let expected = GrantExpectation {
            provider_id: "fixture-provider",
            game_key: "retro-grid",
            game_version: 1,
            cartridge_digest: "digest",
            platform_session_id: session,
            required_scope: "game.launch",
        };
        validate_grant(&grant, &expected, now).expect("grant validates");

        let mut wrong_audience = grant.clone();
        wrong_audience.audience = "other-provider".to_owned();
        assert!(validate_grant(&wrong_audience, &expected, now).is_err());
        assert!(validate_grant(&grant, &expected, now + 31).is_err());
        assert!(validate_grant(&grant, &expected, grant.expires_at).is_err());
        let command_expected = GrantExpectation {
            required_scope: "game.command",
            ..expected
        };
        assert!(validate_grant(&grant, &command_expected, now).is_err());
        let mut over_scoped = grant.clone();
        over_scoped.scopes.push("game.command".to_owned());
        assert!(validate_grant(&over_scoped, &expected, now).is_err());
    }

    #[test]
    fn provider_endpoint_is_registered_loopback_only_in_the_spike() {
        assert!(validate_spike_provider_endpoint("http://127.0.0.1:19091/").is_ok());
        for rejected in [
            "https://games.example/",
            "http://169.254.169.254:80/",
            "http://127.0.0.1:19091/redirect",
            "http://user:pass@127.0.0.1:19091/",
            "http://127.0.0.1:19091/?target=internal",
        ] {
            assert!(validate_spike_provider_endpoint(rejected).is_err());
        }
    }

    #[test]
    fn view_models_are_bounded_and_control_safe() {
        let valid = ViewModel {
            headline: "Your move".to_owned(),
            board: vec![".".to_owned(); 9],
            turn: 1,
            status: "active".to_owned(),
        };
        validate_view_model(&valid).expect("valid view passes");
        let mut invalid = valid;
        invalid.headline = "prompt\u{0000}injection".to_owned();
        assert!(validate_view_model(&invalid).is_err());
    }
}
