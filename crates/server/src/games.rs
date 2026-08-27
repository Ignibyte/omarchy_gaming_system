use std::collections::{HashMap, HashSet};

use omarchy_game_runtime::{
    ApplyGameCommandError, GameRegistry, GameSessionStatus, InitializeGameError,
};
use omarchy_gaming_system_server::{
    cartridge_distribution::CartridgeDistributionRuntime,
    session_cartridges::{self, SessionCartridgePresentation},
};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction, types::Json};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    personas::Persona,
    sessions::{self, SessionError},
    sync::{self, SyncEventKind},
};

pub const DEFAULT_GAME_SESSION_LIMIT: u16 = 50;
pub const MAX_GAME_SESSION_LIMIT: u16 = 100;
pub const MAX_ACTIVE_SOLO_SESSIONS_PER_PERSONA: i64 = 25;
#[cfg_attr(not(test), allow(dead_code))]
const MAX_SESSION_PARTICIPANTS: usize = 8;

#[derive(Debug, PartialEq)]
pub struct GameSession {
    pub id: Uuid,
    pub game_key: String,
    pub game_version: u32,
    pub revision: i64,
    pub status: String,
    pub state: Option<Value>,
    pub authority: String,
    pub provider_release_id: Option<Uuid>,
    pub availability: Option<String>,
    pub presentation: Option<SessionCartridgePresentation>,
    pub result: Option<GameResult>,
    pub participants: Vec<GameParticipant>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq)]
pub struct GameResult {
    pub outcome: String,
    pub public_summary: Value,
    pub provider_revision: i64,
    pub projected_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameParticipant {
    pub seat: u8,
    pub persona: Persona,
}

#[derive(Debug, PartialEq)]
pub struct GameCommandResult {
    pub game_session_id: Uuid,
    pub revision: i64,
    pub status: String,
    pub state: Value,
    pub authority: String,
    pub provider_release_id: Option<Uuid>,
    pub availability: Option<String>,
}

pub struct StartGameSessionInput {
    pub idempotency_key: String,
    pub game_key: String,
    pub game_version: u32,
}

#[derive(Debug, PartialEq)]
pub enum GameSessionStartOutcome {
    Created(GameSession),
    Existing(GameSession),
    Pending(GameSession),
}

pub struct GameCommandInput {
    pub idempotency_key: String,
    pub expected_revision: i64,
    pub command: Value,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum GameError {
    Unauthorized,
    PersonaNotFound,
    GameSessionNotFound,
    InvalidPagination,
    InvalidStart,
    GameUnavailable,
    CartridgeUnavailable,
    InvalidParticipants,
    ActiveSessionLimit,
    InitializationFailed,
    InvalidCommand,
    CommandRejected,
    GameCompleted,
    RevisionConflict,
    IdempotencyConflict,
    ProviderUnavailable,
    Internal,
}

#[derive(sqlx::FromRow)]
struct GameSessionRow {
    id: Uuid,
    game_key: String,
    game_version: i64,
    revision: i64,
    status: String,
    state: Option<Json<Value>>,
    authority: String,
    provider_release_id: Option<Uuid>,
    provider_availability: Option<String>,
    provider_view: Option<Json<Value>>,
    presentation_publisher_id: Option<String>,
    presentation_game_key: Option<String>,
    presentation_rules_version: Option<i64>,
    presentation_cartridge_version: Option<i64>,
    presentation_archive_sha256: Option<String>,
    presentation_signed_identity_sha256: Option<String>,
    presentation_admission_revision: Option<i64>,
    presentation_lifecycle_status: Option<String>,
    presentation_lifecycle_reason: Option<String>,
    presentation_provenance_class: Option<String>,
    presentation_operator_name: Option<String>,
    presentation_operator_authority_id: Option<String>,
    presentation_operator_key_id: Option<String>,
    presentation_operator_key_sha256: Option<String>,
    presentation_operator_warning: Option<String>,
    result_outcome: Option<String>,
    result_summary: Option<Json<Value>>,
    result_revision: Option<i64>,
    result_projected_at: Option<String>,
    completed_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct ParticipantRow {
    game_session_id: Uuid,
    seat: i16,
    persona_id: Uuid,
    handle: String,
    display_name: String,
    bio: String,
    status_message: String,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct LockedGameSessionRow {
    game_key: String,
    game_version: i64,
    revision: i64,
    status: String,
    state: Option<Json<Value>>,
    authority: String,
    actor_seat: i16,
}

#[derive(sqlx::FromRow)]
struct GameCommandReceiptRow {
    actor_persona_id: Uuid,
    expected_revision: i64,
    applied_revision: i64,
    state: Json<Value>,
    session_status: String,
    command_matches: bool,
}

#[derive(sqlx::FromRow)]
struct GameSessionStartReceiptRow {
    game_session_id: Uuid,
    game_key: String,
    game_version: i64,
}

/// Create a version-pinned session inside a trusted caller's transaction.
///
/// The caller owns authorization and commit. Any returned error must be
/// propagated so the transaction is rolled back.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) async fn create_session(
    transaction: &mut Transaction<'_, Postgres>,
    registry: &GameRegistry,
    game_key: &str,
    game_version: u32,
    participant_ids: &[Uuid],
) -> Result<Uuid, GameError> {
    create_session_with_distribution(
        transaction,
        registry,
        None,
        game_key,
        game_version,
        participant_ids,
    )
    .await
}

pub(crate) async fn create_session_with_distribution(
    transaction: &mut Transaction<'_, Postgres>,
    registry: &GameRegistry,
    cartridge_distribution: Option<&CartridgeDistributionRuntime>,
    game_key: &str,
    game_version: u32,
    participant_ids: &[Uuid],
) -> Result<Uuid, GameError> {
    if participant_ids.is_empty() || participant_ids.len() > MAX_SESSION_PARTICIPANTS {
        return Err(GameError::InvalidParticipants);
    }
    let unique_participants = participant_ids.iter().copied().collect::<HashSet<_>>();
    if unique_participants.len() != participant_ids.len() {
        return Err(GameError::InvalidParticipants);
    }
    let player_count =
        u8::try_from(participant_ids.len()).map_err(|_| GameError::InvalidParticipants)?;
    let initialized = registry
        .initialize(game_key, game_version, player_count)
        .map_err(map_initialization_error)?;

    let mut locked_ids = participant_ids.to_vec();
    locked_ids.sort_unstable();
    let found_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM personas WHERE id = ANY($1) ORDER BY id FOR UPDATE",
    )
    .bind(&locked_ids)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "game participant locking"))?;
    if found_ids != locked_ids {
        return Err(GameError::InvalidParticipants);
    }

