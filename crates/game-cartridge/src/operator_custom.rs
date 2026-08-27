//! Server-scoped operator-custom cartridge provenance and acquisition.
//!
//! This contract deliberately carries no marketplace-review claim. The
//! operator signature authenticates one server's custom-content decision while
//! the existing publisher signature, lifecycle policy, host compatibility, and
//! inert cartridge verifier remain independently authoritative.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    AcquisitionServerAdmission, CatalogPolicy, CatalogPrivateKey, CatalogPublicKey, HostProfile,
    MAX_ARCHIVE_BYTES, MAX_JSON_BYTES, PublisherPublicKey, SdkIdentity, SignatureAlgorithm,
    VerifiedRelease,
    error::{CartridgeError, Result},
    keys::valid_identifier,
    lifecycle::verify_catalog_policy_bytes,
    release::verify_release_components,
    validate::canonical_json,
};

pub const OPERATOR_CUSTOM_RELEASE_FORMAT: &str = "omarchygs.operator-custom-release/v1";
pub const OPERATOR_CUSTOM_ACQUISITION_FORMAT: &str = "omarchygs.operator-custom-acquisition/v1";
pub const OPERATOR_CUSTOM_WARNING: &str =
    "Operator-custom content: not reviewed or supported by the OmarchyGS marketplace.";

