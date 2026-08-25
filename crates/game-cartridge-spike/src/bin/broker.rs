use std::{collections::HashSet, env, net::SocketAddr, path::Path, sync::Arc, time::Duration};

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
    ErrorDocument, MAX_GRANT_LIFETIME_SECONDS, MAX_PROVIDER_BODY_BYTES, MessageExpectation,
    PLATFORM_ISSUER, PLATFORM_KEY_ID, PROVIDER_KEY_ID, PUBLISHER_KEY_ID, ProofResponse,
    ProviderCommandRequest, ProviderGrant, ProviderLaunchRequest, ProviderMessage,
    ProviderMessageKind, SignedEnvelope, VerifiedCartridge, load_signing_key, load_verifying_key,
    now_unix_seconds, pairwise_subject, sign_envelope, validate_provider_message,
    validate_spike_provider_endpoint, verify_cartridge, verify_envelope,
};
use reqwest::{Client, Url, redirect::Policy};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct BrokerState {
    config: Arc<BrokerConfig>,
}

struct BrokerConfig {
    provider_url: Url,
    platform_private_key: SigningKey,
    provider_public_key: VerifyingKey,
    pairwise_secret: Vec<u8>,
    cartridge: VerifiedCartridge,
    client: Client,
}

#[derive(Debug)]
struct BrokerError {
    status: StatusCode,
    code: &'static str,
}

impl BrokerError {
    fn unavailable() -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "provider_unavailable",
        }
    }
}

impl IntoResponse for BrokerError {
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
    let bind_address = required_env("OGS_SPIKE_BROKER_BIND")?
        .parse::<SocketAddr>()
        .context("OGS_SPIKE_BROKER_BIND must be a socket address")?;
    if !bind_address.ip().is_loopback() {
        bail!("the proof broker may bind only to loopback");
    }
    let provider_url = validate_spike_provider_endpoint(&required_env("OGS_SPIKE_PROVIDER_URL")?)
        .context("proof provider URL failed the loopback allowlist")?;
    let publisher_public_key =
        load_verifying_key(Path::new(&required_env("OGS_SPIKE_PUBLISHER_PUBLIC_KEY")?))
            .context("failed to load publisher public key")?;
    let cartridge = verify_cartridge(
        Path::new(&required_env("OGS_SPIKE_CARTRIDGE_DIR")?),
        PUBLISHER_KEY_ID,
        &publisher_public_key,
    )
    .context("cartridge verification failed")?;
    let registered_publisher = required_env("OGS_SPIKE_PUBLISHER_ID")?;
    let registered_provider = required_env("OGS_SPIKE_PROVIDER_ID")?;
    if cartridge.manifest.publisher_id != registered_publisher
        || cartridge.manifest.provider_id != registered_provider
    {
        bail!("cartridge identity does not match the proof registry");
    }
    let pairwise_secret = required_env("OGS_SPIKE_PAIRWISE_SECRET")?.into_bytes();
    if pairwise_secret.len() < 32 {
        bail!("OGS_SPIKE_PAIRWISE_SECRET must contain at least 32 bytes");
    }
    let client = Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(4))
        .build()
        .context("failed to build proof provider client")?;
    let state = BrokerState {
        config: Arc::new(BrokerConfig {
            provider_url,
            platform_private_key: load_signing_key(Path::new(&required_env(
                "OGS_SPIKE_PLATFORM_PRIVATE_KEY",
            )?))
            .context("failed to load platform private key")?,
            provider_public_key: load_verifying_key(Path::new(&required_env(
                "OGS_SPIKE_PROVIDER_PUBLIC_KEY",
            )?))
            .context("failed to load provider public key")?,
            pairwise_secret,
            cartridge,
            client,
        }),
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/proof", post(proof))
        .layer(DefaultBodyLimit::max(1024))
        .with_state(state);
    let listener = TcpListener::bind(bind_address)
        .await
        .with_context(|| format!("failed to bind broker at {bind_address}"))?;
    println!("broker listening at http://{bind_address}");
    axum::serve(listener, app)
        .await
        .context("broker stopped unexpectedly")
}

async fn health() -> &'static str {
    "ok"
}

async fn proof(State(state): State<BrokerState>) -> Result<Json<ProofResponse>, BrokerError> {
    println!("broker proof request received");
    let result = run_proof(&state.config).await.map_err(|error| {
        eprintln!("proof failed: {error:#}");
        BrokerError::unavailable()
    })?;
    Ok(Json(result))
}

