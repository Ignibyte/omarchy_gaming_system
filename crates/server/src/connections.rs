//! Persona-scoped social connections and private directional blocks.

use sqlx::{PgPool, Postgres, Transaction};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    inboxes,
    personas::Persona,
    sessions::{self, SessionError},
    sync::{self, SyncEventKind},
};

pub(crate) const MAX_PENDING_REQUESTS_PER_DIRECTION: i64 = 100;

/// Whether an idempotent resource command created or reused its resource.
#[derive(Debug, PartialEq, Eq)]
pub enum ResourceOutcome<T> {
    Created(T),
    Existing(T),
}

/// One incoming or outgoing pending request.
#[derive(Debug, PartialEq, Eq)]
pub struct ConnectionRequest {
    pub persona: Persona,
    pub created_at: String,
}

/// Pending requests split by direction relative to the acting persona.
#[derive(Debug, PartialEq, Eq)]
pub struct ConnectionRequestInventory {
    pub incoming: Vec<ConnectionRequest>,
    pub outgoing: Vec<ConnectionRequest>,
}

/// One accepted mutual connection.
#[derive(Debug, PartialEq, Eq)]
pub struct Connection {
    pub persona: Persona,
    pub connected_at: String,
}

/// One private directional block owned by the acting persona.
#[derive(Debug, PartialEq, Eq)]
pub struct PersonaBlock {
    pub persona: Persona,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionError {
    Unauthorized,
    PersonaNotFound,
    ConnectionUnavailable,
    ConnectionRequestNotFound,
    ConnectionRequestPending,
    ConnectionAlreadyExists,
    Internal,
}

#[derive(sqlx::FromRow)]
struct SocialPersonaRow {
    persona_id: Uuid,
    handle: String,
    display_name: String,
    bio: String,
    status_message: String,
    persona_created_at: String,
    persona_updated_at: String,
    relationship_at: String,
}

struct LockedPair {
    low_id: Uuid,
    high_id: Uuid,
}

/// Create or retry one outgoing connection request.
pub async fn request_connection(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    target_id: &str,
) -> Result<ResourceOutcome<ConnectionRequest>, ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    let target_id = parse_target_id(target_id)?;
    let mut transaction = begin_transaction(pool, "connection request").await?;
    let pair = lock_pair(&mut transaction, account_id, actor_id, target_id).await?;

    if pair_is_blocked(&mut transaction, actor_id, target_id).await? {
        return Err(ConnectionError::ConnectionUnavailable);
    }

    let existing = sqlx::query_as::<_, (String, Uuid, String)>(
        r#"
        SELECT
            status,
            requester_id,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        FROM persona_connections
        WHERE persona_low_id = $1 AND persona_high_id = $2
        FOR UPDATE
        "#,
    )
    .bind(pair.low_id)
    .bind(pair.high_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "existing connection request lookup")
    })?;

    let (outcome_created, created_at) = match existing {
        Some(existing) => match (existing.0.as_str(), existing.1 == actor_id) {
            ("pending", true) => (false, existing.2),
            ("pending", false) => return Err(ConnectionError::ConnectionRequestPending),
            ("accepted", _) => return Err(ConnectionError::ConnectionAlreadyExists),
            _ => return Err(ConnectionError::Internal),
        },
        None => {
            let (outgoing_count, incoming_count) = sqlx::query_as::<_, (i64, i64)>(
                r#"
                SELECT
                    count(*) FILTER (WHERE requester_id = $1),
                    count(*) FILTER (WHERE addressee_id = $2)
                FROM persona_connections
                WHERE status = 'pending'
                  AND (requester_id = $1 OR addressee_id = $2)
                "#,
            )
            .bind(actor_id)
            .bind(target_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|database_error| {
                map_database_error(database_error, "pending connection request count")
            })?;

            if outgoing_count >= MAX_PENDING_REQUESTS_PER_DIRECTION
                || incoming_count >= MAX_PENDING_REQUESTS_PER_DIRECTION
            {
                return Err(ConnectionError::ConnectionUnavailable);
            }

            let created_at = sqlx::query_scalar::<_, String>(
                r#"
            INSERT INTO persona_connections (
                persona_low_id,
                persona_high_id,
                requester_id,
                addressee_id
            )
            VALUES ($1, $2, $3, $4)
            RETURNING to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            "#,
            )
            .bind(pair.low_id)
            .bind(pair.high_id)
            .bind(actor_id)
            .bind(target_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|database_error| {
                map_database_error(database_error, "connection request insertion")
            })?;
            (true, created_at)
        }
    };

    let persona = load_persona(&mut transaction, target_id).await?;
    if outcome_created {
        append_sync_event(
            &mut transaction,
            actor_id,
            SyncEventKind::ConnectionRequests,
        )
        .await?;
        append_sync_event(
            &mut transaction,
            target_id,
            SyncEventKind::ConnectionRequests,
        )
        .await?;
    }
    commit_transaction(transaction, "connection request").await?;
    let request = ConnectionRequest {
        persona,
        created_at,
    };

    if outcome_created {
        info!(%actor_id, %target_id, "connection request created");
        Ok(ResourceOutcome::Created(request))
    } else {
        Ok(ResourceOutcome::Existing(request))
    }
}

