#![cfg(feature = "provider-conformance")]

use std::{
    fs::Permissions,
    net::{Ipv4Addr, SocketAddr, TcpListener},
    os::unix::fs::PermissionsExt as _,
    path::Path,
    process::Stdio,
    time::{Duration, SystemTime},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::SigningKey;
use http::{HeaderMap, HeaderName, HeaderValue};
use omarchy_game_provider::{
    ProviderError,
    broker::{BrokerOperationInput, CallbackDisposition, ProviderBroker},
    egress::GuardedProviderClient,
    model::{
        ActiveSessionPolicy, OperationalKeyInput, OperatorCommand, ProviderEndpoint,
        ProviderQuotas, ProviderScope, RegisterReleaseInput, SessionAdmission,
    },
    protocol::{
        GrantIssuer, HttpMessageSigner, ProviderOperationDisposition, ProviderOperationKind,
        RequestSignatureContext, SignatureHeaders, sha256_hex,
    },
    registry::ProviderRegistry,
};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use serde_json::{Value, json};
use sqlx::{PgPool, Row as _};
use tempfile::TempDir;
use tokio::{net::TcpStream, process::Child};
use uuid::Uuid;

const PROVIDER_ID: &str = "fixture-provider";
const HOST: &str = "provider.example.test";
const BASE_PATH: &str = "/omarchygs/provider/v1/";
const GAME_KEY: &str = "signal_siege";
const RELEASE_ID: Uuid = Uuid::from_u128(0x18181818181818181818181818181818);

struct FixtureProcess {
    child: Child,
}

impl FixtureProcess {
    async fn stop(&mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }
}

impl Drop for FixtureProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL and the separately compiled TLS fixture"]
async fn separate_tls_provider_proves_replay_faults_events_outage_and_reconciliation(pool: PgPool) {
    let temp = TempDir::new().expect("fixture temp directory should create");
    let address = unused_loopback_address();
    let authority = format!("{HOST}:{}", address.port());
    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(vec![HOST.to_owned()]).expect("TLS fixture certificate");
    let certificate_der = cert.der().to_vec();
    let private_key_der = signing_key.serialize_der();
    let provider_signing_seed = [31_u8; 32];
    let provider_signing_key = SigningKey::from_bytes(&provider_signing_seed);
    let grant_issuer = GrantIssuer::new("platform-grant-1", [32; 32], vec![33; 32])
        .expect("grant issuer should construct");
    let platform_message_signer = HttpMessageSigner::new("platform-message-1", [34; 32])
        .expect("message signer should construct");
    let config_path = temp.path().join("fixture-config.json");
    let state_path = temp.path().join("fixture-state.json");
    let mut config = json!({
        "listen": address,
        "certificate_der_base64": STANDARD.encode(&certificate_der),
        "private_key_der_base64": STANDARD.encode(private_key_der),
        "provider_id": PROVIDER_ID,
        "release_id": RELEASE_ID,
        "game_key": GAME_KEY,
        "rules_version": 1,
        "cartridge_digest": "c".repeat(64),
        "endpoint_authority": authority,
        "endpoint_base_path": BASE_PATH,
        "platform_grant_key_id": "platform-grant-1",
        "platform_grant_public_key_base64": STANDARD.encode(grant_issuer.verifying_key().as_bytes()),
        "platform_message_key_id": "platform-message-1",
        "platform_message_public_key_base64": STANDARD.encode(platform_message_signer.verifying_key().as_bytes()),
        "provider_message_key_id": "provider-message-1",
        "provider_message_signing_seed_base64": STANDARD.encode(provider_signing_seed),
        "state_path": state_path,
        "commit_delay_ms": 2200,
        "compatibility_delay_ms": 0,
        "compatibility_fault": "strip_capability"
    });
    std::fs::write(
        &config_path,
        serde_json::to_vec(&config).expect("fixture config should serialize"),
    )
    .expect("fixture config should write");
    std::fs::set_permissions(&config_path, Permissions::from_mode(0o600))
        .expect("fixture config permissions should restrict");

    let mut fixture = spawn_fixture(&config_path, address).await;
    let quotas = ProviderQuotas {
        grants_per_minute: 50,
        requests_per_minute: 50,
        callbacks_per_minute: 50,
        max_concurrent_requests: 4,
        request_body_bytes: 16 * 1024,
        response_body_bytes: 8 * 1024,
        connect_timeout_ms: 250,
        total_timeout_ms: 2_000,
    };
    let endpoint = ProviderEndpoint {
        host: HOST.to_owned(),
        port: address.port(),
        base_path: BASE_PATH.to_owned(),
    };
    let registry = register_release(
        &pool,
        endpoint.clone(),
        quotas.clone(),
        &certificate_der,
        provider_signing_key.verifying_key().as_bytes(),
    )
    .await;
    let broker = ProviderBroker::conformance_loopback(
        registry.clone(),
        grant_issuer,
        platform_message_signer,
        address,
    );
    let persona_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    let platform_session_id = Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb);

    let launch = operation_input(
        persona_id,
        platform_session_id,
        Uuid::new_v4(),
        0,
        ProviderOperationKind::Launch,
        json!({"mode": "solo"}),
    );
    assert!(matches!(
        broker.execute(&launch).await,
        Err(ProviderError::ProtocolRejected)
    ));
    let grants_before_compatibility: i64 =
        sqlx::query_scalar("SELECT count(*) FROM provider_grants WHERE release_id = $1")
            .bind(RELEASE_ID)
            .fetch_one(&pool)
            .await
            .expect("grant count should query");
    let attempts_before_compatibility: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM provider_operation_attempts WHERE release_id = $1",
    )
    .bind(RELEASE_ID)
    .fetch_one(&pool)
    .await
    .expect("operation attempt count should query");
    assert_eq!(grants_before_compatibility, 0);
    assert_eq!(attempts_before_compatibility, 0);
    assert!(
        !state_path.exists(),
        "rejected preflight must not persist provider state"
    );
    fixture.stop().await;
    config["compatibility_fault"] = Value::Null;
    std::fs::write(
        &config_path,
        serde_json::to_vec(&config).expect("updated fixture config should serialize"),
    )
    .expect("updated fixture config should write");
    std::fs::set_permissions(&config_path, Permissions::from_mode(0o600))
        .expect("updated fixture config permissions should restrict");
    fixture = spawn_fixture(&config_path, address).await;
    let launched = execute_with_audit(&broker, &pool, &launch, "TLS launch should pass").await;
    assert_eq!(launched.revision, 0);
    assert_eq!(launched.disposition, ProviderOperationDisposition::Applied);

    let command_id = Uuid::new_v4();
    let command = operation_input(
        persona_id,
        platform_session_id,
        command_id,
        0,
        ProviderOperationKind::Command,
        json!({"action": "advance"}),
    );
    let applied = execute_with_audit(&broker, &pool, &command, "command should apply").await;
    assert_eq!(applied.revision, 1);
    assert_eq!(
        broker
            .execute(&command)
            .await
            .expect("exact replay should resolve"),
        applied
    );
    let conflict = operation_input(
        persona_id,
        platform_session_id,
        command_id,
        0,
        ProviderOperationKind::Command,
        json!({"action": "different"}),
    );
    assert!(matches!(
        broker.execute(&conflict).await,
        Err(ProviderError::Conflict)
    ));

    let stale = operation_input(
        persona_id,
        platform_session_id,
        Uuid::new_v4(),
        0,
        ProviderOperationKind::Command,
        json!({"action": "stale"}),
    );
    let stale_response = broker
        .execute(&stale)
        .await
        .expect("stale revision is signed data");
    assert_eq!(stale_response.revision, 1);
    assert_eq!(
        stale_response.disposition,
        ProviderOperationDisposition::RevisionConflict
    );

    let event_record = read_last_event(&state_path);
    let event_body = STANDARD
        .decode(event_record["body_base64"].as_str().expect("event body"))
        .expect("event body should decode");
    let event_headers = header_map(&event_record["headers"]);
    let parsed_event_headers =
        SignatureHeaders::from_header_map(&event_headers).expect("event headers should parse");
    let event_message_id = parsed_event_headers
        .message_id
        .parse()
        .expect("event message ID should parse");
    let event_path = format!("{BASE_PATH}events");
    let event_context = RequestSignatureContext {
        method: "POST",
        authority: &authority,
        path: &event_path,
        provider_id: PROVIDER_ID,
        release_id: RELEASE_ID,
        message_id: event_message_id,
    };
    let callback_quota_before: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(used), 0)::BIGINT FROM provider_quota_windows WHERE release_id = $1 AND quota_kind = 'callback'",
    )
    .bind(RELEASE_ID)
    .fetch_one(&pool)
    .await
    .expect("callback quota should query");
    let mut invalidly_signed_body = event_body.clone();
    invalidly_signed_body.push(b' ');
    assert!(matches!(
        broker
            .ingest_callback(
                RELEASE_ID,
                &event_context,
                &parsed_event_headers,
                &invalidly_signed_body,
                unix_seconds(),
            )
            .await,
        Err(ProviderError::ProtocolRejected)
    ));
    let callback_quota_after: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(used), 0)::BIGINT FROM provider_quota_windows WHERE release_id = $1 AND quota_kind = 'callback'",
    )
    .bind(RELEASE_ID)
    .fetch_one(&pool)
    .await
    .expect("callback quota should query after rejection");
    assert_eq!(
        callback_quota_after, callback_quota_before,
        "unauthenticated callbacks must not consume provider quota",
    );
    let (first_event, racing_duplicate) = tokio::join!(
        broker.ingest_callback(
            RELEASE_ID,
            &event_context,
            &parsed_event_headers,
            &event_body,
            unix_seconds(),
        ),
        broker.ingest_callback(
            RELEASE_ID,
            &event_context,
            &parsed_event_headers,
            &event_body,
            unix_seconds(),
        )
    );
    let first_event = first_event.expect("signed fixture event should ingest");
    let racing_duplicate = racing_duplicate.expect("racing replay should deduplicate");
    assert!(matches!(
        (first_event.0, racing_duplicate.0),
        (
            CallbackDisposition::Accepted,
            CallbackDisposition::Duplicate
        ) | (
            CallbackDisposition::Duplicate,
            CallbackDisposition::Accepted
        )
    ));
    let duplicate_event = broker
        .ingest_callback(
            RELEASE_ID,
            &event_context,
            &parsed_event_headers,
            &event_body,
            unix_seconds(),
        )
        .await
        .expect("exact event replay should deduplicate");
    assert_eq!(duplicate_event.0, CallbackDisposition::Duplicate);

    let mut legacy_event: Value =
        serde_json::from_slice(&event_body).expect("current event should parse as JSON");
    legacy_event
        .as_object_mut()
        .expect("event should be an object")
        .remove("compatibility");
    let legacy_body = serde_json::to_vec(&legacy_event).expect("legacy event should encode");
    sqlx::query(
        "UPDATE provider_message_receipts SET authenticated_sha256 = $3 WHERE release_id = $1 AND direction = 'callback' AND message_id = $2",
    )
    .bind(RELEASE_ID)
    .bind(event_message_id)
    .bind(sha256_hex(&legacy_body))
    .execute(&pool)
    .await
    .expect("legacy callback receipt should seed");
    let provider_message_signer =
        HttpMessageSigner::new("provider-message-1", provider_signing_seed)
            .expect("provider message signer should construct");
    let legacy_headers = provider_message_signer
        .sign_request(
            &event_context,
            &legacy_body,
            unix_seconds(),
            &format!("legacy-{}", Uuid::new_v4()),
        )
        .expect("legacy callback retry should sign");
    let legacy_duplicate = broker
        .ingest_callback(
            RELEASE_ID,
            &event_context,
            &legacy_headers,
            &legacy_body,
            unix_seconds(),
        )
        .await
        .expect("exact persisted legacy callback should deduplicate");
    assert_eq!(legacy_duplicate.0, CallbackDisposition::Duplicate);

    let new_message_id = Uuid::new_v4();
    legacy_event["message_id"] = json!(new_message_id);
    legacy_event["event_id"] = json!(Uuid::new_v4());
    let new_legacy_body =
        serde_json::to_vec(&legacy_event).expect("new legacy event should encode");
    let new_legacy_context = RequestSignatureContext {
        message_id: new_message_id,
        ..event_context
    };
    let new_legacy_headers = provider_message_signer
        .sign_request(
            &new_legacy_context,
            &new_legacy_body,
            unix_seconds(),
            &format!("legacy-{}", Uuid::new_v4()),
        )
        .expect("new legacy callback should sign");
    assert!(matches!(
        broker
            .ingest_callback(
                RELEASE_ID,
                &new_legacy_context,
                &new_legacy_headers,
                &new_legacy_body,
                unix_seconds(),
            )
            .await,
        Err(ProviderError::ProtocolRejected)
    ));

    let commit_timeout = operation_input(
        persona_id,
        platform_session_id,
        Uuid::new_v4(),
        1,
        ProviderOperationKind::Command,
        json!({"action": "advance", "fault": "commit_then_timeout"}),
    );
    fixture.stop().await;
    config["compatibility_delay_ms"] = json!(1_100);
    config["commit_delay_ms"] = json!(1_200);
    std::fs::write(
        &config_path,
        serde_json::to_vec(&config).expect("deadline fixture config should serialize"),
    )
    .expect("deadline fixture config should write");
    std::fs::set_permissions(&config_path, Permissions::from_mode(0o600))
        .expect("deadline fixture config permissions should restrict");
    fixture = spawn_fixture(&config_path, address).await;
    assert!(matches!(
        broker.execute(&commit_timeout).await,
        Err(ProviderError::Unavailable)
    ));
    fixture.stop().await;
    config["compatibility_delay_ms"] = json!(0);
    config["commit_delay_ms"] = json!(2_200);
    std::fs::write(
        &config_path,
        serde_json::to_vec(&config).expect("restored fixture config should serialize"),
    )
    .expect("restored fixture config should write");
    std::fs::set_permissions(&config_path, Permissions::from_mode(0o600))
        .expect("restored fixture config permissions should restrict");
    fixture = spawn_fixture(&config_path, address).await;
    let recovered = broker
        .execute(&commit_timeout)
        .await
        .expect("stable provider receipt should recover timeout");
    assert_eq!(recovered.revision, 2);

    for fault in ["bad_signature", "redirect", "oversized"] {
        let rejected = operation_input(
            persona_id,
            platform_session_id,
            Uuid::new_v4(),
            2,
            ProviderOperationKind::Reconcile,
            json!({"fault": fault}),
        );
        assert!(
            broker.execute(&rejected).await.is_err(),
            "{fault} should fail closed"
        );
    }

    let wrong_certificate = generate_simple_self_signed(vec![HOST.to_owned()])
        .expect("wrong TLS certificate should generate")
        .cert
        .der()
        .to_vec();
    let wrong_tls_client = GuardedProviderClient::conformance_loopback(
        endpoint,
        address,
        &[wrong_certificate],
        quotas,
    )
    .expect("wrong-root client should still construct");
    assert!(matches!(
        wrong_tls_client
            .post("reconcile", HeaderMap::new(), vec![b'x'])
            .await,
        Err(ProviderError::Unavailable)
    ));

    fixture.stop().await;
    let outage = operation_input(
        persona_id,
        platform_session_id,
        Uuid::new_v4(),
        2,
        ProviderOperationKind::Reconcile,
        json!({}),
    );
    assert!(matches!(
        broker.execute(&outage).await,
        Err(ProviderError::Unavailable)
    ));
    fixture = spawn_fixture(&config_path, address).await;
    let reconciled = broker
        .execute(&outage)
        .await
        .expect("restarted provider should reconcile durable state");
    assert_eq!(reconciled.revision, 2);
    fixture.stop().await;

    let unsafe_storage: Vec<String> = sqlx::query(
        r#"
        SELECT row_to_json(value)::text
        FROM (
            SELECT token_id, release_id, platform_session_id, pairwise_subject, scope,
                   claims_sha256, issued_at, expires_at
            FROM provider_grants
            WHERE release_id = $1
        ) value
        "#,
    )
    .bind(RELEASE_ID)
    .fetch_all(&pool)
    .await
    .expect("grant evidence should query")
    .into_iter()
    .map(|row| row.get(0))
    .collect();
    let retained = unsafe_storage.join("\n");
    assert!(!retained.contains(&persona_id.to_string()));
    assert!(!retained.contains("account_id"));
    assert!(!retained.contains("device_token"));
    let failed_audits: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM provider_security_audit_events WHERE release_id = $1 AND outcome IN ('failed', 'denied')",
    )
    .bind(RELEASE_ID)
    .fetch_one(&pool)
    .await
    .expect("failure audits should query");
    assert!(
        failed_audits >= 5,
        "faults should retain safe audit evidence"
    );
}

