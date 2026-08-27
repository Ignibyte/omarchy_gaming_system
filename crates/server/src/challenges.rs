//! Durable two-person game challenges and exact-session acceptance.

use omarchy_game_runtime::GameRegistry;
use omarchy_gaming_system_server::cartridge_distribution::CartridgeDistributionRuntime;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    connections::{self, ConnectionError},
    games::{self, GameError},
    inboxes::{self, GameChallengeMessage, InboxError},
    personas::Persona,
    sessions::{self, SessionError},
    sync::{self, SyncEventKind},
};

pub const DEFAULT_CHALLENGE_LIMIT: u16 = 50;
pub const MAX_CHALLENGE_LIMIT: u16 = 100;
pub const MAX_PENDING_CHALLENGES_PER_DIRECTION: i64 = 100;

#[derive(Debug, PartialEq, Eq)]
pub struct GameChallenge {
    pub id: Uuid,
    pub game_key: String,
    pub game_version: u32,
    pub direction: ChallengeDirection,
    pub status: String,
    pub challenger: Persona,
    pub challenged: Persona,
    pub game_session_id: Option<Uuid>,
    pub expires_at: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameChallengePage {
    pub challenges: Vec<GameChallenge>,
    pub next_before: Option<Uuid>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChallengeOutcome {
    Created(GameChallenge),
    Existing(GameChallenge),
}

pub struct CreateChallengeInput {
    pub idempotency_key: String,
    pub challenged_persona_id: String,
    pub game_key: String,
    pub game_version: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChallengeError {
    Unauthorized,
    PersonaNotFound,
    ChallengeNotFound,
    InvalidChallenge,
    InvalidPagination,
    GameUnavailable,
    TargetUnavailable,
    PendingLimitReached,
    DuplicatePending,
    IdempotencyConflict,
    TransitionUnavailable,
    ChallengeExpired,
    InitializationFailed,
    Internal,
}

#[derive(sqlx::FromRow)]
struct ChallengeRow {
    challenge_id: Uuid,
    challenger_persona_id: Uuid,
    challenged_persona_id: Uuid,
    game_key: String,
    game_version: i64,
    status: String,
    game_session_id: Option<Uuid>,
    expires_at: String,
    resolved_at: Option<String>,
    challenge_created_at: String,
    challenge_updated_at: String,
    challenger_handle: String,
    challenger_display_name: String,
    challenger_bio: String,
    challenger_status_message: String,
    challenger_created_at: String,
    challenger_updated_at: String,
    challenged_handle: String,
    challenged_display_name: String,
    challenged_bio: String,
    challenged_status_message: String,
    challenged_created_at: String,
    challenged_updated_at: String,
}

#[derive(Clone, Copy)]
enum Transition {
    Accept,
    Decline,
    Cancel,
}

pub async fn create_challenge(
    pool: &PgPool,
    registry: &GameRegistry,
    token: &str,
    actor_id: &str,
    input: CreateChallengeInput,
) -> Result<ChallengeOutcome, ChallengeError> {
    let (account_id, actor_id) = authenticate_owned_persona(pool, token, actor_id).await?;
    let target_id = Uuid::try_parse(&input.challenged_persona_id)
        .map_err(|_| ChallengeError::TargetUnavailable)?;
    let idempotency_key =
        Uuid::try_parse(&input.idempotency_key).map_err(|_| ChallengeError::InvalidChallenge)?;

    if let Some((challenge_id, existing_target, existing_key, existing_version)) =
        sqlx::query_as::<_, (Uuid, Uuid, String, i64)>(
            r#"
            SELECT id, challenged_persona_id, game_key, game_version
            FROM game_challenges
            WHERE challenger_persona_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(actor_id)
        .bind(idempotency_key)
        .fetch_optional(pool)
        .await
        .map_err(|database_error| map_database_error(database_error, "challenge replay lookup"))?
    {
        if existing_target != target_id
            || existing_key != input.game_key
            || existing_version != i64::from(input.game_version)
        {
            return Err(ChallengeError::IdempotencyConflict);
        }
        let challenge = load_owned_challenge(pool, account_id, actor_id, challenge_id).await?;
        return Ok(ChallengeOutcome::Existing(challenge));
    }

    let manifest = registry
        .manifest(&input.game_key, input.game_version)
        .filter(|manifest| manifest.min_human_players <= 2 && manifest.max_human_players >= 2)
        .ok_or(ChallengeError::GameUnavailable)?;

    let mut transaction = begin_transaction(pool, "challenge creation").await?;
    connections::lock_connected_pair(&mut transaction, account_id, actor_id, target_id)
        .await
        .map_err(map_connected_pair_error)?;
    expire_for_personas(&mut transaction, &[actor_id, target_id]).await?;

    if let Some((challenge_id, existing_target, existing_key, existing_version)) =
        sqlx::query_as::<_, (Uuid, Uuid, String, i64)>(
            r#"
            SELECT id, challenged_persona_id, game_key, game_version
            FROM game_challenges
            WHERE challenger_persona_id = $1 AND idempotency_key = $2
            FOR UPDATE
            "#,
        )
        .bind(actor_id)
        .bind(idempotency_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|database_error| map_database_error(database_error, "challenge replay lookup"))?
    {
        if existing_target != target_id
            || existing_key != manifest.key
            || existing_version != i64::from(manifest.version)
        {
            return Err(ChallengeError::IdempotencyConflict);
        }
        let challenge = load_challenge(&mut transaction, actor_id, challenge_id).await?;
        transaction.commit().await.map_err(|database_error| {
            map_database_error(database_error, "challenge replay commit")
        })?;
        return Ok(ChallengeOutcome::Existing(challenge));
    }

    let duplicate = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM game_challenges
            WHERE challenger_persona_id = $1
              AND challenged_persona_id = $2
              AND game_key = $3
              AND game_version = $4
              AND status = 'pending'
        )
        "#,
    )
    .bind(actor_id)
    .bind(target_id)
    .bind(&manifest.key)
    .bind(i64::from(manifest.version))
    .fetch_one(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "pending challenge lookup"))?;
    if duplicate {
        return Err(ChallengeError::DuplicatePending);
    }

    let (outgoing, incoming) = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            count(*) FILTER (WHERE challenger_persona_id = $1),
            count(*) FILTER (WHERE challenged_persona_id = $2)
        FROM game_challenges
        WHERE status = 'pending'
          AND expires_at > now()
          AND (challenger_persona_id = $1 OR challenged_persona_id = $2)
        "#,
    )
    .bind(actor_id)
    .bind(target_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "pending challenge limits"))?;
    if outgoing >= MAX_PENDING_CHALLENGES_PER_DIRECTION
        || incoming >= MAX_PENDING_CHALLENGES_PER_DIRECTION
    {
        return Err(ChallengeError::PendingLimitReached);
    }

    let challenge_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO game_challenges (
            idempotency_key,
            challenger_persona_id,
            challenged_persona_id,
            game_key,
            game_version,
            expires_at
        )
        VALUES ($1, $2, $3, $4, $5, now() + interval '7 days')
        RETURNING id
        "#,
    )
    .bind(idempotency_key)
    .bind(actor_id)
    .bind(target_id)
    .bind(&manifest.key)
    .bind(i64::from(manifest.version))
    .fetch_one(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge insertion"))?;
    let conversation_id = inboxes::record_game_challenge(
        &mut transaction,
        actor_id,
        target_id,
        actor_id,
        challenge_id,
        GameChallengeMessage::Created,
    )
    .await
    .map_err(map_inbox_error)?;
    append_change_events(
        &mut transaction,
        actor_id,
        target_id,
        challenge_id,
        conversation_id,
    )
    .await?;
    let challenge = load_challenge(&mut transaction, actor_id, challenge_id).await?;
    transaction.commit().await.map_err(|database_error| {
        map_database_error(database_error, "challenge creation commit")
    })?;
    info!(%challenge_id, %actor_id, %target_id, game_key = %manifest.key, game_version = manifest.version, "game challenge created");
    Ok(ChallengeOutcome::Created(challenge))
}