/// List pending requests visible to one owned persona.
pub async fn list_connection_requests(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
) -> Result<ConnectionRequestInventory, ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    ensure_actor_owned(pool, account_id, actor_id).await?;

    let incoming = sqlx::query_as::<_, SocialPersonaRow>(
        r#"
        SELECT
            persona.id AS persona_id,
            persona.handle,
            persona.display_name,
            persona.bio,
            persona.status_message,
            to_char(persona.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_created_at,
            to_char(persona.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_updated_at,
            to_char(connection.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS relationship_at
        FROM persona_connections AS connection
        JOIN personas AS persona ON persona.id = connection.requester_id
        WHERE connection.status = 'pending' AND connection.addressee_id = $1
        ORDER BY connection.created_at, persona.id
        "#,
    )
    .bind(actor_id)
    .fetch_all(pool)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "incoming connection request listing")
    })?;

    let outgoing = sqlx::query_as::<_, SocialPersonaRow>(
        r#"
        SELECT
            persona.id AS persona_id,
            persona.handle,
            persona.display_name,
            persona.bio,
            persona.status_message,
            to_char(persona.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_created_at,
            to_char(persona.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_updated_at,
            to_char(connection.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS relationship_at
        FROM persona_connections AS connection
        JOIN personas AS persona ON persona.id = connection.addressee_id
        WHERE connection.status = 'pending' AND connection.requester_id = $1
        ORDER BY connection.created_at, persona.id
        "#,
    )
    .bind(actor_id)
    .fetch_all(pool)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "outgoing connection request listing")
    })?;

    Ok(ConnectionRequestInventory {
        incoming: incoming.into_iter().map(request_from_row).collect(),
        outgoing: outgoing.into_iter().map(request_from_row).collect(),
    })
}

