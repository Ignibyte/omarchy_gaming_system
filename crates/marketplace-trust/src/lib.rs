//! Independent, bounded marketplace trust and client package-channel contract.
//!
//! This crate is deliberately separate from the released Game Cartridge SDK.
//! It authenticates host distribution trust; it does not add cartridge code,
//! networking, filesystem, or gameplay authority.

use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _},
    path::Path,
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};
use omarchygs_game_cartridge::{CatalogPublicKey, SignatureAlgorithm};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use thiserror::Error;
use url::Url;

mod transport;

pub use transport::{ChannelEgressError, ChannelOrigin, GuardedChannelClient};

const TRUST_SIGNATURE_DOMAIN: &[u8] = b"omarchygs-marketplace-trust-channel-v1\0";
const MAX_ROOT_KEY_BYTES: u64 = 64 * 1024;
const MAX_BOOTSTRAP_BYTES: u64 = 64 * 1024;
pub const MAX_TRUST_CHANNEL_BYTES: usize = 256 * 1024;
pub const MAX_TRUST_KEYS: usize = 16;
pub const MAX_PACKAGE_ARTIFACTS: usize = 32;
pub const MAX_PACKAGE_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_TRUST_LIFETIME_SECONDS: u64 = 180 * 24 * 60 * 60;
pub const CLOCK_SKEW_SECONDS: u64 = 5 * 60;

#[derive(Debug, Error)]
pub enum TrustError {
    #[error("invalid marketplace trust input")]
    Invalid,
    #[error("marketplace trust signature is invalid")]
    Signature,
    #[error("marketplace trust input exceeds a bound")]
    Limit,
    #[error("marketplace trust is not currently valid")]
    Time,
    #[error("marketplace trust transition regressed")]
    Rollback,
    #[error("marketplace key is not authorized")]
    KeyDenied,
    #[error("marketplace trust file operation failed")]
    Io,
}

