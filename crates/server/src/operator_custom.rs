//! Server-operator custom cartridge import and public trust configuration.
//!
//! The administrator process is the only process that loads the private key.
//! The normal server receives the matching public key and serves independently
//! verifiable operator-custom evidence without claiming marketplace review.

use std::{
    env,
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
};

use omarchygs_game_cartridge::{
    CatalogPrivateKey, CatalogPublicKey, CatalogStatus, OPERATOR_CUSTOM_WARNING,
    PublisherPublicKey, SecureCartridgeStore, operator_custom_key_sha256, read_catalog_private_key,
    read_catalog_public_key, read_public_key, rich_2d_host_profile, sign_catalog_policy,
    sign_operator_custom_release, signed_operator_custom_release_bytes,
    verify_operator_custom_release_bytes, verify_supported_release_directory,
};
use rustix::process::geteuid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction, types::Json};
use uuid::Uuid;

use crate::cartridge_catalog::SNAPSHOT_ADVISORY_LOCK;

pub const MAX_CUSTOM_COMMAND_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorCustomError {
    InvalidConfig,
    InvalidInput,
    Conflict,
    Denied,
    Internal,
}

impl OperatorCustomError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidConfig => "operator_custom_invalid_config",
            Self::InvalidInput => "operator_custom_invalid_input",
            Self::Conflict => "operator_custom_conflict",
            Self::Denied => "operator_custom_denied",
            Self::Internal => "operator_custom_internal",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OperatorCustomAuthority {
    pub operator_name: String,
    pub authority_id: String,
    pub key_id: String,
    pub key_sha256: String,
    pub public_key: CatalogPublicKey,
}

#[derive(Clone)]
pub struct OperatorCustomPublicConfig {
    pub authority: OperatorCustomAuthority,
    pub store_root: PathBuf,
}

impl OperatorCustomPublicConfig {
    pub fn optional_from_environment() -> Result<Option<Self>, OperatorCustomError> {
        if env::var_os("OGS_CUSTOM_CARTRIDGE_PRIVATE_KEY").is_some() {
            return Err(OperatorCustomError::InvalidConfig);
        }
        let public_key_path = env::var_os("OGS_CUSTOM_CARTRIDGE_PUBLIC_KEY").map(PathBuf::from);
        let operator_name = env::var("OGS_CUSTOM_CARTRIDGE_OPERATOR_NAME").ok();
        let store_root = env::var_os("OGS_CARTRIDGE_STORE_ROOT").map(PathBuf::from);
        if public_key_path.is_none() && operator_name.is_none() {
            return Ok(None);
        }
        let public_key_path = public_key_path.ok_or(OperatorCustomError::InvalidConfig)?;
        let operator_name = operator_name.ok_or(OperatorCustomError::InvalidConfig)?;
        let store_root = store_root.ok_or(OperatorCustomError::InvalidConfig)?;
        if !public_key_path.is_absolute()
            || !store_root.is_absolute()
            || !valid_text(&operator_name, 128)
        {
            return Err(OperatorCustomError::InvalidConfig);
        }
        let public_key = read_catalog_public_key(&public_key_path)
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        let key_sha256 = operator_custom_key_sha256(&public_key)
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        SecureCartridgeStore::open_existing(&store_root)
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        Ok(Some(Self {
            authority: OperatorCustomAuthority {
                operator_name,
                authority_id: public_key.authority_id.clone(),
                key_id: public_key.key_id.clone(),
                key_sha256,
                public_key,
            },
            store_root,
        }))
    }

    pub fn open_store(&self) -> Result<SecureCartridgeStore, OperatorCustomError> {
        SecureCartridgeStore::open_existing(&self.store_root)
            .map_err(|_| OperatorCustomError::InvalidConfig)
    }
}

pub struct OperatorCustomAdminConfig {
    pub operator_name: String,
    pub private_key: CatalogPrivateKey,
    pub public_key: CatalogPublicKey,
    pub key_sha256: String,
    pub store_root: PathBuf,
}