pub async fn list_challenges(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    before: Option<&str>,
    limit: Option<u16>,
) -> Result<GameChallengePage, ChallengeError> {
    let (account_id, actor_id) = authenticate_owned_persona(pool, token, actor_id).await?;
    let before = before
        .map(Uuid::try_parse)
        .transpose()
        .map_err(|_| ChallengeError::InvalidPagination)?;
    let limit = validate_limit(limit)?;
    let mut transaction = begin_transaction(pool, "challenge inventory").await?;
    lock_owned_persona(&mut transaction, account_id, actor_id).await?;
    expire_for_personas(&mut transaction, &[actor_id]).await?;
    if let Some(before) = before {
        let visible = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM game_challenges
                WHERE id = $1
                  AND (challenger_persona_id = $2 OR challenged_persona_id = $2)
            )
            "#,
        )
        .bind(before)
        .bind(actor_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|database_error| map_database_error(database_error, "challenge cursor lookup"))?;
        if !visible {
            return Err(ChallengeError::InvalidPagination);
        }
    }

    let mut rows = sqlx::query_as::<_, ChallengeRow>(
        r#"
        SELECT
            challenge.id AS challenge_id,
            challenge.challenger_persona_id,
            challenge.challenged_persona_id,
            challenge.game_key,
            challenge.game_version,
            challenge.status,
            challenge.game_session_id,
            to_char(challenge.expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS expires_at,
            CASE WHEN challenge.resolved_at IS NULL THEN NULL ELSE to_char(challenge.resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS resolved_at,
            to_char(challenge.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenge_created_at,
            to_char(challenge.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenge_updated_at,
            challenger.handle AS challenger_handle,
            challenger.display_name AS challenger_display_name,
            challenger.bio AS challenger_bio,
            challenger.status_message AS challenger_status_message,
            to_char(challenger.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenger_created_at,
            to_char(challenger.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenger_updated_at,
            challenged.handle AS challenged_handle,
            challenged.display_name AS challenged_display_name,
            challenged.bio AS challenged_bio,
            challenged.status_message AS challenged_status_message,
            to_char(challenged.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenged_created_at,
            to_char(challenged.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenged_updated_at
        FROM game_challenges AS challenge
        JOIN personas AS challenger ON challenger.id = challenge.challenger_persona_id
        JOIN personas AS challenged ON challenged.id = challenge.challenged_persona_id
        WHERE (challenge.challenger_persona_id = $1 OR challenge.challenged_persona_id = $1)
          AND (
              $2::uuid IS NULL
              OR (challenge.created_at, challenge.id) < (
                  SELECT cursor.created_at, cursor.id
                  FROM game_challenges AS cursor
                  WHERE cursor.id = $2
              )
          )
        ORDER BY challenge.created_at DESC, challenge.id DESC
        LIMIT $3
        "#,
    )
    .bind(actor_id)
    .bind(before)
    .bind(i64::from(limit) + 1)
    .fetch_all(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge inventory query"))?;
    let has_more = rows.len() > usize::from(limit);
    rows.truncate(usize::from(limit));
    let next_before = has_more
        .then(|| rows.last().map(|row| row.challenge_id))
        .flatten();
    let challenges = rows
        .into_iter()
        .map(|row| challenge_from_row(actor_id, row))
        .collect::<Result<Vec<_>, _>>()?;
    transaction.commit().await.map_err(|database_error| {
        map_database_error(database_error, "challenge inventory commit")
    })?;
    Ok(GameChallengePage {
        challenges,
        next_before,
    })
}

pub async fn get_challenge(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    challenge_id: &str,
) -> Result<GameChallenge, ChallengeError> {
    let (account_id, actor_id) = authenticate_owned_persona(pool, token, actor_id).await?;
    let challenge_id =
        Uuid::try_parse(challenge_id).map_err(|_| ChallengeError::ChallengeNotFound)?;
    load_owned_challenge(pool, account_id, actor_id, challenge_id).await
}

async fn load_owned_challenge(
    pool: &PgPool,
    account_id: Uuid,
    actor_id: Uuid,
    challenge_id: Uuid,
) -> Result<GameChallenge, ChallengeError> {
    let mut transaction = begin_transaction(pool, "challenge detail").await?;
    lock_owned_persona(&mut transaction, account_id, actor_id).await?;
    expire_challenge_for_participant(&mut transaction, actor_id, challenge_id).await?;
    let challenge = load_challenge(&mut transaction, actor_id, challenge_id).await?;
    transaction
        .commit()
        .await
        .map_err(|database_error| map_database_error(database_error, "challenge detail commit"))?;
    Ok(challenge)
}

pub async fn accept_challenge(
    pool: &PgPool,
    registry: &GameRegistry,
    cartridge_distribution: Option<&CartridgeDistributionRuntime>,
    token: &str,
    actor_id: &str,
    challenge_id: &str,
) -> Result<GameChallenge, ChallengeError> {
    transition_challenge(
        pool,
        registry,
        cartridge_distribution,
        token,
        actor_id,
        challenge_id,
        Transition::Accept,
    )
    .await
}

pub async fn decline_challenge(
    pool: &PgPool,
    registry: &GameRegistry,
    token: &str,
    actor_id: &str,
    challenge_id: &str,
) -> Result<GameChallenge, ChallengeError> {
    transition_challenge(
        pool,
        registry,
        None,
        token,
        actor_id,
        challenge_id,
        Transition::Decline,
    )
    .await
}

pub async fn cancel_challenge(
    pool: &PgPool,
    registry: &GameRegistry,
    token: &str,
    actor_id: &str,
    challenge_id: &str,
) -> Result<GameChallenge, ChallengeError> {
    transition_challenge(
        pool,
        registry,
        None,
        token,
        actor_id,
        challenge_id,
        Transition::Cancel,
    )
    .await
}

async fn transition_challenge(
    pool: &PgPool,
    registry: &GameRegistry,
    cartridge_distribution: Option<&CartridgeDistributionRuntime>,
    token: &str,
    actor_id: &str,
    challenge_id: &str,
    transition: Transition,
) -> Result<GameChallenge, ChallengeError> {
    let (account_id, actor_id) = authenticate_owned_persona(pool, token, actor_id).await?;
    let challenge_id =
        Uuid::try_parse(challenge_id).map_err(|_| ChallengeError::ChallengeNotFound)?;
    let (challenger_id, challenged_id) = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT challenger_persona_id, challenged_persona_id
        FROM game_challenges
        WHERE id = $1
          AND (challenger_persona_id = $2 OR challenged_persona_id = $2)
        "#,
    )
    .bind(challenge_id)
    .bind(actor_id)
    .fetch_optional(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge participant lookup"))?
    .ok_or(ChallengeError::ChallengeNotFound)?;
    let other_id = if actor_id == challenger_id {
        challenged_id
    } else {
        challenger_id
    };

    let mut transaction = begin_transaction(pool, "challenge transition").await?;
    connections::lock_persona_pair(&mut transaction, account_id, actor_id, other_id)
        .await
        .map_err(map_pair_error)?;
    expire_for_personas(&mut transaction, &[actor_id, other_id]).await?;
    let row = lock_challenge(&mut transaction, actor_id, challenge_id).await?;
    if row.status == "expired" {
        transaction.commit().await.map_err(|database_error| {
            map_database_error(database_error, "challenge expiry commit")
        })?;
        return Err(ChallengeError::ChallengeExpired);
    }
    let actor_authorized = match transition {
        Transition::Accept | Transition::Decline => actor_id == row.challenged_persona_id,
        Transition::Cancel => actor_id == row.challenger_persona_id,
    };
    if !actor_authorized {
        return Err(ChallengeError::TransitionUnavailable);
    }
    let desired_status = match transition {
        Transition::Accept => "accepted",
        Transition::Decline => "declined",
        Transition::Cancel => "cancelled",
    };
    if row.status == desired_status {
        let challenge = load_challenge(&mut transaction, actor_id, challenge_id).await?;
        transaction.commit().await.map_err(|database_error| {
            map_database_error(database_error, "challenge retry commit")
        })?;
        return Ok(challenge);
    }
    if row.status != "pending" {
        return Err(ChallengeError::TransitionUnavailable);
    }

    let (game_session_id, message) = match transition {
        Transition::Accept => {
            connections::lock_connected_pair(&mut transaction, account_id, actor_id, other_id)
                .await
                .map_err(map_connected_pair_error)?;
            let game_version =
                u32::try_from(row.game_version).map_err(|_| ChallengeError::Internal)?;
            let game_session_id = games::create_session_with_distribution(
                &mut transaction,
                registry,
                cartridge_distribution,
                &row.game_key,
                game_version,
                &[row.challenger_persona_id, row.challenged_persona_id],
            )
            .await
            .map_err(map_game_error)?;
            (
                Some(game_session_id),
                GameChallengeMessage::Accepted { game_session_id },
            )
        }
        Transition::Decline => (None, GameChallengeMessage::Declined),
        Transition::Cancel => (None, GameChallengeMessage::Cancelled),
    };

    sqlx::query(
        r#"
        UPDATE game_challenges
        SET status = $2,
            game_session_id = $3,
            resolved_at = now(),
            updated_at = now()
        WHERE id = $1 AND status = 'pending'
        "#,
    )
    .bind(challenge_id)
    .bind(desired_status)
    .bind(game_session_id)
    .execute(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge transition update"))?;
    let conversation_id = inboxes::record_game_challenge(
        &mut transaction,
        row.challenger_persona_id,
        row.challenged_persona_id,
        actor_id,
        challenge_id,
        message,
    )
    .await
    .map_err(map_inbox_error)?;
    append_change_events(
        &mut transaction,
        row.challenger_persona_id,
        row.challenged_persona_id,
        challenge_id,
        conversation_id,
    )
    .await?;
    let challenge = load_challenge(&mut transaction, actor_id, challenge_id).await?;
    transaction.commit().await.map_err(|database_error| {
        map_database_error(database_error, "challenge transition commit")
    })?;
    info!(%challenge_id, %actor_id, status = desired_status, "game challenge transitioned");
    Ok(challenge)
}

#[derive(sqlx::FromRow)]
struct LockedChallengeRow {
    challenger_persona_id: Uuid,
    challenged_persona_id: Uuid,
    game_key: String,
    game_version: i64,
    status: String,
}

async fn lock_challenge(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    challenge_id: Uuid,
) -> Result<LockedChallengeRow, ChallengeError> {
    sqlx::query_as::<_, LockedChallengeRow>(
        r#"
        SELECT challenger_persona_id, challenged_persona_id, game_key, game_version, status
        FROM game_challenges
        WHERE id = $1
          AND (challenger_persona_id = $2 OR challenged_persona_id = $2)
        FOR UPDATE
        "#,
    )
    .bind(challenge_id)
    .bind(actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge locking"))?
    .ok_or(ChallengeError::ChallengeNotFound)
}

async fn load_challenge(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    challenge_id: Uuid,
) -> Result<GameChallenge, ChallengeError> {
    let row = sqlx::query_as::<_, ChallengeRow>(
        r#"
        SELECT
            challenge.id AS challenge_id,
            challenge.challenger_persona_id,
            challenge.challenged_persona_id,
            challenge.game_key,
            challenge.game_version,
            challenge.status,
            challenge.game_session_id,
            to_char(challenge.expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS expires_at,
            CASE WHEN challenge.resolved_at IS NULL THEN NULL ELSE to_char(challenge.resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS resolved_at,
            to_char(challenge.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenge_created_at,
            to_char(challenge.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenge_updated_at,
            challenger.handle AS challenger_handle,
            challenger.display_name AS challenger_display_name,
            challenger.bio AS challenger_bio,
            challenger.status_message AS challenger_status_message,
            to_char(challenger.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenger_created_at,
            to_char(challenger.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenger_updated_at,
            challenged.handle AS challenged_handle,
            challenged.display_name AS challenged_display_name,
            challenged.bio AS challenged_bio,
            challenged.status_message AS challenged_status_message,
            to_char(challenged.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenged_created_at,
            to_char(challenged.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS challenged_updated_at
        FROM game_challenges AS challenge
        JOIN personas AS challenger ON challenger.id = challenge.challenger_persona_id
        JOIN personas AS challenged ON challenged.id = challenge.challenged_persona_id
        WHERE challenge.id = $1
          AND (challenge.challenger_persona_id = $2 OR challenge.challenged_persona_id = $2)
        "#,
    )
    .bind(challenge_id)
    .bind(actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge loading"))?
    .ok_or(ChallengeError::ChallengeNotFound)?;
    challenge_from_row(actor_id, row)
}

fn challenge_from_row(actor_id: Uuid, row: ChallengeRow) -> Result<GameChallenge, ChallengeError> {
    let direction = if row.challenger_persona_id == actor_id {
        ChallengeDirection::Outgoing
    } else if row.challenged_persona_id == actor_id {
        ChallengeDirection::Incoming
    } else {
        return Err(ChallengeError::ChallengeNotFound);
    };
    let game_version = u32::try_from(row.game_version).map_err(|_| ChallengeError::Internal)?;
    Ok(GameChallenge {
        id: row.challenge_id,
        game_key: row.game_key,
        game_version,
        direction,
        status: row.status,
        challenger: Persona {
            id: row.challenger_persona_id,
            handle: row.challenger_handle,
            display_name: row.challenger_display_name,
            bio: row.challenger_bio,
            status_message: row.challenger_status_message,
            created_at: row.challenger_created_at,
            updated_at: row.challenger_updated_at,
        },
        challenged: Persona {
            id: row.challenged_persona_id,
            handle: row.challenged_handle,
            display_name: row.challenged_display_name,
            bio: row.challenged_bio,
            status_message: row.challenged_status_message,
            created_at: row.challenged_created_at,
            updated_at: row.challenged_updated_at,
        },
        game_session_id: row.game_session_id,
        expires_at: row.expires_at,
        resolved_at: row.resolved_at,
        created_at: row.challenge_created_at,
        updated_at: row.challenge_updated_at,
    })
}

async fn expire_for_personas(
    transaction: &mut Transaction<'_, Postgres>,
    persona_ids: &[Uuid],
) -> Result<(), ChallengeError> {
    sqlx::query(
        r#"
        UPDATE game_challenges
        SET status = 'expired', resolved_at = now(), updated_at = now()
        WHERE status = 'pending'
          AND expires_at <= now()
          AND (
              challenger_persona_id = ANY($1)
              OR challenged_persona_id = ANY($1)
          )
        "#,
    )
    .bind(persona_ids)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|database_error| map_database_error(database_error, "challenge expiration"))
}

async fn expire_challenge_for_participant(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    challenge_id: Uuid,
) -> Result<(), ChallengeError> {
    sqlx::query(
        r#"
        UPDATE game_challenges
        SET status = 'expired', resolved_at = now(), updated_at = now()
        WHERE id = $1
          AND status = 'pending'
          AND expires_at <= now()
          AND (challenger_persona_id = $2 OR challenged_persona_id = $2)
        "#,
    )
    .bind(challenge_id)
    .bind(actor_id)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|database_error| map_database_error(database_error, "challenge detail expiration"))
}

async fn append_change_events(
    transaction: &mut Transaction<'_, Postgres>,
    first_id: Uuid,
    second_id: Uuid,
    challenge_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), ChallengeError> {
    let mut persona_ids = [first_id, second_id];
    persona_ids.sort_unstable();
    for persona_id in persona_ids {
        sync::append_event(
            transaction,
            persona_id,
            SyncEventKind::GameChallenge(challenge_id),
        )
        .await
        .map_err(|database_error| {
            error!(?database_error, %persona_id, %challenge_id, "challenge sync event append failed");
            ChallengeError::Internal
        })?;
        sync::append_event(
            transaction,
            persona_id,
            SyncEventKind::Conversation(conversation_id),
        )
        .await
        .map_err(|database_error| {
            error!(?database_error, %persona_id, %conversation_id, "challenge inbox event append failed");
            ChallengeError::Internal
        })?;
    }
    Ok(())
}

async fn authenticate_owned_persona(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
) -> Result<(Uuid, Uuid), ChallengeError> {
    let authenticated = sessions::authenticate(pool, token)
        .await
        .map_err(|session_error| match session_error {
            SessionError::Unauthorized => ChallengeError::Unauthorized,
            _ => ChallengeError::Internal,
        })?;
    let actor_id = Uuid::try_parse(actor_id).map_err(|_| ChallengeError::PersonaNotFound)?;
    let owned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM personas WHERE id = $1 AND account_id = $2)",
    )
    .bind(actor_id)
    .bind(authenticated.account_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge actor ownership"))?;
    if owned {
        Ok((authenticated.account_id, actor_id))
    } else {
        Err(ChallengeError::PersonaNotFound)
    }
}

async fn lock_owned_persona(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    actor_id: Uuid,
) -> Result<(), ChallengeError> {
    let locked_account_id =
        sqlx::query_scalar::<_, Uuid>("SELECT account_id FROM personas WHERE id = $1 FOR UPDATE")
            .bind(actor_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|database_error| {
                map_database_error(database_error, "challenge actor locking")
            })?;
    if locked_account_id == Some(account_id) {
        Ok(())
    } else {
        Err(ChallengeError::PersonaNotFound)
    }
}

fn validate_limit(limit: Option<u16>) -> Result<u16, ChallengeError> {
    let limit = limit.unwrap_or(DEFAULT_CHALLENGE_LIMIT);
    if (1..=MAX_CHALLENGE_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(ChallengeError::InvalidPagination)
    }
}

async fn begin_transaction<'a>(
    pool: &'a PgPool,
    operation: &'static str,
) -> Result<Transaction<'a, Postgres>, ChallengeError> {
    pool.begin()
        .await
        .map_err(|database_error| map_database_error(database_error, operation))
}

fn map_connected_pair_error(error: ConnectionError) -> ChallengeError {
    match error {
        ConnectionError::PersonaNotFound => ChallengeError::PersonaNotFound,
        ConnectionError::ConnectionUnavailable
        | ConnectionError::ConnectionRequestNotFound
        | ConnectionError::ConnectionRequestPending
        | ConnectionError::ConnectionAlreadyExists => ChallengeError::TargetUnavailable,
        ConnectionError::Unauthorized => ChallengeError::Unauthorized,
        ConnectionError::Internal => ChallengeError::Internal,
    }
}

fn map_pair_error(error: ConnectionError) -> ChallengeError {
    match error {
        ConnectionError::PersonaNotFound => ChallengeError::PersonaNotFound,
        ConnectionError::ConnectionUnavailable
        | ConnectionError::ConnectionRequestNotFound
        | ConnectionError::ConnectionRequestPending
        | ConnectionError::ConnectionAlreadyExists => ChallengeError::ChallengeNotFound,
        ConnectionError::Unauthorized => ChallengeError::Unauthorized,
        ConnectionError::Internal => ChallengeError::Internal,
    }
}

fn map_inbox_error(error: InboxError) -> ChallengeError {
    error!(?error, "challenge inbox operation failed");
    ChallengeError::Internal
}

fn map_game_error(error: GameError) -> ChallengeError {
    match error {
        GameError::GameUnavailable => ChallengeError::GameUnavailable,
        GameError::InitializationFailed => ChallengeError::InitializationFailed,
        _ => ChallengeError::Internal,
    }
}

fn map_database_error(database_error: sqlx::Error, operation: &'static str) -> ChallengeError {
    error!(
        ?database_error,
        operation, "game challenge database operation failed"
    );
    ChallengeError::Internal
}
