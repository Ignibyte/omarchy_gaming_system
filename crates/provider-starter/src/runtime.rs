use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};

use anyhow::{Context as _, Result as AnyResult};
use axum::{
    Router,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header::HOST},
    response::Response,
    routing::post,
};
use axum_server::tls_rustls::RustlsConfig;
use ed25519_dalek::VerifyingKey;
use omarchygs_provider_sdk::{
    ProviderError, Result,
    protocol::{
        GrantExpectation, HttpMessageSigner, ProviderCompatibilityOffer,
        ProviderCompatibilitySelection, ProviderOperationKind, ProviderOperationRequest,
        ProviderOperationResponse, RequestSignatureContext, SignatureHeaders,
        parse_authenticated_json, sha256_hex, verify_grant, verify_request_signature,
    },
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::task::JoinHandle;
use url::Url;
use uuid::Uuid;

use crate::{
    CallbackConfig, PROVIDER_STARTER_MIGRATOR, ProviderGame, callback, store::StarterStore,
};

const BASE_PATH: &str = "/omarchygs/provider/v1/";

/// Finite public starter limits. A response delay is available only in an
/// explicitly conformance-built provider and occurs after durable commit.
#[derive(Debug, Clone)]
pub struct StarterLimits {
    pub request_body_bytes: usize,
    pub operation_response_delay_after_commit: Option<(ProviderOperationKind, Duration)>,
}

impl Default for StarterLimits {
    fn default() -> Self {
        Self {
            request_body_bytes: 65_536,
            operation_response_delay_after_commit: None,
        }
    }
}

impl StarterLimits {
    fn validate(&self) -> Result<()> {
        if !(1_024..=131_072).contains(&self.request_body_bytes) {
            return Err(ProviderError::InvalidInput);
        }
        if let Some((_, delay)) = self.operation_response_delay_after_commit
            && (!cfg!(feature = "conformance")
                || delay.is_zero()
                || delay > Duration::from_secs(10))
        {
            return Err(ProviderError::InvalidInput);
        }
        Ok(())
    }
}

/// Exact provider-side runtime configuration. It contains no platform
/// database or provider-admission handle.
#[derive(Clone)]
pub struct ProviderStarterConfig {
    release_id: Uuid,
    authority: String,
    platform_grant_key_id: String,
    platform_grant_key: VerifyingKey,
    platform_message_key_id: String,
    platform_message_key: VerifyingKey,
    provider_signer: Arc<HttpMessageSigner>,
    callback: CallbackConfig,
    limits: StarterLimits,
}

impl ProviderStarterConfig {
    /// Construct and validate one exact runtime configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        release_id: Uuid,
        authority: String,
        platform_grant_key_id: String,
        platform_grant_key: VerifyingKey,
        platform_message_key_id: String,
        platform_message_key: VerifyingKey,
        provider_signer: HttpMessageSigner,
        callback: CallbackConfig,
        limits: StarterLimits,
    ) -> Result<Self> {
        if release_id.is_nil()
            || !valid_authority(&authority)
            || !valid_key_id(&platform_grant_key_id)
            || !valid_key_id(&platform_message_key_id)
        {
            return Err(ProviderError::InvalidInput);
        }
        limits.validate()?;
        Ok(Self {
            release_id,
            authority,
            platform_grant_key_id,
            platform_grant_key,
            platform_message_key_id,
            platform_message_key,
            provider_signer: Arc::new(provider_signer),
            callback,
            limits,
        })
    }

    /// Exact configured provider release.
    #[must_use]
    pub const fn release_id(&self) -> Uuid {
        self.release_id
    }
}

/// Reusable provider backend for one deterministic game implementation.
#[derive(Clone)]
pub struct ProviderStarter<G: ProviderGame> {
    game: G,
    config: ProviderStarterConfig,
    store: StarterStore,
}

impl<G: ProviderGame> ProviderStarter<G> {
    /// Connect, migrate, and pin one provider-owned database.
    pub async fn connect(
        game: G,
        config: ProviderStarterConfig,
        database_url: &str,
        max_connections: u32,
    ) -> Result<Self> {
        if database_url.is_empty()
            || database_url.len() > 2_048
            || !(1..=32).contains(&max_connections)
        {
            return Err(ProviderError::InvalidInput);
        }
        game.identity().validate()?;
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(crate::store::database_error)?;
        Self::from_pool(game, config, pool).await
    }

