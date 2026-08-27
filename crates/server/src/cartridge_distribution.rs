//! Authenticated exact-cartridge distribution from the server's retained
//! immutable store.

use std::sync::Arc;

use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CartridgeAcquisition, CatalogPublicKey, CatalogStatus,
    LifecycleUse, OperatorCustomAcquisition, PublisherPublicKey, SecureCartridgeStore,
    SecureResolution, SignedCatalogPolicy, SignedOperatorCustomRelease, rich_2d_host_profile,
    supported_sdk_identity, verify_acquisition_bytes_with_policy_key,
    verify_operator_custom_acquisition_bytes,
};
use omarchygs_marketplace_trust::MarketplaceTrustPayload;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::{
    cartridge_catalog::{self, SNAPSHOT_ADVISORY_LOCK},
    marketplace_sync::{LocalCatalogConfig, LocalMarketplaceTrust},
    operator_custom::{OperatorCustomAuthority, OperatorCustomPublicConfig},
};

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
    marketplace_trust: Option<LocalMarketplaceTrust>,
    operator_custom: Option<OperatorCustomAuthority>,
}

pub(crate) struct CurrentPolicyEvidence<'a> {
    pub signed_bytes: &'a [u8],
    pub marketplace_key: &'a CatalogPublicKey,
    pub snapshot_version: u64,
}

impl CartridgeDistributionRuntime {
    pub async fn from_configs(
        pool: &PgPool,
        marketplace: Option<&LocalCatalogConfig>,
        operator_custom: Option<&OperatorCustomPublicConfig>,
    ) -> Result<Option<Self>, DistributionError> {
        let Some(store_root) = marketplace
            .map(|config| config.store_root.as_path())
            .or_else(|| operator_custom.map(|config| config.store_root.as_path()))
        else {
            return Ok(None);
        };
        if marketplace.is_some_and(|config| config.store_root.as_path() != store_root)
            || operator_custom.is_some_and(|config| config.store_root.as_path() != store_root)
        {
            return Err(DistributionError::Denied);
        }
        if let Some(config) = marketplace {
            cartridge_catalog::authorize_marketplace_trust(
                pool,
                config.marketplace_trust.channel_trust(),
            )
            .await
            .map_err(|error| match error {
                cartridge_catalog::CatalogError::Internal => DistributionError::Internal,
                _ => DistributionError::Denied,
            })?;
        }
        if let Some(config) = operator_custom {
            crate::operator_custom::authorize_public_authority(pool, config)
                .await
                .map_err(|error| match error {
                    crate::operator_custom::OperatorCustomError::Internal => {
                        DistributionError::Internal
                    }
                    _ => DistributionError::Denied,
                })?;
        }
        let store = SecureCartridgeStore::open_existing(store_root)
            .map_err(|_| DistributionError::Internal)?;
        Ok(Some(Self {
            store: Arc::new(store),
            marketplace_trust: marketplace.map(|config| config.marketplace_trust.clone()),
            operator_custom: operator_custom.map(|config| config.authority.clone()),
        }))
    }

    pub async fn from_local_config(
        pool: &PgPool,
        config: &LocalCatalogConfig,
    ) -> Result<Self, DistributionError> {
        cartridge_catalog::authorize_marketplace_trust(
            pool,
            config.marketplace_trust.channel_trust(),
        )
        .await
        .map_err(|error| match error {
            cartridge_catalog::CatalogError::Internal => DistributionError::Internal,
            _ => DistributionError::Denied,
        })?;
        let store = config
            .open_store()
            .map_err(|_| DistributionError::Internal)?;
        Ok(Self {
            store: Arc::new(store),
            marketplace_trust: Some(config.marketplace_trust.clone()),
            operator_custom: None,
        })
    }

    pub fn from_verified_store(
        store: SecureCartridgeStore,
        marketplace_key: CatalogPublicKey,
    ) -> Self {
        Self {
            store: Arc::new(store),
            marketplace_trust: Some(LocalMarketplaceTrust::Manual(marketplace_key)),
            operator_custom: None,
        }
    }

    pub fn from_verified_store_with_trust(
        store: SecureCartridgeStore,
        marketplace_trust: LocalMarketplaceTrust,
    ) -> Self {
        Self {
            store: Arc::new(store),
            marketplace_trust: Some(marketplace_trust),
            operator_custom: None,
        }
    }

    pub fn operator_custom_authority(&self) -> Option<&OperatorCustomAuthority> {
        self.operator_custom.as_ref()
    }

