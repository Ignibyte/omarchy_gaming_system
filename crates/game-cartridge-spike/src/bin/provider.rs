use std::{collections::HashMap, env, net::SocketAddr, path::Path, sync::Arc};

use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use omarchygs_game_cartridge_spike::{
    ErrorDocument, GrantExpectation, MAX_PROVIDER_BODY_BYTES, MessageExpectation, PLATFORM_KEY_ID,
    PROVIDER_KEY_ID, ProviderCommandRequest, ProviderGrant, ProviderLaunchRequest, ProviderMessage,
    ProviderMessageKind, SignedEnvelope, ViewModel, load_signing_key, load_verifying_key,
    now_unix_seconds, sign_envelope, validate_grant, validate_provider_message, verify_envelope,
};
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

#[derive(Clone)]
struct ProviderState {
    config: Arc<ProviderConfig>,
    data: Arc<Mutex<ProviderData>>,
}

struct ProviderConfig {
    platform_public_key: VerifyingKey,
    provider_private_key: SigningKey,
    provider_id: String,
    game_key: String,
    game_version: u32,
    cartridge_digest: String,
}

#[derive(Default)]
struct ProviderData {
    sessions: HashMap<Uuid, GameSession>,
    receipts: HashMap<Uuid, CommandReceipt>,
    consumed_grants: std::collections::HashSet<Uuid>,
}

struct GameSession {
    platform_session_id: Uuid,
    provider_session_id: Uuid,
    subject: String,
    revision: u64,
    view: ViewModel,
}

struct CommandReceipt {
    platform_session_id: Uuid,
    subject: String,
    expected_revision: u64,
    action_id: String,
    response: SignedEnvelope,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
}

impl ApiError {
    fn unauthorized(code: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
        }
    }

    fn conflict(code: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "provider_internal_error",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorDocument {
                code: self.code.to_owned(),
            }),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let bind_address = required_env("OGS_SPIKE_PROVIDER_BIND")?
        .parse::<SocketAddr>()
        .context("OGS_SPIKE_PROVIDER_BIND must be a socket address")?;
    if !bind_address.ip().is_loopback() {
        bail!("the proof provider may bind only to loopback");
    }
    let config = ProviderConfig {
        platform_public_key: load_verifying_key(Path::new(&required_env(
            "OGS_SPIKE_PLATFORM_PUBLIC_KEY",
        )?))
        .context("failed to load platform public key")?,
        provider_private_key: load_signing_key(Path::new(&required_env(
            "OGS_SPIKE_PROVIDER_PRIVATE_KEY",
        )?))
        .context("failed to load provider private key")?,
        provider_id: required_env("OGS_SPIKE_PROVIDER_ID")?,
        game_key: required_env("OGS_SPIKE_GAME_KEY")?,
        game_version: required_env("OGS_SPIKE_GAME_VERSION")?
            .parse()
            .context("OGS_SPIKE_GAME_VERSION must be an integer")?,
        cartridge_digest: required_env("OGS_SPIKE_CARTRIDGE_DIGEST")?,
    };
    let state = ProviderState {
        config: Arc::new(config),
        data: Arc::new(Mutex::new(ProviderData::default())),
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/launch", post(launch))
        .route("/commands", post(command))
        .layer(DefaultBodyLimit::max(MAX_PROVIDER_BODY_BYTES))
        .with_state(state);
    let listener = TcpListener::bind(bind_address)
        .await
        .with_context(|| format!("failed to bind provider at {bind_address}"))?;
    println!("provider listening at http://{bind_address}");
    axum::serve(listener, app)
        .await
        .context("provider stopped unexpectedly")
}

async fn health() -> &'static str {
    "ok"
}

async fn launch(
    State(state): State<ProviderState>,
    Json(request): Json<ProviderLaunchRequest>,
) -> Result<Json<SignedEnvelope>, ApiError> {
    let grant = validate_signed_grant(&state, &request.grant, "game.launch")?;
    let mut data = state.data.lock().await;
    consume_grant(&mut data, grant.token_id)?;

    let session = match data.sessions.entry(grant.platform_session_id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
            if entry.get().subject != grant.subject {
                return Err(ApiError::conflict("provider_session_conflict"));
            }
            entry.into_mut()
        }
        std::collections::hash_map::Entry::Vacant(entry) => entry.insert(GameSession {
            platform_session_id: grant.platform_session_id,
            provider_session_id: Uuid::new_v4(),
            subject: grant.subject,
            revision: 0,
            view: ViewModel {
                headline: "REMOTE LINK ESTABLISHED".to_owned(),
                board: vec!["·".to_owned(); 9],
                turn: 0,
                status: "ready".to_owned(),
            },
        }),
    };
    let message = provider_message(&state.config, session, ProviderMessageKind::Launch);
    validate_provider_message(
        &message,
        &MessageExpectation {
            kind: ProviderMessageKind::Launch,
            provider_id: &state.config.provider_id,
            game_key: &state.config.game_key,
            game_version: state.config.game_version,
            cartridge_digest: &state.config.cartridge_digest,
            platform_session_id: grant.platform_session_id,
        },
    )
    .map_err(|_| ApiError::internal())?;
    let envelope = sign_envelope(
        &message,
        PROVIDER_KEY_ID,
        &state.config.provider_private_key,
    )
    .map_err(|_| ApiError::internal())?;
    Ok(Json(envelope))
}