    /// Migrate and pin an existing provider-owned PostgreSQL pool.
    pub async fn from_pool(game: G, config: ProviderStarterConfig, pool: PgPool) -> Result<Self> {
        game.identity().validate()?;
        config.limits.validate()?;
        PROVIDER_STARTER_MIGRATOR
            .run(&pool)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "provider starter migration failed");
                ProviderError::Internal
            })?;
        let store = StarterStore::new(pool);
        store.initialize(game.identity(), config.release_id).await?;
        Ok(Self {
            game,
            config,
            store,
        })
    }

    /// Build the fixed v1 provider router. No discovery or administration
    /// route is exposed.
    pub fn router(self: &Arc<Self>) -> Router {
        Router::new()
            .route(
                "/omarchygs/provider/v1/compatibility",
                post(handle_compatibility::<G>),
            )
            .route("/omarchygs/provider/v1/launch", post(handle_launch::<G>))
            .route("/omarchygs/provider/v1/commands", post(handle_command::<G>))
            .route(
                "/omarchygs/provider/v1/reconcile",
                post(handle_reconcile::<G>),
            )
            .layer(DefaultBodyLimit::max(self.config.limits.request_body_bytes))
            .with_state(Arc::clone(self))
    }

    /// Start the durable callback worker.
    pub fn spawn_callback_worker(&self) -> AnyResult<JoinHandle<()>> {
        callback::spawn_callback_worker(
            self.store.clone(),
            Arc::clone(&self.config.provider_signer),
            self.game.identity().clone(),
            self.config.release_id,
            self.config.callback.clone(),
        )
    }

    /// Serve the exact router with caller-owned TLS files until Ctrl-C.
    pub async fn serve_tls(
        self: Arc<Self>,
        bind_address: SocketAddr,
        certificate_path: &Path,
        private_key_path: &Path,
    ) -> AnyResult<()> {
        let tls = RustlsConfig::from_pem_file(certificate_path, private_key_path)
            .await
            .context("load provider TLS identity")?;
        let callback = self.spawn_callback_worker()?;
        let handle = axum_server::Handle::new();
        let shutdown = handle.clone();
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                shutdown.graceful_shutdown(Some(Duration::from_secs(5)));
            }
        });
        let result = axum_server::bind_rustls(bind_address, tls)
            .handle(handle)
            .serve(self.router().into_make_service())
            .await
            .context("provider starter server stopped unexpectedly");
        callback.abort();
        let _ = callback.await;
        result
    }

    fn compatibility(&self, headers: &HeaderMap, body: &[u8]) -> Result<Response> {
        self.validate_body(body)?;
        self.validate_host(headers)?;
        let signature = SignatureHeaders::from_header_map(headers)?;
        let message_id = signature
            .message_id
            .parse::<Uuid>()
            .map_err(|_| ProviderError::ProtocolRejected)?;
        let path = format!("{BASE_PATH}compatibility");
        let context = self.request_context(&path, message_id);
        verify_request_signature(
            &signature,
            &context,
            body,
            &self.config.platform_message_key,
            &self.config.platform_message_key_id,
            unix_seconds()?,
        )?;
        let offer: ProviderCompatibilityOffer =
            parse_authenticated_json(body, self.config.limits.request_body_bytes)?;
        offer.validate()?;
        if offer.provider_id != self.game.identity().provider_id
            || offer.release_id != self.config.release_id
            || offer.message_id != message_id
        {
            return Err(ProviderError::ProtocolRejected);
        }
        let selection = ProviderCompatibilitySelection::current(&offer, Uuid::new_v4())?;
        let response_body = serde_json::to_vec(&selection).map_err(|_| ProviderError::Internal)?;
        self.signed_response(&context, selection.message_id, response_body)
    }

    async fn operation(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        operation: ProviderOperationKind,
    ) -> Result<Response> {
        self.validate_body(body)?;
        self.validate_host(headers)?;
        let signature = SignatureHeaders::from_header_map(headers)?;
        let message_id = signature
            .message_id
            .parse::<Uuid>()
            .map_err(|_| ProviderError::ProtocolRejected)?;
        let path = format!("{BASE_PATH}{}", operation.path());
        let context = self.request_context(&path, message_id);
        verify_request_signature(
            &signature,
            &context,
            body,
            &self.config.platform_message_key,
            &self.config.platform_message_key_id,
            unix_seconds()?,
        )?;
        let request: ProviderOperationRequest =
            parse_authenticated_json(body, self.config.limits.request_body_bytes)?;
        request.validate()?;
        let identity = self.game.identity();
        if request.provider_id != identity.provider_id
            || request.release_id != self.config.release_id
            || request.game_key != identity.game_key
            || request.rules_version != identity.rules_version
            || request.cartridge_digest != identity.cartridge_digest
            || request.message_id != message_id
            || request.operation != operation
        {
            return Err(ProviderError::ProtocolRejected);
        }
        let claims = verify_grant(
            &request.grant,
            &self.config.platform_grant_key,
            &GrantExpectation {
                key_id: &self.config.platform_grant_key_id,
                provider_id: &identity.provider_id,
                release_id: self.config.release_id,
                game_key: &identity.game_key,
                rules_version: identity.rules_version,
                cartridge_digest: &identity.cartridge_digest,
                platform_session_id: request.platform_session_id,
                subject: &request.subject,
                scope: operation.scope(),
                compatibility: &request.compatibility,
            },
            unix_seconds()?,
        )?;
        let intent_digest = StarterStore::stable_intent_digest(&request)?;
        let stored = self
            .store
            .apply(
                &self.game,
                self.config.release_id,
                &request,
                claims.token_id,
                &sha256_hex(body),
                &intent_digest,
            )
            .await?;
        if self
            .config
            .limits
            .operation_response_delay_after_commit
            .is_some_and(|(kind, _)| kind == operation)
            && !stored.replayed
        {
            let (_, delay) = self
                .config
                .limits
                .operation_response_delay_after_commit
                .ok_or(ProviderError::Internal)?;
            tokio::time::sleep(delay).await;
        }
        let response: ProviderOperationResponse =
            parse_authenticated_json(&stored.body, self.config.limits.request_body_bytes)?;
        response.validate_for(&request)?;
        self.signed_response(&context, response.message_id, stored.body)
    }

    fn request_context<'a>(
        &'a self,
        path: &'a str,
        message_id: Uuid,
    ) -> RequestSignatureContext<'a> {
        RequestSignatureContext {
            method: "POST",
            authority: &self.config.authority,
            path,
            provider_id: &self.game.identity().provider_id,
            release_id: self.config.release_id,
            message_id,
        }
    }

    fn signed_response(
        &self,
        context: &RequestSignatureContext<'_>,
        message_id: Uuid,
        body: Vec<u8>,
    ) -> Result<Response> {
        let headers = self.config.provider_signer.sign_response(
            StatusCode::OK.as_u16(),
            context,
            message_id,
            &body,
            unix_seconds()?,
            &format!("starter-{}", Uuid::new_v4()),
        )?;
        let mut builder = Response::builder().status(StatusCode::OK);
        for (name, value) in &headers.to_header_map()? {
            builder = builder.header(name, value);
        }
        builder
            .body(Body::from(body))
            .map_err(|_| ProviderError::Internal)
    }

    fn validate_body(&self, body: &[u8]) -> Result<()> {
        if body.is_empty() || body.len() > self.config.limits.request_body_bytes {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(())
        }
    }

    fn validate_host(&self, headers: &HeaderMap) -> Result<()> {
        let host = headers
            .get(HOST)
            .and_then(|value| value.to_str().ok())
            .ok_or(ProviderError::ProtocolRejected)?;
        if host == self.config.authority {
            Ok(())
        } else {
            Err(ProviderError::ProtocolRejected)
        }
    }
}

