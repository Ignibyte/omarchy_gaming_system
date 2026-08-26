use std::path::Path;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    SignatureAlgorithm, VerifiedRelease,
    error::{CartridgeError, Result},
    io::read_bounded_regular_file,
    keys::valid_identifier,
    validate::canonical_json,
};

const CATALOG_SIGNATURE_DOMAIN: &[u8] = b"omarchygs-catalog-policy-v1\0";
const MAX_CATALOG_FILE_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogPrivateKey {
    pub format_version: u32,
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub authority_id: String,
    pub signing_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogPublicKey {
    pub format_version: u32,
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub authority_id: String,
    pub verifying_key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogStatus {
    Active,
    Deprecated,
    Suspended,
    Revoked,
    Retired,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NewLaunchDecision {
    Allow,
    AllowWithWarning,
    Deny,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActiveSessionDecision {
    Continue,
    Suspend,
    Terminate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleUse {
    NewLaunch,
    ActiveSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LifecycleDecision {
    pub status: CatalogStatus,
    pub new_launch: NewLaunchDecision,
    pub active_session: ActiveSessionDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogPolicy {
    pub format: String,
    pub policy_version: u64,
    pub authority_id: String,
    pub game_key: String,
    pub publisher_id: String,
    pub archive_sha256: String,
    pub status: CatalogStatus,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedCatalogPolicy {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogPolicyReport {
    pub report_format: String,
    pub ok: bool,
    pub policy_version: u64,
    pub authority_id: String,
    pub game_key: String,
    pub publisher_id: String,
    pub archive_sha256: String,
    pub decision: LifecycleDecision,
}

pub fn generate_catalog_keypair(
    key_id: &str,
    authority_id: &str,
) -> Result<(CatalogPrivateKey, CatalogPublicKey)> {
    if !valid_identifier(key_id) || !valid_identifier(authority_id) {
        return Err(CartridgeError::InvalidKey);
    }
    let signing = SigningKey::generate(&mut OsRng);
    Ok((
        CatalogPrivateKey {
            format_version: 1,
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: key_id.to_owned(),
            authority_id: authority_id.to_owned(),
            signing_key: URL_SAFE_NO_PAD.encode(signing.to_bytes()),
        },
        CatalogPublicKey {
            format_version: 1,
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: key_id.to_owned(),
            authority_id: authority_id.to_owned(),
            verifying_key: URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes()),
        },
    ))
}

impl CatalogPrivateKey {
    pub(crate) fn decode(&self) -> Result<SigningKey> {
        self.validate()?;
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.signing_key)
            .map_err(|_| CartridgeError::InvalidKey)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| CartridgeError::InvalidKey)?;
        Ok(SigningKey::from_bytes(&bytes))
    }

    fn validate(&self) -> Result<()> {
        if self.format_version != 1
            || self.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.key_id)
            || !valid_identifier(&self.authority_id)
        {
            return Err(CartridgeError::InvalidKey);
        }
        Ok(())
    }

    pub fn public_key(&self) -> Result<CatalogPublicKey> {
        let signing = self.decode()?;
        Ok(CatalogPublicKey {
            format_version: self.format_version,
            algorithm: self.algorithm,
            key_id: self.key_id.clone(),
            authority_id: self.authority_id.clone(),
            verifying_key: URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes()),
        })
    }
}

impl CatalogPublicKey {
    pub(crate) fn decode(&self) -> Result<VerifyingKey> {
        self.validate()?;
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.verifying_key)
            .map_err(|_| CartridgeError::InvalidKey)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| CartridgeError::InvalidKey)?;
        VerifyingKey::from_bytes(&bytes).map_err(|_| CartridgeError::InvalidKey)
    }

    fn validate(&self) -> Result<()> {
        if self.format_version != 1
            || self.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.key_id)
            || !valid_identifier(&self.authority_id)
        {
            return Err(CartridgeError::InvalidKey);
        }
        Ok(())
    }
}

pub fn read_catalog_private_key(path: &Path) -> Result<CatalogPrivateKey> {
    let bytes = read_bounded_regular_file(path, MAX_CATALOG_FILE_BYTES)?;
    let key: CatalogPrivateKey = serde_json::from_slice(&bytes)?;
    key.decode()?;
    Ok(key)
}

pub fn read_catalog_public_key(path: &Path) -> Result<CatalogPublicKey> {
    let bytes = read_bounded_regular_file(path, MAX_CATALOG_FILE_BYTES)?;
    let key: CatalogPublicKey = serde_json::from_slice(&bytes)?;
    key.decode()?;
    Ok(key)
}

pub fn sign_catalog_policy(
    release: &VerifiedRelease,
    key: &CatalogPrivateKey,
    policy_version: u64,
    status: CatalogStatus,
    reason: &str,
) -> Result<SignedCatalogPolicy> {
    if policy_version == 0 || !valid_reason(reason) {
        return Err(CartridgeError::InvalidCatalogPolicy);
    }
    let payload = CatalogPolicy {
        format: "omarchygs.catalog-policy/v1".to_owned(),
        policy_version,
        authority_id: key.authority_id.clone(),
        game_key: release.payload().game_key.clone(),
        publisher_id: release.payload().publisher_id.clone(),
        archive_sha256: release.payload().archive_sha256.clone(),
        status,
        reason: reason.to_owned(),
    };
    let payload_bytes = canonical_json(&payload)?;
    let mut message = Vec::with_capacity(CATALOG_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(CATALOG_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    let signature = key.decode()?.sign(&message);
    Ok(SignedCatalogPolicy {
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: key.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload_bytes),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    })
}

pub fn verify_catalog_policy(
    signed: &SignedCatalogPolicy,
    key: &CatalogPublicKey,
    release: &VerifiedRelease,
) -> Result<CatalogPolicy> {
    let bytes = canonical_json(signed)?;
    verify_catalog_policy_bytes(&bytes, key, release)
}

pub fn verify_catalog_policy_bytes(
    bytes: &[u8],
    key: &CatalogPublicKey,
    release: &VerifiedRelease,
) -> Result<CatalogPolicy> {
    let policy = verify_catalog_policy_signature(bytes, key)?;
    if policy.game_key != release.payload().game_key
        || policy.publisher_id != release.payload().publisher_id
        || policy.archive_sha256 != release.payload().archive_sha256
    {
        return Err(CartridgeError::InvalidCatalogPolicy);
    }
    Ok(policy)
}

pub fn verify_catalog_policy_signature(
    bytes: &[u8],
    key: &CatalogPublicKey,
) -> Result<CatalogPolicy> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_CATALOG_FILE_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let signed: SignedCatalogPolicy = serde_json::from_slice(bytes)?;
    if canonical_json(&signed)? != bytes
        || signed.algorithm != SignatureAlgorithm::Ed25519
        || signed.key_id != key.key_id
    {
        return Err(CartridgeError::InvalidCatalogPolicy);
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(&signed.payload)
        .map_err(|_| CartridgeError::InvalidCatalogPolicy)?;
    if payload_bytes.len() as u64 > MAX_CATALOG_FILE_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&signed.signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|_| CartridgeError::InvalidSignature)?;
    let mut message = Vec::with_capacity(CATALOG_SIGNATURE_DOMAIN.len() + payload_bytes.len());
    message.extend_from_slice(CATALOG_SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload_bytes);
    key.decode()?
        .verify_strict(&message, &signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;
    let policy: CatalogPolicy = serde_json::from_slice(&payload_bytes)?;
    if canonical_json(&policy)? != payload_bytes
        || policy.format != "omarchygs.catalog-policy/v1"
        || policy.policy_version == 0
        || policy.authority_id != key.authority_id
        || !valid_identifier(&policy.game_key)
        || !valid_identifier(&policy.publisher_id)
        || !valid_sha256(&policy.archive_sha256)
        || !valid_reason(&policy.reason)
    {
        return Err(CartridgeError::InvalidCatalogPolicy);
    }
    Ok(policy)
}

pub fn lifecycle_decision(status: CatalogStatus) -> LifecycleDecision {
    let (new_launch, active_session) = match status {
        CatalogStatus::Active => (NewLaunchDecision::Allow, ActiveSessionDecision::Continue),
        CatalogStatus::Deprecated => (
            NewLaunchDecision::AllowWithWarning,
            ActiveSessionDecision::Continue,
        ),
        CatalogStatus::Suspended => (NewLaunchDecision::Deny, ActiveSessionDecision::Suspend),
        CatalogStatus::Revoked => (NewLaunchDecision::Deny, ActiveSessionDecision::Terminate),
        CatalogStatus::Retired => (NewLaunchDecision::Deny, ActiveSessionDecision::Continue),
    };
    LifecycleDecision {
        status,
        new_launch,
        active_session,
    }
}

pub fn policy_report(policy: &CatalogPolicy) -> CatalogPolicyReport {
    CatalogPolicyReport {
        report_format: "omarchygs.catalog-policy-report/v1".to_owned(),
        ok: true,
        policy_version: policy.policy_version,
        authority_id: policy.authority_id.clone(),
        game_key: policy.game_key.clone(),
        publisher_id: policy.publisher_id.clone(),
        archive_sha256: policy.archive_sha256.clone(),
        decision: lifecycle_decision(policy.status),
    }
}

pub fn ensure_allowed(decision: &LifecycleDecision, use_kind: LifecycleUse) -> Result<()> {
    let allowed = match use_kind {
        LifecycleUse::NewLaunch => !matches!(decision.new_launch, NewLaunchDecision::Deny),
        LifecycleUse::ActiveSession => {
            matches!(decision.active_session, ActiveSessionDecision::Continue)
        }
    };
    if allowed {
        Ok(())
    } else {
        Err(CartridgeError::LifecycleDenied)
    }
}

fn valid_reason(reason: &str) -> bool {
    !reason.is_empty() && reason.chars().count() <= 512 && !reason.chars().any(char::is_control)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
