use std::{env, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context as _, Result, anyhow};
use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode, header::HOST},
    response::Response,
    routing::post,
};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::VerifyingKey;
use omarchygs_provider_sdk::{
    ProviderError,
    protocol::{
        GrantExpectation, HttpMessageSigner, ProviderCompatibility,
        ProviderCompatibilityOffer, ProviderCompatibilitySelection, ProviderEvent,
        ProviderEventKind, ProviderOperationDisposition, ProviderOperationKind,
        ProviderOperationRequest, ProviderOperationResponse, ProviderSessionStatus,
        RequestSignatureContext, SignatureHeaders, parse_authenticated_json, sha256_hex,
        verify_grant, verify_request_signature,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use tokio::{signal, task::JoinHandle};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use url::Url;
use uuid::Uuid;

const PROVIDER_ID: &str = "ignibyte";
const GAME_KEY: &str = "door-legends";
const RULES_VERSION: u32 = 1;
const PLATFORM_GRANT_KEY_ID: &str = "ogs-grant-v1";
const PLATFORM_MESSAGE_KEY_ID: &str = "ogs-message-v1";
const PROVIDER_MESSAGE_KEY_ID: &str = "door-legends-message-v1";
const BASE_PATH: &str = "/omarchygs/provider/v1/";

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    release_id: Uuid,
    cartridge_digest: Arc<str>,
    authority: Arc<str>,
    platform_grant_key: VerifyingKey,
    platform_message_key: VerifyingKey,
    provider_signer: Arc<HttpMessageSigner>,
    conformance_reconcile_response_delay: Duration,
}

struct Config {
    bind_address: SocketAddr,
    database_url: String,
    tls_certificate: PathBuf,
    tls_private_key: PathBuf,
    release_id: Uuid,
    cartridge_digest: String,
    authority: String,
    platform_grant_key: VerifyingKey,
    platform_message_key: VerifyingKey,
    provider_signing_seed: [u8; 32],
    callback_url: Url,
    callback_root_der: Vec<u8>,
    callback_socket_override: Option<SocketAddr>,
    conformance_reconcile_response_delay: Duration,
}

#[derive(FromRow)]
struct SessionRow {
    pairwise_subject: String,
    revision: i64,
    status: String,
    room: String,
}

#[derive(FromRow)]
struct ReceiptRow {
    operation: String,
    expected_revision: i64,
    intent_sha256: String,
    response_body: Vec<u8>,
}

#[derive(FromRow)]
struct GrantReceiptRow {
    platform_session_id: Uuid,
    idempotency_key: Uuid,
    request_sha256: String,
}

#[derive(FromRow)]
struct OutboxRow {
    event_id: Uuid,
    message_id: Uuid,
    body: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LaunchPayload {
    player_count: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandEnvelope {
    command: DoorCommand,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DoorCommand {
    action: String,
}

#[derive(Serialize)]
struct Intent<'a> {
    platform_session_id: Uuid,
    subject: &'a str,
    idempotency_key: Uuid,
    expected_revision: u64,
    operation: ProviderOperationKind,
    payload: &'a Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow!("failed to install rustls crypto provider"))?;
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("door_legends_provider=info")),
        )
        .init();
    let config = Config::from_environment()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("failed to connect to the Door Legends database")?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to migrate the Door Legends database")?;
    let provider_signer = Arc::new(
        HttpMessageSigner::new(PROVIDER_MESSAGE_KEY_ID, config.provider_signing_seed)
            .map_err(provider_error)?,
    );
    let state = AppState {
        pool: pool.clone(),
        release_id: config.release_id,
        cartridge_digest: Arc::from(config.cartridge_digest),
        authority: Arc::from(config.authority),
        platform_grant_key: config.platform_grant_key,
        platform_message_key: config.platform_message_key,
        provider_signer: Arc::clone(&provider_signer),
        conformance_reconcile_response_delay: config.conformance_reconcile_response_delay,
    };
    let callback_worker = spawn_callback_worker(
        pool,
        provider_signer,
        config.release_id,
        config.callback_url,
        config.callback_root_der,
        config.callback_socket_override,
    )?;
    let tls = RustlsConfig::from_pem_file(config.tls_certificate, config.tls_private_key)
        .await
        .context("failed to load Door Legends TLS identity")?;
    let app = Router::new()
        .route(
            "/omarchygs/provider/v1/compatibility",
            post(handle_compatibility),
        )
        .route("/omarchygs/provider/v1/launch", post(handle_launch))
        .route("/omarchygs/provider/v1/commands", post(handle_command))
        .route("/omarchygs/provider/v1/reconcile", post(handle_reconcile))
        .with_state(state);
    info!(address = %config.bind_address, "Door Legends provider listening");
    let handle = Handle::new();
    let shutdown_handle = handle.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        shutdown_handle.graceful_shutdown(Some(Duration::from_secs(5)));
    });
    let server = axum_server::bind_rustls(config.bind_address, tls)
        .handle(handle)
        .serve(app.into_make_service());
    let result = server
        .await
        .context("Door Legends provider stopped unexpectedly");
    callback_worker.abort();
    result
}