    let session_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO game_sessions (game_key, game_version, state, authority)
        VALUES ($1, $2, $3, 'platform_compiled')
        RETURNING id
        "#,
    )
    .bind(&initialized.manifest.key)
    .bind(i64::from(initialized.manifest.version))
    .bind(Json(&initialized.state))
    .fetch_one(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "game session insertion"))?;

    for (seat, persona_id) in participant_ids.iter().enumerate() {
        let seat = i16::try_from(seat).map_err(|_| GameError::InvalidParticipants)?;
        sqlx::query(
            r#"
            INSERT INTO game_session_participants (game_session_id, persona_id, seat)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(session_id)
        .bind(persona_id)
        .bind(seat)
        .execute(&mut **transaction)
        .await
        .map_err(|database_error| {
            map_database_error(database_error, "game participant insertion")
        })?;
    }

    if let Some(runtime) = cartridge_distribution {
        session_cartridges::pin_new_session(
            transaction,
            runtime,
            session_id,
            &initialized.manifest.key,
            initialized.manifest.version,
            None,
        )
        .await
        .map_err(|_| GameError::CartridgeUnavailable)?;
    }

    for persona_id in locked_ids {
        sync::append_event(
            transaction,
            persona_id,
            SyncEventKind::GameSession(session_id),
        )
        .await
        .map_err(|database_error| {
            error!(?database_error, %persona_id, %session_id, "game sync event append failed");
            GameError::Internal
        })?;
    }

    info!(%session_id, game_key = %initialized.manifest.key, game_version = initialized.manifest.version, "game session initialized");
    Ok(session_id)
}