/// Accept an incoming request or retry an already accepted connection.
pub async fn accept_connection(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    requester_id: &str,
) -> Result<Connection, ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    let requester_id = parse_target_id(requester_id)?;
    let mut transaction = begin_transaction(pool, "connection acceptance").await?;
    let pair = lock_pair(&mut transaction, account_id, actor_id, requester_id).await?;

    if pair_is_blocked(&mut transaction, actor_id, requester_id).await? {
        return Err(ConnectionError::ConnectionUnavailable);
    }

    let existing = sqlx::query_as::<_, (String, Uuid, Uuid, Option<String>)>(
        r#"
        SELECT
            status,
            requester_id,
            addressee_id,
            CASE
                WHEN accepted_at IS NULL THEN NULL
                ELSE to_char(accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            END
        FROM persona_connections
        WHERE persona_low_id = $1 AND persona_high_id = $2
        FOR UPDATE
        "#,
    )
    .bind(pair.low_id)
    .bind(pair.high_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "connection acceptance lookup"))?
    .ok_or(ConnectionError::ConnectionRequestNotFound)?;

    let (connected_at, transitioned) = match existing.0.as_str() {
        "accepted" => (existing.3.ok_or(ConnectionError::Internal)?, false),
        "pending" if existing.1 == requester_id && existing.2 == actor_id => {
            let connected_at = sqlx::query_scalar::<_, String>(
                r#"
                UPDATE persona_connections
                SET status = 'accepted', accepted_at = now(), updated_at = now()
                WHERE persona_low_id = $1 AND persona_high_id = $2
                RETURNING to_char(accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
                "#,
            )
            .bind(pair.low_id)
            .bind(pair.high_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|database_error| {
                map_database_error(database_error, "connection acceptance update")
            })?;
            (connected_at, true)
        }
        "pending" => return Err(ConnectionError::ConnectionRequestNotFound),
        _ => return Err(ConnectionError::Internal),
    };

    if transitioned {
        let conversation_id = inboxes::record_connection_accepted(
            &mut transaction,
            pair.low_id,
            pair.high_id,
            actor_id,
        )
        .await
        .map_err(|inbox_error| {
            error!(?inbox_error, "accepted connection inbox creation failed");
            ConnectionError::Internal
        })?;
        for persona_id in [actor_id, requester_id] {
            append_sync_event(
                &mut transaction,
                persona_id,
                SyncEventKind::ConnectionRequests,
            )
            .await?;
            append_sync_event(&mut transaction, persona_id, SyncEventKind::Connections).await?;
            append_sync_event(
                &mut transaction,
                persona_id,
                SyncEventKind::Conversation(conversation_id),
            )
            .await?;
        }
    }

    let persona = load_persona(&mut transaction, requester_id).await?;
    commit_transaction(transaction, "connection acceptance").await?;
    info!(%actor_id, %requester_id, "connection accepted");
    Ok(Connection {
        persona,
        connected_at,
    })
}

/// List accepted mutual connections for one owned persona.
pub async fn list_connections(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
) -> Result<Vec<Connection>, ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    ensure_actor_owned(pool, account_id, actor_id).await?;

    let rows = sqlx::query_as::<_, SocialPersonaRow>(
        r#"
        SELECT
            persona.id AS persona_id,
            persona.handle,
            persona.display_name,
            persona.bio,
            persona.status_message,
            to_char(persona.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_created_at,
            to_char(persona.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_updated_at,
            to_char(connection.accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS relationship_at
        FROM persona_connections AS connection
        JOIN personas AS persona
          ON persona.id = CASE
              WHEN connection.persona_low_id = $1 THEN connection.persona_high_id
              ELSE connection.persona_low_id
          END
        WHERE connection.status = 'accepted'
          AND (connection.persona_low_id = $1 OR connection.persona_high_id = $1)
        ORDER BY connection.accepted_at, persona.id
        "#,
    )
    .bind(actor_id)
    .fetch_all(pool)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "accepted connection listing")
    })?;

    Ok(rows.into_iter().map(connection_from_row).collect())
}

/// Remove any pending or accepted state for a pair. Missing target state is a no-op.
pub async fn remove_connection(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    target_id: &str,
) -> Result<(), ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    let Ok(target_id) = Uuid::try_parse(target_id) else {
        ensure_actor_owned(pool, account_id, actor_id).await?;
        return Ok(());
    };
    let mut transaction = begin_transaction(pool, "connection removal").await?;
    let pair = match lock_pair(&mut transaction, account_id, actor_id, target_id).await {
        Ok(pair) => pair,
        Err(ConnectionError::ConnectionUnavailable) => return Ok(()),
        Err(error) => return Err(error),
    };

    let removed = sqlx::query(
        "DELETE FROM persona_connections WHERE persona_low_id = $1 AND persona_high_id = $2",
    )
    .bind(pair.low_id)
    .bind(pair.high_id)
    .execute(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "connection removal"))?;
    if removed.rows_affected() > 0 {
        for persona_id in [actor_id, target_id] {
            append_sync_event(
                &mut transaction,
                persona_id,
                SyncEventKind::ConnectionRequests,
            )
            .await?;
            append_sync_event(&mut transaction, persona_id, SyncEventKind::Connections).await?;
        }
    }
    commit_transaction(transaction, "connection removal").await?;
    info!(%actor_id, %target_id, "connection state removed");
    Ok(())
}

