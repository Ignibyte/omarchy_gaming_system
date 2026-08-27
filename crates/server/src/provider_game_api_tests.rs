use std::{
    env,
    net::{Ipv4Addr, SocketAddr, TcpListener},
    path::Path,
    process::Stdio,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    body::Body,
    http::{
        HeaderMap, Method, Request, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE, HOST},
    },
};
use axum_server::tls_rustls::RustlsConfig;
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use ed25519_dalek::SigningKey;
use http_body_util::BodyExt;
use omarchy_game_provider::{
    broker::ProviderBroker,
    model::{
        AchievementDefinitionInput, ActivatePilotInput, ActiveSessionPolicy, OperationalKeyInput,
        OperatorCommand, PilotStatus, ProviderEndpoint, ProviderQuotas, ProviderScope,
        RegisterReleaseInput,
    },
    protocol::{
        GrantIssuer, HttpMessageSigner, ProviderEvent, ProviderEventKind, RequestSignatureContext,
        pairwise_subject,
    },
    registry::ProviderRegistry,
};
use omarchy_game_runtime::GameRegistry;
use rcgen::{CertifiedKey, generate_simple_self_signed};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tempfile::TempDir;
use tokio::{net::TcpStream, process::Child, task::JoinHandle};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    accounts::{self, RegistrationInput},
    app::router_with_runtimes,
    cartridge_catalog_api_tests::acquisition_fixture,
    mfa::MfaCipher,
    personas::{self, CreatePersonaInput},
    provider_games::ProviderRuntime,
    sessions::{self, CreateSessionInput, SessionCreation},
    sync::SyncHub,
};

const PROVIDER_ID: &str = "ignibyte";
const PROVIDER_HOST: &str = "provider.example.test";
const CALLBACK_HOST: &str = "callbacks.example.test";
const RELEASE_ID: Uuid = Uuid::from_u128(0x19191919191919191919191919191919);
struct TestPersona {
    id: Uuid,
    token: String,
}

struct TestResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

impl TestResponse {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("response should be JSON")
    }
}

struct ProviderProcess {
    child: Child,
}

impl ProviderProcess {
    async fn stop(&mut self) {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }
}

