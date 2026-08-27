use std::{
    env,
    fs::File,
    io::Read as _,
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use omarchygs_game_cartridge::{
    CatalogPublicKey, MAX_ARCHIVE_BYTES, MAX_JSON_BYTES, MAX_MARKETPLACE_SNAPSHOT_BYTES,
    MarketplaceReleaseEntry, SecureCartridgeStore, read_catalog_public_key, rich_2d_host_profile,
    supported_sdk_identity, verify_catalog_policy_bytes, verify_marketplace_snapshot_bytes,
    verify_release_components,
};
use omarchygs_marketplace_trust::{
    MAX_TRUST_CHANNEL_BYTES, MarketplaceTrust, MarketplaceTrustPayload, read_trust_root_public_key,
    verify_marketplace_trust_bytes, verify_persisted_trust_continuity,
};
use sqlx::PgPool;

use crate::{
    cartridge_catalog::{
        self, CatalogError, MarketplaceSyncReceipt, ReviewedReleaseInput, SnapshotPreflight,
    },
    marketplace_egress::{GuardedMarketplaceClient, MarketplaceEgressError, MarketplaceOrigin},
};

const SNAPSHOT_PATH: &str = "snapshot.signed.json";
const MAX_TLS_ROOT_BYTES: usize = 32 * 1024;
const MAX_RELEASE_RECORD_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketplaceSyncError {
    InvalidConfig,
    Unavailable,
    Rejected,
    Conflict,
    Denied,
    Internal,
}

impl MarketplaceSyncError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidConfig => "marketplace_invalid_config",
            Self::Unavailable => "marketplace_unavailable",
            Self::Rejected => "marketplace_rejected",
            Self::Conflict => "marketplace_conflict",
            Self::Denied => "marketplace_denied",
            Self::Internal => "marketplace_internal",
        }
    }
}

#[derive(Clone)]
pub enum LocalMarketplaceTrust {
    Manual(CatalogPublicKey),
    Channel(Arc<MarketplaceTrust>),
}

impl LocalMarketplaceTrust {
    pub fn channel_trust(&self) -> Option<&MarketplaceTrust> {
        match self {
            Self::Manual(_) => None,
            Self::Channel(trust) => Some(trust.as_ref()),
        }
    }

    pub fn active_key(&self) -> Result<CatalogPublicKey, MarketplaceSyncError> {
        match self {
            Self::Manual(key) => Ok(key.clone()),
            Self::Channel(trust) => {
                trust
                    .validate_now(now_unix()?)
                    .map_err(|_| MarketplaceSyncError::Denied)?;
                Ok(trust.active_key().clone())
            }
        }
    }

    /// Confirm that this process's loaded trust is at least as authoritative as
    /// the trust persisted in the request's database snapshot. A newer valid
    /// local bundle may be used while persistence catches up, but a process
    /// holding older or manual trust must fail closed once the database advances.
    pub fn authorize_persisted_state(
        &self,
        persisted_root_sha256: Option<&str>,
        persisted_payload: Option<&MarketplaceTrustPayload>,
    ) -> Result<(), MarketplaceSyncError> {
        match (self, persisted_root_sha256, persisted_payload) {
            (Self::Manual(_), None, None) => Ok(()),
            (Self::Channel(current), Some(root_sha256), Some(payload)) => {
                verify_persisted_trust_continuity(root_sha256, payload, current.as_ref())
                    .map_err(|_| MarketplaceSyncError::Denied)
            }
            _ => Err(MarketplaceSyncError::Denied),
        }
    }

    pub fn authorize_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<(), MarketplaceSyncError> {
        if snapshot_version == 0 {
            return Err(MarketplaceSyncError::Denied);
        }
        match self {
            Self::Manual(expected) if expected == key => Ok(()),
            Self::Manual(_) => Err(MarketplaceSyncError::Denied),
            Self::Channel(trust) => {
                trust
                    .validate_now(now_unix()?)
                    .map_err(|_| MarketplaceSyncError::Denied)?;
                trust
                    .authorize_key(key, snapshot_version)
                    .map(|_| ())
                    .map_err(|_| MarketplaceSyncError::Denied)
            }
        }
    }

