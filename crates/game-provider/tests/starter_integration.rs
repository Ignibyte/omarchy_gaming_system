#![cfg(feature = "provider-conformance")]

use std::{
    fs::Permissions,
    net::{Ipv4Addr, SocketAddr, TcpListener},
    os::unix::fs::PermissionsExt as _,
    path::Path,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, SystemTime},
};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use axum_server::tls_rustls::RustlsConfig;
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use ed25519_dalek::SigningKey;
use omarchy_game_provider::{
    ProviderError,
    broker::{BrokerOperationInput, CallbackDisposition, ProviderBroker},
    egress::SidecarTarget,
    model::{
        ActiveSessionPolicy, OperationalKeyInput, OperatorCommand, ProviderEndpoint,
        ProviderQuotas, ProviderScope, RegisterReleaseInput, SessionAdmission,
    },
    protocol::{
        GrantIssuer, HttpMessageSigner, ProviderOperationDisposition, ProviderOperationKind,
        RequestSignatureContext, SignatureHeaders,
    },
    registry::ProviderRegistry,
};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use serde_json::{Value, json};
use sqlx::PgPool;
use tempfile::TempDir;
use tokio::{net::TcpStream, process::Child};
use uuid::Uuid;

const PROVIDER_ID: &str = "relay-labs";
const HOST: &str = "relay.example.test";
const GAME_KEY: &str = "relay-forge";
const RELEASE_ID: Uuid = Uuid::from_u128(0x45454545454545458545454545454545);

struct ProviderProcess(Child);

impl Drop for ProviderProcess {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
    }
}