async fn run_proof(config: &BrokerConfig) -> Result<ProofResponse> {
    let manifest = &config.cartridge.manifest;
    let platform_session_id = Uuid::new_v4();
    let internal_persona_id = Uuid::new_v4();
    let subject = pairwise_subject(
        &config.pairwise_secret,
        &manifest.provider_id,
        &manifest.game_key,
        internal_persona_id,
    )
    .context("pairwise persona derivation failed")?;

    let launch_grant = grant(
        config,
        &subject,
        platform_session_id,
        "game.launch",
        Uuid::new_v4(),
    )?;
    let launch_envelope = post_provider(
        config,
        "launch",
        &ProviderLaunchRequest {
            grant: launch_grant,
        },
    )
    .await?;
    let launch = verify_provider_message(
        config,
        &launch_envelope,
        platform_session_id,
        ProviderMessageKind::Launch,
    )?;
    if launch.revision != 0 {
        bail!("provider launch returned a nonzero revision");
    }

    let idempotency_key = Uuid::new_v4();
    let first_command = ProviderCommandRequest {
        grant: grant(
            config,
            &subject,
            platform_session_id,
            "game.command",
            Uuid::new_v4(),
        )?,
        idempotency_key,
        expected_revision: launch.revision,
        action_id: "advance".to_owned(),
    };
    let first_envelope = post_provider(config, "commands", &first_command).await?;
    let first = verify_provider_message(
        config,
        &first_envelope,
        platform_session_id,
        ProviderMessageKind::CommandResult,
    )?;
    if first.revision != 1 || first.provider_session_id != launch.provider_session_id {
        bail!("provider revision/session invariant failed");
    }

    let retry_command = ProviderCommandRequest {
        grant: grant(
            config,
            &subject,
            platform_session_id,
            "game.command",
            Uuid::new_v4(),
        )?,
        ..first_command
    };
    let retry_envelope = post_provider(config, "commands", &retry_command).await?;
    let retry = verify_provider_message(
        config,
        &retry_envelope,
        platform_session_id,
        ProviderMessageKind::CommandResult,
    )?;
    let idempotent_replay = retry_envelope == first_envelope && retry == first;
    if !idempotent_replay {
        bail!("provider did not replay the exact idempotent receipt");
    }

    let mut event_receipts = HashSet::new();
    if !event_receipts.insert(first.event_id) {
        bail!("fresh provider event was already present");
    }
    let duplicate_event_rejected = !event_receipts.insert(retry.event_id);
    if !duplicate_event_rejected {
        bail!("duplicate provider event was accepted");
    }

    Ok(ProofResponse {
        status: "ready".to_owned(),
        title: manifest.display_name.clone(),
        detail: "signed cartridge and isolated provider command verified".to_owned(),
        revision: first.revision,
        pairwise_subject_verified: subject.len() == 43,
        cartridge_digest: config.cartridge.digest.clone(),
        idempotent_replay,
        duplicate_event_rejected,
        raw_persona_disclosed: false,
        device_token_disclosed: false,
        database_access_disclosed: false,
        presentation: config.cartridge.presentation.clone(),
        view: first.view,
    })
}

fn grant(
    config: &BrokerConfig,
    subject: &str,
    platform_session_id: Uuid,
    scope: &str,
    token_id: Uuid,
) -> Result<SignedEnvelope> {
    let now = now_unix_seconds().context("clock unavailable")?;
    let manifest = &config.cartridge.manifest;
    let value = ProviderGrant {
        issuer: PLATFORM_ISSUER.to_owned(),
        audience: manifest.provider_id.clone(),
        subject: subject.to_owned(),
        provider_id: manifest.provider_id.clone(),
        game_key: manifest.game_key.clone(),
        game_version: manifest.rules_version,
        cartridge_digest: config.cartridge.digest.clone(),
        platform_session_id,
        issued_at: now,
        expires_at: now + MAX_GRANT_LIFETIME_SECONDS,
        token_id,
        scopes: vec![scope.to_owned()],
    };
    sign_envelope(&value, PLATFORM_KEY_ID, &config.platform_private_key)
        .context("failed to sign provider grant")
}

async fn post_provider<T: serde::Serialize>(
    config: &BrokerConfig,
    path: &str,
    body: &T,
) -> Result<SignedEnvelope> {
    let url = config
        .provider_url
        .join(path)
        .context("invalid registered provider path")?;
    let mut response = config
        .client
        .post(url)
        .json(body)
        .send()
        .await
        .context("provider request failed")?;
    if !response.status().is_success() {
        bail!("provider rejected request with {}", response.status());
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_PROVIDER_BODY_BYTES as u64)
    {
        bail!("provider response exceeded declared limit");
    }
    let mut bytes = Vec::with_capacity(
        response
            .content_length()
            .unwrap_or(0)
            .min(MAX_PROVIDER_BODY_BYTES as u64) as usize,
    );
    while let Some(chunk) = response
        .chunk()
        .await
        .context("failed to read provider response")?
    {
        if bytes
            .len()
            .checked_add(chunk.len())
            .is_none_or(|length| length > MAX_PROVIDER_BODY_BYTES)
        {
            bail!("provider response exceeded limit");
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).context("provider response was not a signed envelope")
}

fn verify_provider_message(
    config: &BrokerConfig,
    envelope: &SignedEnvelope,
    platform_session_id: Uuid,
    kind: ProviderMessageKind,
) -> Result<ProviderMessage> {
    let message =
        verify_envelope::<ProviderMessage>(envelope, PROVIDER_KEY_ID, &config.provider_public_key)
            .context("provider message signature failed")?;
    let manifest = &config.cartridge.manifest;
    validate_provider_message(
        &message,
        &MessageExpectation {
            kind,
            provider_id: &manifest.provider_id,
            game_key: &manifest.game_key,
            game_version: manifest.rules_version,
            cartridge_digest: &config.cartridge.digest,
            platform_session_id,
        },
    )
    .context("provider message binding failed")?;
    Ok(message)
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}
