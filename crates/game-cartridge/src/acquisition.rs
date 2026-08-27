//! Strict transport contract for one server-admitted cartridge acquisition.
//!
//! The envelope contains inert public evidence and exact release bytes. It
//! carries no destination, credential, local path, or executable content.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

use crate::{
    CatalogPolicy, CatalogPublicKey, HostProfile, MAX_ARCHIVE_BYTES, MAX_JSON_BYTES,
    MAX_MARKETPLACE_SNAPSHOT_BYTES, MarketplaceReleaseEntry, MarketplaceSnapshotPayload,
    VerifiedRelease,
    error::{CartridgeError, Result},
    lifecycle::verify_catalog_policy_bytes,
    release::verify_release_components,
    sdk::SdkIdentity,
    validate::canonical_json,
    verify_marketplace_snapshot_bytes,
};

pub const ACQUISITION_FORMAT: &str = "omarchygs.cartridge-acquisition/v1";
pub const ACQUISITION_FORMAT_V2: &str = "omarchygs.cartridge-acquisition/v2";
pub const MAX_ACQUISITION_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
const MAX_RELEASE_RECORD_BYTES: usize = 512 * 1024;

/// Selected-server admission bound to the exact bytes in an acquisition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AcquisitionServerAdmission {
    pub server_id: String,
    pub game_key: String,
    pub publisher_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub admission_revision: u64,
}

/// Exact public evidence and immutable release bytes returned by one selected
/// OmarchyGS server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CartridgeAcquisition {
    pub format: String,
    pub server_admission: AcquisitionServerAdmission,
    pub marketplace_key: CatalogPublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_marketplace_key: Option<CatalogPublicKey>,
    pub signed_marketplace_snapshot: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_policy_marketplace_snapshot: Option<String>,
    pub archive: String,
    pub conformance: String,
    pub release_attestation: String,
}

impl CartridgeAcquisition {
    #[allow(clippy::too_many_arguments)]
    pub fn from_verified_bytes(
        server_admission: AcquisitionServerAdmission,
        marketplace_key: CatalogPublicKey,
        signed_marketplace_snapshot: &[u8],
        archive: &[u8],
        conformance: &[u8],
        release_attestation: &[u8],
    ) -> Result<Self> {
        if signed_marketplace_snapshot.is_empty()
            || signed_marketplace_snapshot.len() > MAX_MARKETPLACE_SNAPSHOT_BYTES
            || archive.is_empty()
            || archive.len() > MAX_ARCHIVE_BYTES
            || conformance.is_empty()
            || conformance.len() > MAX_RELEASE_RECORD_BYTES
            || release_attestation.is_empty()
            || release_attestation.len() > MAX_RELEASE_RECORD_BYTES
        {
            return Err(CartridgeError::LimitExceeded);
        }
        Ok(Self {
            format: ACQUISITION_FORMAT.to_owned(),
            server_admission,
            marketplace_key,
            policy_marketplace_key: None,
            signed_marketplace_snapshot: URL_SAFE_NO_PAD.encode(signed_marketplace_snapshot),
            signed_policy_marketplace_snapshot: None,
            archive: URL_SAFE_NO_PAD.encode(archive),
            conformance: URL_SAFE_NO_PAD.encode(conformance),
            release_attestation: URL_SAFE_NO_PAD.encode(release_attestation),
        })
    }

    /// Construct a rotation-aware acquisition. Snapshot provenance and the
    /// current lifecycle decision are independently authenticated because a
    /// retained snapshot may have been signed by a retired marketplace key.
    #[allow(clippy::too_many_arguments)]
    pub fn from_verified_bytes_with_policy(
        server_admission: AcquisitionServerAdmission,
        marketplace_key: CatalogPublicKey,
        policy_marketplace_key: CatalogPublicKey,
        signed_marketplace_snapshot: &[u8],
        signed_policy_marketplace_snapshot: &[u8],
        archive: &[u8],
        conformance: &[u8],
        release_attestation: &[u8],
    ) -> Result<Self> {
        if signed_policy_marketplace_snapshot.is_empty()
            || signed_policy_marketplace_snapshot.len() > MAX_MARKETPLACE_SNAPSHOT_BYTES
        {
            return Err(CartridgeError::LimitExceeded);
        }
        let mut document = Self::from_verified_bytes(
            server_admission,
            marketplace_key,
            signed_marketplace_snapshot,
            archive,
            conformance,
            release_attestation,
        )?;
        document.format = ACQUISITION_FORMAT_V2.to_owned();
        document.policy_marketplace_key = Some(policy_marketplace_key);
        document.signed_policy_marketplace_snapshot =
            Some(URL_SAFE_NO_PAD.encode(signed_policy_marketplace_snapshot));
        Ok(document)
    }