    fn authorize_operator_custom_key(
        &self,
        key: &CatalogPublicKey,
    ) -> Result<(), DistributionError> {
        if self
            .operator_custom
            .as_ref()
            .is_some_and(|authority| &authority.public_key == key)
        {
            Ok(())
        } else {
            Err(DistributionError::Denied)
        }
    }

    pub fn authorize_marketplace_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<(), DistributionError> {
        self.marketplace_trust
            .as_ref()
            .ok_or(DistributionError::Denied)?
            .authorize_key(key, snapshot_version)
            .map_err(|_| DistributionError::Denied)
    }

    pub fn authorize_current_marketplace_key(
        &self,
        key: &CatalogPublicKey,
        snapshot_version: u64,
    ) -> Result<(), DistributionError> {
        self.marketplace_trust
            .as_ref()
            .ok_or(DistributionError::Denied)?
            .authorize_new_snapshot(key, snapshot_version)
            .map_err(|_| DistributionError::Denied)
    }

    pub(crate) fn authorize_persisted_marketplace_trust(
        &self,
        root_sha256: Option<&str>,
        payload: Option<&MarketplaceTrustPayload>,
    ) -> Result<(), DistributionError> {
        self.marketplace_trust
            .as_ref()
            .ok_or(DistributionError::Denied)?
            .authorize_persisted_state(root_sha256, payload)
            .map_err(|_| DistributionError::Denied)
    }

    /// Re-resolve one exact retained release through the production secure
    /// store under the supplied signed lifecycle policy.
    pub(crate) fn resolve_exact_release(
        &self,
        game_key: &str,
        archive_sha256: &str,
        publisher_key: &PublisherPublicKey,
        policy: CurrentPolicyEvidence<'_>,
        use_kind: LifecycleUse,
    ) -> Result<SecureResolution, DistributionError> {
        self.authorize_current_marketplace_key(policy.marketplace_key, policy.snapshot_version)?;
        self.store
            .resolve_exact(
                game_key,
                archive_sha256,
                publisher_key,
                &rich_2d_host_profile(),
                policy.signed_bytes,
                policy.marketplace_key,
                use_kind,
            )
            .map_err(|_| DistributionError::Denied)
    }

