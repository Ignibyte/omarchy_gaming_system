use std::{sync::Arc, time::SystemTime};

use axum::http::HeaderMap;
use omarchy_game_provider::{
    ProviderError,
    broker::{
        AuthenticatedProviderEvent, BrokerOperationInput, CallbackDisposition,
        CallbackReceiptOutcome, ProviderBroker,
    },
    model::SessionAdmission,
    protocol::{
        GrantIssuer, HttpMessageSigner, ProviderEvent, ProviderEventKind,
        ProviderOperationDisposition, ProviderOperationKind, ProviderOperationResponse,
        ProviderSessionStatus, RequestSignatureContext, SignatureHeaders, sha256_hex,
    },
    registry::ProviderRegistry,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{FromRow, PgPool, Postgres, Transaction, types::Json};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    config::ProviderConfig,
    games::{
        self, GameCommandInput, GameCommandResult, GameError, GameSessionStartOutcome,
        StartGameSessionInput,
    },
    sync::{self, SyncEventKind},
};

const PROVIDER_GRANT_KEY_ID: &str = "ogs-grant-v1";
const PROVIDER_MESSAGE_KEY_ID: &str = "ogs-message-v1";
const CALLBACK_PATH_PREFIX: &str = "/v1/provider-events/";

#[derive(Clone)]
pub(crate) struct ProviderRuntime {
    broker: Arc<ProviderBroker>,
    callback_authority: Arc<str>,
}

impl ProviderRuntime {
    pub(crate) fn production(pool: PgPool, config: ProviderConfig) -> Result<Self, ProviderError> {
        let grant_issuer = GrantIssuer::new(
            PROVIDER_GRANT_KEY_ID,
            config.grant_signing_seed,
            config.pairwise_secret,
        )?;
        let message_signer =
            HttpMessageSigner::new(PROVIDER_MESSAGE_KEY_ID, config.message_signing_seed)?;
        Ok(Self {
            broker: Arc::new(ProviderBroker::new(
                ProviderRegistry::new(pool),
                grant_issuer,
                message_signer,
            )),
            callback_authority: Arc::from(config.callback_authority),
        })
    }

