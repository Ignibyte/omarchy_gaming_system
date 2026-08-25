//! Private one-to-one persona conversations, messages, and unread state.

use sqlx::{FromRow, PgPool, Postgres, Transaction};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    connections::{self, ConnectionError},
    personas::Persona,
    sessions::{self, SessionError},
    sync::{self, SyncEventKind},
};

const MAX_MESSAGE_CHARACTERS: usize = 4_000;
const DEFAULT_HISTORY_LIMIT: u16 = 50;
const MAX_HISTORY_LIMIT: u16 = 100;
const DEFAULT_CONVERSATION_LIMIT: u16 = 50;
const MAX_CONVERSATION_LIMIT: u16 = 100;

#[derive(Debug, PartialEq, Eq)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub other_persona: Persona,
    pub unread_count: i64,
    pub latest_message: Option<InboxMessage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MessagePage {
    pub messages: Vec<InboxMessage>,
    pub next_before: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReadReceipt {
    pub through_message_id: Uuid,
    pub unread_count: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InboxMessage {
    pub id: Uuid,
    pub sequence: i64,
    pub content: InboxMessageContent,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InboxMessageContent {
    User { sender: Persona, body: String },
    System(SystemMessage),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SystemMessage {
    ConnectionAccepted {
        actor: Persona,
    },
    GameChallengeCreated {
        actor: Persona,
        challenge_id: Uuid,
    },
    GameChallengeAccepted {
        actor: Persona,
        challenge_id: Uuid,
        game_session_id: Uuid,
    },
    GameChallengeDeclined {
        actor: Persona,
        challenge_id: Uuid,
    },
    GameChallengeCancelled {
        actor: Persona,
        challenge_id: Uuid,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameChallengeMessage {
    Created,
    Accepted { game_session_id: Uuid },
    Declined,
    Cancelled,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InboxError {
    Unauthorized,
    PersonaNotFound,
    ConversationNotFound,
    ConversationUnavailable,
    MessageNotFound,
    InvalidMessageBody,
    InvalidPagination,
    Internal,
}

#[derive(FromRow)]
struct MessageRow {
    message_id: Uuid,
    message_sequence: i64,
    message_type: String,
    user_body: Option<String>,
    message_created_at: String,
    sender_id: Option<Uuid>,
    sender_handle: Option<String>,
    sender_display_name: Option<String>,
    sender_bio: Option<String>,
    sender_status_message: Option<String>,
    sender_created_at: Option<String>,
    sender_updated_at: Option<String>,
    system_type: Option<String>,
    system_game_challenge_id: Option<Uuid>,
    system_game_session_id: Option<Uuid>,
    actor_id: Option<Uuid>,
    actor_handle: Option<String>,
    actor_display_name: Option<String>,
    actor_bio: Option<String>,
    actor_status_message: Option<String>,
    actor_created_at: Option<String>,
    actor_updated_at: Option<String>,
}

#[derive(FromRow)]
struct ConversationRow {
    conversation_id: Uuid,
    other_id: Uuid,
    other_handle: String,
    other_display_name: String,
    other_bio: String,
    other_status_message: String,
    other_created_at: String,
    other_updated_at: String,
    unread_count: i64,
    conversation_created_at: String,
    conversation_updated_at: String,
    message_id: Option<Uuid>,
    message_sequence: Option<i64>,
    message_type: Option<String>,
    user_body: Option<String>,
    message_created_at: Option<String>,
    sender_id: Option<Uuid>,
    sender_handle: Option<String>,
    sender_display_name: Option<String>,
    sender_bio: Option<String>,
    sender_status_message: Option<String>,
    sender_created_at: Option<String>,
    sender_updated_at: Option<String>,
    system_type: Option<String>,
    system_game_challenge_id: Option<Uuid>,
    system_game_session_id: Option<Uuid>,
    actor_id: Option<Uuid>,
    actor_handle: Option<String>,
    actor_display_name: Option<String>,
    actor_bio: Option<String>,
    actor_status_message: Option<String>,
    actor_created_at: Option<String>,
    actor_updated_at: Option<String>,
}

pub async fn list_conversations(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    limit: Option<u16>,
) -> Result<Vec<ConversationSummary>, InboxError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    ensure_actor_owned(pool, account_id, actor_id).await?;
    let limit = validate_conversation_limit(limit)?;

    let rows = sqlx::query_as::<_, ConversationRow>(
        r#"
        SELECT
            conversation.id AS conversation_id,
            other.id AS other_id,
            other.handle AS other_handle,
            other.display_name AS other_display_name,
            other.bio AS other_bio,
            other.status_message AS other_status_message,
            to_char(other.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS other_created_at,
            to_char(other.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS other_updated_at,
            (
                SELECT count(*)
                FROM inbox_messages AS unread
                WHERE unread.conversation_id = conversation.id
                  AND unread.message_sequence > CASE
                      WHEN conversation.persona_low_id = $1
                          THEN conversation.low_last_read_sequence
                      ELSE conversation.high_last_read_sequence
                  END
                  AND (unread.sender_persona_id IS NULL OR unread.sender_persona_id <> $1)
            ) AS unread_count,
            to_char(conversation.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS conversation_created_at,
            to_char(conversation.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS conversation_updated_at,
            latest.id AS message_id,
            latest.message_sequence,
            latest.message_type,
            latest.user_body,
            to_char(latest.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS message_created_at,
            sender.id AS sender_id,
            sender.handle AS sender_handle,
            sender.display_name AS sender_display_name,
            sender.bio AS sender_bio,
            sender.status_message AS sender_status_message,
            CASE WHEN sender.id IS NULL THEN NULL ELSE to_char(sender.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS sender_created_at,
            CASE WHEN sender.id IS NULL THEN NULL ELSE to_char(sender.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS sender_updated_at,
            latest.system_type,
            latest.system_game_challenge_id,
            latest.system_game_session_id,
            actor.id AS actor_id,
            actor.handle AS actor_handle,
            actor.display_name AS actor_display_name,
            actor.bio AS actor_bio,
            actor.status_message AS actor_status_message,
            CASE WHEN actor.id IS NULL THEN NULL ELSE to_char(actor.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS actor_created_at,
            CASE WHEN actor.id IS NULL THEN NULL ELSE to_char(actor.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS actor_updated_at
        FROM inbox_conversations AS conversation
        JOIN personas AS other
          ON other.id = CASE
              WHEN conversation.persona_low_id = $1 THEN conversation.persona_high_id
              ELSE conversation.persona_low_id
          END
        LEFT JOIN LATERAL (
            SELECT *
            FROM inbox_messages AS message
            WHERE message.conversation_id = conversation.id
            ORDER BY message.message_sequence DESC
            LIMIT 1
        ) AS latest ON true
        LEFT JOIN personas AS sender ON sender.id = latest.sender_persona_id
        LEFT JOIN personas AS actor ON actor.id = latest.system_actor_persona_id
        WHERE conversation.persona_low_id = $1 OR conversation.persona_high_id = $1
        ORDER BY conversation.updated_at DESC, conversation.id
        LIMIT $2
        "#,
    )
    .bind(actor_id)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "conversation listing"))?;

    rows.into_iter().map(summary_from_row).collect()
}

pub async fn list_messages(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    conversation_id: &str,
    before: Option<i64>,
    limit: Option<u16>,
) -> Result<MessagePage, InboxError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    ensure_actor_owned(pool, account_id, actor_id).await?;
    let conversation_id = parse_conversation_id(conversation_id)?;
    let (before, limit) = validate_history_page(before, limit)?;
    ensure_conversation_participant(pool, conversation_id, actor_id).await?;

    let mut rows = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT
            message.id AS message_id,
            message.message_sequence,
            message.message_type,
            message.user_body,
            to_char(message.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS message_created_at,
            sender.id AS sender_id,
            sender.handle AS sender_handle,
            sender.display_name AS sender_display_name,
            sender.bio AS sender_bio,
            sender.status_message AS sender_status_message,
            CASE WHEN sender.id IS NULL THEN NULL ELSE to_char(sender.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS sender_created_at,
            CASE WHEN sender.id IS NULL THEN NULL ELSE to_char(sender.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS sender_updated_at,
            message.system_type,
            message.system_game_challenge_id,
            message.system_game_session_id,
            actor.id AS actor_id,
            actor.handle AS actor_handle,
            actor.display_name AS actor_display_name,
            actor.bio AS actor_bio,
            actor.status_message AS actor_status_message,
            CASE WHEN actor.id IS NULL THEN NULL ELSE to_char(actor.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS actor_created_at,
            CASE WHEN actor.id IS NULL THEN NULL ELSE to_char(actor.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') END AS actor_updated_at
        FROM inbox_messages AS message
        LEFT JOIN personas AS sender ON sender.id = message.sender_persona_id
        LEFT JOIN personas AS actor ON actor.id = message.system_actor_persona_id
        WHERE message.conversation_id = $1
          AND ($2::bigint IS NULL OR message.message_sequence < $2)
        ORDER BY message.message_sequence DESC
        LIMIT $3
        "#,
    )
    .bind(conversation_id)
    .bind(before)
    .bind(i64::from(limit) + 1)
    .fetch_all(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "message history listing"))?;

    let has_more = rows.len() > usize::from(limit);
    rows.truncate(usize::from(limit));
    let next_before = has_more
        .then(|| rows.last().map(|row| row.message_sequence))
        .flatten();
    rows.reverse();

    Ok(MessagePage {
        messages: rows
            .into_iter()
            .map(message_from_row)
            .collect::<Result<_, _>>()?,
        next_before,
    })
}

pub async fn send_user_message(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    conversation_id: &str,
    body: &str,
) -> Result<InboxMessage, InboxError> {
    let body = validate_message_body(body)?;
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    let conversation_id = parse_conversation_id(conversation_id)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|database_error| map_database_error(database_error, "message send transaction"))?;
    let (low_id, high_id) =
        load_conversation_pair(&mut transaction, conversation_id, actor_id).await?;
    let other_id = if actor_id == low_id { high_id } else { low_id };

    connections::lock_connected_pair(&mut transaction, account_id, actor_id, other_id)
        .await
        .map_err(map_connection_send_error)?;
    let sequence =
        lock_conversation_and_next_sequence(&mut transaction, conversation_id, low_id, high_id)
            .await?;

    let (message_id, sequence, created_at) = sqlx::query_as::<_, (Uuid, i64, String)>(
        r#"
        INSERT INTO inbox_messages (
            conversation_id,
            message_sequence,
            sender_persona_id,
            message_type,
            user_body
        )
        VALUES ($1, $2, $3, 'user', $4)
        RETURNING
            id,
            message_sequence,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        "#,
    )
    .bind(conversation_id)
    .bind(sequence)
    .bind(actor_id)
    .bind(&body)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "user message insertion"))?;

    update_latest_and_actor_read(&mut transaction, conversation_id, actor_id, sequence).await?;
    let sender = load_persona(&mut transaction, actor_id).await?;
    append_conversation_event(&mut transaction, low_id, conversation_id).await?;
    append_conversation_event(&mut transaction, high_id, conversation_id).await?;
    transaction
        .commit()
        .await
        .map_err(|database_error| map_database_error(database_error, "message send commit"))?;
    info!(%conversation_id, %actor_id, %message_id, "inbox user message created");

    Ok(InboxMessage {
        id: message_id,
        sequence,
        content: InboxMessageContent::User { sender, body },
        created_at,
    })
}

pub async fn mark_read(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    conversation_id: &str,
    message_id: &str,
) -> Result<ReadReceipt, InboxError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    ensure_actor_owned(pool, account_id, actor_id).await?;
    let conversation_id = parse_conversation_id(conversation_id)?;
    let message_id = Uuid::try_parse(message_id).map_err(|_| InboxError::MessageNotFound)?;
    let mut transaction = pool.begin().await.map_err(|database_error| {
        map_database_error(database_error, "read acknowledgement transaction")
    })?;
    let (low_id, high_id) =
        lock_conversation_for_participant(&mut transaction, conversation_id, actor_id).await?;
    let sequence = sqlx::query_scalar::<_, i64>(
        "SELECT message_sequence FROM inbox_messages WHERE id = $1 AND conversation_id = $2",
    )
    .bind(message_id)
    .bind(conversation_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "read message lookup"))?
    .ok_or(InboxError::MessageNotFound)?;

    let changed = update_actor_read(&mut transaction, conversation_id, actor_id, sequence).await?;
    let unread_count = unread_count(&mut transaction, conversation_id, actor_id).await?;
    if changed {
        append_conversation_event(&mut transaction, actor_id, conversation_id).await?;
    }
    debug_assert!(actor_id == low_id || actor_id == high_id);
    transaction.commit().await.map_err(|database_error| {
        map_database_error(database_error, "read acknowledgement commit")
    })?;

    Ok(ReadReceipt {
        through_message_id: message_id,
        unread_count,
    })
}

pub(crate) async fn record_connection_accepted(
    transaction: &mut Transaction<'_, Postgres>,
    low_id: Uuid,
    high_id: Uuid,
    accepting_actor_id: Uuid,
) -> Result<Uuid, InboxError> {
    let conversation_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO inbox_conversations (persona_low_id, persona_high_id)
        VALUES ($1, $2)
        ON CONFLICT (persona_low_id, persona_high_id) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(low_id)
    .bind(high_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "accepted conversation insertion")
    })?;

    let conversation_id = match conversation_id {
        Some(conversation_id) => conversation_id,
        None => sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM inbox_conversations
            WHERE persona_low_id = $1 AND persona_high_id = $2
            FOR UPDATE
            "#,
        )
        .bind(low_id)
        .bind(high_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|database_error| {
            map_database_error(database_error, "accepted conversation locking")
        })?,
    };

    let sequence =
        lock_conversation_and_next_sequence(transaction, conversation_id, low_id, high_id).await?;
    sqlx::query(
        r#"
        INSERT INTO inbox_messages (
            conversation_id,
            message_sequence,
            message_type,
            system_type,
            system_actor_persona_id
        )
        VALUES ($1, $2, 'system', 'connection_accepted', $3)
        "#,
    )
    .bind(conversation_id)
    .bind(sequence)
    .bind(accepting_actor_id)
    .execute(&mut **transaction)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "connection system message insertion")
    })?;
    update_latest_and_actor_read(transaction, conversation_id, accepting_actor_id, sequence)
        .await?;
    Ok(conversation_id)
}

pub(crate) async fn record_game_challenge(
    transaction: &mut Transaction<'_, Postgres>,
    first_id: Uuid,
    second_id: Uuid,
    actor_id: Uuid,
    challenge_id: Uuid,
    message: GameChallengeMessage,
) -> Result<Uuid, InboxError> {
    let (low_id, high_id) = if first_id < second_id {
        (first_id, second_id)
    } else {
        (second_id, first_id)
    };
    let conversation_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM inbox_conversations
        WHERE persona_low_id = $1 AND persona_high_id = $2
        FOR UPDATE
        "#,
    )
    .bind(low_id)
    .bind(high_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "challenge conversation locking"))?
    .ok_or(InboxError::ConversationNotFound)?;
    let sequence =
        lock_conversation_and_next_sequence(transaction, conversation_id, low_id, high_id).await?;
    let (system_type, game_session_id) = match message {
        GameChallengeMessage::Created => ("game_challenge_created", None),
        GameChallengeMessage::Accepted { game_session_id } => {
            ("game_challenge_accepted", Some(game_session_id))
        }
        GameChallengeMessage::Declined => ("game_challenge_declined", None),
        GameChallengeMessage::Cancelled => ("game_challenge_cancelled", None),
    };
    sqlx::query(
        r#"
        INSERT INTO inbox_messages (
            conversation_id,
            message_sequence,
            message_type,
            system_type,
            system_actor_persona_id,
            system_game_challenge_id,
            system_game_session_id
        )
        VALUES ($1, $2, 'system', $3, $4, $5, $6)
        "#,
    )
    .bind(conversation_id)
    .bind(sequence)
    .bind(system_type)
    .bind(actor_id)
    .bind(challenge_id)
    .bind(game_session_id)
    .execute(&mut **transaction)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "challenge system message insertion")
    })?;
    update_latest_and_actor_read(transaction, conversation_id, actor_id, sequence).await?;
    Ok(conversation_id)
}

