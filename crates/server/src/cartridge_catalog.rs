use omarchygs_game_cartridge::{
    CatalogPolicy, CatalogPublicKey, CatalogStatus, HostProfile, LifecycleUse,
    MAX_MARKETPLACE_SNAPSHOT_BYTES, MarketplaceReleaseEntry, MarketplaceSnapshotPayload,
    NewLaunchDecision, PublisherPublicKey, SecureCartridgeStore, SignedCatalogPolicy,
    lifecycle_decision,
};
use omarchygs_marketplace_trust::{
    MarketplaceTrust, MarketplaceTrustPayload, verify_persisted_trust_continuity,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use sqlx::{FromRow, PgPool, Postgres, Transaction, types::Json};
use uuid::Uuid;

pub const SNAPSHOT_ADVISORY_LOCK: i64 = 0x4f47_534d_4152_4b54;
const MAX_INVENTORY_RELEASES: i64 = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogError {
    InvalidInput,
    Conflict,
    Denied,
    Internal,
}

impl CatalogError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "catalog_invalid_input",
            Self::Conflict => "catalog_conflict",
            Self::Denied => "catalog_denied",
            Self::Internal => "catalog_internal",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReviewedReleaseInput {
    pub entry: MarketplaceReleaseEntry,
    pub policy: CatalogPolicy,
    pub display_name: String,
    pub compatible: bool,
    pub imported: bool,
}

pub struct SnapshotPublication<'a> {
    pub origin: &'a str,
    pub key: &'a CatalogPublicKey,
    pub payload: &'a MarketplaceSnapshotPayload,
    pub digest: &'a str,
    pub signed_snapshot: &'a [u8],
    pub releases: &'a [ReviewedReleaseInput],
    pub marketplace_trust: Option<&'a MarketplaceTrust>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotPreflight {
    New,
    Replay,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MarketplaceSyncReceipt {
    pub format: &'static str,
    pub marketplace_id: String,
    pub snapshot_version: u64,
    pub snapshot_sha256: String,
    pub releases: usize,
    pub imported: usize,
    pub replayed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatalogInventory {
    pub snapshot: Option<MarketplaceSnapshotSummary>,
    pub releases: Vec<CatalogInventoryRelease>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MarketplaceSnapshotSummary {
    pub marketplace_origin: String,
    pub marketplace_id: String,
    pub marketplace_name: String,
    pub key_id: String,
    pub snapshot_version: u64,
    pub snapshot_sha256: String,
    pub synchronized_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatalogInventoryRelease {
    pub game_key: String,
    pub publisher_id: String,
    pub publisher_key_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub display_name: String,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_custom: Option<PlayerOperatorCustomProvenance>,
    pub policy_version: u64,
    pub policy_status: String,
    pub policy_reason: String,
    pub compatible: bool,
    pub imported: bool,
    pub present: bool,
    pub selected: bool,
    pub effective: bool,
    pub admission_revision: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PlayerCartridgeRelease {
    pub game_key: String,
    pub publisher_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub display_name: String,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketplace: Option<PlayerMarketplaceProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_custom: Option<PlayerOperatorCustomProvenance>,
    pub server_admission: PlayerServerAdmission,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PlayerOperatorCustomProvenance {
    pub provenance_class: &'static str,
    pub operator_name: String,
    pub authority_id: String,
    pub key_id: String,
    pub key_sha256: String,
    pub warning: String,
    pub policy_version: u64,
    pub lifecycle_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PlayerMarketplaceProvenance {
    pub provenance_class: &'static str,
    pub marketplace_id: String,
    pub marketplace_name: String,
    pub reviewed_by: String,
    pub review_summary: String,
    pub policy_version: u64,
    pub lifecycle_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PlayerServerAdmission {
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum CatalogSelection {
    Inactive,
    Release { archive_sha256: String },
    CustomRelease { archive_sha256: String },
}

impl CatalogSelection {
    fn digest(&self) -> Option<&str> {
        match self {
            Self::Inactive => None,
            Self::Release { archive_sha256 } | Self::CustomRelease { archive_sha256 } => {
                Some(archive_sha256)
            }
        }
    }

    fn provenance_class(&self) -> Option<&'static str> {
        match self {
            Self::Inactive => None,
            Self::Release { .. } => Some("marketplace_vetted"),
            Self::CustomRelease { .. } => Some("operator_custom"),
        }
    }

    fn valid(&self) -> bool {
        self.digest().is_none_or(valid_sha256)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CatalogCommand {
    pub idempotency_key: Uuid,
    pub game_key: String,
    pub expected: CatalogSelection,
    pub desired: CatalogSelection,
    pub actor: String,
    pub reason: String,
}

impl CatalogCommand {
    pub fn validate(&self) -> Result<(), CatalogError> {
        if self.idempotency_key.is_nil()
            || !valid_identifier(&self.game_key)
            || !self.expected.valid()
            || !self.desired.valid()
            || self.expected == self.desired
            || !valid_plain_text(&self.actor, 64)
            || !valid_plain_text(&self.reason, 500)
        {
            Err(CatalogError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatalogAuditReceipt {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub game_key: String,
    pub action: String,
    pub previous_archive_sha256: Option<String>,
    pub resulting_archive_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_provenance_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resulting_provenance_class: Option<String>,
    pub admission_revision: u64,
    pub created_at: String,
}

pub fn snapshot_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub async fn preflight_snapshot(
    pool: &PgPool,
    origin: &str,
    key: &CatalogPublicKey,
    payload: &MarketplaceSnapshotPayload,
    digest: &str,
    marketplace_trust: Option<&MarketplaceTrust>,
) -> Result<SnapshotPreflight, CatalogError> {
    let row = sqlx::query_as::<_, SyncStateRow>(
        r#"
        SELECT marketplace_origin, authority_id, key_id, marketplace_name,
               snapshot_version, snapshot_sha256,
               trust_root_sha256, trust_payload,
               to_char(synchronized_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS synchronized_at
        FROM marketplace_sync_state
        WHERE singleton
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| CatalogError::Internal)?;
    verify_marketplace_trust_state(row.as_ref(), marketplace_trust)?;
    compare_snapshot_state(row.as_ref(), origin, key, payload, digest)
}

pub async fn authorize_marketplace_trust(
    pool: &PgPool,
    marketplace_trust: Option<&MarketplaceTrust>,
) -> Result<(), CatalogError> {
    let row = sqlx::query_as::<_, SyncStateRow>(
        r#"
        SELECT marketplace_origin, authority_id, key_id, marketplace_name,
               snapshot_version, snapshot_sha256,
               trust_root_sha256, trust_payload,
               to_char(synchronized_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS synchronized_at
        FROM marketplace_sync_state
        WHERE singleton
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| CatalogError::Internal)?;
    verify_marketplace_trust_state(row.as_ref(), marketplace_trust)
}

pub async fn publish_snapshot(
    pool: &PgPool,
    publication: SnapshotPublication<'_>,
) -> Result<MarketplaceSyncReceipt, CatalogError> {
    let SnapshotPublication {
        origin,
        key,
        payload,
        digest,
        signed_snapshot,
        releases,
        marketplace_trust,
    } = publication;
    if !valid_snapshot_evidence(digest, signed_snapshot) || releases.len() != payload.releases.len()
    {
        return Err(CatalogError::InvalidInput);
    }
    let snapshot_version =
        i64::try_from(payload.snapshot_version).map_err(|_| CatalogError::InvalidInput)?;
    let mut transaction = pool.begin().await.map_err(|_| CatalogError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CatalogError::Internal)?;
    let current = fetch_sync_state_for_update(&mut transaction).await?;
    verify_marketplace_trust_state(current.as_ref(), marketplace_trust)?;
    if compare_snapshot_state(current.as_ref(), origin, key, payload, digest)?
        == SnapshotPreflight::Replay
    {
        retain_replayed_snapshot_evidence(&mut transaction, key, digest, signed_snapshot).await?;
        let release_ids = release_ids_for_payload(&mut transaction, payload).await?;
        retain_release_acquisition_evidence(
            &mut transaction,
            key,
            payload.snapshot_version,
            digest,
            signed_snapshot,
            &release_ids,
        )
        .await?;
        persist_marketplace_trust(&mut transaction, marketplace_trust).await?;
        transaction
            .commit()
            .await
            .map_err(|_| CatalogError::Internal)?;
        return Ok(sync_receipt(payload, digest, releases, true));
    }

    let mut release_ids = Vec::with_capacity(releases.len());
    for release in releases {
        let policy_version =
            i64::try_from(release.policy.policy_version).map_err(|_| CatalogError::InvalidInput)?;
        let rules_version = i64::from(release.entry.rules_version);
        let cartridge_version = i64::from(release.entry.cartridge_version);
        let inserted = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO marketplace_releases (
                game_key, publisher_id, publisher_key, rules_version,
                cartridge_version, archive_sha256, signed_identity_sha256,
                display_name, release_path, reviewed_by, review_summary,
                signed_policy, policy_marketplace_key, policy_snapshot_version,
                policy_version, policy_status, policy_reason,
                compatible, imported, first_seen_snapshot_version,
                last_seen_snapshot_version, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17, $18, $19, $20, $20,
                clock_timestamp(), clock_timestamp()
            )
            ON CONFLICT (archive_sha256) DO UPDATE SET
                release_path = EXCLUDED.release_path,
                reviewed_by = EXCLUDED.reviewed_by,
                review_summary = EXCLUDED.review_summary,
                signed_policy = EXCLUDED.signed_policy,
                policy_marketplace_key = EXCLUDED.policy_marketplace_key,
                policy_snapshot_version = EXCLUDED.policy_snapshot_version,
                policy_version = EXCLUDED.policy_version,
                policy_status = EXCLUDED.policy_status,
                policy_reason = EXCLUDED.policy_reason,
                imported = marketplace_releases.imported OR EXCLUDED.imported,
                last_seen_snapshot_version = EXCLUDED.last_seen_snapshot_version,
                updated_at = clock_timestamp()
            WHERE marketplace_releases.game_key = EXCLUDED.game_key
              AND marketplace_releases.publisher_id = EXCLUDED.publisher_id
              AND marketplace_releases.publisher_key = EXCLUDED.publisher_key
              AND marketplace_releases.rules_version = EXCLUDED.rules_version
              AND marketplace_releases.cartridge_version = EXCLUDED.cartridge_version
              AND marketplace_releases.signed_identity_sha256 = EXCLUDED.signed_identity_sha256
              AND marketplace_releases.display_name = EXCLUDED.display_name
              AND marketplace_releases.compatible = EXCLUDED.compatible
              AND marketplace_releases.policy_version <= EXCLUDED.policy_version
              AND (
                    marketplace_releases.policy_version < EXCLUDED.policy_version
                    OR (
                        marketplace_releases.signed_policy = EXCLUDED.signed_policy
                        AND marketplace_releases.policy_marketplace_key
                            IS NOT DISTINCT FROM EXCLUDED.policy_marketplace_key
                    )
                  )
              AND (
                    marketplace_releases.policy_snapshot_version IS NULL
                    OR marketplace_releases.policy_snapshot_version
                        <= EXCLUDED.policy_snapshot_version
                  )
              AND (
                    marketplace_releases.policy_snapshot_version IS NULL
                    OR marketplace_releases.policy_snapshot_version
                        < EXCLUDED.policy_snapshot_version
                    OR (
                        marketplace_releases.policy_marketplace_key
                            = EXCLUDED.policy_marketplace_key
                        AND marketplace_releases.signed_policy = EXCLUDED.signed_policy
                    )
                  )
            RETURNING id
            "#,
        )
        .bind(&release.entry.game_key)
        .bind(&release.entry.publisher_id)
        .bind(Json(&release.entry.publisher_key))
        .bind(rules_version)
        .bind(cartridge_version)
        .bind(&release.entry.archive_sha256)
        .bind(&release.entry.signed_identity_sha256)
        .bind(&release.display_name)
        .bind(&release.entry.release_path)
        .bind(&release.entry.reviewed_by)
        .bind(&release.entry.review_summary)
        .bind(Json(&release.entry.policy))
        .bind(Json(key))
        .bind(snapshot_version)
        .bind(policy_version)
        .bind(catalog_status_name(release.policy.status))
        .bind(&release.policy.reason)
        .bind(release.compatible)
        .bind(release.imported)
        .bind(snapshot_version)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CatalogError::Internal)?;
        release_ids.push(inserted.ok_or(CatalogError::Conflict)?);
    }

    retain_release_acquisition_evidence(
        &mut transaction,
        key,
        payload.snapshot_version,
        digest,
        signed_snapshot,
        &release_ids,
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO marketplace_sync_state (
            singleton, marketplace_origin, authority_id, key_id,
            marketplace_name, snapshot_version, snapshot_sha256,
            signed_snapshot, marketplace_key, trust_root_sha256,
            trust_payload, synchronized_at
        )
        VALUES (TRUE, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, clock_timestamp())
        ON CONFLICT (singleton) DO UPDATE SET
            key_id = EXCLUDED.key_id,
            marketplace_name = EXCLUDED.marketplace_name,
            snapshot_version = EXCLUDED.snapshot_version,
            snapshot_sha256 = EXCLUDED.snapshot_sha256,
            signed_snapshot = EXCLUDED.signed_snapshot,
            marketplace_key = EXCLUDED.marketplace_key,
            trust_root_sha256 = COALESCE(
                EXCLUDED.trust_root_sha256,
                marketplace_sync_state.trust_root_sha256
            ),
            trust_payload = COALESCE(
                EXCLUDED.trust_payload,
                marketplace_sync_state.trust_payload
            ),
            synchronized_at = EXCLUDED.synchronized_at
        "#,
    )
    .bind(origin)
    .bind(&key.authority_id)
    .bind(&key.key_id)
    .bind(&payload.marketplace_name)
    .bind(snapshot_version)
    .bind(digest)
    .bind(signed_snapshot)
    .bind(Json(key))
    .bind(marketplace_trust.map(|trust| trust.root_sha256().to_owned()))
    .bind(marketplace_trust.map(|trust| Json(trust.payload().clone())))
    .execute(&mut *transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    transaction
        .commit()
        .await
        .map_err(|_| CatalogError::Internal)?;
    Ok(sync_receipt(payload, digest, releases, false))
}

/// Retain the exact signed snapshot evidence when an upgrade synchronizes an
/// already-current Ticket 032 snapshot. The evidence may be filled once but
/// can never be replaced for the same version and digest.
pub async fn retain_snapshot_evidence(
    pool: &PgPool,
    origin: &str,
    key: &CatalogPublicKey,
    payload: &MarketplaceSnapshotPayload,
    digest: &str,
    signed_snapshot: &[u8],
    marketplace_trust: Option<&MarketplaceTrust>,
) -> Result<(), CatalogError> {
    if !valid_snapshot_evidence(digest, signed_snapshot) {
        return Err(CatalogError::InvalidInput);
    }
    let mut transaction = pool.begin().await.map_err(|_| CatalogError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CatalogError::Internal)?;
    let current = fetch_sync_state_for_update(&mut transaction).await?;
    verify_marketplace_trust_state(current.as_ref(), marketplace_trust)?;
    if compare_snapshot_state(current.as_ref(), origin, key, payload, digest)?
        != SnapshotPreflight::Replay
    {
        return Err(CatalogError::Conflict);
    }
    retain_replayed_snapshot_evidence(&mut transaction, key, digest, signed_snapshot).await?;
    let release_ids = release_ids_for_payload(&mut transaction, payload).await?;
    retain_release_acquisition_evidence(
        &mut transaction,
        key,
        payload.snapshot_version,
        digest,
        signed_snapshot,
        &release_ids,
    )
    .await?;
    persist_marketplace_trust(&mut transaction, marketplace_trust).await?;
    transaction
        .commit()
        .await
        .map_err(|_| CatalogError::Internal)
}

pub async fn list_inventory(pool: &PgPool) -> Result<CatalogInventory, CatalogError> {
    let snapshot = sqlx::query_as::<_, SyncStateRow>(
        r#"
        SELECT marketplace_origin, authority_id, key_id, marketplace_name,
               snapshot_version, snapshot_sha256,
               trust_root_sha256, trust_payload,
               to_char(synchronized_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS synchronized_at
        FROM marketplace_sync_state
        WHERE singleton
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| CatalogError::Internal)?;
    let rows = sqlx::query_as::<_, InventoryRow>(
        r#"
        SELECT * FROM (
            SELECT r.game_key, r.publisher_id,
                   r.publisher_key ->> 'key_id' AS publisher_key_id,
                   r.rules_version, r.cartridge_version, r.display_name,
                   r.archive_sha256, r.signed_identity_sha256,
                   'marketplace_vetted'::text AS provenance_class,
                   r.reviewed_by, r.review_summary,
                   NULL::text AS operator_name, NULL::text AS authority_id,
                   NULL::text AS operator_key_id, NULL::text AS operator_key_sha256,
                   NULL::text AS custom_warning,
                   r.policy_version, r.policy_status, r.policy_reason,
                   r.compatible, r.imported, r.last_seen_snapshot_version,
                   c.active_release_id = r.id AS selected,
                   COALESCE(c.admission_revision, 0) AS admission_revision
            FROM marketplace_releases r
            LEFT JOIN server_cartridge_catalogs c ON c.game_key = r.game_key
            UNION ALL
            SELECT r.game_key, r.publisher_id,
                   r.publisher_key ->> 'key_id' AS publisher_key_id,
                   r.rules_version, r.cartridge_version, r.display_name,
                   r.archive_sha256, r.signed_identity_sha256,
                   'operator_custom'::text AS provenance_class,
                   NULL::text AS reviewed_by, NULL::text AS review_summary,
                   r.operator_name, r.operator_key ->> 'authority_id' AS authority_id,
                   r.operator_key ->> 'key_id' AS operator_key_id,
                   r.operator_key_sha256, r.warning AS custom_warning,
                   r.policy_version, r.policy_status, r.policy_reason,
                   r.compatible, r.imported, NULL::bigint AS last_seen_snapshot_version,
                   c.active_custom_release_id = r.id AS selected,
                   COALESCE(c.admission_revision, 0) AS admission_revision
            FROM operator_custom_releases r
            LEFT JOIN server_cartridge_catalogs c ON c.game_key = r.game_key
        ) AS inventory
        ORDER BY game_key, rules_version DESC, cartridge_version DESC, archive_sha256
        LIMIT $1
        "#,
    )
    .bind(MAX_INVENTORY_RELEASES + 1)
    .fetch_all(pool)
    .await
    .map_err(|_| CatalogError::Internal)?;
    if rows.len() as i64 > MAX_INVENTORY_RELEASES {
        return Err(CatalogError::Internal);
    }
    let current_version = snapshot.as_ref().map(|row| row.snapshot_version);
    let releases = rows
        .into_iter()
        .map(|row| inventory_release(row, current_version))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CatalogInventory {
        snapshot: snapshot.map(snapshot_summary).transpose()?,
        releases,
    })
}

pub async fn list_player_catalog(
    pool: &PgPool,
) -> Result<Vec<PlayerCartridgeRelease>, CatalogError> {
    let rows = sqlx::query_as::<_, PlayerCatalogRow>(
        r#"
        SELECT * FROM (
            SELECT r.game_key, r.publisher_id, r.rules_version,
                   r.cartridge_version, r.display_name, r.archive_sha256,
                   r.signed_identity_sha256,
                   'marketplace_vetted'::text AS provenance_class,
                   s.authority_id AS marketplace_id, s.marketplace_name,
                   r.reviewed_by, r.review_summary,
                   NULL::text AS operator_name, NULL::text AS authority_id,
                   NULL::text AS key_id, NULL::text AS key_sha256,
                   NULL::text AS custom_warning,
                   r.policy_version, r.policy_status, r.policy_reason,
                   c.admission_revision
            FROM server_cartridge_catalogs c
            JOIN marketplace_releases r ON r.id = c.active_release_id
            JOIN marketplace_sync_state s ON s.singleton
            WHERE r.imported
              AND r.compatible
              AND r.last_seen_snapshot_version = s.snapshot_version
              AND r.policy_status IN ('active', 'deprecated')
            UNION ALL
            SELECT r.game_key, r.publisher_id, r.rules_version,
                   r.cartridge_version, r.display_name, r.archive_sha256,
                   r.signed_identity_sha256,
                   'operator_custom'::text AS provenance_class,
                   NULL::text AS marketplace_id, NULL::text AS marketplace_name,
                   NULL::text AS reviewed_by, NULL::text AS review_summary,
                   r.operator_name, r.operator_key ->> 'authority_id' AS authority_id,
                   r.operator_key ->> 'key_id' AS key_id,
                   r.operator_key_sha256 AS key_sha256,
                   r.warning AS custom_warning,
                   r.policy_version, r.policy_status, r.policy_reason,
                   c.admission_revision
            FROM server_cartridge_catalogs c
            JOIN operator_custom_releases r ON r.id = c.active_custom_release_id
            WHERE r.imported
              AND r.compatible
              AND r.policy_status IN ('active', 'deprecated')
        ) AS releases
        ORDER BY game_key, rules_version, cartridge_version
        LIMIT 129
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| CatalogError::Internal)?;
    if rows.len() > 128 {
        return Err(CatalogError::Internal);
    }
    rows.into_iter().map(player_release).collect()
}

pub async fn apply_catalog_command(
    pool: &PgPool,
    store: &SecureCartridgeStore,
    marketplace_key: &CatalogPublicKey,
    host: &HostProfile,
    command: &CatalogCommand,
) -> Result<CatalogAuditReceipt, CatalogError> {
    apply_catalog_command_with_sources(pool, store, Some(marketplace_key), None, host, command)
        .await
}

pub async fn apply_catalog_command_with_sources(
    pool: &PgPool,
    store: &SecureCartridgeStore,
    marketplace_key: Option<&CatalogPublicKey>,
    operator_custom_key: Option<&CatalogPublicKey>,
    host: &HostProfile,
    command: &CatalogCommand,
) -> Result<CatalogAuditReceipt, CatalogError> {
    command.validate()?;
    let mut transaction = pool.begin().await.map_err(|_| CatalogError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, $2))")
        .bind(&command.game_key)
        .bind(SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CatalogError::Internal)?;

    if let Some(replay) = fetch_audit_replay(&mut transaction, command.idempotency_key).await? {
        let receipt = exact_audit_replay(replay, command)?;
        transaction
            .commit()
            .await
            .map_err(|_| CatalogError::Internal)?;
        return Ok(receipt);
    }

    sqlx::query(
        r#"
        INSERT INTO server_cartridge_catalogs (game_key)
        VALUES ($1)
        ON CONFLICT (game_key) DO NOTHING
        "#,
    )
    .bind(&command.game_key)
    .execute(&mut *transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    let current = fetch_catalog_state(&mut transaction, &command.game_key).await?;
    if current.selection()? != command.expected {
        return Err(CatalogError::Conflict);
    }

    let desired = match &command.desired {
        CatalogSelection::Inactive => None,
        CatalogSelection::Release { archive_sha256 } => Some(DesiredRelease::Marketplace(
            fetch_activatable_release(&mut transaction, &command.game_key, archive_sha256)
                .await?
                .ok_or(CatalogError::Denied)?,
        )),
        CatalogSelection::CustomRelease { archive_sha256 } => Some(DesiredRelease::Custom(
            fetch_activatable_custom_release(&mut transaction, &command.game_key, archive_sha256)
                .await?
                .ok_or(CatalogError::Denied)?,
        )),
    };
    if let Some(desired) = &desired {
        match desired {
            DesiredRelease::Marketplace(desired) => {
                let marketplace_key = marketplace_key.ok_or(CatalogError::Denied)?;
                let sync_key = fetch_marketplace_key(&mut transaction)
                    .await?
                    .ok_or(CatalogError::Denied)?;
                if sync_key.marketplace_key.as_ref().map(|key| &key.0) != Some(marketplace_key)
                    || desired.policy_marketplace_key.as_ref().map(|key| &key.0)
                        != Some(marketplace_key)
                    || desired
                        .policy_snapshot_version
                        .is_none_or(|version| version <= 0)
                {
                    return Err(CatalogError::Denied);
                }
                resolve_selected_release(store, desired, marketplace_key, host, &command.game_key)?;
            }
            DesiredRelease::Custom(desired) => {
                let operator_custom_key = operator_custom_key.ok_or(CatalogError::Denied)?;
                if &desired.operator_key.0 != operator_custom_key {
                    return Err(CatalogError::Denied);
                }
                resolve_custom_selected_release(
                    store,
                    desired,
                    operator_custom_key,
                    host,
                    &command.game_key,
                )?;
            }
        }
    }

    let action = transition_action(&current, desired.as_ref())?;
    let resulting_marketplace_release_id = desired.as_ref().and_then(|release| match release {
        DesiredRelease::Marketplace(release) => Some(release.id),
        DesiredRelease::Custom(_) => None,
    });
    let resulting_custom_release_id = desired.as_ref().and_then(|release| match release {
        DesiredRelease::Marketplace(_) => None,
        DesiredRelease::Custom(release) => Some(release.id),
    });
    let revision = current
        .admission_revision
        .checked_add(1)
        .ok_or(CatalogError::Internal)?;
    sqlx::query(
        r#"
        UPDATE server_cartridge_catalogs
        SET active_release_id = $2,
            active_custom_release_id = $3,
            admission_revision = $4,
            updated_at = clock_timestamp()
        WHERE id = $1
        "#,
    )
    .bind(current.id)
    .bind(resulting_marketplace_release_id)
    .bind(resulting_custom_release_id)
    .bind(revision)
    .execute(&mut *transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    let row = sqlx::query_as::<_, AuditRow>(
        r#"
        INSERT INTO cartridge_catalog_audit_events (
            operation_id, catalog_id, action, actor, reason,
            previous_archive_sha256, resulting_archive_sha256,
            previous_provenance_class, resulting_provenance_class,
            admission_revision, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, clock_timestamp())
        RETURNING id, operation_id, $11::text AS game_key, action, actor, reason,
                  previous_archive_sha256, resulting_archive_sha256,
                  previous_provenance_class, resulting_provenance_class,
                  admission_revision,
                  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
        "#,
    )
    .bind(command.idempotency_key)
    .bind(current.id)
    .bind(action)
    .bind(&command.actor)
    .bind(&command.reason)
    .bind(&current.archive_sha256)
    .bind(command.desired.digest())
    .bind(current.provenance_class())
    .bind(command.desired.provenance_class())
    .bind(revision)
    .bind(&command.game_key)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    transaction
        .commit()
        .await
        .map_err(|_| CatalogError::Internal)?;
    audit_receipt(row)
}

fn compare_snapshot_state(
    current: Option<&SyncStateRow>,
    origin: &str,
    key: &CatalogPublicKey,
    payload: &MarketplaceSnapshotPayload,
    digest: &str,
) -> Result<SnapshotPreflight, CatalogError> {
    let requested =
        i64::try_from(payload.snapshot_version).map_err(|_| CatalogError::InvalidInput)?;
    let Some(current) = current else {
        return Ok(SnapshotPreflight::New);
    };
    if current.marketplace_origin != origin || current.authority_id != key.authority_id {
        return Err(CatalogError::Denied);
    }
    if requested < current.snapshot_version {
        return Err(CatalogError::Conflict);
    }
    if requested == current.snapshot_version {
        return if current.key_id == key.key_id && current.snapshot_sha256 == digest {
            Ok(SnapshotPreflight::Replay)
        } else {
            Err(CatalogError::Conflict)
        };
    }
    Ok(SnapshotPreflight::New)
}

fn verify_marketplace_trust_state(
    current: Option<&SyncStateRow>,
    marketplace_trust: Option<&MarketplaceTrust>,
) -> Result<(), CatalogError> {
    let Some(current) = current else {
        return Ok(());
    };
    match (
        current.trust_root_sha256.as_deref(),
        current.trust_payload.as_ref(),
        marketplace_trust,
    ) {
        (None, None, _) => Ok(()),
        (Some(root_sha256), Some(Json(payload)), Some(trust)) => {
            verify_persisted_trust_continuity(root_sha256, payload, trust)
                .map_err(|_| CatalogError::Denied)
        }
        (Some(_), Some(_), None) => Err(CatalogError::Denied),
        _ => Err(CatalogError::Internal),
    }
}

async fn persist_marketplace_trust(
    transaction: &mut Transaction<'_, Postgres>,
    marketplace_trust: Option<&MarketplaceTrust>,
) -> Result<(), CatalogError> {
    let Some(trust) = marketplace_trust else {
        return Ok(());
    };
    sqlx::query(
        r#"
        UPDATE marketplace_sync_state
        SET trust_root_sha256 = $1,
            trust_payload = $2
        WHERE singleton
        "#,
    )
    .bind(trust.root_sha256())
    .bind(Json(trust.payload()))
    .execute(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    Ok(())
}

async fn fetch_sync_state_for_update(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Option<SyncStateRow>, CatalogError> {
    sqlx::query_as::<_, SyncStateRow>(
        r#"
        SELECT marketplace_origin, authority_id, key_id, marketplace_name,
               snapshot_version, snapshot_sha256,
               trust_root_sha256, trust_payload,
               to_char(synchronized_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS synchronized_at
        FROM marketplace_sync_state
        WHERE singleton
        FOR UPDATE
        "#,
    )
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)
}

async fn fetch_catalog_state(
    transaction: &mut Transaction<'_, Postgres>,
    game_key: &str,
) -> Result<CatalogStateRow, CatalogError> {
    sqlx::query_as::<_, CatalogStateRow>(
        r#"
        SELECT c.id, c.admission_revision,
               r.id AS release_id, custom.id AS custom_release_id,
               COALESCE(r.archive_sha256, custom.archive_sha256) AS archive_sha256,
               COALESCE(r.rules_version, custom.rules_version) AS rules_version,
               COALESCE(r.cartridge_version, custom.cartridge_version) AS cartridge_version
        FROM server_cartridge_catalogs c
        LEFT JOIN marketplace_releases r ON r.id = c.active_release_id
        LEFT JOIN operator_custom_releases custom ON custom.id = c.active_custom_release_id
        WHERE c.game_key = $1
        FOR UPDATE OF c
        "#,
    )
    .bind(game_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?
    .ok_or(CatalogError::Internal)
}

async fn fetch_activatable_custom_release(
    transaction: &mut Transaction<'_, Postgres>,
    game_key: &str,
    digest: &str,
) -> Result<Option<ActivatableCustomReleaseRow>, CatalogError> {
    sqlx::query_as::<_, ActivatableCustomReleaseRow>(
        r#"
        SELECT id, archive_sha256, rules_version, cartridge_version,
               publisher_key, signed_policy, operator_key
        FROM operator_custom_releases
        WHERE game_key = $1
          AND archive_sha256 = $2
          AND imported
          AND compatible
          AND policy_status IN ('active', 'deprecated')
        FOR SHARE
        "#,
    )
    .bind(game_key)
    .bind(digest)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)
}

fn resolve_selected_release(
    store: &SecureCartridgeStore,
    desired: &ActivatableReleaseRow,
    marketplace_key: &CatalogPublicKey,
    host: &HostProfile,
    game_key: &str,
) -> Result<(), CatalogError> {
    let policy_bytes =
        serde_json::to_vec(&desired.signed_policy.0).map_err(|_| CatalogError::Internal)?;
    store
        .resolve_exact(
            game_key,
            &desired.archive_sha256,
            &desired.publisher_key.0,
            host,
            &policy_bytes,
            marketplace_key,
            LifecycleUse::NewLaunch,
        )
        .map(|_| ())
        .map_err(map_store_error)
}

fn resolve_custom_selected_release(
    store: &SecureCartridgeStore,
    desired: &ActivatableCustomReleaseRow,
    operator_custom_key: &CatalogPublicKey,
    host: &HostProfile,
    game_key: &str,
) -> Result<(), CatalogError> {
    let policy_bytes =
        serde_json::to_vec(&desired.signed_policy.0).map_err(|_| CatalogError::Internal)?;
    store
        .resolve_exact(
            game_key,
            &desired.archive_sha256,
            &desired.publisher_key.0,
            host,
            &policy_bytes,
            operator_custom_key,
            LifecycleUse::NewLaunch,
        )
        .map(|_| ())
        .map_err(map_store_error)
}

fn map_store_error(error: omarchygs_game_cartridge::CartridgeError) -> CatalogError {
    match error {
        omarchygs_game_cartridge::CartridgeError::Io(_) => CatalogError::Internal,
        _ => CatalogError::Denied,
    }
}

async fn fetch_activatable_release(
    transaction: &mut Transaction<'_, Postgres>,
    game_key: &str,
    digest: &str,
) -> Result<Option<ActivatableReleaseRow>, CatalogError> {
    sqlx::query_as::<_, ActivatableReleaseRow>(
        r#"
        SELECT r.id, r.archive_sha256, r.rules_version, r.cartridge_version,
               r.publisher_key, r.signed_policy,
               r.policy_marketplace_key, r.policy_snapshot_version
        FROM marketplace_releases r
        JOIN marketplace_sync_state s ON s.singleton
        WHERE r.game_key = $1
          AND r.archive_sha256 = $2
          AND r.imported
          AND r.compatible
          AND r.last_seen_snapshot_version = s.snapshot_version
          AND r.policy_status IN ('active', 'deprecated')
        FOR SHARE OF r
        "#,
    )
    .bind(game_key)
    .bind(digest)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)
}

async fn fetch_marketplace_key(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Option<MarketplaceKeyRow>, CatalogError> {
    sqlx::query_as::<_, MarketplaceKeyRow>(
        "SELECT marketplace_key FROM marketplace_sync_state WHERE singleton",
    )
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)
}

async fn fetch_audit_replay(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
) -> Result<Option<AuditRow>, CatalogError> {
    sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT a.id, a.operation_id, c.game_key, a.action, a.actor, a.reason,
               a.previous_archive_sha256, a.resulting_archive_sha256,
               a.previous_provenance_class, a.resulting_provenance_class,
               a.admission_revision,
               to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
        FROM cartridge_catalog_audit_events a
        JOIN server_cartridge_catalogs c ON c.id = a.catalog_id
        WHERE a.operation_id = $1
        "#,
    )
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)
}

fn exact_audit_replay(
    row: AuditRow,
    command: &CatalogCommand,
) -> Result<CatalogAuditReceipt, CatalogError> {
    if row.game_key != command.game_key
        || row.previous_archive_sha256.as_deref() != command.expected.digest()
        || row.resulting_archive_sha256.as_deref() != command.desired.digest()
        || row.previous_provenance_class.as_deref() != command.expected.provenance_class()
        || row.resulting_provenance_class.as_deref() != command.desired.provenance_class()
        || row.actor != command.actor
        || row.reason != command.reason
    {
        return Err(CatalogError::Conflict);
    }
    audit_receipt(row)
}

fn transition_action(
    current: &CatalogStateRow,
    desired: Option<&DesiredRelease>,
) -> Result<&'static str, CatalogError> {
    match (current.archive_sha256.as_ref(), desired) {
        (None, Some(_)) => Ok("activate_cartridge"),
        (Some(_), None) => Ok("deactivate_cartridge"),
        (Some(_), Some(desired)) => {
            let previous = (
                current.rules_version.ok_or(CatalogError::Internal)?,
                current.cartridge_version.ok_or(CatalogError::Internal)?,
            );
            let next = (desired.rules_version(), desired.cartridge_version());
            match next.cmp(&previous) {
                std::cmp::Ordering::Less => Ok("rollback_cartridge"),
                std::cmp::Ordering::Greater => Ok("upgrade_cartridge"),
                std::cmp::Ordering::Equal => Err(CatalogError::Denied),
            }
        }
        (None, None) => Err(CatalogError::Denied),
    }
}

fn sync_receipt(
    payload: &MarketplaceSnapshotPayload,
    digest: &str,
    releases: &[ReviewedReleaseInput],
    replayed: bool,
) -> MarketplaceSyncReceipt {
    MarketplaceSyncReceipt {
        format: "omarchygs.marketplace-sync-receipt/v1",
        marketplace_id: payload.authority_id.clone(),
        snapshot_version: payload.snapshot_version,
        snapshot_sha256: digest.to_owned(),
        releases: releases.len(),
        imported: releases.iter().filter(|release| release.imported).count(),
        replayed,
    }
}

fn snapshot_summary(row: SyncStateRow) -> Result<MarketplaceSnapshotSummary, CatalogError> {
    Ok(MarketplaceSnapshotSummary {
        marketplace_origin: row.marketplace_origin,
        marketplace_id: row.authority_id,
        marketplace_name: row.marketplace_name,
        key_id: row.key_id,
        snapshot_version: u64::try_from(row.snapshot_version)
            .map_err(|_| CatalogError::Internal)?,
        snapshot_sha256: row.snapshot_sha256,
        synchronized_at: row.synchronized_at,
    })
}

fn inventory_release(
    row: InventoryRow,
    current_version: Option<i64>,
) -> Result<CatalogInventoryRelease, CatalogError> {
    let present = match row.provenance_class.as_str() {
        "marketplace_vetted" => current_version == row.last_seen_snapshot_version,
        "operator_custom" => true,
        _ => return Err(CatalogError::Internal),
    };
    let permitted = matches!(row.policy_status.as_str(), "active" | "deprecated");
    let operator_custom = if row.provenance_class == "operator_custom" {
        Some(PlayerOperatorCustomProvenance {
            provenance_class: "operator_custom",
            operator_name: row.operator_name.ok_or(CatalogError::Internal)?,
            authority_id: row.authority_id.ok_or(CatalogError::Internal)?,
            key_id: row.operator_key_id.ok_or(CatalogError::Internal)?,
            key_sha256: row.operator_key_sha256.ok_or(CatalogError::Internal)?,
            warning: row.custom_warning.ok_or(CatalogError::Internal)?,
            policy_version: u64::try_from(row.policy_version)
                .map_err(|_| CatalogError::Internal)?,
            lifecycle_status: row.policy_status.clone(),
        })
    } else {
        None
    };
    Ok(CatalogInventoryRelease {
        game_key: row.game_key,
        publisher_id: row.publisher_id,
        publisher_key_id: row.publisher_key_id,
        rules_version: u32::try_from(row.rules_version).map_err(|_| CatalogError::Internal)?,
        cartridge_version: u32::try_from(row.cartridge_version)
            .map_err(|_| CatalogError::Internal)?,
        display_name: row.display_name,
        archive_sha256: row.archive_sha256,
        signed_identity_sha256: row.signed_identity_sha256,
        reviewed_by: row.reviewed_by,
        review_summary: row.review_summary,
        operator_custom,
        policy_version: u64::try_from(row.policy_version).map_err(|_| CatalogError::Internal)?,
        policy_status: row.policy_status,
        policy_reason: row.policy_reason,
        compatible: row.compatible,
        imported: row.imported,
        present,
        selected: row.selected.unwrap_or(false),
        effective: row.selected.unwrap_or(false)
            && present
            && row.imported
            && row.compatible
            && permitted,
        admission_revision: u64::try_from(row.admission_revision)
            .map_err(|_| CatalogError::Internal)?,
    })
}

fn player_release(row: PlayerCatalogRow) -> Result<PlayerCartridgeRelease, CatalogError> {
    let policy_version = u64::try_from(row.policy_version).map_err(|_| CatalogError::Internal)?;
    let (marketplace, operator_custom, warning) = match row.provenance_class.as_str() {
        "marketplace_vetted" => (
            Some(PlayerMarketplaceProvenance {
                provenance_class: "marketplace_vetted",
                marketplace_id: row.marketplace_id.ok_or(CatalogError::Internal)?,
                marketplace_name: row.marketplace_name.ok_or(CatalogError::Internal)?,
                reviewed_by: row.reviewed_by.ok_or(CatalogError::Internal)?,
                review_summary: row.review_summary.ok_or(CatalogError::Internal)?,
                policy_version,
                lifecycle_status: row.policy_status.clone(),
            }),
            None,
            (row.policy_status == "deprecated").then(|| row.policy_reason.clone()),
        ),
        "operator_custom" => {
            let custom_warning = row.custom_warning.ok_or(CatalogError::Internal)?;
            (
                None,
                Some(PlayerOperatorCustomProvenance {
                    provenance_class: "operator_custom",
                    operator_name: row.operator_name.ok_or(CatalogError::Internal)?,
                    authority_id: row.authority_id.ok_or(CatalogError::Internal)?,
                    key_id: row.key_id.ok_or(CatalogError::Internal)?,
                    key_sha256: row.key_sha256.ok_or(CatalogError::Internal)?,
                    warning: custom_warning.clone(),
                    policy_version,
                    lifecycle_status: row.policy_status.clone(),
                }),
                Some(if row.policy_status == "deprecated" {
                    format!("{custom_warning} {}", row.policy_reason)
                } else {
                    custom_warning
                }),
            )
        }
        _ => return Err(CatalogError::Internal),
    };
    Ok(PlayerCartridgeRelease {
        game_key: row.game_key,
        publisher_id: row.publisher_id,
        rules_version: u32::try_from(row.rules_version).map_err(|_| CatalogError::Internal)?,
        cartridge_version: u32::try_from(row.cartridge_version)
            .map_err(|_| CatalogError::Internal)?,
        display_name: row.display_name,
        archive_sha256: row.archive_sha256,
        signed_identity_sha256: row.signed_identity_sha256,
        marketplace,
        operator_custom,
        server_admission: PlayerServerAdmission {
            revision: u64::try_from(row.admission_revision).map_err(|_| CatalogError::Internal)?,
        },
        warning,
    })
}

fn audit_receipt(row: AuditRow) -> Result<CatalogAuditReceipt, CatalogError> {
    Ok(CatalogAuditReceipt {
        id: row.id,
        operation_id: row.operation_id,
        game_key: row.game_key,
        action: row.action,
        previous_archive_sha256: row.previous_archive_sha256,
        resulting_archive_sha256: row.resulting_archive_sha256,
        previous_provenance_class: row.previous_provenance_class,
        resulting_provenance_class: row.resulting_provenance_class,
        admission_revision: u64::try_from(row.admission_revision)
            .map_err(|_| CatalogError::Internal)?,
        created_at: row.created_at,
    })
}

fn catalog_status_name(status: CatalogStatus) -> &'static str {
    match status {
        CatalogStatus::Active => "active",
        CatalogStatus::Deprecated => "deprecated",
        CatalogStatus::Suspended => "suspended",
        CatalogStatus::Revoked => "revoked",
        CatalogStatus::Retired => "retired",
    }
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
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

fn valid_snapshot_evidence(digest: &str, signed_snapshot: &[u8]) -> bool {
    valid_sha256(digest)
        && !signed_snapshot.is_empty()
        && signed_snapshot.len() <= MAX_MARKETPLACE_SNAPSHOT_BYTES
        && snapshot_sha256(signed_snapshot) == digest
}

async fn retain_replayed_snapshot_evidence(
    transaction: &mut Transaction<'_, Postgres>,
    key: &CatalogPublicKey,
    digest: &str,
    signed_snapshot: &[u8],
) -> Result<(), CatalogError> {
    let result = sqlx::query(
        r#"
        UPDATE marketplace_sync_state
        SET signed_snapshot = $1,
            marketplace_key = $2
        WHERE singleton
          AND snapshot_sha256 = $3
          AND (
                signed_snapshot IS NULL
                OR (
                    signed_snapshot = $1
                    AND marketplace_key = $2
                )
              )
        "#,
    )
    .bind(signed_snapshot)
    .bind(Json(key))
    .bind(digest)
    .execute(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(CatalogError::Conflict)
    }
}

async fn release_ids_for_payload(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &MarketplaceSnapshotPayload,
) -> Result<Vec<Uuid>, CatalogError> {
    let digests = payload
        .releases
        .iter()
        .map(|release| release.archive_sha256.clone())
        .collect::<Vec<_>>();
    let ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM marketplace_releases
        WHERE archive_sha256 = ANY($1)
        ORDER BY archive_sha256
        "#,
    )
    .bind(&digests)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    if ids.len() != digests.len() {
        Err(CatalogError::Conflict)
    } else {
        Ok(ids)
    }
}

async fn retain_release_acquisition_evidence(
    transaction: &mut Transaction<'_, Postgres>,
    key: &CatalogPublicKey,
    snapshot_version: u64,
    digest: &str,
    signed_snapshot: &[u8],
    release_ids: &[Uuid],
) -> Result<(), CatalogError> {
    if release_ids.is_empty() {
        return Ok(());
    }
    let existing = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM marketplace_release_acquisition_evidence
        WHERE marketplace_release_id = ANY($1)
        "#,
    )
    .bind(release_ids)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    if usize::try_from(existing).map_err(|_| CatalogError::Internal)? == release_ids.len() {
        return Ok(());
    }
    let snapshot_version =
        i64::try_from(snapshot_version).map_err(|_| CatalogError::InvalidInput)?;
    sqlx::query(
        r#"
        INSERT INTO marketplace_snapshot_acquisition_evidence (
            snapshot_sha256, snapshot_version, marketplace_key, signed_snapshot
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (snapshot_sha256) DO NOTHING
        "#,
    )
    .bind(digest)
    .bind(snapshot_version)
    .bind(Json(key))
    .bind(signed_snapshot)
    .execute(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    let exact = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT snapshot_version = $2
           AND marketplace_key = $3
           AND signed_snapshot = $4
        FROM marketplace_snapshot_acquisition_evidence
        WHERE snapshot_sha256 = $1
        "#,
    )
    .bind(digest)
    .bind(snapshot_version)
    .bind(Json(key))
    .bind(signed_snapshot)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?
    .unwrap_or(false);
    if !exact {
        return Err(CatalogError::Conflict);
    }
    sqlx::query(
        r#"
        INSERT INTO marketplace_release_acquisition_evidence (
            marketplace_release_id, snapshot_sha256
        )
        SELECT release_id, $2
        FROM UNNEST($1::uuid[]) AS release_ids(release_id)
        ON CONFLICT (marketplace_release_id) DO NOTHING
        "#,
    )
    .bind(release_ids)
    .bind(digest)
    .execute(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    let retained = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM marketplace_release_acquisition_evidence
        WHERE marketplace_release_id = ANY($1)
        "#,
    )
    .bind(release_ids)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| CatalogError::Internal)?;
    if usize::try_from(retained).map_err(|_| CatalogError::Internal)? == release_ids.len() {
        Ok(())
    } else {
        Err(CatalogError::Conflict)
    }
}

#[derive(FromRow)]
struct SyncStateRow {
    marketplace_origin: String,
    authority_id: String,
    key_id: String,
    marketplace_name: String,
    snapshot_version: i64,
    snapshot_sha256: String,
    trust_root_sha256: Option<String>,
    trust_payload: Option<Json<MarketplaceTrustPayload>>,
    synchronized_at: String,
}

#[derive(FromRow)]
struct InventoryRow {
    game_key: String,
    publisher_id: String,
    publisher_key_id: String,
    rules_version: i64,
    cartridge_version: i64,
    display_name: String,
    archive_sha256: String,
    signed_identity_sha256: String,
    provenance_class: String,
    reviewed_by: Option<String>,
    review_summary: Option<String>,
    operator_name: Option<String>,
    authority_id: Option<String>,
    operator_key_id: Option<String>,
    operator_key_sha256: Option<String>,
    custom_warning: Option<String>,
    policy_version: i64,
    policy_status: String,
    policy_reason: String,
    compatible: bool,
    imported: bool,
    last_seen_snapshot_version: Option<i64>,
    selected: Option<bool>,
    admission_revision: i64,
}

#[derive(FromRow)]
struct PlayerCatalogRow {
    game_key: String,
    publisher_id: String,
    rules_version: i64,
    cartridge_version: i64,
    display_name: String,
    archive_sha256: String,
    signed_identity_sha256: String,
    provenance_class: String,
    marketplace_id: Option<String>,
    marketplace_name: Option<String>,
    reviewed_by: Option<String>,
    review_summary: Option<String>,
    operator_name: Option<String>,
    authority_id: Option<String>,
    key_id: Option<String>,
    key_sha256: Option<String>,
    custom_warning: Option<String>,
    policy_version: i64,
    policy_status: String,
    policy_reason: String,
    admission_revision: i64,
}

#[derive(FromRow)]
struct CatalogStateRow {
    id: Uuid,
    admission_revision: i64,
    release_id: Option<Uuid>,
    custom_release_id: Option<Uuid>,
    archive_sha256: Option<String>,
    rules_version: Option<i64>,
    cartridge_version: Option<i64>,
}

impl CatalogStateRow {
    fn provenance_class(&self) -> Option<&'static str> {
        match (self.release_id, self.custom_release_id) {
            (Some(_), None) => Some("marketplace_vetted"),
            (None, Some(_)) => Some("operator_custom"),
            (None, None) => None,
            (Some(_), Some(_)) => None,
        }
    }

    fn selection(&self) -> Result<CatalogSelection, CatalogError> {
        match (self.provenance_class(), self.archive_sha256.as_ref()) {
            (None, None) if self.release_id.is_none() && self.custom_release_id.is_none() => {
                Ok(CatalogSelection::Inactive)
            }
            (Some("marketplace_vetted"), Some(archive_sha256)) => Ok(CatalogSelection::Release {
                archive_sha256: archive_sha256.clone(),
            }),
            (Some("operator_custom"), Some(archive_sha256)) => {
                Ok(CatalogSelection::CustomRelease {
                    archive_sha256: archive_sha256.clone(),
                })
            }
            _ => Err(CatalogError::Internal),
        }
    }
}

#[derive(FromRow)]
struct ActivatableReleaseRow {
    id: Uuid,
    archive_sha256: String,
    rules_version: i64,
    cartridge_version: i64,
    publisher_key: Json<PublisherPublicKey>,
    signed_policy: Json<SignedCatalogPolicy>,
    policy_marketplace_key: Option<Json<CatalogPublicKey>>,
    policy_snapshot_version: Option<i64>,
}

#[derive(FromRow)]
struct ActivatableCustomReleaseRow {
    id: Uuid,
    archive_sha256: String,
    rules_version: i64,
    cartridge_version: i64,
    publisher_key: Json<PublisherPublicKey>,
    signed_policy: Json<SignedCatalogPolicy>,
    operator_key: Json<CatalogPublicKey>,
}

enum DesiredRelease {
    Marketplace(ActivatableReleaseRow),
    Custom(ActivatableCustomReleaseRow),
}

impl DesiredRelease {
    fn rules_version(&self) -> i64 {
        match self {
            Self::Marketplace(release) => release.rules_version,
            Self::Custom(release) => release.rules_version,
        }
    }

    fn cartridge_version(&self) -> i64 {
        match self {
            Self::Marketplace(release) => release.cartridge_version,
            Self::Custom(release) => release.cartridge_version,
        }
    }
}

#[derive(FromRow)]
struct MarketplaceKeyRow {
    marketplace_key: Option<Json<CatalogPublicKey>>,
}

#[derive(FromRow)]
struct AuditRow {
    id: Uuid,
    operation_id: Uuid,
    game_key: String,
    action: String,
    actor: String,
    reason: String,
    previous_archive_sha256: Option<String>,
    resulting_archive_sha256: Option<String>,
    previous_provenance_class: Option<String>,
    resulting_provenance_class: Option<String>,
    admission_revision: i64,
    created_at: String,
}

#[allow(dead_code)]
fn _lifecycle_contract(status: CatalogStatus) -> bool {
    !matches!(
        lifecycle_decision(status).new_launch,
        NewLaunchDecision::Deny
    )
}
