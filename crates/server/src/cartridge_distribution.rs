//! Authenticated exact-cartridge distribution from the server's retained
//! immutable store.

use std::sync::Arc;

use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CartridgeAcquisition, CatalogPublicKey, LifecycleUse,
    PublisherPublicKey, SecureCartridgeStore, SignedCatalogPolicy, rich_2d_host_profile,
    supported_sdk_identity, verify_acquisition_bytes,
};
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::marketplace_sync::LocalCatalogConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionError {
    InvalidInput,
    Denied,
    Internal,
}

impl DistributionError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "cartridge_acquisition_invalid_input",
            Self::Denied => "cartridge_acquisition_denied",
            Self::Internal => "cartridge_acquisition_internal",
        }
    }
}

#[derive(Clone)]
pub struct CartridgeDistributionRuntime {
    store: Arc<SecureCartridgeStore>,
    marketplace_key: CatalogPublicKey,
}

impl CartridgeDistributionRuntime {
    pub fn from_local_config(config: &LocalCatalogConfig) -> Result<Self, DistributionError> {
        let store = config
            .open_store()
            .map_err(|_| DistributionError::Internal)?;
        Ok(Self {
            store: Arc::new(store),
            marketplace_key: config.marketplace_key.clone(),
        })
    }

    pub fn from_verified_store(
        store: SecureCartridgeStore,
        marketplace_key: CatalogPublicKey,
    ) -> Self {
        Self {
            store: Arc::new(store),
            marketplace_key,
        }
    }
}

/// Build and self-verify one exact acquisition document from the currently
/// effective selected release. The database selection and immutable store must
/// agree; no fallback release is considered.
pub async fn acquire_exact(
    pool: &PgPool,
    runtime: &CartridgeDistributionRuntime,
    game_key: &str,
    archive_sha256: &str,
) -> Result<Vec<u8>, DistributionError> {
    if !valid_identifier(game_key) || !valid_sha256(archive_sha256) {
        return Err(DistributionError::InvalidInput);
    }
    let row = sqlx::query_as::<_, AcquisitionRow>(
        r#"
        SELECT i.id AS server_id,
               r.game_key, r.publisher_id, r.publisher_key,
               r.rules_version, r.cartridge_version,
               r.archive_sha256, r.signed_identity_sha256,
               r.signed_policy, c.admission_revision,
               s.signed_snapshot, s.marketplace_key
        FROM server_cartridge_catalogs c
        JOIN marketplace_releases r ON r.id = c.active_release_id
        JOIN marketplace_sync_state s ON s.singleton
        JOIN server_identity i ON i.singleton
        WHERE c.game_key = $1
          AND r.archive_sha256 = $2
          AND r.imported
          AND r.compatible
          AND r.last_seen_snapshot_version = s.snapshot_version
          AND r.policy_status IN ('active', 'deprecated')
          AND s.signed_snapshot IS NOT NULL
          AND s.marketplace_key IS NOT NULL
        "#,
    )
    .bind(game_key)
    .bind(archive_sha256)
    .fetch_optional(pool)
    .await
    .map_err(|_| DistributionError::Internal)?
    .ok_or(DistributionError::Denied)?;
    let database_key = row.marketplace_key.ok_or(DistributionError::Denied)?.0;
    if database_key != runtime.marketplace_key {
        return Err(DistributionError::Denied);
    }
    let policy_bytes =
        serde_json::to_vec(&row.signed_policy.0).map_err(|_| DistributionError::Internal)?;
    let resolution = runtime
        .store
        .resolve_exact(
            &row.game_key,
            &row.archive_sha256,
            &row.publisher_key.0,
            &rich_2d_host_profile(),
            &policy_bytes,
            &runtime.marketplace_key,
            LifecycleUse::NewLaunch,
        )
        .map_err(|_| DistributionError::Denied)?;
    let admission = AcquisitionServerAdmission {
        server_id: row.server_id.to_string(),
        game_key: row.game_key,
        publisher_id: row.publisher_id,
        rules_version: u32::try_from(row.rules_version).map_err(|_| DistributionError::Internal)?,
        cartridge_version: u32::try_from(row.cartridge_version)
            .map_err(|_| DistributionError::Internal)?,
        archive_sha256: row.archive_sha256,
        signed_identity_sha256: row.signed_identity_sha256,
        admission_revision: u64::try_from(row.admission_revision)
            .map_err(|_| DistributionError::Internal)?,
    };
    let document = CartridgeAcquisition::from_verified_bytes(
        admission.clone(),
        runtime.marketplace_key.clone(),
        &row.signed_snapshot.ok_or(DistributionError::Denied)?,
        resolution.archive_bytes(),
        resolution.conformance_bytes(),
        resolution.attestation_bytes(),
    )
    .map_err(|_| DistributionError::Internal)?;
    let bytes = document
        .to_bounded_json()
        .map_err(|_| DistributionError::Internal)?;
    let sdk = supported_sdk_identity().map_err(|_| DistributionError::Internal)?;
    verify_acquisition_bytes(
        &bytes,
        &admission,
        &runtime.marketplace_key,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| DistributionError::Denied)?;
    Ok(bytes)
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(FromRow)]
struct AcquisitionRow {
    server_id: Uuid,
    game_key: String,
    publisher_id: String,
    publisher_key: Json<PublisherPublicKey>,
    rules_version: i64,
    cartridge_version: i64,
    archive_sha256: String,
    signed_identity_sha256: String,
    signed_policy: Json<SignedCatalogPolicy>,
    admission_revision: i64,
    signed_snapshot: Option<Vec<u8>>,
    marketplace_key: Option<Json<CatalogPublicKey>>,
}
