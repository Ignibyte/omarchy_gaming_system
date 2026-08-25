//! Durable persona-local change feeds and WebSocket wake-up hints.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::extract::ws::{Message, WebSocket};
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction, postgres::PgListener};
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{error, warn};
use uuid::Uuid;

use crate::sessions::{self, AuthenticatedSession, SessionError};

pub(crate) const DEFAULT_SYNC_LIMIT: u16 = 50;
pub(crate) const MAX_SYNC_LIMIT: u16 = 100;
pub(crate) const MAX_RETAINED_EVENTS: i64 = 10_000;
const SYNC_NOTIFICATION_CHANNEL: &str = "persona_sync_changed";
const NOTIFICATION_BUFFER: usize = 256;
const MAX_SOCKETS_PER_PERSONA: usize = 5;
const MAX_SOCKETS_PER_ACCOUNT: usize = 20;
const MAX_SOCKETS_PER_PROCESS: usize = 256;
#[cfg(not(test))]
const SESSION_REVALIDATION_INTERVAL: Duration = Duration::from_secs(30);
#[cfg(test)]
const SESSION_REVALIDATION_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncEventKind {
    ConnectionRequests,
    Connections,
    Blocks,
    Conversation(Uuid),
    GameSession(Uuid),
}

impl SyncEventKind {
    fn database_parts(self) -> (&'static str, Option<Uuid>, Option<Uuid>) {
        match self {
            Self::ConnectionRequests => ("connection_requests_changed", None, None),
            Self::Connections => ("connections_changed", None, None),
            Self::Blocks => ("blocks_changed", None, None),
            Self::Conversation(conversation_id) => {
                ("conversation_changed", Some(conversation_id), None)
            }
            Self::GameSession(game_session_id) => {
                ("game_session_changed", None, Some(game_session_id))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SyncEvent {
    pub cursor: i64,
    pub kind: SyncEventKind,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SyncPage {
    pub events: Vec<SyncEvent>,
    pub next_cursor: i64,
    pub has_more: bool,
    pub reset_required: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SyncError {
    Unauthorized,
    PersonaNotFound,
    InvalidCursor,
    InvalidPagination,
    SocketLimitReached,
    Internal,
}

#[derive(Clone)]
pub(crate) struct SyncHub {
    sender: broadcast::Sender<Uuid>,
    counts: Arc<Mutex<SocketCounts>>,
}

#[derive(Default)]
struct SocketCounts {
    total: usize,
    per_account: HashMap<Uuid, usize>,
    per_persona: HashMap<Uuid, usize>,
}

pub(crate) struct SocketPermit {
    account_id: Uuid,
    persona_id: Uuid,
    counts: Arc<Mutex<SocketCounts>>,
}

impl SyncHub {
    pub(crate) fn new() -> Self {
        let (sender, _) = broadcast::channel(NOTIFICATION_BUFFER);
        Self {
            sender,
            counts: Arc::new(Mutex::new(SocketCounts::default())),
        }
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<Uuid> {
        self.sender.subscribe()
    }

    pub(crate) fn publish(&self, persona_id: Uuid) {
        let _ = self.sender.send(persona_id);
    }

    pub(crate) fn acquire(
        &self,
        account_id: Uuid,
        persona_id: Uuid,
    ) -> Result<SocketPermit, SyncError> {
        let mut counts = self
            .counts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let account_count = counts.per_account.get(&account_id).copied().unwrap_or(0);
        let persona_count = counts.per_persona.get(&persona_id).copied().unwrap_or(0);
        if counts.total >= MAX_SOCKETS_PER_PROCESS
            || account_count >= MAX_SOCKETS_PER_ACCOUNT
            || persona_count >= MAX_SOCKETS_PER_PERSONA
        {
            return Err(SyncError::SocketLimitReached);
        }
        counts.total += 1;
        *counts.per_account.entry(account_id).or_default() += 1;
        *counts.per_persona.entry(persona_id).or_default() += 1;
        drop(counts);
        Ok(SocketPermit {
            account_id,
            persona_id,
            counts: Arc::clone(&self.counts),
        })
    }
}

impl Drop for SocketPermit {
    fn drop(&mut self) {
        let mut counts = self
            .counts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        counts.total = counts.total.saturating_sub(1);
        if let Some(count) = counts.per_account.get_mut(&self.account_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                counts.per_account.remove(&self.account_id);
            }
        }
        if let Some(count) = counts.per_persona.get_mut(&self.persona_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                counts.per_persona.remove(&self.persona_id);
            }
        }
    }
}

/// Append, prune, and announce a persona event inside its owning mutation transaction.
pub(crate) async fn append_event(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: Uuid,
    kind: SyncEventKind,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO persona_sync_state (persona_id)
        VALUES ($1)
        ON CONFLICT (persona_id) DO NOTHING
        "#,
    )
    .bind(persona_id)
    .execute(&mut **transaction)
    .await?;

    let next_cursor = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE persona_sync_state
        SET last_event_sequence = last_event_sequence + 1
        WHERE persona_id = $1
        RETURNING last_event_sequence
        "#,
    )
    .bind(persona_id)
    .fetch_one(&mut **transaction)
    .await?;
    let (event_type, conversation_id, game_session_id) = kind.database_parts();

    sqlx::query(
        r#"
        INSERT INTO persona_sync_events (
            persona_id,
            event_sequence,
            event_type,
            conversation_id,
            game_session_id
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(persona_id)
    .bind(next_cursor)
    .bind(event_type)
    .bind(conversation_id)
    .bind(game_session_id)
    .execute(&mut **transaction)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM persona_sync_events
        WHERE persona_id = $1 AND event_sequence <= $2
        "#,
    )
    .bind(persona_id)
    .bind(next_cursor - MAX_RETAINED_EVENTS)
    .execute(&mut **transaction)
    .await?;

    sqlx::query("SELECT pg_notify($1, $2)")
        .bind(SYNC_NOTIFICATION_CHANNEL)
        .bind(persona_id.to_string())
        .execute(&mut **transaction)
        .await?;

    Ok(next_cursor)
}

pub async fn list_events(
    pool: &PgPool,
    token: &str,
    persona_id: &str,
    after: Option<i64>,
    limit: Option<u16>,
) -> Result<SyncPage, SyncError> {
    let (persona_id, _) = authenticate_owned_persona(pool, token, persona_id).await?;
    let limit = validate_limit(limit)?;
    let current_cursor = current_cursor(pool, persona_id).await?;

    let Some(after) = after else {
        return Ok(SyncPage {
            events: Vec::new(),
            next_cursor: current_cursor,
            has_more: false,
            reset_required: false,
        });
    };
    if after < 0 || after > current_cursor {
        return Err(SyncError::InvalidCursor);
    }

    let earliest = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT min(event_sequence) FROM persona_sync_events WHERE persona_id = $1",
    )
    .bind(persona_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "sync retention lookup"))?;
    if earliest.is_some_and(|earliest| after < earliest.saturating_sub(1)) {
        return Ok(SyncPage {
            events: Vec::new(),
            next_cursor: current_cursor,
            has_more: false,
            reset_required: true,
        });
    }

    let mut rows = sqlx::query_as::<_, (i64, String, Option<Uuid>, Option<Uuid>, String)>(
        r#"
        SELECT
            event_sequence,
            event_type,
            conversation_id,
            game_session_id,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        FROM persona_sync_events
        WHERE persona_id = $1 AND event_sequence > $2
        ORDER BY event_sequence
        LIMIT $3
        "#,
    )
    .bind(persona_id)
    .bind(after)
    .bind(i64::from(limit) + 1)
    .fetch_all(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "sync event listing"))?;

    if incremental_page_has_gap(after, current_cursor, rows.first().map(|row| row.0)) {
        return Ok(SyncPage {
            events: Vec::new(),
            next_cursor: current_cursor,
            has_more: false,
            reset_required: true,
        });
    }

    let has_more = rows.len() > usize::from(limit);
    rows.truncate(usize::from(limit));
    let next_cursor = rows.last().map_or(after, |row| row.0);
    let events = rows
        .into_iter()
        .map(event_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SyncPage {
        events,
        next_cursor,
        has_more,
        reset_required: false,
    })
}

pub(crate) async fn prepare_socket(
    pool: &PgPool,
    token: &str,
    persona_id: &str,
    hub: &SyncHub,
) -> Result<PreparedSocket, SyncError> {
    let (persona_id, authenticated) = authenticate_owned_persona(pool, token, persona_id).await?;
    let permit = hub.acquire(authenticated.account_id, persona_id)?;
    let receiver = hub.subscribe();
    let cursor = current_cursor(pool, persona_id).await?;
    Ok(PreparedSocket {
        pool: pool.clone(),
        account_id: authenticated.account_id,
        session_id: authenticated.session_id,
        persona_id,
        cursor,
        receiver,
        permit,
    })
}

pub(crate) struct PreparedSocket {
    pool: PgPool,
    account_id: Uuid,
    session_id: Uuid,
    persona_id: Uuid,
    cursor: i64,
    receiver: broadcast::Receiver<Uuid>,
    permit: SocketPermit,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SocketMessage {
    Ready { cursor: i64 },
    Changed,
    ResyncRequired,
}

pub(crate) async fn serve_socket(mut socket: WebSocket, prepared: PreparedSocket) {
    let PreparedSocket {
        pool,
        account_id,
        session_id,
        persona_id,
        cursor,
        mut receiver,
        permit,
    } = prepared;
    let _permit = permit;
    if !socket_session_authorized(&pool, account_id, session_id).await {
        let _ = socket.send(Message::Close(None)).await;
        return;
    }
    if send_socket_message(&mut socket, SocketMessage::Ready { cursor })
        .await
        .is_err()
    {
        return;
    }
    let mut authorization_checks = tokio::time::interval_at(
        tokio::time::Instant::now() + SESSION_REVALIDATION_INTERVAL,
        SESSION_REVALIDATION_INTERVAL,
    );
    authorization_checks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            notification = receiver.recv() => {
                let message = match notification {
                    Ok(changed_persona_id) if changed_persona_id == persona_id => Some(SocketMessage::Changed),
                    Ok(_) => None,
                    Err(broadcast::error::RecvError::Lagged(_)) => Some(SocketMessage::ResyncRequired),
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                if let Some(message) = message {
                    if !socket_session_authorized(&pool, account_id, session_id).await {
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    if send_socket_message(&mut socket, message).await.is_err() {
                        break;
                    }
                }
            }
            _ = authorization_checks.tick() => {
                if !socket_session_authorized(&pool, account_id, session_id).await {
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(Message::Text(_) | Message::Binary(_))) => {
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Some(Ok(Message::Pong(_))) => {}
                }
            }
        }
    }
}

async fn socket_session_authorized(pool: &PgPool, account_id: Uuid, session_id: Uuid) -> bool {
    matches!(
        sessions::remains_authorized(pool, account_id, session_id).await,
        Ok(true)
    )
}

async fn send_socket_message(
    socket: &mut WebSocket,
    message: SocketMessage,
) -> Result<(), axum::Error> {
    let encoded = serde_json::to_string(&message).map_err(axum::Error::new)?;
    socket.send(Message::Text(encoded.into())).await
}

pub(crate) async fn start_postgres_listener(
    pool: &PgPool,
    hub: SyncHub,
) -> Result<JoinHandle<()>, sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(SYNC_NOTIFICATION_CHANNEL).await?;
    Ok(tokio::spawn(async move {
        loop {
            match listener.recv().await {
                Ok(notification) => match Uuid::try_parse(notification.payload()) {
                    Ok(persona_id) => hub.publish(persona_id),
                    Err(parse_error) => {
                        warn!(?parse_error, "ignored invalid persona sync notification")
                    }
                },
                Err(database_error) => {
                    error!(?database_error, "persona sync listener receive failed");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }))
}

async fn authenticate_owned_persona(
    pool: &PgPool,
    token: &str,
    persona_id: &str,
) -> Result<(Uuid, AuthenticatedSession), SyncError> {
    let authenticated = sessions::authenticate(pool, token)
        .await
        .map_err(|session_error| match session_error {
            SessionError::Unauthorized => SyncError::Unauthorized,
            _ => SyncError::Internal,
        })?;
    let account_id = authenticated.account_id;
    let persona_id = Uuid::try_parse(persona_id).map_err(|_| SyncError::PersonaNotFound)?;
    let owned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM personas WHERE id = $1 AND account_id = $2)",
    )
    .bind(persona_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| map_database_error(database_error, "sync persona ownership"))?;
    if owned {
        Ok((persona_id, authenticated))
    } else {
        Err(SyncError::PersonaNotFound)
    }
}

async fn current_cursor(pool: &PgPool, persona_id: Uuid) -> Result<i64, SyncError> {
    sqlx::query_scalar::<_, i64>(
        "SELECT last_event_sequence FROM persona_sync_state WHERE persona_id = $1",
    )
    .bind(persona_id)
    .fetch_optional(pool)
    .await
    .map(|cursor| cursor.unwrap_or(0))
    .map_err(|database_error| map_database_error(database_error, "sync cursor lookup"))
}

fn validate_limit(limit: Option<u16>) -> Result<u16, SyncError> {
    let limit = limit.unwrap_or(DEFAULT_SYNC_LIMIT);
    if (1..=MAX_SYNC_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(SyncError::InvalidPagination)
    }
}

fn incremental_page_has_gap(after: i64, current_cursor: i64, first_cursor: Option<i64>) -> bool {
    match first_cursor {
        Some(first_cursor) => after.checked_add(1) != Some(first_cursor),
        None => after < current_cursor,
    }
}

fn event_from_row(
    row: (i64, String, Option<Uuid>, Option<Uuid>, String),
) -> Result<SyncEvent, SyncError> {
    let kind = match (row.1.as_str(), row.2, row.3) {
        ("connection_requests_changed", None, None) => SyncEventKind::ConnectionRequests,
        ("connections_changed", None, None) => SyncEventKind::Connections,
        ("blocks_changed", None, None) => SyncEventKind::Blocks,
        ("conversation_changed", Some(conversation_id), None) => {
            SyncEventKind::Conversation(conversation_id)
        }
        ("game_session_changed", None, Some(game_session_id)) => {
            SyncEventKind::GameSession(game_session_id)
        }
        _ => return Err(SyncError::Internal),
    };
    Ok(SyncEvent {
        cursor: row.0,
        kind,
        created_at: row.4,
    })
}

fn map_database_error(database_error: sqlx::Error, operation: &'static str) -> SyncError {
    error!(
        ?database_error,
        operation, "persona sync database operation failed"
    );
    SyncError::Internal
}

#[cfg(test)]
mod tests {
    use super::incremental_page_has_gap;

    #[test]
    fn incremental_page_detects_pruned_or_missing_events() {
        assert!(!incremental_page_has_gap(1, 10_000, Some(2)));
        assert!(incremental_page_has_gap(1, 10_000, Some(3)));
        assert!(incremental_page_has_gap(9_999, 10_000, None));
        assert!(!incremental_page_has_gap(10_000, 10_000, None));
    }
}
