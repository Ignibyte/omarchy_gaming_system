//! Deterministic static marketplace publication and offline-root operations.
//!
//! This is operator tooling, not part of the exported Game Cartridge SDK. It
//! composes the existing publisher, catalog, trust-channel, and package
//! contracts without introducing a new client or server protocol.

mod probe;
mod store;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions, Permissions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

use omarchygs_game_cartridge::{
    CatalogStatus, MAX_ARCHIVE_BYTES, MAX_JSON_BYTES, MAX_MARKETPLACE_RELEASES,
    MarketplaceReleaseEntry, MarketplaceSnapshotPayload, PublisherPublicKey, RELEASE_ARCHIVE_PATH,
    RELEASE_ATTESTATION_PATH, RELEASE_CONFORMANCE_PATH, read_catalog_private_key,
    rich_2d_host_profile, sign_catalog_policy, sign_marketplace_snapshot,
    verify_marketplace_snapshot_bytes, verify_release_components, verify_sdk_directory,
};
use omarchygs_marketplace_trust::{
    ClientPackageArtifact, MAX_PACKAGE_ARTIFACTS, MAX_PACKAGE_BYTES, MAX_TRUST_CHANNEL_BYTES,
    MarketplaceTrust, MarketplaceTrustKey, MarketplaceTrustPayload, SignedMarketplaceTrust,
    TrustRootPublicKey, catalog_key_sha256, read_trust_root_private_key,
    read_trust_root_public_key, sign_marketplace_trust, signed_trust_bytes, trust_root_sha256,
    validate_marketplace_trust_payload, validate_marketplace_trust_transition,
    verify_marketplace_trust_bytes, verify_marketplace_trust_bytes_at_rest,
    verify_trust_transition,
};
use rustix::fs::{Mode, OFlags, open};
use rustix::process::geteuid;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest as _, Sha256};
use thiserror::Error;

pub use probe::{
    ProbeFloor, ProbeOrigin, ProbeReceipt, probe_mirrors, probe_mirrors_with_clients,
    probe_publication, probe_publication_with_clients,
};
pub use store::{activate_publication, finalize_publication, verify_current, verify_version};

pub const MAX_PLAN_BYTES: u64 = 512 * 1024;
pub const MAX_REQUEST_BYTES: u64 = 1024 * 1024;
pub const MAX_PUBLICATION_MANIFEST_BYTES: usize = 1024 * 1024;
pub const MAX_PUBLICATION_FILES: usize = 2 + MAX_MARKETPLACE_RELEASES * 3 + MAX_PACKAGE_ARTIFACTS;
pub const MAX_FINALIZED_VERSIONS: usize = 16;

pub const PREPARED_FILE: &str = "prepared.json";
pub const OFFLINE_REQUEST_FILE: &str = "offline-request.json";
pub const PUBLIC_DIRECTORY: &str = "public";
pub const CHANNEL_NAMESPACE: &str = "channel";
pub const MARKETPLACE_NAMESPACE: &str = "marketplace";
pub const PUBLICATION_MANIFEST_FILE: &str = "publication.json";
pub const TRUST_FILE: &str = "trust.signed.json";
pub const SNAPSHOT_FILE: &str = "snapshot.signed.json";

const PLAN_FORMAT: &str = "omarchygs.marketplace-publication-plan/v1";
const PREPARED_FORMAT: &str = "omarchygs.marketplace-prepared-publication/v1";
const REQUEST_FORMAT: &str = "omarchygs.marketplace-offline-request/v1";
const RESPONSE_FORMAT: &str = "omarchygs.marketplace-offline-response/v1";
const MANIFEST_FORMAT: &str = "omarchygs.marketplace-publication/v1";
const RECEIPT_FORMAT: &str = "omarchygs.marketplace-publication-receipt/v1";

/// Stable publication identity derived from the root-authenticated bundle.
#[must_use]
pub fn publication_id(bundle_version: u64) -> String {
    format!("publication-{bundle_version:020}")
}

#[derive(Debug, Error)]
pub enum PublisherError {
    #[error("invalid marketplace publication input")]
    InvalidInput,
    #[error("marketplace publication input was rejected")]
    Rejected,
    #[error("marketplace publication would roll back authenticated state")]
    Rollback,
    #[error("marketplace publication storage operation failed")]
    Storage,
    #[error("hosted marketplace publication is unavailable")]
    Unavailable,
}

impl PublisherError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput => "marketplace_publication_invalid_input",
            Self::Rejected => "marketplace_publication_rejected",
            Self::Rollback => "marketplace_publication_rollback",
            Self::Storage => "marketplace_publication_storage_failure",
            Self::Unavailable => "marketplace_publication_unavailable",
        }
    }
}