    pub(crate) fn resolve_exact_custom_release(
        &self,
        game_key: &str,
        archive_sha256: &str,
        publisher_key: &PublisherPublicKey,
        signed_policy_bytes: &[u8],
        operator_key: &CatalogPublicKey,
        use_kind: LifecycleUse,
    ) -> Result<SecureResolution, DistributionError> {
        self.authorize_operator_custom_key(operator_key)?;
        self.store
            .resolve_exact(
                game_key,
                archive_sha256,
                publisher_key,
                &rich_2d_host_profile(),
                signed_policy_bytes,
                operator_key,
                use_kind,
            )
            .map_err(|_| DistributionError::Denied)
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
    let provenance = sqlx::query_scalar::<_, String>(
        r#"
        SELECT CASE
            WHEN active_release_id IS NOT NULL THEN 'marketplace_vetted'
            WHEN active_custom_release_id IS NOT NULL THEN 'operator_custom'
        END
        FROM server_cartridge_catalogs
        WHERE game_key = $1
          AND COALESCE(
              (SELECT archive_sha256 FROM marketplace_releases WHERE id = active_release_id),
              (SELECT archive_sha256 FROM operator_custom_releases WHERE id = active_custom_release_id)
          ) = $2
        "#,
    )
    .bind(game_key)
    .bind(archive_sha256)
    .fetch_optional(pool)
    .await
    .map_err(|_| DistributionError::Internal)?
    .ok_or(DistributionError::Denied)?;
    if provenance == "operator_custom" {
        return acquire_custom_current(pool, runtime, game_key, archive_sha256).await;
    }
    if provenance != "marketplace_vetted" {
        return Err(DistributionError::Denied);
    }
    let row = sqlx::query_as::<_, AcquisitionRow>(
        r#"
        SELECT i.id AS server_id,
               r.game_key, r.publisher_id, r.publisher_key,
               r.rules_version, r.cartridge_version,
               r.archive_sha256, r.signed_identity_sha256,
               r.signed_policy, r.policy_marketplace_key,
               r.policy_snapshot_version, c.admission_revision,
               s.signed_snapshot, s.marketplace_key,
               s.snapshot_version AS evidence_snapshot_version,
               s.signed_snapshot AS policy_signed_snapshot,
               s.trust_root_sha256,
               s.trust_payload
        FROM server_cartridge_catalogs c
        JOIN marketplace_releases r ON r.id = c.active_release_id
        JOIN marketplace_sync_state s ON s.singleton
        JOIN server_identity i ON i.singleton
        WHERE c.game_key = $1
          AND c.active_custom_release_id IS NULL
          AND r.archive_sha256 = $2
          AND r.imported
          AND r.compatible
          AND r.last_seen_snapshot_version = s.snapshot_version
          AND r.policy_snapshot_version = s.snapshot_version
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
    build_acquisition(runtime, row, LifecycleUse::NewLaunch)
}

/// Build one participant-authorized acquisition from the exact immutable
/// cartridge presentation pinned to a session. Current catalog selection is
/// deliberately not a release selector for this historical path.
pub async fn acquire_session_exact(
    pool: &PgPool,
    runtime: &CartridgeDistributionRuntime,
    actor_id: Uuid,
    game_session_id: Uuid,
) -> Result<Vec<u8>, DistributionError> {
    if actor_id.is_nil() || game_session_id.is_nil() {
        return Err(DistributionError::InvalidInput);
    }
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| DistributionError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock_shared($1)")
        .bind(SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DistributionError::Internal)?;
    let provenance = sqlx::query_scalar::<_, String>(
        r#"
        SELECT presentation.provenance_class
        FROM game_session_cartridge_presentations AS presentation
        JOIN game_session_participants AS participant
          ON participant.game_session_id = presentation.game_session_id
         AND participant.persona_id = $1
        WHERE presentation.game_session_id = $2
        FOR SHARE OF presentation, participant
        "#,
    )
    .bind(actor_id)
    .bind(game_session_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| DistributionError::Internal)?
    .ok_or(DistributionError::Denied)?;
    if provenance == "operator_custom" {
        let row = fetch_custom_session(&mut transaction, actor_id, game_session_id).await?;
        transaction
            .commit()
            .await
            .map_err(|_| DistributionError::Internal)?;
        return build_custom_acquisition(runtime, row, LifecycleUse::ActiveSession);
    }
    if provenance != "marketplace_vetted" {
        return Err(DistributionError::Denied);
    }
    let row = sqlx::query_as::<_, AcquisitionRow>(
        r#"
        SELECT identity.id AS server_id,
               release.game_key,
               release.publisher_id,
               release.publisher_key,
               release.rules_version,
               release.cartridge_version,
               release.archive_sha256,
               release.signed_identity_sha256,
               release.signed_policy,
               release.policy_marketplace_key,
               release.policy_snapshot_version,
               presentation.admission_revision,
               snapshot.signed_snapshot,
               snapshot.marketplace_key,
               snapshot.snapshot_version AS evidence_snapshot_version,
               policy_snapshot.signed_snapshot AS policy_signed_snapshot,
               policy_snapshot.trust_root_sha256,
               policy_snapshot.trust_payload
        FROM game_session_cartridge_presentations AS presentation
        JOIN game_sessions AS session
          ON session.id = presentation.game_session_id
        JOIN game_session_participants AS participant
          ON participant.game_session_id = session.id
         AND participant.persona_id = $1
        JOIN marketplace_releases AS release
          ON release.id = presentation.marketplace_release_id
        JOIN marketplace_release_acquisition_evidence AS evidence
          ON evidence.marketplace_release_id = release.id
        JOIN marketplace_snapshot_acquisition_evidence AS snapshot
          ON snapshot.snapshot_sha256 = evidence.snapshot_sha256
        JOIN marketplace_sync_state AS policy_snapshot
          ON policy_snapshot.singleton
        JOIN server_identity AS identity
          ON identity.singleton
        WHERE session.id = $2
          AND presentation.provenance_class = 'marketplace_vetted'
          AND release.imported
          AND release.compatible
          AND release.policy_snapshot_version = policy_snapshot.snapshot_version
          AND policy_snapshot.signed_snapshot IS NOT NULL
        FOR SHARE OF session, participant, release, evidence, snapshot, policy_snapshot, identity
        "#,
    )
    .bind(actor_id)
    .bind(game_session_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| DistributionError::Internal)?
    .ok_or(DistributionError::Denied)?;
    transaction
        .commit()
        .await
        .map_err(|_| DistributionError::Internal)?;
    build_acquisition(runtime, row, LifecycleUse::ActiveSession)
}

async fn acquire_custom_current(
    pool: &PgPool,
    runtime: &CartridgeDistributionRuntime,
    game_key: &str,
    archive_sha256: &str,
) -> Result<Vec<u8>, DistributionError> {
    let row = sqlx::query_as::<_, CustomAcquisitionRow>(
        r#"
        SELECT identity.id AS server_id,
               release.game_key, release.publisher_id, release.publisher_key,
               release.rules_version, release.cartridge_version,
               release.archive_sha256, release.signed_identity_sha256,
               release.signed_operator_attestation, release.signed_policy,
               release.operator_key, release.policy_version,
               release.policy_status, catalog.admission_revision
        FROM server_cartridge_catalogs AS catalog
        JOIN operator_custom_releases AS release
          ON release.id = catalog.active_custom_release_id
        JOIN operator_custom_authority AS authority ON authority.singleton
        JOIN server_identity AS identity ON identity.singleton
        WHERE catalog.game_key = $1
          AND release.archive_sha256 = $2
          AND release.imported
          AND release.compatible
          AND release.policy_status IN ('active', 'deprecated')
          AND authority.server_id = identity.id
          AND authority.public_key = release.operator_key
        "#,
    )
    .bind(game_key)
    .bind(archive_sha256)
    .fetch_optional(pool)
    .await
    .map_err(|_| DistributionError::Internal)?
    .ok_or(DistributionError::Denied)?;
    build_custom_acquisition(runtime, row, LifecycleUse::NewLaunch)
}

async fn fetch_custom_session(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor_id: Uuid,
    game_session_id: Uuid,
) -> Result<CustomAcquisitionRow, DistributionError> {
    sqlx::query_as::<_, CustomAcquisitionRow>(
        r#"
        SELECT identity.id AS server_id,
               release.game_key, release.publisher_id, release.publisher_key,
               release.rules_version, release.cartridge_version,
               release.archive_sha256, release.signed_identity_sha256,
               release.signed_operator_attestation, release.signed_policy,
               release.operator_key, release.policy_version,
               release.policy_status, presentation.admission_revision
        FROM game_session_cartridge_presentations AS presentation
        JOIN game_sessions AS session ON session.id = presentation.game_session_id
        JOIN game_session_participants AS participant
          ON participant.game_session_id = session.id
         AND participant.persona_id = $1
        JOIN operator_custom_releases AS release
          ON release.id = presentation.operator_custom_release_id
        JOIN operator_custom_authority AS authority ON authority.singleton
        JOIN server_identity AS identity ON identity.singleton
        WHERE session.id = $2
          AND presentation.provenance_class = 'operator_custom'
          AND release.imported
          AND release.compatible
          AND authority.server_id = identity.id
          AND authority.public_key = release.operator_key
        FOR SHARE OF session, participant, presentation, release, authority, identity
        "#,
    )
    .bind(actor_id)
    .bind(game_session_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| DistributionError::Internal)?
    .ok_or(DistributionError::Denied)
}

fn build_custom_acquisition(
    runtime: &CartridgeDistributionRuntime,
    row: CustomAcquisitionRow,
    use_kind: LifecycleUse,
) -> Result<Vec<u8>, DistributionError> {
    let policy_bytes =
        serde_json::to_vec(&row.signed_policy.0).map_err(|_| DistributionError::Internal)?;
    let signed_operator_bytes = serde_json::to_vec(&row.signed_operator_attestation.0)
        .map_err(|_| DistributionError::Internal)?;
    let resolution = runtime.resolve_exact_custom_release(
        &row.game_key,
        &row.archive_sha256,
        &row.publisher_key.0,
        &policy_bytes,
        &row.operator_key.0,
        use_kind,
    )?;
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
    let document = OperatorCustomAcquisition::from_verified_bytes(
        admission.clone(),
        row.operator_key.0.clone(),
        &signed_operator_bytes,
        &policy_bytes,
        resolution.archive_bytes(),
        resolution.conformance_bytes(),
        resolution.attestation_bytes(),
    )
    .map_err(|_| DistributionError::Internal)?;
    let bytes = document
        .to_bounded_json()
        .map_err(|_| DistributionError::Internal)?;
    let sdk = supported_sdk_identity().map_err(|_| DistributionError::Internal)?;
    let verified = verify_operator_custom_acquisition_bytes(
        &bytes,
        &admission,
        &row.operator_key.0,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| DistributionError::Denied)?;
    let policy_version =
        u64::try_from(row.policy_version).map_err(|_| DistributionError::Internal)?;
    if verified.policy().policy_version != policy_version
        || status_name(verified.policy().status) != row.policy_status
        || verified.policy_bytes() != policy_bytes
    {
        return Err(DistributionError::Denied);
    }
    Ok(bytes)
}

fn build_acquisition(
    runtime: &CartridgeDistributionRuntime,
    row: AcquisitionRow,
    use_kind: LifecycleUse,
) -> Result<Vec<u8>, DistributionError> {
    runtime.authorize_persisted_marketplace_trust(
        row.trust_root_sha256.as_deref(),
        row.trust_payload.as_ref().map(|payload| &payload.0),
    )?;
    let evidence_key = row.marketplace_key.ok_or(DistributionError::Denied)?.0;
    let evidence_snapshot_version =
        u64::try_from(row.evidence_snapshot_version).map_err(|_| DistributionError::Internal)?;
    runtime.authorize_marketplace_key(&evidence_key, evidence_snapshot_version)?;
    let policy_key = row
        .policy_marketplace_key
        .ok_or(DistributionError::Denied)?
        .0;
    let policy_snapshot_version = row
        .policy_snapshot_version
        .and_then(|version| u64::try_from(version).ok())
        .ok_or(DistributionError::Denied)?;
    let policy_bytes =
        serde_json::to_vec(&row.signed_policy.0).map_err(|_| DistributionError::Internal)?;
    let resolution = runtime.resolve_exact_release(
        &row.game_key,
        &row.archive_sha256,
        &row.publisher_key.0,
        CurrentPolicyEvidence {
            signed_bytes: &policy_bytes,
            marketplace_key: &policy_key,
            snapshot_version: policy_snapshot_version,
        },
        use_kind,
    )?;
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
    let document = CartridgeAcquisition::from_verified_bytes_with_policy(
        admission.clone(),
        evidence_key.clone(),
        policy_key.clone(),
        &row.signed_snapshot.ok_or(DistributionError::Denied)?,
        &row.policy_signed_snapshot
            .ok_or(DistributionError::Denied)?,
        resolution.archive_bytes(),
        resolution.conformance_bytes(),
        resolution.attestation_bytes(),
    )
    .map_err(|_| DistributionError::Internal)?;
    let bytes = document
        .to_bounded_json()
        .map_err(|_| DistributionError::Internal)?;
    let sdk = supported_sdk_identity().map_err(|_| DistributionError::Internal)?;
    let verified = verify_acquisition_bytes_with_policy_key(
        &bytes,
        &admission,
        &evidence_key,
        &policy_key,
        &sdk,
        &rich_2d_host_profile(),
    )
    .map_err(|_| DistributionError::Denied)?;
    if verified.snapshot().snapshot_version != evidence_snapshot_version
        || verified.policy_snapshot_version() != policy_snapshot_version
        || verified.policy_bytes() != policy_bytes
    {
        return Err(DistributionError::Denied);
    }
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

fn status_name(status: CatalogStatus) -> &'static str {
    match status {
        CatalogStatus::Active => "active",
        CatalogStatus::Deprecated => "deprecated",
        CatalogStatus::Suspended => "suspended",
        CatalogStatus::Revoked => "revoked",
        CatalogStatus::Retired => "retired",
    }
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
    policy_marketplace_key: Option<Json<CatalogPublicKey>>,
    policy_snapshot_version: Option<i64>,
    admission_revision: i64,
    signed_snapshot: Option<Vec<u8>>,
    marketplace_key: Option<Json<CatalogPublicKey>>,
    evidence_snapshot_version: i64,
    policy_signed_snapshot: Option<Vec<u8>>,
    trust_root_sha256: Option<String>,
    trust_payload: Option<Json<MarketplaceTrustPayload>>,
}

#[derive(FromRow)]
struct CustomAcquisitionRow {
    server_id: Uuid,
    game_key: String,
    publisher_id: String,
    publisher_key: Json<PublisherPublicKey>,
    rules_version: i64,
    cartridge_version: i64,
    archive_sha256: String,
    signed_identity_sha256: String,
    signed_operator_attestation: Json<SignedOperatorCustomRelease>,
    signed_policy: Json<SignedCatalogPolicy>,
    operator_key: Json<CatalogPublicKey>,
    policy_version: i64,
    policy_status: String,
    admission_revision: i64,
}