fn validate_message_body(body: &str) -> Result<String, InboxError> {
    let body = body.trim();
    let character_count = body.chars().count();
    if !(1..=MAX_MESSAGE_CHARACTERS).contains(&character_count)
        || body
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
    {
        return Err(InboxError::InvalidMessageBody);
    }
    Ok(body.to_owned())
}

fn validate_conversation_limit(limit: Option<u16>) -> Result<u16, InboxError> {
    let limit = limit.unwrap_or(DEFAULT_CONVERSATION_LIMIT);
    if (1..=MAX_CONVERSATION_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(InboxError::InvalidPagination)
    }
}

fn validate_history_page(
    before: Option<i64>,
    limit: Option<u16>,
) -> Result<(Option<i64>, u16), InboxError> {
    if before.is_some_and(|before| before <= 0) {
        return Err(InboxError::InvalidPagination);
    }
    let limit = limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
    if !(1..=MAX_HISTORY_LIMIT).contains(&limit) {
        return Err(InboxError::InvalidPagination);
    }
    Ok((before, limit))
}

async fn authenticated_account_id(pool: &PgPool, token: &str) -> Result<Uuid, InboxError> {
    sessions::authenticate(pool, token)
        .await
        .map(|authenticated| authenticated.account_id)
        .map_err(|session_error| match session_error {
            SessionError::Unauthorized => InboxError::Unauthorized,
            _ => InboxError::Internal,
        })
}