impl OperatorCustomAdminConfig {
    pub fn from_environment() -> Result<Self, OperatorCustomError> {
        let private_key_path = env::var_os("OGS_CUSTOM_CARTRIDGE_PRIVATE_KEY")
            .map(PathBuf::from)
            .ok_or(OperatorCustomError::InvalidConfig)?;
        let public_key_path = env::var_os("OGS_CUSTOM_CARTRIDGE_PUBLIC_KEY")
            .map(PathBuf::from)
            .ok_or(OperatorCustomError::InvalidConfig)?;
        let operator_name = env::var("OGS_CUSTOM_CARTRIDGE_OPERATOR_NAME")
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        let store_root = env::var_os("OGS_CARTRIDGE_STORE_ROOT")
            .map(PathBuf::from)
            .ok_or(OperatorCustomError::InvalidConfig)?;
        if !private_key_path.is_absolute()
            || !public_key_path.is_absolute()
            || !store_root.is_absolute()
            || !valid_text(&operator_name, 128)
        {
            return Err(OperatorCustomError::InvalidConfig);
        }
        validate_private_key_file(&private_key_path)?;
        let private_key = read_catalog_private_key(&private_key_path)
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        let public_key = private_key
            .public_key()
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        let configured_public_key = read_catalog_public_key(&public_key_path)
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        if public_key != configured_public_key {
            return Err(OperatorCustomError::InvalidConfig);
        }
        let key_sha256 = operator_custom_key_sha256(&public_key)
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        SecureCartridgeStore::open_existing(&store_root)
            .map_err(|_| OperatorCustomError::InvalidConfig)?;
        Ok(Self {
            operator_name,
            private_key,
            public_key,
            key_sha256,
            store_root,
        })
    }

    pub fn open_store(&self) -> Result<SecureCartridgeStore, OperatorCustomError> {
        SecureCartridgeStore::open_existing(&self.store_root)
            .map_err(|_| OperatorCustomError::InvalidConfig)
    }