#[derive(Clone)]
struct CallbackState {
    broker: Arc<ProviderBroker>,
    authority: String,
    path: String,
    accepted: Arc<AtomicU32>,
    duplicates: Arc<AtomicU32>,
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL and Relay Forge; run scripts/test-provider-starter-conformance.sh"]
async fn relay_forge_uses_real_broker_with_distinct_durable_state(pool: PgPool) {
    let provider_binary = std::env::var("OMARCHYGS_RELAY_FORGE_BIN")
        .expect("Relay Forge binary path must be configured");
    let provider_database_url = std::env::var("OMARCHYGS_RELAY_FORGE_DATABASE_URL")
        .expect("Relay Forge database must be configured");
    let platform_database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("platform database identity");
    assert!(!provider_database_url.ends_with(&format!("/{platform_database}")));

    let temp = TempDir::new().expect("temporary fixture");
    let provider_address = unused_loopback_address();
    let callback_address = unused_loopback_address();
    let provider_authority = format!("{HOST}:{}", provider_address.port());
    let callback_authority = format!("callback.example.test:{}", callback_address.port());
    let callback_path = format!("/v1/provider-events/{RELEASE_ID}");

    let CertifiedKey {
        cert: provider_cert,
        signing_key: provider_tls_key,
    } = generate_simple_self_signed(vec![HOST.to_owned()]).expect("provider TLS identity");
    let CertifiedKey {
        cert: callback_cert,
        signing_key: callback_tls_key,
    } = generate_simple_self_signed(vec!["callback.example.test".to_owned()])
        .expect("callback TLS identity");
    let provider_cert_path = temp.path().join("provider-cert.pem");
    let provider_key_path = temp.path().join("provider-key.pem");
    let callback_cert_path = temp.path().join("callback-cert.pem");
    let callback_key_path = temp.path().join("callback-key.pem");
    std::fs::write(&provider_cert_path, provider_cert.pem()).expect("provider cert");
    std::fs::write(&provider_key_path, provider_tls_key.serialize_pem()).expect("provider key");
    std::fs::write(&callback_cert_path, callback_cert.pem()).expect("callback cert");
    std::fs::write(&callback_key_path, callback_tls_key.serialize_pem()).expect("callback key");

    let grant_seed = [51; 32];
    let message_seed = [52; 32];
    let provider_seed = [53; 32];
    let grant_issuer =
        GrantIssuer::new("platform-grant-1", grant_seed, vec![54; 32]).expect("grant issuer");
    let message_signer = HttpMessageSigner::new("platform-message-1", message_seed)
        .expect("platform message signer");
    let provider_signing_key = SigningKey::from_bytes(&provider_seed);
    let quotas = ProviderQuotas {
        grants_per_minute: 100,
        requests_per_minute: 100,
        callbacks_per_minute: 100,
        max_concurrent_requests: 4,
        request_body_bytes: 65_536,
        response_body_bytes: 65_536,
        connect_timeout_ms: 500,
        total_timeout_ms: 3_000,
    };
    let registry = register_release(
        &pool,
        ProviderEndpoint {
            host: HOST.to_owned(),
            port: provider_address.port(),
            base_path: "/omarchygs/provider/v1/".to_owned(),
        },
        quotas,
        provider_cert.der(),
        provider_signing_key.verifying_key().as_bytes(),
    )
    .await;
    let broker = Arc::new(ProviderBroker::sidecar(
        registry,
        grant_issuer,
        message_signer,
        SidecarTarget::new(RELEASE_ID, provider_address).expect("exact sidecar target"),
    ));

    let accepted = Arc::new(AtomicU32::new(0));
    let duplicates = Arc::new(AtomicU32::new(0));
    let callback_state = CallbackState {
        broker: Arc::clone(&broker),
        authority: callback_authority.clone(),
        path: callback_path.clone(),
        accepted: Arc::clone(&accepted),
        duplicates: Arc::clone(&duplicates),
    };
    let callback_tls = RustlsConfig::from_pem_file(&callback_cert_path, &callback_key_path)
        .await
        .expect("callback TLS config");
    let callback_handle = axum_server::Handle::new();
    let serving = callback_handle.clone();
    let callback_route = callback_path.clone();
    let callback_task = tokio::spawn(async move {
        axum_server::bind_rustls(callback_address, callback_tls)
            .handle(serving)
            .serve(
                Router::new()
                    .route(&callback_route, post(callback))
                    .with_state(callback_state)
                    .into_make_service(),
            )
            .await
    });
    wait_for_port(callback_address).await;

    let config_path = temp.path().join("provider-config.json");
    let config = json!({
        "authority": provider_authority,
        "bind_address": provider_address,
        "callback_sidecar_socket": callback_address,
        "callback_socket_override": null,
        "callback_tls_root_der_base64": URL_SAFE_NO_PAD.encode(callback_cert.der()),
        "callback_url": format!("https://{callback_authority}{callback_path}"),
        "cartridge_digest": "a".repeat(64),
        "command_response_delay_ms": 0,
        "database_url": provider_database_url,
        "platform_grant_key_id": "platform-grant-1",
        "platform_grant_public_key_base64": URL_SAFE_NO_PAD.encode(broker_grant_public(grant_seed)),
        "platform_message_key_id": "platform-message-1",
        "platform_message_public_key_base64": URL_SAFE_NO_PAD.encode(broker_message_public(message_seed)),
        "provider_message_key_id": "provider-message-1",
        "provider_message_signing_seed_base64": URL_SAFE_NO_PAD.encode(provider_seed),
        "release_id": RELEASE_ID,
        "tls_certificate": provider_cert_path,
        "tls_private_key": provider_key_path,
    });
    std::fs::write(
        &config_path,
        serde_json::to_vec(&config).expect("provider config JSON"),
    )
    .expect("provider config");
    std::fs::set_permissions(&config_path, Permissions::from_mode(0o600))
        .expect("provider config permissions");
    let mut provider =
        spawn_provider(Path::new(&provider_binary), &config_path, provider_address).await;

    let persona_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let launch = input(
        persona_id,
        session_id,
        Uuid::new_v4(),
        0,
        ProviderOperationKind::Launch,
        json!({"player_count": 1}),
    );
    let launched = broker.execute(&launch).await.expect("real broker launch");
    assert_eq!(launched.revision, 0);

    let mut revision = 0;
    for action in ["mine", "mine", "charge", "forge"] {
        let command = input(
            persona_id,
            session_id,
            Uuid::new_v4(),
            revision,
            ProviderOperationKind::Command,
            json!({"command": {"action": action}}),
        );
        let applied = broker.execute(&command).await.expect("sidecar command");
        revision += 1;
        assert_eq!(applied.revision, revision);
        assert_eq!(applied.disposition, ProviderOperationDisposition::Applied);
    }

    provider.0.start_kill().expect("crash provider");
    provider.0.wait().await.expect("reap crashed provider");
    let denied_launch = input(
        persona_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
        0,
        ProviderOperationKind::Launch,
        json!({"player_count": 1}),
    );
    assert!(matches!(
        broker.execute(&denied_launch).await,
        Err(ProviderError::Unavailable)
    ));

    let CertifiedKey {
        cert: hostile_cert,
        signing_key: hostile_key,
    } = generate_simple_self_signed(vec![HOST.to_owned()]).expect("hostile TLS identity");
    let hostile_cert_path = temp.path().join("hostile-cert.pem");
    let hostile_key_path = temp.path().join("hostile-key.pem");
    std::fs::write(&hostile_cert_path, hostile_cert.pem()).expect("hostile cert");
    std::fs::write(&hostile_key_path, hostile_key.serialize_pem()).expect("hostile key");
    let hostile_tls = RustlsConfig::from_pem_file(&hostile_cert_path, &hostile_key_path)
        .await
        .expect("hostile TLS config");
    let hostile_handle = axum_server::Handle::new();
    let serving = hostile_handle.clone();
    let hostile_task = tokio::spawn(async move {
        axum_server::bind_rustls(provider_address, hostile_tls)
            .handle(serving)
            .serve(Router::new().into_make_service())
            .await
    });
    wait_for_port(provider_address).await;
    assert!(matches!(
        broker.execute(&denied_launch).await,
        Err(ProviderError::Unavailable)
    ));
    hostile_handle.graceful_shutdown(Some(Duration::from_secs(2)));
    hostile_task
        .await
        .expect("hostile listener join")
        .expect("hostile listener stop");

    provider = spawn_provider(Path::new(&provider_binary), &config_path, provider_address).await;

    let reconcile = input(
        persona_id,
        session_id,
        Uuid::new_v4(),
        revision,
        ProviderOperationKind::Reconcile,
        json!({}),
    );
    let reconciled = broker
        .execute(&reconcile)
        .await
        .expect("real broker reconcile");
    assert_eq!(reconciled.revision, revision);

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while duplicates.load(Ordering::SeqCst) == 0 && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert_eq!(accepted.load(Ordering::SeqCst), 1);
    assert!(duplicates.load(Ordering::SeqCst) >= 1);

    let provider_pool = PgPool::connect(&provider_database_url)
        .await
        .expect("provider database");
    let provider_sessions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM provider_starter_sessions")
            .fetch_one(&provider_pool)
            .await
            .expect("provider sessions");
    assert_eq!(provider_sessions, 1);
    let platform_has_starter: bool =
        sqlx::query_scalar("SELECT to_regclass('public.provider_starter_sessions') IS NOT NULL")
            .fetch_one(&pool)
            .await
            .expect("platform schema separation");
    assert!(!platform_has_starter);

    provider.0.start_kill().expect("stop provider");
    provider.0.wait().await.expect("reap provider");
    callback_handle.graceful_shutdown(Some(Duration::from_secs(2)));
    callback_task
        .await
        .expect("callback join")
        .expect("callback server");
}

async fn callback(
    State(state): State<CallbackState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let signature = match SignatureHeaders::from_header_map(&headers) {
        Ok(value) => value,
        Err(_) => return StatusCode::UNAUTHORIZED,
    };
    let message_id = match signature.message_id.parse::<Uuid>() {
        Ok(value) => value,
        Err(_) => return StatusCode::UNAUTHORIZED,
    };
    let context = RequestSignatureContext {
        method: "POST",
        authority: &state.authority,
        path: &state.path,
        provider_id: PROVIDER_ID,
        release_id: RELEASE_ID,
        message_id,
    };
    match state
        .broker
        .ingest_callback(RELEASE_ID, &context, &signature, &body, unix_seconds())
        .await
    {
        Ok((CallbackDisposition::Accepted, _)) => {
            state.accepted.fetch_add(1, Ordering::SeqCst);
            StatusCode::SERVICE_UNAVAILABLE
        }
        Ok((CallbackDisposition::Duplicate, _)) => {
            state.duplicates.fetch_add(1, Ordering::SeqCst);
            StatusCode::ACCEPTED
        }
        Err(_) => StatusCode::UNAUTHORIZED,
    }
}

fn input(
    persona_id: Uuid,
    platform_session_id: Uuid,
    idempotency_key: Uuid,
    expected_revision: u64,
    operation: ProviderOperationKind,
    payload: Value,
) -> BrokerOperationInput {
    BrokerOperationInput {
        release_id: RELEASE_ID,
        persona_id,
        platform_session_id,
        idempotency_key,
        expected_revision,
        operation,
        session: if operation == ProviderOperationKind::Launch {
            SessionAdmission::New
        } else {
            SessionAdmission::Existing
        },
        payload,
    }
}

async fn register_release(
    pool: &PgPool,
    endpoint: ProviderEndpoint,
    quotas: ProviderQuotas,
    certificate_der: &[u8],
    provider_message_key: &[u8; 32],
) -> ProviderRegistry {
    let now: i64 = sqlx::query_scalar("SELECT EXTRACT(EPOCH FROM clock_timestamp())::BIGINT")
        .fetch_one(pool)
        .await
        .expect("database clock");
    let registry = ProviderRegistry::new(pool.clone());
    registry
        .apply_operator_command(&OperatorCommand::RegisterRelease {
            actor: "conformance-operator".to_owned(),
            reason: "prove the public starter through the real broker".to_owned(),
            registration: RegisterReleaseInput {
                provider_id: PROVIDER_ID.to_owned(),
                display_name: "Relay Forge".to_owned(),
                release_id: RELEASE_ID,
                game_key: GAME_KEY.to_owned(),
                rules_version: 1,
                cartridge_digest: "a".repeat(64),
                endpoint,
                active_session_policy: ActiveSessionPolicy::Continue,
                scopes: vec![
                    ProviderScope::Launch,
                    ProviderScope::Command,
                    ProviderScope::Reconcile,
                    ProviderScope::Event,
                ],
                message_keys: vec![OperationalKeyInput {
                    key_id: "provider-message-1".to_owned(),
                    public_material_base64: STANDARD.encode(provider_message_key),
                    valid_from: now - 60,
                    valid_until: None,
                }],
                tls_roots: vec![OperationalKeyInput {
                    key_id: "provider-tls-1".to_owned(),
                    public_material_base64: STANDARD.encode(certificate_der),
                    valid_from: now - 60,
                    valid_until: None,
                }],
                quotas,
            },
        })
        .await
        .expect("register Relay Forge only in ephemeral database");
    registry
}

fn broker_grant_public(seed: [u8; 32]) -> [u8; 32] {
    *GrantIssuer::new("platform-grant-1", seed, vec![54; 32])
        .expect("grant issuer")
        .verifying_key()
        .as_bytes()
}

fn broker_message_public(seed: [u8; 32]) -> [u8; 32] {
    *HttpMessageSigner::new("platform-message-1", seed)
        .expect("message signer")
        .verifying_key()
        .as_bytes()
}

fn unused_loopback_address() -> SocketAddr {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("loopback socket")
        .local_addr()
        .expect("loopback address")
}

async fn spawn_provider(binary: &Path, config: &Path, address: SocketAddr) -> ProviderProcess {
    let child = tokio::process::Command::new(binary)
        .arg(config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn Relay Forge");
    wait_for_port(address).await;
    ProviderProcess(child)
}

async fn wait_for_port(address: SocketAddr) {
    for _ in 0..100 {
        if TcpStream::connect(address).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("process did not bind {address}");
}

fn unix_seconds() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("clock")
            .as_secs(),
    )
    .expect("Unix time")
}