/// Create or retry a private directional block and remove all pair relationship state.
pub async fn block_persona(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    target_id: &str,
) -> Result<ResourceOutcome<PersonaBlock>, ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    let target_id = parse_target_id(target_id)?;
    let mut transaction = begin_transaction(pool, "persona block").await?;
    let pair = lock_pair(&mut transaction, account_id, actor_id, target_id).await?;

    let created_at = sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO persona_blocks (blocker_id, blocked_id)
        VALUES ($1, $2)
        ON CONFLICT (blocker_id, blocked_id) DO NOTHING
        RETURNING to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        "#,
    )
    .bind(actor_id)
    .bind(target_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "persona block insertion"))?;

    let (outcome_created, created_at) = if let Some(created_at) = created_at {
        (true, created_at)
    } else {
        let created_at = sqlx::query_scalar::<_, String>(
            r#"
            SELECT to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            FROM persona_blocks
            WHERE blocker_id = $1 AND blocked_id = $2
            "#,
        )
        .bind(actor_id)
        .bind(target_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|database_error| {
            map_database_error(database_error, "existing persona block lookup")
        })?;
        (false, created_at)
    };

    let relationship_removed = sqlx::query(
        "DELETE FROM persona_connections WHERE persona_low_id = $1 AND persona_high_id = $2",
    )
    .bind(pair.low_id)
    .bind(pair.high_id)
    .execute(&mut *transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "blocked relationship removal"))?;

    let persona = load_persona(&mut transaction, target_id).await?;
    if outcome_created {
        append_sync_event(&mut transaction, actor_id, SyncEventKind::Blocks).await?;
    }
    if relationship_removed.rows_affected() > 0 {
        for persona_id in [actor_id, target_id] {
            append_sync_event(
                &mut transaction,
                persona_id,
                SyncEventKind::ConnectionRequests,
            )
            .await?;
            append_sync_event(&mut transaction, persona_id, SyncEventKind::Connections).await?;
        }
    }
    commit_transaction(transaction, "persona block").await?;
    let block = PersonaBlock {
        persona,
        created_at,
    };

    if outcome_created {
        info!(%actor_id, %target_id, "persona blocked");
        Ok(ResourceOutcome::Created(block))
    } else {
        Ok(ResourceOutcome::Existing(block))
    }
}