fn parse_actor_id(actor_id: &str) -> Result<Uuid, InboxError> {
    Uuid::try_parse(actor_id).map_err(|_| InboxError::PersonaNotFound)
}

fn parse_conversation_id(conversation_id: &str) -> Result<Uuid, InboxError> {
    Uuid::try_parse(conversation_id).map_err(|_| InboxError::ConversationNotFound)
}

async fn ensure_actor_owned(
    pool: &PgPool,
    account_id: Uuid,
    actor_id: Uuid,
) -> Result<(), InboxError> {
    let owned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM personas WHERE id = $1 AND account_id = $2)",
    )
    .bind(actor_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "inbox actor ownership"))?;
    if owned {
        Ok(())
    } else {
        Err(InboxError::PersonaNotFound)
    }
}

async fn ensure_conversation_participant(
    pool: &PgPool,
    conversation_id: Uuid,
    actor_id: Uuid,
) -> Result<(), InboxError> {
    let participant = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM inbox_conversations
            WHERE id = $1 AND (persona_low_id = $2 OR persona_high_id = $2)
        )
        "#,
    )
    .bind(conversation_id)
    .bind(actor_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "conversation participant check")
    })?;
    if participant {
        Ok(())
    } else {
        Err(InboxError::ConversationNotFound)
    }
}