pub type Result<T> = std::result::Result<T, TrustError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TrustRootPrivateKey {
    pub format_version: u32,
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub channel_id: String,
    pub signing_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TrustRootPublicKey {
    pub format_version: u32,
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub channel_id: String,
    pub verifying_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicChannelBootstrap {
    pub format: String,
    pub channel_id: String,
    pub channel_origin: String,
    pub manifest_path: String,
    pub minimum_bundle_version: u64,
    pub minimum_current_snapshot_version: u64,
    pub platform: String,
    pub architecture: String,
    pub installed_package_version: String,
    pub root: TrustRootPublicKey,
}

impl PublicChannelBootstrap {
    pub fn validate(&self) -> Result<()> {
        self.root.validate()?;
        if self.format != "omarchygs.public-channel-bootstrap/v1"
            || self.channel_id != self.root.channel_id
            || !valid_identifier(&self.channel_id)
            || ChannelOrigin::parse(&self.channel_origin).is_err()
            || !valid_relative_path(&self.manifest_path)
            || self.minimum_bundle_version == 0
            || self.minimum_current_snapshot_version == 0
            || !valid_identifier(&self.platform)
            || !valid_identifier(&self.architecture)
            || !valid_version(&self.installed_package_version)
        {
            Err(TrustError::Invalid)
        } else {
            Ok(())
        }
    }

    /// Enforce the authenticated freshness floor shipped with the client
    /// package. This gives a client with no prior local state enough knowledge
    /// to reject a still-valid bundle that predates a known rotation or
    /// revocation.
    pub fn authorize_trust(&self, trust: &MarketplaceTrust) -> Result<()> {
        self.validate()?;
        if trust.root_sha256 != trust_root_sha256(&self.root)?
            || trust.payload.channel_id != self.channel_id
            || trust.payload.channel_origin != self.channel_origin
            || trust.payload.bundle_version < self.minimum_bundle_version
            || trust.payload.current_snapshot_version < self.minimum_current_snapshot_version
        {
            Err(TrustError::Rollback)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarketplaceKeyStatus {
    Active,
    Retired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MarketplaceTrustKey {
    pub key: CatalogPublicKey,
    pub key_sha256: String,
    pub status: MarketplaceKeyStatus,
    pub first_snapshot_version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_snapshot_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClientPackageArtifact {
    pub platform: String,
    pub architecture: String,
    pub package_version: String,
    pub filename: String,
    pub relative_path: String,
    pub bytes: u64,
    pub sha256: String,
    pub source_revision: String,
    pub source_sha256: String,
    pub build_provenance_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MarketplaceTrustPayload {
    pub format: String,
    pub channel_id: String,
    pub channel_name: String,
    pub channel_origin: String,
    pub marketplace_origin: String,
    pub marketplace_authority_id: String,
    pub bundle_version: u64,
    pub current_snapshot_version: u64,
    pub not_before_unix: u64,
    pub expires_at_unix: u64,
    pub keys: Vec<MarketplaceTrustKey>,
    pub packages: Vec<ClientPackageArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedMarketplaceTrust {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketplaceTrust {
    signed_bytes: Vec<u8>,
    payload: MarketplaceTrustPayload,
    root_sha256: String,
}

pub fn generate_trust_root_keypair(
    key_id: &str,
    channel_id: &str,
) -> Result<(TrustRootPrivateKey, TrustRootPublicKey)> {
    if !valid_identifier(key_id) || !valid_identifier(channel_id) {
        return Err(TrustError::Invalid);
    }
    let signing = SigningKey::generate(&mut OsRng);
    Ok((
        TrustRootPrivateKey {
            format_version: 1,
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: key_id.to_owned(),
            channel_id: channel_id.to_owned(),
            signing_key: URL_SAFE_NO_PAD.encode(signing.to_bytes()),
        },
        TrustRootPublicKey {
            format_version: 1,
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: key_id.to_owned(),
            channel_id: channel_id.to_owned(),
            verifying_key: URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes()),
        },
    ))
}

impl TrustRootPrivateKey {
    fn decode(&self) -> Result<SigningKey> {
        self.validate()?;
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.signing_key)
            .map_err(|_| TrustError::Invalid)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| TrustError::Invalid)?;
        Ok(SigningKey::from_bytes(&bytes))
    }

    fn validate(&self) -> Result<()> {
        if self.format_version != 1
            || self.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.key_id)
            || !valid_identifier(&self.channel_id)
        {
            return Err(TrustError::Invalid);
        }
        Ok(())
    }

    pub fn public_key(&self) -> Result<TrustRootPublicKey> {
        let signing = self.decode()?;
        Ok(TrustRootPublicKey {
            format_version: self.format_version,
            algorithm: self.algorithm,
            key_id: self.key_id.clone(),
            channel_id: self.channel_id.clone(),
            verifying_key: URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes()),
        })
    }
}

impl TrustRootPublicKey {
    fn decode(&self) -> Result<VerifyingKey> {
        self.validate()?;
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.verifying_key)
            .map_err(|_| TrustError::Invalid)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| TrustError::Invalid)?;
        VerifyingKey::from_bytes(&bytes).map_err(|_| TrustError::Invalid)
    }

    fn validate(&self) -> Result<()> {
        if self.format_version != 1
            || self.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.key_id)
            || !valid_identifier(&self.channel_id)
        {
            return Err(TrustError::Invalid);
        }
        Ok(())
    }
}

pub fn sign_marketplace_trust(
    payload: &MarketplaceTrustPayload,
    root: &TrustRootPrivateKey,
) -> Result<SignedMarketplaceTrust> {
    let public = root.public_key()?;
    validate_payload(payload, &public)?;
    let payload_bytes = canonical_json(payload)?;
    let mut message = Vec::with_capacity(TRUST_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(TRUST_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    let signature = root.decode()?.sign(&message);
    Ok(SignedMarketplaceTrust {
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: root.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

pub fn signed_trust_bytes(signed: &SignedMarketplaceTrust) -> Result<Vec<u8>> {
    let bytes = canonical_json(signed)?;
    if bytes.len() > MAX_TRUST_CHANNEL_BYTES {
        Err(TrustError::Limit)
    } else {
        Ok(bytes)
    }
}

pub fn verify_marketplace_trust_bytes(
    bytes: &[u8],
    root: &TrustRootPublicKey,
    expected_channel_id: &str,
    expected_channel_origin: &str,
    now_unix: u64,
) -> Result<MarketplaceTrust> {
    let trust = verify_marketplace_trust_bytes_at_rest(
        bytes,
        root,
        expected_channel_id,
        expected_channel_origin,
    )?;
    trust.validate_now(now_unix)?;
    Ok(trust)
}

/// Verify root signature and complete channel identity while deliberately not
/// granting time-valid authority. This is for loading the highest persisted
/// bundle so expiry cannot erase rollback or terminal-revocation history.
pub fn verify_marketplace_trust_bytes_at_rest(
    bytes: &[u8],
    root: &TrustRootPublicKey,
    expected_channel_id: &str,
    expected_channel_origin: &str,
) -> Result<MarketplaceTrust> {
    if bytes.is_empty() || bytes.len() > MAX_TRUST_CHANNEL_BYTES {
        return Err(TrustError::Limit);
    }
    root.validate()?;
    let signed: SignedMarketplaceTrust =
        serde_json::from_slice(bytes).map_err(|_| TrustError::Invalid)?;
    if canonical_json(&signed)? != bytes
        || signed.algorithm != SignatureAlgorithm::Ed25519
        || signed.key_id != root.key_id
    {
        return Err(TrustError::Invalid);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(&signed.payload)
        .map_err(|_| TrustError::Invalid)?;
    if payload_bytes.is_empty() || payload_bytes.len() > MAX_TRUST_CHANNEL_BYTES {
        return Err(TrustError::Limit);
    }
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&signed.signature)
        .map_err(|_| TrustError::Signature)?;
    let signature = Signature::from_slice(&signature_bytes).map_err(|_| TrustError::Signature)?;
    let mut message = Vec::with_capacity(TRUST_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(TRUST_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    root.decode()?
        .verify_strict(&message, &signature)
        .map_err(|_| TrustError::Signature)?;
    let payload: MarketplaceTrustPayload =
        serde_json::from_slice(&payload_bytes).map_err(|_| TrustError::Invalid)?;
    if canonical_json(&payload)? != payload_bytes {
        return Err(TrustError::Invalid);
    }
    validate_payload(&payload, root)?;
    if payload.channel_id != expected_channel_id
        || payload.channel_origin != expected_channel_origin
    {
        return Err(TrustError::Invalid);
    }
    Ok(MarketplaceTrust {
        signed_bytes: bytes.to_vec(),
        payload,
        root_sha256: trust_root_sha256(root)?,
    })
}

impl MarketplaceTrust {
    pub fn payload(&self) -> &MarketplaceTrustPayload {
        &self.payload
    }

    pub fn signed_bytes(&self) -> &[u8] {
        &self.signed_bytes
    }

    pub fn root_sha256(&self) -> &str {
        &self.root_sha256
    }

    pub fn active_key(&self) -> &CatalogPublicKey {
        &self
            .payload
            .keys
            .last()
            .expect("validated trust always has an active key")
            .key
    }

    pub fn key_by_fingerprint(
        &self,
        fingerprint: &str,
        snapshot_version: u64,
    ) -> Result<&CatalogPublicKey> {
        let record = self
            .payload
            .keys
            .iter()
            .find(|record| record.key_sha256 == fingerprint)
            .ok_or(TrustError::KeyDenied)?;
        authorize_record(record, snapshot_version)?;
        Ok(&record.key)
    }

    pub fn authorize_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<&MarketplaceTrustKey> {
        let fingerprint = catalog_key_sha256(key)?;
        let record = self
            .payload
            .keys
            .iter()
            .find(|record| record.key_sha256 == fingerprint && record.key == *key)
            .ok_or(TrustError::KeyDenied)?;
        authorize_record(record, snapshot_version)?;
        Ok(record)
    }

    pub fn authorize_new_snapshot(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<()> {
        let record = self.authorize_key(key, snapshot_version)?;
        if record.status == MarketplaceKeyStatus::Active
            && snapshot_version == self.payload.current_snapshot_version
        {
            Ok(())
        } else {
            Err(TrustError::KeyDenied)
        }
    }

    pub fn validate_now(&self, now_unix: u64) -> Result<()> {
        validate_time(&self.payload, now_unix)
    }
}

pub fn verify_trust_transition(previous: &MarketplaceTrust, next: &MarketplaceTrust) -> Result<()> {
    if previous.root_sha256 != next.root_sha256 {
        return Err(TrustError::Rollback);
    }
    verify_payload_transition(previous.payload(), next.payload())
}

/// Verify one already-authenticated persisted payload against a currently
/// root-verified trust bundle. Exact replay is accepted; rollback, root
/// replacement, and non-monotonic key history are denied.
pub fn verify_persisted_trust_continuity(
    previous_root_sha256: &str,
    previous_payload: &MarketplaceTrustPayload,
    next: &MarketplaceTrust,
) -> Result<()> {
    if previous_root_sha256 != next.root_sha256 {
        return Err(TrustError::Rollback);
    }
    if previous_payload == next.payload() {
        return Ok(());
    }
    verify_payload_transition(previous_payload, next.payload())
}

fn verify_payload_transition(
    old: &MarketplaceTrustPayload,
    new: &MarketplaceTrustPayload,
) -> Result<()> {
    if old.channel_id != new.channel_id
        || old.channel_origin != new.channel_origin
        || old.marketplace_origin != new.marketplace_origin
        || old.marketplace_authority_id != new.marketplace_authority_id
        || new.bundle_version <= old.bundle_version
        || new.current_snapshot_version < old.current_snapshot_version
        || new.not_before_unix < old.not_before_unix
        || new.keys.len() < old.keys.len()
    {
        return Err(TrustError::Rollback);
    }

    for prior in &old.keys {
        let current = new
            .keys
            .iter()
            .find(|candidate| candidate.key_sha256 == prior.key_sha256)
            .ok_or(TrustError::Rollback)?;
        if current.key != prior.key
            || current.first_snapshot_version != prior.first_snapshot_version
        {
            return Err(TrustError::Rollback);
        }
        let allowed = match (prior.status, current.status) {
            (MarketplaceKeyStatus::Active, MarketplaceKeyStatus::Active) => {
                prior.last_snapshot_version.is_none() && current.last_snapshot_version.is_none()
            }
            (MarketplaceKeyStatus::Active, MarketplaceKeyStatus::Retired)
            | (MarketplaceKeyStatus::Active, MarketplaceKeyStatus::Revoked) => {
                prior.last_snapshot_version.is_none()
                    && current
                        .last_snapshot_version
                        .is_some_and(|last| last >= old.current_snapshot_version)
            }
            (MarketplaceKeyStatus::Retired, MarketplaceKeyStatus::Retired)
            | (MarketplaceKeyStatus::Revoked, MarketplaceKeyStatus::Revoked) => {
                current.last_snapshot_version == prior.last_snapshot_version
            }
            (MarketplaceKeyStatus::Retired, MarketplaceKeyStatus::Revoked) => {
                current.last_snapshot_version == prior.last_snapshot_version
            }
            _ => false,
        };
        if !allowed {
            return Err(TrustError::Rollback);
        }
    }
    Ok(())
}

pub fn catalog_key_sha256(key: &CatalogPublicKey) -> Result<String> {
    validate_catalog_key(key)?;
    let bytes = canonical_json(key)?;
    Ok(sha256_hex(&bytes))
}

fn validate_catalog_key(key: &CatalogPublicKey) -> Result<()> {
    if key.format_version != 1
        || key.algorithm != SignatureAlgorithm::Ed25519
        || !valid_identifier(&key.key_id)
        || !valid_identifier(&key.authority_id)
    {
        return Err(TrustError::Invalid);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(&key.verifying_key)
        .map_err(|_| TrustError::Invalid)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| TrustError::Invalid)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| TrustError::Invalid)?;
    Ok(())
}

pub fn trust_root_sha256(key: &TrustRootPublicKey) -> Result<String> {
    key.validate()?;
    Ok(sha256_hex(&canonical_json(key)?))
}

pub fn read_trust_root_private_key(path: &Path) -> Result<TrustRootPrivateKey> {
    let bytes = read_checked_file(path, MAX_ROOT_KEY_BYTES, true)?;
    let key: TrustRootPrivateKey =
        serde_json::from_slice(&bytes).map_err(|_| TrustError::Invalid)?;
    key.decode()?;
    Ok(key)
}

pub fn read_trust_root_public_key(path: &Path) -> Result<TrustRootPublicKey> {
    let bytes = read_checked_file(path, MAX_ROOT_KEY_BYTES, false)?;
    let key: TrustRootPublicKey =
        serde_json::from_slice(&bytes).map_err(|_| TrustError::Invalid)?;
    key.decode()?;
    Ok(key)
}

pub fn read_public_channel_bootstrap(path: &Path) -> Result<PublicChannelBootstrap> {
    let bytes = read_checked_file(path, MAX_BOOTSTRAP_BYTES, false)?;
    let bootstrap: PublicChannelBootstrap =
        serde_json::from_slice(&bytes).map_err(|_| TrustError::Invalid)?;
    if canonical_json(&bootstrap)? != bytes {
        return Err(TrustError::Invalid);
    }
    bootstrap.validate()?;
    Ok(bootstrap)
}

pub fn write_new_public_channel_bootstrap(
    path: &Path,
    bootstrap: &PublicChannelBootstrap,
) -> Result<()> {
    bootstrap.validate()?;
    write_new(path, &canonical_json(bootstrap)?, 0o644)
}

pub fn write_new_private_key(path: &Path, key: &TrustRootPrivateKey) -> Result<()> {
    key.decode()?;
    write_new(path, &canonical_json(key)?, 0o600)
}

pub fn write_new_public_key(path: &Path, key: &TrustRootPublicKey) -> Result<()> {
    key.decode()?;
    write_new(path, &canonical_json(key)?, 0o644)
}

pub fn write_new_signed_trust(path: &Path, trust: &SignedMarketplaceTrust) -> Result<()> {
    write_new(path, &signed_trust_bytes(trust)?, 0o644)
}

fn validate_payload(payload: &MarketplaceTrustPayload, root: &TrustRootPublicKey) -> Result<()> {
    if payload.format != "omarchygs.marketplace-trust-channel/v2"
        || payload.channel_id != root.channel_id
        || !valid_identifier(&payload.channel_id)
        || !valid_text(&payload.channel_name, 128)
        || !valid_origin(&payload.channel_origin)
        || !valid_origin(&payload.marketplace_origin)
        || !valid_identifier(&payload.marketplace_authority_id)
        || payload.bundle_version == 0
        || payload.current_snapshot_version == 0
        || payload.not_before_unix == 0
        || payload.expires_at_unix <= payload.not_before_unix
        || payload
            .expires_at_unix
            .checked_sub(payload.not_before_unix)
            .is_none_or(|lifetime| lifetime > MAX_TRUST_LIFETIME_SECONDS)
        || payload.keys.is_empty()
        || payload.keys.len() > MAX_TRUST_KEYS
        || payload.packages.len() > MAX_PACKAGE_ARTIFACTS
    {
        return Err(TrustError::Invalid);
    }

    let mut prior_last = 0_u64;
    let mut key_ids = BTreeSet::new();
    let mut key_bytes = BTreeSet::new();
    let mut key_fingerprints = BTreeSet::new();
    for (index, record) in payload.keys.iter().enumerate() {
        let fingerprint = catalog_key_sha256(&record.key)?;
        if record.key.authority_id != payload.marketplace_authority_id
            || record.key_sha256 != fingerprint
            || record.first_snapshot_version != prior_last.saturating_add(1)
            || !key_ids.insert(record.key.key_id.clone())
            || !key_bytes.insert(record.key.verifying_key.clone())
            || !key_fingerprints.insert(record.key_sha256.clone())
        {
            return Err(TrustError::Invalid);
        }
        let final_record = index + 1 == payload.keys.len();
        if final_record {
            if record.status != MarketplaceKeyStatus::Active
                || record.last_snapshot_version.is_some()
                || payload.current_snapshot_version < record.first_snapshot_version
            {
                return Err(TrustError::Invalid);
            }
        } else {
            if record.status == MarketplaceKeyStatus::Active {
                return Err(TrustError::Invalid);
            }
            let last = record.last_snapshot_version.ok_or(TrustError::Invalid)?;
            if last < record.first_snapshot_version {
                return Err(TrustError::Invalid);
            }
            prior_last = last;
        }
    }

    let mut package_order: Option<(String, String, String, String)> = None;
    let mut package_paths = BTreeSet::new();
    let mut package_digests = BTreeSet::new();
    for artifact in &payload.packages {
        validate_artifact(artifact)?;
        let order = (
            artifact.platform.clone(),
            artifact.architecture.clone(),
            artifact.package_version.clone(),
            artifact.sha256.clone(),
        );
        if package_order.as_ref().is_some_and(|prior| prior >= &order)
            || !package_paths.insert(artifact.relative_path.clone())
            || !package_digests.insert(artifact.sha256.clone())
        {
            return Err(TrustError::Invalid);
        }
        package_order = Some(order);
    }
    Ok(())
}

fn validate_artifact(artifact: &ClientPackageArtifact) -> Result<()> {
    if !valid_identifier(&artifact.platform)
        || !valid_identifier(&artifact.architecture)
        || !valid_version(&artifact.package_version)
        || !valid_filename(&artifact.filename)
        || !valid_relative_path(&artifact.relative_path)
        || !artifact.relative_path.ends_with(&artifact.filename)
        || artifact.bytes == 0
        || artifact.bytes > MAX_PACKAGE_BYTES
        || !valid_sha256(&artifact.sha256)
        || !valid_revision(&artifact.source_revision)
        || !valid_sha256(&artifact.source_sha256)
        || !valid_sha256(&artifact.build_provenance_sha256)
    {
        return Err(TrustError::Invalid);
    }
    Ok(())
}

fn authorize_record(record: &MarketplaceTrustKey, snapshot_version: u64) -> Result<()> {
    if snapshot_version < record.first_snapshot_version
        || record
            .last_snapshot_version
            .is_some_and(|last| snapshot_version > last)
        || record.status == MarketplaceKeyStatus::Revoked
    {
        Err(TrustError::KeyDenied)
    } else {
        Ok(())
    }
}

fn validate_time(payload: &MarketplaceTrustPayload, now_unix: u64) -> Result<()> {
    if now_unix.saturating_add(CLOCK_SKEW_SECONDS) < payload.not_before_unix
        || now_unix >= payload.expires_at_unix
    {
        Err(TrustError::Time)
    } else {
        Ok(())
    }
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| TrustError::Invalid)
}

fn read_checked_file(path: &Path, maximum: u64, private: bool) -> Result<Vec<u8>> {
    let link = fs::symlink_metadata(path).map_err(|_| TrustError::Io)?;
    if link.file_type().is_symlink() || !link.is_file() || link.len() == 0 || link.len() > maximum {
        return Err(TrustError::Io);
    }
    let file = File::open(path).map_err(|_| TrustError::Io)?;
    let metadata = file.metadata().map_err(|_| TrustError::Io)?;
    if !metadata.is_file()
        || metadata.dev() != link.dev()
        || metadata.ino() != link.ino()
        || metadata.len() != link.len()
        || (private
            && (metadata.uid() != rustix::process::geteuid().as_raw()
                || metadata.nlink() != 1
                || metadata.mode() & 0o077 != 0))
    {
        return Err(TrustError::Io);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| TrustError::Io)?;
    if bytes.is_empty() || bytes.len() as u64 > maximum {
        Err(TrustError::Io)
    } else {
        Ok(bytes)
    }
}

fn write_new(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)
        .map_err(|_| TrustError::Io)?;
    file.write_all(bytes).map_err(|_| TrustError::Io)?;
    file.sync_all().map_err(|_| TrustError::Io)?;
    file.set_permissions(fs::Permissions::from_mode(mode))
        .map_err(|_| TrustError::Io)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= maximum
        && !value.chars().any(char::is_control)
}

fn valid_origin(value: &str) -> bool {
    if value.len() < 9 || value.len() > 512 || !value.ends_with('/') {
        return false;
    }
    let Ok(origin) = Url::parse(value) else {
        return false;
    };
    origin.as_str() == value
        && origin.scheme() == "https"
        && origin.has_host()
        && origin.username().is_empty()
        && origin.password().is_none()
        && origin.query().is_none()
        && origin.fragment().is_none()
}

fn valid_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains("//")
        && !value.contains('%')
        && !value.contains('?')
        && !value.contains('#')
        && !value.contains('\\')
        && value.split('/').all(valid_path_segment)
}

fn valid_filename(value: &str) -> bool {
    value.len() <= 192 && valid_path_segment(value)
}

fn valid_path_segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value.len() <= 192
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b'_' | b'+')
        })
}

fn valid_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+'))
}

fn valid_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use omarchygs_game_cartridge::generate_catalog_keypair;

    use super::*;

    fn payload(key: CatalogPublicKey, now: u64) -> MarketplaceTrustPayload {
        MarketplaceTrustPayload {
            format: "omarchygs.marketplace-trust-channel/v2".to_owned(),
            channel_id: "official".to_owned(),
            channel_name: "Official OmarchyGS".to_owned(),
            channel_origin: "https://packages.example.test/v1/".to_owned(),
            marketplace_origin: "https://market.example.test/v1/".to_owned(),
            marketplace_authority_id: "official-marketplace".to_owned(),
            bundle_version: 1,
            current_snapshot_version: 1,
            not_before_unix: now - 10,
            expires_at_unix: now + 3600,
            keys: vec![MarketplaceTrustKey {
                key_sha256: catalog_key_sha256(&key).expect("key hashes"),
                key,
                status: MarketplaceKeyStatus::Active,
                first_snapshot_version: 1,
                last_snapshot_version: None,
            }],
            packages: Vec::new(),
        }
    }

    #[test]
    fn signed_trust_is_canonical_exact_and_domain_authenticated() {
        let now = 1_800_000_000;
        let (root_private, root_public) =
            generate_trust_root_keypair("root-1", "official").expect("root");
        let (_, catalog) =
            generate_catalog_keypair("catalog-1", "official-marketplace").expect("catalog");
        let signed =
            sign_marketplace_trust(&payload(catalog.clone(), now), &root_private).expect("sign");
        let bytes = signed_trust_bytes(&signed).expect("bytes");
        let trust = verify_marketplace_trust_bytes(
            &bytes,
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("verify");
        trust.authorize_new_snapshot(&catalog, 1).expect("active");
        assert!(trust.authorize_new_snapshot(&catalog, 2).is_err());

        let mut tampered = bytes;
        let index = tampered.len() / 2;
        tampered[index] ^= 1;
        assert!(
            verify_marketplace_trust_bytes(
                &tampered,
                &root_public,
                "official",
                "https://packages.example.test/v1/",
                now,
            )
            .is_err()
        );
    }

    #[test]
    fn rotation_preserves_retired_history_and_revocation_is_terminal() {
        let now = 1_800_000_000;
        let (root_private, root_public) =
            generate_trust_root_keypair("root-1", "official").expect("root");
        let (_, first) =
            generate_catalog_keypair("catalog-1", "official-marketplace").expect("first");
        let (_, second) =
            generate_catalog_keypair("catalog-2", "official-marketplace").expect("second");
        let initial_signed =
            sign_marketplace_trust(&payload(first.clone(), now), &root_private).expect("initial");
        let initial = verify_marketplace_trust_bytes(
            &signed_trust_bytes(&initial_signed).expect("bytes"),
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("initial verify");

        let mut rotated_payload = payload(first.clone(), now);
        rotated_payload.bundle_version = 2;
        rotated_payload.current_snapshot_version = 8;
        rotated_payload.keys[0].status = MarketplaceKeyStatus::Retired;
        rotated_payload.keys[0].last_snapshot_version = Some(7);
        rotated_payload.keys.push(MarketplaceTrustKey {
            key_sha256: catalog_key_sha256(&second).expect("second hash"),
            key: second.clone(),
            status: MarketplaceKeyStatus::Active,
            first_snapshot_version: 8,
            last_snapshot_version: None,
        });
        let rotated_signed = sign_marketplace_trust(&rotated_payload, &root_private).expect("sign");
        let rotated = verify_marketplace_trust_bytes(
            &signed_trust_bytes(&rotated_signed).expect("bytes"),
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("rotated verify");
        verify_trust_transition(&initial, &rotated).expect("transition");
        rotated.authorize_key(&first, 7).expect("history");
        assert!(rotated.authorize_new_snapshot(&first, 7).is_err());
        assert!(rotated.authorize_new_snapshot(&second, 7).is_err());
        rotated
            .authorize_new_snapshot(&second, 8)
            .expect("new active");
        assert!(rotated.authorize_new_snapshot(&second, 9).is_err());

        let mut revoked_payload = rotated_payload;
        revoked_payload.bundle_version = 3;
        revoked_payload.keys[0].status = MarketplaceKeyStatus::Revoked;
        let revoked_signed = sign_marketplace_trust(&revoked_payload, &root_private).expect("sign");
        let revoked = verify_marketplace_trust_bytes(
            &signed_trust_bytes(&revoked_signed).expect("bytes"),
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("revoked verify");
        verify_trust_transition(&rotated, &revoked).expect("revocation transition");
        assert!(revoked.authorize_key(&first, 7).is_err());
        assert!(verify_trust_transition(&revoked, &rotated).is_err());
    }

    #[test]
    fn transition_preserves_authenticated_current_snapshot_ownership() {
        let now = 1_800_000_000;
        let (root_private, root_public) =
            generate_trust_root_keypair("root-1", "official").expect("root");
        let (_, first) =
            generate_catalog_keypair("catalog-1", "official-marketplace").expect("first");
        let (_, second) =
            generate_catalog_keypair("catalog-2", "official-marketplace").expect("second");

        let mut current_payload = payload(first.clone(), now);
        current_payload.current_snapshot_version = 5;
        let current_signed =
            sign_marketplace_trust(&current_payload, &root_private).expect("current sign");
        let current = verify_marketplace_trust_bytes(
            &signed_trust_bytes(&current_signed).expect("current bytes"),
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("current verify");

        let mut package_only_payload = current_payload.clone();
        package_only_payload.bundle_version = 2;
        let package_only_signed =
            sign_marketplace_trust(&package_only_payload, &root_private).expect("package sign");
        let package_only = verify_marketplace_trust_bytes(
            &signed_trust_bytes(&package_only_signed).expect("package bytes"),
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("package verify");
        verify_trust_transition(&current, &package_only).expect("package-only transition");
        verify_persisted_trust_continuity(current.root_sha256(), current.payload(), &current)
            .expect("exact persisted trust replay");
        assert!(
            verify_persisted_trust_continuity(
                package_only.root_sha256(),
                package_only.payload(),
                &current,
            )
            .is_err()
        );

        let mut reassigned_payload = current_payload;
        reassigned_payload.bundle_version = 3;
        reassigned_payload.keys[0].status = MarketplaceKeyStatus::Retired;
        reassigned_payload.keys[0].last_snapshot_version = Some(4);
        reassigned_payload.keys.push(MarketplaceTrustKey {
            key_sha256: catalog_key_sha256(&second).expect("second hash"),
            key: second,
            status: MarketplaceKeyStatus::Active,
            first_snapshot_version: 5,
            last_snapshot_version: None,
        });
        let reassigned_signed =
            sign_marketplace_trust(&reassigned_payload, &root_private).expect("reassigned sign");
        let reassigned = verify_marketplace_trust_bytes(
            &signed_trust_bytes(&reassigned_signed).expect("reassigned bytes"),
            &root_public,
            "official",
            "https://packages.example.test/v1/",
            now,
        )
        .expect("reassigned bundle is individually valid");
        assert!(verify_trust_transition(&package_only, &reassigned).is_err());
    }

    #[test]
    fn keyring_artifact_time_and_private_key_boundaries_fail_closed() {
        let now = 1_800_000_000;
        let (root_private, root_public) =
            generate_trust_root_keypair("root-1", "official").expect("root");
        let (_, first) =
            generate_catalog_keypair("catalog-1", "official-marketplace").expect("first");
        let (_, second) =
            generate_catalog_keypair("catalog-2", "official-marketplace").expect("second");

        let mut invalid = payload(first.clone(), now);
        invalid.keys[0].status = MarketplaceKeyStatus::Retired;
        invalid.keys[0].last_snapshot_version = Some(4);
        invalid.keys.push(MarketplaceTrustKey {
            key_sha256: catalog_key_sha256(&second).expect("second hash"),
            key: second.clone(),
            status: MarketplaceKeyStatus::Active,
            first_snapshot_version: 6,
            last_snapshot_version: None,
        });
        assert!(sign_marketplace_trust(&invalid, &root_private).is_err());

        invalid.keys[1].first_snapshot_version = 5;
        invalid.keys[1].key.key_id = invalid.keys[0].key.key_id.clone();
        invalid.keys[1].key_sha256 = catalog_key_sha256(&invalid.keys[1].key).expect("hash");
        assert!(sign_marketplace_trust(&invalid, &root_private).is_err());

        let mut with_packages = payload(first, now);
        with_packages.packages = vec![artifact("0.2.0-1", 'a'), artifact("0.3.0-1", 'b')];
        with_packages.packages[1].relative_path = with_packages.packages[0].relative_path.clone();
        assert!(sign_marketplace_trust(&with_packages, &root_private).is_err());
        with_packages.packages[1].relative_path = "packages/second.pkg.tar.zst".to_owned();
        with_packages.packages[1].sha256 = with_packages.packages[0].sha256.clone();
        assert!(sign_marketplace_trust(&with_packages, &root_private).is_err());
        with_packages.packages[1].sha256 = "b".repeat(64);
        with_packages.packages[1].relative_path = "../second.pkg.tar.zst".to_owned();
        assert!(sign_marketplace_trust(&with_packages, &root_private).is_err());

        let valid = payload(second, now);
        let signed = sign_marketplace_trust(&valid, &root_private).expect("valid sign");
        assert!(
            verify_marketplace_trust_bytes(
                &signed_trust_bytes(&signed).expect("signed bytes"),
                &root_public,
                "official",
                "https://packages.example.test/v1/",
                valid.expires_at_unix,
            )
            .is_err()
        );

        let temp = tempfile::tempdir().expect("temp should create");
        let private_path = temp.path().join("root.private.json");
        write_new_private_key(&private_path, &root_private).expect("private key should write");
        fs::set_permissions(&private_path, fs::Permissions::from_mode(0o644))
            .expect("fixture mode should change");
        assert!(read_trust_root_private_key(&private_path).is_err());
    }

    fn artifact(version: &str, digest: char) -> ClientPackageArtifact {
        ClientPackageArtifact {
            platform: "arch-linux".to_owned(),
            architecture: "x86_64".to_owned(),
            package_version: version.to_owned(),
            filename: format!("client-{version}.pkg.tar.zst"),
            relative_path: format!("packages/client-{version}.pkg.tar.zst"),
            bytes: 1024,
            sha256: digest.to_string().repeat(64),
            source_revision: "c".repeat(40),
            source_sha256: "d".repeat(64),
            build_provenance_sha256: "e".repeat(64),
        }
    }
}
