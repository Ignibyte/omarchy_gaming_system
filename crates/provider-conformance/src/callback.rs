use std::{
    collections::BTreeMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use anyhow::{Context as _, Result, anyhow};
use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use ed25519_dalek::VerifyingKey;
use omarchygs_provider_sdk::protocol::{
    ProviderEvent, RequestSignatureContext, SignatureHeaders, parse_authenticated_json, sha256_hex,
    verify_request_signature,
};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use uuid::Uuid;

const MAX_EVENT_BYTES: usize = 65_536;

/// Exact callback identity and TLS files for a local conformance sink.
#[derive(Clone)]
pub struct CallbackSinkConfig {
    pub bind_address: SocketAddr,
    pub authority: String,
    pub path: String,
    pub provider_id: String,
    pub release_id: Uuid,
    pub game_key: String,
    pub rules_version: u32,
    pub cartridge_digest: String,
    pub subject: String,
    pub provider_message_key_id: String,
    pub provider_message_key: VerifyingKey,
    pub certificate_pem: PathBuf,
    pub private_key_pem: PathBuf,
}

/// Secret-free facts retained for one authenticated event identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CallbackObservation {
    pub event_id: Uuid,
    pub deliveries: u32,
    pub platform_session_id: Uuid,
    pub revision: u64,
    pub body_sha256: String,
}

#[derive(Clone)]
struct SinkState {
    config: CallbackSinkConfig,
    observations: Arc<Mutex<BTreeMap<Uuid, CallbackObservation>>>,
}

/// Running exact TLS callback sink. The first authenticated delivery returns
/// 503 to force durable retry; the duplicate returns 202.
pub struct CallbackSink {
    observations: Arc<Mutex<BTreeMap<Uuid, CallbackObservation>>>,
    server: axum_server::Handle<SocketAddr>,
    task: JoinHandle<Result<()>>,
}

impl CallbackSink {
    /// Bind the fixed callback route using caller-owned test TLS files.
    pub async fn start(config: CallbackSinkConfig) -> Result<Self> {
        validate_config(&config)?;
        let observations = Arc::new(Mutex::new(BTreeMap::new()));
        let state = SinkState {
            config: config.clone(),
            observations: Arc::clone(&observations),
        };
        let router = Router::new()
            .route(&config.path, post(receive))
            .with_state(state);
        let tls = axum_server::tls_rustls::RustlsConfig::from_pem_file(
            &config.certificate_pem,
            &config.private_key_pem,
        )
        .await
        .context("load callback sink TLS identity")?;
        let server = axum_server::Handle::new();
        let serving = server.clone();
        let address = config.bind_address;
        let task = tokio::spawn(async move {
            axum_server::bind_rustls(address, tls)
                .handle(serving)
                .serve(router.into_make_service())
                .await
                .context("callback sink stopped unexpectedly")
        });
        wait_for_listener(address).await?;
        Ok(Self {
            observations,
            server,
            task,
        })
    }

    /// Snapshot authenticated observations in stable event-ID order.
    pub fn observations(&self) -> Result<Vec<CallbackObservation>> {
        self.observations
            .lock()
            .map(|values| values.values().cloned().collect())
            .map_err(|_| anyhow!("callback observation lock poisoned"))
    }

    /// Wait until the expected session/revision event has been delivered twice.
    pub async fn wait_for_duplicate(
        &self,
        platform_session_id: Uuid,
        revision: u64,
        timeout: Duration,
    ) -> Result<CallbackObservation> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(value) = self.observations()?.into_iter().find(|value| {
                value.platform_session_id == platform_session_id
                    && value.revision == revision
                    && value.deliveries >= 2
            }) {
                return Ok(value);
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(anyhow!("authenticated callback duplicate did not arrive"));
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    /// Gracefully stop and reap the local server.
    pub async fn stop(self) -> Result<()> {
        self.server.graceful_shutdown(Some(Duration::from_secs(2)));
        self.task.await.context("join callback sink")??;
        Ok(())
    }
}

async fn receive(State(state): State<SinkState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    match admit(&state, &headers, &body) {
        Ok(1) => StatusCode::SERVICE_UNAVAILABLE,
        Ok(_) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::UNAUTHORIZED,
    }
}

fn admit(state: &SinkState, headers: &HeaderMap, body: &[u8]) -> Result<u32> {
    let signature = SignatureHeaders::from_header_map(headers)
        .map_err(|_| anyhow!("callback signature headers rejected"))?;
    let message_id = signature
        .message_id
        .parse::<Uuid>()
        .map_err(|_| anyhow!("callback message ID rejected"))?;
    let context = RequestSignatureContext {
        method: "POST",
        authority: &state.config.authority,
        path: &state.config.path,
        provider_id: &state.config.provider_id,
        release_id: state.config.release_id,
        message_id,
    };
    verify_request_signature(
        &signature,
        &context,
        body,
        &state.config.provider_message_key,
        &state.config.provider_message_key_id,
        unix_seconds()?,
    )
    .map_err(|_| anyhow!("callback signature rejected"))?;
    let event: ProviderEvent = parse_authenticated_json(body, MAX_EVENT_BYTES)
        .map_err(|_| anyhow!("callback event rejected"))?;
    event
        .validate()
        .map_err(|_| anyhow!("callback event rejected"))?;
    if event.provider_id != state.config.provider_id
        || event.release_id != state.config.release_id
        || event.game_key != state.config.game_key
        || event.rules_version != state.config.rules_version
        || event.cartridge_digest != state.config.cartridge_digest
        || event.subject != state.config.subject
        || event.message_id != message_id
    {
        return Err(anyhow!("callback identity mismatch"));
    }
    let mut values = state
        .observations
        .lock()
        .map_err(|_| anyhow!("callback observation lock poisoned"))?;
    let value = values.entry(event.event_id).or_insert(CallbackObservation {
        event_id: event.event_id,
        deliveries: 0,
        platform_session_id: event.platform_session_id,
        revision: event.revision,
        body_sha256: sha256_hex(body),
    });
    if value.platform_session_id != event.platform_session_id
        || value.revision != event.revision
        || value.body_sha256 != sha256_hex(body)
    {
        return Err(anyhow!("callback event identity changed"));
    }
    value.deliveries = value
        .deliveries
        .checked_add(1)
        .ok_or_else(|| anyhow!("callback delivery count overflow"))?;
    Ok(value.deliveries)
}

fn validate_config(config: &CallbackSinkConfig) -> Result<()> {
    if !config.bind_address.ip().is_loopback()
        || config.authority.is_empty()
        || config.path != format!("/v1/provider-events/{}", config.release_id)
        || config.provider_id.is_empty()
        || config.release_id.is_nil()
        || config.game_key.is_empty()
        || config.rules_version == 0
        || config.cartridge_digest.len() != 64
        || config.subject.is_empty()
    {
        return Err(anyhow!("callback sink requires exact loopback identity"));
    }
    Ok(())
}

async fn wait_for_listener(address: SocketAddr) -> Result<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        if tokio::net::TcpStream::connect(address).await.is_ok() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(anyhow!("callback sink did not become ready"));
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn unix_seconds() -> Result<i64> {
    i64::try_from(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("system clock before Unix epoch")?
            .as_secs(),
    )
    .context("system clock overflow")
}