async fn load_conversation_pair(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    actor_id: Uuid,
) -> Result<(Uuid, Uuid), InboxError> {
    sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT persona_low_id, persona_high_id
        FROM inbox_conversations
        WHERE id = $1 AND (persona_low_id = $2 OR persona_high_id = $2)
        "#,
    )
    .bind(conversation_id)
    .bind(actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "conversation pair lookup"))?
    .ok_or(InboxError::ConversationNotFound)
}

async fn lock_conversation_and_next_sequence(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    low_id: Uuid,
    high_id: Uuid,
) -> Result<i64, InboxError> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(last_message_sequence, 0) + 1
        FROM inbox_conversations
        WHERE id = $1 AND persona_low_id = $2 AND persona_high_id = $3
        FOR UPDATE
        "#,
    )
    .bind(conversation_id)
    .bind(low_id)
    .bind(high_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "conversation locking"))?
    .ok_or(InboxError::ConversationNotFound)
}

async fn lock_conversation_for_participant(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    actor_id: Uuid,
) -> Result<(Uuid, Uuid), InboxError> {
    sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT persona_low_id, persona_high_id
        FROM inbox_conversations
        WHERE id = $1 AND (persona_low_id = $2 OR persona_high_id = $2)
        FOR UPDATE
        "#,
    )
    .bind(conversation_id)
    .bind(actor_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "participant conversation locking")
    })?
    .ok_or(InboxError::ConversationNotFound)
}