async fn handle_compatibility<G: ProviderGame>(
    State(starter): State<Arc<ProviderStarter<G>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    starter
        .compatibility(&headers, &body)
        .unwrap_or_else(error_response)
}

async fn handle_launch<G: ProviderGame>(
    State(starter): State<Arc<ProviderStarter<G>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_operation(starter, headers, body, ProviderOperationKind::Launch).await
}

async fn handle_command<G: ProviderGame>(
    State(starter): State<Arc<ProviderStarter<G>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_operation(starter, headers, body, ProviderOperationKind::Command).await
}

async fn handle_reconcile<G: ProviderGame>(
    State(starter): State<Arc<ProviderStarter<G>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_operation(starter, headers, body, ProviderOperationKind::Reconcile).await
}

async fn handle_operation<G: ProviderGame>(
    starter: Arc<ProviderStarter<G>>,
    headers: HeaderMap,
    body: Bytes,
    operation: ProviderOperationKind,
) -> Response {
    starter
        .operation(&headers, &body, operation)
        .await
        .unwrap_or_else(error_response)
}

fn error_response(error: ProviderError) -> Response {
    tracing::warn!(code = error.code(), "provider starter request rejected");
    let status = match error {
        ProviderError::Conflict => StatusCode::CONFLICT,
        ProviderError::Denied | ProviderError::ProtocolRejected => StatusCode::UNAUTHORIZED,
        ProviderError::InvalidInput => StatusCode::UNPROCESSABLE_ENTITY,
        ProviderError::NotFound => StatusCode::NOT_FOUND,
        ProviderError::QuotaExceeded => StatusCode::TOO_MANY_REQUESTS,
        ProviderError::Unavailable | ProviderError::Internal => StatusCode::SERVICE_UNAVAILABLE,
    };
    Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn valid_authority(authority: &str) -> bool {
    if authority.is_empty() || authority.len() > 255 || authority.contains(['/', '@', '?', '#']) {
        return false;
    }
    Url::parse(&format!("https://{authority}/"))
        .ok()
        .is_some_and(|url| url.host_str().is_some() && url.path() == "/")
}

fn valid_key_id(value: &str) -> bool {
    (3..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn unix_seconds() -> Result<i64> {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| ProviderError::Internal)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| ProviderError::Internal)
}
