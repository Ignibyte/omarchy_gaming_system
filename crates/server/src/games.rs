use std::collections::{HashMap, HashSet};

use omarchy_game_runtime::{ApplyGameCommandError, GameRegistry, InitializeGameError};
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
#[cfg_attr(not(test), allow(dead_code))]
const MAX_SESSION_PARTICIPANTS: usize = 8;

#[derive(Debug, PartialEq)]
pub struct GameSession {
    pub id: Uuid,
    pub game_key: String,
    pub game_version: u32,
    pub revision: i64,
    pub status: String,
    pub state: Value,
    pub participants: Vec<GameParticipant>,
    pub created_at: String,
    pub updated_at: String,
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
    pub state: Value,
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
    GameUnavailable,
    InvalidParticipants,
    InitializationFailed,
    InvalidCommand,
    CommandRejected,
    RevisionConflict,
    IdempotencyConflict,
    Internal,
}

#[derive(sqlx::FromRow)]
struct GameSessionRow {
    id: Uuid,
    game_key: String,
    game_version: i64,
    revision: i64,
    status: String,
    state: Json<Value>,
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
    state: Json<Value>,
    actor_seat: i16,
}

#[derive(sqlx::FromRow)]
struct GameCommandReceiptRow {
    actor_persona_id: Uuid,
    expected_revision: i64,
    applied_revision: i64,
    state: Json<Value>,
    command_matches: bool,
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
        INSERT INTO game_sessions (game_key, game_version, state)
        VALUES ($1, $2, $3)
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
            to_char(session.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(session.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
        FROM game_sessions AS session
        JOIN game_session_participants AS actor
          ON actor.game_session_id = session.id AND actor.persona_id = $1
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
    let row = sqlx::query_as::<_, GameSessionRow>(
        r#"
        SELECT
            session.id,
            session.game_key,
            session.game_version,
            session.revision,
            session.status,
            session.state,
            to_char(session.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(session.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
        FROM game_sessions AS session
        JOIN game_session_participants AS actor
          ON actor.game_session_id = session.id AND actor.persona_id = $1
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
            session.state,
            actor.seat AS actor_seat
        FROM game_sessions AS session
        JOIN game_session_participants AS actor
          ON actor.game_session_id = session.id AND actor.persona_id = $1
        WHERE session.id = $2 AND session.status = 'active'
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
            state: receipt.state.0,
        });
    }

    if locked.revision != expected_revision {
        return Err(GameError::RevisionConflict);
    }
    let game_version = u32::try_from(locked.game_version).map_err(|_| GameError::Internal)?;
    let actor_seat = u8::try_from(locked.actor_seat).map_err(|_| GameError::Internal)?;
    let next_state = registry
        .apply_command(
            &locked.game_key,
            game_version,
            &locked.state.0,
            actor_seat,
            &command,
        )
        .map_err(map_command_error)?;
    let applied_revision = locked.revision.checked_add(1).ok_or(GameError::Internal)?;

    let stored_revision = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE game_sessions
        SET state = $1,
            revision = revision + 1,
            updated_at = GREATEST(updated_at, clock_timestamp())
        WHERE id = $2 AND revision = $3
        RETURNING revision
        "#,
    )
    .bind(Json(&next_state))
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
            state
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(session_id)
    .bind(idempotency_key)
    .bind(actor_id)
    .bind(expected_revision)
    .bind(applied_revision)
    .bind(Json(&command))
    .bind(Json(&next_state))
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
    info!(%session_id, %actor_id, revision = applied_revision, "game command committed");
    Ok(GameCommandResult {
        game_session_id: session_id,
        revision: applied_revision,
        state: next_state,
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
                state: row.state.0,
                participants,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .collect()
}

async fn authenticate_owned_persona(
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