async fn update_latest_and_actor_read(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    actor_id: Uuid,
    sequence: i64,
) -> Result<(), InboxError> {
    sqlx::query(
        r#"
        UPDATE inbox_conversations
        SET
            last_message_sequence = $3,
            low_last_read_sequence = CASE
                WHEN persona_low_id = $2 THEN GREATEST(low_last_read_sequence, $3)
                ELSE low_last_read_sequence
            END,
            high_last_read_sequence = CASE
                WHEN persona_high_id = $2 THEN GREATEST(high_last_read_sequence, $3)
                ELSE high_last_read_sequence
            END,
            updated_at = now()
        WHERE id = $1 AND (persona_low_id = $2 OR persona_high_id = $2)
        "#,
    )
    .bind(conversation_id)
    .bind(actor_id)
    .bind(sequence)
    .execute(&mut **transaction)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "conversation latest message update")
    })?;
    Ok(())
}

async fn update_actor_read(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    actor_id: Uuid,
    sequence: i64,
) -> Result<bool, InboxError> {
    let updated = sqlx::query(
        r#"
        UPDATE inbox_conversations
        SET
            low_last_read_sequence = CASE
                WHEN persona_low_id = $2 THEN GREATEST(low_last_read_sequence, $3)
                ELSE low_last_read_sequence
            END,
            high_last_read_sequence = CASE
                WHEN persona_high_id = $2 THEN GREATEST(high_last_read_sequence, $3)
                ELSE high_last_read_sequence
            END
        WHERE id = $1
          AND (
              (persona_low_id = $2 AND low_last_read_sequence < $3)
              OR (persona_high_id = $2 AND high_last_read_sequence < $3)
          )
        "#,
    )
    .bind(conversation_id)
    .bind(actor_id)
    .bind(sequence)
    .execute(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "conversation read update"))?;
    Ok(updated.rows_affected() > 0)
}