    #[cfg(test)]
    pub(crate) fn for_broker(broker: ProviderBroker, callback_authority: &str) -> Self {
        Self {
            broker: Arc::new(broker),
            callback_authority: Arc::from(callback_authority),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderGameManifest {
    pub key: String,
    pub version: u32,
    pub display_name: String,
    pub min_human_players: u8,
    pub max_human_players: u8,
    pub release_id: Uuid,
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct ProviderAchievement {
    pub key: String,
    pub display_name: String,
    pub description: String,
    pub game_key: String,
    pub game_version: u32,
    pub provider_release_id: Uuid,
    pub game_session_id: Uuid,
    pub provider_revision: i64,
    pub awarded_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallbackApplyOutcome {
    Accepted,
    Duplicate,
    Ignored,
}

#[derive(FromRow)]
struct PilotManifestRow {
    game_key: String,
    rules_version: i64,
    display_name: String,
    min_human_players: i16,
    max_human_players: i16,
    release_id: Uuid,
}

#[derive(FromRow)]
struct ProviderSessionOperationRow {
    release_id: Uuid,
}

#[derive(FromRow)]
struct CallbackSessionRow {
    release_id: Uuid,
    provider_id: String,
    game_key: String,
    game_version: i64,
    cartridge_digest: String,
    persona_id: Uuid,
}

#[derive(FromRow)]
struct ProviderStartReceiptRow {
    game_session_id: Uuid,
    game_key: String,
    game_version: i64,
    authority: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderResponsePayload {
    view: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultEventPayload {
    outcome: String,
    public_summary: Value,
    achievements: Vec<String>,
    view: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TurnReadyEventPayload {
    view: Value,
}

pub(crate) async fn active_catalog(pool: &PgPool) -> Result<Vec<ProviderGameManifest>, GameError> {
    let rows = sqlx::query_as::<_, PilotManifestRow>(
        r#"
        SELECT
            release.game_key,
            release.rules_version,
            pilot.display_name,
            pilot.min_human_players,
            pilot.max_human_players,
            release.release_id
        FROM provider_game_pilots AS pilot
        JOIN provider_releases AS release ON release.release_id = pilot.release_id
        JOIN provider_registrations AS provider ON provider.provider_id = release.provider_id
        WHERE pilot.status = 'active'
          AND release.status = 'active'
          AND provider.status = 'active'
        ORDER BY release.game_key, release.rules_version
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| database_error(error, "load active provider catalog"))?;
    rows.into_iter().map(convert_manifest).collect()
}

pub(crate) async fn is_provider_session(
    pool: &PgPool,
    session_id: &str,
) -> Result<bool, GameError> {
    let session_id = Uuid::try_parse(session_id).map_err(|_| GameError::GameSessionNotFound)?;
    let authority: Option<String> =
        sqlx::query_scalar("SELECT authority FROM game_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| database_error(error, "resolve game session authority"))?;
    Ok(authority.as_deref() == Some("registered_provider"))
}

pub(crate) async fn start_solo_session(
    pool: &PgPool,
    runtime: &ProviderRuntime,
    token: &str,
    actor_id: &str,
    input: StartGameSessionInput,
) -> Result<Option<GameSessionStartOutcome>, GameError> {
    let StartGameSessionInput {
        idempotency_key,
        game_key,
        game_version,
    } = input;
    let actor_id = games::authenticate_owned_persona(pool, token, actor_id).await?;
    let idempotency_key = Uuid::try_parse(&idempotency_key).map_err(|_| GameError::InvalidStart)?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| database_error(error, "begin provider game start"))?;
    let actor_exists: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM personas WHERE id = $1 FOR UPDATE")
            .bind(actor_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|error| database_error(error, "lock provider game persona"))?;
    if actor_exists.is_none() {
        return Err(GameError::PersonaNotFound);
    }
    let existing = sqlx::query_as::<_, ProviderStartReceiptRow>(
        r#"
        SELECT start.game_session_id, start.game_key, start.game_version, session.authority
        FROM game_session_starts AS start
        JOIN game_sessions AS session ON session.id = start.game_session_id
        WHERE start.persona_id = $1 AND start.idempotency_key = $2
        "#,
    )
    .bind(actor_id)
    .bind(idempotency_key)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| database_error(error, "load provider game start replay"))?;
    let (session_id, created) = if let Some(existing) = existing {
        if existing.game_key != game_key || existing.game_version != i64::from(game_version) {
            return Err(GameError::IdempotencyConflict);
        }
        if existing.authority != "registered_provider" {
            transaction
                .commit()
                .await
                .map_err(|error| database_error(error, "commit non-provider start lookup"))?;
            return Ok(None);
        }
        (existing.game_session_id, false)
    } else {
        let pilot = load_active_manifest(&mut transaction, &game_key, game_version).await?;
        let Some(pilot) = pilot else {
            transaction
                .commit()
                .await
                .map_err(|error| database_error(error, "commit provider catalog miss"))?;
            return Ok(None);
        };
        if pilot.min_human_players != 1 || pilot.max_human_players != 1 {
            return Err(GameError::InvalidParticipants);
        }
        let active_count: i64 = sqlx::query_scalar(
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
        .map_err(|error| database_error(error, "count active provider sessions"))?;
        if active_count >= games::MAX_ACTIVE_SOLO_SESSIONS_PER_PERSONA {
            return Err(GameError::ActiveSessionLimit);
        }
        let session_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO game_sessions (
                game_key,
                game_version,
                revision,
                status,
                state,
                authority,
                provider_release_id,
                provider_availability
            )
            VALUES ($1, $2, 0, 'active', NULL, 'registered_provider', $3, 'provisioning')
            RETURNING id
            "#,
        )
        .bind(&pilot.key)
        .bind(i64::from(pilot.version))
        .bind(pilot.release_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| database_error(error, "insert provider session envelope"))?;
        sqlx::query(
            r#"
            INSERT INTO game_session_participants (game_session_id, persona_id, seat)
            VALUES ($1, $2, 0)
            "#,
        )
        .bind(session_id)
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| database_error(error, "insert provider session participant"))?;
        sqlx::query(
            r#"
            INSERT INTO game_session_starts (
                persona_id, idempotency_key, game_session_id, game_key, game_version
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(actor_id)
        .bind(idempotency_key)
        .bind(session_id)
        .bind(&pilot.key)
        .bind(i64::from(pilot.version))
        .execute(&mut *transaction)
        .await
        .map_err(|error| database_error(error, "insert provider start receipt"))?;
        append_game_sync(&mut transaction, actor_id, session_id).await?;
        (session_id, true)
    };
    transaction
        .commit()
        .await
        .map_err(|error| database_error(error, "commit provider session envelope"))?;

    let operation = ProviderOperationKind::Launch;
    let operation_result = execute_operation(
        pool,
        runtime,
        actor_id,
        session_id,
        idempotency_key,
        0,
        operation,
        json!({"player_count": 1}),
    )
    .await;
    match operation_result {
        Ok(_) | Err(GameError::RevisionConflict) => {}
        Err(error) => {
            mark_operation_failure(pool, session_id, &error).await?;
        }
    }
    let session = games::load_session_for_participant(pool, actor_id, session_id).await?;
    if session.availability.as_deref() != Some("ready") {
        Ok(Some(GameSessionStartOutcome::Pending(session)))
    } else if created {
        info!(%session_id, %actor_id, "provider game session started");
        Ok(Some(GameSessionStartOutcome::Created(session)))
    } else {
        Ok(Some(GameSessionStartOutcome::Existing(session)))
    }
}

pub(crate) async fn apply_command(
    pool: &PgPool,
    runtime: &ProviderRuntime,
    token: &str,
    actor_id: &str,
    session_id: &str,
    input: GameCommandInput,
) -> Result<GameCommandResult, GameError> {
    provider_operation(
        pool,
        runtime,
        token,
        actor_id,
        session_id,
        input,
        ProviderOperationKind::Command,
    )
    .await
}

pub(crate) async fn reconcile(
    pool: &PgPool,
    runtime: &ProviderRuntime,
    token: &str,
    actor_id: &str,
    session_id: &str,
    idempotency_key: String,
    expected_revision: i64,
) -> Result<GameCommandResult, GameError> {
    provider_operation(
        pool,
        runtime,
        token,
        actor_id,
        session_id,
        GameCommandInput {
            idempotency_key,
            expected_revision,
            command: json!({}),
        },
        ProviderOperationKind::Reconcile,
    )
    .await
}

async fn provider_operation(
    pool: &PgPool,
    runtime: &ProviderRuntime,
    token: &str,
    actor_id: &str,
    session_id: &str,
    input: GameCommandInput,
    operation: ProviderOperationKind,
) -> Result<GameCommandResult, GameError> {
    let actor_id = games::authenticate_owned_persona(pool, token, actor_id).await?;
    let session_id = Uuid::try_parse(session_id).map_err(|_| GameError::GameSessionNotFound)?;
    let idempotency_key =
        Uuid::try_parse(&input.idempotency_key).map_err(|_| GameError::InvalidCommand)?;
    if input.expected_revision < 0 || !input.command.is_object() {
        return Err(GameError::InvalidCommand);
    }
    load_operation_session(pool, actor_id, session_id).await?;
    let expected_revision =
        u64::try_from(input.expected_revision).map_err(|_| GameError::InvalidCommand)?;
    let payload = match operation {
        ProviderOperationKind::Command => json!({"command": input.command}),
        ProviderOperationKind::Reconcile => json!({}),
        ProviderOperationKind::Launch => return Err(GameError::Internal),
    };
    match execute_operation(
        pool,
        runtime,
        actor_id,
        session_id,
        idempotency_key,
        expected_revision,
        operation,
        payload,
    )
    .await
    {
        Ok(result) => Ok(result),
        Err(error) => {
            mark_operation_failure(pool, session_id, &error).await?;
            Err(error)
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_operation(
    pool: &PgPool,
    runtime: &ProviderRuntime,
    persona_id: Uuid,
    session_id: Uuid,
    idempotency_key: Uuid,
    expected_revision: u64,
    operation: ProviderOperationKind,
    payload: Value,
) -> Result<GameCommandResult, GameError> {
    let row = load_operation_session(pool, persona_id, session_id).await?;
    let response = runtime
        .broker
        .execute(&BrokerOperationInput {
            release_id: row.release_id,
            persona_id,
            platform_session_id: session_id,
            idempotency_key,
            expected_revision,
            operation,
            session: if operation == ProviderOperationKind::Launch {
                SessionAdmission::New
            } else {
                SessionAdmission::Existing
            },
            payload,
        })
        .await
        .map_err(map_provider_error)?;
    apply_authenticated_response(pool, persona_id, session_id, expected_revision, response).await
}

async fn apply_authenticated_response(
    pool: &PgPool,
    persona_id: Uuid,
    session_id: Uuid,
    expected_revision: u64,
    response: ProviderOperationResponse,
) -> Result<GameCommandResult, GameError> {
    let response_payload: ProviderResponsePayload =
        serde_json::from_value(response.payload.clone())
            .map_err(|_| GameError::ProviderUnavailable)?;
    validate_door_legends_view(&response_payload.view)?;
    let response_revision =
        i64::try_from(response.revision).map_err(|_| GameError::ProviderUnavailable)?;
    let expected_revision =
        i64::try_from(expected_revision).map_err(|_| GameError::ProviderUnavailable)?;
    let response_digest =
        sha256_hex(&serde_json::to_vec(&response).map_err(|_| GameError::Internal)?);
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| database_error(error, "begin provider response projection"))?;
    let current = sqlx::query_as::<_, (i64, String, Uuid, Option<String>)>(
        r#"
        SELECT session.revision,
               session.status,
               session.provider_release_id,
               pilot.status
        FROM game_sessions AS session
        LEFT JOIN provider_game_pilots AS pilot
          ON pilot.release_id = session.provider_release_id
        WHERE session.id = $1
          AND session.authority = 'registered_provider'
          AND session.provider_release_id IS NOT NULL
          AND EXISTS (
              SELECT 1 FROM game_session_participants
              WHERE game_session_id = session.id AND persona_id = $2
          )
        FOR UPDATE OF session
        "#,
    )
    .bind(session_id)
    .bind(persona_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| database_error(error, "lock provider response envelope"))?
    .ok_or(GameError::GameSessionNotFound)?;
    if current.2 != response.release_id || response.platform_session_id != session_id {
        return Err(GameError::ProviderUnavailable);
    }
    if response_revision < current.0 {
        return Err(GameError::RevisionConflict);
    }
    if response.disposition == ProviderOperationDisposition::Applied
        && response.revision > 0
        && current.0 != expected_revision
        && current.0 != response_revision
    {
        return Err(GameError::RevisionConflict);
    }
    upsert_view(
        &mut transaction,
        session_id,
        response.release_id,
        response_revision,
        &response_digest,
        &response_payload.view,
    )
    .await?;
    let status = match response.status {
        ProviderSessionStatus::Active => "active",
        ProviderSessionStatus::Completed => "completed",
    };
    let availability = match current.3.as_deref() {
        Some("suspended") => "suspended",
        Some("retired") => "retired",
        _ => "ready",
    };
    sqlx::query(
        r#"
        WITH mutation AS (SELECT clock_timestamp() AS at)
        UPDATE game_sessions
        SET revision = GREATEST(revision, $2),
            status = CASE WHEN $3 = 'completed' THEN 'completed' ELSE status END,
            provider_availability = $4,
            completed_at = CASE
                WHEN $3 = 'completed' THEN COALESCE(completed_at, mutation.at)
                ELSE completed_at
            END,
            updated_at = GREATEST(updated_at, mutation.at)
        FROM mutation
        WHERE id = $1 AND revision <= $2
        "#,
    )
    .bind(session_id)
    .bind(response_revision)
    .bind(status)
    .bind(availability)
    .execute(&mut *transaction)
    .await
    .map_err(|error| database_error(error, "project provider response envelope"))?;
    append_game_sync(&mut transaction, persona_id, session_id).await?;
    transaction
        .commit()
        .await
        .map_err(|error| database_error(error, "commit provider response projection"))?;
    if response.disposition == ProviderOperationDisposition::RevisionConflict {
        return Err(GameError::RevisionConflict);
    }
    Ok(GameCommandResult {
        game_session_id: session_id,
        revision: response_revision,
        status: status.to_owned(),
        state: response_payload.view,
        authority: "registered_provider".to_owned(),
        provider_release_id: Some(response.release_id),
        availability: Some(availability.to_owned()),
    })
}

pub(crate) async fn apply_callback(
    pool: &PgPool,
    runtime: &ProviderRuntime,
    release_id: &str,
    received_authority: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<CallbackApplyOutcome, GameError> {
    let release_id = Uuid::try_parse(release_id).map_err(|_| GameError::GameSessionNotFound)?;
    if received_authority != runtime.callback_authority.as_ref() {
        return Err(GameError::Unauthorized);
    }
    let signature_headers =
        SignatureHeaders::from_header_map(headers).map_err(|_| GameError::Unauthorized)?;
    let message_id = signature_headers
        .message_id
        .parse::<Uuid>()
        .map_err(|_| GameError::Unauthorized)?;
    let untrusted: ProviderEvent =
        serde_json::from_slice(body).map_err(|_| GameError::Unauthorized)?;
    untrusted.validate().map_err(|_| GameError::Unauthorized)?;
    if untrusted.release_id != release_id || untrusted.message_id != message_id {
        return Err(GameError::Unauthorized);
    }
    let row = load_callback_session(pool, untrusted.platform_session_id).await?;
    if row.release_id != release_id
        || row.provider_id != signature_headers.provider_id
        || row.game_key != untrusted.game_key
        || row.game_version != i64::from(untrusted.rules_version)
        || row.cartridge_digest != untrusted.cartridge_digest
    {
        return Err(GameError::Unauthorized);
    }
    let subject = runtime
        .broker
        .pairwise_subject(&row.provider_id, &row.game_key, row.persona_id)
        .map_err(map_provider_error)?;
    let path = format!("{CALLBACK_PATH_PREFIX}{release_id}");
    let context = RequestSignatureContext {
        method: "POST",
        authority: runtime.callback_authority.as_ref(),
        path: &path,
        provider_id: &row.provider_id,
        release_id,
        message_id,
    };
    let now = current_unix_seconds()?;
    let authenticated = runtime
        .broker
        .authenticate_callback(
            release_id,
            Some(&subject),
            &context,
            &signature_headers,
            body,
            now,
        )
        .await
        .map_err(map_callback_auth_error)?;
    project_callback(pool, runtime, row, authenticated).await
}

async fn project_callback(
    pool: &PgPool,
    runtime: &ProviderRuntime,
    expected: CallbackSessionRow,
    authenticated: AuthenticatedProviderEvent,
) -> Result<CallbackApplyOutcome, GameError> {
    let event = authenticated.event();
    let event_revision =
        i64::try_from(event.revision).map_err(|_| GameError::ProviderUnavailable)?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| database_error(error, "begin provider callback projection"))?;
    sqlx::query("SELECT release_id FROM provider_releases WHERE release_id = $1 FOR UPDATE")
        .bind(event.release_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| database_error(error, "lock provider callback release"))?;
    let pilot_status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM provider_game_pilots WHERE release_id = $1 FOR UPDATE",
    )
    .bind(event.release_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| database_error(error, "lock provider callback pilot"))?;
    if pilot_status
        .as_deref()
        .is_some_and(|status| status != "active")
    {
        return Err(GameError::Unauthorized);
    }
    let locked = sqlx::query_as::<_, (Uuid, i64, String, Uuid)>(
        r#"
        SELECT session.provider_release_id, session.revision, session.status, participant.persona_id
        FROM game_sessions AS session
        JOIN game_session_participants AS participant
          ON participant.game_session_id = session.id AND participant.seat = 0
        WHERE session.id = $1
          AND session.authority = 'registered_provider'
        FOR UPDATE OF session
        "#,
    )
    .bind(event.platform_session_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| database_error(error, "lock provider callback envelope"))?
    .ok_or(GameError::GameSessionNotFound)?;
    if locked.0 != event.release_id || locked.3 != expected.persona_id {
        return Err(GameError::Unauthorized);
    }
    let policy_valid = callback_policy_valid(&mut transaction, event, locked.1).await?;
    let receipt_outcome = if policy_valid {
        CallbackReceiptOutcome::Accepted
    } else {
        CallbackReceiptOutcome::Ignored
    };
    let receipt = runtime
        .broker
        .claim_callback_receipt(&mut transaction, &authenticated, receipt_outcome)
        .await
        .map_err(map_provider_error)?;
    if receipt == CallbackDisposition::Duplicate {
        insert_callback_audit(
            &mut transaction,
            &expected.provider_id,
            event,
            authenticated.key_id(),
            "callback_duplicate",
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| database_error(error, "commit duplicate provider callback"))?;
        return Ok(CallbackApplyOutcome::Duplicate);
    }
    if !policy_valid {
        insert_callback_audit(
            &mut transaction,
            &expected.provider_id,
            event,
            authenticated.key_id(),
            "callback_policy_ignored",
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|error| database_error(error, "commit ignored provider callback"))?;
        warn!(event_id = %event.event_id, session_id = %event.platform_session_id, "ignored authenticated provider event outside platform policy");
        return Ok(CallbackApplyOutcome::Ignored);
    }
    match event.kind {
        ProviderEventKind::ResultAvailable => {
            let payload: ResultEventPayload =
                serde_json::from_value(event.payload.clone()).map_err(|_| GameError::Internal)?;
            upsert_view(
                &mut transaction,
                event.platform_session_id,
                event.release_id,
                event_revision,
                &sha256_hex(&serde_json::to_vec(event).map_err(|_| GameError::Internal)?),
                &payload.view,
            )
            .await?;
            sqlx::query(
                r#"
                INSERT INTO provider_game_results (
                    game_session_id, release_id, provider_revision, outcome, public_summary
                )
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(event.platform_session_id)
            .bind(event.release_id)
            .bind(event_revision)
            .bind(&payload.outcome)
            .bind(Json(&payload.public_summary))
            .execute(&mut *transaction)
            .await
            .map_err(|error| database_error(error, "insert provider result projection"))?;
            for achievement_key in payload.achievements {
                sqlx::query(
                    r#"
                    INSERT INTO persona_provider_achievements (
                        persona_id,
                        release_id,
                        achievement_key,
                        game_session_id,
                        provider_revision
                    )
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (persona_id, release_id, achievement_key) DO NOTHING
                    "#,
                )
                .bind(expected.persona_id)
                .bind(event.release_id)
                .bind(achievement_key)
                .bind(event.platform_session_id)
                .bind(event_revision)
                .execute(&mut *transaction)
                .await
                .map_err(|error| database_error(error, "insert provider achievement projection"))?;
            }
            sqlx::query(
                r#"
                WITH mutation AS (SELECT clock_timestamp() AS at)
                UPDATE game_sessions
                SET revision = GREATEST(revision, $2),
                    status = 'completed',
                    provider_availability = 'ready',
                    completed_at = COALESCE(completed_at, mutation.at),
                    updated_at = GREATEST(updated_at, mutation.at)
                FROM mutation
                WHERE id = $1
                "#,
            )
            .bind(event.platform_session_id)
            .bind(event_revision)
            .execute(&mut *transaction)
            .await
            .map_err(|error| database_error(error, "complete provider session projection"))?;
        }
        ProviderEventKind::TurnReady => {
            let payload: TurnReadyEventPayload =
                serde_json::from_value(event.payload.clone()).map_err(|_| GameError::Internal)?;
            upsert_view(
                &mut transaction,
                event.platform_session_id,
                event.release_id,
                event_revision,
                &sha256_hex(&serde_json::to_vec(event).map_err(|_| GameError::Internal)?),
                &payload.view,
            )
            .await?;
            sqlx::query(
                r#"
                UPDATE game_sessions
                SET revision = GREATEST(revision, $2),
                    provider_availability = 'ready',
                    updated_at = GREATEST(updated_at, clock_timestamp())
                WHERE id = $1
                "#,
            )
            .bind(event.platform_session_id)
            .bind(event_revision)
            .execute(&mut *transaction)
            .await
            .map_err(|error| database_error(error, "project provider turn-ready event"))?;
        }
        ProviderEventKind::ReconciliationRequired => {
            sqlx::query(
                r#"
                UPDATE game_sessions
                SET provider_availability = 'reconciling',
                    updated_at = GREATEST(updated_at, clock_timestamp())
                WHERE id = $1
                "#,
            )
            .bind(event.platform_session_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| database_error(error, "project provider reconciliation request"))?;
        }
    }
    append_game_sync(
        &mut transaction,
        expected.persona_id,
        event.platform_session_id,
    )
    .await?;
    insert_callback_audit(
        &mut transaction,
        &expected.provider_id,
        event,
        authenticated.key_id(),
        "callback_projected",
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(|error| database_error(error, "commit provider callback projection"))?;
    Ok(CallbackApplyOutcome::Accepted)
}

async fn callback_policy_valid(
    transaction: &mut Transaction<'_, Postgres>,
    event: &ProviderEvent,
    current_revision: i64,
) -> Result<bool, GameError> {
    let event_revision = match i64::try_from(event.revision) {
        Ok(revision) => revision,
        Err(_) => return Ok(false),
    };
    if event_revision < current_revision {
        return Ok(false);
    }
    match event.kind {
        ProviderEventKind::ResultAvailable => {
            if event_revision == 0 {
                return Ok(false);
            }
            let Ok(payload) = serde_json::from_value::<ResultEventPayload>(event.payload.clone())
            else {
                return Ok(false);
            };
            if !canonical_identifier(&payload.outcome, 2, 32)
                || !bounded_public_object(&payload.public_summary, 8 * 1024)
                || payload.achievements.len() > 64
                || validate_door_legends_view(&payload.view).is_err()
            {
                return Ok(false);
            }
            let mut unique = payload.achievements.clone();
            unique.sort();
            unique.dedup();
            if unique.len() != payload.achievements.len()
                || unique.iter().any(|key| !canonical_identifier(key, 2, 48))
            {
                return Ok(false);
            }
            let count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM provider_achievement_definitions
                WHERE release_id = $1 AND achievement_key = ANY($2)
                "#,
            )
            .bind(event.release_id)
            .bind(&unique)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|error| database_error(error, "validate provider achievement claims"))?;
            let no_result: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM provider_game_results WHERE game_session_id = $1)",
            )
            .bind(event.platform_session_id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|error| database_error(error, "validate first provider result"))?;
            Ok(count == i64::try_from(unique.len()).unwrap_or(-1) && no_result)
        }
        ProviderEventKind::TurnReady => {
            let Ok(payload) =
                serde_json::from_value::<TurnReadyEventPayload>(event.payload.clone())
            else {
                return Ok(false);
            };
            Ok(validate_door_legends_view(&payload.view).is_ok())
        }
        ProviderEventKind::ReconciliationRequired => {
            Ok(event.payload.as_object().is_some_and(Map::is_empty))
        }
    }
}

pub(crate) async fn list_achievements(
    pool: &PgPool,
    token: &str,
    actor_id: &str,
) -> Result<Vec<ProviderAchievement>, GameError> {
    let actor_id = games::authenticate_owned_persona(pool, token, actor_id).await?;
    #[derive(FromRow)]
    struct Row {
        key: String,
        display_name: String,
        description: String,
        game_key: String,
        game_version: i64,
        provider_release_id: Uuid,
        game_session_id: Uuid,
        provider_revision: i64,
        awarded_at: String,
    }
    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            definition.achievement_key AS key,
            definition.display_name,
            definition.description,
            release.game_key,
            release.rules_version AS game_version,
            award.release_id AS provider_release_id,
            award.game_session_id,
            award.provider_revision,
            to_char(award.awarded_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS awarded_at
        FROM persona_provider_achievements AS award
        JOIN provider_achievement_definitions AS definition
          ON definition.release_id = award.release_id
         AND definition.achievement_key = award.achievement_key
        JOIN provider_releases AS release ON release.release_id = award.release_id
        WHERE award.persona_id = $1
        ORDER BY award.awarded_at DESC, award.release_id, award.achievement_key
        "#,
    )
    .bind(actor_id)
    .fetch_all(pool)
    .await
    .map_err(|error| database_error(error, "list provider achievements"))?;
    rows.into_iter()
        .map(|row| {
            Ok(ProviderAchievement {
                key: row.key,
                display_name: row.display_name,
                description: row.description,
                game_key: row.game_key,
                game_version: u32::try_from(row.game_version).map_err(|_| GameError::Internal)?,
                provider_release_id: row.provider_release_id,
                game_session_id: row.game_session_id,
                provider_revision: row.provider_revision,
                awarded_at: row.awarded_at,
            })
        })
        .collect()
}

async fn load_active_manifest(
    transaction: &mut Transaction<'_, Postgres>,
    game_key: &str,
    game_version: u32,
) -> Result<Option<ProviderGameManifest>, GameError> {
    let row = sqlx::query_as::<_, PilotManifestRow>(
        r#"
        SELECT
            release.game_key,
            release.rules_version,
            pilot.display_name,
            pilot.min_human_players,
            pilot.max_human_players,
            release.release_id
        FROM provider_game_pilots AS pilot
        JOIN provider_releases AS release ON release.release_id = pilot.release_id
        JOIN provider_registrations AS provider ON provider.provider_id = release.provider_id
        WHERE pilot.status = 'active'
          AND release.status = 'active'
          AND provider.status = 'active'
          AND release.game_key = $1
          AND release.rules_version = $2
        FOR UPDATE OF pilot, release, provider
        "#,
    )
    .bind(game_key)
    .bind(i64::from(game_version))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| database_error(error, "load exact provider pilot"))?;
    row.map(convert_manifest).transpose()
}

fn convert_manifest(row: PilotManifestRow) -> Result<ProviderGameManifest, GameError> {
    Ok(ProviderGameManifest {
        key: row.game_key,
        version: u32::try_from(row.rules_version).map_err(|_| GameError::Internal)?,
        display_name: row.display_name,
        min_human_players: u8::try_from(row.min_human_players).map_err(|_| GameError::Internal)?,
        max_human_players: u8::try_from(row.max_human_players).map_err(|_| GameError::Internal)?,
        release_id: row.release_id,
    })
}

async fn load_operation_session(
    pool: &PgPool,
    persona_id: Uuid,
    session_id: Uuid,
) -> Result<ProviderSessionOperationRow, GameError> {
    sqlx::query_as::<_, ProviderSessionOperationRow>(
        r#"
        SELECT
            session.provider_release_id AS release_id
        FROM game_sessions AS session
        JOIN provider_releases AS release ON release.release_id = session.provider_release_id
        JOIN game_session_participants AS participant
          ON participant.game_session_id = session.id AND participant.persona_id = $2
        WHERE session.id = $1 AND session.authority = 'registered_provider'
        "#,
    )
    .bind(session_id)
    .bind(persona_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| database_error(error, "load provider operation session"))?
    .ok_or(GameError::GameSessionNotFound)
}

async fn load_callback_session(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<CallbackSessionRow, GameError> {
    sqlx::query_as::<_, CallbackSessionRow>(
        r#"
        SELECT
            session.provider_release_id AS release_id,
            release.provider_id,
            session.game_key,
            session.game_version,
            release.cartridge_digest,
            participant.persona_id
        FROM game_sessions AS session
        JOIN provider_releases AS release ON release.release_id = session.provider_release_id
        JOIN game_session_participants AS participant
          ON participant.game_session_id = session.id AND participant.seat = 0
        WHERE session.id = $1 AND session.authority = 'registered_provider'
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| database_error(error, "load provider callback session"))?
    .ok_or(GameError::GameSessionNotFound)
}

async fn upsert_view(
    transaction: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    release_id: Uuid,
    revision: i64,
    digest: &str,
    view: &Value,
) -> Result<(), GameError> {
    validate_door_legends_view(view)?;
    sqlx::query(
        r#"
        INSERT INTO provider_game_session_views (
            game_session_id, release_id, provider_revision, authenticated_sha256, view
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (game_session_id) DO UPDATE
        SET provider_revision = EXCLUDED.provider_revision,
            authenticated_sha256 = EXCLUDED.authenticated_sha256,
            view = EXCLUDED.view,
            updated_at = clock_timestamp()
        WHERE provider_game_session_views.release_id = EXCLUDED.release_id
          AND provider_game_session_views.provider_revision <= EXCLUDED.provider_revision
        "#,
    )
    .bind(session_id)
    .bind(release_id)
    .bind(revision)
    .bind(digest)
    .bind(Json(view))
    .execute(&mut **transaction)
    .await
    .map_err(|error| database_error(error, "upsert authenticated provider view"))?;
    Ok(())
}

fn validate_door_legends_view(view: &Value) -> Result<(), GameError> {
    let Some(object) = view.as_object() else {
        return Err(GameError::ProviderUnavailable);
    };
    if object.len() != 3
        || !object.contains_key("enter_label")
        || !object.contains_key("status")
        || !object.contains_key("welcome")
        || object.values().any(|value| {
            value
                .as_str()
                .is_none_or(|value| value.is_empty() || value.chars().count() > 256)
        })
    {
        Err(GameError::ProviderUnavailable)
    } else {
        Ok(())
    }
}

fn bounded_public_object(value: &Value, max_bytes: usize) -> bool {
    value.is_object()
        && serde_json::to_vec(value)
            .ok()
            .is_some_and(|bytes| bytes.len() <= max_bytes)
}

fn canonical_identifier(value: &str, min: usize, max: usize) -> bool {
    let bytes = value.as_bytes();
    (min..=max).contains(&bytes.len())
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

async fn mark_operation_failure(
    pool: &PgPool,
    session_id: Uuid,
    error_kind: &GameError,
) -> Result<(), GameError> {
    let availability = match error_kind {
        GameError::GameUnavailable => "suspended",
        GameError::ProviderUnavailable
        | GameError::RevisionConflict
        | GameError::IdempotencyConflict => "reconciling",
        _ => "unavailable",
    };
    sqlx::query(
        r#"
        UPDATE game_sessions
        SET provider_availability = $2,
            updated_at = GREATEST(updated_at, clock_timestamp())
        WHERE id = $1 AND authority = 'registered_provider' AND status = 'active'
        "#,
    )
    .bind(session_id)
    .bind(availability)
    .execute(pool)
    .await
    .map_err(|error| database_error(error, "mark provider operation failure"))?;
    Ok(())
}

async fn append_game_sync(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: Uuid,
    session_id: Uuid,
) -> Result<(), GameError> {
    sync::append_event(
        transaction,
        persona_id,
        SyncEventKind::GameSession(session_id),
    )
    .await
    .map_err(|error| database_error(error, "append provider game sync event"))?;
    Ok(())
}

async fn insert_callback_audit(
    transaction: &mut Transaction<'_, Postgres>,
    provider_id: &str,
    event: &ProviderEvent,
    key_id: &str,
    reason_code: &str,
) -> Result<(), GameError> {
    sqlx::query(
        r#"
        INSERT INTO provider_security_audit_events (
            provider_id,
            release_id,
            actor_type,
            actor_id,
            event_type,
            outcome,
            reason_code,
            correlation_id,
            safe_details
        )
        VALUES ($1, $2, 'provider', 'registered_provider', 'callback_projection',
                'recorded', $3, $4, $5)
        "#,
    )
    .bind(provider_id)
    .bind(event.release_id)
    .bind(reason_code)
    .bind(event.message_id)
    .bind(json!({
        "event_id": event.event_id,
        "provider_revision": event.revision,
        "event_kind": event.kind,
        "key_id": key_id
    }))
    .execute(&mut **transaction)
    .await
    .map_err(|error| database_error(error, "insert provider callback audit"))?;
    Ok(())
}

fn map_provider_error(error: ProviderError) -> GameError {
    match error {
        ProviderError::InvalidInput => GameError::InvalidCommand,
        ProviderError::Conflict => GameError::IdempotencyConflict,
        ProviderError::NotFound | ProviderError::Denied => GameError::GameUnavailable,
        ProviderError::QuotaExceeded | ProviderError::Unavailable => GameError::ProviderUnavailable,
        ProviderError::ProtocolRejected => GameError::ProviderUnavailable,
        ProviderError::Internal => GameError::Internal,
    }
}

fn map_callback_auth_error(error: ProviderError) -> GameError {
    match error {
        ProviderError::InvalidInput
        | ProviderError::Conflict
        | ProviderError::NotFound
        | ProviderError::Denied
        | ProviderError::ProtocolRejected => GameError::Unauthorized,
        ProviderError::QuotaExceeded | ProviderError::Unavailable | ProviderError::Internal => {
            GameError::ProviderUnavailable
        }
    }
}

fn current_unix_seconds() -> Result<i64, GameError> {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| GameError::Internal)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| GameError::Internal)
}

fn database_error(error_value: sqlx::Error, operation: &'static str) -> GameError {
    error!(
        ?error_value,
        operation, "provider game database operation failed"
    );
    GameError::Internal
}