/// List private directional blocks owned by one persona.
pub async fn list_blocks(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
) -> Result<Vec<PersonaBlock>, ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    ensure_actor_owned(pool, account_id, actor_id).await?;

    let rows = sqlx::query_as::<_, SocialPersonaRow>(
        r#"
        SELECT
            persona.id AS persona_id,
            persona.handle,
            persona.display_name,
            persona.bio,
            persona.status_message,
            to_char(persona.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_created_at,
            to_char(persona.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS persona_updated_at,
            to_char(block.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS relationship_at
        FROM persona_blocks AS block
        JOIN personas AS persona ON persona.id = block.blocked_id
        WHERE block.blocker_id = $1
        ORDER BY block.created_at, persona.id
        "#,
    )
    .bind(actor_id)
    .fetch_all(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "persona block listing"))?;

    Ok(rows.into_iter().map(block_from_row).collect())
}

/// Remove a private directional block. Missing target state is a no-op.
pub async fn unblock_persona(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
    target_id: &str,
) -> Result<(), ConnectionError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let actor_id = parse_actor_id(actor_id)?;
    let Ok(target_id) = Uuid::try_parse(target_id) else {
        ensure_actor_owned(pool, account_id, actor_id).await?;
        return Ok(());
    };
    let mut transaction = begin_transaction(pool, "persona unblock").await?;
    match lock_pair(&mut transaction, account_id, actor_id, target_id).await {
        Ok(_) => {}
        Err(ConnectionError::ConnectionUnavailable) => return Ok(()),
        Err(error) => return Err(error),
    }

    let removed =
        sqlx::query("DELETE FROM persona_blocks WHERE blocker_id = $1 AND blocked_id = $2")
            .bind(actor_id)
            .bind(target_id)
            .execute(&mut *transaction)
            .await
            .map_err(|database_error| map_database_error(database_error, "persona unblock"))?;
    if removed.rows_affected() > 0 {
        append_sync_event(&mut transaction, actor_id, SyncEventKind::Blocks).await?;
    }
    commit_transaction(transaction, "persona unblock").await?;
    info!(%actor_id, %target_id, "persona unblocked");
    Ok(())
}

async fn authenticated_account_id(pool: &PgPool, token: &str) -> Result<Uuid, ConnectionError> {
    sessions::authenticate(pool, token)
        .await
        .map(|authenticated| authenticated.account_id)
        .map_err(|session_error| match session_error {
            SessionError::Unauthorized => ConnectionError::Unauthorized,
            _ => ConnectionError::Internal,
        })
}

fn parse_actor_id(persona_id: &str) -> Result<Uuid, ConnectionError> {
    Uuid::try_parse(persona_id).map_err(|_| ConnectionError::PersonaNotFound)
}

fn parse_target_id(persona_id: &str) -> Result<Uuid, ConnectionError> {
    Uuid::try_parse(persona_id).map_err(|_| ConnectionError::ConnectionUnavailable)
}

async fn begin_transaction<'a>(
    pool: &'a PgPool,
    operation: &'static str,
) -> Result<Transaction<'a, Postgres>, ConnectionError> {
    pool.begin()
        .await
        .map_err(|database_error| map_database_error(database_error, operation))
}

async fn commit_transaction(
    transaction: Transaction<'_, Postgres>,
    operation: &'static str,
) -> Result<(), ConnectionError> {
    transaction
        .commit()
        .await
        .map_err(|database_error| map_database_error(database_error, operation))
}

async fn append_sync_event(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: Uuid,
    kind: SyncEventKind,
) -> Result<(), ConnectionError> {
    sync::append_event(transaction, persona_id, kind)
        .await
        .map(|_| ())
        .map_err(|database_error| {
            error!(?database_error, %persona_id, "connection sync event append failed");
            ConnectionError::Internal
        })
}

async fn ensure_actor_owned(
    pool: &PgPool,
    account_id: Uuid,
    actor_id: Uuid,
) -> Result<(), ConnectionError> {
    let is_owned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM personas WHERE id = $1 AND account_id = $2)",
    )
    .bind(actor_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| {
        map_database_error(database_error, "acting persona ownership check")
    })?;

    if is_owned {
        Ok(())
    } else {
        Err(ConnectionError::PersonaNotFound)
    }
}