async fn append_conversation_event(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: Uuid,
    conversation_id: Uuid,
) -> Result<(), InboxError> {
    sync::append_event(
        transaction,
        persona_id,
        SyncEventKind::Conversation(conversation_id),
    )
    .await
    .map(|_| ())
    .map_err(|database_error| {
        error!(?database_error, %persona_id, %conversation_id, "inbox sync event append failed");
        InboxError::Internal
    })
}

async fn unread_count(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    actor_id: Uuid,
) -> Result<i64, InboxError> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM inbox_messages AS message
        JOIN inbox_conversations AS conversation ON conversation.id = message.conversation_id
        WHERE conversation.id = $1
          AND (conversation.persona_low_id = $2 OR conversation.persona_high_id = $2)
          AND message.message_sequence > CASE
              WHEN conversation.persona_low_id = $2
                  THEN conversation.low_last_read_sequence
              ELSE conversation.high_last_read_sequence
          END
          AND (message.sender_persona_id IS NULL OR message.sender_persona_id <> $2)
        "#,
    )
    .bind(conversation_id)
    .bind(actor_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "unread count"))
}

async fn load_persona(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: Uuid,
) -> Result<Persona, InboxError> {
    let row = sqlx::query_as::<_, (Uuid, String, String, String, String, String, String)>(
        r#"
        SELECT
            id,
            handle,
            display_name,
            bio,
            status_message,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        FROM personas
        WHERE id = $1
        "#,
    )
    .bind(persona_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "inbox persona lookup"))?
    .ok_or(InboxError::Internal)?;
    Ok(Persona {
        id: row.0,
        handle: row.1,
        display_name: row.2,
        bio: row.3,
        status_message: row.4,
        created_at: row.5,
        updated_at: row.6,
    })
}

