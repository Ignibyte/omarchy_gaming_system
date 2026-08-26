//! Signed, bounded marketplace snapshot contract.

use std::collections::BTreeSet;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer as _};
use serde::{Deserialize, Serialize};

use crate::{
    CatalogPrivateKey, CatalogPublicKey, PublisherPublicKey, SignatureAlgorithm,
    SignedCatalogPolicy,
    error::{CartridgeError, Result},
    keys::valid_identifier,
    lifecycle::verify_catalog_policy_signature,
    validate::canonical_json,
};

const MARKETPLACE_SIGNATURE_DOMAIN: &[u8] = b"omarchygs-marketplace-snapshot-v1\0";
pub const MAX_MARKETPLACE_SNAPSHOT_BYTES: usize = 1024 * 1024;
pub const MAX_MARKETPLACE_RELEASES: usize = 128;
const MAX_MARKETPLACE_NAME_CHARS: usize = 128;
const MAX_REVIEW_SUMMARY_CHARS: usize = 512;
const MAX_RELEASE_PATH_BYTES: usize = 256;
const MAX_RELEASE_PATH_SEGMENTS: usize = 12;

/// One exact publisher release reviewed by a marketplace authority.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MarketplaceReleaseEntry {
    pub release_path: String,
    pub game_key: String,
    pub publisher_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub publisher_key: PublisherPublicKey,
    pub reviewed_by: String,
    pub review_summary: String,
    pub policy: SignedCatalogPolicy,
}

impl MarketplaceReleaseEntry {
    /// Exact canonical lifecycle document supplied to the secure store.
    pub fn policy_bytes(&self) -> Result<Vec<u8>> {
        canonical_json(&self.policy)
    }
}

/// Canonical payload authenticated by one configured marketplace key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MarketplaceSnapshotPayload {
    pub format: String,
    pub snapshot_version: u64,
    pub authority_id: String,
    pub marketplace_name: String,
    pub releases: Vec<MarketplaceReleaseEntry>,
}

/// Domain-separated signature wrapper for one snapshot payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedMarketplaceSnapshot {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

/// Sign a fully validated marketplace snapshot.
pub fn sign_marketplace_snapshot(
    payload: &MarketplaceSnapshotPayload,
    key: &CatalogPrivateKey,
) -> Result<SignedMarketplaceSnapshot> {
    let public_key = key.public_key()?;
    validate_payload(payload, &public_key)?;
    let payload_bytes = canonical_json(payload)?;
    let mut message = Vec::with_capacity(MARKETPLACE_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(MARKETPLACE_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    let signature = key.decode()?.sign(&message);
    Ok(SignedMarketplaceSnapshot {
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: key.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

/// Verify exact canonical signed snapshot bytes against the configured key.
pub fn verify_marketplace_snapshot_bytes(
    bytes: &[u8],
    key: &CatalogPublicKey,
) -> Result<MarketplaceSnapshotPayload> {
    if bytes.is_empty() || bytes.len() > MAX_MARKETPLACE_SNAPSHOT_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let signed: SignedMarketplaceSnapshot = serde_json::from_slice(bytes)?;
    if canonical_json(&signed)? != bytes
        || signed.algorithm != SignatureAlgorithm::Ed25519
        || signed.key_id != key.key_id
    {
        return Err(CartridgeError::InvalidMarketplaceSnapshot);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(&signed.payload)
        .map_err(|_| CartridgeError::InvalidMarketplaceSnapshot)?;
    if payload_bytes.is_empty() || payload_bytes.len() > MAX_MARKETPLACE_SNAPSHOT_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&signed.signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|_| CartridgeError::InvalidSignature)?;
    let mut message = Vec::with_capacity(MARKETPLACE_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(MARKETPLACE_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    key.decode()?
        .verify_strict(&message, &signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;
    let payload: MarketplaceSnapshotPayload = serde_json::from_slice(&payload_bytes)?;
    if canonical_json(&payload)? != payload_bytes {
        return Err(CartridgeError::InvalidMarketplaceSnapshot);
    }
    validate_payload(&payload, key)?;
    Ok(payload)
}

fn validate_payload(payload: &MarketplaceSnapshotPayload, key: &CatalogPublicKey) -> Result<()> {
    key.decode()?;
    if payload.format != "omarchygs.marketplace-snapshot/v1"
        || payload.snapshot_version == 0
        || payload.snapshot_version > i64::MAX as u64
        || payload.authority_id != key.authority_id
        || !valid_plain_text(&payload.marketplace_name, MAX_MARKETPLACE_NAME_CHARS)
        || payload.releases.len() > MAX_MARKETPLACE_RELEASES
    {
        return Err(CartridgeError::InvalidMarketplaceSnapshot);
    }

    let mut prior_sort_key: Option<(String, u32, u32, String)> = None;
    let mut digests = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for entry in &payload.releases {
        entry.publisher_key.validate()?;
        if !valid_release_path(&entry.release_path)
            || !valid_identifier(&entry.game_key)
            || !valid_identifier(&entry.publisher_id)
            || entry.rules_version == 0
            || entry.cartridge_version == 0
            || !valid_sha256(&entry.archive_sha256)
            || !valid_sha256(&entry.signed_identity_sha256)
            || entry.publisher_key.publisher_id != entry.publisher_id
            || !valid_identifier(&entry.reviewed_by)
            || !valid_plain_text(&entry.review_summary, MAX_REVIEW_SUMMARY_CHARS)
        {
            return Err(CartridgeError::InvalidMarketplaceSnapshot);
        }
        let policy_bytes = entry.policy_bytes()?;
        let policy = verify_catalog_policy_signature(&policy_bytes, key)?;
        if policy.policy_version > i64::MAX as u64
            || policy.game_key != entry.game_key
            || policy.publisher_id != entry.publisher_id
            || policy.archive_sha256 != entry.archive_sha256
        {
            return Err(CartridgeError::InvalidMarketplaceSnapshot);
        }
        let sort_key = (
            entry.game_key.clone(),
            entry.rules_version,
            entry.cartridge_version,
            entry.archive_sha256.clone(),
        );
        if prior_sort_key
            .as_ref()
            .is_some_and(|prior| prior >= &sort_key)
            || !digests.insert(entry.archive_sha256.clone())
            || !paths.insert(entry.release_path.clone())
        {
            return Err(CartridgeError::InvalidMarketplaceSnapshot);
        }
        prior_sort_key = Some(sort_key);
    }
    Ok(())
}

fn valid_release_path(path: &str) -> bool {
    if path.is_empty()
        || path.len() > MAX_RELEASE_PATH_BYTES
        || path.starts_with('/')
        || !path.ends_with('/')
        || path.contains("//")
        || path.contains('%')
        || path.contains('?')
        || path.contains('#')
        || path.contains('\\')
    {
        return false;
    }
    let segments = path.trim_end_matches('/').split('/').collect::<Vec<_>>();
    !segments.is_empty()
        && segments.len() <= MAX_RELEASE_PATH_SEGMENTS
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && *segment != "."
                && *segment != ".."
                && segment.len() <= 96
                && segment.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'-' | b'_')
                })
        })
}

fn valid_plain_text(value: &str, max_chars: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