    /// Serialize under the fixed acquisition response ceiling.
    pub fn to_bounded_json(&self) -> Result<Vec<u8>> {
        let bytes = canonical_json(self)?;
        if bytes.len() > MAX_ACQUISITION_DOCUMENT_BYTES {
            Err(CartridgeError::LimitExceeded)
        } else {
            Ok(bytes)
        }
    }
}

/// Independently verified acquisition ready for secure-store staging.
#[derive(Debug)]
pub struct VerifiedAcquisition {
    release: VerifiedRelease,
    snapshot: MarketplaceSnapshotPayload,
    policy_snapshot: MarketplaceSnapshotPayload,
    entry: MarketplaceReleaseEntry,
    policy: CatalogPolicy,
    policy_bytes: Vec<u8>,
    signed_snapshot_bytes: Vec<u8>,
    marketplace_key: CatalogPublicKey,
    policy_marketplace_key: CatalogPublicKey,
    policy_snapshot_version: u64,
}

impl VerifiedAcquisition {
    pub fn release(&self) -> &VerifiedRelease {
        &self.release
    }

    pub fn snapshot(&self) -> &MarketplaceSnapshotPayload {
        &self.snapshot
    }

    pub fn entry(&self) -> &MarketplaceReleaseEntry {
        &self.entry
    }

    pub fn policy_snapshot(&self) -> &MarketplaceSnapshotPayload {
        &self.policy_snapshot
    }

    pub fn policy(&self) -> &CatalogPolicy {
        &self.policy
    }

    pub fn policy_bytes(&self) -> &[u8] {
        &self.policy_bytes
    }

    pub fn signed_snapshot_bytes(&self) -> &[u8] {
        &self.signed_snapshot_bytes
    }

    pub fn marketplace_key(&self) -> &CatalogPublicKey {
        &self.marketplace_key
    }

    pub fn policy_marketplace_key(&self) -> &CatalogPublicKey {
        &self.policy_marketplace_key
    }

    pub fn policy_snapshot_version(&self) -> u64 {
        self.policy_snapshot_version
    }
}

/// Parse an exact-schema acquisition and independently verify every public
/// trust claim and release component against the expected selected-server
/// admission.
pub fn verify_acquisition_bytes(
    bytes: &[u8],
    expected: &AcquisitionServerAdmission,
    trusted_marketplace_key: &CatalogPublicKey,
    sdk: &SdkIdentity,
    host: &HostProfile,
) -> Result<VerifiedAcquisition> {
    verify_acquisition_bytes_with_policy_key(
        bytes,
        expected,
        trusted_marketplace_key,
        trusted_marketplace_key,
        sdk,
        host,
    )
}

