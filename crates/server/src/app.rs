use std::sync::Arc;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State, WebSocketUpgrade},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE, HOST, WWW_AUTHENTICATE},
    },
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use omarchy_game_runtime::{GameManifest, GameRegistry};

use crate::accounts::{self, RegistrationError, RegistrationInput, RegistrationOutcome};
use crate::challenges::{
    self, ChallengeDirection, ChallengeError, ChallengeOutcome, CreateChallengeInput, GameChallenge,
};
use crate::connections::{
    self, Connection, ConnectionError, ConnectionRequest, PersonaBlock, ResourceOutcome,
};
use crate::games::{
    self, GameCommandInput, GameCommandResult, GameError, GameParticipant, GameSession,
    GameSessionStartOutcome, StartGameSessionInput,
};
use crate::inboxes::{
    self, ConversationSummary, InboxError, InboxMessage, InboxMessageContent, SystemMessage,
};
use crate::mfa::{
    self, BeginEnrollmentInput, ConfirmEnrollmentInput, DisableMfaInput, MfaCipher, MfaError,
};
use crate::personas::{self, CreatePersonaInput, Persona, PersonaError, UpdatePersonaInput};
use crate::provider_games::{self, CallbackApplyOutcome, ProviderRuntime};
use crate::reports::{self, CreateReportInput, PlayerReportReceipt, ReportError, ReportOutcome};
use crate::server_discovery;
use crate::sessions::{self, CreateSessionInput, DeviceSession, SessionCreation, SessionError};
use crate::sync::{self, SyncError, SyncEvent, SyncEventKind, SyncHub};
use omarchy_gaming_system_server::cartridge_catalog::{self, CatalogError, PlayerCartridgeRelease};
use omarchy_gaming_system_server::cartridge_distribution::{
    self, CartridgeDistributionRuntime, DistributionError,
};
use omarchy_gaming_system_server::session_cartridges::{
    self, SessionCartridgeError, SessionCartridgePresentation,
};