impl Drop for ProviderProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL, an independent provider database, and the clean-clone Door Legends binary"]
async fn clean_clone_door_legends_owns_state_restarts_and_projects_results(pool: PgPool) {
    let (Ok(binary), Ok(provider_database_url)) = (
        env::var("DOOR_LEGENDS_PROVIDER_BINARY"),
        env::var("DOOR_LEGENDS_TEST_DATABASE_URL"),
    ) else {
        eprintln!(
            "skipping clean-clone provider process; stage 18 supplies its binary and database"
        );
        return;
    };
    let _ = rustls::crypto::ring::default_provider().install_default();
    let cartridge_fixture = acquisition_fixture(&pool).await;
    let cartridge_digest = cartridge_fixture.admission.archive_sha256.clone();
    let temp = TempDir::new().expect("test temp directory should create");
    let provider_address = unused_address();
    let callback_address = unused_address();
    let provider_authority = format!("{PROVIDER_HOST}:{}", provider_address.port());
    let callback_authority = format!("{CALLBACK_HOST}:{}", callback_address.port());
    let CertifiedKey {
        cert: provider_certificate,
        signing_key: provider_tls_key,
    } = generate_simple_self_signed(vec![PROVIDER_HOST.to_owned()])
        .expect("provider TLS identity should generate");
    let CertifiedKey {
        cert: callback_certificate,
        signing_key: callback_tls_key,
    } = generate_simple_self_signed(vec![CALLBACK_HOST.to_owned()])
        .expect("callback TLS identity should generate");
    let provider_certificate_path = temp.path().join("provider-cert.pem");
    let provider_key_path = temp.path().join("provider-key.pem");
    std::fs::write(&provider_certificate_path, provider_certificate.pem())
        .expect("provider certificate should write");
    std::fs::write(&provider_key_path, provider_tls_key.serialize_pem())
        .expect("provider private key should write");

    let grant_issuer = GrantIssuer::new("ogs-grant-v1", [31_u8; 32], vec![32_u8; 32])
        .expect("grant issuer should construct");
    let platform_message_signer = HttpMessageSigner::new("ogs-message-v1", [33_u8; 32])
        .expect("platform message signer should construct");
    let provider_message_seed = [34_u8; 32];
    let provider_message_key = SigningKey::from_bytes(&provider_message_seed).verifying_key();
    let registry = register_pilot(
        &pool,
        provider_address,
        provider_certificate.der(),
        provider_message_key.as_bytes(),
        &cartridge_digest,
    )
    .await;
    let broker = ProviderBroker::conformance_loopback(
        registry,
        grant_issuer,
        platform_message_signer,
        provider_address,
    );
    let runtime = ProviderRuntime::for_broker(broker, &callback_authority);
    let app = router_with_runtimes(
        pool.clone(),
        MfaCipher::test_cipher(),
        SyncHub::new(),
        GameRegistry::empty(),
        Some(runtime),
        Some(cartridge_fixture.runtime.clone()),
        std::sync::Arc::from(crate::config::DEFAULT_SERVER_NAME),
    );
    let callback_server = spawn_callback_server(
        callback_address,
        callback_certificate.der().to_vec(),
        callback_tls_key.serialize_der(),
        app.clone(),
    )
    .await;
    let mut provider = spawn_provider(
        Path::new(&binary),
        &provider_database_url,
        provider_address,
        &provider_authority,
        &provider_certificate_path,
        &provider_key_path,
        &callback_authority,
        callback_address,
        callback_certificate.der(),
        provider_message_seed,
        [31_u8; 32],
        [33_u8; 32],
        &cartridge_digest,
        None,
    )
    .await;

    let alice = create_persona(&pool, "Pilot_Alice", "pilot_alice").await;
    let stranger = create_persona(&pool, "Pilot_Stranger", "pilot_stranger").await;
    let catalog = request(app.clone(), Method::GET, "/v1/games", None).await;
    assert_eq!(catalog.status, StatusCode::OK);
    assert_eq!(catalog.json()["games"].as_array().map(Vec::len), Some(1));
    assert_eq!(catalog.json()["games"][0]["key"], "door-legends");
    assert_eq!(
        catalog.json()["games"][0]["authority"],
        "registered_provider"
    );
    assert_eq!(
        catalog.json()["games"][0]["provider_release_id"],
        RELEASE_ID.to_string()
    );

    let start_key = Uuid::new_v4();
    let start_path = format!("/v1/personas/{}/game-sessions", alice.id);
    let start = request_json(
        app.clone(),
        &start_path,
        &alice.token,
        json!({
            "idempotency_key": start_key,
            "game_key": "door-legends",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(start.status, StatusCode::CREATED, "{}", start.body);
    assert_no_store(&start);
    let session_id = start.json()["id"]
        .as_str()
        .and_then(|value| value.parse::<Uuid>().ok())
        .expect("start response should contain a session UUID");
    assert_eq!(start.json()["authority"], "registered_provider");
    assert_eq!(start.json()["availability"], "ready");
    assert_eq!(start.json()["state"]["enter_label"], "Enter the brass door");
    assert_eq!(
        start.json()["presentation"],
        json!({
            "format": "omarchygs.session-cartridge/v1",
            "publisher_id": "ignibyte",
            "game_key": "door-legends",
            "rules_version": 1,
            "cartridge_version": 2,
            "archive_sha256": cartridge_digest,
            "signed_identity_sha256": cartridge_fixture.admission.signed_identity_sha256,
            "admission_revision": cartridge_fixture.admission.admission_revision,
            "lifecycle_status": "active",
            "active_session_policy": "continue"
        })
    );
    assert_private(&start.body);
    let replay = request_json(
        app.clone(),
        &start_path,
        &alice.token,
        json!({
            "idempotency_key": start_key,
            "game_key": "door-legends",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.json()["id"], session_id.to_string());
    let pinned: (String, i64, i64) = sqlx::query_as(
        r#"
        SELECT release.archive_sha256,
               presentation.admission_revision,
               count(*) OVER ()
        FROM game_session_cartridge_presentations AS presentation
        JOIN marketplace_releases AS release
          ON release.id = presentation.marketplace_release_id
        WHERE presentation.game_session_id = $1
        "#,
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .expect("exact session cartridge should remain pinned");
    assert_eq!(pinned.0, cartridge_digest);
    assert_eq!(
        u64::try_from(pinned.1).expect("revision should fit"),
        cartridge_fixture.admission.admission_revision
    );
    assert_eq!(
        pinned.2, 1,
        "start replay must not create or repin a cartridge"
    );

    let platform_authority: (String, Option<Value>, Uuid) = sqlx::query_as(
        "SELECT authority, state, provider_release_id FROM game_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .expect("platform envelope should exist");
    assert_eq!(platform_authority.0, "registered_provider");
    assert!(
        platform_authority.1.is_none(),
        "platform must not store rules state"
    );
    assert_eq!(platform_authority.2, RELEASE_ID);
    assert!(
        sqlx::query("UPDATE game_sessions SET game_key = 'signal_siege' WHERE id = $1")
            .bind(session_id)
            .execute(&pool)
            .await
            .is_err(),
        "a cartridge-bound session identity must be immutable"
    );
    assert!(
        sqlx::query(
            "UPDATE game_session_cartridge_presentations SET admission_revision = admission_revision + 1 WHERE game_session_id = $1",
        )
        .bind(session_id)
        .execute(&pool)
        .await
        .is_err(),
        "the exact session cartridge pin must be immutable"
    );
    assert!(
        sqlx::query("DELETE FROM game_session_cartridge_presentations WHERE game_session_id = $1",)
            .bind(session_id)
            .execute(&pool)
            .await
            .is_err(),
        "a session cartridge pin must not be removable"
    );

    sqlx::query(
        r#"
        UPDATE server_cartridge_catalogs
        SET active_release_id = NULL,
            admission_revision = admission_revision + 1,
            updated_at = clock_timestamp()
        WHERE game_key = 'door-legends'
        "#,
    )
    .execute(&pool)
    .await
    .expect("catalog should advance away from the session's old pinned release");
    let current_acquisition = request(
        app.clone(),
        Method::GET,
        &format!("/v1/cartridges/door-legends/{cartridge_digest}/acquisition"),
        Some(&alice.token),
    )
    .await;
    assert_eq!(current_acquisition.status, StatusCode::NOT_FOUND);
    let historical_acquisition_path = format!(
        "/v1/personas/{}/game-sessions/{session_id}/cartridge-acquisition",
        alice.id
    );
    let historical_acquisition = request(
        app.clone(),
        Method::GET,
        &historical_acquisition_path,
        Some(&alice.token),
    )
    .await;
    assert_eq!(
        historical_acquisition.status,
        StatusCode::OK,
        "{}",
        historical_acquisition.body
    );
    assert_eq!(
        historical_acquisition.json()["server_admission"],
        json!({
            "server_id": cartridge_fixture.admission.server_id,
            "game_key": "door-legends",
            "publisher_id": "ignibyte",
            "rules_version": 1,
            "cartridge_version": 2,
            "archive_sha256": cartridge_digest,
            "signed_identity_sha256": cartridge_fixture.admission.signed_identity_sha256,
            "admission_revision": cartridge_fixture.admission.admission_revision
        })
    );
    let foreign_historical_acquisition = request(
        app.clone(),
        Method::GET,
        &format!(
            "/v1/personas/{}/game-sessions/{session_id}/cartridge-acquisition",
            stranger.id
        ),
        Some(&stranger.token),
    )
    .await;
    assert_eq!(foreign_historical_acquisition.status, StatusCode::NOT_FOUND);

    let foreign = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions/{session_id}", stranger.id),
        Some(&stranger.token),
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);

    let command_key = Uuid::new_v4();
    let command_path = format!(
        "/v1/personas/{}/game-sessions/{session_id}/cartridge-actions",
        alice.id
    );
    let wrong_digest = request_json(
        app.clone(),
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "expected_revision": 0,
            "archive_sha256": "f".repeat(64),
            "screen_id": "lobby",
            "action": "enter",
            "payload": {}
        }),
    )
    .await;
    assert_eq!(wrong_digest.status, StatusCode::CONFLICT);
    assert_eq!(
        wrong_digest.json()["error"]["code"],
        "session_cartridge_unavailable"
    );
    let undeclared_payload = request_json(
        app.clone(),
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "expected_revision": 0,
            "archive_sha256": cartridge_digest,
            "screen_id": "chronicle",
            "action": "enter",
            "payload": {"credential": "not-forwarded"}
        }),
    )
    .await;
    assert_eq!(undeclared_payload.status, StatusCode::CONFLICT);
    let foreign_action = request_json(
        app.clone(),
        &format!(
            "/v1/personas/{}/game-sessions/{session_id}/cartridge-actions",
            stranger.id
        ),
        &stranger.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "expected_revision": 0,
            "archive_sha256": cartridge_digest,
            "screen_id": "lobby",
            "action": "enter",
            "payload": {}
        }),
    )
    .await;
    assert_eq!(foreign_action.status, StatusCode::NOT_FOUND);
    let navigation_injection = request_json(
        app.clone(),
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "expected_revision": 0,
            "archive_sha256": cartridge_digest,
            "screen_id": "lobby",
            "action": "navigate.chronicle",
            "payload": {}
        }),
    )
    .await;
    assert_eq!(navigation_injection.status, StatusCode::CONFLICT);
    let command = request_json(
        app.clone(),
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": command_key,
            "expected_revision": 0,
            "archive_sha256": cartridge_digest,
            "screen_id": "chronicle",
            "action": "enter",
            "payload": {}
        }),
    )
    .await;
    assert_eq!(command.status, StatusCode::OK, "{}", command.body);
    assert_eq!(command.json()["revision"], 1);
    assert_eq!(command.json()["status"], "completed");
    assert_eq!(command.json()["authority"], "registered_provider");
    assert_eq!(command.json()["archive_sha256"], cartridge_digest);
    assert_private(&command.body);
    let command_replay = request_json(
        app.clone(),
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": command_key,
            "expected_revision": 0,
            "archive_sha256": cartridge_digest,
            "screen_id": "chronicle",
            "action": "enter",
            "payload": {}
        }),
    )
    .await;
    assert_eq!(command_replay.status, StatusCode::OK);
    assert_eq!(command_replay.json()["revision"], 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM game_session_cartridge_action_admissions WHERE game_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("provider cartridge admission count should read"),
        1,
        "provider replay must reuse one durable cartridge admission"
    );
    let mut lifecycle_writer = pool.begin().await.expect("lifecycle writer should begin");
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(omarchy_gaming_system_server::cartridge_catalog::SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *lifecycle_writer)
        .await
        .expect("lifecycle writer should lock");
    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET signed_policy = $1,
            policy_version = 2,
            policy_status = 'suspended',
            policy_reason = 'Review paused.',
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $2
        "#,
    )
    .bind(sqlx::types::Json(&cartridge_fixture.suspended_policy))
    .bind(&cartridge_digest)
    .execute(&mut *lifecycle_writer)
    .await
    .expect("valid suspended lifecycle should stage");
    lifecycle_writer
        .commit()
        .await
        .expect("valid suspended lifecycle should commit");
    let post_suspension_replay = request_json(
        app.clone(),
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": command_key,
            "expected_revision": 0,
            "archive_sha256": cartridge_digest,
            "screen_id": "chronicle",
            "action": "enter",
            "payload": {}
        }),
    )
    .await;
    assert_eq!(
        post_suspension_replay.status,
        StatusCode::OK,
        "{}",
        post_suspension_replay.body
    );
    assert_eq!(post_suspension_replay.json()["revision"], 1);

    let provider_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&provider_database_url)
        .await
        .expect("provider database should remain independently reachable");
    wait_for_result(&pool, &provider_pool, session_id).await;
    wait_for_outbox_delivered(&provider_pool, session_id).await;
    sqlx::query(
        "UPDATE door_legends_event_outbox SET status = 'pending', delivered_at = NULL WHERE platform_session_id = $1",
    )
    .bind(session_id)
    .execute(&provider_pool)
    .await
    .expect("test should requeue the exact signed callback");
    wait_for_outbox_attempts(&provider_pool, session_id, 2).await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM provider_message_receipts WHERE release_id = $1 AND direction = 'callback'",
        )
        .bind(RELEASE_ID)
        .fetch_one(&pool)
        .await
        .expect("callback receipt should count"),
        1,
        "exact callback replay must not create a second receipt"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM persona_provider_achievements WHERE game_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("achievement projection should count"),
        1,
        "exact callback replay must not duplicate platform effects"
    );
    let ignored_event = ProviderEvent::new(
        PROVIDER_ID.to_owned(),
        RELEASE_ID,
        "door-legends".to_owned(),
        1,
        cartridge_digest.clone(),
        session_id,
        pairwise_subject(&[32_u8; 32], PROVIDER_ID, "door-legends", alice.id)
            .expect("pairwise callback subject should derive"),
        Uuid::new_v4(),
        Uuid::new_v4(),
        1,
        ProviderEventKind::ResultAvailable,
        json!({
            "outcome": "escaped",
            "public_summary": {"ending": "unapproved_rewrite"},
            "achievements": ["unapproved_claim"],
            "view": {
                "chronicle_label": "Read the chronicle",
                "enter_label": "Play again later",
                "lobby_label": "Return to the lobby",
                "status": "You escaped through the sunlit gate.",
                "welcome": "Door Legends remembers your first escape."
            }
        }),
    );
    let ignored_body = serde_json::to_vec(&ignored_event).expect("callback body should serialize");
    let callback_path = format!("/v1/provider-events/{RELEASE_ID}");
    let callback_context = RequestSignatureContext {
        method: "POST",
        authority: &callback_authority,
        path: &callback_path,
        provider_id: PROVIDER_ID,
        release_id: RELEASE_ID,
        message_id: ignored_event.message_id,
    };
    let callback_signer = HttpMessageSigner::new("door-legends-message-v1", provider_message_seed)
        .expect("provider callback signer should construct");
    let mut callback_headers = callback_signer
        .sign_request(
            &callback_context,
            &ignored_body,
            unix_seconds(),
            "policy-event-0001",
        )
        .expect("callback should sign")
        .to_header_map()
        .expect("callback headers should render");
    callback_headers.insert(
        HOST,
        callback_authority
            .parse()
            .expect("callback authority should be a header value"),
    );
    let mut tampered_body = ignored_body.clone();
    tampered_body.push(b' ');
    let tampered = send_callback(
        app.clone(),
        &callback_path,
        callback_headers.clone(),
        tampered_body,
    )
    .await;
    assert_eq!(tampered, StatusCode::UNAUTHORIZED);
    let ignored = send_callback(app.clone(), &callback_path, callback_headers, ignored_body).await;
    assert_eq!(ignored, StatusCode::ACCEPTED);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM provider_game_results WHERE game_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("result projection should count"),
        1,
        "authenticated events outside platform policy must not mutate projections"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM provider_message_receipts WHERE release_id = $1 AND direction = 'callback'",
        )
        .bind(RELEASE_ID)
        .fetch_one(&pool)
        .await
        .expect("callback receipts should count"),
        2,
        "only the accepted and authenticated-ignored callbacks should be durable"
    );
    let detail = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions/{session_id}", alice.id),
        Some(&alice.token),
    )
    .await;
    assert_eq!(detail.status, StatusCode::OK);
    assert_eq!(detail.json()["result"]["outcome"], "escaped");
    assert_eq!(
        detail.json()["result"]["public_summary"]["ending"],
        "sunlit_gate"
    );
    assert_private(&detail.body);
    let achievements = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/achievements", alice.id),
        Some(&alice.token),
    )
    .await;
    assert_eq!(achievements.status, StatusCode::OK);
    assert_eq!(
        achievements.json()["achievements"][0]["key"],
        "first_escape"
    );
    assert_eq!(
        achievements.json()["achievements"][0]["game_session_id"],
        session_id.to_string()
    );
    assert_private(&achievements.body);

    provider.stop().await;
    let unavailable = request_json(
        app.clone(),
        &format!(
            "/v1/personas/{}/game-sessions/{session_id}/reconcile",
            alice.id
        ),
        &alice.token,
        json!({"idempotency_key": Uuid::new_v4(), "expected_revision": 1}),
    )
    .await;
    assert_eq!(unavailable.status, StatusCode::SERVICE_UNAVAILABLE);
    provider = spawn_provider(
        Path::new(&binary),
        &provider_database_url,
        provider_address,
        &provider_authority,
        &provider_certificate_path,
        &provider_key_path,
        &callback_authority,
        callback_address,
        callback_certificate.der(),
        provider_message_seed,
        [31_u8; 32],
        [33_u8; 32],
        &cartridge_digest,
        None,
    )
    .await;
    let recovered = request_json(
        app.clone(),
        &format!(
            "/v1/personas/{}/game-sessions/{session_id}/reconcile",
            alice.id
        ),
        &alice.token,
        json!({"idempotency_key": Uuid::new_v4(), "expected_revision": 1}),
    )
    .await;
    assert_eq!(recovered.status, StatusCode::OK, "{}", recovered.body);
    assert_eq!(recovered.json()["revision"], 1);
    assert_eq!(recovered.json()["availability"], "ready");

    provider.stop().await;
    provider = spawn_provider(
        Path::new(&binary),
        &provider_database_url,
        provider_address,
        &provider_authority,
        &provider_certificate_path,
        &provider_key_path,
        &callback_authority,
        callback_address,
        callback_certificate.der(),
        provider_message_seed,
        [31_u8; 32],
        [33_u8; 32],
        &cartridge_digest,
        Some(3_500),
    )
    .await;
    let timeout_reconcile_key = Uuid::new_v4();
    let timeout_reconcile_path = format!(
        "/v1/personas/{}/game-sessions/{session_id}/reconcile",
        alice.id
    );
    let timed_out = request_json(
        app.clone(),
        &timeout_reconcile_path,
        &alice.token,
        json!({
            "idempotency_key": timeout_reconcile_key,
            "expected_revision": 1
        }),
    )
    .await;
    assert_eq!(timed_out.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM door_legends_operation_receipts WHERE platform_session_id = $1 AND idempotency_key = $2 AND operation = 'reconcile'",
        )
        .bind(session_id)
        .bind(timeout_reconcile_key)
        .fetch_one(&provider_pool)
        .await
        .expect("provider reconciliation receipt should count"),
        1,
        "the provider must commit its stable receipt before the broker times out"
    );

    provider.stop().await;
    provider = spawn_provider(
        Path::new(&binary),
        &provider_database_url,
        provider_address,
        &provider_authority,
        &provider_certificate_path,
        &provider_key_path,
        &callback_authority,
        callback_address,
        callback_certificate.der(),
        provider_message_seed,
        [31_u8; 32],
        [33_u8; 32],
        &cartridge_digest,
        None,
    )
    .await;
    let recovered_timeout = request_json(
        app.clone(),
        &timeout_reconcile_path,
        &alice.token,
        json!({
            "idempotency_key": timeout_reconcile_key,
            "expected_revision": 1
        }),
    )
    .await;
    assert_eq!(
        recovered_timeout.status,
        StatusCode::OK,
        "{}",
        recovered_timeout.body
    );
    assert_eq!(recovered_timeout.json()["revision"], 1);
    assert!(
        !callback_server.is_finished(),
        "callback TLS server must survive provider restarts"
    );

    let race_start = request_json(
        app.clone(),
        &start_path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "game_key": "door-legends",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(
        race_start.status,
        StatusCode::CREATED,
        "{}",
        race_start.body
    );
    let race_session_id = race_start.json()["id"]
        .as_str()
        .and_then(|value| value.parse::<Uuid>().ok())
        .expect("race session should have a UUID");
    let race_path = format!(
        "/v1/personas/{}/game-sessions/{race_session_id}/commands",
        alice.id
    );
    let race_key_a = Uuid::new_v4();
    let race_key_b = Uuid::new_v4();
    let (race_a, race_b) = tokio::join!(
        request_json(
            app.clone(),
            &race_path,
            &alice.token,
            json!({
                "idempotency_key": race_key_a,
                "expected_revision": 0,
                "command": {"action": "enter"}
            }),
        ),
        request_json(
            app.clone(),
            &race_path,
            &alice.token,
            json!({
                "idempotency_key": race_key_b,
                "expected_revision": 0,
                "command": {"action": "enter"}
            }),
        )
    );
    assert!(
        matches!(
            (race_a.status, race_b.status),
            (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::CONFLICT, StatusCode::OK)
        ),
        "one expected-revision command must win: a={} b={}",
        race_a.body,
        race_b.body
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM door_legends_sessions WHERE platform_session_id = $1 AND revision = 1",
        )
        .bind(race_session_id)
        .fetch_one(&provider_pool)
        .await
        .expect("racing provider session should count"),
        1,
        "concurrent commands must advance the authoritative provider once"
    );
    wait_for_result(&pool, &provider_pool, race_session_id).await;
    wait_for_outbox_delivered(&provider_pool, race_session_id).await;

    let lifecycle_start = request_json(
        app.clone(),
        &start_path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "game_key": "door-legends",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(
        lifecycle_start.status,
        StatusCode::CREATED,
        "{}",
        lifecycle_start.body
    );
    let lifecycle_session_id = lifecycle_start.json()["id"]
        .as_str()
        .and_then(|value| value.parse::<Uuid>().ok())
        .expect("lifecycle session should have a UUID");
    let lifecycle_registry = ProviderRegistry::new(pool.clone());
    lifecycle_registry
        .apply_operator_command(&OperatorCommand::SetPilotStatus {
            actor: "pilot-operator".to_owned(),
            reason: "prove suspended pilot containment".to_owned(),
            release_id: RELEASE_ID,
            status: PilotStatus::Suspended,
        })
        .await
        .expect("pilot should suspend");
    let suspended_command_path = format!(
        "/v1/personas/{}/game-sessions/{lifecycle_session_id}/commands",
        alice.id
    );
    let suspended_command = request_json(
        app.clone(),
        &suspended_command_path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "expected_revision": 0,
            "command": {"action": "enter"}
        }),
    )
    .await;
    assert_eq!(suspended_command.status, StatusCode::CONFLICT);
    let suspended_reconcile_path = format!(
        "/v1/personas/{}/game-sessions/{lifecycle_session_id}/reconcile",
        alice.id
    );
    let suspended_reconcile = request_json(
        app.clone(),
        &suspended_reconcile_path,
        &alice.token,
        json!({"idempotency_key": Uuid::new_v4(), "expected_revision": 0}),
    )
    .await;
    assert_eq!(
        suspended_reconcile.status,
        StatusCode::OK,
        "{}",
        suspended_reconcile.body
    );
    assert_eq!(suspended_reconcile.json()["availability"], "suspended");
    let suspended_event = ProviderEvent::new(
        PROVIDER_ID.to_owned(),
        RELEASE_ID,
        "door-legends".to_owned(),
        1,
        cartridge_digest.clone(),
        lifecycle_session_id,
        pairwise_subject(&[32_u8; 32], PROVIDER_ID, "door-legends", alice.id)
            .expect("pairwise callback subject should derive"),
        Uuid::new_v4(),
        Uuid::new_v4(),
        0,
        ProviderEventKind::TurnReady,
        json!({
            "view": {
                "chronicle_label": "Read the chronicle",
                "enter_label": "Enter the brass door",
                "lobby_label": "Return to the lobby",
                "status": "A weathered brass door waits in the dark.",
                "welcome": "Welcome to Door Legends. One choice opens the way."
            }
        }),
    );
    let suspended_body =
        serde_json::to_vec(&suspended_event).expect("suspended callback should serialize");
    let suspended_context = RequestSignatureContext {
        method: "POST",
        authority: &callback_authority,
        path: &callback_path,
        provider_id: PROVIDER_ID,
        release_id: RELEASE_ID,
        message_id: suspended_event.message_id,
    };
    let mut suspended_headers = callback_signer
        .sign_request(
            &suspended_context,
            &suspended_body,
            unix_seconds(),
            "suspended-event-0001",
        )
        .expect("suspended callback should sign")
        .to_header_map()
        .expect("suspended callback headers should render");
    suspended_headers.insert(
        HOST,
        callback_authority
            .parse()
            .expect("callback authority should be a header value"),
    );
    assert_eq!(
        send_callback(
            app.clone(),
            &callback_path,
            suspended_headers,
            suspended_body,
        )
        .await,
        StatusCode::UNAUTHORIZED,
    );
    lifecycle_registry
        .apply_operator_command(&OperatorCommand::SetPilotStatus {
            actor: "pilot-operator".to_owned(),
            reason: "restore only after authenticated reconciliation".to_owned(),
            release_id: RELEASE_ID,
            status: PilotStatus::Active,
        })
        .await
        .expect("pilot should reactivate");
    let active_reconcile = request_json(
        app.clone(),
        &suspended_reconcile_path,
        &alice.token,
        json!({"idempotency_key": Uuid::new_v4(), "expected_revision": 0}),
    )
    .await;
    assert_eq!(
        active_reconcile.status,
        StatusCode::OK,
        "{}",
        active_reconcile.body
    );
    assert_eq!(active_reconcile.json()["availability"], "ready");
    lifecycle_registry
        .apply_operator_command(&OperatorCommand::SetPilotStatus {
            actor: "pilot-operator".to_owned(),
            reason: "prove permanent pilot retirement".to_owned(),
            release_id: RELEASE_ID,
            status: PilotStatus::Retired,
        })
        .await
        .expect("pilot should retire");
    let retired_command = request_json(
        app.clone(),
        &suspended_command_path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "expected_revision": 0,
            "command": {"action": "enter"}
        }),
    )
    .await;
    assert_eq!(retired_command.status, StatusCode::CONFLICT);
    let retired_catalog = request(app.clone(), Method::GET, "/v1/games", None).await;
    assert_eq!(retired_catalog.status, StatusCode::OK);
    assert!(
        retired_catalog.json()["games"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "retired pilot must remain absent from the catalog",
    );
    provider.stop().await;
    callback_server.abort();
}

