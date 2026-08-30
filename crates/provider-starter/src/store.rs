use omarchygs_provider_sdk::{
    ProviderError, Result,
    protocol::{
        ProviderEvent, ProviderOperationDisposition, ProviderOperationKind,
        ProviderOperationRequest, ProviderOperationResponse, ProviderSessionStatus, sha256_hex,
    },
};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{GameIdentity, GameState, ProviderGame};

const MAX_STATE_BYTES: usize = 32 * 1024;
const MAX_RESPONSE_BYTES: usize = 65_536;

#[derive(Clone)]
pub(crate) struct StarterStore {
    pool: PgPool,
}

pub(crate) struct StoredResponse {
    pub(crate) body: Vec<u8>,
    pub(crate) replayed: bool,
}

#[derive(FromRow)]
struct SessionRow {
    pairwise_subject: String,
    revision: i64,
    status: String,
    game_state: Value,
}

#[derive(FromRow)]
struct GrantRow {
    provider_id: String,
    release_id: Uuid,
    platform_session_id: Uuid,
    idempotency_key: Uuid,
    request_sha256: String,
}

#[derive(FromRow)]
struct ReceiptRow {
    provider_id: String,
    release_id: Uuid,
    operation: String,
    expected_revision: i64,
    intent_sha256: String,
    response_body: Vec<u8>,
}

#[derive(Serialize)]
struct StableIntent<'a> {
    provider_id: &'a str,
    release_id: Uuid,
    game_key: &'a str,
    rules_version: u32,
    cartridge_digest: &'a str,
    platform_session_id: Uuid,
    subject: &'a str,
    idempotency_key: Uuid,
    expected_revision: u64,
    operation: ProviderOperationKind,
    payload: &'a Value,
}