    pub fn public_config(&self) -> OperatorCustomPublicConfig {
        OperatorCustomPublicConfig {
            authority: OperatorCustomAuthority {
                operator_name: self.operator_name.clone(),
                authority_id: self.public_key.authority_id.clone(),
                key_id: self.public_key.key_id.clone(),
                key_sha256: self.key_sha256.clone(),
                public_key: self.public_key.clone(),
            },
            store_root: self.store_root.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CustomImportCommand {
    pub idempotency_key: Uuid,
    pub release_directory: PathBuf,
    pub publisher_public_key_file: PathBuf,
    pub policy_version: u64,
    pub lifecycle_status: CatalogStatus,
    pub actor: String,
    pub reason: String,
    pub acknowledge_marketplace_warning: bool,
}

impl CustomImportCommand {
    pub fn validate(&self) -> Result<(), OperatorCustomError> {
        if self.idempotency_key.is_nil()
            || !self.release_directory.is_absolute()
            || !self.publisher_public_key_file.is_absolute()
            || self.policy_version == 0
            || !valid_text(&self.actor, 64)
            || !valid_text(&self.reason, 500)
            || !self.acknowledge_marketplace_warning
        {
            Err(OperatorCustomError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CustomPolicyCommand {
    pub idempotency_key: Uuid,
    pub game_key: String,
    pub archive_sha256: String,
    pub release_directory: PathBuf,
    pub publisher_public_key_file: PathBuf,
    pub policy_version: u64,
    pub lifecycle_status: CatalogStatus,
    pub actor: String,
    pub reason: String,
}

impl CustomPolicyCommand {
    pub fn validate(&self) -> Result<(), OperatorCustomError> {
        if self.idempotency_key.is_nil()
            || !valid_identifier(&self.game_key)
            || !valid_sha256(&self.archive_sha256)
            || !self.release_directory.is_absolute()
            || !self.publisher_public_key_file.is_absolute()
            || self.policy_version == 0
            || !valid_text(&self.actor, 64)
            || !valid_text(&self.reason, 500)
        {
            Err(OperatorCustomError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CustomOperationReceipt {
    pub format: &'static str,
    pub operation_id: Uuid,
    pub release_id: Uuid,
    pub game_key: String,
    pub archive_sha256: String,
    pub policy_version: u64,
    pub lifecycle_status: String,
    pub imported: bool,
    pub replayed: bool,
}

pub async fn authorize_public_authority(
    pool: &PgPool,
    config: &OperatorCustomPublicConfig,
) -> Result<(), OperatorCustomError> {
    let row = sqlx::query_as::<_, AuthorityRow>(
        r#"
        SELECT operator_name, authority_id, key_id, public_key, key_sha256
        FROM operator_custom_authority
        WHERE singleton
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| OperatorCustomError::Internal)?
    .ok_or(OperatorCustomError::Denied)?;
    if row.operator_name != config.authority.operator_name
        || row.authority_id != config.authority.authority_id
        || row.key_id != config.authority.key_id
        || row.public_key.0 != config.authority.public_key
        || row.key_sha256 != config.authority.key_sha256
    {
        return Err(OperatorCustomError::Denied);
    }
    Ok(())
}

pub async fn import_custom_release(
    pool: &PgPool,
    config: &OperatorCustomAdminConfig,
    command: &CustomImportCommand,
) -> Result<CustomOperationReceipt, OperatorCustomError> {
    command.validate()?;
    let publisher_key = read_public_key(&command.publisher_public_key_file)
        .map_err(|_| OperatorCustomError::InvalidInput)?;
    let release = verify_supported_release_directory(
        &command.release_directory,
        &publisher_key,
        &rich_2d_host_profile(),
    )
    .map_err(|_| OperatorCustomError::Denied)?;
    let server_id = fetch_server_id(pool).await?;
    let signed_operator_release = sign_operator_custom_release(
        &release,
        &publisher_key,
        &config.private_key,
        &server_id.to_string(),
        &config.operator_name,
    )
    .map_err(|_| OperatorCustomError::Denied)?;
    let signed_operator_bytes = signed_operator_custom_release_bytes(&signed_operator_release)
        .map_err(|_| OperatorCustomError::Internal)?;
    let attestation = verify_operator_custom_release_bytes(
        &signed_operator_bytes,
        &config.public_key,
        &server_id.to_string(),
        &release,
    )
    .map_err(|_| OperatorCustomError::Denied)?;
    let signed_policy = sign_catalog_policy(
        &release,
        &config.private_key,
        command.policy_version,
        command.lifecycle_status,
        &command.reason,
    )
    .map_err(|_| OperatorCustomError::InvalidInput)?;
    let policy_bytes =
        serde_json::to_vec(&signed_policy).map_err(|_| OperatorCustomError::Internal)?;
    let store = config.open_store()?;
    let staged = store
        .stage_reviewed_release(&release, &policy_bytes, &config.public_key)
        .map_err(map_cartridge_stage_error)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| OperatorCustomError::Internal)?;
    lock_custom_game(&mut transaction, &attestation.game_key).await?;
    ensure_authority(&mut transaction, server_id, config).await?;
    if let Some(row) = fetch_operation(&mut transaction, command.idempotency_key).await? {
        let receipt = exact_import_replay(row, command, &attestation, staged.installed)?;
        transaction
            .commit()
            .await
            .map_err(|_| OperatorCustomError::Internal)?;
        return Ok(receipt);
    }
    let release_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO operator_custom_releases (
            import_operation_id, game_key, publisher_id, publisher_key,
            rules_version, cartridge_version, archive_sha256,
            signed_identity_sha256, display_name, operator_key,
            operator_key_sha256, operator_name, signed_operator_attestation,
            attestation_version, warning, signed_policy, policy_version,
            policy_status, policy_reason, compatible, imported
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16, $17, $18, $19, $20, $21
        )
        RETURNING id
        "#,
    )
    .bind(command.idempotency_key)
    .bind(&attestation.game_key)
    .bind(&attestation.publisher_id)
    .bind(Json(&publisher_key))
    .bind(i64::from(attestation.rules_version))
    .bind(i64::from(attestation.cartridge_version))
    .bind(&attestation.archive_sha256)
    .bind(&attestation.signed_identity_sha256)
    .bind(release.cartridge().manifest().display_name.as_str())
    .bind(Json(&config.public_key))
    .bind(&config.key_sha256)
    .bind(&config.operator_name)
    .bind(Json(&signed_operator_release))
    .bind(
        i64::try_from(attestation.attestation_version)
            .map_err(|_| OperatorCustomError::Internal)?,
    )
    .bind(OPERATOR_CUSTOM_WARNING)
    .bind(Json(&signed_policy))
    .bind(i64::try_from(command.policy_version).map_err(|_| OperatorCustomError::InvalidInput)?)
    .bind(status_name(command.lifecycle_status))
    .bind(&command.reason)
    .bind(release.cartridge().compatibility().compatible)
    .bind(staged.installed)
    .fetch_one(&mut *transaction)
    .await
    .map_err(map_insert_error)?;
    insert_audit(
        &mut transaction,
        command.idempotency_key,
        release_id,
        "import_custom_cartridge",
        &command.actor,
        &command.reason,
        None,
        None,
        command.policy_version,
        command.lifecycle_status,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(|_| OperatorCustomError::Internal)?;
    Ok(receipt(
        command.idempotency_key,
        release_id,
        &attestation.game_key,
        &attestation.archive_sha256,
        command.policy_version,
        command.lifecycle_status,
        staged.installed,
        false,
    ))
}

pub async fn apply_custom_policy(
    pool: &PgPool,
    config: &OperatorCustomAdminConfig,
    command: &CustomPolicyCommand,
) -> Result<CustomOperationReceipt, OperatorCustomError> {
    command.validate()?;
    let publisher_key = read_public_key(&command.publisher_public_key_file)
        .map_err(|_| OperatorCustomError::InvalidInput)?;
    let release = verify_supported_release_directory(
        &command.release_directory,
        &publisher_key,
        &rich_2d_host_profile(),
    )
    .map_err(|_| OperatorCustomError::Denied)?;
    if release.payload().game_key != command.game_key
        || release.payload().archive_sha256 != command.archive_sha256
    {
        return Err(OperatorCustomError::Conflict);
    }
    let signed_policy = sign_catalog_policy(
        &release,
        &config.private_key,
        command.policy_version,
        command.lifecycle_status,
        &command.reason,
    )
    .map_err(|_| OperatorCustomError::InvalidInput)?;
    let policy_bytes =
        serde_json::to_vec(&signed_policy).map_err(|_| OperatorCustomError::Internal)?;
    let store = config.open_store()?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| OperatorCustomError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(|_| OperatorCustomError::Internal)?;
    lock_custom_game(&mut transaction, &command.game_key).await?;
    let server_id = fetch_server_id_transaction(&mut transaction).await?;
    ensure_authority(&mut transaction, server_id, config).await?;
    let staged = store
        .stage_reviewed_release(&release, &policy_bytes, &config.public_key)
        .map_err(map_cartridge_stage_error)?;
    if let Some(row) = fetch_operation(&mut transaction, command.idempotency_key).await? {
        let receipt = exact_policy_replay(row, command)?;
        transaction
            .commit()
            .await
            .map_err(|_| OperatorCustomError::Internal)?;
        return Ok(receipt);
    }
    let current = sqlx::query_as::<_, CustomReleaseRow>(
        r#"
        SELECT id, import_operation_id, game_key, publisher_id, publisher_key,
               rules_version, cartridge_version, archive_sha256,
               signed_identity_sha256, policy_version, policy_status,
               policy_reason, imported
        FROM operator_custom_releases
        WHERE game_key = $1 AND archive_sha256 = $2
        FOR UPDATE
        "#,
    )
    .bind(&command.game_key)
    .bind(&command.archive_sha256)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| OperatorCustomError::Internal)?
    .ok_or(OperatorCustomError::Denied)?;
    if current.game_key != command.game_key
        || current.archive_sha256 != command.archive_sha256
        || current.publisher_key.0 != publisher_key
        || current.publisher_id != release.payload().publisher_id
        || current.rules_version != i64::from(release.payload().rules_version)
        || current.cartridge_version != i64::from(release.payload().cartridge_version)
        || current.signed_identity_sha256 != release.payload().signed_identity_sha256
        || command.policy_version
            <= u64::try_from(current.policy_version).map_err(|_| OperatorCustomError::Internal)?
    {
        return Err(OperatorCustomError::Conflict);
    }
    let resulting_imported = current.imported || staged.installed;
    sqlx::query(
        r#"
        UPDATE operator_custom_releases
        SET signed_policy = $2,
            policy_version = $3,
            policy_status = $4,
            policy_reason = $5,
            imported = $6,
            updated_at = clock_timestamp()
        WHERE id = $1
        "#,
    )
    .bind(current.id)
    .bind(Json(&signed_policy))
    .bind(i64::try_from(command.policy_version).map_err(|_| OperatorCustomError::InvalidInput)?)
    .bind(status_name(command.lifecycle_status))
    .bind(&command.reason)
    .bind(resulting_imported)
    .execute(&mut *transaction)
    .await
    .map_err(|_| OperatorCustomError::Internal)?;
    insert_audit(
        &mut transaction,
        command.idempotency_key,
        current.id,
        "set_custom_policy",
        &command.actor,
        &command.reason,
        Some(u64::try_from(current.policy_version).map_err(|_| OperatorCustomError::Internal)?),
        Some(&current.policy_status),
        command.policy_version,
        command.lifecycle_status,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(|_| OperatorCustomError::Internal)?;
    Ok(receipt(
        command.idempotency_key,
        current.id,
        &command.game_key,
        &command.archive_sha256,
        command.policy_version,
        command.lifecycle_status,
        resulting_imported,
        false,
    ))
}

async fn ensure_authority(
    transaction: &mut Transaction<'_, Postgres>,
    server_id: Uuid,
    config: &OperatorCustomAdminConfig,
) -> Result<(), OperatorCustomError> {
    sqlx::query(
        r#"
        INSERT INTO operator_custom_authority (
            singleton, server_id, operator_name, authority_id, key_id,
            public_key, key_sha256
        )
        VALUES (TRUE, $1, $2, $3, $4, $5, $6)
        ON CONFLICT (singleton) DO NOTHING
        "#,
    )
    .bind(server_id)
    .bind(&config.operator_name)
    .bind(&config.public_key.authority_id)
    .bind(&config.public_key.key_id)
    .bind(Json(&config.public_key))
    .bind(&config.key_sha256)
    .execute(&mut **transaction)
    .await
    .map_err(|_| OperatorCustomError::Internal)?;
    let row = sqlx::query_as::<_, AuthorityWithServerRow>(
        r#"
        SELECT server_id, operator_name, authority_id, key_id, public_key, key_sha256
        FROM operator_custom_authority
        WHERE singleton
        FOR SHARE
        "#,
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| OperatorCustomError::Internal)?;
    if row.server_id != server_id
        || row.operator_name != config.operator_name
        || row.authority_id != config.public_key.authority_id
        || row.key_id != config.public_key.key_id
        || row.public_key.0 != config.public_key
        || row.key_sha256 != config.key_sha256
    {
        return Err(OperatorCustomError::Denied);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_audit(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
    release_id: Uuid,
    action: &str,
    actor: &str,
    reason: &str,
    previous_version: Option<u64>,
    previous_status: Option<&str>,
    resulting_version: u64,
    resulting_status: CatalogStatus,
) -> Result<(), OperatorCustomError> {
    sqlx::query(
        r#"
        INSERT INTO operator_custom_audit_events (
            operation_id, release_id, action, actor, reason,
            previous_policy_version, previous_policy_status,
            resulting_policy_version, resulting_policy_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(operation_id)
    .bind(release_id)
    .bind(action)
    .bind(actor)
    .bind(reason)
    .bind(previous_version.and_then(|value| i64::try_from(value).ok()))
    .bind(previous_status)
    .bind(i64::try_from(resulting_version).map_err(|_| OperatorCustomError::InvalidInput)?)
    .bind(status_name(resulting_status))
    .execute(&mut **transaction)
    .await
    .map_err(|_| OperatorCustomError::Internal)?;
    Ok(())
}

async fn fetch_operation(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
) -> Result<Option<OperationRow>, OperatorCustomError> {
    sqlx::query_as::<_, OperationRow>(
        r#"
        SELECT audit.operation_id, audit.action, audit.actor, audit.reason,
               audit.previous_policy_version, audit.previous_policy_status,
               audit.resulting_policy_version, audit.resulting_policy_status,
               release.id AS release_id, release.game_key,
               release.archive_sha256, release.imported
        FROM operator_custom_audit_events AS audit
        JOIN operator_custom_releases AS release ON release.id = audit.release_id
        WHERE audit.operation_id = $1
        "#,
    )
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| OperatorCustomError::Internal)
}

fn exact_import_replay(
    row: OperationRow,
    command: &CustomImportCommand,
    attestation: &omarchygs_game_cartridge::OperatorCustomReleasePayload,
    imported: bool,
) -> Result<CustomOperationReceipt, OperatorCustomError> {
    if row.action != "import_custom_cartridge"
        || row.actor != command.actor
        || row.reason != command.reason
        || row.game_key != attestation.game_key
        || row.archive_sha256 != attestation.archive_sha256
        || row.resulting_policy_version
            != i64::try_from(command.policy_version)
                .map_err(|_| OperatorCustomError::InvalidInput)?
        || row.resulting_policy_status != status_name(command.lifecycle_status)
        || row.imported != imported
    {
        return Err(OperatorCustomError::Conflict);
    }
    Ok(receipt(
        row.operation_id,
        row.release_id,
        &row.game_key,
        &row.archive_sha256,
        command.policy_version,
        command.lifecycle_status,
        row.imported,
        true,
    ))
}

fn exact_policy_replay(
    row: OperationRow,
    command: &CustomPolicyCommand,
) -> Result<CustomOperationReceipt, OperatorCustomError> {
    if row.action != "set_custom_policy"
        || row.actor != command.actor
        || row.reason != command.reason
        || row.game_key != command.game_key
        || row.archive_sha256 != command.archive_sha256
        || row.resulting_policy_version
            != i64::try_from(command.policy_version)
                .map_err(|_| OperatorCustomError::InvalidInput)?
        || row.resulting_policy_status != status_name(command.lifecycle_status)
    {
        return Err(OperatorCustomError::Conflict);
    }
    Ok(receipt(
        row.operation_id,
        row.release_id,
        &row.game_key,
        &row.archive_sha256,
        command.policy_version,
        command.lifecycle_status,
        row.imported,
        true,
    ))
}

async fn lock_custom_game(
    transaction: &mut Transaction<'_, Postgres>,
    game_key: &str,
) -> Result<(), OperatorCustomError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, $2))")
        .bind(game_key)
        .bind(SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut **transaction)
        .await
        .map_err(|_| OperatorCustomError::Internal)?;
    Ok(())
}

async fn fetch_server_id(pool: &PgPool) -> Result<Uuid, OperatorCustomError> {
    sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton")
        .fetch_one(pool)
        .await
        .map_err(|_| OperatorCustomError::Internal)
}

async fn fetch_server_id_transaction(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, OperatorCustomError> {
    sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton FOR SHARE")
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| OperatorCustomError::Internal)
}

#[allow(clippy::too_many_arguments)]
fn receipt(
    operation_id: Uuid,
    release_id: Uuid,
    game_key: &str,
    archive_sha256: &str,
    policy_version: u64,
    lifecycle_status: CatalogStatus,
    imported: bool,
    replayed: bool,
) -> CustomOperationReceipt {
    CustomOperationReceipt {
        format: "omarchygs.operator-custom-operation-receipt/v1",
        operation_id,
        release_id,
        game_key: game_key.to_owned(),
        archive_sha256: archive_sha256.to_owned(),
        policy_version,
        lifecycle_status: status_name(lifecycle_status).to_owned(),
        imported,
        replayed,
    }
}

fn validate_private_key_file(path: &Path) -> Result<(), OperatorCustomError> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| OperatorCustomError::InvalidConfig)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.mode() & 0o777 != 0o600
        || metadata.uid() != geteuid().as_raw()
    {
        return Err(OperatorCustomError::InvalidConfig);
    }
    Ok(())
}

fn map_cartridge_stage_error(
    error: omarchygs_game_cartridge::CartridgeError,
) -> OperatorCustomError {
    match error {
        omarchygs_game_cartridge::CartridgeError::Io(_) => OperatorCustomError::Internal,
        _ => OperatorCustomError::Denied,
    }
}

fn map_insert_error(error: sqlx::Error) -> OperatorCustomError {
    if error
        .as_database_error()
        .is_some_and(|error| matches!(error.code().as_deref(), Some("23505") | Some("23514")))
    {
        OperatorCustomError::Conflict
    } else {
        OperatorCustomError::Internal
    }
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

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

#[derive(FromRow)]
struct AuthorityRow {
    operator_name: String,
    authority_id: String,
    key_id: String,
    public_key: Json<CatalogPublicKey>,
    key_sha256: String,
}

#[derive(FromRow)]
struct AuthorityWithServerRow {
    server_id: Uuid,
    operator_name: String,
    authority_id: String,
    key_id: String,
    public_key: Json<CatalogPublicKey>,
    key_sha256: String,
}

#[derive(FromRow)]
struct CustomReleaseRow {
    id: Uuid,
    #[allow(dead_code)]
    import_operation_id: Uuid,
    game_key: String,
    publisher_id: String,
    publisher_key: Json<PublisherPublicKey>,
    rules_version: i64,
    cartridge_version: i64,
    archive_sha256: String,
    signed_identity_sha256: String,
    policy_version: i64,
    policy_status: String,
    #[allow(dead_code)]
    policy_reason: String,
    #[allow(dead_code)]
    imported: bool,
}

#[derive(FromRow)]
struct OperationRow {
    operation_id: Uuid,
    action: String,
    actor: String,
    reason: String,
    #[allow(dead_code)]
    previous_policy_version: Option<i64>,
    #[allow(dead_code)]
    previous_policy_status: Option<String>,
    resulting_policy_version: i64,
    resulting_policy_status: String,
    release_id: Uuid,
    game_key: String,
    archive_sha256: String,
    imported: bool,
}