pub async fn start_solo_session(
    pool: &PgPool,
    registry: &GameRegistry,
    cartridge_distribution: Option<&CartridgeDistributionRuntime>,
    token: &str,
    actor_id: &str,
    input: StartGameSessionInput,
) -> Result<GameSessionStartOutcome, GameError> {
    let StartGameSessionInput {
        idempotency_key,
        game_key,
        game_version,
    } = input;
    let actor_id = authenticate_owned_persona(pool, token, actor_id).await?;
    let idempotency_key = Uuid::try_parse(&idempotency_key).map_err(|_| GameError::InvalidStart)?;

    let mut transaction = pool.begin().await.map_err(|database_error| {
        map_database_error(database_error, "solo game start transaction")
    })?;
    let actor_exists =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM personas WHERE id = $1 FOR UPDATE")
            .bind(actor_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|database_error| {
                map_database_error(database_error, "solo game persona lock")
            })?;
    if actor_exists.is_none() {
        return Err(GameError::PersonaNotFound);
    }

    let existing = sqlx::query_as::<_, GameSessionStartReceiptRow>(
        r#"
        SELECT game_session_id, game_key, game_version
        FROM game_session_starts
        WHERE persona_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(actor_id)
    .bind(idempotency_key)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "solo game replay lookup"))?;
    if let Some(existing) = existing {
        if existing.game_key != game_key || existing.game_version != i64::from(game_version) {
            return Err(GameError::IdempotencyConflict);
        }
        transaction.commit().await.map_err(|database_error| {
            map_database_error(database_error, "solo game replay commit")
        })?;
        let session =
            load_session_for_participant(pool, actor_id, existing.game_session_id).await?;
        return Ok(GameSessionStartOutcome::Existing(session));
    }

    let manifest = registry
        .manifest(&game_key, game_version)
        .ok_or(GameError::GameUnavailable)?;
    if manifest.min_human_players != 1 || manifest.max_human_players != 1 {
        return Err(GameError::InvalidParticipants);
    }

    let active_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM game_session_starts AS start
        JOIN game_sessions AS session ON session.id = start.game_session_id
        WHERE start.persona_id = $1 AND session.status = 'active'
        "#,
    )
    .bind(actor_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "solo game active count"))?;
    if active_count >= MAX_ACTIVE_SOLO_SESSIONS_PER_PERSONA {
        return Err(GameError::ActiveSessionLimit);
    }

    let game_session_id = create_session_with_distribution(
        &mut transaction,
        registry,
        cartridge_distribution,
        &manifest.key,
        manifest.version,
        &[actor_id],
    )
    .await?;
    sqlx::query(
        r#"
        INSERT INTO game_session_starts (
            persona_id,
            idempotency_key,
            game_session_id,
            game_key,
            game_version
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(actor_id)
    .bind(idempotency_key)
    .bind(game_session_id)
    .bind(&manifest.key)
    .bind(i64::from(manifest.version))
    .execute(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "solo game receipt insert"))?;
    transaction
        .commit()
        .await
        .map_err(|database_error| map_database_error(database_error, "solo game start commit"))?;

    let session = load_session_for_participant(pool, actor_id, game_session_id).await?;
    info!(%game_session_id, %actor_id, game_key = %manifest.key, game_version = manifest.version, "solo game session started");
    Ok(GameSessionStartOutcome::Created(session))
}