pub type Result<T> = std::result::Result<T, PublisherError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicationPlan {
    pub format: String,
    pub publication_id: String,
    pub created_at_unix: u64,
    pub ceremony_unix: u64,
    pub channel_id: String,
    pub channel_name: String,
    pub channel_origin: String,
    pub marketplace_origin: String,
    pub marketplace_authority_id: String,
    pub marketplace_name: String,
    pub bundle_version: u64,
    pub snapshot_version: u64,
    pub not_before_unix: u64,
    pub expires_at_unix: u64,
    pub keys: Vec<MarketplaceTrustKey>,
    pub releases: Vec<PublicationReleasePlan>,
    pub packages: Vec<PublicationPackagePlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_trust_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicationReleasePlan {
    pub input_directory: String,
    pub publisher_key_path: String,
    pub release_path: String,
    pub policy_version: u64,
    pub status: CatalogStatus,
    pub reason: String,
    pub reviewed_by: String,
    pub review_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicationPackagePlan {
    pub input_path: String,
    pub relative_path: String,
    pub platform: String,
    pub architecture: String,
    pub package_version: String,
    pub filename: String,
    pub source_revision: String,
    pub source_sha256: String,
    pub build_provenance_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PublicationNamespace {
    Channel,
    Marketplace,
}

impl PublicationNamespace {
    pub(crate) const fn directory(&self) -> &'static str {
        match self {
            Self::Channel => CHANNEL_NAMESPACE,
            Self::Marketplace => MARKETPLACE_NAMESPACE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicationFile {
    pub namespace: PublicationNamespace,
    pub relative_path: String,
    pub media_type: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreparedPublication {
    pub format: String,
    pub plan: PublicationPlan,
    pub plan_sha256: String,
    pub request_sha256: String,
    pub snapshot_sha256: String,
    pub files: Vec<PublicationFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OfflineSigningRequest {
    pub format: String,
    pub publication_id: String,
    pub plan_sha256: String,
    pub prepared_files_sha256: String,
    pub ceremony_unix: u64,
    pub root: TrustRootPublicKey,
    pub trust_payload: MarketplaceTrustPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_trust: Option<SignedMarketplaceTrust>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OfflineSigningResponse {
    pub format: String,
    pub request_sha256: String,
    pub root_sha256: String,
    pub trust_sha256: String,
    pub signed_trust: SignedMarketplaceTrust,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicationManifest {
    pub format: String,
    pub publication_id: String,
    pub created_at_unix: u64,
    pub channel_origin: String,
    pub marketplace_origin: String,
    pub bundle_version: u64,
    pub snapshot_version: u64,
    pub root_sha256: String,
    pub catalog_key_sha256: String,
    pub trust_sha256: String,
    pub snapshot_sha256: String,
    pub files: Vec<PublicationFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperationReceipt {
    pub format: String,
    pub ok: bool,
    pub operation: String,
    pub publication_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_sha256: Option<String>,
    pub evidence_sha256: String,
    pub bundle_version: u64,
    pub snapshot_version: u64,
    pub root_sha256: String,
    pub catalog_key_sha256: String,
    pub files: usize,
    pub recorded_at_unix: u64,
}

pub struct PrepareOptions<'a> {
    pub plan_path: &'a Path,
    pub input_root: &'a Path,
    pub sdk_root: &'a Path,
    pub catalog_private_key_path: &'a Path,
    pub root_public_key_path: &'a Path,
    pub previous_trust_path: Option<&'a Path>,
    pub output_root: &'a Path,
}

/// Verify all online inputs and create a deterministic public offline request.
pub fn prepare_publication(options: PrepareOptions<'_>) -> Result<OperationReceipt> {
    require_absolute_private_directory(options.input_root)?;
    require_absolute_private_file(options.catalog_private_key_path, 0o600)?;
    require_absolute(options.output_root)?;
    if options.output_root.exists() {
        return Err(PublisherError::Storage);
    }
    let plan_bytes = read_regular_file(options.plan_path, MAX_PLAN_BYTES, false)?;
    let plan: PublicationPlan = parse_canonical(&plan_bytes, MAX_PLAN_BYTES as usize)?;
    validate_plan(&plan)?;
    let plan_sha256 = sha256(&plan_bytes);
    let root = read_trust_root_public_key(options.root_public_key_path)
        .map_err(|_| PublisherError::Rejected)?;
    if root.channel_id != plan.channel_id {
        return Err(PublisherError::Rejected);
    }
    let catalog_private = read_catalog_private_key(options.catalog_private_key_path)
        .map_err(|_| PublisherError::Rejected)?;
    let catalog_public = catalog_private
        .public_key()
        .map_err(|_| PublisherError::Rejected)?;
    let active = plan.keys.last().ok_or(PublisherError::InvalidInput)?;
    if active.key != catalog_public
        || active.key_sha256
            != catalog_key_sha256(&catalog_public).map_err(|_| PublisherError::Rejected)?
    {
        return Err(PublisherError::Rejected);
    }

    create_directory(options.output_root, 0o700)?;
    let public_root = options.output_root.join(PUBLIC_DIRECTORY);
    create_directory(&public_root, 0o700)?;
    create_directory(&public_root.join(CHANNEL_NAMESPACE), 0o700)?;
    create_directory(&public_root.join(MARKETPLACE_NAMESPACE), 0o700)?;

    let sdk = verify_sdk_directory(options.sdk_root).map_err(|_| PublisherError::Rejected)?;
    let host = rich_2d_host_profile();
    let mut entries = Vec::with_capacity(plan.releases.len());
    let mut files = Vec::new();
    for release_plan in &plan.releases {
        let release_root = safe_join(options.input_root, &release_plan.input_directory)?;
        let publisher_path = safe_join(options.input_root, &release_plan.publisher_key_path)?;
        let publisher_bytes = read_regular_file(&publisher_path, 64 * 1024, false)?;
        let publisher: PublisherPublicKey = parse_canonical(&publisher_bytes, 64 * 1024)?;
        publisher.validate().map_err(|_| PublisherError::Rejected)?;
        let archive = read_regular_file(
            &release_root.join(RELEASE_ARCHIVE_PATH),
            MAX_ARCHIVE_BYTES as u64,
            false,
        )?;
        let conformance = read_regular_file(
            &release_root.join(RELEASE_CONFORMANCE_PATH),
            MAX_JSON_BYTES as u64,
            false,
        )?;
        let attestation = read_regular_file(
            &release_root.join(RELEASE_ATTESTATION_PATH),
            MAX_JSON_BYTES as u64,
            false,
        )?;
        let release = verify_release_components(
            &archive,
            &conformance,
            &attestation,
            &publisher,
            &sdk,
            &host,
        )
        .map_err(|_| PublisherError::Rejected)?;
        let policy = sign_catalog_policy(
            &release,
            &catalog_private,
            release_plan.policy_version,
            release_plan.status,
            &release_plan.reason,
        )
        .map_err(|_| PublisherError::Rejected)?;
        let payload = release.payload();
        entries.push(MarketplaceReleaseEntry {
            release_path: release_plan.release_path.clone(),
            game_key: payload.game_key.clone(),
            publisher_id: payload.publisher_id.clone(),
            rules_version: payload.rules_version,
            cartridge_version: payload.cartridge_version,
            archive_sha256: payload.archive_sha256.clone(),
            signed_identity_sha256: payload.signed_identity_sha256.clone(),
            publisher_key: publisher,
            reviewed_by: release_plan.reviewed_by.clone(),
            review_summary: release_plan.review_summary.clone(),
            policy,
        });
        for (name, bytes, media_type) in [
            (
                RELEASE_ARCHIVE_PATH,
                archive.as_slice(),
                "application/octet-stream",
            ),
            (
                RELEASE_CONFORMANCE_PATH,
                conformance.as_slice(),
                "application/json",
            ),
            (
                RELEASE_ATTESTATION_PATH,
                attestation.as_slice(),
                "application/json",
            ),
        ] {
            let relative_path = format!("{}{name}", release_plan.release_path);
            let destination = safe_join(&public_root.join(MARKETPLACE_NAMESPACE), &relative_path)?;
            write_public_file(&destination, bytes)?;
            files.push(publication_file(
                PublicationNamespace::Marketplace,
                relative_path,
                media_type,
                bytes,
            ));
        }
    }
    entries.sort_by(|left, right| {
        (
            &left.game_key,
            left.rules_version,
            left.cartridge_version,
            &left.archive_sha256,
        )
            .cmp(&(
                &right.game_key,
                right.rules_version,
                right.cartridge_version,
                &right.archive_sha256,
            ))
    });
    let snapshot_payload = MarketplaceSnapshotPayload {
        format: "omarchygs.marketplace-snapshot/v1".to_owned(),
        snapshot_version: plan.snapshot_version,
        authority_id: plan.marketplace_authority_id.clone(),
        marketplace_name: plan.marketplace_name.clone(),
        releases: entries,
    };
    let signed_snapshot = sign_marketplace_snapshot(&snapshot_payload, &catalog_private)
        .map_err(|_| PublisherError::Rejected)?;
    let snapshot_bytes = canonical_json(&signed_snapshot)?;
    verify_marketplace_snapshot_bytes(&snapshot_bytes, &catalog_public)
        .map_err(|_| PublisherError::Rejected)?;
    write_public_file(
        &public_root.join(MARKETPLACE_NAMESPACE).join(SNAPSHOT_FILE),
        &snapshot_bytes,
    )?;
    let snapshot_sha256 = sha256(&snapshot_bytes);
    files.push(publication_file(
        PublicationNamespace::Marketplace,
        SNAPSHOT_FILE.to_owned(),
        "application/json",
        &snapshot_bytes,
    ));

    let mut packages = Vec::with_capacity(plan.packages.len());
    for package_plan in &plan.packages {
        let source = safe_join(options.input_root, &package_plan.input_path)?;
        let destination = safe_join(
            &public_root.join(CHANNEL_NAMESPACE),
            &package_plan.relative_path,
        )?;
        let (package_bytes, package_sha256) =
            copy_public_file(&source, &destination, MAX_PACKAGE_BYTES)?;
        let artifact = ClientPackageArtifact {
            platform: package_plan.platform.clone(),
            architecture: package_plan.architecture.clone(),
            package_version: package_plan.package_version.clone(),
            filename: package_plan.filename.clone(),
            relative_path: package_plan.relative_path.clone(),
            bytes: package_bytes,
            sha256: package_sha256.clone(),
            source_revision: package_plan.source_revision.clone(),
            source_sha256: package_plan.source_sha256.clone(),
            build_provenance_sha256: package_plan.build_provenance_sha256.clone(),
        };
        files.push(PublicationFile {
            namespace: PublicationNamespace::Channel,
            relative_path: artifact.relative_path.clone(),
            media_type: "application/vnd.archlinux.package".to_owned(),
            bytes: package_bytes,
            sha256: package_sha256,
        });
        packages.push(artifact);
    }
    packages.sort_by(|left, right| {
        (
            &left.platform,
            &left.architecture,
            &left.package_version,
            &left.sha256,
        )
            .cmp(&(
                &right.platform,
                &right.architecture,
                &right.package_version,
                &right.sha256,
            ))
    });

    let trust_payload = MarketplaceTrustPayload {
        format: "omarchygs.marketplace-trust-channel/v2".to_owned(),
        channel_id: plan.channel_id.clone(),
        channel_name: plan.channel_name.clone(),
        channel_origin: plan.channel_origin.clone(),
        marketplace_origin: plan.marketplace_origin.clone(),
        marketplace_authority_id: plan.marketplace_authority_id.clone(),
        bundle_version: plan.bundle_version,
        current_snapshot_version: plan.snapshot_version,
        not_before_unix: plan.not_before_unix,
        expires_at_unix: plan.expires_at_unix,
        keys: plan.keys.clone(),
        packages,
    };
    validate_marketplace_trust_payload(&trust_payload, &root)
        .map_err(|_| PublisherError::Rejected)?;
    let previous = load_previous_trust(
        options.previous_trust_path,
        &root,
        &plan,
        Some(&trust_payload),
    )?;
    files.sort_by(publication_file_order);
    validate_file_inventory(&files, false)?;
    let prepared_files_sha256 = sha256(&canonical_json(&files)?);
    let request = OfflineSigningRequest {
        format: REQUEST_FORMAT.to_owned(),
        publication_id: plan.publication_id.clone(),
        plan_sha256: plan_sha256.clone(),
        prepared_files_sha256,
        ceremony_unix: plan.ceremony_unix,
        root: root.clone(),
        trust_payload,
        previous_trust: previous,
    };
    validate_request(&request)?;
    let request_bytes = canonical_json(&request)?;
    let request_sha256 = sha256(&request_bytes);
    write_public_file(
        &options.output_root.join(OFFLINE_REQUEST_FILE),
        &request_bytes,
    )?;
    let prepared = PreparedPublication {
        format: PREPARED_FORMAT.to_owned(),
        plan: plan.clone(),
        plan_sha256,
        request_sha256,
        snapshot_sha256,
        files,
    };
    write_public_file(
        &options.output_root.join(PREPARED_FILE),
        &canonical_json(&prepared)?,
    )?;
    fsync_tree(options.output_root)?;
    receipt(
        "prepare",
        &plan,
        None,
        &prepared.request_sha256,
        trust_root_sha256(&root).map_err(|_| PublisherError::Rejected)?,
        catalog_key_sha256(&catalog_public).map_err(|_| PublisherError::Rejected)?,
        prepared.files.len(),
    )
}

/// Sign one public offline request without contacting a network destination.
pub fn offline_sign(
    request_path: &Path,
    root_private_path: &Path,
    output_path: &Path,
) -> Result<OperationReceipt> {
    require_absolute_private_file(root_private_path, 0o600)?;
    require_absolute(output_path)?;
    let request_bytes = read_regular_file(request_path, MAX_REQUEST_BYTES, false)?;
    let request: OfflineSigningRequest =
        parse_canonical(&request_bytes, MAX_REQUEST_BYTES as usize)?;
    validate_request(&request)?;
    let private =
        read_trust_root_private_key(root_private_path).map_err(|_| PublisherError::Rejected)?;
    let public = private.public_key().map_err(|_| PublisherError::Rejected)?;
    if public != request.root {
        return Err(PublisherError::Rejected);
    }
    let signed = sign_marketplace_trust(&request.trust_payload, &private)
        .map_err(|_| PublisherError::Rejected)?;
    let signed_bytes = signed_trust_bytes(&signed).map_err(|_| PublisherError::Rejected)?;
    let current = verify_marketplace_trust_bytes(
        &signed_bytes,
        &public,
        &request.trust_payload.channel_id,
        &request.trust_payload.channel_origin,
        request.ceremony_unix,
    )
    .map_err(|_| PublisherError::Rejected)?;
    if let Some(previous) = request.previous_trust.as_ref() {
        let previous_bytes = canonical_json(previous)?;
        let previous = verify_marketplace_trust_bytes_at_rest(
            &previous_bytes,
            &public,
            &request.trust_payload.channel_id,
            &request.trust_payload.channel_origin,
        )
        .map_err(|_| PublisherError::Rejected)?;
        verify_trust_transition(&previous, &current).map_err(|_| PublisherError::Rollback)?;
    }
    let response = OfflineSigningResponse {
        format: RESPONSE_FORMAT.to_owned(),
        request_sha256: sha256(&request_bytes),
        root_sha256: trust_root_sha256(&public).map_err(|_| PublisherError::Rejected)?,
        trust_sha256: sha256(&signed_bytes),
        signed_trust: signed,
    };
    write_new_file(output_path, &canonical_json(&response)?, 0o644)?;
    receipt_from_request("offline_sign", &request, None, &response.trust_sha256, 0)
}

pub(crate) fn load_prepared(root: &Path) -> Result<(PreparedPublication, OfflineSigningRequest)> {
    let prepared_bytes = read_regular_file(&root.join(PREPARED_FILE), MAX_REQUEST_BYTES, false)?;
    let prepared: PreparedPublication =
        parse_canonical(&prepared_bytes, MAX_REQUEST_BYTES as usize)?;
    if prepared.format != PREPARED_FORMAT {
        return Err(PublisherError::InvalidInput);
    }
    validate_plan(&prepared.plan)?;
    let request_bytes =
        read_regular_file(&root.join(OFFLINE_REQUEST_FILE), MAX_REQUEST_BYTES, false)?;
    let request: OfflineSigningRequest =
        parse_canonical(&request_bytes, MAX_REQUEST_BYTES as usize)?;
    validate_request(&request)?;
    if prepared.plan_sha256 != sha256(&canonical_json(&prepared.plan)?)
        || prepared.request_sha256 != sha256(&request_bytes)
        || request.plan_sha256 != prepared.plan_sha256
        || request.prepared_files_sha256 != sha256(&canonical_json(&prepared.files)?)
    {
        return Err(PublisherError::Rejected);
    }
    validate_file_inventory(&prepared.files, false)?;
    verify_prepared_files(root, &prepared.files)?;
    Ok((prepared, request))
}

pub(crate) fn load_response(
    path: &Path,
    request: &OfflineSigningRequest,
) -> Result<(OfflineSigningResponse, MarketplaceTrust)> {
    let response_bytes = read_regular_file(path, MAX_REQUEST_BYTES, false)?;
    let response: OfflineSigningResponse =
        parse_canonical(&response_bytes, MAX_REQUEST_BYTES as usize)?;
    if response.format != RESPONSE_FORMAT {
        return Err(PublisherError::InvalidInput);
    }
    let request_sha256 = sha256(&canonical_json(request)?);
    let signed_bytes =
        signed_trust_bytes(&response.signed_trust).map_err(|_| PublisherError::Rejected)?;
    if response.request_sha256 != request_sha256
        || response.root_sha256
            != trust_root_sha256(&request.root).map_err(|_| PublisherError::Rejected)?
        || response.trust_sha256 != sha256(&signed_bytes)
    {
        return Err(PublisherError::Rejected);
    }
    let trust = verify_marketplace_trust_bytes_at_rest(
        &signed_bytes,
        &request.root,
        &request.trust_payload.channel_id,
        &request.trust_payload.channel_origin,
    )
    .map_err(|_| PublisherError::Rejected)?;
    if trust.payload() != &request.trust_payload {
        return Err(PublisherError::Rejected);
    }
    if let Some(previous) = request.previous_trust.as_ref() {
        let previous = verify_marketplace_trust_bytes_at_rest(
            &canonical_json(previous)?,
            &request.root,
            &request.trust_payload.channel_id,
            &request.trust_payload.channel_origin,
        )
        .map_err(|_| PublisherError::Rejected)?;
        verify_trust_transition(&previous, &trust).map_err(|_| PublisherError::Rollback)?;
    }
    Ok((response, trust))
}

pub(crate) fn verify_prepared_files(root: &Path, files: &[PublicationFile]) -> Result<()> {
    for record in files {
        let path = safe_join(
            &root
                .join(PUBLIC_DIRECTORY)
                .join(record.namespace.directory()),
            &record.relative_path,
        )?;
        if record.media_type == "application/vnd.archlinux.package" {
            verify_regular_file_exact(&path, record)?;
        } else {
            let bytes = read_regular_file(&path, file_limit(record), false)?;
            if bytes.len() as u64 != record.bytes || sha256(&bytes) != record.sha256 {
                return Err(PublisherError::Rejected);
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_publication_core(
    manifest: &PublicationManifest,
    trust_bytes: &[u8],
    snapshot_bytes: &[u8],
    root: &TrustRootPublicKey,
    now_unix: u64,
) -> Result<(MarketplaceTrust, MarketplaceSnapshotPayload)> {
    validate_manifest(manifest)?;
    if manifest.root_sha256 != trust_root_sha256(root).map_err(|_| PublisherError::Rejected)? {
        return Err(PublisherError::Rejected);
    }
    let trust = verify_marketplace_trust_bytes(
        trust_bytes,
        root,
        &root.channel_id,
        &manifest.channel_origin,
        now_unix,
    )
    .map_err(|_| PublisherError::Rejected)?;
    if trust.payload().marketplace_origin != manifest.marketplace_origin
        || trust.payload().bundle_version != manifest.bundle_version
        || trust.payload().current_snapshot_version != manifest.snapshot_version
        || trust.payload().not_before_unix != manifest.created_at_unix
        || sha256(trust_bytes) != manifest.trust_sha256
    {
        return Err(PublisherError::Rejected);
    }
    let active = trust.active_key();
    if catalog_key_sha256(active).map_err(|_| PublisherError::Rejected)?
        != manifest.catalog_key_sha256
    {
        return Err(PublisherError::Rejected);
    }
    let snapshot = verify_marketplace_snapshot_bytes(snapshot_bytes, active)
        .map_err(|_| PublisherError::Rejected)?;
    trust
        .authorize_new_snapshot(active, snapshot.snapshot_version)
        .map_err(|_| PublisherError::Rejected)?;
    if snapshot.snapshot_version != manifest.snapshot_version
        || sha256(snapshot_bytes) != manifest.snapshot_sha256
    {
        return Err(PublisherError::Rejected);
    }
    Ok((trust, snapshot))
}

pub(crate) fn validate_authenticated_inventory(
    manifest: &PublicationManifest,
    trust: &MarketplaceTrust,
    snapshot: &MarketplaceSnapshotPayload,
) -> Result<()> {
    let mut authenticated = BTreeMap::from([
        (
            (PublicationNamespace::Channel, TRUST_FILE.to_owned()),
            "application/json",
        ),
        (
            (PublicationNamespace::Marketplace, SNAPSHOT_FILE.to_owned()),
            "application/json",
        ),
    ]);
    for entry in &snapshot.releases {
        for name in [
            RELEASE_ARCHIVE_PATH,
            RELEASE_CONFORMANCE_PATH,
            RELEASE_ATTESTATION_PATH,
        ] {
            authenticated.insert(
                (
                    PublicationNamespace::Marketplace,
                    format!("{}{name}", entry.release_path),
                ),
                if name == RELEASE_ARCHIVE_PATH {
                    "application/octet-stream"
                } else {
                    "application/json"
                },
            );
        }
    }
    for artifact in &trust.payload().packages {
        let key = (
            PublicationNamespace::Channel,
            artifact.relative_path.clone(),
        );
        authenticated.insert(key.clone(), "application/vnd.archlinux.package");
        let record = publication_record(manifest, &key.0, &key.1)?;
        if record.media_type != "application/vnd.archlinux.package"
            || record.bytes != artifact.bytes
            || record.sha256 != artifact.sha256
        {
            return Err(PublisherError::Rejected);
        }
    }
    let inventory = manifest
        .files
        .iter()
        .map(|record| {
            (
                (record.namespace.clone(), record.relative_path.clone()),
                record.media_type.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if inventory != authenticated {
        return Err(PublisherError::Rejected);
    }
    Ok(())
}

pub(crate) fn validate_release_entry(
    entry: &MarketplaceReleaseEntry,
    archive: &[u8],
    conformance: &[u8],
    attestation: &[u8],
    catalog_key: &omarchygs_game_cartridge::CatalogPublicKey,
) -> Result<()> {
    let sdk =
        omarchygs_game_cartridge::supported_sdk_identity().map_err(|_| PublisherError::Rejected)?;
    let release = verify_release_components(
        archive,
        conformance,
        attestation,
        &entry.publisher_key,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| PublisherError::Rejected)?;
    let payload = release.payload();
    if payload.game_key != entry.game_key
        || payload.publisher_id != entry.publisher_id
        || payload.rules_version != entry.rules_version
        || payload.cartridge_version != entry.cartridge_version
        || payload.archive_sha256 != entry.archive_sha256
        || payload.signed_identity_sha256 != entry.signed_identity_sha256
    {
        return Err(PublisherError::Rejected);
    }
    omarchygs_game_cartridge::verify_catalog_policy(&entry.policy, catalog_key, &release)
        .map(|_| ())
        .map_err(|_| PublisherError::Rejected)
}

pub(crate) fn publication_record<'a>(
    manifest: &'a PublicationManifest,
    namespace: &PublicationNamespace,
    relative_path: &str,
) -> Result<&'a PublicationFile> {
    manifest
        .files
        .iter()
        .find(|record| record.namespace == *namespace && record.relative_path == relative_path)
        .ok_or(PublisherError::Rejected)
}

pub(crate) fn validate_manifest(manifest: &PublicationManifest) -> Result<()> {
    if manifest.format != MANIFEST_FORMAT
        || !valid_identifier(&manifest.publication_id)
        || manifest.publication_id != publication_id(manifest.bundle_version)
        || manifest.created_at_unix == 0
        || manifest.bundle_version == 0
        || manifest.snapshot_version == 0
        || !valid_origin(&manifest.channel_origin)
        || !valid_origin(&manifest.marketplace_origin)
        || !valid_sha256(&manifest.root_sha256)
        || !valid_sha256(&manifest.catalog_key_sha256)
        || !valid_sha256(&manifest.trust_sha256)
        || !valid_sha256(&manifest.snapshot_sha256)
    {
        return Err(PublisherError::InvalidInput);
    }
    validate_file_inventory(&manifest.files, true)
}

pub(crate) fn manifest_sha256(manifest: &PublicationManifest) -> Result<String> {
    Ok(sha256(&canonical_json(manifest)?))
}

pub(crate) fn receipt_for_manifest(
    operation: &str,
    manifest: &PublicationManifest,
) -> Result<OperationReceipt> {
    let publication_sha256 = manifest_sha256(manifest)?;
    Ok(OperationReceipt {
        format: RECEIPT_FORMAT.to_owned(),
        ok: true,
        operation: operation.to_owned(),
        publication_id: manifest.publication_id.clone(),
        publication_sha256: Some(publication_sha256.clone()),
        evidence_sha256: publication_sha256,
        bundle_version: manifest.bundle_version,
        snapshot_version: manifest.snapshot_version,
        root_sha256: manifest.root_sha256.clone(),
        catalog_key_sha256: manifest.catalog_key_sha256.clone(),
        files: manifest.files.len(),
        recorded_at_unix: manifest.created_at_unix,
    })
}

fn validate_plan(plan: &PublicationPlan) -> Result<()> {
    if plan.format != PLAN_FORMAT
        || !valid_identifier(&plan.publication_id)
        || plan.publication_id != publication_id(plan.bundle_version)
        || plan.created_at_unix == 0
        || plan.created_at_unix != plan.not_before_unix
        || plan.ceremony_unix == 0
        || plan.ceremony_unix < plan.created_at_unix
        || !valid_identifier(&plan.channel_id)
        || !valid_text(&plan.channel_name, 128)
        || !valid_origin(&plan.channel_origin)
        || !valid_origin(&plan.marketplace_origin)
        || !valid_identifier(&plan.marketplace_authority_id)
        || !valid_text(&plan.marketplace_name, 128)
        || plan.bundle_version == 0
        || plan.snapshot_version == 0
        || plan.not_before_unix == 0
        || plan.expires_at_unix <= plan.not_before_unix
        || plan.releases.len() > MAX_MARKETPLACE_RELEASES
        || plan.packages.len() > MAX_PACKAGE_ARTIFACTS
        || plan.keys.is_empty()
    {
        return Err(PublisherError::InvalidInput);
    }
    let mut release_order = None;
    let mut release_inputs = BTreeSet::new();
    let mut release_paths = BTreeSet::new();
    for release in &plan.releases {
        if !valid_relative_path(&release.input_directory, false)
            || !valid_relative_path(&release.publisher_key_path, false)
            || !valid_relative_path(&release.release_path, true)
            || release.policy_version == 0
            || !valid_identifier(&release.reviewed_by)
            || !valid_text(&release.reason, 512)
            || !valid_text(&release.review_summary, 512)
            || release_order
                .as_ref()
                .is_some_and(|prior: &String| prior >= &release.input_directory)
            || !release_inputs.insert(release.input_directory.clone())
            || !release_paths.insert(release.release_path.clone())
        {
            return Err(PublisherError::InvalidInput);
        }
        release_order = Some(release.input_directory.clone());
    }
    let mut package_order = None;
    let mut package_paths = BTreeSet::new();
    for package in &plan.packages {
        if !valid_relative_path(&package.input_path, false)
            || !valid_relative_path(&package.relative_path, false)
            || !valid_identifier(&package.platform)
            || !valid_identifier(&package.architecture)
            || !valid_version(&package.package_version)
            || !valid_filename(&package.filename)
            || !package.relative_path.ends_with(&package.filename)
            || !valid_revision(&package.source_revision)
            || !valid_sha256(&package.source_sha256)
            || !valid_sha256(&package.build_provenance_sha256)
            || package_order
                .as_ref()
                .is_some_and(|prior: &String| prior >= &package.input_path)
            || !package_paths.insert(package.relative_path.clone())
        {
            return Err(PublisherError::InvalidInput);
        }
        package_order = Some(package.input_path.clone());
    }
    if plan
        .previous_trust_sha256
        .as_ref()
        .is_some_and(|value| !valid_sha256(value))
    {
        return Err(PublisherError::InvalidInput);
    }
    Ok(())
}

fn validate_request(request: &OfflineSigningRequest) -> Result<()> {
    if request.format != REQUEST_FORMAT
        || !valid_identifier(&request.publication_id)
        || !valid_sha256(&request.plan_sha256)
        || !valid_sha256(&request.prepared_files_sha256)
        || request.ceremony_unix == 0
    {
        return Err(PublisherError::InvalidInput);
    }
    validate_marketplace_trust_payload(&request.trust_payload, &request.root)
        .map_err(|_| PublisherError::Rejected)?;
    if request.ceremony_unix < request.trust_payload.not_before_unix
        || request.ceremony_unix >= request.trust_payload.expires_at_unix
    {
        return Err(PublisherError::Rejected);
    }
    if let Some(previous_signed) = request.previous_trust.as_ref() {
        let previous = verify_marketplace_trust_bytes_at_rest(
            &canonical_json(previous_signed)?,
            &request.root,
            &request.trust_payload.channel_id,
            &request.trust_payload.channel_origin,
        )
        .map_err(|_| PublisherError::Rejected)?;
        validate_marketplace_trust_transition(&previous, &request.trust_payload, &request.root)
            .map_err(|_| PublisherError::Rollback)?;
    } else if request.trust_payload.bundle_version != 1 {
        return Err(PublisherError::Rollback);
    }
    Ok(())
}

fn load_previous_trust(
    path: Option<&Path>,
    root: &TrustRootPublicKey,
    plan: &PublicationPlan,
    next: Option<&MarketplaceTrustPayload>,
) -> Result<Option<SignedMarketplaceTrust>> {
    match (path, plan.previous_trust_sha256.as_ref()) {
        (None, None) if plan.bundle_version == 1 => Ok(None),
        (Some(path), Some(expected)) => {
            let bytes = read_regular_file(path, MAX_TRUST_CHANNEL_BYTES as u64, false)?;
            if sha256(&bytes) != *expected {
                return Err(PublisherError::Rejected);
            }
            let signed: SignedMarketplaceTrust = parse_canonical(&bytes, MAX_TRUST_CHANNEL_BYTES)?;
            let previous = verify_marketplace_trust_bytes_at_rest(
                &bytes,
                root,
                &plan.channel_id,
                &plan.channel_origin,
            )
            .map_err(|_| PublisherError::Rejected)?;
            if plan.bundle_version != previous.payload().bundle_version.saturating_add(1) {
                return Err(PublisherError::Rollback);
            }
            if let Some(next) = next {
                validate_marketplace_trust_transition(&previous, next, root)
                    .map_err(|_| PublisherError::Rollback)?;
            }
            Ok(Some(signed))
        }
        _ => Err(PublisherError::Rollback),
    }
}

pub(crate) fn validate_file_inventory(
    files: &[PublicationFile],
    require_trust: bool,
) -> Result<()> {
    if files.is_empty() || files.len() > MAX_PUBLICATION_FILES {
        return Err(PublisherError::InvalidInput);
    }
    let mut prior = None;
    let mut paths = BTreeSet::new();
    let mut has_snapshot = false;
    let mut has_trust = false;
    for file in files {
        if !valid_relative_path(&file.relative_path, false)
            || !valid_media_type(&file.media_type)
            || file.bytes == 0
            || file.bytes > file_limit(file)
            || !valid_sha256(&file.sha256)
        {
            return Err(PublisherError::InvalidInput);
        }
        let key = (file.namespace.clone(), file.relative_path.clone());
        if prior.as_ref().is_some_and(|value| value >= &key) || !paths.insert(key.clone()) {
            return Err(PublisherError::InvalidInput);
        }
        has_snapshot |= file.namespace == PublicationNamespace::Marketplace
            && file.relative_path == SNAPSHOT_FILE;
        has_trust |=
            file.namespace == PublicationNamespace::Channel && file.relative_path == TRUST_FILE;
        prior = Some(key);
    }
    if !has_snapshot || (require_trust && !has_trust) || (!require_trust && has_trust) {
        return Err(PublisherError::InvalidInput);
    }
    Ok(())
}

fn receipt(
    operation: &str,
    plan: &PublicationPlan,
    publication_sha256: Option<&str>,
    evidence_sha256: &str,
    root_sha256: String,
    catalog_key_sha256: String,
    files: usize,
) -> Result<OperationReceipt> {
    Ok(OperationReceipt {
        format: RECEIPT_FORMAT.to_owned(),
        ok: true,
        operation: operation.to_owned(),
        publication_id: plan.publication_id.clone(),
        publication_sha256: publication_sha256.map(str::to_owned),
        evidence_sha256: evidence_sha256.to_owned(),
        bundle_version: plan.bundle_version,
        snapshot_version: plan.snapshot_version,
        root_sha256,
        catalog_key_sha256,
        files,
        recorded_at_unix: plan.created_at_unix,
    })
}

fn receipt_from_request(
    operation: &str,
    request: &OfflineSigningRequest,
    publication_sha256: Option<&str>,
    evidence_sha256: &str,
    files: usize,
) -> Result<OperationReceipt> {
    let active = request
        .trust_payload
        .keys
        .last()
        .ok_or(PublisherError::Rejected)?;
    Ok(OperationReceipt {
        format: RECEIPT_FORMAT.to_owned(),
        ok: true,
        operation: operation.to_owned(),
        publication_id: request.publication_id.clone(),
        publication_sha256: publication_sha256.map(str::to_owned),
        evidence_sha256: evidence_sha256.to_owned(),
        bundle_version: request.trust_payload.bundle_version,
        snapshot_version: request.trust_payload.current_snapshot_version,
        root_sha256: trust_root_sha256(&request.root).map_err(|_| PublisherError::Rejected)?,
        catalog_key_sha256: active.key_sha256.clone(),
        files,
        recorded_at_unix: request.ceremony_unix,
    })
}

fn publication_file(
    namespace: PublicationNamespace,
    relative_path: String,
    media_type: &str,
    bytes: &[u8],
) -> PublicationFile {
    PublicationFile {
        namespace,
        relative_path,
        media_type: media_type.to_owned(),
        bytes: bytes.len() as u64,
        sha256: sha256(bytes),
    }
}

pub(crate) fn publication_file_order(
    left: &PublicationFile,
    right: &PublicationFile,
) -> std::cmp::Ordering {
    (&left.namespace, &left.relative_path).cmp(&(&right.namespace, &right.relative_path))
}

pub(crate) fn file_limit(file: &PublicationFile) -> u64 {
    match file.media_type.as_str() {
        "application/vnd.archlinux.package" => MAX_PACKAGE_BYTES,
        "application/octet-stream" => MAX_ARCHIVE_BYTES as u64,
        "application/json" => MAX_PUBLICATION_MANIFEST_BYTES as u64,
        _ => 0,
    }
}

pub(crate) fn parse_canonical<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    maximum: usize,
) -> Result<T> {
    if bytes.is_empty() || bytes.len() > maximum {
        return Err(PublisherError::InvalidInput);
    }
    let value: T = serde_json::from_slice(bytes).map_err(|_| PublisherError::InvalidInput)?;
    if canonical_json(&value)? != bytes {
        return Err(PublisherError::InvalidInput);
    }
    Ok(value)
}

pub(crate) fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| PublisherError::InvalidInput)
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
    if !valid_relative_path(relative, relative.ends_with('/')) {
        return Err(PublisherError::InvalidInput);
    }
    let joined = root.join(relative.trim_end_matches('/'));
    let mut current = root.to_path_buf();
    for component in Path::new(relative.trim_end_matches('/')).components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(PublisherError::Rejected);
            }
            Ok(metadata) if current != joined && !metadata.is_dir() => {
                return Err(PublisherError::Rejected);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(_) => return Err(PublisherError::Storage),
        }
    }
    Ok(joined)
}

pub(crate) fn read_regular_file(path: &Path, maximum: u64, private: bool) -> Result<Vec<u8>> {
    let (file, metadata) = open_regular_file(path, maximum, private)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    std::io::Read::take(file, maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| PublisherError::Storage)?;
    if bytes.is_empty() || bytes.len() as u64 > maximum {
        return Err(PublisherError::Rejected);
    }
    Ok(bytes)
}

fn open_regular_file(path: &Path, maximum: u64, private: bool) -> Result<(fs::File, fs::Metadata)> {
    let link = fs::symlink_metadata(path).map_err(|_| PublisherError::Storage)?;
    if link.file_type().is_symlink()
        || !link.is_file()
        || link.len() == 0
        || link.len() > maximum
        || link.nlink() != 1
        || (private && (link.uid() != geteuid().as_raw() || link.mode() & 0o077 != 0))
    {
        return Err(PublisherError::Rejected);
    }
    let file = open(path, OFlags::RDONLY | OFlags::NOFOLLOW, Mode::empty())
        .map(fs::File::from)
        .map_err(|_| PublisherError::Storage)?;
    let metadata = file.metadata().map_err(|_| PublisherError::Storage)?;
    if metadata.dev() != link.dev()
        || metadata.ino() != link.ino()
        || metadata.len() != link.len()
        || metadata.nlink() != 1
    {
        return Err(PublisherError::Rejected);
    }
    Ok((file, metadata))
}

pub(crate) fn verify_regular_file_exact(path: &Path, record: &PublicationFile) -> Result<()> {
    let (bytes, digest) = digest_regular_file(path, file_limit(record))?;
    if bytes != record.bytes || digest != record.sha256 {
        Err(PublisherError::Rejected)
    } else {
        Ok(())
    }
}

fn digest_regular_file(path: &Path, maximum: u64) -> Result<(u64, String)> {
    let (mut file, metadata) = open_regular_file(path, maximum, false)?;
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes = 0_u64;
    let mut digest = Sha256::new();
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| PublisherError::Storage)?;
        if read == 0 {
            break;
        }
        bytes = bytes
            .checked_add(read as u64)
            .filter(|bytes| *bytes <= maximum)
            .ok_or(PublisherError::Rejected)?;
        digest.update(&buffer[..read]);
    }
    let after = file.metadata().map_err(|_| PublisherError::Storage)?;
    if bytes == 0
        || bytes != metadata.len()
        || after.dev() != metadata.dev()
        || after.ino() != metadata.ino()
        || after.len() != metadata.len()
    {
        return Err(PublisherError::Rejected);
    }
    Ok((
        bytes,
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    ))
}

pub(crate) fn copy_public_file(
    source: &Path,
    destination: &Path,
    maximum: u64,
) -> Result<(u64, String)> {
    let (mut source_file, source_metadata) = open_regular_file(source, maximum, false)?;
    if let Some(parent) = destination.parent() {
        create_directory_all(parent, 0o700)?;
    }
    let mut destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o444)
        .open(destination)
        .map_err(|_| PublisherError::Storage)?;
    let result = (|| {
        let mut buffer = [0_u8; 64 * 1024];
        let mut bytes = 0_u64;
        let mut digest = Sha256::new();
        loop {
            let read = source_file
                .read(&mut buffer)
                .map_err(|_| PublisherError::Storage)?;
            if read == 0 {
                break;
            }
            bytes = bytes
                .checked_add(read as u64)
                .filter(|bytes| *bytes <= maximum)
                .ok_or(PublisherError::Rejected)?;
            digest.update(&buffer[..read]);
            destination_file
                .write_all(&buffer[..read])
                .map_err(|_| PublisherError::Storage)?;
        }
        let after = source_file
            .metadata()
            .map_err(|_| PublisherError::Storage)?;
        if bytes == 0
            || bytes != source_metadata.len()
            || after.dev() != source_metadata.dev()
            || after.ino() != source_metadata.ino()
            || after.len() != source_metadata.len()
        {
            return Err(PublisherError::Rejected);
        }
        destination_file
            .set_permissions(Permissions::from_mode(0o444))
            .map_err(|_| PublisherError::Storage)?;
        destination_file
            .sync_all()
            .map_err(|_| PublisherError::Storage)?;
        Ok((
            bytes,
            digest
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        ))
    })();
    if result.is_err() {
        let _ = fs::remove_file(destination);
    }
    result
}

pub(crate) fn write_public_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_directory_all(parent, 0o700)?;
    }
    write_new_file(path, bytes, 0o444)
}

pub(crate) fn write_new_file(path: &Path, bytes: &[u8], mode: u32) -> Result<()> {
    if bytes.is_empty() {
        return Err(PublisherError::InvalidInput);
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)
        .map_err(|_| PublisherError::Storage)?;
    file.write_all(bytes).map_err(|_| PublisherError::Storage)?;
    file.set_permissions(Permissions::from_mode(mode))
        .map_err(|_| PublisherError::Storage)?;
    file.sync_all().map_err(|_| PublisherError::Storage)?;
    Ok(())
}

pub(crate) fn create_directory(path: &Path, mode: u32) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.mode(mode);
    builder.create(path).map_err(|_| PublisherError::Storage)?;
    let directory = open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|_| PublisherError::Storage)?;
    fs::File::from(directory)
        .set_permissions(Permissions::from_mode(mode))
        .map_err(|_| PublisherError::Storage)
}

pub(crate) fn create_directory_all(path: &Path, mode: u32) -> Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(PublisherError::Rejected);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_directory(&current, mode)?;
            }
            Err(_) => return Err(PublisherError::Storage),
        }
    }
    Ok(())
}