fn operation_input(
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

async fn execute_with_audit(
    broker: &ProviderBroker,
    pool: &PgPool,
    input: &BrokerOperationInput,
    expectation: &str,
) -> omarchy_game_provider::protocol::ProviderOperationResponse {
    match broker.execute(input).await {
        Ok(response) => response,
        Err(error) => {
            let evidence: Vec<(String, String)> = sqlx::query_as(
                "SELECT event_type, reason_code FROM provider_security_audit_events WHERE release_id = $1 ORDER BY sequence",
            )
            .bind(RELEASE_ID)
            .fetch_all(pool)
            .await
            .expect("diagnostic audit evidence should query");
            panic!("{expectation}: {error:?}; audit={evidence:?}");
        }
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
        .expect("database clock should read");
    let registry = ProviderRegistry::new(pool.clone());
    registry
        .apply_operator_command(&OperatorCommand::RegisterRelease {
            actor: "conformance-operator".to_owned(),
            reason: "exercise the production provider security boundary".to_owned(),
            registration: RegisterReleaseInput {
                provider_id: PROVIDER_ID.to_owned(),
                display_name: "TLS Fixture Provider".to_owned(),
                release_id: RELEASE_ID,
                game_key: GAME_KEY.to_owned(),
                rules_version: 1,
                cartridge_digest: "c".repeat(64),
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
        .expect("TLS fixture release should register");
    registry
}

fn unused_loopback_address() -> SocketAddr {
    let listener =
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("ephemeral loopback port should bind");
    listener
        .local_addr()
        .expect("loopback address should resolve")
}

async fn spawn_fixture(config_path: &Path, address: SocketAddr) -> FixtureProcess {
    let child = tokio::process::Command::new(env!("CARGO_BIN_EXE_omarchygs-provider-fixture"))
        .arg(config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("fixture process should spawn");
    for _ in 0..100 {
        if TcpStream::connect(address).await.is_ok() {
            return FixtureProcess { child };
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("fixture process did not bind {address}");
}

fn read_last_event(path: &Path) -> Value {
    let state: Value = serde_json::from_slice(
        &std::fs::read(path).expect("fixture durable state should be readable"),
    )
    .expect("fixture durable state should parse");
    state["last_event"].clone()
}

fn header_map(value: &Value) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in value
        .as_object()
        .expect("event headers should be an object")
    {
        headers.insert(
            HeaderName::from_bytes(name.as_bytes()).expect("event header name should parse"),
            HeaderValue::from_str(value.as_str().expect("event header value should be text"))
                .expect("event header value should parse"),
        );
    }
    headers
}

fn unix_seconds() -> i64 {
    i64::try_from(
        SystemTime::UNIX_EPOCH
            .elapsed()
            .expect("system clock should follow epoch")
            .as_secs(),
    )
    .expect("Unix time should fit i64")
}