    pub fn authorize_new_snapshot(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<(), MarketplaceSyncError> {
        match self {
            Self::Manual(_) => self.authorize_key(key, snapshot_version),
            Self::Channel(trust) => {
                trust
                    .validate_now(now_unix()?)
                    .map_err(|_| MarketplaceSyncError::Denied)?;
                trust
                    .authorize_new_snapshot(key, snapshot_version)
                    .map_err(|_| MarketplaceSyncError::Denied)
            }
        }
    }
}

#[derive(Clone)]
pub struct LocalCatalogConfig {
    pub marketplace_trust: LocalMarketplaceTrust,
    pub store_root: PathBuf,
}

impl LocalCatalogConfig {
    pub fn from_environment() -> Result<Self, MarketplaceSyncError> {
        Self::optional_from_environment()?.ok_or(MarketplaceSyncError::InvalidConfig)
    }

    /// Load the all-or-nothing normal-server distribution configuration.
    /// No trust/store inputs means the metadata-only deployment profile;
    /// manual and root-authenticated channel modes are mutually exclusive.
    pub fn optional_from_environment() -> Result<Option<Self>, MarketplaceSyncError> {
        let key_path = env::var_os("OGS_MARKETPLACE_PUBLIC_KEY").map(PathBuf::from);
        let root_path = env::var_os("OGS_MARKETPLACE_TRUST_ROOT").map(PathBuf::from);
        let bundle_path = env::var_os("OGS_MARKETPLACE_TRUST_BUNDLE").map(PathBuf::from);
        let channel_origin = env::var("OGS_MARKETPLACE_TRUST_CHANNEL_ORIGIN").ok();
        let store_root = env::var_os("OGS_CARTRIDGE_STORE_ROOT").map(PathBuf::from);
        let any_trust = key_path.is_some()
            || root_path.is_some()
            || bundle_path.is_some()
            || channel_origin.is_some();
        if !any_trust {
            if store_root.is_none() {
                return Ok(None);
            }
            let custom_key_present = env::var_os("OGS_CUSTOM_CARTRIDGE_PUBLIC_KEY").is_some()
                || env::var_os("OGS_CUSTOM_CARTRIDGE_PRIVATE_KEY").is_some();
            let custom_name_present = env::var_os("OGS_CUSTOM_CARTRIDGE_OPERATOR_NAME").is_some();
            return if custom_key_present && custom_name_present {
                Ok(None)
            } else {
                Err(MarketplaceSyncError::InvalidConfig)
            };
        }
        let store_root = store_root.ok_or(MarketplaceSyncError::InvalidConfig)?;
        let marketplace_trust = match (key_path, root_path, bundle_path, channel_origin) {
            (Some(key_path), None, None, None) => LocalMarketplaceTrust::Manual(
                read_catalog_public_key(&key_path)
                    .map_err(|_| MarketplaceSyncError::InvalidConfig)?,
            ),
            (None, Some(root_path), Some(bundle_path), Some(channel_origin)) => {
                let root = read_trust_root_public_key(&root_path)
                    .map_err(|_| MarketplaceSyncError::InvalidConfig)?;
                let bundle = read_checked_file(&bundle_path, MAX_TRUST_CHANNEL_BYTES)?;
                let trust = verify_marketplace_trust_bytes(
                    &bundle,
                    &root,
                    &root.channel_id,
                    &channel_origin,
                    now_unix()?,
                )
                .map_err(|_| MarketplaceSyncError::InvalidConfig)?;
                LocalMarketplaceTrust::Channel(Arc::new(trust))
            }
            _ => return Err(MarketplaceSyncError::InvalidConfig),
        };
        SecureCartridgeStore::open_existing(&store_root)
            .map_err(|_| MarketplaceSyncError::InvalidConfig)?;
        Ok(Some(Self {
            marketplace_trust,
            store_root,
        }))
    }

    pub fn open_store(&self) -> Result<SecureCartridgeStore, MarketplaceSyncError> {
        SecureCartridgeStore::open_existing(&self.store_root)
            .map_err(|_| MarketplaceSyncError::InvalidConfig)
    }