/// Verify an acquisition while independently pinning snapshot evidence and
/// current lifecycle policy to their exact authorized marketplace keys.
pub fn verify_acquisition_bytes_with_policy_key(
    bytes: &[u8],
    expected: &AcquisitionServerAdmission,
    trusted_marketplace_key: &CatalogPublicKey,
    trusted_policy_marketplace_key: &CatalogPublicKey,
    sdk: &SdkIdentity,
    host: &HostProfile,
) -> Result<VerifiedAcquisition> {
    if bytes.is_empty() || bytes.len() > MAX_ACQUISITION_DOCUMENT_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let document: CartridgeAcquisition = serde_json::from_slice(bytes)?;
    if canonical_json(&document)? != bytes
        || &document.server_admission != expected
        || &document.marketplace_key != trusted_marketplace_key
    {
        return Err(CartridgeError::InvalidActivation);
    }
    let signed_policy_snapshot_bytes = match document.format.as_str() {
        ACQUISITION_FORMAT
            if document.policy_marketplace_key.is_none()
                && document.signed_policy_marketplace_snapshot.is_none()
                && trusted_policy_marketplace_key == trusted_marketplace_key =>
        {
            None
        }
        ACQUISITION_FORMAT_V2
            if document.policy_marketplace_key.as_ref() == Some(trusted_policy_marketplace_key)
                && document.signed_policy_marketplace_snapshot.is_some() =>
        {
            Some(decode_bounded(
                document
                    .signed_policy_marketplace_snapshot
                    .as_deref()
                    .ok_or(CartridgeError::InvalidActivation)?,
                MAX_MARKETPLACE_SNAPSHOT_BYTES,
            )?)
        }
        _ => return Err(CartridgeError::InvalidActivation),
    };
    let signed_snapshot_bytes = decode_bounded(
        &document.signed_marketplace_snapshot,
        MAX_MARKETPLACE_SNAPSHOT_BYTES,
    )?;
    let snapshot =
        verify_marketplace_snapshot_bytes(&signed_snapshot_bytes, trusted_marketplace_key)?;
    let entry = exact_entry(&snapshot, expected)
        .cloned()
        .ok_or(CartridgeError::InvalidMarketplaceSnapshot)?;
    let policy_snapshot = match signed_policy_snapshot_bytes {
        Some(bytes) => verify_marketplace_snapshot_bytes(&bytes, trusted_policy_marketplace_key)?,
        None => snapshot.clone(),
    };
    let policy_entry = exact_entry(&policy_snapshot, expected)
        .ok_or(CartridgeError::InvalidMarketplaceSnapshot)?;
    if policy_entry.publisher_key != entry.publisher_key {
        return Err(CartridgeError::InvalidMarketplaceSnapshot);
    }
    let policy_snapshot_version = policy_snapshot.snapshot_version;
    let archive = decode_bounded(&document.archive, MAX_ARCHIVE_BYTES)?;
    let conformance = decode_bounded(&document.conformance, MAX_RELEASE_RECORD_BYTES)?;
    let release_attestation =
        decode_bounded(&document.release_attestation, MAX_RELEASE_RECORD_BYTES)?;
    if conformance.len() > MAX_JSON_BYTES || release_attestation.len() > MAX_JSON_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let release = verify_release_components(
        &archive,
        &conformance,
        &release_attestation,
        &entry.publisher_key,
        sdk,
        host,
    )?;
    let payload = release.payload();
    if payload.game_key != expected.game_key
        || payload.publisher_id != expected.publisher_id
        || payload.rules_version != expected.rules_version
        || payload.cartridge_version != expected.cartridge_version
        || payload.archive_sha256 != expected.archive_sha256
        || payload.signed_identity_sha256 != expected.signed_identity_sha256
    {
        return Err(CartridgeError::InvalidActivation);
    }
    let policy_bytes = policy_entry.policy_bytes()?;
    let policy =
        verify_catalog_policy_bytes(&policy_bytes, trusted_policy_marketplace_key, &release)?;
    Ok(VerifiedAcquisition {
        release,
        snapshot,
        policy_snapshot,
        entry,
        policy,
        policy_bytes,
        signed_snapshot_bytes,
        marketplace_key: trusted_marketplace_key.clone(),
        policy_marketplace_key: trusted_policy_marketplace_key.clone(),
        policy_snapshot_version,
    })
}

fn exact_entry<'a>(
    snapshot: &'a MarketplaceSnapshotPayload,
    expected: &AcquisitionServerAdmission,
) -> Option<&'a MarketplaceReleaseEntry> {
    snapshot.releases.iter().find(|entry| {
        entry.game_key == expected.game_key
            && entry.publisher_id == expected.publisher_id
            && entry.rules_version == expected.rules_version
            && entry.cartridge_version == expected.cartridge_version
            && entry.archive_sha256 == expected.archive_sha256
            && entry.signed_identity_sha256 == expected.signed_identity_sha256
    })
}

fn decode_bounded(value: &str, maximum: usize) -> Result<Vec<u8>> {
    if value.is_empty() || value.len() > maximum.saturating_mul(4).div_ceil(3) + 4 {
        return Err(CartridgeError::LimitExceeded);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| CartridgeError::InvalidActivation)?;
    if bytes.is_empty() || bytes.len() > maximum {
        Err(CartridgeError::LimitExceeded)
    } else {
        Ok(bytes)
    }
}