async fn register_pilot(
    pool: &PgPool,
    address: SocketAddr,
    certificate_der: &[u8],
    provider_message_key: &[u8; 32],
    cartridge_digest: &str,
) -> ProviderRegistry {
    let now: i64 = sqlx::query_scalar("SELECT EXTRACT(EPOCH FROM clock_timestamp())::BIGINT")
        .fetch_one(pool)
        .await
        .expect("database clock should read");
    let registry = ProviderRegistry::new(pool.clone());
    registry
        .apply_operator_command(&OperatorCommand::RegisterRelease {
            actor: "pilot-operator".to_owned(),
            reason: "activate the first-party Door Legends authority pilot".to_owned(),
            registration: RegisterReleaseInput {
                provider_id: PROVIDER_ID.to_owned(),
                display_name: "Ignibyte First-Party Games".to_owned(),
                release_id: RELEASE_ID,
                game_key: "door-legends".to_owned(),
                rules_version: 1,
                cartridge_digest: cartridge_digest.to_owned(),
                endpoint: ProviderEndpoint {
                    host: PROVIDER_HOST.to_owned(),
                    port: address.port(),
                    base_path: "/omarchygs/provider/v1/".to_owned(),
                },
                active_session_policy: ActiveSessionPolicy::ReadOnly,
                scopes: vec![
                    ProviderScope::Launch,
                    ProviderScope::Command,
                    ProviderScope::Reconcile,
                    ProviderScope::Event,
                ],
                message_keys: vec![OperationalKeyInput {
                    key_id: "door-legends-message-v1".to_owned(),
                    public_material_base64: STANDARD.encode(provider_message_key),
                    valid_from: now - 60,
                    valid_until: None,
                }],
                tls_roots: vec![OperationalKeyInput {
                    key_id: "door-legends-tls-v1".to_owned(),
                    public_material_base64: STANDARD.encode(certificate_der),
                    valid_from: now - 60,
                    valid_until: None,
                }],
                quotas: ProviderQuotas {
                    grants_per_minute: 100,
                    requests_per_minute: 100,
                    callbacks_per_minute: 100,
                    max_concurrent_requests: 8,
                    request_body_bytes: 16 * 1024,
                    response_body_bytes: 16 * 1024,
                    connect_timeout_ms: 500,
                    total_timeout_ms: 3_000,
                },
            },
        })
        .await
        .expect("Door Legends release should register");
    registry
        .apply_operator_command(&OperatorCommand::ActivatePilot {
            actor: "pilot-operator".to_owned(),
            reason: "enable the sole first-party remote authority pilot".to_owned(),
            pilot: ActivatePilotInput {
                release_id: RELEASE_ID,
                display_name: "Door Legends".to_owned(),
                min_human_players: 1,
                max_human_players: 1,
                achievements: vec![AchievementDefinitionInput {
                    key: "first_escape".to_owned(),
                    display_name: "First Escape".to_owned(),
                    description: "Escape through the sunlit gate.".to_owned(),
                }],
            },
        })
        .await
        .expect("Door Legends pilot should activate");
    registry
}