    pub fn active_key(&self) -> Result<CatalogPublicKey, MarketplaceSyncError> {
        self.marketplace_trust.active_key()
    }
}

pub struct MarketplaceSyncConfig {
    pub local: LocalCatalogConfig,
    pub origin: MarketplaceOrigin,
    tls_root_der: Vec<u8>,
}

impl MarketplaceSyncConfig {
    pub fn from_environment() -> Result<Self, MarketplaceSyncError> {
        let local = LocalCatalogConfig::from_environment()?;
        let origin = env::var("OGS_MARKETPLACE_ORIGIN")
            .map_err(|_| MarketplaceSyncError::InvalidConfig)
            .and_then(|value| {
                MarketplaceOrigin::parse(&value).map_err(|_| MarketplaceSyncError::InvalidConfig)
            })?;
        let tls_path = env::var_os("OGS_MARKETPLACE_TLS_ROOT_DER")
            .map(PathBuf::from)
            .ok_or(MarketplaceSyncError::InvalidConfig)?;
        let tls_root_der = read_checked_file(&tls_path, MAX_TLS_ROOT_BYTES)?;
        if let LocalMarketplaceTrust::Channel(trust) = &local.marketplace_trust
            && trust.payload().marketplace_origin != origin.as_str()
        {
            return Err(MarketplaceSyncError::InvalidConfig);
        }
        Ok(Self {
            local,
            origin,
            tls_root_der,
        })
    }

    #[cfg(test)]
    pub fn for_test(
        local: LocalCatalogConfig,
        origin: MarketplaceOrigin,
        tls_root_der: Vec<u8>,
    ) -> Self {
        Self {
            local,
            origin,
            tls_root_der,
        }
    }
}

pub async fn synchronize(
    pool: &PgPool,
    config: &MarketplaceSyncConfig,
) -> Result<MarketplaceSyncReceipt, MarketplaceSyncError> {
    let client =
        GuardedMarketplaceClient::production(config.origin.clone(), config.tls_root_der.as_slice())
            .await
            .map_err(map_egress)?;
    synchronize_with_client(pool, config, &client).await
}

pub async fn synchronize_with_client(
    pool: &PgPool,
    config: &MarketplaceSyncConfig,
    client: &GuardedMarketplaceClient,
) -> Result<MarketplaceSyncReceipt, MarketplaceSyncError> {
    let active_key = config.local.active_key()?;
    let channel_trust = config.local.marketplace_trust.channel_trust();
    let snapshot_bytes = client
        .get(SNAPSHOT_PATH, MAX_MARKETPLACE_SNAPSHOT_BYTES)
        .await
        .map_err(map_egress)?;
    let payload = verify_marketplace_snapshot_bytes(&snapshot_bytes, &active_key)
        .map_err(|_| MarketplaceSyncError::Rejected)?;
    config
        .local
        .marketplace_trust
        .authorize_new_snapshot(&active_key, payload.snapshot_version)?;
    let snapshot_digest = cartridge_catalog::snapshot_sha256(&snapshot_bytes);
    match cartridge_catalog::preflight_snapshot(
        pool,
        config.origin.as_str(),
        &active_key,
        &payload,
        &snapshot_digest,
        channel_trust,
    )
    .await
    .map_err(map_catalog)?
    {
        SnapshotPreflight::Replay => {
            cartridge_catalog::retain_snapshot_evidence(
                pool,
                config.origin.as_str(),
                &active_key,
                &payload,
                &snapshot_digest,
                &snapshot_bytes,
                channel_trust,
            )
            .await
            .map_err(map_catalog)?;
            let inventory = cartridge_catalog::list_inventory(pool)
                .await
                .map_err(map_catalog)?;
            return Ok(MarketplaceSyncReceipt {
                format: "omarchygs.marketplace-sync-receipt/v1",
                marketplace_id: payload.authority_id,
                snapshot_version: payload.snapshot_version,
                snapshot_sha256: snapshot_digest,
                releases: payload.releases.len(),
                imported: inventory
                    .releases
                    .iter()
                    .filter(|release| release.present && release.imported)
                    .count(),
                replayed: true,
            });
        }
        SnapshotPreflight::New => {}
    }

    let sdk = supported_sdk_identity().map_err(|_| MarketplaceSyncError::Internal)?;
    let host = rich_2d_host_profile();
    let store = config.local.open_store()?;
    let mut reviewed = Vec::with_capacity(payload.releases.len());
    for entry in &payload.releases {
        let archive =
            download_component(client, entry, "cartridge.ogsc", MAX_ARCHIVE_BYTES).await?;
        let conformance =
            download_component(client, entry, "conformance.json", MAX_RELEASE_RECORD_BYTES).await?;
        let attestation = download_component(
            client,
            entry,
            "release.signed.json",
            MAX_RELEASE_RECORD_BYTES,
        )
        .await?;
        if conformance.len() > MAX_JSON_BYTES || attestation.len() > MAX_JSON_BYTES {
            return Err(MarketplaceSyncError::Rejected);
        }
        let release = verify_release_components(
            &archive,
            &conformance,
            &attestation,
            &entry.publisher_key,
            &sdk,
            &host,
        )
        .map_err(|_| MarketplaceSyncError::Rejected)?;
        if release.payload().game_key != entry.game_key
            || release.payload().publisher_id != entry.publisher_id
            || release.payload().rules_version != entry.rules_version
            || release.payload().cartridge_version != entry.cartridge_version
            || release.payload().archive_sha256 != entry.archive_sha256
            || release.payload().signed_identity_sha256 != entry.signed_identity_sha256
        {
            return Err(MarketplaceSyncError::Rejected);
        }
        let policy_bytes = entry
            .policy_bytes()
            .map_err(|_| MarketplaceSyncError::Rejected)?;
        let policy = verify_catalog_policy_bytes(&policy_bytes, &active_key, &release)
            .map_err(|_| MarketplaceSyncError::Rejected)?;
        let staged = store
            .stage_reviewed_release(&release, &policy_bytes, &active_key)
            .map_err(|error| match error {
                omarchygs_game_cartridge::CartridgeError::Io(_) => MarketplaceSyncError::Internal,
                _ => MarketplaceSyncError::Rejected,
            })?;
        reviewed.push(ReviewedReleaseInput {
            entry: entry.clone(),
            policy,
            display_name: release.cartridge().manifest().display_name.clone(),
            compatible: release.cartridge().compatibility().compatible,
            imported: staged.installed,
        });
    }
    cartridge_catalog::publish_snapshot(
        pool,
        cartridge_catalog::SnapshotPublication {
            origin: config.origin.as_str(),
            key: &active_key,
            payload: &payload,
            digest: &snapshot_digest,
            signed_snapshot: &snapshot_bytes,
            releases: &reviewed,
            marketplace_trust: channel_trust,
        },
    )
    .await
    .map_err(map_catalog)
}

fn now_unix() -> Result<u64, MarketplaceSyncError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| MarketplaceSyncError::InvalidConfig)
}