async fn lock_pair(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<LockedPair, ConnectionError> {
    if actor_id == target_id {
        let actor_account_id = lock_persona(transaction, actor_id).await?;
        if actor_account_id != Some(account_id) {
            return Err(ConnectionError::PersonaNotFound);
        }
        return Err(ConnectionError::ConnectionUnavailable);
    }

    let (low_id, high_id) = canonical_pair(actor_id, target_id);
    let low_account_id = lock_persona(transaction, low_id).await?;
    let high_account_id = lock_persona(transaction, high_id).await?;
    let actor_account_id = if actor_id == low_id {
        low_account_id
    } else {
        high_account_id
    };
    let target_account_id = if target_id == low_id {
        low_account_id
    } else {
        high_account_id
    };

    if actor_account_id != Some(account_id) {
        return Err(ConnectionError::PersonaNotFound);
    }
    if target_account_id.is_none() || target_account_id == Some(account_id) {
        return Err(ConnectionError::ConnectionUnavailable);
    }

    Ok(LockedPair { low_id, high_id })
}

/// Lock a cross-account persona pair in canonical order and prove actor ownership.
pub(crate) async fn lock_persona_pair(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<(), ConnectionError> {
    lock_pair(transaction, account_id, actor_id, target_id)
        .await
        .map(|_| ())
}

async fn lock_persona(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: Uuid,
) -> Result<Option<Uuid>, ConnectionError> {
    sqlx::query_scalar::<_, Uuid>("SELECT account_id FROM personas WHERE id = $1 FOR UPDATE")
        .bind(persona_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|database_error| map_database_error(database_error, "persona pair locking"))
}

async fn pair_is_blocked(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<bool, ConnectionError> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM persona_blocks
            WHERE (blocker_id = $1 AND blocked_id = $2)
               OR (blocker_id = $2 AND blocked_id = $1)
        )
        "#,
    )
    .bind(actor_id)
    .bind(target_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "persona block check"))
}

/// Lock and verify a currently accepted, unblocked pair for another social domain.
pub(crate) async fn lock_connected_pair(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<(), ConnectionError> {
    let pair = lock_pair(transaction, account_id, actor_id, target_id).await?;
    if pair_is_blocked(transaction, actor_id, target_id).await? {
        return Err(ConnectionError::ConnectionUnavailable);
    }
    let connected = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM persona_connections
            WHERE persona_low_id = $1
              AND persona_high_id = $2
              AND status = 'accepted'
        )
        "#,
    )
    .bind(pair.low_id)
    .bind(pair.high_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|database_error| map_database_error(database_error, "connected pair check"))?;
    if connected {
        Ok(())
    } else {
        Err(ConnectionError::ConnectionUnavailable)
    }
}

async fn load_persona(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: Uuid,
) -> Result<Persona, ConnectionError> {
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
    .map_err(|database_error| map_database_error(database_error, "social persona lookup"))?
    .ok_or(ConnectionError::ConnectionUnavailable)?;

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

fn canonical_pair(first_id: Uuid, second_id: Uuid) -> (Uuid, Uuid) {
    if first_id < second_id {
        (first_id, second_id)
    } else {
        (second_id, first_id)
    }
}

fn request_from_row(row: SocialPersonaRow) -> ConnectionRequest {
    ConnectionRequest {
        created_at: row.relationship_at.clone(),
        persona: persona_from_social_row(row),
    }
}

fn connection_from_row(row: SocialPersonaRow) -> Connection {
    Connection {
        connected_at: row.relationship_at.clone(),
        persona: persona_from_social_row(row),
    }
}

fn block_from_row(row: SocialPersonaRow) -> PersonaBlock {
    PersonaBlock {
        created_at: row.relationship_at.clone(),
        persona: persona_from_social_row(row),
    }
}

fn persona_from_social_row(row: SocialPersonaRow) -> Persona {
    Persona {
        id: row.persona_id,
        handle: row.handle,
        display_name: row.display_name,
        bio: row.bio,
        status_message: row.status_message,
        created_at: row.persona_created_at,
        updated_at: row.persona_updated_at,
    }
}

fn map_database_error(database_error: sqlx::Error, operation: &'static str) -> ConnectionError {
    error!(error = %database_error, operation, "connection database operation failed");
    ConnectionError::Internal
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::canonical_pair;

    #[test]
    fn canonical_pairs_are_order_independent() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);

        assert_eq!(canonical_pair(first, second), (first, second));
        assert_eq!(canonical_pair(second, first), (first, second));
    }
}