async fn command(
    State(state): State<ProviderState>,
    Json(request): Json<ProviderCommandRequest>,
) -> Result<Json<SignedEnvelope>, ApiError> {
    let grant = validate_signed_grant(&state, &request.grant, "game.command")?;
    let mut data = state.data.lock().await;
    consume_grant(&mut data, grant.token_id)?;

    if let Some(receipt) = data.receipts.get(&request.idempotency_key) {
        if receipt.platform_session_id == grant.platform_session_id
            && receipt.subject == grant.subject
            && receipt.expected_revision == request.expected_revision
            && receipt.action_id == request.action_id
        {
            return Ok(Json(receipt.response.clone()));
        }
        return Err(ApiError::conflict("idempotency_key_conflict"));
    }

    let session = data
        .sessions
        .get_mut(&grant.platform_session_id)
        .ok_or_else(|| ApiError::conflict("provider_session_missing"))?;
    if session.platform_session_id != grant.platform_session_id || session.subject != grant.subject
    {
        return Err(ApiError::unauthorized("provider_subject_mismatch"));
    }
    if session.revision != request.expected_revision {
        return Err(ApiError::conflict("provider_revision_conflict"));
    }
    if request.action_id != "advance" {
        return Err(ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "unknown_action",
        });
    }

    session.revision = session
        .revision
        .checked_add(1)
        .ok_or_else(ApiError::internal)?;
    session.view.board[0] = "X".to_owned();
    session.view.turn = 1;
    session.view.status = "move accepted".to_owned();
    let message = provider_message(&state.config, session, ProviderMessageKind::CommandResult);
    validate_provider_message(
        &message,
        &MessageExpectation {
            kind: ProviderMessageKind::CommandResult,
            provider_id: &state.config.provider_id,
            game_key: &state.config.game_key,
            game_version: state.config.game_version,
            cartridge_digest: &state.config.cartridge_digest,
            platform_session_id: grant.platform_session_id,
        },
    )
    .map_err(|_| ApiError::internal())?;
    let response = sign_envelope(
        &message,
        PROVIDER_KEY_ID,
        &state.config.provider_private_key,
    )
    .map_err(|_| ApiError::internal())?;
    data.receipts.insert(
        request.idempotency_key,
        CommandReceipt {
            platform_session_id: grant.platform_session_id,
            subject: grant.subject,
            expected_revision: request.expected_revision,
            action_id: request.action_id,
            response: response.clone(),
        },
    );
    Ok(Json(response))
}

fn validate_signed_grant(
    state: &ProviderState,
    envelope: &SignedEnvelope,
    required_scope: &str,
) -> Result<ProviderGrant, ApiError> {
    let grant = verify_envelope::<ProviderGrant>(
        envelope,
        PLATFORM_KEY_ID,
        &state.config.platform_public_key,
    )
    .map_err(|_| ApiError::unauthorized("invalid_platform_signature"))?;
    let expectation = GrantExpectation {
        provider_id: &state.config.provider_id,
        game_key: &state.config.game_key,
        game_version: state.config.game_version,
        cartridge_digest: &state.config.cartridge_digest,
        platform_session_id: grant.platform_session_id,
        required_scope,
    };
    let now = now_unix_seconds().map_err(|_| ApiError::internal())?;
    validate_grant(&grant, &expectation, now)
        .map_err(|_| ApiError::unauthorized("invalid_provider_grant"))?;
    Ok(grant)
}

fn consume_grant(data: &mut ProviderData, token_id: Uuid) -> Result<(), ApiError> {
    if data.consumed_grants.insert(token_id) {
        Ok(())
    } else {
        Err(ApiError::unauthorized("provider_grant_replayed"))
    }
}

fn provider_message(
    config: &ProviderConfig,
    session: &GameSession,
    kind: ProviderMessageKind,
) -> ProviderMessage {
    ProviderMessage {
        kind,
        provider_id: config.provider_id.clone(),
        game_key: config.game_key.clone(),
        game_version: config.game_version,
        cartridge_digest: config.cartridge_digest.clone(),
        platform_session_id: session.platform_session_id,
        provider_session_id: session.provider_session_id,
        event_id: Uuid::new_v4(),
        revision: session.revision,
        view: session.view.clone(),
    }
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}