impl StarterStore {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) async fn initialize(&self, identity: &GameIdentity, release_id: Uuid) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        sqlx::query(
            r#"
            INSERT INTO provider_starter_identity (
                singleton, provider_id, release_id, game_key, rules_version,
                cartridge_digest
            )
            VALUES (TRUE, $1, $2, $3, $4, $5)
            ON CONFLICT (singleton) DO NOTHING
            "#,
        )
        .bind(&identity.provider_id)
        .bind(release_id)
        .bind(&identity.game_key)
        .bind(i64::from(identity.rules_version))
        .bind(&identity.cartridge_digest)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        lock_identity(&mut transaction, identity, release_id).await?;
        transaction.commit().await.map_err(database_error)
    }

    pub(crate) fn stable_intent_digest(request: &ProviderOperationRequest) -> Result<String> {
        let bytes = serde_json::to_vec(&StableIntent {
            provider_id: &request.provider_id,
            release_id: request.release_id,
            game_key: &request.game_key,
            rules_version: request.rules_version,
            cartridge_digest: &request.cartridge_digest,
            platform_session_id: request.platform_session_id,
            subject: &request.subject,
            idempotency_key: request.idempotency_key,
            expected_revision: request.expected_revision,
            operation: request.operation,
            payload: &request.payload,
        })
        .map_err(|_| ProviderError::Internal)?;
        if bytes.is_empty() || bytes.len() > MAX_RESPONSE_BYTES {
            return Err(ProviderError::InvalidInput);
        }
        Ok(sha256_hex(&bytes))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn apply<G: ProviderGame>(
        &self,
        game: &G,
        release_id: Uuid,
        request: &ProviderOperationRequest,
        token_id: Uuid,
        request_digest: &str,
        intent_digest: &str,
    ) -> Result<StoredResponse> {
        let identity = game.identity();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        lock_identity(&mut transaction, identity, release_id).await?;
        admit_grant(
            &mut transaction,
            identity,
            release_id,
            request,
            token_id,
            request_digest,
        )
        .await?;

        if let Some(receipt) = load_receipt(&mut transaction, request).await? {
            if receipt.provider_id != identity.provider_id
                || receipt.release_id != release_id
                || receipt.operation != operation_name(request.operation)
                || receipt.expected_revision != revision_i64(request.expected_revision)?
                || receipt.intent_sha256 != intent_digest
            {
                return Err(ProviderError::Conflict);
            }
            transaction.commit().await.map_err(database_error)?;
            return Ok(StoredResponse {
                body: receipt.response_body,
                replayed: true,
            });
        }

        let (session, disposition, emit_event) = match request.operation {
            ProviderOperationKind::Launch => {
                if request.expected_revision != 0 {
                    return Err(ProviderError::InvalidInput);
                }
                if lock_session(&mut transaction, request.platform_session_id)
                    .await?
                    .is_some()
                {
                    return Err(ProviderError::Conflict);
                }
                let initial = game.launch(&request.payload)?;
                validate_game_state(&initial)?;
                let session = SessionRow {
                    pairwise_subject: request.subject.clone(),
                    revision: 0,
                    status: status_name(initial.status).to_owned(),
                    game_state: initial.state,
                };
                insert_session(
                    &mut transaction,
                    identity,
                    release_id,
                    request.platform_session_id,
                    &session,
                )
                .await?;
                (session, ProviderOperationDisposition::Applied, false)
            }
            ProviderOperationKind::Command => {
                let current = lock_session(&mut transaction, request.platform_session_id)
                    .await?
                    .ok_or(ProviderError::NotFound)?;
                verify_subject(&current, request)?;
                if current.revision != revision_i64(request.expected_revision)? {
                    (
                        current,
                        ProviderOperationDisposition::RevisionConflict,
                        false,
                    )
                } else {
                    if current.status == "completed" {
                        return Err(ProviderError::Conflict);
                    }
                    let transition = game.command(&row_state(&current)?, &request.payload)?;
                    let next = GameState {
                        status: transition.status,
                        state: transition.state,
                    };
                    validate_game_state(&next)?;
                    let revision = current
                        .revision
                        .checked_add(1)
                        .ok_or(ProviderError::Internal)?;
                    let session = SessionRow {
                        pairwise_subject: current.pairwise_subject,
                        revision,
                        status: status_name(next.status).to_owned(),
                        game_state: next.state,
                    };
                    update_session(&mut transaction, request.platform_session_id, &session).await?;
                    (session, ProviderOperationDisposition::Applied, true)
                }
            }
            ProviderOperationKind::Reconcile => {
                if request
                    .payload
                    .as_object()
                    .is_none_or(|value| !value.is_empty())
                {
                    return Err(ProviderError::InvalidInput);
                }
                let current = lock_session(&mut transaction, request.platform_session_id)
                    .await?
                    .ok_or(ProviderError::NotFound)?;
                verify_subject(&current, request)?;
                let disposition = if current.revision < revision_i64(request.expected_revision)? {
                    ProviderOperationDisposition::RevisionConflict
                } else {
                    ProviderOperationDisposition::Applied
                };
                (current, disposition, false)
            }
        };

        let state = row_state(&session)?;
        let response = ProviderOperationResponse::new(
            identity.provider_id.clone(),
            release_id,
            identity.game_key.clone(),
            identity.rules_version,
            identity.cartridge_digest.clone(),
            request.platform_session_id,
            request.subject.clone(),
            Uuid::new_v4(),
            request.idempotency_key,
            u64::try_from(session.revision).map_err(|_| ProviderError::Internal)?,
            disposition,
            state.status,
            request.compatibility.clone(),
            json!({"view": game.view(&state)?}),
        );
        response.validate_for(request)?;
        let response_body = serde_json::to_vec(&response).map_err(|_| ProviderError::Internal)?;
        if response_body.is_empty() || response_body.len() > MAX_RESPONSE_BYTES {
            return Err(ProviderError::Internal);
        }

        if emit_event
            && disposition == ProviderOperationDisposition::Applied
            && let Some(event) = game.event(&state)?
        {
            let event = ProviderEvent::new(
                identity.provider_id.clone(),
                release_id,
                identity.game_key.clone(),
                identity.rules_version,
                identity.cartridge_digest.clone(),
                request.platform_session_id,
                request.subject.clone(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                u64::try_from(session.revision).map_err(|_| ProviderError::Internal)?,
                event.kind,
                request.compatibility.clone(),
                event.payload,
            );
            event.validate()?;
            let body = serde_json::to_vec(&event).map_err(|_| ProviderError::Internal)?;
            sqlx::query(
                r#"
                    INSERT INTO provider_starter_event_outbox (
                        event_id, provider_id, release_id, platform_session_id,
                        message_id, provider_revision, body
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
            )
            .bind(event.event_id)
            .bind(&identity.provider_id)
            .bind(release_id)
            .bind(request.platform_session_id)
            .bind(event.message_id)
            .bind(session.revision)
            .bind(body)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        sqlx::query(
            r#"
            INSERT INTO provider_starter_operation_receipts (
                platform_session_id, idempotency_key, provider_id, release_id,
                operation, expected_revision, intent_sha256, response_body,
                provider_revision
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(request.platform_session_id)
        .bind(request.idempotency_key)
        .bind(&identity.provider_id)
        .bind(release_id)
        .bind(operation_name(request.operation))
        .bind(revision_i64(request.expected_revision)?)
        .bind(intent_digest)
        .bind(&response_body)
        .bind(session.revision)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        transaction.commit().await.map_err(database_error)?;
        Ok(StoredResponse {
            body: response_body,
            replayed: false,
        })
    }
}

async fn lock_identity(
    transaction: &mut Transaction<'_, Postgres>,
    identity: &GameIdentity,
    release_id: Uuid,
) -> Result<()> {
    let row: (String, Uuid, String, i64, String) = sqlx::query_as(
        r#"
        SELECT provider_id, release_id, game_key, rules_version, cartridge_digest
        FROM provider_starter_identity
        WHERE singleton = TRUE
        FOR UPDATE
        "#,
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(database_error)?;
    if row.0 != identity.provider_id
        || row.1 != release_id
        || row.2 != identity.game_key
        || row.3 != i64::from(identity.rules_version)
        || row.4 != identity.cartridge_digest
    {
        return Err(ProviderError::Denied);
    }
    Ok(())
}

async fn admit_grant(
    transaction: &mut Transaction<'_, Postgres>,
    identity: &GameIdentity,
    release_id: Uuid,
    request: &ProviderOperationRequest,
    token_id: Uuid,
    request_digest: &str,
) -> Result<()> {
    let existing = sqlx::query_as::<_, GrantRow>(
        r#"
        SELECT provider_id, release_id, platform_session_id, idempotency_key,
               request_sha256
        FROM provider_starter_consumed_grants
        WHERE token_id = $1
        FOR UPDATE
        "#,
    )
    .bind(token_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    if let Some(existing) = existing {
        if existing.provider_id != identity.provider_id
            || existing.release_id != release_id
            || existing.platform_session_id != request.platform_session_id
            || existing.idempotency_key != request.idempotency_key
            || existing.request_sha256 != request_digest
        {
            return Err(ProviderError::ProtocolRejected);
        }
        return Ok(());
    }
    sqlx::query(
        r#"
        INSERT INTO provider_starter_consumed_grants (
            token_id, provider_id, release_id, platform_session_id,
            idempotency_key, request_sha256
        ) VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(token_id)
    .bind(&identity.provider_id)
    .bind(release_id)
    .bind(request.platform_session_id)
    .bind(request.idempotency_key)
    .bind(request_digest)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn load_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    request: &ProviderOperationRequest,
) -> Result<Option<ReceiptRow>> {
    sqlx::query_as::<_, ReceiptRow>(
        r#"
        SELECT provider_id, release_id, operation, expected_revision,
               intent_sha256, response_body
        FROM provider_starter_operation_receipts
        WHERE platform_session_id = $1 AND idempotency_key = $2
        FOR UPDATE
        "#,
    )
    .bind(request.platform_session_id)
    .bind(request.idempotency_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)
}

async fn lock_session(
    transaction: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
) -> Result<Option<SessionRow>> {
    sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT pairwise_subject, revision, status, game_state
        FROM provider_starter_sessions
        WHERE platform_session_id = $1
        FOR UPDATE
        "#,
    )
    .bind(session_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)
}

async fn insert_session(
    transaction: &mut Transaction<'_, Postgres>,
    identity: &GameIdentity,
    release_id: Uuid,
    session_id: Uuid,
    session: &SessionRow,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO provider_starter_sessions (
            platform_session_id, provider_id, release_id, game_key,
            rules_version, cartridge_digest, pairwise_subject, revision,
            status, game_state, completed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                  CASE WHEN $9 = 'completed' THEN clock_timestamp() ELSE NULL END)
        "#,
    )
    .bind(session_id)
    .bind(&identity.provider_id)
    .bind(release_id)
    .bind(&identity.game_key)
    .bind(i64::from(identity.rules_version))
    .bind(&identity.cartridge_digest)
    .bind(&session.pairwise_subject)
    .bind(session.revision)
    .bind(&session.status)
    .bind(&session.game_state)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn update_session(
    transaction: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    session: &SessionRow,
) -> Result<()> {
    let changed = sqlx::query(
        r#"
        UPDATE provider_starter_sessions
        SET revision = $2,
            status = $3,
            game_state = $4,
            completed_at = CASE
                WHEN $3 = 'completed' THEN COALESCE(completed_at, clock_timestamp())
                ELSE NULL
            END,
            updated_at = clock_timestamp()
        WHERE platform_session_id = $1
        "#,
    )
    .bind(session_id)
    .bind(session.revision)
    .bind(&session.status)
    .bind(&session.game_state)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    if changed.rows_affected() != 1 {
        return Err(ProviderError::Internal);
    }
    Ok(())
}

fn verify_subject(session: &SessionRow, request: &ProviderOperationRequest) -> Result<()> {
    if session.pairwise_subject == request.subject {
        Ok(())
    } else {
        Err(ProviderError::ProtocolRejected)
    }
}

fn row_state(session: &SessionRow) -> Result<GameState> {
    let status = match session.status.as_str() {
        "active" => ProviderSessionStatus::Active,
        "completed" => ProviderSessionStatus::Completed,
        _ => return Err(ProviderError::Internal),
    };
    let state = GameState {
        status,
        state: session.game_state.clone(),
    };
    validate_game_state(&state)?;
    Ok(state)
}

fn validate_game_state(state: &GameState) -> Result<()> {
    if !state.state.is_object() {
        return Err(ProviderError::InvalidInput);
    }
    validate_value(&state.state, 0)?;
    let bytes = serde_json::to_vec(&state.state).map_err(|_| ProviderError::Internal)?;
    if bytes.len() > MAX_STATE_BYTES {
        return Err(ProviderError::InvalidInput);
    }
    Ok(())
}

fn validate_value(value: &Value, depth: usize) -> Result<()> {
    if depth > 16 {
        return Err(ProviderError::InvalidInput);
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
        Value::String(value) if value.len() <= 4_096 => Ok(()),
        Value::String(_) => Err(ProviderError::InvalidInput),
        Value::Array(values) if values.len() <= 256 => {
            for value in values {
                validate_value(value, depth + 1)?;
            }
            Ok(())
        }
        Value::Object(values) if values.len() <= 256 => {
            for (key, value) in values {
                if key.is_empty() || key.len() > 128 {
                    return Err(ProviderError::InvalidInput);
                }
                validate_value(value, depth + 1)?;
            }
            Ok(())
        }
        Value::Array(_) | Value::Object(_) => Err(ProviderError::InvalidInput),
    }
}

fn status_name(status: ProviderSessionStatus) -> &'static str {
    match status {
        ProviderSessionStatus::Active => "active",
        ProviderSessionStatus::Completed => "completed",
    }
}

fn operation_name(operation: ProviderOperationKind) -> &'static str {
    match operation {
        ProviderOperationKind::Launch => "launch",
        ProviderOperationKind::Command => "command",
        ProviderOperationKind::Reconcile => "reconcile",
    }
}

fn revision_i64(revision: u64) -> Result<i64> {
    i64::try_from(revision).map_err(|_| ProviderError::InvalidInput)
}

pub(crate) fn database_error(error: sqlx::Error) -> ProviderError {
    tracing::error!(error = %error, "provider starter database operation failed");
    ProviderError::Internal
}