const SYNC_SOCKET_MAX_CLIENT_BYTES: usize = 1024;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    mfa_cipher: MfaCipher,
    sync_hub: SyncHub,
    game_registry: GameRegistry,
    provider_runtime: Option<ProviderRuntime>,
    cartridge_distribution: Option<CartridgeDistributionRuntime>,
    server_name: Arc<str>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HealthResponse {
    service: &'static str,
    version: &'static str,
    status: &'static str,
    database: &'static str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistrationRequest {
    invite_code: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct RegistrationResponse {
    id: String,
    username: String,
}

#[derive(Deserialize)]
struct CreateSessionRequest {
    username: String,
    password: String,
    device_name: String,
}

#[derive(Serialize)]
struct CreatedSessionResponse {
    token: String,
    session: DeviceSessionResponse,
}

#[derive(Serialize)]
struct MfaChallengeResponse {
    mfa_required: bool,
    challenge_token: String,
    expires_at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompleteMfaSessionRequest {
    challenge_token: String,
    code: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BeginMfaEnrollmentRequest {
    password: String,
}

#[derive(Serialize)]
struct MfaEnrollmentResponse {
    secret: String,
    provisioning_uri: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfirmMfaEnrollmentRequest {
    code: String,
}

#[derive(Serialize)]
struct ConfirmedMfaEnrollmentResponse {
    recovery_codes: Vec<String>,
}

#[derive(Serialize)]
struct MfaStatusResponse {
    enabled: bool,
    recovery_codes_remaining: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DisableMfaRequest {
    password: String,
    code: String,
}

#[derive(Serialize)]
struct SessionListResponse {
    sessions: Vec<DeviceSessionResponse>,
}

#[derive(Serialize)]
struct DeviceSessionResponse {
    id: String,
    device_name: String,
    created_at: String,
    last_used_at: String,
    expires_at: String,
    revoked_at: Option<String>,
    current: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatePersonaRequest {
    handle: String,
    display_name: String,
    #[serde(default)]
    bio: String,
    #[serde(default)]
    status_message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdatePersonaRequest {
    handle: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    status_message: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateReportRequest {
    idempotency_key: String,
    subject_persona_id: String,
    category: String,
    detail: String,
}

#[derive(Serialize)]
struct PlayerReportReceiptResponse {
    id: String,
    idempotency_key: String,
    status: String,
    created_at: String,
}

#[derive(Serialize)]
struct PersonaListResponse {
    personas: Vec<PersonaResponse>,
}

#[derive(Serialize)]
struct PersonaResponse {
    id: String,
    handle: String,
    display_name: String,
    bio: String,
    status_message: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ConnectionRequestInventoryResponse {
    incoming: Vec<ConnectionRequestResponse>,
    outgoing: Vec<ConnectionRequestResponse>,
}

#[derive(Serialize)]
struct ConnectionRequestResponse {
    persona: PersonaResponse,
    created_at: String,
}

#[derive(Serialize)]
struct ConnectionListResponse {
    connections: Vec<ConnectionResponse>,
}

#[derive(Serialize)]
struct ConnectionResponse {
    persona: PersonaResponse,
    connected_at: String,
}

#[derive(Serialize)]
struct BlockListResponse {
    blocks: Vec<BlockResponse>,
}

#[derive(Serialize)]
struct BlockResponse {
    persona: PersonaResponse,
    created_at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConversationListQuery {
    limit: Option<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MessageHistoryQuery {
    before: Option<i64>,
    limit: Option<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SendMessageRequest {
    body: String,
}

#[derive(Serialize)]
struct ConversationListResponse {
    conversations: Vec<ConversationResponse>,
}

#[derive(Serialize)]
struct ConversationResponse {
    id: String,
    other_persona: PersonaResponse,
    unread_count: i64,
    latest_message: Option<MessageResponse>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct MessageListResponse {
    messages: Vec<MessageResponse>,
    next_before: Option<i64>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MessageResponse {
    User {
        id: String,
        sequence: i64,
        sender: PersonaResponse,
        body: String,
        created_at: String,
    },
    System {
        id: String,
        sequence: i64,
        system: SystemMessageResponse,
        created_at: String,
    },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SystemMessageResponse {
    ConnectionAccepted {
        actor: PersonaResponse,
    },
    GameChallengeCreated {
        actor: PersonaResponse,
        challenge_id: String,
    },
    GameChallengeAccepted {
        actor: PersonaResponse,
        challenge_id: String,
        game_session_id: String,
    },
    GameChallengeDeclined {
        actor: PersonaResponse,
        challenge_id: String,
    },
    GameChallengeCancelled {
        actor: PersonaResponse,
        challenge_id: String,
    },
}

#[derive(Serialize)]
struct ReadReceiptResponse {
    through_message_id: String,
    unread_count: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyncQuery {
    after: Option<i64>,
    limit: Option<u16>,
}

#[derive(Serialize)]
struct SyncPageResponse {
    events: Vec<SyncEventResponse>,
    next_cursor: i64,
    has_more: bool,
    reset_required: bool,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SyncEventResponse {
    #[serde(rename = "connection_requests_changed")]
    ConnectionRequests { cursor: i64, created_at: String },
    #[serde(rename = "connections_changed")]
    Connections { cursor: i64, created_at: String },
    #[serde(rename = "blocks_changed")]
    Blocks { cursor: i64, created_at: String },
    #[serde(rename = "conversation_changed")]
    Conversation {
        cursor: i64,
        conversation_id: String,
        created_at: String,
    },
    #[serde(rename = "game_session_changed")]
    GameSession {
        cursor: i64,
        game_session_id: String,
        created_at: String,
    },
    #[serde(rename = "game_challenge_changed")]
    GameChallenge {
        cursor: i64,
        game_challenge_id: String,
        created_at: String,
    },
}

#[derive(Serialize)]
struct GameCatalogResponse {
    games: Vec<GameManifestResponse>,
}

#[derive(Serialize)]
struct CartridgeCatalogResponse {
    cartridges: Vec<PlayerCartridgeRelease>,
}

#[derive(Serialize)]
struct GameManifestResponse {
    key: String,
    version: u32,
    display_name: String,
    min_human_players: u8,
    max_human_players: u8,
    authority: &'static str,
    provider_release_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GameSessionQuery {
    limit: Option<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartGameSessionRequest {
    idempotency_key: String,
    game_key: String,
    game_version: u32,
}

#[derive(Serialize)]
struct GameSessionListResponse {
    sessions: Vec<GameSessionResponse>,
}

#[derive(Serialize)]
struct GameSessionResponse {
    id: String,
    game_key: String,
    game_version: u32,
    revision: i64,
    status: String,
    state: Option<serde_json::Value>,
    authority: String,
    provider_release_id: Option<String>,
    availability: Option<String>,
    presentation: Option<SessionCartridgePresentation>,
    result: Option<GameResultResponse>,
    participants: Vec<GameParticipantResponse>,
    completed_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct GameResultResponse {
    outcome: String,
    public_summary: serde_json::Value,
    provider_revision: i64,
    projected_at: String,
}

#[derive(Serialize)]
struct GameParticipantResponse {
    seat: u8,
    persona: PersonaResponse,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GameCommandRequest {
    idempotency_key: String,
    expected_revision: i64,
    command: serde_json::Value,
}

#[derive(Serialize)]
struct GameCommandResponse {
    game_session_id: String,
    revision: i64,
    status: String,
    state: serde_json::Value,
    authority: String,
    provider_release_id: Option<String>,
    availability: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CartridgeActionRequest {
    idempotency_key: String,
    expected_revision: i64,
    archive_sha256: String,
    #[serde(default)]
    screen_id: Option<String>,
    action: String,
    payload: serde_json::Value,
}

#[derive(Serialize)]
struct CartridgeActionResponse {
    game_session_id: String,
    revision: i64,
    status: String,
    state: serde_json::Value,
    authority: String,
    provider_release_id: Option<String>,
    availability: Option<String>,
    archive_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReconcileGameRequest {
    idempotency_key: String,
    expected_revision: i64,
}

#[derive(Serialize)]
struct ProviderAchievementListResponse {
    achievements: Vec<provider_games::ProviderAchievement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateGameChallengeRequest {
    idempotency_key: String,
    challenged_persona_id: String,
    game_key: String,
    game_version: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GameChallengeQuery {
    before: Option<String>,
    limit: Option<u16>,
}

#[derive(Serialize)]
struct GameChallengeListResponse {
    challenges: Vec<GameChallengeResponse>,
    next_before: Option<String>,
}

#[derive(Serialize)]
struct GameChallengeResponse {
    id: String,
    game_key: String,
    game_version: u32,
    direction: &'static str,
    status: String,
    challenger: PersonaResponse,
    challenged: PersonaResponse,
    game_session_id: Option<String>,
    expires_at: String,
    resolved_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug)]
enum ApiError {
    Registration(RegistrationError),
    Session(SessionError),
    Mfa(MfaError),
    Persona(PersonaError),
    Report(ReportError),
    Connection(ConnectionError),
    Challenge(ChallengeError),
    Catalog(CatalogError),
    Distribution(DistributionError),
    Inbox(InboxError),
    Game(GameError),
    Sync(SyncError),
}

#[cfg(test)]
pub fn router(pool: PgPool, mfa_cipher: MfaCipher) -> Router {
    router_with_runtime(pool, mfa_cipher, SyncHub::new(), GameRegistry::empty())
}

#[cfg(test)]
pub(crate) fn router_with_sync_hub(
    pool: PgPool,
    mfa_cipher: MfaCipher,
    sync_hub: SyncHub,
) -> Router {
    router_with_runtime(pool, mfa_cipher, sync_hub, GameRegistry::empty())
}

#[cfg(test)]
pub(crate) fn router_with_game_registry(
    pool: PgPool,
    mfa_cipher: MfaCipher,
    game_registry: GameRegistry,
) -> Router {
    router_with_runtime(pool, mfa_cipher, SyncHub::new(), game_registry)
}

#[cfg(test)]
pub(crate) fn router_with_runtime(
    pool: PgPool,
    mfa_cipher: MfaCipher,
    sync_hub: SyncHub,
    game_registry: GameRegistry,
) -> Router {
    router_with_provider_runtime(
        pool,
        mfa_cipher,
        sync_hub,
        game_registry,
        None,
        Arc::from(crate::config::DEFAULT_SERVER_NAME),
    )
}

#[cfg(test)]
pub(crate) fn router_with_server_name(
    pool: PgPool,
    mfa_cipher: MfaCipher,
    server_name: Arc<str>,
) -> Router {
    router_with_provider_runtime(
        pool,
        mfa_cipher,
        SyncHub::new(),
        GameRegistry::empty(),
        None,
        server_name,
    )
}

#[cfg(test)]
pub(crate) fn router_with_provider_runtime(
    pool: PgPool,
    mfa_cipher: MfaCipher,
    sync_hub: SyncHub,
    game_registry: GameRegistry,
    provider_runtime: Option<ProviderRuntime>,
    server_name: Arc<str>,
) -> Router {
    router_with_runtimes(
        pool,
        mfa_cipher,
        sync_hub,
        game_registry,
        provider_runtime,
        None,
        server_name,
    )
}

pub(crate) fn router_with_runtimes(
    pool: PgPool,
    mfa_cipher: MfaCipher,
    sync_hub: SyncHub,
    game_registry: GameRegistry,
    provider_runtime: Option<ProviderRuntime>,
    cartridge_distribution: Option<CartridgeDistributionRuntime>,
    server_name: Arc<str>,
) -> Router {
    let discovery_routes = Router::new()
        .route("/.well-known/omarchygs", get(discover_server))
        .layer(middleware::map_response(inbox_no_store));
    let registration_routes = Router::new()
        .route(
            "/v1/accounts",
            post(register_account).layer(DefaultBodyLimit::max(1024)),
        )
        .layer(middleware::map_response(inbox_no_store));
    let mut cartridge_routes = Router::new().route("/v1/cartridges", get(list_cartridges));
    if cartridge_distribution.is_some() {
        cartridge_routes = cartridge_routes.route(
            "/v1/cartridges/{game_key}/{archive_sha256}/acquisition",
            get(acquire_cartridge),
        );
    }
    let cartridge_routes = cartridge_routes.layer(middleware::map_response(inbox_no_store));
    let report_routes = Router::new()
        .route(
            "/v1/personas/{persona_id}/reports",
            post(create_report).layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .layer(middleware::map_response(inbox_no_store));
    let inbox_routes = Router::new()
        .route(
            "/v1/personas/{persona_id}/conversations",
            get(list_conversations),
        )
        .route(
            "/v1/personas/{persona_id}/conversations/{conversation_id}/messages",
            get(list_messages)
                .post(send_message)
                .layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .route(
            "/v1/personas/{persona_id}/conversations/{conversation_id}/read/{message_id}",
            put(mark_conversation_read),
        )
        .layer(middleware::map_response(inbox_no_store));
    let sync_routes = Router::new()
        .route("/v1/personas/{persona_id}/sync", get(list_sync_events))
        .route("/v1/personas/{persona_id}/sync/live", get(open_sync_socket))
        .layer(middleware::map_response(inbox_no_store));
    let mut game_routes = Router::new()
        .route(
            "/v1/personas/{persona_id}/game-sessions",
            get(list_game_sessions)
                .post(start_game_session)
                .layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .route(
            "/v1/personas/{persona_id}/game-sessions/{game_session_id}",
            get(get_game_session),
        )
        .route(
            "/v1/personas/{persona_id}/game-sessions/{game_session_id}/commands",
            post(apply_game_command).layer(DefaultBodyLimit::max(32 * 1024)),
        )
        .route(
            "/v1/personas/{persona_id}/game-sessions/{game_session_id}/cartridge-actions",
            post(apply_cartridge_action).layer(DefaultBodyLimit::max(32 * 1024)),
        )
        .route(
            "/v1/personas/{persona_id}/game-sessions/{game_session_id}/reconcile",
            post(reconcile_game_session).layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .route(
            "/v1/personas/{persona_id}/achievements",
            get(list_provider_achievements),
        );
    if cartridge_distribution.is_some() {
        game_routes = game_routes.route(
            "/v1/personas/{persona_id}/game-sessions/{game_session_id}/cartridge-acquisition",
            get(acquire_session_cartridge),
        );
    }
    let game_routes = game_routes.layer(middleware::map_response(inbox_no_store));
    let challenge_routes = Router::new()
        .route(
            "/v1/personas/{persona_id}/game-challenges",
            get(list_game_challenges)
                .post(create_game_challenge)
                .layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .route(
            "/v1/personas/{persona_id}/game-challenges/{challenge_id}",
            get(get_game_challenge).delete(cancel_game_challenge),
        )
        .route(
            "/v1/personas/{persona_id}/game-challenges/{challenge_id}/accept",
            put(accept_game_challenge),
        )
        .route(
            "/v1/personas/{persona_id}/game-challenges/{challenge_id}/decline",
            put(decline_game_challenge),
        )
        .layer(middleware::map_response(inbox_no_store));

    Router::new()
        .route("/health", get(health))
        .merge(discovery_routes)
        .route("/v1/games", get(list_games))
        .merge(cartridge_routes)
        .route(
            "/v1/provider-events/{release_id}",
            post(receive_provider_event).layer(DefaultBodyLimit::max(512 * 1024)),
        )
        .merge(registration_routes)
        .route(
            "/v1/sessions",
            get(list_sessions)
                .post(create_session)
                .layer(DefaultBodyLimit::max(1024)),
        )
        .route("/v1/sessions/{session_id}", delete(revoke_session))
        .route(
            "/v1/sessions/mfa",
            post(complete_mfa_session).layer(DefaultBodyLimit::max(1024)),
        )
        .route(
            "/v1/account/mfa",
            get(get_mfa_status)
                .post(begin_mfa_enrollment)
                .delete(disable_mfa)
                .layer(DefaultBodyLimit::max(1024)),
        )
        .route(
            "/v1/account/mfa/confirm",
            post(confirm_mfa_enrollment).layer(DefaultBodyLimit::max(1024)),
        )
        .route(
            "/v1/personas",
            get(list_personas)
                .post(create_persona)
                .layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .route(
            "/v1/personas/{persona_id}",
            patch(update_persona).layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .route(
            "/v1/personas/by-handle/{handle}",
            get(get_persona_by_handle),
        )
        .route(
            "/v1/personas/{persona_id}/connection-requests",
            get(list_connection_requests),
        )
        .route(
            "/v1/personas/{persona_id}/connection-requests/{other_persona_id}",
            put(request_connection),
        )
        .route(
            "/v1/personas/{persona_id}/connections",
            get(list_connections),
        )
        .route(
            "/v1/personas/{persona_id}/connections/{other_persona_id}",
            put(accept_connection).delete(remove_connection),
        )
        .route("/v1/personas/{persona_id}/blocks", get(list_blocks))
        .route(
            "/v1/personas/{persona_id}/blocks/{other_persona_id}",
            put(block_persona).delete(unblock_persona),
        )
        .merge(report_routes)
        .merge(inbox_routes)
        .merge(sync_routes)
        .merge(game_routes)
        .merge(challenge_routes)
        .with_state(AppState {
            pool,
            mfa_cipher,
            sync_hub,
            game_registry,
            provider_runtime,
            cartridge_distribution,
            server_name,
        })
        .layer(TraceLayer::new_for_http())
}

async fn discover_server(State(state): State<AppState>) -> Response {
    match server_discovery::document(
        &state.pool,
        &state.server_name,
        state.provider_runtime.is_some(),
        state.cartridge_distribution.is_some(),
        state
            .cartridge_distribution
            .as_ref()
            .and_then(CartridgeDistributionRuntime::operator_custom_authority),
    )
    .await
    {
        Ok(document) => (StatusCode::OK, Json(document)).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code: "server_discovery_unavailable",
                    message: "server identity is temporarily unavailable",
                },
            }),
        )
            .into_response(),
    }
}

async fn register_account(
    State(state): State<AppState>,
    Json(request): Json<RegistrationRequest>,
) -> Result<(StatusCode, Json<RegistrationResponse>), ApiError> {
    let account = accounts::register_account(
        &state.pool,
        RegistrationInput {
            invite_code: request.invite_code,
            username: request.username,
            password: request.password,
        },
    )
    .await
    .map_err(ApiError::Registration)?;

    let (status, account) = match account {
        RegistrationOutcome::Created(account) => (StatusCode::CREATED, account),
        RegistrationOutcome::Existing(account) => (StatusCode::OK, account),
    };
    Ok((
        status,
        Json(RegistrationResponse {
            id: account.id,
            username: account.username,
        }),
    ))
}

async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Response, ApiError> {
    let outcome = sessions::create_session(
        &state.pool,
        CreateSessionInput {
            username: request.username,
            password: request.password,
            device_name: request.device_name,
        },
    )
    .await
    .map_err(ApiError::Session)?;

    let response = match outcome {
        SessionCreation::Created(created) => (
            StatusCode::CREATED,
            Json(CreatedSessionResponse {
                token: created.token,
                session: session_response(created.session),
            }),
        )
            .into_response(),
        SessionCreation::MfaRequired(challenge) => (
            StatusCode::ACCEPTED,
            Json(MfaChallengeResponse {
                mfa_required: true,
                challenge_token: challenge.token,
                expires_at: challenge.expires_at,
            }),
        )
            .into_response(),
    };

    Ok(no_store(response))
}

async fn complete_mfa_session(
    State(state): State<AppState>,
    Json(request): Json<CompleteMfaSessionRequest>,
) -> Result<Response, ApiError> {
    let created = mfa::complete_login_challenge(
        &state.pool,
        &state.mfa_cipher,
        &request.challenge_token,
        &request.code,
    )
    .await
    .map_err(ApiError::Mfa)?;

    Ok(no_store(
        (
            StatusCode::CREATED,
            Json(CreatedSessionResponse {
                token: created.token,
                session: session_response(created.session),
            }),
        )
            .into_response(),
    ))
}

async fn begin_mfa_enrollment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BeginMfaEnrollmentRequest>,
) -> Result<Response, ApiError> {
    let session_token = bearer_token(&headers)?.to_owned();
    let enrollment = mfa::begin_enrollment(
        &state.pool,
        &state.mfa_cipher,
        BeginEnrollmentInput {
            session_token,
            password: request.password,
        },
    )
    .await
    .map_err(ApiError::Mfa)?;

    Ok(no_store(
        (
            StatusCode::CREATED,
            Json(MfaEnrollmentResponse {
                secret: enrollment.secret,
                provisioning_uri: enrollment.provisioning_uri,
            }),
        )
            .into_response(),
    ))
}

async fn confirm_mfa_enrollment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ConfirmMfaEnrollmentRequest>,
) -> Result<Response, ApiError> {
    let session_token = bearer_token(&headers)?.to_owned();
    let confirmed = mfa::confirm_enrollment(
        &state.pool,
        &state.mfa_cipher,
        ConfirmEnrollmentInput {
            session_token,
            code: request.code,
        },
    )
    .await
    .map_err(ApiError::Mfa)?;

    Ok(no_store(
        (
            StatusCode::OK,
            Json(ConfirmedMfaEnrollmentResponse {
                recovery_codes: confirmed.recovery_codes,
            }),
        )
            .into_response(),
    ))
}

async fn get_mfa_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let session_token = bearer_token(&headers)?;
    let status = mfa::status(&state.pool, session_token)
        .await
        .map_err(ApiError::Mfa)?;

    Ok(no_store(
        Json(MfaStatusResponse {
            enabled: status.enabled,
            recovery_codes_remaining: status.recovery_codes_remaining,
        })
        .into_response(),
    ))
}

async fn disable_mfa(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DisableMfaRequest>,
) -> Result<StatusCode, ApiError> {
    let session_token = bearer_token(&headers)?.to_owned();
    mfa::disable(
        &state.pool,
        &state.mfa_cipher,
        DisableMfaInput {
            session_token,
            password: request.password,
            code: request.code,
        },
    )
    .await
    .map_err(ApiError::Mfa)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let sessions = sessions::list_sessions(&state.pool, token)
        .await
        .map_err(ApiError::Session)?;

    let mut response = Json(SessionListResponse {
        sessions: sessions.into_iter().map(session_response).collect(),
    })
    .into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));

    Ok(response)
}

async fn revoke_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&headers)?;
    let session_id = Uuid::try_parse(&session_id)
        .map_err(|_| ApiError::Session(SessionError::SessionNotFound))?;
    sessions::revoke_session(&state.pool, token, session_id)
        .await
        .map_err(ApiError::Session)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn create_persona(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreatePersonaRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let persona = personas::create_persona(
        &state.pool,
        token,
        CreatePersonaInput {
            handle: request.handle,
            display_name: request.display_name,
            bio: request.bio,
            status_message: request.status_message,
        },
    )
    .await
    .map_err(ApiError::Persona)?;

    Ok(no_store(
        (StatusCode::CREATED, Json(persona_response(persona))).into_response(),
    ))
}

async fn list_personas(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let personas = personas::list_personas(&state.pool, token)
        .await
        .map_err(ApiError::Persona)?;

    Ok(no_store(
        Json(PersonaListResponse {
            personas: personas.into_iter().map(persona_response).collect(),
        })
        .into_response(),
    ))
}

async fn get_persona_by_handle(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Json<PersonaResponse>, ApiError> {
    let persona = personas::get_persona_by_handle(&state.pool, &handle)
        .await
        .map_err(ApiError::Persona)?;

    Ok(Json(persona_response(persona)))
}

async fn update_persona(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<UpdatePersonaRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let persona = personas::update_persona(
        &state.pool,
        token,
        &persona_id,
        UpdatePersonaInput {
            handle: request.handle,
            display_name: request.display_name,
            bio: request.bio,
            status_message: request.status_message,
        },
    )
    .await
    .map_err(ApiError::Persona)?;

    Ok(no_store(Json(persona_response(persona)).into_response()))
}

async fn request_connection(
    State(state): State<AppState>,
    Path((persona_id, other_persona_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let outcome =
        connections::request_connection(&state.pool, token, &persona_id, &other_persona_id)
            .await
            .map_err(ApiError::Connection)?;
    let (status, request) = match outcome {
        ResourceOutcome::Created(request) => (StatusCode::CREATED, request),
        ResourceOutcome::Existing(request) => (StatusCode::OK, request),
    };

    Ok(no_store(
        (status, Json(connection_request_response(request))).into_response(),
    ))
}

async fn list_connection_requests(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let inventory = connections::list_connection_requests(&state.pool, token, &persona_id)
        .await
        .map_err(ApiError::Connection)?;

    Ok(no_store(
        Json(ConnectionRequestInventoryResponse {
            incoming: inventory
                .incoming
                .into_iter()
                .map(connection_request_response)
                .collect(),
            outgoing: inventory
                .outgoing
                .into_iter()
                .map(connection_request_response)
                .collect(),
        })
        .into_response(),
    ))
}

async fn accept_connection(
    State(state): State<AppState>,
    Path((persona_id, other_persona_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let connection =
        connections::accept_connection(&state.pool, token, &persona_id, &other_persona_id)
            .await
            .map_err(ApiError::Connection)?;

    Ok(no_store(
        Json(connection_response(connection)).into_response(),
    ))
}

async fn list_connections(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let connections = connections::list_connections(&state.pool, token, &persona_id)
        .await
        .map_err(ApiError::Connection)?;

    Ok(no_store(
        Json(ConnectionListResponse {
            connections: connections.into_iter().map(connection_response).collect(),
        })
        .into_response(),
    ))
}

async fn remove_connection(
    State(state): State<AppState>,
    Path((persona_id, other_persona_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&headers)?;
    connections::remove_connection(&state.pool, token, &persona_id, &other_persona_id)
        .await
        .map_err(ApiError::Connection)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn block_persona(
    State(state): State<AppState>,
    Path((persona_id, other_persona_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let outcome = connections::block_persona(&state.pool, token, &persona_id, &other_persona_id)
        .await
        .map_err(ApiError::Connection)?;
    let (status, block) = match outcome {
        ResourceOutcome::Created(block) => (StatusCode::CREATED, block),
        ResourceOutcome::Existing(block) => (StatusCode::OK, block),
    };

    Ok(no_store(
        (status, Json(block_response(block))).into_response(),
    ))
}

async fn list_blocks(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let blocks = connections::list_blocks(&state.pool, token, &persona_id)
        .await
        .map_err(ApiError::Connection)?;

    Ok(no_store(
        Json(BlockListResponse {
            blocks: blocks.into_iter().map(block_response).collect(),
        })
        .into_response(),
    ))
}

async fn unblock_persona(
    State(state): State<AppState>,
    Path((persona_id, other_persona_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let token = bearer_token(&headers)?;
    connections::unblock_persona(&state.pool, token, &persona_id, &other_persona_id)
        .await
        .map_err(ApiError::Connection)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_conversations(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Query(query): Query<ConversationListQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let conversations = inboxes::list_conversations(&state.pool, token, &persona_id, query.limit)
        .await
        .map_err(ApiError::Inbox)?;

    Ok(no_store(
        Json(ConversationListResponse {
            conversations: conversations
                .into_iter()
                .map(conversation_response)
                .collect(),
        })
        .into_response(),
    ))
}

async fn list_messages(
    State(state): State<AppState>,
    Path((persona_id, conversation_id)): Path<(String, String)>,
    Query(query): Query<MessageHistoryQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let page = inboxes::list_messages(
        &state.pool,
        token,
        &persona_id,
        &conversation_id,
        query.before,
        query.limit,
    )
    .await
    .map_err(ApiError::Inbox)?;

    Ok(no_store(
        Json(MessageListResponse {
            messages: page.messages.into_iter().map(message_response).collect(),
            next_before: page.next_before,
        })
        .into_response(),
    ))
}

async fn send_message(
    State(state): State<AppState>,
    Path((persona_id, conversation_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(request): Json<SendMessageRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let message = inboxes::send_user_message(
        &state.pool,
        token,
        &persona_id,
        &conversation_id,
        &request.body,
    )
    .await
    .map_err(ApiError::Inbox)?;

    Ok(no_store(
        (StatusCode::CREATED, Json(message_response(message))).into_response(),
    ))
}

async fn mark_conversation_read(
    State(state): State<AppState>,
    Path((persona_id, conversation_id, message_id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let receipt = inboxes::mark_read(
        &state.pool,
        token,
        &persona_id,
        &conversation_id,
        &message_id,
    )
    .await
    .map_err(ApiError::Inbox)?;

    Ok(no_store(
        Json(ReadReceiptResponse {
            through_message_id: receipt.through_message_id.to_string(),
            unread_count: receipt.unread_count,
        })
        .into_response(),
    ))
}

async fn list_games(State(state): State<AppState>) -> Result<Json<GameCatalogResponse>, ApiError> {
    let mut games = state
        .game_registry
        .catalog()
        .into_iter()
        .map(game_manifest_response)
        .collect::<Vec<_>>();
    if state.provider_runtime.is_some() {
        games.extend(
            provider_games::active_catalog(&state.pool)
                .await
                .map_err(ApiError::Game)?
                .into_iter()
                .map(|manifest| GameManifestResponse {
                    key: manifest.key,
                    version: manifest.version,
                    display_name: manifest.display_name,
                    min_human_players: manifest.min_human_players,
                    max_human_players: manifest.max_human_players,
                    authority: "registered_provider",
                    provider_release_id: Some(manifest.release_id.to_string()),
                }),
        );
        games.sort_by(|left, right| {
            (left.key.as_str(), left.version).cmp(&(right.key.as_str(), right.version))
        });
    }
    Ok(Json(GameCatalogResponse { games }))
}

async fn list_cartridges(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    sessions::authenticate(&state.pool, token)
        .await
        .map_err(ApiError::Session)?;
    let cartridges = cartridge_catalog::list_player_catalog(&state.pool)
        .await
        .map_err(ApiError::Catalog)?;
    Ok(no_store(
        Json(CartridgeCatalogResponse { cartridges }).into_response(),
    ))
}

async fn acquire_cartridge(
    State(state): State<AppState>,
    Path((game_key, archive_sha256)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    sessions::authenticate(&state.pool, token)
        .await
        .map_err(ApiError::Session)?;
    let runtime = state
        .cartridge_distribution
        .as_ref()
        .ok_or(ApiError::Distribution(DistributionError::Denied))?;
    let document =
        cartridge_distribution::acquire_exact(&state.pool, runtime, &game_key, &archive_sha256)
            .await
            .map_err(ApiError::Distribution)?;
    Ok(no_store(
        (
            [(CONTENT_TYPE, HeaderValue::from_static("application/json"))],
            document,
        )
            .into_response(),
    ))
}

async fn acquire_session_cartridge(
    State(state): State<AppState>,
    Path((persona_id, game_session_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let actor_id = games::authenticate_owned_persona(&state.pool, token, &persona_id)
        .await
        .map_err(ApiError::Game)?;
    let session_id = Uuid::try_parse(&game_session_id)
        .map_err(|_| ApiError::Distribution(DistributionError::InvalidInput))?;
    let runtime = state
        .cartridge_distribution
        .as_ref()
        .ok_or(ApiError::Distribution(DistributionError::Denied))?;
    let document =
        cartridge_distribution::acquire_session_exact(&state.pool, runtime, actor_id, session_id)
            .await
            .map_err(ApiError::Distribution)?;
    Ok(no_store(
        (
            [(CONTENT_TYPE, HeaderValue::from_static("application/json"))],
            document,
        )
            .into_response(),
    ))
}

async fn list_game_sessions(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Query(query): Query<GameSessionQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let sessions = games::list_sessions(&state.pool, token, &persona_id, query.limit)
        .await
        .map_err(ApiError::Game)?;

    Ok(no_store(
        Json(GameSessionListResponse {
            sessions: sessions.into_iter().map(game_session_response).collect(),
        })
        .into_response(),
    ))
}

async fn start_game_session(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<StartGameSessionRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let input = StartGameSessionInput {
        idempotency_key: request.idempotency_key.clone(),
        game_key: request.game_key.clone(),
        game_version: request.game_version,
    };
    let outcome = if let Some(runtime) = &state.provider_runtime {
        match provider_games::start_solo_session(
            &state.pool,
            runtime,
            state.cartridge_distribution.as_ref(),
            token,
            &persona_id,
            input,
        )
        .await
        .map_err(ApiError::Game)?
        {
            Some(outcome) => outcome,
            None => {
                return start_compiled_game_session(state, token, persona_id, request).await;
            }
        }
    } else {
        games::start_solo_session(
            &state.pool,
            &state.game_registry,
            state.cartridge_distribution.as_ref(),
            token,
            &persona_id,
            StartGameSessionInput {
                idempotency_key: request.idempotency_key,
                game_key: request.game_key,
                game_version: request.game_version,
            },
        )
        .await
        .map_err(ApiError::Game)?
    };
    let (status, session) = match outcome {
        GameSessionStartOutcome::Created(session) => (StatusCode::CREATED, session),
        GameSessionStartOutcome::Existing(session) => (StatusCode::OK, session),
        GameSessionStartOutcome::Pending(session) => (StatusCode::ACCEPTED, session),
    };
    Ok(no_store(
        (status, Json(game_session_response(session))).into_response(),
    ))
}

async fn start_compiled_game_session(
    state: AppState,
    token: &str,
    persona_id: String,
    request: StartGameSessionRequest,
) -> Result<Response, ApiError> {
    let outcome = games::start_solo_session(
        &state.pool,
        &state.game_registry,
        state.cartridge_distribution.as_ref(),
        token,
        &persona_id,
        StartGameSessionInput {
            idempotency_key: request.idempotency_key,
            game_key: request.game_key,
            game_version: request.game_version,
        },
    )
    .await
    .map_err(ApiError::Game)?;
    let (status, session) = match outcome {
        GameSessionStartOutcome::Created(session) => (StatusCode::CREATED, session),
        GameSessionStartOutcome::Existing(session) => (StatusCode::OK, session),
        GameSessionStartOutcome::Pending(_) => return Err(ApiError::Game(GameError::Internal)),
    };
    Ok(no_store(
        (status, Json(game_session_response(session))).into_response(),
    ))
}

async fn get_game_session(
    State(state): State<AppState>,
    Path((persona_id, game_session_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let session = games::get_session(&state.pool, token, &persona_id, &game_session_id)
        .await
        .map_err(ApiError::Game)?;

    Ok(no_store(
        Json(game_session_response(session)).into_response(),
    ))
}

async fn apply_game_command(
    State(state): State<AppState>,
    Path((persona_id, game_session_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(request): Json<GameCommandRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let input = GameCommandInput {
        idempotency_key: request.idempotency_key,
        expected_revision: request.expected_revision,
        command: request.command,
    };
    let result = if provider_games::is_provider_session(&state.pool, &game_session_id)
        .await
        .map_err(ApiError::Game)?
    {
        let runtime = state
            .provider_runtime
            .as_ref()
            .ok_or(ApiError::Game(GameError::GameUnavailable))?;
        provider_games::apply_command(
            &state.pool,
            runtime,
            token,
            &persona_id,
            &game_session_id,
            input,
        )
        .await
        .map_err(ApiError::Game)?
    } else {
        games::apply_command(
            &state.pool,
            &state.game_registry,
            token,
            &persona_id,
            &game_session_id,
            input,
        )
        .await
        .map_err(ApiError::Game)?
    };

    Ok(no_store(
        Json(game_command_response(result)).into_response(),
    ))
}

async fn apply_cartridge_action(
    State(state): State<AppState>,
    Path((persona_id, game_session_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(request): Json<CartridgeActionRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let actor_id = games::authenticate_owned_persona(&state.pool, token, &persona_id)
        .await
        .map_err(ApiError::Game)?;
    let session_id = Uuid::try_parse(&game_session_id)
        .map_err(|_| ApiError::Game(GameError::GameSessionNotFound))?;
    let runtime = state
        .cartridge_distribution
        .as_ref()
        .ok_or(ApiError::Game(GameError::CartridgeUnavailable))?;
    let idempotency_key = Uuid::try_parse(&request.idempotency_key)
        .map_err(|_| ApiError::Game(GameError::InvalidCommand))?;
    let validated = session_cartridges::admit_session_action(
        &state.pool,
        runtime,
        actor_id,
        session_id,
        idempotency_key,
        request.expected_revision,
        &request.archive_sha256,
        request.screen_id.as_deref(),
        &request.action,
        &request.payload,
    )
    .await
    .map_err(|error| ApiError::Game(map_session_cartridge_error(error)))?;
    let input = GameCommandInput {
        idempotency_key: idempotency_key.to_string(),
        expected_revision: request.expected_revision,
        command: validated.command,
    };
    let result = if validated.authority == "registered_provider" {
        let provider_runtime = state
            .provider_runtime
            .as_ref()
            .ok_or(ApiError::Game(GameError::GameUnavailable))?;
        provider_games::apply_command(
            &state.pool,
            provider_runtime,
            token,
            &persona_id,
            &game_session_id,
            input,
        )
        .await
        .map_err(ApiError::Game)?
    } else {
        games::apply_command(
            &state.pool,
            &state.game_registry,
            token,
            &persona_id,
            &game_session_id,
            input,
        )
        .await
        .map_err(ApiError::Game)?
    };

    Ok(no_store(
        Json(cartridge_action_response(result, validated.archive_sha256)).into_response(),
    ))
}

async fn reconcile_game_session(
    State(state): State<AppState>,
    Path((persona_id, game_session_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(request): Json<ReconcileGameRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let runtime = state
        .provider_runtime
        .as_ref()
        .ok_or(ApiError::Game(GameError::GameUnavailable))?;
    let result = provider_games::reconcile(
        &state.pool,
        runtime,
        token,
        &persona_id,
        &game_session_id,
        request.idempotency_key,
        request.expected_revision,
    )
    .await
    .map_err(ApiError::Game)?;
    Ok(no_store(
        Json(game_command_response(result)).into_response(),
    ))
}

async fn list_provider_achievements(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let achievements = provider_games::list_achievements(&state.pool, token, &persona_id)
        .await
        .map_err(ApiError::Game)?;
    Ok(no_store(
        Json(ProviderAchievementListResponse { achievements }).into_response(),
    ))
}

async fn receive_provider_event(
    State(state): State<AppState>,
    Path(release_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let Some(runtime) = &state.provider_runtime else {
        return Ok(StatusCode::NOT_FOUND);
    };
    let authority = headers
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Game(GameError::Unauthorized))?;
    let outcome = provider_games::apply_callback(
        &state.pool,
        runtime,
        &release_id,
        authority,
        &headers,
        &body,
    )
    .await
    .map_err(ApiError::Game)?;
    Ok(match outcome {
        CallbackApplyOutcome::Accepted | CallbackApplyOutcome::Duplicate => StatusCode::NO_CONTENT,
        CallbackApplyOutcome::Ignored => StatusCode::ACCEPTED,
    })
}

async fn create_report(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateReportRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let outcome = reports::create_report(
        &state.pool,
        token,
        &persona_id,
        CreateReportInput {
            idempotency_key: request.idempotency_key,
            subject_persona_id: request.subject_persona_id,
            category: request.category,
            detail: request.detail,
        },
    )
    .await
    .map_err(ApiError::Report)?;
    let (status, receipt) = match outcome {
        ReportOutcome::Created(receipt) => (StatusCode::CREATED, receipt),
        ReportOutcome::Existing(receipt) => (StatusCode::OK, receipt),
    };
    Ok(no_store(
        (status, Json(player_report_receipt_response(receipt))).into_response(),
    ))
}

async fn create_game_challenge(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateGameChallengeRequest>,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let outcome = challenges::create_challenge(
        &state.pool,
        &state.game_registry,
        token,
        &persona_id,
        CreateChallengeInput {
            idempotency_key: request.idempotency_key,
            challenged_persona_id: request.challenged_persona_id,
            game_key: request.game_key,
            game_version: request.game_version,
        },
    )
    .await
    .map_err(ApiError::Challenge)?;
    let (status, challenge) = match outcome {
        ChallengeOutcome::Created(challenge) => (StatusCode::CREATED, challenge),
        ChallengeOutcome::Existing(challenge) => (StatusCode::OK, challenge),
    };
    Ok((status, Json(game_challenge_response(challenge))).into_response())
}

async fn list_game_challenges(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Query(query): Query<GameChallengeQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let page = challenges::list_challenges(
        &state.pool,
        token,
        &persona_id,
        query.before.as_deref(),
        query.limit,
    )
    .await
    .map_err(ApiError::Challenge)?;
    Ok(Json(GameChallengeListResponse {
        challenges: page
            .challenges
            .into_iter()
            .map(game_challenge_response)
            .collect(),
        next_before: page.next_before.map(|id| id.to_string()),
    })
    .into_response())
}

async fn get_game_challenge(
    State(state): State<AppState>,
    Path((persona_id, challenge_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let challenge = challenges::get_challenge(&state.pool, token, &persona_id, &challenge_id)
        .await
        .map_err(ApiError::Challenge)?;
    Ok(Json(game_challenge_response(challenge)).into_response())
}

async fn accept_game_challenge(
    State(state): State<AppState>,
    Path((persona_id, challenge_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let challenge = challenges::accept_challenge(
        &state.pool,
        &state.game_registry,
        state.cartridge_distribution.as_ref(),
        token,
        &persona_id,
        &challenge_id,
    )
    .await
    .map_err(ApiError::Challenge)?;
    Ok(Json(game_challenge_response(challenge)).into_response())
}

async fn decline_game_challenge(
    State(state): State<AppState>,
    Path((persona_id, challenge_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let challenge = challenges::decline_challenge(
        &state.pool,
        &state.game_registry,
        token,
        &persona_id,
        &challenge_id,
    )
    .await
    .map_err(ApiError::Challenge)?;
    Ok(Json(game_challenge_response(challenge)).into_response())
}

async fn cancel_game_challenge(
    State(state): State<AppState>,
    Path((persona_id, challenge_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let challenge = challenges::cancel_challenge(
        &state.pool,
        &state.game_registry,
        token,
        &persona_id,
        &challenge_id,
    )
    .await
    .map_err(ApiError::Challenge)?;
    Ok(Json(game_challenge_response(challenge)).into_response())
}

async fn list_sync_events(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Query(query): Query<SyncQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let page = sync::list_events(&state.pool, token, &persona_id, query.after, query.limit)
        .await
        .map_err(ApiError::Sync)?;

    Ok(no_store(
        Json(SyncPageResponse {
            events: page.events.into_iter().map(sync_event_response).collect(),
            next_cursor: page.next_cursor,
            has_more: page.has_more,
            reset_required: page.reset_required,
        })
        .into_response(),
    ))
}

async fn open_sync_socket(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let token = bearer_token(&headers)?;
    let prepared = sync::prepare_socket(&state.pool, token, &persona_id, &state.sync_hub)
        .await
        .map_err(ApiError::Sync)?;
    Ok(websocket
        .max_message_size(SYNC_SOCKET_MAX_CLIENT_BYTES)
        .max_frame_size(SYNC_SOCKET_MAX_CLIENT_BYTES)
        .on_upgrade(move |socket| sync::serve_socket(socket, prepared))
        .into_response())
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    let authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Session(SessionError::Unauthorized))?;
    let mut parts = authorization.split_ascii_whitespace();
    let scheme = parts.next().unwrap_or_default();
    let token = parts.next().unwrap_or_default();

    if !scheme.eq_ignore_ascii_case("bearer") || token.is_empty() || parts.next().is_some() {
        return Err(ApiError::Session(SessionError::Unauthorized));
    }

    Ok(token)
}

fn session_response(session: DeviceSession) -> DeviceSessionResponse {
    DeviceSessionResponse {
        id: session.id.to_string(),
        device_name: session.device_name,
        created_at: session.created_at,
        last_used_at: session.last_used_at,
        expires_at: session.expires_at,
        revoked_at: session.revoked_at,
        current: session.current,
    }
}

fn persona_response(persona: Persona) -> PersonaResponse {
    PersonaResponse {
        id: persona.id.to_string(),
        handle: persona.handle,
        display_name: persona.display_name,
        bio: persona.bio,
        status_message: persona.status_message,
        created_at: persona.created_at,
        updated_at: persona.updated_at,
    }
}

fn player_report_receipt_response(receipt: PlayerReportReceipt) -> PlayerReportReceiptResponse {
    PlayerReportReceiptResponse {
        id: receipt.id.to_string(),
        idempotency_key: receipt.idempotency_key.to_string(),
        status: receipt.status,
        created_at: receipt.created_at,
    }
}

fn connection_request_response(request: ConnectionRequest) -> ConnectionRequestResponse {
    ConnectionRequestResponse {
        persona: persona_response(request.persona),
        created_at: request.created_at,
    }
}

fn connection_response(connection: Connection) -> ConnectionResponse {
    ConnectionResponse {
        persona: persona_response(connection.persona),
        connected_at: connection.connected_at,
    }
}

fn block_response(block: PersonaBlock) -> BlockResponse {
    BlockResponse {
        persona: persona_response(block.persona),
        created_at: block.created_at,
    }
}

fn conversation_response(conversation: ConversationSummary) -> ConversationResponse {
    ConversationResponse {
        id: conversation.id.to_string(),
        other_persona: persona_response(conversation.other_persona),
        unread_count: conversation.unread_count,
        latest_message: conversation.latest_message.map(message_response),
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
    }
}

fn message_response(message: InboxMessage) -> MessageResponse {
    match message.content {
        InboxMessageContent::User { sender, body } => MessageResponse::User {
            id: message.id.to_string(),
            sequence: message.sequence,
            sender: persona_response(sender),
            body,
            created_at: message.created_at,
        },
        InboxMessageContent::System(SystemMessage::ConnectionAccepted { actor }) => {
            MessageResponse::System {
                id: message.id.to_string(),
                sequence: message.sequence,
                system: SystemMessageResponse::ConnectionAccepted {
                    actor: persona_response(actor),
                },
                created_at: message.created_at,
            }
        }
        InboxMessageContent::System(SystemMessage::GameChallengeCreated {
            actor,
            challenge_id,
        }) => MessageResponse::System {
            id: message.id.to_string(),
            sequence: message.sequence,
            system: SystemMessageResponse::GameChallengeCreated {
                actor: persona_response(actor),
                challenge_id: challenge_id.to_string(),
            },
            created_at: message.created_at,
        },
        InboxMessageContent::System(SystemMessage::GameChallengeAccepted {
            actor,
            challenge_id,
            game_session_id,
        }) => MessageResponse::System {
            id: message.id.to_string(),
            sequence: message.sequence,
            system: SystemMessageResponse::GameChallengeAccepted {
                actor: persona_response(actor),
                challenge_id: challenge_id.to_string(),
                game_session_id: game_session_id.to_string(),
            },
            created_at: message.created_at,
        },
        InboxMessageContent::System(SystemMessage::GameChallengeDeclined {
            actor,
            challenge_id,
        }) => MessageResponse::System {
            id: message.id.to_string(),
            sequence: message.sequence,
            system: SystemMessageResponse::GameChallengeDeclined {
                actor: persona_response(actor),
                challenge_id: challenge_id.to_string(),
            },
            created_at: message.created_at,
        },
        InboxMessageContent::System(SystemMessage::GameChallengeCancelled {
            actor,
            challenge_id,
        }) => MessageResponse::System {
            id: message.id.to_string(),
            sequence: message.sequence,
            system: SystemMessageResponse::GameChallengeCancelled {
                actor: persona_response(actor),
                challenge_id: challenge_id.to_string(),
            },
            created_at: message.created_at,
        },
    }
}

fn game_manifest_response(manifest: GameManifest) -> GameManifestResponse {
    GameManifestResponse {
        key: manifest.key,
        version: manifest.version,
        display_name: manifest.display_name,
        min_human_players: manifest.min_human_players,
        max_human_players: manifest.max_human_players,
        authority: "platform_compiled",
        provider_release_id: None,
    }
}

fn game_session_response(session: GameSession) -> GameSessionResponse {
    GameSessionResponse {
        id: session.id.to_string(),
        game_key: session.game_key,
        game_version: session.game_version,
        revision: session.revision,
        status: session.status,
        state: session.state,
        authority: session.authority,
        provider_release_id: session
            .provider_release_id
            .map(|release_id| release_id.to_string()),
        availability: session.availability,
        presentation: session.presentation,
        result: session.result.map(|result| GameResultResponse {
            outcome: result.outcome,
            public_summary: result.public_summary,
            provider_revision: result.provider_revision,
            projected_at: result.projected_at,
        }),
        participants: session
            .participants
            .into_iter()
            .map(game_participant_response)
            .collect(),
        completed_at: session.completed_at,
        created_at: session.created_at,
        updated_at: session.updated_at,
    }
}

fn game_participant_response(participant: GameParticipant) -> GameParticipantResponse {
    GameParticipantResponse {
        seat: participant.seat,
        persona: persona_response(participant.persona),
    }
}

fn game_command_response(result: GameCommandResult) -> GameCommandResponse {
    GameCommandResponse {
        game_session_id: result.game_session_id.to_string(),
        revision: result.revision,
        status: result.status,
        state: result.state,
        authority: result.authority,
        provider_release_id: result
            .provider_release_id
            .map(|release_id| release_id.to_string()),
        availability: result.availability,
    }
}

fn cartridge_action_response(
    result: GameCommandResult,
    archive_sha256: String,
) -> CartridgeActionResponse {
    CartridgeActionResponse {
        game_session_id: result.game_session_id.to_string(),
        revision: result.revision,
        status: result.status,
        state: result.state,
        authority: result.authority,
        provider_release_id: result
            .provider_release_id
            .map(|release_id| release_id.to_string()),
        availability: result.availability,
        archive_sha256,
    }
}

fn map_session_cartridge_error(error: SessionCartridgeError) -> GameError {
    match error {
        SessionCartridgeError::InvalidInput => GameError::InvalidCommand,
        SessionCartridgeError::NotFound => GameError::GameSessionNotFound,
        SessionCartridgeError::Denied => GameError::CartridgeUnavailable,
        SessionCartridgeError::RevisionConflict => GameError::RevisionConflict,
        SessionCartridgeError::IdempotencyConflict => GameError::IdempotencyConflict,
        SessionCartridgeError::Completed => GameError::GameCompleted,
        SessionCartridgeError::Internal => GameError::Internal,
    }
}

fn game_challenge_response(challenge: GameChallenge) -> GameChallengeResponse {
    GameChallengeResponse {
        id: challenge.id.to_string(),
        game_key: challenge.game_key,
        game_version: challenge.game_version,
        direction: match challenge.direction {
            ChallengeDirection::Incoming => "incoming",
            ChallengeDirection::Outgoing => "outgoing",
        },
        status: challenge.status,
        challenger: persona_response(challenge.challenger),
        challenged: persona_response(challenge.challenged),
        game_session_id: challenge.game_session_id.map(|id| id.to_string()),
        expires_at: challenge.expires_at,
        resolved_at: challenge.resolved_at,
        created_at: challenge.created_at,
        updated_at: challenge.updated_at,
    }
}

fn sync_event_response(event: SyncEvent) -> SyncEventResponse {
    match event.kind {
        SyncEventKind::ConnectionRequests => SyncEventResponse::ConnectionRequests {
            cursor: event.cursor,
            created_at: event.created_at,
        },
        SyncEventKind::Connections => SyncEventResponse::Connections {
            cursor: event.cursor,
            created_at: event.created_at,
        },
        SyncEventKind::Blocks => SyncEventResponse::Blocks {
            cursor: event.cursor,
            created_at: event.created_at,
        },
        SyncEventKind::Conversation(conversation_id) => SyncEventResponse::Conversation {
            cursor: event.cursor,
            conversation_id: conversation_id.to_string(),
            created_at: event.created_at,
        },
        SyncEventKind::GameSession(game_session_id) => SyncEventResponse::GameSession {
            cursor: event.cursor,
            game_session_id: game_session_id.to_string(),
            created_at: event.created_at,
        },
        SyncEventKind::GameChallenge(game_challenge_id) => SyncEventResponse::GameChallenge {
            cursor: event.cursor,
            game_challenge_id: game_challenge_id.to_string(),
            created_at: event.created_at,
        },
    }
}

fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn inbox_no_store(response: Response) -> Response {
    no_store(response)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Registration(RegistrationError::InvalidUsername) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_username",
                "username must be 3-32 characters, begin with a letter or number, and contain only letters, numbers, underscores, or hyphens",
            ),
            ApiError::Registration(RegistrationError::InvalidPassword) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_password",
                "password must be 12-128 bytes",
            ),
            ApiError::Registration(RegistrationError::InvalidInvitation) => (
                StatusCode::FORBIDDEN,
                "invalid_invitation",
                "registration invitation is invalid",
            ),
            ApiError::Registration(RegistrationError::UsernameTaken) => (
                StatusCode::CONFLICT,
                "username_taken",
                "username is already registered",
            ),
            ApiError::Registration(RegistrationError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "account registration failed",
            ),
            ApiError::Session(SessionError::InvalidDeviceName) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_device_name",
                "device name must contain 1-64 non-control characters",
            ),
            ApiError::Session(SessionError::InvalidCredentials) => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "username or password is incorrect",
            ),
            ApiError::Session(SessionError::RateLimited) => (
                StatusCode::TOO_MANY_REQUESTS,
                "mfa_rate_limited",
                "too many active authentication challenges; try again later",
            ),
            ApiError::Session(SessionError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Session(SessionError::SessionNotFound) => (
                StatusCode::NOT_FOUND,
                "session_not_found",
                "device session was not found",
            ),
            ApiError::Session(SessionError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "device session operation failed",
            ),
            ApiError::Catalog(CatalogError::InvalidInput) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "catalog_invalid_input",
                "cartridge catalog input is invalid",
            ),
            ApiError::Catalog(CatalogError::Conflict) => (
                StatusCode::CONFLICT,
                "catalog_conflict",
                "cartridge catalog state changed",
            ),
            ApiError::Catalog(CatalogError::Denied) => (
                StatusCode::FORBIDDEN,
                "catalog_denied",
                "cartridge catalog operation is not permitted",
            ),
            ApiError::Catalog(CatalogError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "cartridge catalog operation failed",
            ),
            ApiError::Distribution(DistributionError::InvalidInput) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "cartridge_acquisition_invalid_input",
                "cartridge acquisition identity is invalid",
            ),
            ApiError::Distribution(DistributionError::Denied) => (
                StatusCode::NOT_FOUND,
                "cartridge_acquisition_denied",
                "the exact cartridge is not available for acquisition",
            ),
            ApiError::Distribution(DistributionError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "cartridge acquisition failed",
            ),
            ApiError::Mfa(MfaError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Mfa(MfaError::InvalidCredentials) => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "password is incorrect",
            ),
            ApiError::Mfa(MfaError::AlreadyEnabled) => (
                StatusCode::CONFLICT,
                "mfa_already_enabled",
                "two-factor authentication is already enabled",
            ),
            ApiError::Mfa(MfaError::NotEnabled) => (
                StatusCode::CONFLICT,
                "mfa_not_enabled",
                "two-factor authentication is not enabled",
            ),
            ApiError::Mfa(MfaError::EnrollmentNotFound) => (
                StatusCode::CONFLICT,
                "mfa_enrollment_not_found",
                "start a new two-factor enrollment",
            ),
            ApiError::Mfa(MfaError::InvalidCode) => (
                StatusCode::UNAUTHORIZED,
                "invalid_mfa_code",
                "the authentication code is invalid",
            ),
            ApiError::Mfa(MfaError::RateLimited) => (
                StatusCode::TOO_MANY_REQUESTS,
                "mfa_rate_limited",
                "too many authentication attempts; try again later",
            ),
            ApiError::Mfa(MfaError::InvalidChallenge) => (
                StatusCode::UNAUTHORIZED,
                "invalid_mfa_challenge",
                "the authentication challenge is invalid or expired",
            ),
            ApiError::Mfa(MfaError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "two-factor authentication operation failed",
            ),
            ApiError::Persona(PersonaError::InvalidHandle) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_handle",
                "handle must be 3-24 characters, begin with a letter or number, and contain only letters, numbers, underscores, or hyphens",
            ),
            ApiError::Persona(PersonaError::InvalidDisplayName) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_display_name",
                "display name must contain 1-64 non-control characters",
            ),
            ApiError::Persona(PersonaError::InvalidBio) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_bio",
                "bio must contain at most 1000 characters and no control characters other than tabs or newlines",
            ),
            ApiError::Persona(PersonaError::InvalidStatusMessage) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_status_message",
                "status message must contain at most 160 non-control characters",
            ),
            ApiError::Persona(PersonaError::EmptyPatch) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "empty_persona_patch",
                "at least one editable persona field is required",
            ),
            ApiError::Persona(PersonaError::HandleTaken) => (
                StatusCode::CONFLICT,
                "handle_taken",
                "handle is already in use",
            ),
            ApiError::Persona(PersonaError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Persona(PersonaError::PersonaNotFound) => (
                StatusCode::NOT_FOUND,
                "persona_not_found",
                "persona was not found",
            ),
            ApiError::Persona(PersonaError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "persona operation failed",
            ),
            ApiError::Report(ReportError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Report(ReportError::PersonaNotFound) => (
                StatusCode::NOT_FOUND,
                "persona_not_found",
                "persona was not found",
            ),
            ApiError::Report(ReportError::InvalidReport) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_report",
                "report input is invalid",
            ),
            ApiError::Report(ReportError::IdempotencyConflict) => (
                StatusCode::CONFLICT,
                "report_idempotency_conflict",
                "the report idempotency key was already used",
            ),
            ApiError::Report(ReportError::OpenLimitReached) => (
                StatusCode::TOO_MANY_REQUESTS,
                "report_limit_reached",
                "the open report limit was reached",
            ),
            ApiError::Report(ReportError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "report operation failed",
            ),
            ApiError::Connection(ConnectionError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Connection(ConnectionError::PersonaNotFound) => (
                StatusCode::NOT_FOUND,
                "persona_not_found",
                "persona was not found",
            ),
            ApiError::Connection(ConnectionError::ConnectionUnavailable) => (
                StatusCode::CONFLICT,
                "connection_unavailable",
                "the requested connection is unavailable",
            ),
            ApiError::Connection(ConnectionError::ConnectionRequestNotFound) => (
                StatusCode::NOT_FOUND,
                "connection_request_not_found",
                "incoming connection request was not found",
            ),
            ApiError::Connection(ConnectionError::ConnectionRequestPending) => (
                StatusCode::CONFLICT,
                "connection_request_pending",
                "an incoming connection request is already pending",
            ),
            ApiError::Connection(ConnectionError::ConnectionAlreadyExists) => (
                StatusCode::CONFLICT,
                "connection_already_exists",
                "the personas are already connected",
            ),
            ApiError::Connection(ConnectionError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "connection operation failed",
            ),
            ApiError::Challenge(ChallengeError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Challenge(ChallengeError::PersonaNotFound) => (
                StatusCode::NOT_FOUND,
                "persona_not_found",
                "persona was not found",
            ),
            ApiError::Challenge(ChallengeError::ChallengeNotFound) => (
                StatusCode::NOT_FOUND,
                "game_challenge_not_found",
                "game challenge was not found",
            ),
            ApiError::Challenge(ChallengeError::InvalidChallenge) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_game_challenge",
                "game challenge input is invalid",
            ),
            ApiError::Challenge(ChallengeError::InvalidPagination) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_pagination",
                "game challenge pagination is invalid",
            ),
            ApiError::Challenge(ChallengeError::GameUnavailable) => (
                StatusCode::CONFLICT,
                "game_unavailable",
                "the requested game version is unavailable",
            ),
            ApiError::Challenge(ChallengeError::TargetUnavailable) => (
                StatusCode::CONFLICT,
                "challenge_target_unavailable",
                "the challenge target is unavailable",
            ),
            ApiError::Challenge(ChallengeError::PendingLimitReached) => (
                StatusCode::CONFLICT,
                "game_challenge_limit_reached",
                "the pending game challenge limit was reached",
            ),
            ApiError::Challenge(ChallengeError::DuplicatePending) => (
                StatusCode::CONFLICT,
                "game_challenge_pending",
                "an equivalent game challenge is already pending",
            ),
            ApiError::Challenge(ChallengeError::IdempotencyConflict) => (
                StatusCode::CONFLICT,
                "game_challenge_idempotency_conflict",
                "the game challenge idempotency key was already used",
            ),
            ApiError::Challenge(ChallengeError::TransitionUnavailable) => (
                StatusCode::CONFLICT,
                "game_challenge_transition_unavailable",
                "the requested game challenge transition is unavailable",
            ),
            ApiError::Challenge(ChallengeError::ChallengeExpired) => (
                StatusCode::CONFLICT,
                "game_challenge_expired",
                "the game challenge has expired",
            ),
            ApiError::Challenge(
                ChallengeError::InitializationFailed | ChallengeError::Internal,
            ) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "game challenge operation failed",
            ),
            ApiError::Inbox(InboxError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Inbox(InboxError::PersonaNotFound) => (
                StatusCode::NOT_FOUND,
                "persona_not_found",
                "persona was not found",
            ),
            ApiError::Inbox(InboxError::ConversationNotFound) => (
                StatusCode::NOT_FOUND,
                "conversation_not_found",
                "conversation was not found",
            ),
            ApiError::Inbox(InboxError::ConversationUnavailable) => (
                StatusCode::CONFLICT,
                "conversation_unavailable",
                "the conversation cannot accept a new message",
            ),
            ApiError::Inbox(InboxError::MessageNotFound) => (
                StatusCode::NOT_FOUND,
                "message_not_found",
                "message was not found",
            ),
            ApiError::Inbox(InboxError::InvalidMessageBody) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_message_body",
                "message body must contain 1-4000 characters and no control characters other than tabs or newlines",
            ),
            ApiError::Inbox(InboxError::InvalidPagination) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_pagination",
                "pagination limits must be 1-100 and before must be a positive sequence",
            ),
            ApiError::Inbox(InboxError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "inbox operation failed",
            ),
            ApiError::Game(GameError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Game(GameError::PersonaNotFound) => (
                StatusCode::NOT_FOUND,
                "persona_not_found",
                "persona was not found",
            ),
            ApiError::Game(GameError::GameSessionNotFound) => (
                StatusCode::NOT_FOUND,
                "game_session_not_found",
                "game session was not found",
            ),
            ApiError::Game(GameError::InvalidPagination) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_pagination",
                "game session pagination limit must be 1-100",
            ),
            ApiError::Game(GameError::InvalidStart) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_game_start",
                "game start idempotency key must be a UUID",
            ),
            ApiError::Game(GameError::GameUnavailable) => (
                StatusCode::CONFLICT,
                "game_unavailable",
                "the requested game version is unavailable",
            ),
            ApiError::Game(GameError::CartridgeUnavailable) => (
                StatusCode::CONFLICT,
                "session_cartridge_unavailable",
                "the exact session cartridge is unavailable or denied",
            ),
            ApiError::Game(GameError::InvalidParticipants) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_game_participants",
                "game participants are invalid",
            ),
            ApiError::Game(GameError::ActiveSessionLimit) => (
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_active_game_sessions",
                "the persona has too many active solo game sessions",
            ),
            ApiError::Game(GameError::InvalidCommand) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_game_command",
                "the game command is invalid",
            ),
            ApiError::Game(GameError::CommandRejected) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "game_command_rejected",
                "the game command was rejected",
            ),
            ApiError::Game(GameError::GameCompleted) => (
                StatusCode::CONFLICT,
                "game_completed",
                "the game session is already completed",
            ),
            ApiError::Game(GameError::RevisionConflict) => (
                StatusCode::CONFLICT,
                "game_revision_conflict",
                "the game session revision has changed",
            ),
            ApiError::Game(GameError::IdempotencyConflict) => (
                StatusCode::CONFLICT,
                "game_idempotency_conflict",
                "the game idempotency key was already used for another request",
            ),
            ApiError::Game(GameError::ProviderUnavailable) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "provider_unavailable",
                "the game provider is unavailable; reconcile the session before retrying",
            ),
            ApiError::Game(GameError::InitializationFailed | GameError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "game operation failed",
            ),
            ApiError::Sync(SyncError::Unauthorized) => (
                StatusCode::UNAUTHORIZED,
                "invalid_session",
                "a valid device session is required",
            ),
            ApiError::Sync(SyncError::PersonaNotFound) => (
                StatusCode::NOT_FOUND,
                "persona_not_found",
                "persona was not found",
            ),
            ApiError::Sync(SyncError::InvalidCursor) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_sync_cursor",
                "sync cursor must identify the current or an earlier persona event",
            ),
            ApiError::Sync(SyncError::InvalidPagination) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_pagination",
                "sync pagination limit must be 1-100",
            ),
            ApiError::Sync(SyncError::SocketLimitReached) => (
                StatusCode::TOO_MANY_REQUESTS,
                "sync_socket_limit_reached",
                "too many live sync connections are already open",
            ),
            ApiError::Sync(SyncError::Internal) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "persona sync operation failed",
            ),
        };

        let mut response = (
            status,
            Json(ErrorEnvelope {
                error: ErrorBody { code, message },
            }),
        )
            .into_response();
        if status == StatusCode::UNAUTHORIZED
            && matches!(code, "invalid_credentials" | "invalid_session")
        {
            response
                .headers_mut()
                .insert(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }

        response
    }
}

async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(1) => (StatusCode::OK, Json(health_document(true))),
        Ok(_) | Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(health_document(false)),
        ),
    }
}

fn health_document(database_ok: bool) -> HealthResponse {
    HealthResponse {
        service: "omarchy-gaming-system",
        version: env!("CARGO_PKG_VERSION"),
        status: if database_ok { "ok" } else { "degraded" },
        database: if database_ok { "ok" } else { "unavailable" },
    }
}

#[cfg(test)]
mod tests {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::CONTENT_TYPE},
    };
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use tower::ServiceExt;

    use crate::{accounts, mfa::MfaCipher};

    use super::{HealthResponse, health_document, router};

    #[test]
    fn healthy_document_reports_service_and_database() {
        assert_eq!(
            health_document(true),
            HealthResponse {
                service: "omarchy-gaming-system",
                version: env!("CARGO_PKG_VERSION"),
                status: "ok",
                database: "ok",
            }
        );
    }

    #[test]
    fn degraded_document_reports_database_failure() {
        let document = health_document(false);

        assert_eq!(document.status, "degraded");
        assert_eq!(document.database, "unavailable");
    }

    #[tokio::test]
    async fn registration_rejects_oversized_request_bodies_before_database_work() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://test:test@127.0.0.1:5432/test")
            .expect("test database URL should parse without connecting");
        let oversized_payload = json!({
            "invite_code": "ogsi_invalid",
            "username": "valid_player",
            "password": "x".repeat(1024)
        });

        let response = router(pool, MfaCipher::test_cipher())
            .oneshot(
                Request::post("/v1/accounts")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(oversized_payload.to_string()))
                    .expect("request should be valid"),
            )
            .await
            .expect("router should return a response");

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
    async fn registration_persists_a_canonical_argon2id_account(pool: PgPool) {
        let password = "TEST-ONLY-registration-passphrase";
        let invite_code = accounts::create_test_invite(&pool).await;
        let (status, document) = post_registration(
            pool.clone(),
            json!({
                "invite_code": invite_code,
                "username": "  Player_One  ",
                "password": password
            }),
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(document["username"], "player_one");
        assert_eq!(document.as_object().map(|object| object.len()), Some(2));

        let (id, username, password_hash, account_status) =
            sqlx::query_as::<_, (String, String, String, String)>(
                "SELECT id::text, username, password_hash, status FROM accounts",
            )
            .fetch_one(&pool)
            .await
            .expect("registered account should be stored");

        assert_eq!(document["id"], id);
        assert_eq!(username, "player_one");
        assert_eq!(account_status, "active");
        assert_ne!(password_hash, password);
        assert!(password_hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));

        let parsed_hash =
            PasswordHash::new(&password_hash).expect("stored hash should be PHC encoded");
        assert!(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        );

        let duplicate_invite = accounts::create_test_invite(&pool).await;
        let (duplicate_status, duplicate_document) = post_registration(
            pool.clone(),
            json!({
                "invite_code": duplicate_invite,
                "username": "PLAYER_ONE",
                "password": "TEST-ONLY-a-different-passphrase"
            }),
        )
        .await;

        assert_eq!(duplicate_status, StatusCode::CONFLICT);
        assert_eq!(duplicate_document["error"]["code"], "username_taken");

        let (account_count, unchanged_hash) =
            sqlx::query_as::<_, (i64, String)>("SELECT count(*), min(password_hash) FROM accounts")
                .fetch_one(&pool)
                .await
                .expect("account count should be readable");
        assert_eq!(account_count, 1);
        assert_eq!(unchanged_hash, password_hash);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
    async fn registration_rejects_invalid_input_without_inserting(pool: PgPool) {
        for (mut payload, expected_code) in [
            (
                json!({
                    "username": "-invalid",
                    "password": "TEST-ONLY-registration-passphrase"
                }),
                "invalid_username",
            ),
            (
                json!({"username": "valid_player", "password": "too-short"}),
                "invalid_password",
            ),
        ] {
            payload["invite_code"] = accounts::create_test_invite(&pool).await.into();
            let (status, document) = post_registration(pool.clone(), payload).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(document["error"]["code"], expected_code);
        }

        let account_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .expect("account count should be readable");
        assert_eq!(account_count, 0);
    }

    async fn post_registration(pool: PgPool, payload: Value) -> (StatusCode, Value) {
        let response = router(pool, MfaCipher::test_cipher())
            .oneshot(
                Request::post("/v1/accounts")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request should be valid"),
            )
            .await
            .expect("router should return a response");
        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("response body should be readable")
            .to_bytes();
        let document =
            serde_json::from_slice(&body).expect("response body should contain valid JSON");

        (status, document)
    }
}