fn summary_from_row(row: ConversationRow) -> Result<ConversationSummary, InboxError> {
    let other_persona = Persona {
        id: row.other_id,
        handle: row.other_handle,
        display_name: row.other_display_name,
        bio: row.other_bio,
        status_message: row.other_status_message,
        created_at: row.other_created_at,
        updated_at: row.other_updated_at,
    };
    let latest_message = match row.message_id {
        Some(message_id) => Some(message_from_row(MessageRow {
            message_id,
            message_sequence: row.message_sequence.ok_or(InboxError::Internal)?,
            message_type: row.message_type.ok_or(InboxError::Internal)?,
            user_body: row.user_body,
            message_created_at: row.message_created_at.ok_or(InboxError::Internal)?,
            sender_id: row.sender_id,
            sender_handle: row.sender_handle,
            sender_display_name: row.sender_display_name,
            sender_bio: row.sender_bio,
            sender_status_message: row.sender_status_message,
            sender_created_at: row.sender_created_at,
            sender_updated_at: row.sender_updated_at,
            system_type: row.system_type,
            system_game_challenge_id: row.system_game_challenge_id,
            system_game_session_id: row.system_game_session_id,
            actor_id: row.actor_id,
            actor_handle: row.actor_handle,
            actor_display_name: row.actor_display_name,
            actor_bio: row.actor_bio,
            actor_status_message: row.actor_status_message,
            actor_created_at: row.actor_created_at,
            actor_updated_at: row.actor_updated_at,
        })?),
        None => None,
    };

    Ok(ConversationSummary {
        id: row.conversation_id,
        other_persona,
        unread_count: row.unread_count,
        latest_message,
        created_at: row.conversation_created_at,
        updated_at: row.conversation_updated_at,
    })
}

