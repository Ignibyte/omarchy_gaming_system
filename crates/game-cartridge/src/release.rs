use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer};
use serde::{Deserialize, Serialize};

use crate::{
    ConformanceReport, HostProfile, PublisherPrivateKey, PublisherPublicKey, SignatureAlgorithm,
    VerifiedCartridge,
    archive::{pack_directory, sha256_hex, verify_archive_bytes},
    error::{CartridgeError, Result},
    io::read_bounded_regular_file,
    sdk::{CARTRIDGE_TOOL_NAME, SdkIdentity, TOOL_VERSION, verify_sdk_directory},
    validate::canonical_json,
};

pub const RELEASE_ARCHIVE_PATH: &str = "cartridge.ogsc";
pub const RELEASE_CONFORMANCE_PATH: &str = "conformance.json";
pub const RELEASE_ATTESTATION_PATH: &str = "release.signed.json";

const RELEASE_SIGNATURE_DOMAIN: &[u8] = b"omarchygs-cartridge-release-v1\0";
const MAX_RELEASE_JSON_BYTES: u64 = 512 * 1024;
const MAX_BUILDER_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuilderIdentity {
    pub name: String,
    pub version: String,
    pub binary_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReleasePayload {
    pub format: String,
    pub source_revision: String,
    pub builder: BuilderIdentity,
    pub sdk: SdkIdentity,
    pub publisher_id: String,
    pub key_id: String,
    pub game_key: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub conformance_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedReleaseAttestation {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseReport {
    pub report_format: String,
    pub ok: bool,
    pub source_revision: String,
    pub sdk_lock_sha256: String,
    pub archive_sha256: String,
    pub conformance_sha256: String,
    pub attestation_sha256: String,
    pub publisher_id: String,
    pub key_id: String,
    pub game_key: String,
    pub reproducible_inputs: bool,
    pub provider_contacted: bool,
    pub database_required: bool,
    pub platform_credentials_read: bool,
}

#[derive(Debug, Clone)]
pub struct VerifiedRelease {
    cartridge: VerifiedCartridge,
    payload: ReleasePayload,
    sdk: SdkIdentity,
    conformance_bytes: Vec<u8>,
    attestation_bytes: Vec<u8>,
}

impl VerifiedRelease {
    pub fn cartridge(&self) -> &VerifiedCartridge {
        &self.cartridge
    }

    pub fn payload(&self) -> &ReleasePayload {
        &self.payload
    }

    pub fn sdk(&self) -> &SdkIdentity {
        &self.sdk
    }

    pub fn conformance_bytes(&self) -> &[u8] {
        &self.conformance_bytes
    }

    pub fn attestation_bytes(&self) -> &[u8] {
        &self.attestation_bytes
    }

    pub fn report(&self) -> ReleaseReport {
        release_report(
            &self.payload,
            &self.conformance_bytes,
            &self.attestation_bytes,
        )
    }
}

pub fn builder_sha256(path: &Path) -> Result<String> {
    Ok(sha256_hex(&read_bounded_regular_file(
        path,
        MAX_BUILDER_BYTES,
    )?))
}

pub fn create_release(
    source: &Path,
    private_key: &PublisherPrivateKey,
    sdk_root: &Path,
    source_revision: &str,
    builder_binary_sha256: &str,
    host: &HostProfile,
    output: &Path,
) -> Result<ReleaseReport> {
    validate_source_revision(source_revision)?;
    if !valid_sha256(builder_binary_sha256) {
        return Err(CartridgeError::InvalidRelease);
    }
    require_empty_directory(output)?;
    let sdk = verify_sdk_directory(sdk_root)?;
    let archive = pack_directory(source, private_key)?;
    let public_key = private_key.public_key()?;
    let verified = verify_archive_bytes(&archive, &public_key, host)?;
    if !verified.compatibility().compatible {
        return Err(CartridgeError::Incompatible);
    }
    let conformance_bytes = canonical_json(&verified.conformance_report())?;
    let payload = ReleasePayload {
        format: "omarchygs.cartridge-release/v1".to_owned(),
        source_revision: source_revision.to_owned(),
        builder: BuilderIdentity {
            name: CARTRIDGE_TOOL_NAME.to_owned(),
            version: TOOL_VERSION.to_owned(),
            binary_sha256: builder_binary_sha256.to_owned(),
        },
        sdk,
        publisher_id: verified.manifest().publisher_id.clone(),
        key_id: private_key.key_id.clone(),
        game_key: verified.manifest().game_key.clone(),
        rules_version: verified.manifest().rules_version,
        cartridge_version: verified.manifest().cartridge_version,
        archive_sha256: verified.archive_sha256().to_owned(),
        signed_identity_sha256: verified.signed_identity_sha256().to_owned(),
        conformance_sha256: sha256_hex(&conformance_bytes),
    };
    let payload_bytes = canonical_json(&payload)?;
    let mut message = Vec::with_capacity(RELEASE_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(RELEASE_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    let signature = private_key.decode()?.sign(&message);
    let attestation = SignedReleaseAttestation {
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: private_key.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    };
    let attestation_bytes = canonical_json(&attestation)?;
    write_new_read_only(&output.join(RELEASE_ARCHIVE_PATH), &archive)?;
    write_new_read_only(&output.join(RELEASE_CONFORMANCE_PATH), &conformance_bytes)?;
    write_new_read_only(&output.join(RELEASE_ATTESTATION_PATH), &attestation_bytes)?;
    File::open(output)?.sync_all()?;
    Ok(release_report(
        &payload,
        &conformance_bytes,
        &attestation_bytes,
    ))
}

pub fn verify_release_directory(
    release_root: &Path,
    public_key: &PublisherPublicKey,
    sdk_root: &Path,
    host: &HostProfile,
) -> Result<VerifiedRelease> {
    validate_release_inventory(release_root)?;
    let sdk = verify_sdk_directory(sdk_root)?;
    let archive = read_bounded_regular_file(
        &release_root.join(RELEASE_ARCHIVE_PATH),
        crate::MAX_ARCHIVE_BYTES as u64,
    )?;
    let conformance_bytes = read_bounded_regular_file(
        &release_root.join(RELEASE_CONFORMANCE_PATH),
        MAX_RELEASE_JSON_BYTES,
    )?;
    let attestation_bytes = read_bounded_regular_file(
        &release_root.join(RELEASE_ATTESTATION_PATH),
        MAX_RELEASE_JSON_BYTES,
    )?;
    verify_release_components(
        &archive,
        &conformance_bytes,
        &attestation_bytes,
        public_key,
        &sdk,
        host,
    )
}

pub fn verify_release_components(
    archive: &[u8],
    conformance_bytes: &[u8],
    attestation_bytes: &[u8],
    public_key: &PublisherPublicKey,
    sdk: &SdkIdentity,
    host: &HostProfile,
) -> Result<VerifiedRelease> {
    if conformance_bytes.len() as u64 > MAX_RELEASE_JSON_BYTES
        || attestation_bytes.len() as u64 > MAX_RELEASE_JSON_BYTES
    {
        return Err(CartridgeError::LimitExceeded);
    }
    let cartridge = verify_archive_bytes(archive, public_key, host)?;
    if !cartridge.compatibility().compatible {
        return Err(CartridgeError::Incompatible);
    }
    let conformance: ConformanceReport = serde_json::from_slice(conformance_bytes)?;
    if canonical_json(&conformance)? != conformance_bytes
        || canonical_json(&cartridge.conformance_report())? != conformance_bytes
    {
        return Err(CartridgeError::InvalidRelease);
    }
    let attestation: SignedReleaseAttestation = serde_json::from_slice(attestation_bytes)?;
    if canonical_json(&attestation)? != attestation_bytes
        || attestation.algorithm != SignatureAlgorithm::Ed25519
        || attestation.key_id != public_key.key_id
    {
        return Err(CartridgeError::InvalidRelease);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(&attestation.payload)
        .map_err(|_| CartridgeError::InvalidRelease)?;
    if payload_bytes.len() as u64 > MAX_RELEASE_JSON_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&attestation.signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|_| CartridgeError::InvalidSignature)?;
    let mut message = Vec::with_capacity(RELEASE_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(RELEASE_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    public_key
        .decode()?
        .verify_strict(&message, &signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;
    let payload: ReleasePayload = serde_json::from_slice(&payload_bytes)?;
    if canonical_json(&payload)? != payload_bytes
        || payload.format != "omarchygs.cartridge-release/v1"
        || validate_source_revision(&payload.source_revision).is_err()
        || payload.builder.name != CARTRIDGE_TOOL_NAME
        || payload.builder.version != TOOL_VERSION
        || !valid_sha256(&payload.builder.binary_sha256)
        || payload.sdk != *sdk
        || payload.publisher_id != public_key.publisher_id
        || payload.key_id != public_key.key_id
        || payload.game_key != cartridge.manifest().game_key
        || payload.rules_version != cartridge.manifest().rules_version
        || payload.cartridge_version != cartridge.manifest().cartridge_version
        || payload.archive_sha256 != cartridge.archive_sha256()
        || payload.signed_identity_sha256 != cartridge.signed_identity_sha256()
        || payload.conformance_sha256 != sha256_hex(conformance_bytes)
    {
        return Err(CartridgeError::InvalidRelease);
    }
    Ok(VerifiedRelease {
        cartridge,
        payload,
        sdk: sdk.clone(),
        conformance_bytes: conformance_bytes.to_vec(),
        attestation_bytes: attestation_bytes.to_vec(),
    })
}

fn release_report(
    payload: &ReleasePayload,
    conformance_bytes: &[u8],
    attestation_bytes: &[u8],
) -> ReleaseReport {
    ReleaseReport {
        report_format: "omarchygs.cartridge.release-report/v1".to_owned(),
        ok: true,
        source_revision: payload.source_revision.clone(),
        sdk_lock_sha256: payload.sdk.lock_sha256.clone(),
        archive_sha256: payload.archive_sha256.clone(),
        conformance_sha256: sha256_hex(conformance_bytes),
        attestation_sha256: sha256_hex(attestation_bytes),
        publisher_id: payload.publisher_id.clone(),
        key_id: payload.key_id.clone(),
        game_key: payload.game_key.clone(),
        reproducible_inputs: true,
        provider_contacted: false,
        database_required: false,
        platform_credentials_read: false,
    }
}

fn validate_release_inventory(root: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    let expected = [
        RELEASE_ARCHIVE_PATH,
        RELEASE_ATTESTATION_PATH,
        RELEASE_CONFORMANCE_PATH,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| CartridgeError::InvalidRelease)?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(CartridgeError::UnsafeFilesystemPath);
        }
        actual.insert(name);
    }
    if actual != expected.into_iter().map(str::to_owned).collect() {
        return Err(CartridgeError::InvalidRelease);
    }
    Ok(())
}

fn validate_source_revision(value: &str) -> Result<()> {
    if !matches!(value.len(), 40 | 64)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CartridgeError::InvalidRelease);
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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
