use omarchygs_marketplace_trust::{
    ChannelEgressError, ChannelOrigin, GuardedChannelClient, TrustRootPublicKey,
};
use serde::{Deserialize, Serialize};

use crate::{
    MAX_PUBLICATION_MANIFEST_BYTES, PUBLICATION_MANIFEST_FILE, PublicationFile,
    PublicationManifest, PublicationNamespace, PublisherError, Result, SNAPSHOT_FILE, TRUST_FILE,
    canonical_json, file_limit, manifest_sha256, parse_canonical, publication_record, sha256,
    validate_authenticated_inventory, validate_publication_core, validate_release_entry,
};

const PROBE_RECEIPT_FORMAT: &str = "omarchygs.marketplace-publication-probe/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProbeOrigin {
    pub channel_origin: String,
    pub marketplace_origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProbeFloor {
    pub minimum_bundle_version: u64,
    pub minimum_snapshot_version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_publication_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProbeReceipt {
    pub format: String,
    pub ok: bool,
    pub operation: String,
    pub publication_id: String,
    pub publication_sha256: String,
    pub bundle_version: u64,
    pub snapshot_version: u64,
    pub root_sha256: String,
    pub catalog_key_sha256: String,
    pub files: usize,
    pub mirrors: usize,
    pub observed_at_unix: u64,
}

/// Verify one hosted channel/marketplace origin pair through guarded HTTPS.
pub async fn probe_publication(
    channel_origin: &str,
    marketplace_origin: &str,
    root: &TrustRootPublicKey,
    floor: &ProbeFloor,
    now_unix: u64,
) -> Result<ProbeReceipt> {
    validate_floor(floor)?;
    let channel = ChannelOrigin::parse(channel_origin).map_err(map_transport)?;
    let marketplace = ChannelOrigin::parse(marketplace_origin).map_err(map_transport)?;
    let channel_client = GuardedChannelClient::production(channel)
        .await
        .map_err(map_transport)?;
    let marketplace_client = GuardedChannelClient::production(marketplace)
        .await
        .map_err(map_transport)?;
    let (receipt, _) =
        probe_with_clients(&channel_client, &marketplace_client, root, floor, now_unix).await?;
    Ok(receipt)
}

/// Verify several operator-supplied mirrors and require one exact identity.
pub async fn probe_mirrors(
    origins: &[ProbeOrigin],
    root: &TrustRootPublicKey,
    floor: &ProbeFloor,
    now_unix: u64,
) -> Result<ProbeReceipt> {
    if origins.is_empty() || origins.len() > 16 {
        return Err(PublisherError::InvalidInput);
    }
    let mut receipts = Vec::with_capacity(origins.len());
    for origin in origins {
        receipts.push(
            probe_publication(
                &origin.channel_origin,
                &origin.marketplace_origin,
                root,
                floor,
                now_unix,
            )
            .await?,
        );
    }
    combine_receipts(receipts)
}

/// Conformance seam for comparing exact loopback mirrors without weakening
/// the production public-destination policy.
#[doc(hidden)]
pub async fn probe_mirrors_with_clients(
    clients: &[(&GuardedChannelClient, &GuardedChannelClient)],
    root: &TrustRootPublicKey,
    floor: &ProbeFloor,
    now_unix: u64,
) -> Result<ProbeReceipt> {
    if clients.is_empty() || clients.len() > 16 {
        return Err(PublisherError::InvalidInput);
    }
    let mut receipts = Vec::with_capacity(clients.len());
    for (channel, marketplace) in clients {
        receipts.push(
            probe_publication_with_clients(channel, marketplace, root, floor, now_unix).await?,
        );
    }
    combine_receipts(receipts)
}

/// Test seam for exact TLS loopback origins; production callers use
/// [`probe_publication`] or [`probe_mirrors`].
#[doc(hidden)]
pub async fn probe_publication_with_clients(
    channel_client: &GuardedChannelClient,
    marketplace_client: &GuardedChannelClient,
    root: &TrustRootPublicKey,
    floor: &ProbeFloor,
    now_unix: u64,
) -> Result<ProbeReceipt> {
    probe_with_clients(channel_client, marketplace_client, root, floor, now_unix)
        .await
        .map(|(receipt, _)| receipt)
}

async fn probe_with_clients(
    channel_client: &GuardedChannelClient,
    marketplace_client: &GuardedChannelClient,
    root: &TrustRootPublicKey,
    floor: &ProbeFloor,
    now_unix: u64,
) -> Result<(ProbeReceipt, PublicationManifest)> {
    let channel_manifest = channel_client
        .get_bytes(
            PUBLICATION_MANIFEST_FILE,
            MAX_PUBLICATION_MANIFEST_BYTES,
            "application/json",
        )
        .await
        .map_err(map_transport)?;
    let marketplace_manifest = marketplace_client
        .get_bytes(
            PUBLICATION_MANIFEST_FILE,
            MAX_PUBLICATION_MANIFEST_BYTES,
            "application/json",
        )
        .await
        .map_err(map_transport)?;
    if channel_manifest != marketplace_manifest {
        return Err(PublisherError::Rejected);
    }
    let manifest: PublicationManifest =
        parse_canonical(&channel_manifest, MAX_PUBLICATION_MANIFEST_BYTES)?;
    if canonical_json(&manifest)? != channel_manifest {
        return Err(PublisherError::Rejected);
    }
    validate_floor(floor)?;
    let publication_sha256 = manifest_sha256(&manifest)?;
    if manifest.bundle_version < floor.minimum_bundle_version
        || manifest.snapshot_version < floor.minimum_snapshot_version
        || floor
            .expected_publication_sha256
            .as_ref()
            .is_some_and(|expected| expected != &publication_sha256)
    {
        return Err(PublisherError::Rollback);
    }

    let trust_bytes = fetch_record(
        channel_client,
        publication_record(&manifest, &PublicationNamespace::Channel, TRUST_FILE)?,
    )
    .await?;
    let snapshot_bytes = fetch_record(
        marketplace_client,
        publication_record(&manifest, &PublicationNamespace::Marketplace, SNAPSHOT_FILE)?,
    )
    .await?;
    let (trust, snapshot) =
        validate_publication_core(&manifest, &trust_bytes, &snapshot_bytes, root, now_unix)?;
    validate_authenticated_inventory(&manifest, &trust, &snapshot)?;
    let catalog_key = trust.active_key();
    for entry in &snapshot.releases {
        let archive = fetch_record(
            marketplace_client,
            publication_record(
                &manifest,
                &PublicationNamespace::Marketplace,
                &format!(
                    "{}{}",
                    entry.release_path,
                    omarchygs_game_cartridge::RELEASE_ARCHIVE_PATH
                ),
            )?,
        )
        .await?;
        let conformance = fetch_record(
            marketplace_client,
            publication_record(
                &manifest,
                &PublicationNamespace::Marketplace,
                &format!(
                    "{}{}",
                    entry.release_path,
                    omarchygs_game_cartridge::RELEASE_CONFORMANCE_PATH
                ),
            )?,
        )
        .await?;
        let attestation = fetch_record(
            marketplace_client,
            publication_record(
                &manifest,
                &PublicationNamespace::Marketplace,
                &format!(
                    "{}{}",
                    entry.release_path,
                    omarchygs_game_cartridge::RELEASE_ATTESTATION_PATH
                ),
            )?,
        )
        .await?;
        validate_release_entry(entry, &archive, &conformance, &attestation, catalog_key)?;
    }
    for artifact in &trust.payload().packages {
        let record = publication_record(
            &manifest,
            &PublicationNamespace::Channel,
            &artifact.relative_path,
        )?;
        channel_client
            .download_exact(
                &record.relative_path,
                &record.media_type,
                record.bytes,
                &record.sha256,
                &mut tokio::io::sink(),
            )
            .await
            .map_err(map_transport)?;
    }
    let receipt = ProbeReceipt {
        format: PROBE_RECEIPT_FORMAT.to_owned(),
        ok: true,
        operation: "probe".to_owned(),
        publication_id: manifest.publication_id.clone(),
        publication_sha256,
        bundle_version: manifest.bundle_version,
        snapshot_version: manifest.snapshot_version,
        root_sha256: manifest.root_sha256.clone(),
        catalog_key_sha256: manifest.catalog_key_sha256.clone(),
        files: manifest.files.len(),
        mirrors: 1,
        observed_at_unix: now_unix,
    };
    Ok((receipt, manifest))
}

fn validate_floor(floor: &ProbeFloor) -> Result<()> {
    if floor.minimum_bundle_version == 0
        || floor.minimum_snapshot_version == 0
        || floor
            .expected_publication_sha256
            .as_ref()
            .is_some_and(|value| {
                value.len() != 64
                    || !value
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
    {
        Err(PublisherError::InvalidInput)
    } else {
        Ok(())
    }
}

async fn fetch_record(client: &GuardedChannelClient, record: &PublicationFile) -> Result<Vec<u8>> {
    let limit = usize::try_from(file_limit(record)).map_err(|_| PublisherError::Rejected)?;
    let bytes = client
        .get_bytes(&record.relative_path, limit, &record.media_type)
        .await
        .map_err(map_transport)?;
    if bytes.len() as u64 != record.bytes || sha256(&bytes) != record.sha256 {
        return Err(PublisherError::Rejected);
    }
    Ok(bytes)
}

fn map_transport(error: ChannelEgressError) -> PublisherError {
    match error {
        ChannelEgressError::InvalidInput => PublisherError::InvalidInput,
        ChannelEgressError::Unavailable | ChannelEgressError::Internal => {
            PublisherError::Unavailable
        }
        ChannelEgressError::Denied | ChannelEgressError::Rejected => PublisherError::Rejected,
    }
}

fn combine_receipts(receipts: Vec<ProbeReceipt>) -> Result<ProbeReceipt> {
    let mut expected = None;
    let mirrors = receipts.len();
    let mut receipt = None;
    for observed in receipts {
        let identity = (
            observed.publication_sha256.clone(),
            observed.bundle_version,
            observed.snapshot_version,
            observed.root_sha256.clone(),
            observed.catalog_key_sha256.clone(),
        );
        if expected.as_ref().is_some_and(|value| value != &identity) {
            return Err(PublisherError::Rejected);
        }
        expected = Some(identity);
        receipt = Some(observed);
    }
    let mut receipt = receipt.ok_or(PublisherError::InvalidInput)?;
    receipt.operation = "probe_mirrors".to_owned();
    receipt.mirrors = mirrors;
    Ok(receipt)
}