async fn handle_compatibility(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    match process_compatibility(&state, &headers, &body) {
        Ok(response) => response,
        Err(error_value) => {
            warn!(code = error_value.code(), "Door Legends compatibility request rejected");
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
    }
}

fn process_compatibility(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<Response, ProviderError> {
    if body.is_empty() || body.len() > 65_536 {
        return Err(ProviderError::InvalidInput);
    }
    let received_authority = headers
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(ProviderError::ProtocolRejected)?;
    if received_authority != state.authority.as_ref() {
        return Err(ProviderError::ProtocolRejected);
    }
    let signature_headers = SignatureHeaders::from_header_map(headers)?;
    let message_id = signature_headers
        .message_id
        .parse::<Uuid>()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let path = format!("{BASE_PATH}compatibility");
    let context = RequestSignatureContext {
        method: "POST",
        authority: state.authority.as_ref(),
        path: &path,
        provider_id: PROVIDER_ID,
        release_id: state.release_id,
        message_id,
    };
    verify_request_signature(
        &signature_headers,
        &context,
        body,
        &state.platform_message_key,
        PLATFORM_MESSAGE_KEY_ID,
        current_unix_seconds()?,
    )?;
    let offer: ProviderCompatibilityOffer = parse_authenticated_json(body, 65_536)?;
    offer.validate()?;
    if offer.provider_id != PROVIDER_ID
        || offer.release_id != state.release_id
        || offer.message_id != message_id
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let selection = ProviderCompatibilitySelection::current(&offer, Uuid::new_v4())?;
    let response_body = serde_json::to_vec(&selection).map_err(|_| ProviderError::Internal)?;
    let response_headers = state.provider_signer.sign_response(
        200,
        &context,
        selection.message_id,
        &response_body,
        current_unix_seconds()?,
        &format!("door-{}", Uuid::new_v4()),
    )?;
    let mut response = Response::builder().status(StatusCode::OK);
    for (name, value) in &response_headers.to_header_map()? {
        response = response.header(name, value);
    }
    response
        .body(Body::from(response_body))
        .map_err(|_| ProviderError::Internal)
}

async fn handle_launch(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    handle_operation(state, headers, body, ProviderOperationKind::Launch).await
}

async fn handle_command(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_operation(state, headers, body, ProviderOperationKind::Command).await
}

async fn handle_reconcile(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_operation(state, headers, body, ProviderOperationKind::Reconcile).await
}

async fn handle_operation(
    state: AppState,
    headers: HeaderMap,
    body: Bytes,
    operation: ProviderOperationKind,
) -> Response {
    match process_operation(&state, &headers, &body, operation).await {
        Ok(response) => response,
        Err(error_value) => {
            warn!(code = error_value.code(), operation = ?operation, "Door Legends provider request rejected");
            Response::builder()
                .status(match error_value {
                    ProviderError::Conflict => StatusCode::CONFLICT,
                    ProviderError::Denied | ProviderError::ProtocolRejected => {
                        StatusCode::UNAUTHORIZED
                    }
                    ProviderError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
                    ProviderError::NotFound => StatusCode::NOT_FOUND,
                    ProviderError::QuotaExceeded => StatusCode::TOO_MANY_REQUESTS,
                    ProviderError::Unavailable | ProviderError::Internal => {
                        StatusCode::SERVICE_UNAVAILABLE
                    }
                })
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
    }
}

async fn process_operation(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
    operation: ProviderOperationKind,
) -> Result<Response, ProviderError> {
    if body.is_empty() || body.len() > 65_536 {
        return Err(ProviderError::InvalidInput);
    }
    let received_authority = headers
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(ProviderError::ProtocolRejected)?;
    if received_authority != state.authority.as_ref() {
        return Err(ProviderError::ProtocolRejected);
    }
    let signature_headers = SignatureHeaders::from_header_map(headers)?;
    let message_id = signature_headers
        .message_id
        .parse::<Uuid>()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let path = format!("{BASE_PATH}{}", operation.path());
    let context = RequestSignatureContext {
        method: "POST",
        authority: state.authority.as_ref(),
        path: &path,
        provider_id: PROVIDER_ID,
        release_id: state.release_id,
        message_id,
    };
    verify_request_signature(
        &signature_headers,
        &context,
        body,
        &state.platform_message_key,
        PLATFORM_MESSAGE_KEY_ID,
        current_unix_seconds()?,
    )?;
    let request: ProviderOperationRequest = parse_authenticated_json(body, 65_536)?;
    request.validate()?;
    if request.provider_id != PROVIDER_ID
        || request.release_id != state.release_id
        || request.game_key != GAME_KEY
        || request.rules_version != RULES_VERSION
        || request.cartridge_digest != state.cartridge_digest.as_ref()
        || request.message_id != message_id
        || request.operation != operation
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let claims = verify_grant(
        &request.grant,
        &state.platform_grant_key,
        &GrantExpectation {
            key_id: PLATFORM_GRANT_KEY_ID,
            provider_id: PROVIDER_ID,
            release_id: state.release_id,
            game_key: GAME_KEY,
            rules_version: RULES_VERSION,
            cartridge_digest: state.cartridge_digest.as_ref(),
            platform_session_id: request.platform_session_id,
            subject: &request.subject,
            scope: operation.scope(),
            compatibility: &request.compatibility,
        },
        current_unix_seconds()?,
    )?;
    let intent_bytes = serde_json::to_vec(&Intent {
        platform_session_id: request.platform_session_id,
        subject: &request.subject,
        idempotency_key: request.idempotency_key,
        expected_revision: request.expected_revision,
        operation,
        payload: &request.payload,
    })
    .map_err(|_| ProviderError::Internal)?;
    let intent_digest = sha256_hex(&intent_bytes);
    let request_digest = sha256_hex(body);
    let response_body = apply_operation_transaction(
        &state.pool,
        state,
        &request,
        claims.token_id,
        &request_digest,
        &intent_digest,
    )
    .await?;
    if operation == ProviderOperationKind::Reconcile
        && !state.conformance_reconcile_response_delay.is_zero()
    {
        tokio::time::sleep(state.conformance_reconcile_response_delay).await;
    }
    let response = ProviderOperationResponse::from_persisted_v1_bytes(
        &response_body,
        65_536,
        request.compatibility.clone(),
    )?;
    response.validate_for(&request)?;
    let response_body = serde_json::to_vec(&response).map_err(|_| ProviderError::Internal)?;
    let response_headers = state.provider_signer.sign_response(
        200,
        &context,
        response.message_id,
        &response_body,
        current_unix_seconds()?,
        &format!("door-{}", Uuid::new_v4()),
    )?;
    let mut response_builder = Response::builder().status(StatusCode::OK);
    for (name, value) in &response_headers.to_header_map()? {
        response_builder = response_builder.header(name, value);
    }
    response_builder
        .body(Body::from(response_body))
        .map_err(|_| ProviderError::Internal)
}

async fn apply_operation_transaction(
    pool: &PgPool,
    state: &AppState,
    request: &ProviderOperationRequest,
    token_id: Uuid,
    request_digest: &str,
    intent_digest: &str,
) -> Result<Vec<u8>, ProviderError> {
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let existing_grant = sqlx::query_as::<_, GrantReceiptRow>(
        r#"
        SELECT platform_session_id, idempotency_key, request_sha256
        FROM door_legends_consumed_grants
        WHERE token_id = $1
        FOR UPDATE
        "#,
    )
    .bind(token_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?;
    if let Some(existing) = existing_grant {
        if existing.platform_session_id != request.platform_session_id
            || existing.idempotency_key != request.idempotency_key
            || existing.request_sha256 != request_digest
        {
            return Err(ProviderError::ProtocolRejected);
        }
    } else {
        sqlx::query(
            r#"
            INSERT INTO door_legends_consumed_grants (
                token_id, platform_session_id, idempotency_key, request_sha256
            )
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(token_id)
        .bind(request.platform_session_id)
        .bind(request.idempotency_key)
        .bind(request_digest)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }
    let receipt = sqlx::query_as::<_, ReceiptRow>(
        r#"
        SELECT operation, expected_revision, intent_sha256, response_body
        FROM door_legends_operation_receipts
        WHERE platform_session_id = $1 AND idempotency_key = $2
        FOR UPDATE
        "#,
    )
    .bind(request.platform_session_id)
    .bind(request.idempotency_key)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?;
    if let Some(receipt) = receipt {
        if receipt.operation != operation_name(request.operation)
            || receipt.expected_revision
                != i64::try_from(request.expected_revision)
                    .map_err(|_| ProviderError::InvalidInput)?
            || receipt.intent_sha256 != intent_digest
        {
            return Err(ProviderError::Conflict);
        }
        transaction.commit().await.map_err(database_error)?;
        return Ok(receipt.response_body);
    }
    let response = match request.operation {
        ProviderOperationKind::Launch => launch(&mut transaction, state, request).await?,
        ProviderOperationKind::Command => command(&mut transaction, state, request).await?,
        ProviderOperationKind::Reconcile => reconcile(&mut transaction, state, request).await?,
    };
    let response_body = serde_json::to_vec(&response).map_err(|_| ProviderError::Internal)?;
    if response_body.is_empty() || response_body.len() > 65_536 {
        return Err(ProviderError::Internal);
    }
    sqlx::query(
        r#"
        INSERT INTO door_legends_operation_receipts (
            platform_session_id,
            idempotency_key,
            operation,
            expected_revision,
            intent_sha256,
            response_body,
            provider_revision
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(request.platform_session_id)
    .bind(request.idempotency_key)
    .bind(operation_name(request.operation))
    .bind(i64::try_from(request.expected_revision).map_err(|_| ProviderError::InvalidInput)?)
    .bind(intent_digest)
    .bind(&response_body)
    .bind(i64::try_from(response.revision).map_err(|_| ProviderError::Internal)?)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    transaction.commit().await.map_err(database_error)?;
    Ok(response_body)
}

async fn launch(
    transaction: &mut Transaction<'_, Postgres>,
    state: &AppState,
    request: &ProviderOperationRequest,
) -> Result<ProviderOperationResponse, ProviderError> {
    let payload: LaunchPayload =
        serde_json::from_value(request.payload.clone()).map_err(|_| ProviderError::InvalidInput)?;
    if payload.player_count != 1 || request.expected_revision != 0 {
        return Err(ProviderError::InvalidInput);
    }
    sqlx::query(
        r#"
        INSERT INTO door_legends_sessions (platform_session_id, pairwise_subject)
        VALUES ($1, $2)
        ON CONFLICT (platform_session_id) DO NOTHING
        "#,
    )
    .bind(request.platform_session_id)
    .bind(&request.subject)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    let session = lock_session(transaction, request.platform_session_id).await?;
    if session.pairwise_subject != request.subject {
        return Err(ProviderError::Conflict);
    }
    build_response(
        state,
        request,
        &session,
        ProviderOperationDisposition::Applied,
    )
}

async fn command(
    transaction: &mut Transaction<'_, Postgres>,
    state: &AppState,
    request: &ProviderOperationRequest,
) -> Result<ProviderOperationResponse, ProviderError> {
    let payload: CommandEnvelope =
        serde_json::from_value(request.payload.clone()).map_err(|_| ProviderError::InvalidInput)?;
    let mut session = lock_session(transaction, request.platform_session_id).await?;
    if session.pairwise_subject != request.subject {
        return Err(ProviderError::ProtocolRejected);
    }
    if session.revision
        != i64::try_from(request.expected_revision).map_err(|_| ProviderError::InvalidInput)?
    {
        return build_response(
            state,
            request,
            &session,
            ProviderOperationDisposition::RevisionConflict,
        );
    }
    if session.status == "completed" {
        return Err(ProviderError::Conflict);
    }
    if payload.command.action != "enter" || session.room != "brass_door" {
        return Err(ProviderError::InvalidInput);
    }
    session.revision = session
        .revision
        .checked_add(1)
        .ok_or(ProviderError::Internal)?;
    session.status = "completed".to_owned();
    session.room = "sunlit_gate".to_owned();
    sqlx::query(
        r#"
        UPDATE door_legends_sessions
        SET revision = $2,
            status = 'completed',
            room = 'sunlit_gate',
            completed_at = clock_timestamp(),
            updated_at = clock_timestamp()
        WHERE platform_session_id = $1
        "#,
    )
    .bind(request.platform_session_id)
    .bind(session.revision)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    enqueue_result(transaction, state, request, &session).await?;
    build_response(
        state,
        request,
        &session,
        ProviderOperationDisposition::Applied,
    )
}

async fn reconcile(
    transaction: &mut Transaction<'_, Postgres>,
    state: &AppState,
    request: &ProviderOperationRequest,
) -> Result<ProviderOperationResponse, ProviderError> {
    if request
        .payload
        .as_object()
        .is_none_or(|object| !object.is_empty())
    {
        return Err(ProviderError::InvalidInput);
    }
    let session = lock_session(transaction, request.platform_session_id).await?;
    if session.pairwise_subject != request.subject {
        return Err(ProviderError::ProtocolRejected);
    }
    let disposition = if session.revision
        < i64::try_from(request.expected_revision).map_err(|_| ProviderError::InvalidInput)?
    {
        ProviderOperationDisposition::RevisionConflict
    } else {
        ProviderOperationDisposition::Applied
    };
    build_response(state, request, &session, disposition)
}

async fn lock_session(
    transaction: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
) -> Result<SessionRow, ProviderError> {
    sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT pairwise_subject, revision, status, room
        FROM door_legends_sessions
        WHERE platform_session_id = $1
        FOR UPDATE
        "#,
    )
    .bind(session_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?
    .ok_or(ProviderError::NotFound)
}

fn build_response(
    state: &AppState,
    request: &ProviderOperationRequest,
    session: &SessionRow,
    disposition: ProviderOperationDisposition,
) -> Result<ProviderOperationResponse, ProviderError> {
    Ok(ProviderOperationResponse::new(
        PROVIDER_ID.to_owned(),
        state.release_id,
        GAME_KEY.to_owned(),
        RULES_VERSION,
        state.cartridge_digest.to_string(),
        request.platform_session_id,
        request.subject.clone(),
        Uuid::new_v4(),
        request.idempotency_key,
        u64::try_from(session.revision).map_err(|_| ProviderError::Internal)?,
        disposition,
        if session.status == "completed" {
            ProviderSessionStatus::Completed
        } else {
            ProviderSessionStatus::Active
        },
        request.compatibility.clone(),
        json!({"view": view_for(session)}),
    ))
}

fn view_for(session: &SessionRow) -> Value {
    if session.room == "sunlit_gate" {
        json!({
            "chronicle_label": "Read the chronicle",
            "enter_label": "Play again later",
            "lobby_label": "Return to the lobby",
            "status": "You escaped through the sunlit gate.",
            "welcome": "Door Legends remembers your first escape."
        })
    } else {
        json!({
            "chronicle_label": "Read the chronicle",
            "enter_label": "Enter the brass door",
            "lobby_label": "Return to the lobby",
            "status": "A weathered brass door waits in the dark.",
            "welcome": "Welcome to Door Legends. One choice opens the way."
        })
    }
}

async fn enqueue_result(
    transaction: &mut Transaction<'_, Postgres>,
    state: &AppState,
    request: &ProviderOperationRequest,
    session: &SessionRow,
) -> Result<(), ProviderError> {
    let event_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();
    let event = ProviderEvent::new(
        PROVIDER_ID.to_owned(),
        state.release_id,
        GAME_KEY.to_owned(),
        RULES_VERSION,
        state.cartridge_digest.to_string(),
        request.platform_session_id,
        request.subject.clone(),
        message_id,
        event_id,
        u64::try_from(session.revision).map_err(|_| ProviderError::Internal)?,
        ProviderEventKind::ResultAvailable,
        request.compatibility.clone(),
        json!({
            "outcome": "escaped",
            "public_summary": {"ending": "sunlit_gate"},
            "achievements": ["first_escape"],
            "view": view_for(session)
        }),
    );
    event.validate()?;
    let body = serde_json::to_vec(&event).map_err(|_| ProviderError::Internal)?;
    sqlx::query(
        r#"
        INSERT INTO door_legends_event_outbox (
            event_id, platform_session_id, message_id, provider_revision, body
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(event_id)
    .bind(request.platform_session_id)
    .bind(message_id)
    .bind(session.revision)
    .bind(body)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

fn spawn_callback_worker(
    pool: PgPool,
    signer: Arc<HttpMessageSigner>,
    release_id: Uuid,
    callback_url: Url,
    callback_root_der: Vec<u8>,
    socket_override: Option<SocketAddr>,
) -> Result<JoinHandle<()>> {
    if callback_url.scheme() != "https"
        || callback_url.query().is_some()
        || callback_url.fragment().is_some()
        || callback_url.path() != format!("/v1/provider-events/{release_id}")
    {
        return Err(anyhow!("callback URL must be exact HTTPS release path"));
    }
    let authority = callback_url
        .host_str()
        .map(|host| match callback_url.port() {
            Some(port) if port != 443 => format!("{host}:{port}"),
            _ => host.to_owned(),
        })
        .ok_or_else(|| anyhow!("callback URL requires a DNS host"))?;
    let certificate =
        reqwest::Certificate::from_der(&callback_root_der).context("invalid callback TLS root")?;
    let mut client_builder = reqwest::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .tls_certs_only([certificate])
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5));
    #[cfg(feature = "conformance")]
    if let Some(socket) = socket_override {
        client_builder = client_builder.resolve(
            callback_url
                .host_str()
                .ok_or_else(|| anyhow!("callback URL requires a host"))?,
            socket,
        );
    }
    #[cfg(not(feature = "conformance"))]
    if socket_override.is_some() {
        return Err(anyhow!(
            "callback socket overrides require the conformance build"
        ));
    }
    let client = client_builder.build().context("build callback client")?;
    let path = callback_url.path().to_owned();
    Ok(tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            if let Err(error_value) = deliver_one_callback(
                &pool,
                &signer,
                &client,
                &callback_url,
                &authority,
                &path,
                release_id,
            )
            .await
            {
                warn!(?error_value, "Door Legends callback delivery deferred");
            }
        }
    }))
}

async fn deliver_one_callback(
    pool: &PgPool,
    signer: &HttpMessageSigner,
    client: &reqwest::Client,
    callback_url: &Url,
    authority: &str,
    path: &str,
    release_id: Uuid,
) -> Result<()> {
    let row = sqlx::query_as::<_, OutboxRow>(
        r#"
        SELECT event_id, message_id, body
        FROM door_legends_event_outbox
        WHERE status = 'pending'
        ORDER BY created_at, event_id
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(());
    };
    let (event, upgraded_body) =
        match parse_authenticated_json::<ProviderEvent>(&row.body, 65_536) {
            Ok(event) => (event, None),
            Err(_) => {
                let event = ProviderEvent::from_persisted_v1_bytes(
                    &row.body,
                    65_536,
                    ProviderCompatibility::current(),
                )
                .map_err(provider_error)?;
                let upgraded = serde_json::to_vec(&event).context("upgrade persisted callback")?;
                (event, Some(upgraded))
            }
        };
    if event.provider_id != PROVIDER_ID
        || event.release_id != release_id
        || event.event_id != row.event_id
        || event.message_id != row.message_id
    {
        return Err(anyhow!("persisted callback identity mismatch"));
    }
    let body = row.body;
    let context = RequestSignatureContext {
        method: "POST",
        authority,
        path,
        provider_id: PROVIDER_ID,
        release_id,
        message_id: row.message_id,
    };
    let headers = signer
        .sign_request(
            &context,
            &body,
            current_unix_seconds().map_err(provider_error)?,
            &format!("callback-{}", Uuid::new_v4()),
        )
        .map_err(provider_error)?;
    info!(event_id = %row.event_id, "Door Legends callback delivery started");
    let response = client
        .post(callback_url.clone())
        .headers(headers.to_header_map().map_err(provider_error)?)
        .body(body)
        .send()
        .await?;
    let response_status = response.status();
    let delivered = matches!(
        response_status,
        StatusCode::NO_CONTENT | StatusCode::ACCEPTED
    );
    let upgrade_rejected_legacy =
        !delivered && response_status == StatusCode::UNAUTHORIZED && upgraded_body.is_some();
    info!(event_id = %row.event_id, status = %response_status, "Door Legends callback delivery completed");
    sqlx::query(
        r#"
        UPDATE door_legends_event_outbox
        SET attempt_count = attempt_count + 1,
            status = CASE WHEN $2 THEN 'delivered' ELSE status END,
            delivered_at = CASE WHEN $2 THEN clock_timestamp() ELSE delivered_at END,
            body = CASE WHEN $3 THEN $4 ELSE body END,
            updated_at = clock_timestamp()
        WHERE event_id = $1 AND status = 'pending'
        "#,
    )
    .bind(row.event_id)
    .bind(delivered)
    .bind(upgrade_rejected_legacy)
    .bind(upgraded_body)
    .execute(pool)
    .await?;
    Ok(())
}

impl Config {
    fn from_environment() -> Result<Self> {
        let release_id = required("DOOR_LEGENDS_RELEASE_ID")?
            .parse()
            .context("DOOR_LEGENDS_RELEASE_ID must be a UUID")?;
        let cartridge_digest = required("DOOR_LEGENDS_CARTRIDGE_DIGEST")?;
        if cartridge_digest.len() != 64
            || !cartridge_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(anyhow!(
                "DOOR_LEGENDS_CARTRIDGE_DIGEST must be lowercase SHA-256"
            ));
        }
        let callback_url = Url::parse(&required("DOOR_LEGENDS_CALLBACK_URL")?)
            .context("DOOR_LEGENDS_CALLBACK_URL must be a URL")?;
        Ok(Self {
            bind_address: required("DOOR_LEGENDS_BIND_ADDRESS")?
                .parse()
                .context("DOOR_LEGENDS_BIND_ADDRESS must be a socket address")?,
            database_url: required("DATABASE_URL")?,
            tls_certificate: required("DOOR_LEGENDS_TLS_CERTIFICATE")?.into(),
            tls_private_key: required("DOOR_LEGENDS_TLS_PRIVATE_KEY")?.into(),
            release_id,
            cartridge_digest,
            authority: required("DOOR_LEGENDS_AUTHORITY")?,
            platform_grant_key: decode_verifying_key("OGS_PROVIDER_GRANT_PUBLIC_KEY")?,
            platform_message_key: decode_verifying_key("OGS_PROVIDER_MESSAGE_PUBLIC_KEY")?,
            provider_signing_seed: decode_exact("DOOR_LEGENDS_MESSAGE_SIGNING_SEED", 32)?
                .try_into()
                .map_err(|_| anyhow!("provider message seed must be 32 bytes"))?,
            callback_url,
            callback_root_der: decode_exact("DOOR_LEGENDS_CALLBACK_TLS_ROOT_DER", 4096)?,
            callback_socket_override: parse_callback_socket_override()?,
            conformance_reconcile_response_delay: parse_conformance_reconcile_response_delay()?,
        })
    }
}

fn parse_conformance_reconcile_response_delay() -> Result<Duration> {
    #[cfg(feature = "conformance")]
    {
        let milliseconds = env::var("DOOR_LEGENDS_RECONCILE_RESPONSE_DELAY_MS")
            .ok()
            .map(|value| value.parse::<u64>())
            .transpose()
            .context("reconciliation response delay must be milliseconds")?
            .unwrap_or(0);
        if milliseconds > 10_000 {
            return Err(anyhow!(
                "reconciliation response delay exceeds conformance limit"
            ));
        }
        Ok(Duration::from_millis(milliseconds))
    }
    #[cfg(not(feature = "conformance"))]
    {
        if env::var_os("DOOR_LEGENDS_RECONCILE_RESPONSE_DELAY_MS").is_some() {
            Err(anyhow!(
                "DOOR_LEGENDS_RECONCILE_RESPONSE_DELAY_MS requires the conformance feature"
            ))
        } else {
            Ok(Duration::ZERO)
        }
    }
}

fn parse_callback_socket_override() -> Result<Option<SocketAddr>> {
    #[cfg(feature = "conformance")]
    {
        env::var("DOOR_LEGENDS_CALLBACK_SOCKET_OVERRIDE")
            .ok()
            .map(|value| value.parse())
            .transpose()
            .context("callback socket override must be a socket address")
    }
    #[cfg(not(feature = "conformance"))]
    {
        if env::var_os("DOOR_LEGENDS_CALLBACK_SOCKET_OVERRIDE").is_some() {
            Err(anyhow!(
                "DOOR_LEGENDS_CALLBACK_SOCKET_OVERRIDE requires the conformance feature"
            ))
        } else {
            Ok(None)
        }
    }
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn decode_verifying_key(name: &str) -> Result<VerifyingKey> {
    let bytes: [u8; 32] = decode_exact(name, 32)?
        .try_into()
        .map_err(|_| anyhow!("{name} must decode to 32 bytes"))?;
    VerifyingKey::from_bytes(&bytes).with_context(|| format!("{name} is not an Ed25519 key"))
}

fn decode_exact(name: &str, expected: usize) -> Result<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD
        .decode(required(name)?)
        .with_context(|| format!("{name} must be unpadded base64url"))?;
    if expected == 4096 {
        if !(64..=expected).contains(&bytes.len()) {
            return Err(anyhow!("{name} must decode to a bounded DER certificate"));
        }
    } else if bytes.len() != expected {
        return Err(anyhow!("{name} must decode to {expected} bytes"));
    }
    Ok(bytes)
}

fn operation_name(operation: ProviderOperationKind) -> &'static str {
    match operation {
        ProviderOperationKind::Launch => "launch",
        ProviderOperationKind::Command => "command",
        ProviderOperationKind::Reconcile => "reconcile",
    }
}

fn current_unix_seconds() -> Result<i64, ProviderError> {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(|_| ProviderError::Internal)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| ProviderError::Internal)
}

fn database_error(error_value: sqlx::Error) -> ProviderError {
    error!(?error_value, "Door Legends database operation failed");
    ProviderError::Internal
}

fn provider_error(error_value: ProviderError) -> anyhow::Error {
    anyhow!(error_value.code())
}

async fn shutdown_signal() {
    let interrupt = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {},
        () = terminate => {},
    }
}
