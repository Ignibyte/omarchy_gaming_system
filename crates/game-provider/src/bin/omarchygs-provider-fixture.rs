//! Separate-process TLS provider used only by the opt-in conformance corpus.

use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, OriginalUri, State},
    http::{HeaderMap, HeaderValue, Response, StatusCode, header::LOCATION},
    routing::post,
};
use axum_server::tls_rustls::RustlsConfig;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::VerifyingKey;
use omarchy_game_provider::{
    ProviderError,
    protocol::{
        GrantExpectation, HttpMessageSigner, ProviderCompatibilityOffer,
        ProviderCompatibilitySelection, ProviderEvent, ProviderEventKind,
        ProviderOperationDisposition, ProviderOperationKind, ProviderOperationRequest,
        ProviderOperationResponse, ProviderSessionStatus, RequestSignatureContext,
        SignatureHeaders, parse_authenticated_json, verify_grant, verify_request_signature,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use uuid::Uuid;

const MAX_CONFIG_BYTES: usize = 256 * 1024;
const MAX_FIXTURE_BODY_BYTES: usize = 128 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureConfig {
    listen: SocketAddr,
    certificate_der_base64: String,
    private_key_der_base64: String,
    provider_id: String,
    release_id: Uuid,
    game_key: String,
    rules_version: u32,
    cartridge_digest: String,
    endpoint_authority: String,
    endpoint_base_path: String,
    platform_grant_key_id: String,
    platform_grant_public_key_base64: String,
    platform_message_key_id: String,
    platform_message_public_key_base64: String,
    provider_message_key_id: String,
    provider_message_signing_seed_base64: String,
    state_path: PathBuf,
    commit_delay_ms: u64,
    #[serde(default)]
    compatibility_delay_ms: u64,
    #[serde(default)]
    compatibility_fault: Option<String>,
}

struct AppState {
    config: FixtureConfig,
    platform_grant_key: VerifyingKey,
    platform_message_key: VerifyingKey,
    provider_signer: HttpMessageSigner,
    durable: Mutex<DurableState>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DurableState {
    consumed_grants: BTreeSet<Uuid>,
    sessions: BTreeMap<Uuid, ProviderSession>,
    receipts: BTreeMap<Uuid, OperationReceipt>,
    last_event: Option<EventRecord>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProviderSession {
    subject: String,
    revision: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OperationReceipt {
    platform_session_id: Uuid,
    subject: String,
    operation: ProviderOperationKind,
    expected_revision: u64,
    payload: Value,
    response: ProviderOperationResponse,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EventRecord {
    body_base64: String,
    headers: BTreeMap<String, String>,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("provider fixture failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let config_path = std::env::args_os()
        .nth(1)
        .ok_or("missing fixture configuration")?;
    let config_bytes = tokio::fs::read(config_path).await?;
    if config_bytes.is_empty() || config_bytes.len() > MAX_CONFIG_BYTES {
        return Err("invalid fixture configuration size".into());
    }
    let config: FixtureConfig = serde_json::from_slice(&config_bytes)?;
    let certificate_der = decode_bounded(&config.certificate_der_base64, 32 * 1024)?;
    let private_key_der = decode_bounded(&config.private_key_der_base64, 32 * 1024)?;
    let platform_grant_key = decode_verifying_key(&config.platform_grant_public_key_base64)?;
    let platform_message_key = decode_verifying_key(&config.platform_message_public_key_base64)?;
    let provider_seed = decode_seed(&config.provider_message_signing_seed_base64)?;
    let durable = load_state(&config.state_path).await?;
    let listen = config.listen;
    let app_state = Arc::new(AppState {
        provider_signer: HttpMessageSigner::new(&config.provider_message_key_id, provider_seed)?,
        config,
        platform_grant_key,
        platform_message_key,
        durable: Mutex::new(durable),
    });
    let router = Router::new()
        .route("/omarchygs/provider/v1/compatibility", post(compatibility))
        .route("/omarchygs/provider/v1/launch", post(operation))
        .route("/omarchygs/provider/v1/commands", post(operation))
        .route("/omarchygs/provider/v1/reconcile", post(operation))
        .layer(DefaultBodyLimit::max(MAX_FIXTURE_BODY_BYTES))
        .with_state(app_state);
    let tls = RustlsConfig::from_der(vec![certificate_der], private_key_der).await?;
    axum_server::bind_rustls(listen, tls)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

async fn compatibility(
    State(state): State<Arc<AppState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    if state.config.compatibility_delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(state.config.compatibility_delay_ms)).await;
    }
    match handle_compatibility(&state, uri.path(), &headers, &body) {
        Ok(response) => response,
        Err(error) => {
            eprintln!("fixture rejected compatibility request: {}", error.code());
            safe_error_response(error)
        }
    }
}

fn handle_compatibility(
    state: &AppState,
    path: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<Response<Body>, ProviderError> {
    if path != "/omarchygs/provider/v1/compatibility" {
        return Err(ProviderError::ProtocolRejected);
    }
    let signed_headers = SignatureHeaders::from_header_map(headers)?;
    let message_id = signed_headers
        .message_id
        .parse::<Uuid>()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let context = RequestSignatureContext {
        method: "POST",
        authority: &state.config.endpoint_authority,
        path,
        provider_id: &state.config.provider_id,
        release_id: state.config.release_id,
        message_id,
    };
    verify_request_signature(
        &signed_headers,
        &context,
        body,
        &state.platform_message_key,
        &state.config.platform_message_key_id,
        unix_seconds()?,
    )?;
    let offer: ProviderCompatibilityOffer = parse_authenticated_json(body, MAX_FIXTURE_BODY_BYTES)?;
    offer.validate()?;
    if offer.message_id != message_id
        || offer.provider_id != state.config.provider_id
        || offer.release_id != state.config.release_id
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let mut selection = ProviderCompatibilitySelection::current(&offer, Uuid::new_v4())?;
    if state.config.compatibility_fault.as_deref() == Some("strip_capability") {
        selection.selected.capabilities.pop();
    }
    let response_body = serde_json::to_vec(&selection).map_err(|_| ProviderError::Internal)?;
    let response_headers = state.provider_signer.sign_response(
        StatusCode::OK.as_u16(),
        &context,
        selection.message_id,
        &response_body,
        unix_seconds()?,
        &format!("n-{}", Uuid::new_v4()),
    )?;
    let mut response = Response::builder().status(StatusCode::OK);
    for (name, value) in signature_header_values(&response_headers) {
        response = response.header(name, value);
    }
    response
        .body(Body::from(response_body))
        .map_err(|_| ProviderError::Internal)
}

async fn operation(
    State(state): State<Arc<AppState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    match handle_operation(&state, uri.path(), &headers, &body).await {
        Ok(response) => response,
        Err(error) => {
            eprintln!("fixture rejected provider request: {}", error.code());
            safe_error_response(error)
        }
    }
}

async fn handle_operation(
    state: &Arc<AppState>,
    path: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<Response<Body>, ProviderError> {
    let operation = operation_from_path(path)?;
    let signed_headers = SignatureHeaders::from_header_map(headers)?;
    let message_id = signed_headers
        .message_id
        .parse::<Uuid>()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    let context = RequestSignatureContext {
        method: "POST",
        authority: &state.config.endpoint_authority,
        path,
        provider_id: &state.config.provider_id,
        release_id: state.config.release_id,
        message_id,
    };
    verify_request_signature(
        &signed_headers,
        &context,
        body,
        &state.platform_message_key,
        &state.config.platform_message_key_id,
        unix_seconds()?,
    )?;
    let request: ProviderOperationRequest = parse_authenticated_json(body, MAX_FIXTURE_BODY_BYTES)?;
    request
        .validate()
        .map_err(|_| ProviderError::ProtocolRejected)?;
    if request.operation != operation
        || request.message_id != message_id
        || request.provider_id != state.config.provider_id
        || request.release_id != state.config.release_id
        || request.game_key != state.config.game_key
        || request.rules_version != state.config.rules_version
        || request.cartridge_digest != state.config.cartridge_digest
    {
        return Err(ProviderError::ProtocolRejected);
    }
    let claims = verify_grant(
        &request.grant,
        &state.platform_grant_key,
        &GrantExpectation {
            key_id: &state.config.platform_grant_key_id,
            provider_id: &state.config.provider_id,
            release_id: state.config.release_id,
            game_key: &state.config.game_key,
            rules_version: state.config.rules_version,
            cartridge_digest: &state.config.cartridge_digest,
            platform_session_id: request.platform_session_id,
            subject: &request.subject,
            scope: operation.scope(),
            compatibility: &request.compatibility,
        },
        unix_seconds()?,
    )?;

    let fault = request
        .payload
        .get("fault")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if fault == "redirect" {
        return Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "https://redirect.invalid/")
            .body(Body::from("redirect denied"))
            .map_err(|_| ProviderError::Internal);
    }
    if fault == "oversized" {
        return Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(vec![b'x'; MAX_FIXTURE_BODY_BYTES]))
            .map_err(|_| ProviderError::Internal);
    }

    let (response, delay_after_commit, replayed_receipt) = {
        let mut durable = state.durable.lock().await;
        if !durable.consumed_grants.insert(claims.token_id) {
            return Err(ProviderError::Conflict);
        }
        if let Some(receipt) = durable.receipts.get(&request.idempotency_key) {
            if receipt.platform_session_id != request.platform_session_id
                || receipt.subject != request.subject
                || receipt.operation != request.operation
                || receipt.expected_revision != request.expected_revision
                || receipt.payload != request.payload
            {
                return Err(ProviderError::Conflict);
            }
            let response = receipt.response.clone();
            persist_state(&state.config.state_path, &durable).await?;
            (response, false, true)
        } else {
            let response = apply_operation(state, &mut durable, &request)?;
            durable.receipts.insert(
                request.idempotency_key,
                OperationReceipt {
                    platform_session_id: request.platform_session_id,
                    subject: request.subject.clone(),
                    operation: request.operation,
                    expected_revision: request.expected_revision,
                    payload: request.payload.clone(),
                    response: response.clone(),
                },
            );
            if request.operation == ProviderOperationKind::Command
                && response.disposition == ProviderOperationDisposition::Applied
            {
                durable.last_event = Some(create_event_record(state, &request, response.revision)?);
            }
            persist_state(&state.config.state_path, &durable).await?;
            (response, fault == "commit_then_timeout", false)
        }
    };
    if delay_after_commit {
        tokio::time::sleep(Duration::from_millis(state.config.commit_delay_ms)).await;
    }
    signed_response(
        state,
        &context,
        response,
        fault == "bad_signature" && !replayed_receipt,
    )
}

fn apply_operation(
    state: &AppState,
    durable: &mut DurableState,
    request: &ProviderOperationRequest,
) -> Result<ProviderOperationResponse, ProviderError> {
    let (revision, disposition) = match request.operation {
        ProviderOperationKind::Launch => {
            if durable.sessions.contains_key(&request.platform_session_id) {
                return Err(ProviderError::Conflict);
            }
            durable.sessions.insert(
                request.platform_session_id,
                ProviderSession {
                    subject: request.subject.clone(),
                    revision: 0,
                },
            );
            (0, ProviderOperationDisposition::Applied)
        }
        ProviderOperationKind::Command => {
            let session = durable
                .sessions
                .get_mut(&request.platform_session_id)
                .ok_or(ProviderError::NotFound)?;
            if session.subject != request.subject {
                return Err(ProviderError::ProtocolRejected);
            }
            if session.revision != request.expected_revision {
                (
                    session.revision,
                    ProviderOperationDisposition::RevisionConflict,
                )
            } else {
                session.revision = session
                    .revision
                    .checked_add(1)
                    .ok_or(ProviderError::Internal)?;
                (session.revision, ProviderOperationDisposition::Applied)
            }
        }
        ProviderOperationKind::Reconcile => {
            let session = durable
                .sessions
                .get(&request.platform_session_id)
                .ok_or(ProviderError::NotFound)?;
            if session.subject != request.subject {
                return Err(ProviderError::ProtocolRejected);
            }
            (session.revision, ProviderOperationDisposition::Applied)
        }
    };
    Ok(ProviderOperationResponse::new(
        state.config.provider_id.clone(),
        state.config.release_id,
        state.config.game_key.clone(),
        state.config.rules_version,
        state.config.cartridge_digest.clone(),
        request.platform_session_id,
        request.subject.clone(),
        Uuid::new_v4(),
        request.idempotency_key,
        revision,
        disposition,
        ProviderSessionStatus::Active,
        request.compatibility.clone(),
        json!({"turn": revision}),
    ))
}

fn create_event_record(
    state: &AppState,
    request: &ProviderOperationRequest,
    revision: u64,
) -> Result<EventRecord, ProviderError> {
    let message_id = Uuid::new_v4();
    let event = ProviderEvent::new(
        state.config.provider_id.clone(),
        state.config.release_id,
        state.config.game_key.clone(),
        state.config.rules_version,
        state.config.cartridge_digest.clone(),
        request.platform_session_id,
        request.subject.clone(),
        message_id,
        Uuid::new_v4(),
        revision,
        ProviderEventKind::TurnReady,
        request.compatibility.clone(),
        json!({"turn": revision}),
    );
    event.validate()?;
    let body = serde_json::to_vec(&event).map_err(|_| ProviderError::Internal)?;
    let event_path = format!("{}events", state.config.endpoint_base_path);
    let context = RequestSignatureContext {
        method: "POST",
        authority: &state.config.endpoint_authority,
        path: &event_path,
        provider_id: &state.config.provider_id,
        release_id: state.config.release_id,
        message_id,
    };
    let headers = state.provider_signer.sign_request(
        &context,
        &body,
        unix_seconds()?,
        &format!("n-{}", Uuid::new_v4()),
    )?;
    Ok(EventRecord {
        body_base64: STANDARD.encode(body),
        headers: signature_header_values(&headers),
    })
}

fn signed_response(
    state: &AppState,
    context: &RequestSignatureContext<'_>,
    response: ProviderOperationResponse,
    corrupt_signature: bool,
) -> Result<Response<Body>, ProviderError> {
    let body = serde_json::to_vec(&response).map_err(|_| ProviderError::Internal)?;
    let mut headers = state.provider_signer.sign_response(
        StatusCode::OK.as_u16(),
        context,
        response.message_id,
        &body,
        unix_seconds()?,
        &format!("n-{}", Uuid::new_v4()),
    )?;
    if corrupt_signature {
        headers.signature.push('x');
    }
    let mut outgoing = Response::builder().status(StatusCode::OK);
    for (name, value) in signature_header_values(&headers) {
        outgoing = outgoing.header(name, value);
    }
    outgoing
        .body(Body::from(body))
        .map_err(|_| ProviderError::Internal)
}

fn operation_from_path(path: &str) -> Result<ProviderOperationKind, ProviderError> {
    match path {
        "/omarchygs/provider/v1/launch" => Ok(ProviderOperationKind::Launch),
        "/omarchygs/provider/v1/commands" => Ok(ProviderOperationKind::Command),
        "/omarchygs/provider/v1/reconcile" => Ok(ProviderOperationKind::Reconcile),
        _ => Err(ProviderError::ProtocolRejected),
    }
}

fn signature_header_values(headers: &SignatureHeaders) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("x-ogs-provider".to_owned(), headers.provider_id.clone()),
        ("x-ogs-release".to_owned(), headers.release_id.clone()),
        ("x-ogs-message-id".to_owned(), headers.message_id.clone()),
        ("content-digest".to_owned(), headers.content_digest.clone()),
        ("content-type".to_owned(), headers.content_type.clone()),
        (
            "signature-input".to_owned(),
            headers.signature_input.clone(),
        ),
        ("signature".to_owned(), headers.signature.clone()),
    ])
}

fn safe_error_response(error: ProviderError) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("content-type", HeaderValue::from_static("text/plain"))
        .body(Body::from(error.code()))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

async fn load_state(path: &Path) -> Result<DurableState, Box<dyn std::error::Error>> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(DurableState::default()),
        Err(error) => Err(error.into()),
    }
}

async fn persist_state(path: &Path, state: &DurableState) -> Result<(), ProviderError> {
    let bytes = serde_json::to_vec(state).map_err(|_| ProviderError::Internal)?;
    let temporary = path.with_extension("tmp");
    tokio::fs::write(&temporary, bytes)
        .await
        .map_err(|_| ProviderError::Internal)?;
    tokio::fs::rename(temporary, path)
        .await
        .map_err(|_| ProviderError::Internal)
}

fn decode_bounded(value: &str, limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = STANDARD.decode(value)?;
    if bytes.is_empty() || bytes.len() > limit {
        return Err("invalid encoded fixture material".into());
    }
    Ok(bytes)
}

fn decode_seed(value: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    decode_bounded(value, 32)?
        .try_into()
        .map_err(|_| "invalid fixture signing seed".into())
}

fn decode_verifying_key(value: &str) -> Result<VerifyingKey, Box<dyn std::error::Error>> {
    let bytes: [u8; 32] = decode_bounded(value, 32)?
        .try_into()
        .map_err(|_| "invalid fixture public key")?;
    Ok(VerifyingKey::from_bytes(&bytes)?)
}

fn unix_seconds() -> Result<i64, ProviderError> {
    i64::try_from(
        SystemTime::UNIX_EPOCH
            .elapsed()
            .map_err(|_| ProviderError::Internal)?
            .as_secs(),
    )
    .map_err(|_| ProviderError::Internal)
}