pub async fn list_sessions(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    limit: Option<u16>,
) -> Result<Vec<GameSession>, GameError> {
    let actor_id = authenticate_owned_persona(pool, token, actor_id).await?;
    let limit = validate_limit(limit)?;
    let rows = sqlx::query_as::<_, GameSessionRow>(
        r#"
        SELECT
            session.id,
            session.game_key,
            session.game_version,
            session.revision,
            session.status,
            session.state,
            session.authority,
            session.provider_release_id,
            session.provider_availability,
            view.view AS provider_view,
            COALESCE(release.publisher_id, custom.publisher_id) AS presentation_publisher_id,
            COALESCE(release.game_key, custom.game_key) AS presentation_game_key,
            COALESCE(release.rules_version, custom.rules_version) AS presentation_rules_version,
            COALESCE(release.cartridge_version, custom.cartridge_version)
                AS presentation_cartridge_version,
            COALESCE(release.archive_sha256, custom.archive_sha256)
                AS presentation_archive_sha256,
            COALESCE(release.signed_identity_sha256, custom.signed_identity_sha256)
                AS presentation_signed_identity_sha256,
            presentation.admission_revision AS presentation_admission_revision,
            COALESCE(release.policy_status, custom.policy_status)
                AS presentation_lifecycle_status,
            COALESCE(release.policy_reason, custom.policy_reason)
                AS presentation_lifecycle_reason,
            presentation.provenance_class AS presentation_provenance_class,
            custom.operator_name AS presentation_operator_name,
            custom.operator_key ->> 'authority_id' AS presentation_operator_authority_id,
            custom.operator_key ->> 'key_id' AS presentation_operator_key_id,
            custom.operator_key_sha256 AS presentation_operator_key_sha256,
            custom.warning AS presentation_operator_warning,
            result.outcome AS result_outcome,
            result.public_summary AS result_summary,
            result.provider_revision AS result_revision,
            CASE WHEN result.projected_at IS NULL THEN NULL ELSE
              to_char(result.projected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            END AS result_projected_at,
            CASE WHEN session.completed_at IS NULL THEN NULL ELSE
              to_char(session.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            END AS completed_at,
            to_char(session.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(session.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
        FROM game_sessions AS session
        JOIN game_session_participants AS actor
          ON actor.game_session_id = session.id AND actor.persona_id = $1
        LEFT JOIN provider_game_session_views AS view ON view.game_session_id = session.id
        LEFT JOIN provider_game_results AS result ON result.game_session_id = session.id
        LEFT JOIN game_session_cartridge_presentations AS presentation
          ON presentation.game_session_id = session.id
        LEFT JOIN marketplace_releases AS release
          ON release.id = presentation.marketplace_release_id
        LEFT JOIN operator_custom_releases AS custom
          ON custom.id = presentation.operator_custom_release_id
        ORDER BY session.created_at DESC, session.id DESC
        LIMIT $2
        "#,
    )
    .bind(actor_id)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "game session listing"))?;
    load_session_participants(pool, rows).await
}

pub async fn get_session(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    session_id: &str,
) -> Result<GameSession, GameError> {
    let actor_id = authenticate_owned_persona(pool, token, actor_id).await?;
    let session_id = Uuid::try_parse(session_id).map_err(|_| GameError::GameSessionNotFound)?;
    load_session_for_participant(pool, actor_id, session_id).await
}

pub(crate) async fn load_session_for_participant(
    pool: &PgPool,
    actor_id: Uuid,
    session_id: Uuid,
) -> Result<GameSession, GameError> {
    let row = sqlx::query_as::<_, GameSessionRow>(
        r#"
        SELECT
            session.id,
            session.game_key,
            session.game_version,
            session.revision,
            session.status,
            session.state,
            session.authority,
            session.provider_release_id,
            session.provider_availability,
            view.view AS provider_view,
            COALESCE(release.publisher_id, custom.publisher_id) AS presentation_publisher_id,
            COALESCE(release.game_key, custom.game_key) AS presentation_game_key,
            COALESCE(release.rules_version, custom.rules_version) AS presentation_rules_version,
            COALESCE(release.cartridge_version, custom.cartridge_version)
                AS presentation_cartridge_version,
            COALESCE(release.archive_sha256, custom.archive_sha256)
                AS presentation_archive_sha256,
            COALESCE(release.signed_identity_sha256, custom.signed_identity_sha256)
                AS presentation_signed_identity_sha256,
            presentation.admission_revision AS presentation_admission_revision,
            COALESCE(release.policy_status, custom.policy_status)
                AS presentation_lifecycle_status,
            COALESCE(release.policy_reason, custom.policy_reason)
                AS presentation_lifecycle_reason,
            presentation.provenance_class AS presentation_provenance_class,
            custom.operator_name AS presentation_operator_name,
            custom.operator_key ->> 'authority_id' AS presentation_operator_authority_id,
            custom.operator_key ->> 'key_id' AS presentation_operator_key_id,
            custom.operator_key_sha256 AS presentation_operator_key_sha256,
            custom.warning AS presentation_operator_warning,
            result.outcome AS result_outcome,
            result.public_summary AS result_summary,
            result.provider_revision AS result_revision,
            CASE WHEN result.projected_at IS NULL THEN NULL ELSE
              to_char(result.projected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            END AS result_projected_at,
            CASE WHEN session.completed_at IS NULL THEN NULL ELSE
              to_char(session.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            END AS completed_at,
            to_char(session.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(session.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
        FROM game_sessions AS session
        JOIN game_session_participants AS actor
          ON actor.game_session_id = session.id AND actor.persona_id = $1
        LEFT JOIN provider_game_session_views AS view ON view.game_session_id = session.id
        LEFT JOIN provider_game_results AS result ON result.game_session_id = session.id
        LEFT JOIN game_session_cartridge_presentations AS presentation
          ON presentation.game_session_id = session.id
        LEFT JOIN marketplace_releases AS release
          ON release.id = presentation.marketplace_release_id
        LEFT JOIN operator_custom_releases AS custom
          ON custom.id = presentation.operator_custom_release_id
        WHERE session.id = $2
        "#,
    )
    .bind(actor_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "game session lookup"))?
    .ok_or(GameError::GameSessionNotFound)?;
    load_session_participants(pool, vec![row])
        .await?
        .pop()
        .ok_or(GameError::Internal)
}

pub async fn apply_command(
    pool: &PgPool,
    registry: &GameRegistry,
    token: &str,
    actor_id: &str,
    session_id: &str,
    input: GameCommandInput,
) -> Result<GameCommandResult, GameError> {
    let GameCommandInput {
        idempotency_key,
        expected_revision,
        command,
    } = input;
    let actor_id = authenticate_owned_persona(pool, token, actor_id).await?;
    let session_id = Uuid::try_parse(session_id).map_err(|_| GameError::GameSessionNotFound)?;
    let idempotency_key =
        Uuid::try_parse(&idempotency_key).map_err(|_| GameError::InvalidCommand)?;
    if expected_revision < 0 {
        return Err(GameError::InvalidCommand);
    }

    let mut transaction = pool.begin().await.map_err(|database_error| {
        map_database_error(database_error, "game command transaction start")
    })?;
    let locked = sqlx::query_as::<_, LockedGameSessionRow>(
        r#"
        SELECT
            session.game_key,
            session.game_version,
            session.revision,
            session.status,
            session.state,
            session.authority,
            actor.seat AS actor_seat
        FROM game_sessions AS session
        JOIN game_session_participants AS actor
          ON actor.game_session_id = session.id AND actor.persona_id = $1
        WHERE session.id = $2
        FOR UPDATE OF session
        "#,
    )
    .bind(actor_id)
    .bind(session_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "game session command lock"))?
    .ok_or(GameError::GameSessionNotFound)?;

    let receipt = sqlx::query_as::<_, GameCommandReceiptRow>(
        r#"
        SELECT
            actor_persona_id,
            expected_revision,
            applied_revision,
            state,
            session_status,
            command = $3 AS command_matches
        FROM game_session_commands
        WHERE game_session_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(session_id)
    .bind(idempotency_key)
    .bind(Json(&command))
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "game command receipt lookup"))?;

    if let Some(receipt) = receipt {
        if receipt.actor_persona_id != actor_id
            || receipt.expected_revision != expected_revision
            || !receipt.command_matches
        {
            return Err(GameError::IdempotencyConflict);
        }
        transaction.commit().await.map_err(|database_error| {
            map_database_error(database_error, "game command replay commit")
        })?;
        return Ok(GameCommandResult {
            game_session_id: session_id,
            revision: receipt.applied_revision,
            status: receipt.session_status,
            state: receipt.state.0,
            authority: "platform_compiled".to_owned(),
            provider_release_id: None,
            availability: None,
        });
    }

    if locked.status == GameSessionStatus::Completed.as_str() {
        return Err(GameError::GameCompleted);
    }
    if locked.status != GameSessionStatus::Active.as_str() {
        return Err(GameError::Internal);
    }
    if locked.authority != "platform_compiled" {
        return Err(GameError::GameUnavailable);
    }
    if locked.revision != expected_revision {
        return Err(GameError::RevisionConflict);
    }
    let game_version = u32::try_from(locked.game_version).map_err(|_| GameError::Internal)?;
    let actor_seat = u8::try_from(locked.actor_seat).map_err(|_| GameError::Internal)?;
    let transition = registry
        .apply_command(
            &locked.game_key,
            game_version,
            &locked.state.as_ref().ok_or(GameError::Internal)?.0,
            actor_seat,
            &command,
        )
        .map_err(map_command_error)?;
    let applied_revision = locked.revision.checked_add(1).ok_or(GameError::Internal)?;

    let stored_revision = sqlx::query_scalar::<_, i64>(
        r#"
        WITH mutation AS (SELECT clock_timestamp() AS at)
        UPDATE game_sessions
        SET state = $1,
            status = $2,
            revision = revision + 1,
            completed_at = CASE WHEN $2 = 'completed' THEN mutation.at ELSE NULL END,
            updated_at = GREATEST(updated_at, mutation.at)
        FROM mutation
        WHERE id = $3 AND revision = $4 AND status = 'active'
        RETURNING revision
        "#,
    )
    .bind(Json(&transition.state))
    .bind(transition.status.as_str())
    .bind(session_id)
    .bind(locked.revision)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "game session command update"))?
    .ok_or(GameError::Internal)?;
    if stored_revision != applied_revision {
        return Err(GameError::Internal);
    }

    sqlx::query(
        r#"
        INSERT INTO game_session_commands (
            game_session_id,
            idempotency_key,
            actor_persona_id,
            expected_revision,
            applied_revision,
            command,
            state,
            session_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(session_id)
    .bind(idempotency_key)
    .bind(actor_id)
    .bind(expected_revision)
    .bind(applied_revision)
    .bind(Json(&command))
    .bind(Json(&transition.state))
    .bind(transition.status.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "game command receipt insert"))?;

    let participant_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT persona_id
        FROM game_session_participants
        WHERE game_session_id = $1
        ORDER BY persona_id
        "#,
    )
    .bind(session_id)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "game command participants"))?;
    if participant_ids.is_empty() {
        return Err(GameError::Internal);
    }
    for persona_id in participant_ids {
        sync::append_event(
            &mut transaction,
            persona_id,
            SyncEventKind::GameSession(session_id),
        )
        .await
        .map_err(|database_error| {
            error!(?database_error, %persona_id, %session_id, "game command sync append failed");
            GameError::Internal
        })?;
    }

    transaction.commit().await.map_err(|database_error| {
        map_database_error(database_error, "game command transaction commit")
    })?;
    info!(%session_id, %actor_id, revision = applied_revision, status = transition.status.as_str(), "game command committed");
    Ok(GameCommandResult {
        game_session_id: session_id,
        revision: applied_revision,
        status: transition.status.as_str().to_owned(),
        state: transition.state,
        authority: "platform_compiled".to_owned(),
        provider_release_id: None,
        availability: None,
    })
}