const OPERATOR_CUSTOM_SIGNATURE_DOMAIN: &[u8] = b"omarchygs-operator-custom-release-v1\0";
const MAX_OPERATOR_CUSTOM_RECORD_BYTES: usize = 512 * 1024;
const MAX_OPERATOR_NAME_CHARS: usize = 128;
const MAX_OPERATOR_WARNING_CHARS: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperatorCustomReleasePayload {
    pub format: String,
    pub attestation_version: u64,
    pub server_id: String,
    pub operator_name: String,
    pub authority_id: String,
    pub operator_key_sha256: String,
    pub publisher_key: PublisherPublicKey,
    pub game_key: String,
    pub publisher_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedOperatorCustomRelease {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperatorCustomAcquisition {
    pub format: String,
    pub server_admission: AcquisitionServerAdmission,
    pub operator_key: CatalogPublicKey,
    pub signed_operator_release: String,
    pub signed_policy: String,
    pub archive: String,
    pub conformance: String,
    pub release_attestation: String,
}

#[derive(Debug)]
pub struct VerifiedOperatorCustomAcquisition {
    release: VerifiedRelease,
    attestation: OperatorCustomReleasePayload,
    policy: CatalogPolicy,
    policy_bytes: Vec<u8>,
    signed_attestation_bytes: Vec<u8>,
    operator_key: CatalogPublicKey,
}

impl VerifiedOperatorCustomAcquisition {
    pub fn release(&self) -> &VerifiedRelease {
        &self.release
    }

    pub fn attestation(&self) -> &OperatorCustomReleasePayload {
        &self.attestation
    }

    pub fn policy(&self) -> &CatalogPolicy {
        &self.policy
    }

    pub fn policy_bytes(&self) -> &[u8] {
        &self.policy_bytes
    }

    pub fn signed_attestation_bytes(&self) -> &[u8] {
        &self.signed_attestation_bytes
    }

    pub fn operator_key(&self) -> &CatalogPublicKey {
        &self.operator_key
    }
}

pub fn operator_custom_key_sha256(key: &CatalogPublicKey) -> Result<String> {
    key.decode()?;
    Ok(sha256_hex(&canonical_json(key)?))
}

pub fn sign_operator_custom_release(
    release: &VerifiedRelease,
    publisher_key: &PublisherPublicKey,
    key: &CatalogPrivateKey,
    server_id: &str,
    operator_name: &str,
) -> Result<SignedOperatorCustomRelease> {
    let public = key.public_key()?;
    if !valid_server_id(server_id) || !valid_text(operator_name, MAX_OPERATOR_NAME_CHARS) {
        return Err(CartridgeError::InvalidOperatorCustom);
    }
    let payload = release.payload();
    if publisher_key.publisher_id != payload.publisher_id || publisher_key.key_id != payload.key_id
    {
        return Err(CartridgeError::InvalidOperatorCustom);
    }
    publisher_key.validate()?;
    let custom = OperatorCustomReleasePayload {
        format: OPERATOR_CUSTOM_RELEASE_FORMAT.to_owned(),
        attestation_version: 1,
        server_id: server_id.to_owned(),
        operator_name: operator_name.to_owned(),
        authority_id: public.authority_id.clone(),
        operator_key_sha256: operator_custom_key_sha256(&public)?,
        publisher_key: publisher_key.clone(),
        game_key: payload.game_key.clone(),
        publisher_id: payload.publisher_id.clone(),
        rules_version: payload.rules_version,
        cartridge_version: payload.cartridge_version,
        archive_sha256: payload.archive_sha256.clone(),
        signed_identity_sha256: payload.signed_identity_sha256.clone(),
        warning: OPERATOR_CUSTOM_WARNING.to_owned(),
    };
    let payload_bytes = canonical_json(&custom)?;
    if payload_bytes.len() > MAX_OPERATOR_CUSTOM_RECORD_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let mut message =
        Vec::with_capacity(OPERATOR_CUSTOM_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(OPERATOR_CUSTOM_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    let signature = key.decode()?.sign(&message);
    Ok(SignedOperatorCustomRelease {
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: key.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

pub fn signed_operator_custom_release_bytes(
    signed: &SignedOperatorCustomRelease,
) -> Result<Vec<u8>> {
    let bytes = canonical_json(signed)?;
    if bytes.is_empty() || bytes.len() > MAX_OPERATOR_CUSTOM_RECORD_BYTES {
        Err(CartridgeError::LimitExceeded)
    } else {
        Ok(bytes)
    }
}

pub fn verify_operator_custom_release_bytes(
    bytes: &[u8],
    key: &CatalogPublicKey,
    expected_server_id: &str,
    release: &VerifiedRelease,
) -> Result<OperatorCustomReleasePayload> {
    let payload = verify_operator_custom_signature(bytes, key)?;
    let release_payload = release.payload();
    if payload.server_id != expected_server_id
        || payload.publisher_key.publisher_id != release_payload.publisher_id
        || payload.publisher_key.key_id != release_payload.key_id
        || payload.game_key != release_payload.game_key
        || payload.publisher_id != release_payload.publisher_id
        || payload.rules_version != release_payload.rules_version
        || payload.cartridge_version != release_payload.cartridge_version
        || payload.archive_sha256 != release_payload.archive_sha256
        || payload.signed_identity_sha256 != release_payload.signed_identity_sha256
    {
        return Err(CartridgeError::InvalidOperatorCustom);
    }
    Ok(payload)
}

impl OperatorCustomAcquisition {
    #[allow(clippy::too_many_arguments)]
    pub fn from_verified_bytes(
        server_admission: AcquisitionServerAdmission,
        operator_key: CatalogPublicKey,
        signed_operator_release: &[u8],
        signed_policy: &[u8],
        archive: &[u8],
        conformance: &[u8],
        release_attestation: &[u8],
    ) -> Result<Self> {
        if !valid_admission(&server_admission)
            || !valid_component(signed_operator_release, MAX_OPERATOR_CUSTOM_RECORD_BYTES)
            || !valid_component(signed_policy, MAX_OPERATOR_CUSTOM_RECORD_BYTES)
            || !valid_component(archive, MAX_ARCHIVE_BYTES)
            || !valid_component(conformance, MAX_OPERATOR_CUSTOM_RECORD_BYTES)
            || !valid_component(release_attestation, MAX_OPERATOR_CUSTOM_RECORD_BYTES)
            || conformance.len() > MAX_JSON_BYTES
            || release_attestation.len() > MAX_JSON_BYTES
        {
            return Err(CartridgeError::LimitExceeded);
        }
        operator_custom_key_sha256(&operator_key)?;
        Ok(Self {
            format: OPERATOR_CUSTOM_ACQUISITION_FORMAT.to_owned(),
            server_admission,
            operator_key,
            signed_operator_release: URL_SAFE_NO_PAD.encode(signed_operator_release),
            signed_policy: URL_SAFE_NO_PAD.encode(signed_policy),
            archive: URL_SAFE_NO_PAD.encode(archive),
            conformance: URL_SAFE_NO_PAD.encode(conformance),
            release_attestation: URL_SAFE_NO_PAD.encode(release_attestation),
        })
    }

    pub fn to_bounded_json(&self) -> Result<Vec<u8>> {
        let bytes = canonical_json(self)?;
        if bytes.is_empty() || bytes.len() > crate::MAX_ACQUISITION_DOCUMENT_BYTES {
            Err(CartridgeError::LimitExceeded)
        } else {
            Ok(bytes)
        }
    }
}

pub fn verify_operator_custom_acquisition_bytes(
    bytes: &[u8],
    expected: &AcquisitionServerAdmission,
    trusted_operator_key: &CatalogPublicKey,
    sdk: &SdkIdentity,
    host: &HostProfile,
) -> Result<VerifiedOperatorCustomAcquisition> {
    if bytes.is_empty() || bytes.len() > crate::MAX_ACQUISITION_DOCUMENT_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let document: OperatorCustomAcquisition = serde_json::from_slice(bytes)?;
    if canonical_json(&document)? != bytes
        || document.format != OPERATOR_CUSTOM_ACQUISITION_FORMAT
        || &document.server_admission != expected
        || &document.operator_key != trusted_operator_key
        || !valid_admission(expected)
    {
        return Err(CartridgeError::InvalidOperatorCustom);
    }
    let signed_attestation_bytes = decode_bounded(
        &document.signed_operator_release,
        MAX_OPERATOR_CUSTOM_RECORD_BYTES,
    )?;
    let policy_bytes = decode_bounded(&document.signed_policy, MAX_OPERATOR_CUSTOM_RECORD_BYTES)?;
    let archive = decode_bounded(&document.archive, MAX_ARCHIVE_BYTES)?;
    let conformance = decode_bounded(&document.conformance, MAX_OPERATOR_CUSTOM_RECORD_BYTES)?;
    let release_attestation = decode_bounded(
        &document.release_attestation,
        MAX_OPERATOR_CUSTOM_RECORD_BYTES,
    )?;
    if conformance.len() > MAX_JSON_BYTES || release_attestation.len() > MAX_JSON_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let signed_payload =
        verify_operator_custom_signature(&signed_attestation_bytes, trusted_operator_key)?;
    let release = verify_release_components(
        &archive,
        &conformance,
        &release_attestation,
        &signed_payload.publisher_key,
        sdk,
        host,
    )?;
    let attestation = verify_operator_custom_release_bytes(
        &signed_attestation_bytes,
        trusted_operator_key,
        &expected.server_id,
        &release,
    )?;
    if expected.game_key != attestation.game_key
        || expected.publisher_id != attestation.publisher_id
        || expected.rules_version != attestation.rules_version
        || expected.cartridge_version != attestation.cartridge_version
        || expected.archive_sha256 != attestation.archive_sha256
        || expected.signed_identity_sha256 != attestation.signed_identity_sha256
    {
        return Err(CartridgeError::InvalidOperatorCustom);
    }
    let policy = verify_catalog_policy_bytes(&policy_bytes, trusted_operator_key, &release)?;
    Ok(VerifiedOperatorCustomAcquisition {
        release,
        attestation,
        policy,
        policy_bytes,
        signed_attestation_bytes,
        operator_key: trusted_operator_key.clone(),
    })
}

fn verify_operator_custom_signature(
    bytes: &[u8],
    key: &CatalogPublicKey,
) -> Result<OperatorCustomReleasePayload> {
    if !valid_component(bytes, MAX_OPERATOR_CUSTOM_RECORD_BYTES) {
        return Err(CartridgeError::LimitExceeded);
    }
    let signed: SignedOperatorCustomRelease = serde_json::from_slice(bytes)?;
    if canonical_json(&signed)? != bytes
        || signed.algorithm != SignatureAlgorithm::Ed25519
        || signed.key_id != key.key_id
    {
        return Err(CartridgeError::InvalidOperatorCustom);
    }
    let payload_bytes = decode_bounded(&signed.payload, MAX_OPERATOR_CUSTOM_RECORD_BYTES)?;
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&signed.signature)
        .map_err(|_| CartridgeError::InvalidOperatorCustom)?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| CartridgeError::InvalidOperatorCustom)?;
    let mut message =
        Vec::with_capacity(OPERATOR_CUSTOM_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(OPERATOR_CUSTOM_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    key.decode()?
        .verify_strict(&message, &signature)
        .map_err(|_| CartridgeError::InvalidOperatorCustom)?;
    let payload: OperatorCustomReleasePayload = serde_json::from_slice(&payload_bytes)?;
    if canonical_json(&payload)? != payload_bytes
        || payload.format != OPERATOR_CUSTOM_RELEASE_FORMAT
        || payload.attestation_version != 1
        || !valid_server_id(&payload.server_id)
        || !valid_text(&payload.operator_name, MAX_OPERATOR_NAME_CHARS)
        || payload.authority_id != key.authority_id
        || payload.operator_key_sha256 != operator_custom_key_sha256(key)?
        || payload.publisher_key.validate().is_err()
        || !valid_identifier(&payload.game_key)
        || !valid_identifier(&payload.publisher_id)
        || payload.publisher_key.publisher_id != payload.publisher_id
        || payload.rules_version == 0
        || payload.cartridge_version == 0
        || !valid_sha256(&payload.archive_sha256)
        || !valid_sha256(&payload.signed_identity_sha256)
        || payload.warning != OPERATOR_CUSTOM_WARNING
        || !valid_text(&payload.warning, MAX_OPERATOR_WARNING_CHARS)
    {
        return Err(CartridgeError::InvalidOperatorCustom);
    }
    Ok(payload)
}

fn decode_bounded(value: &str, maximum: usize) -> Result<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| CartridgeError::InvalidOperatorCustom)?;
    if !valid_component(&bytes, maximum) {
        Err(CartridgeError::LimitExceeded)
    } else {
        Ok(bytes)
    }
}

fn valid_component(bytes: &[u8], maximum: usize) -> bool {
    !bytes.is_empty() && bytes.len() <= maximum
}

fn valid_admission(value: &AcquisitionServerAdmission) -> bool {
    valid_server_id(&value.server_id)
        && valid_identifier(&value.game_key)
        && valid_identifier(&value.publisher_id)
        && value.rules_version > 0
        && value.cartridge_version > 0
        && valid_sha256(&value.archive_sha256)
        && valid_sha256(&value.signed_identity_sha256)
        && value.admission_revision > 0
}

fn valid_server_id(value: &str) -> bool {
    Uuid::try_parse(value).is_ok_and(|parsed| !parsed.is_nil() && parsed.to_string() == value)
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= maximum
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