async fn download_component(
    client: &GuardedMarketplaceClient,
    entry: &MarketplaceReleaseEntry,
    name: &str,
    limit: usize,
) -> Result<Vec<u8>, MarketplaceSyncError> {
    client
        .get(&format!("{}{name}", entry.release_path), limit)
        .await
        .map_err(map_egress)
}

fn read_checked_file(path: &Path, limit: usize) -> Result<Vec<u8>, MarketplaceSyncError> {
    let link_metadata =
        std::fs::symlink_metadata(path).map_err(|_| MarketplaceSyncError::InvalidConfig)?;
    if link_metadata.file_type().is_symlink()
        || !link_metadata.is_file()
        || link_metadata.len() > limit as u64
    {
        return Err(MarketplaceSyncError::InvalidConfig);
    }
    let file = File::open(path).map_err(|_| MarketplaceSyncError::InvalidConfig)?;
    let metadata = file
        .metadata()
        .map_err(|_| MarketplaceSyncError::InvalidConfig)?;
    if !metadata.is_file()
        || metadata.len() > limit as u64
        || metadata.dev() != link_metadata.dev()
        || metadata.ino() != link_metadata.ino()
    {
        return Err(MarketplaceSyncError::InvalidConfig);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| MarketplaceSyncError::InvalidConfig)?;
    if !(64..=limit).contains(&bytes.len()) {
        return Err(MarketplaceSyncError::InvalidConfig);
    }
    Ok(bytes)
}

fn map_egress(error: MarketplaceEgressError) -> MarketplaceSyncError {
    match error {
        MarketplaceEgressError::InvalidInput => MarketplaceSyncError::Rejected,
        MarketplaceEgressError::Denied => MarketplaceSyncError::Denied,
        MarketplaceEgressError::Unavailable => MarketplaceSyncError::Unavailable,
        MarketplaceEgressError::Rejected => MarketplaceSyncError::Rejected,
        MarketplaceEgressError::Internal => MarketplaceSyncError::Internal,
    }
}

fn map_catalog(error: CatalogError) -> MarketplaceSyncError {
    match error {
        CatalogError::InvalidInput => MarketplaceSyncError::Rejected,
        CatalogError::Denied => MarketplaceSyncError::Denied,
        CatalogError::Conflict => MarketplaceSyncError::Conflict,
        CatalogError::Internal => MarketplaceSyncError::Internal,
    }
}