fn message_from_row(row: MessageRow) -> Result<InboxMessage, InboxError> {
    let content = match row.message_type.as_str() {
        "user" => InboxMessageContent::User {
            sender: persona_from_options(
                row.sender_id,
                row.sender_handle,
                row.sender_display_name,
                row.sender_bio,
                row.sender_status_message,
                row.sender_created_at,
                row.sender_updated_at,
            )?,
            body: row.user_body.ok_or(InboxError::Internal)?,
        },
        "system" => {
            let actor = persona_from_options(
                row.actor_id,
                row.actor_handle,
                row.actor_display_name,
                row.actor_bio,
                row.actor_status_message,
                row.actor_created_at,
                row.actor_updated_at,
            )?;
            let system = match row.system_type.as_deref() {
                Some("connection_accepted")
                    if row.system_game_challenge_id.is_none()
                        && row.system_game_session_id.is_none() =>
                {
                    SystemMessage::ConnectionAccepted { actor }
                }
                Some("game_challenge_created") if row.system_game_session_id.is_none() => {
                    SystemMessage::GameChallengeCreated {
                        actor,
                        challenge_id: row.system_game_challenge_id.ok_or(InboxError::Internal)?,
                    }
                }
                Some("game_challenge_accepted") => SystemMessage::GameChallengeAccepted {
                    actor,
                    challenge_id: row.system_game_challenge_id.ok_or(InboxError::Internal)?,
                    game_session_id: row.system_game_session_id.ok_or(InboxError::Internal)?,
                },
                Some("game_challenge_declined") if row.system_game_session_id.is_none() => {
                    SystemMessage::GameChallengeDeclined {
                        actor,
                        challenge_id: row.system_game_challenge_id.ok_or(InboxError::Internal)?,
                    }
                }
                Some("game_challenge_cancelled") if row.system_game_session_id.is_none() => {
                    SystemMessage::GameChallengeCancelled {
                        actor,
                        challenge_id: row.system_game_challenge_id.ok_or(InboxError::Internal)?,
                    }
                }
                _ => return Err(InboxError::Internal),
            };
            InboxMessageContent::System(system)
        }
        _ => return Err(InboxError::Internal),
    };
    Ok(InboxMessage {
        id: row.message_id,
        sequence: row.message_sequence,
        content,
        created_at: row.message_created_at,
    })
}

#[allow(clippy::too_many_arguments)]
fn persona_from_options(
    id: Option<Uuid>,
    handle: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    status_message: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
) -> Result<Persona, InboxError> {
    Ok(Persona {
        id: id.ok_or(InboxError::Internal)?,
        handle: handle.ok_or(InboxError::Internal)?,
        display_name: display_name.ok_or(InboxError::Internal)?,
        bio: bio.ok_or(InboxError::Internal)?,
        status_message: status_message.ok_or(InboxError::Internal)?,
        created_at: created_at.ok_or(InboxError::Internal)?,
        updated_at: updated_at.ok_or(InboxError::Internal)?,
    })
}

fn map_connection_send_error(error: ConnectionError) -> InboxError {
    match error {
        ConnectionError::PersonaNotFound => InboxError::PersonaNotFound,
        ConnectionError::Internal => InboxError::Internal,
        _ => InboxError::ConversationUnavailable,
    }
}

fn map_database_error(database_error: sqlx::Error, operation: &'static str) -> InboxError {
    error!(error = %database_error, operation, "inbox database operation failed");
    InboxError::Internal
}

#[cfg(test)]
mod tests {
    use super::{
        InboxError, validate_conversation_limit, validate_history_page, validate_message_body,
    };

    #[test]
    fn message_body_is_trimmed_bounded_and_control_safe() {
        assert_eq!(
            validate_message_body("  ready\nnext turn  "),
            Ok("ready\nnext turn".to_owned())
        );
        assert_eq!(
            validate_message_body(" \t "),
            Err(InboxError::InvalidMessageBody)
        );
        assert_eq!(
            validate_message_body("bad\rbody"),
            Err(InboxError::InvalidMessageBody)
        );
        assert!(validate_message_body(&"界".repeat(4_000)).is_ok());
        assert_eq!(
            validate_message_body(&"界".repeat(4_001)),
            Err(InboxError::InvalidMessageBody)
        );
    }

    #[test]
    fn pagination_is_positive_and_bounded() {
        assert_eq!(validate_conversation_limit(None), Ok(50));
        assert_eq!(validate_conversation_limit(Some(100)), Ok(100));
        assert_eq!(
            validate_conversation_limit(Some(0)),
            Err(InboxError::InvalidPagination)
        );
        assert_eq!(validate_history_page(None, None), Ok((None, 50)));
        assert_eq!(
            validate_history_page(Some(8), Some(100)),
            Ok((Some(8), 100))
        );
        assert_eq!(
            validate_history_page(Some(0), Some(10)),
            Err(InboxError::InvalidPagination)
        );
        assert_eq!(
            validate_history_page(None, Some(101)),
            Err(InboxError::InvalidPagination)
        );
    }
}