#[allow(clippy::too_many_arguments)]
async fn spawn_provider(
    binary: &Path,
    database_url: &str,
    address: SocketAddr,
    authority: &str,
    certificate_path: &Path,
    private_key_path: &Path,
    callback_authority: &str,
    callback_address: SocketAddr,
    callback_root: &[u8],
    provider_seed: [u8; 32],
    grant_seed: [u8; 32],
    message_seed: [u8; 32],
    cartridge_digest: &str,
    reconcile_response_delay_ms: Option<u64>,
) -> ProviderProcess {
    let grant_key = SigningKey::from_bytes(&grant_seed).verifying_key();
    let message_key = SigningKey::from_bytes(&message_seed).verifying_key();
    let mut command = tokio::process::Command::new(binary);
    command
        .env("RUST_LOG", "door_legends_provider=info")
        .env("DATABASE_URL", database_url)
        .env("DOOR_LEGENDS_BIND_ADDRESS", address.to_string())
        .env("DOOR_LEGENDS_TLS_CERTIFICATE", certificate_path)
        .env("DOOR_LEGENDS_TLS_PRIVATE_KEY", private_key_path)
        .env("DOOR_LEGENDS_RELEASE_ID", RELEASE_ID.to_string())
        .env("DOOR_LEGENDS_CARTRIDGE_DIGEST", cartridge_digest)
        .env("DOOR_LEGENDS_AUTHORITY", authority)
        .env(
            "OGS_PROVIDER_GRANT_PUBLIC_KEY",
            URL_SAFE_NO_PAD.encode(grant_key.as_bytes()),
        )
        .env(
            "OGS_PROVIDER_MESSAGE_PUBLIC_KEY",
            URL_SAFE_NO_PAD.encode(message_key.as_bytes()),
        )
        .env(
            "DOOR_LEGENDS_MESSAGE_SIGNING_SEED",
            URL_SAFE_NO_PAD.encode(provider_seed),
        )
        .env(
            "DOOR_LEGENDS_CALLBACK_URL",
            format!("https://{callback_authority}/v1/provider-events/{RELEASE_ID}"),
        )
        .env(
            "DOOR_LEGENDS_CALLBACK_TLS_ROOT_DER",
            URL_SAFE_NO_PAD.encode(callback_root),
        )
        .env(
            "DOOR_LEGENDS_CALLBACK_SOCKET_OVERRIDE",
            callback_address.to_string(),
        );
    if let Some(milliseconds) = reconcile_response_delay_ms {
        command.env(
            "DOOR_LEGENDS_RECONCILE_RESPONSE_DELAY_MS",
            milliseconds.to_string(),
        );
    }
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("clean-clone Door Legends provider should spawn");
    wait_for_listener(address).await;
    ProviderProcess { child }
}