pub(crate) fn require_absolute(path: &Path) -> Result<()> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(PublisherError::InvalidInput)
    }
}

pub(crate) fn require_absolute_private_directory(path: &Path) -> Result<()> {
    require_absolute(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| PublisherError::Storage)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        Err(PublisherError::Rejected)
    } else {
        Ok(())
    }
}

fn require_absolute_private_file(path: &Path, mode: u32) -> Result<()> {
    require_absolute(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| PublisherError::Storage)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.uid() != geteuid().as_raw()
        || metadata.mode() & 0o777 != mode
        || metadata.nlink() != 1
    {
        Err(PublisherError::Rejected)
    } else {
        Ok(())
    }
}

pub(crate) fn fsync_tree(root: &Path) -> Result<()> {
    let mut directories = vec![root.to_path_buf()];
    let mut index = 0;
    while index < directories.len() {
        let current = directories[index].clone();
        index += 1;
        for entry in fs::read_dir(&current).map_err(|_| PublisherError::Storage)? {
            let entry = entry.map_err(|_| PublisherError::Storage)?;
            let metadata = entry.metadata().map_err(|_| PublisherError::Storage)?;
            if metadata.is_dir() {
                directories.push(entry.path());
            }
        }
    }
    for directory in directories.into_iter().rev() {
        fs::File::open(directory)
            .and_then(|file| file.sync_all())
            .map_err(|_| PublisherError::Storage)?;
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.chars().count() <= maximum && !value.chars().any(char::is_control)
}

fn valid_origin(value: &str) -> bool {
    omarchygs_marketplace_trust::ChannelOrigin::parse(value).is_ok()
}

fn valid_relative_path(value: &str, directory: bool) -> bool {
    if value.is_empty()
        || value.len() > 256
        || value.starts_with('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || directory != value.ends_with('/')
    {
        return false;
    }
    value.trim_end_matches('/').split('/').all(|segment| {
        !segment.is_empty() && segment != "." && segment != ".." && segment.len() <= 96
    })
}

fn valid_filename(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+'))
}

fn valid_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+'))
}

fn valid_revision(value: &str) -> bool {
    (7..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_media_type(value: &str) -> bool {
    matches!(
        value,
        "application/json" | "application/vnd.archlinux.package" | "application/octet-stream"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_and_canonical_documents_fail_closed() {
        for invalid in ["", "/absolute", "../escape", "a/../b", "a//b", "a\\b"] {
            assert!(!valid_relative_path(invalid, false));
        }
        assert!(valid_relative_path("releases/door/1/", true));
        assert!(valid_relative_path("packages/client.pkg.tar.zst", false));

        let bytes = br#"{"format":"wrong"}"#;
        assert!(parse_canonical::<PublicationPlan>(bytes, MAX_PLAN_BYTES as usize).is_err());
    }

    #[test]
    fn file_inventory_requires_order_uniqueness_and_fixed_roots() {
        let snapshot = PublicationFile {
            namespace: PublicationNamespace::Marketplace,
            relative_path: SNAPSHOT_FILE.to_owned(),
            media_type: "application/json".to_owned(),
            bytes: 100,
            sha256: "a".repeat(64),
        };
        assert!(validate_file_inventory(std::slice::from_ref(&snapshot), false).is_ok());
        let duplicate = vec![snapshot.clone(), snapshot];
        assert!(validate_file_inventory(&duplicate, false).is_err());
    }
}