async fn load_session_participants(
    pool: &PgPool,
    rows: Vec<GameSessionRow>,
) -> Result<Vec<GameSession>, GameError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let session_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let participant_rows = sqlx::query_as::<_, ParticipantRow>(
        r#"
        SELECT
            participant.game_session_id,
            participant.seat,
            persona.id AS persona_id,
            persona.handle,
            persona.display_name,
            persona.bio,
            persona.status_message,
            to_char(persona.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(persona.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
        FROM game_session_participants AS participant
        JOIN personas AS persona ON persona.id = participant.persona_id
        WHERE participant.game_session_id = ANY($1)
        ORDER BY participant.game_session_id, participant.seat
        "#,
    )
    .bind(&session_ids)
    .fetch_all(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "game participants loading"))?;

    let mut participants_by_session: HashMap<Uuid, Vec<GameParticipant>> = HashMap::new();
    for row in participant_rows {
        let seat = u8::try_from(row.seat).map_err(|_| GameError::Internal)?;
        participants_by_session
            .entry(row.game_session_id)
            .or_default()
            .push(GameParticipant {
                seat,
                persona: Persona {
                    id: row.persona_id,
                    handle: row.handle,
                    display_name: row.display_name,
                    bio: row.bio,
                    status_message: row.status_message,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
            });
    }

    rows.into_iter()
        .map(|row| {
            let game_version = u32::try_from(row.game_version).map_err(|_| GameError::Internal)?;
            let participants = participants_by_session
                .remove(&row.id)
                .ok_or(GameError::Internal)?;
            Ok(GameSession {
                id: row.id,
                game_key: row.game_key,
                game_version,
                revision: row.revision,
                status: row.status,
                state: match row.authority.as_str() {
                    "platform_compiled" => Some(row.state.ok_or(GameError::Internal)?.0),
                    "registered_provider" => row.provider_view.map(|view| view.0),
                    _ => return Err(GameError::Internal),
                },
                authority: row.authority,
                provider_release_id: row.provider_release_id,
                availability: row.provider_availability,
                presentation: match (
                    row.presentation_publisher_id,
                    row.presentation_game_key,
                    row.presentation_rules_version,
                    row.presentation_cartridge_version,
                    row.presentation_archive_sha256,
                    row.presentation_signed_identity_sha256,
                    row.presentation_admission_revision,
                    row.presentation_lifecycle_status,
                    row.presentation_lifecycle_reason,
                    row.presentation_provenance_class,
                    row.presentation_operator_name,
                    row.presentation_operator_authority_id,
                    row.presentation_operator_key_id,
                    row.presentation_operator_key_sha256,
                    row.presentation_operator_warning,
                ) {
                    (
                        Some(publisher_id),
                        Some(game_key),
                        Some(rules_version),
                        Some(cartridge_version),
                        Some(archive_sha256),
                        Some(signed_identity_sha256),
                        Some(admission_revision),
                        Some(lifecycle_status),
                        Some(lifecycle_reason),
                        Some(provenance_class),
                        operator_name,
                        operator_authority_id,
                        operator_key_id,
                        operator_key_sha256,
                        operator_warning,
                    ) => Some(
                        session_cartridges::project_presentation(
                            publisher_id,
                            game_key,
                            rules_version,
                            cartridge_version,
                            archive_sha256,
                            signed_identity_sha256,
                            admission_revision,
                            lifecycle_status,
                            lifecycle_reason,
                            provenance_class,
                            operator_name,
                            operator_authority_id,
                            operator_key_id,
                            operator_key_sha256,
                            operator_warning,
                        )
                        .map_err(|_| GameError::Internal)?,
                    ),
                    (
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ) => None,
                    _ => return Err(GameError::Internal),
                },
                result: match (
                    row.result_outcome,
                    row.result_summary,
                    row.result_revision,
                    row.result_projected_at,
                ) {
                    (Some(outcome), Some(summary), Some(provider_revision), Some(projected_at)) => {
                        Some(GameResult {
                            outcome,
                            public_summary: summary.0,
                            provider_revision,
                            projected_at,
                        })
                    }
                    (None, None, None, None) => None,
                    _ => return Err(GameError::Internal),
                },
                participants,
                completed_at: row.completed_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .collect()
}

pub(crate) async fn authenticate_owned_persona(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
) -> Result<Uuid, GameError> {
    let authenticated = sessions::authenticate(pool, token)
        .await
        .map_err(|session_error| match session_error {
            SessionError::Unauthorized => GameError::Unauthorized,
            _ => GameError::Internal,
        })?;
    let actor_id = Uuid::try_parse(actor_id).map_err(|_| GameError::PersonaNotFound)?;
    let owned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM personas WHERE id = $1 AND account_id = $2)",
    )
    .bind(actor_id)
    .bind(authenticated.account_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "game persona ownership"))?;
    if owned {
        Ok(actor_id)
    } else {
        Err(GameError::PersonaNotFound)
    }
}

fn validate_limit(limit: Option<u16>) -> Result<u16, GameError> {
    let limit = limit.unwrap_or(DEFAULT_GAME_SESSION_LIMIT);
    if (1..=MAX_GAME_SESSION_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(GameError::InvalidPagination)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn map_initialization_error(error: InitializeGameError) -> GameError {
    match error {
        InitializeGameError::GameUnavailable => GameError::GameUnavailable,
        InitializeGameError::InvalidPlayerCount => GameError::InvalidParticipants,
        InitializeGameError::InitializationFailed | InitializeGameError::InvalidInitialState => {
            GameError::InitializationFailed
        }
    }
}

fn map_command_error(error: ApplyGameCommandError) -> GameError {
    match error {
        ApplyGameCommandError::GameUnavailable => GameError::GameUnavailable,
        ApplyGameCommandError::InvalidCommand => GameError::InvalidCommand,
        ApplyGameCommandError::CommandRejected => GameError::CommandRejected,
        ApplyGameCommandError::InvalidState
        | ApplyGameCommandError::InvalidActorSeat
        | ApplyGameCommandError::InvalidTransition => GameError::Internal,
    }
}

fn map_database_error(database_error: sqlx::Error, operation: &'static str) -> GameError {
    error!(?database_error, operation, "game database operation failed");
    GameError::Internal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_is_positive_and_bounded() {
        assert_eq!(validate_limit(None), Ok(DEFAULT_GAME_SESSION_LIMIT));
        assert_eq!(validate_limit(Some(1)), Ok(1));
        assert_eq!(validate_limit(Some(MAX_GAME_SESSION_LIMIT)), Ok(100));
        assert_eq!(validate_limit(Some(0)), Err(GameError::InvalidPagination));
        assert_eq!(
            validate_limit(Some(MAX_GAME_SESSION_LIMIT + 1)),
            Err(GameError::InvalidPagination)
        );
    }
}