async fn spawn_callback_server(
    address: SocketAddr,
    certificate_der: Vec<u8>,
    private_key_der: Vec<u8>,
    app: Router,
) -> JoinHandle<()> {
    let tls = RustlsConfig::from_der(vec![certificate_der], private_key_der)
        .await
        .expect("callback TLS config should load");
    let task = tokio::spawn(async move {
        axum_server::bind_rustls(address, tls)
            .serve(app.into_make_service())
            .await
            .expect("callback test server should run");
    });
    wait_for_listener(address).await;
    task
}

async fn send_callback(app: Router, path: &str, headers: HeaderMap, body: Vec<u8>) -> StatusCode {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(path)
        .body(Body::from(body))
        .expect("callback request should build");
    *request.headers_mut() = headers;
    app.oneshot(request)
        .await
        .expect("callback request should complete")
        .status()
}

fn unix_seconds() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow epoch")
            .as_secs(),
    )
    .expect("Unix time should fit i64")
}

async fn create_persona(pool: &PgPool, username: &str, handle: &str) -> TestPersona {
    accounts::register_account(
        pool,
        RegistrationInput {
            invite_code: accounts::create_test_invite(pool).await,
            username: username.to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await
    .expect("test account should register");
    let token = match sessions::create_session(
        pool,
        CreateSessionInput {
            username: username.to_owned(),
            password: "correct horse battery staple".to_owned(),
            device_name: "provider pilot test".to_owned(),
        },
    )
    .await
    .expect("test session should create")
    {
        SessionCreation::Created(created) => created.token,
        SessionCreation::MfaRequired(_) => panic!("new account should not require MFA"),
    };
    let persona = personas::create_persona(
        pool,
        &token,
        CreatePersonaInput {
            handle: handle.to_owned(),
            display_name: format!("{handle} display"),
            bio: String::new(),
            status_message: String::new(),
        },
    )
    .await
    .expect("test persona should create");
    TestPersona {
        id: persona.id,
        token,
    }
}

async fn request(app: Router, method: Method, path: &str, token: Option<&str>) -> TestResponse {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    collect(
        app.oneshot(builder.body(Body::empty()).expect("request should build"))
            .await
            .expect("router should respond"),
    )
    .await
}

async fn request_json(app: Router, path: &str, token: &str, body: Value) -> TestResponse {
    collect(
        app.oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("JSON request should build"),
        )
        .await
        .expect("router should respond"),
    )
    .await
}

async fn collect(response: axum::response::Response) -> TestResponse {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body should read")
        .to_bytes();
    TestResponse {
        status,
        headers,
        body: String::from_utf8(body.to_vec()).expect("response should be UTF-8"),
    }
}

async fn wait_for_result(pool: &PgPool, provider_pool: &PgPool, session_id: Uuid) {
    for _ in 0..200 {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM provider_game_results WHERE game_session_id = $1)",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("result projection should query");
        if exists {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let platform: Option<(i64, String, Option<String>)> = sqlx::query_as(
        "SELECT revision, status, provider_availability FROM game_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .expect("platform callback diagnostics should query");
    let outbox: Option<(String, i32)> = sqlx::query_as(
        "SELECT status, attempt_count FROM door_legends_event_outbox WHERE platform_session_id = $1",
    )
    .bind(session_id)
    .fetch_optional(provider_pool)
    .await
    .expect("provider callback diagnostics should query");
    panic!(
        "provider result callback did not project for {session_id}: platform={platform:?} outbox={outbox:?}"
    );
}

async fn wait_for_outbox_delivered(pool: &PgPool, session_id: Uuid) {
    for _ in 0..200 {
        let delivered: bool = sqlx::query_scalar(
            "SELECT status = 'delivered' FROM door_legends_event_outbox WHERE platform_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("outbox delivery status should query");
        if delivered {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("provider callback outbox did not deliver for {session_id}");
}

async fn wait_for_outbox_attempts(pool: &PgPool, session_id: Uuid, expected: i32) {
    for _ in 0..100 {
        let attempts: i32 = sqlx::query_scalar(
            "SELECT attempt_count FROM door_legends_event_outbox WHERE platform_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("outbox attempt count should query");
        if attempts >= expected {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("provider callback replay did not complete");
}

async fn wait_for_listener(address: SocketAddr) {
    for _ in 0..100 {
        if TcpStream::connect(address).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("process did not listen on {address}");
}

fn unused_address() -> SocketAddr {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral port should bind")
        .local_addr()
        .expect("ephemeral address should resolve")
}

fn assert_no_store(response: &TestResponse) {
    assert_eq!(
        response
            .headers
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
}

fn assert_private(body: &str) {
    for forbidden in [
        "provider.example.test",
        "callbacks.example.test",
        "pairwise_subject",
        "signed_grant",
        "database_url",
        "account_id",
        "request_sha256",
    ] {
        assert!(!body.contains(forbidden), "response leaked {forbidden}");
    }
}
