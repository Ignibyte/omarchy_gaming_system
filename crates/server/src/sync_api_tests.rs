//! Database-backed contract tests for durable persona sync and live hints.

use std::time::Duration;

use axum::{
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header::CONTENT_TYPE},
};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::{net::TcpListener, time::timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WebSocketError, Message as ClientMessage, client::IntoClientRequest},
};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    app::{router, router_with_sync_hub},
    mfa::MfaCipher,
    sync::{self, SyncEventKind, SyncHub},
};

struct TestResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

impl TestResponse {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("response should contain valid JSON")
    }
}

struct TestPersona {
    token: String,
    id: Uuid,
}

#[test]
fn live_socket_limits_are_bounded_and_permits_are_released() {
    let hub = SyncHub::new();
    let account_id = Uuid::from_u128(1);
    let persona_id = Uuid::from_u128(1);
    let permits = (0..5)
        .map(|_| {
            hub.acquire(account_id, persona_id)
                .expect("first five sockets should fit")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        hub.acquire(account_id, persona_id).err(),
        Some(sync::SyncError::SocketLimitReached)
    );
    drop(permits);
    assert!(hub.acquire(account_id, persona_id).is_ok());

    let account_hub = SyncHub::new();
    let account_permits = (1..=20)
        .map(|index| {
            account_hub
                .acquire(account_id, Uuid::from_u128(index))
                .expect("first twenty account sockets should fit")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        account_hub.acquire(account_id, Uuid::from_u128(21)).err(),
        Some(sync::SyncError::SocketLimitReached)
    );
    assert!(
        account_hub
            .acquire(Uuid::from_u128(2), Uuid::from_u128(21))
            .is_ok(),
        "one account must not consume another account's allowance"
    );
    drop(account_permits);
    assert!(account_hub.acquire(account_id, Uuid::from_u128(21)).is_ok());

    let process_hub = SyncHub::new();
    let process_permits = (1..=256)
        .map(|index| {
            process_hub
                .acquire(Uuid::from_u128(index), Uuid::from_u128(index))
                .expect("process socket capacity should fit")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        process_hub
            .acquire(Uuid::from_u128(257), Uuid::from_u128(257))
            .err(),
        Some(sync::SyncError::SocketLimitReached)
    );
    drop(process_permits);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn mutations_emit_minimal_owner_local_events_and_noop_retries_emit_none(pool: PgPool) {
    let alice = create_test_persona(&pool, "Sync_Alice", "sync_alice").await;
    let bob = create_test_persona(&pool, "Sync_Bob", "sync_bob").await;

    let baseline = sync_page(&pool, &alice, None, None).await;
    assert_no_store(&baseline);
    assert_eq!(
        baseline.json(),
        json!({
            "events": [],
            "next_cursor": 0,
            "has_more": false,
            "reset_required": false
        })
    );
    let foreign = request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/sync", alice.id),
        &bob.token,
        None,
    )
    .await;
    let absent = request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/sync", Uuid::nil()),
        &bob.token,
        None,
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign.body, absent.body);
    assert_no_store(&foreign);
    for invalid_query in ["after=-1", "limit=0", "limit=101"] {
        let response = request(
            &pool,
            Method::GET,
            &format!("/v1/personas/{}/sync?{invalid_query}", alice.id),
            &alice.token,
            None,
        )
        .await;
        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_no_store(&response);
    }

    let request_path = format!("/v1/personas/{}/connection-requests/{}", alice.id, bob.id);
    assert_eq!(
        request(&pool, Method::PUT, &request_path, &alice.token, None)
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        request(&pool, Method::PUT, &request_path, &alice.token, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_event_types(
        &sync_page(&pool, &alice, Some(0), None).await.json(),
        &["connection_requests_changed"],
    );
    assert_event_types(
        &sync_page(&pool, &bob, Some(0), None).await.json(),
        &["connection_requests_changed"],
    );

    let acceptance_path = format!("/v1/personas/{}/connections/{}", bob.id, alice.id);
    for _ in 0..2 {
        assert_eq!(
            request(&pool, Method::PUT, &acceptance_path, &bob.token, None)
                .await
                .status,
            StatusCode::OK
        );
    }
    let conversation_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM inbox_conversations")
        .fetch_one(&pool)
        .await
        .expect("acceptance should create a conversation");
    for persona in [&alice, &bob] {
        let page = sync_page(&pool, persona, Some(1), Some(2)).await;
        assert_no_store(&page);
        assert_event_types(
            &page.json(),
            &["connection_requests_changed", "connections_changed"],
        );
        assert_eq!(page.json()["has_more"], true);
        assert_eq!(page.json()["next_cursor"], 3);
        let tail = sync_page(&pool, persona, Some(3), None).await;
        assert_event_types(&tail.json(), &["conversation_changed"]);
        assert_eq!(
            tail.json()["events"][0]["conversation_id"],
            conversation_id.to_string()
        );
    }

    let sent = request(
        &pool,
        Method::POST,
        &format!(
            "/v1/personas/{}/conversations/{conversation_id}/messages",
            alice.id
        ),
        &alice.token,
        Some(json!({"body": "secret move"})),
    )
    .await;
    assert_eq!(sent.status, StatusCode::CREATED);
    let message_id = sent.json()["id"]
        .as_str()
        .expect("message should have an ID")
        .to_owned();
    for persona in [&alice, &bob] {
        let page = sync_page(&pool, persona, Some(4), None).await;
        assert_event_types(&page.json(), &["conversation_changed"]);
        assert!(!page.body.contains("secret move"));
        assert!(!page.body.contains("account_id"));
    }

    let read_path = format!(
        "/v1/personas/{}/conversations/{conversation_id}/read/{message_id}",
        bob.id
    );
    for _ in 0..2 {
        assert_eq!(
            request(&pool, Method::PUT, &read_path, &bob.token, None)
                .await
                .status,
            StatusCode::OK
        );
    }
    assert_event_types(
        &sync_page(&pool, &bob, Some(5), None).await.json(),
        &["conversation_changed"],
    );
    assert_eq!(
        sync_page(&pool, &alice, Some(5), None).await.json()["events"],
        json!([])
    );

    let remove_path = format!("/v1/personas/{}/connections/{}", alice.id, bob.id);
    for _ in 0..2 {
        assert_eq!(
            request(&pool, Method::DELETE, &remove_path, &alice.token, None)
                .await
                .status,
            StatusCode::NO_CONTENT
        );
    }
    assert_event_types(
        &sync_page(&pool, &alice, Some(5), None).await.json(),
        &["connection_requests_changed", "connections_changed"],
    );
    assert_event_types(
        &sync_page(&pool, &bob, Some(6), None).await.json(),
        &["connection_requests_changed", "connections_changed"],
    );

    let block_path = format!("/v1/personas/{}/blocks/{}", alice.id, bob.id);
    assert_eq!(
        request(&pool, Method::PUT, &block_path, &alice.token, None)
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        request(&pool, Method::PUT, &block_path, &alice.token, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_event_types(
        &sync_page(&pool, &alice, Some(7), None).await.json(),
        &["blocks_changed"],
    );
    assert_eq!(
        sync_page(&pool, &bob, Some(8), None).await.json()["events"],
        json!([])
    );

    for _ in 0..2 {
        assert_eq!(
            request(&pool, Method::DELETE, &block_path, &alice.token, None)
                .await
                .status,
            StatusCode::NO_CONTENT
        );
    }
    assert_event_types(
        &sync_page(&pool, &alice, Some(8), None).await.json(),
        &["blocks_changed"],
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn retention_prunes_per_persona_and_expired_clients_receive_reset(pool: PgPool) {
    let alice = create_test_persona(&pool, "Retention_Alice", "retention_alice").await;
    let bob = create_test_persona(&pool, "Retention_Bob", "retention_bob").await;
    sqlx::query(
        "INSERT INTO persona_sync_state (persona_id, last_event_sequence) VALUES ($1, 10000)",
    )
    .bind(alice.id)
    .execute(&pool)
    .await
    .expect("sync state should be seedable");
    sqlx::query(
        r#"
        INSERT INTO persona_sync_events (persona_id, event_sequence, event_type)
        SELECT $1, sequence, 'connections_changed'
        FROM generate_series(1, 10000) AS sequence
        "#,
    )
    .bind(alice.id)
    .execute(&pool)
    .await
    .expect("sync events should be seedable");

    let mut transaction = pool.begin().await.expect("transaction should start");
    sync::append_event(&mut transaction, alice.id, SyncEventKind::Blocks)
        .await
        .expect("sync append should work");
    transaction
        .commit()
        .await
        .expect("transaction should commit");

    let (count, minimum, maximum) = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT count(*), min(event_sequence), max(event_sequence) FROM persona_sync_events WHERE persona_id = $1",
    )
    .bind(alice.id)
    .fetch_one(&pool)
    .await
    .expect("retained events should be readable");
    assert_eq!((count, minimum, maximum), (10_000, 2, 10_001));

    let reset = sync_page(&pool, &alice, Some(0), None).await;
    assert_eq!(reset.status, StatusCode::OK);
    assert_eq!(reset.json()["reset_required"], true);
    assert_eq!(reset.json()["events"], json!([]));
    assert_eq!(reset.json()["next_cursor"], 10_001);
    let valid = sync_page(&pool, &alice, Some(1), Some(1)).await;
    assert_eq!(valid.json()["events"][0]["cursor"], 2);
    assert_eq!(valid.json()["has_more"], true);

    let future = sync_page(&pool, &alice, Some(10_002), None).await;
    assert_eq!(future.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(future.json()["error"]["code"], "invalid_sync_cursor");
    assert_no_store(&future);
    assert_eq!(
        sync_page(&pool, &bob, None, None).await.json()["next_cursor"],
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn postgres_hints_and_events_are_visible_only_after_commit(pool: PgPool) {
    let alice = create_test_persona(&pool, "Commit_Alice", "commit_alice").await;
    let hub = SyncHub::new();
    let mut receiver = hub.subscribe();
    let listener = sync::start_postgres_listener(&pool, hub)
        .await
        .expect("listener should start");

    let mut rolled_back = pool.begin().await.expect("transaction should start");
    sync::append_event(&mut rolled_back, alice.id, SyncEventKind::Connections)
        .await
        .expect("event should append inside transaction");
    assert!(
        timeout(Duration::from_millis(100), receiver.recv())
            .await
            .is_err()
    );
    rolled_back.rollback().await.expect("rollback should work");
    assert!(
        timeout(Duration::from_millis(100), receiver.recv())
            .await
            .is_err()
    );

    let mut committed = pool.begin().await.expect("transaction should start");
    sync::append_event(&mut committed, alice.id, SyncEventKind::Connections)
        .await
        .expect("event should append inside transaction");
    committed.commit().await.expect("commit should work");
    assert_eq!(
        timeout(Duration::from_secs(2), receiver.recv())
            .await
            .expect("commit should publish a hint")
            .expect("hint channel should remain open"),
        alice.id
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM persona_sync_events")
            .fetch_one(&pool)
            .await
            .expect("events should be countable"),
        1
    );
    listener.abort();
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn websocket_is_header_authenticated_owner_scoped_and_hint_only(pool: PgPool) {
    let alice = create_test_persona(&pool, "Socket_Alice", "socket_alice").await;
    let bob = create_test_persona(&pool, "Socket_Bob", "socket_bob").await;
    let hub = SyncHub::new();
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener should bind");
    let address = listener.local_addr().expect("listener should have address");
    let server_hub = hub.clone();
    let server_pool = pool.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            router_with_sync_hub(server_pool, MfaCipher::test_cipher(), server_hub),
        )
        .await
        .expect("test server should run");
    });

    let socket_url = format!("ws://{address}/v1/personas/{}/sync/live", alice.id);
    let query_token_error = connect_async(format!("{socket_url}?token={}", alice.token))
        .await
        .expect_err("query tokens must not authenticate");
    assert_http_error(query_token_error, StatusCode::UNAUTHORIZED);
    let mut foreign_request = socket_url
        .clone()
        .into_client_request()
        .expect("WebSocket request should build");
    foreign_request.headers_mut().insert(
        "authorization",
        format!("Bearer {}", bob.token)
            .parse()
            .expect("authorization header should parse"),
    );
    let foreign_error = connect_async(foreign_request)
        .await
        .expect_err("foreign persona socket must not upgrade");
    assert_http_error(foreign_error, StatusCode::NOT_FOUND);

    let mut request = socket_url
        .clone()
        .into_client_request()
        .expect("WebSocket request should build");
    request.headers_mut().insert(
        "authorization",
        format!("Bearer {}", alice.token)
            .parse()
            .expect("authorization header should parse"),
    );
    let (mut socket, response) = connect_async(request)
        .await
        .expect("owned persona socket should upgrade");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let ready = next_socket_json(&mut socket).await;
    assert_eq!(ready, json!({"type": "ready", "cursor": 0}));

    hub.publish(bob.id);
    assert!(
        timeout(Duration::from_millis(100), socket.next())
            .await
            .is_err()
    );
    hub.publish(alice.id);
    assert_eq!(
        next_socket_json(&mut socket).await,
        json!({"type": "changed"})
    );

    socket
        .send(ClientMessage::Binary(vec![0_u8; 1025].into()))
        .await
        .expect("oversized client frame should reach the transport");
    assert_socket_terminated(&mut socket, "oversized client frame").await;

    let mut text_socket = open_owned_socket(&socket_url, &alice.token).await;
    text_socket
        .send(ClientMessage::Text("x".repeat(1025).into()))
        .await
        .expect("oversized client text should reach the transport");
    assert_socket_terminated(&mut text_socket, "oversized client text").await;
    server.abort();
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn websocket_route_enforces_persona_and_account_limits_and_releases_permits(pool: PgPool) {
    let alice = create_test_persona(&pool, "Limit_Alice", "limit_alice").await;
    let hub = SyncHub::new();
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener should bind");
    let address = listener.local_addr().expect("listener should have address");
    let server_pool = pool.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            router_with_sync_hub(server_pool, MfaCipher::test_cipher(), hub),
        )
        .await
        .expect("test server should run");
    });
    let socket_url = format!("ws://{address}/v1/personas/{}/sync/live", alice.id);

    let mut sockets = Vec::new();
    for _ in 0..5 {
        sockets.push(open_owned_socket(&socket_url, &alice.token).await);
    }
    let limit_error = connect_async(owned_socket_request(&socket_url, &alice.token))
        .await
        .expect_err("sixth persona socket must not upgrade");
    assert_http_error(limit_error, StatusCode::TOO_MANY_REQUESTS);

    sockets
        .pop()
        .expect("one socket should be releasable")
        .close(None)
        .await
        .expect("client socket should close");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    let replacement = loop {
        match connect_async(owned_socket_request(&socket_url, &alice.token)).await {
            Ok((mut replacement, response)) => {
                assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
                assert_eq!(
                    next_socket_json(&mut replacement).await,
                    json!({"type": "ready", "cursor": 0})
                );
                break replacement;
            }
            Err(WebSocketError::Http(response))
                if response.status() == StatusCode::TOO_MANY_REQUESTS
                    && tokio::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            other => panic!("released permit should allow another upgrade: {other:?}"),
        }
    };
    sockets.push(replacement);

    for index in 2..=4 {
        let persona_id =
            create_owned_test_persona(&pool, &alice.token, &format!("limit_alice_{index}")).await;
        let persona_url = format!("ws://{address}/v1/personas/{persona_id}/sync/live");
        for _ in 0..5 {
            sockets.push(open_owned_socket(&persona_url, &alice.token).await);
        }
    }
    let overflow_persona_id =
        create_owned_test_persona(&pool, &alice.token, "limit_alice_overflow").await;
    let overflow_url = format!("ws://{address}/v1/personas/{overflow_persona_id}/sync/live");
    let account_limit_error = connect_async(owned_socket_request(&overflow_url, &alice.token))
        .await
        .expect_err("twenty-first account socket must not upgrade");
    assert_http_error(account_limit_error, StatusCode::TOO_MANY_REQUESTS);

    let bob = create_test_persona(&pool, "Limit_Bob", "limit_bob").await;
    let bob_url = format!("ws://{address}/v1/personas/{}/sync/live", bob.id);
    let bob_socket = open_owned_socket(&bob_url, &bob.token).await;

    sockets
        .pop()
        .expect("one account socket should be releasable")
        .close(None)
        .await
        .expect("client socket should close");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    let replacement = loop {
        match connect_async(owned_socket_request(&overflow_url, &alice.token)).await {
            Ok((mut replacement, response)) => {
                assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
                assert_eq!(
                    next_socket_json(&mut replacement).await,
                    json!({"type": "ready", "cursor": 0})
                );
                break replacement;
            }
            Err(WebSocketError::Http(response))
                if response.status() == StatusCode::TOO_MANY_REQUESTS
                    && tokio::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            other => panic!("released account permit should allow another upgrade: {other:?}"),
        }
    };
    sockets.push(replacement);
    drop(bob_socket);
    drop(sockets);
    server.abort();
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn websocket_revalidates_sessions_without_extending_idle_lifetime(pool: PgPool) {
    let alice = create_test_persona(&pool, "Lifecycle_Alice", "lifecycle_alice").await;
    let bob = create_test_persona(&pool, "Lifecycle_Bob", "lifecycle_bob").await;
    let carol = create_test_persona(&pool, "Lifecycle_Carol", "lifecycle_carol").await;
    let hub = SyncHub::new();
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener should bind");
    let address = listener.local_addr().expect("listener should have address");
    let server_hub = hub.clone();
    let server_pool = pool.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            router_with_sync_hub(server_pool, MfaCipher::test_cipher(), server_hub),
        )
        .await
        .expect("test server should run");
    });

    let alice_url = format!("ws://{address}/v1/personas/{}/sync/live", alice.id);
    let mut alice_socket = open_owned_socket(&alice_url, &alice.token).await;
    let (alice_account_id, alice_session_id) = session_identity(&pool, alice.id).await;
    sqlx::query(
        "UPDATE account_sessions SET last_used_at = now() - interval '1 hour' WHERE id = $1",
    )
    .bind(alice_session_id)
    .execute(&pool)
    .await
    .expect("session idle time should be adjustable");
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT last_used_at < now() - interval '30 minutes' FROM account_sessions WHERE id = $1",
        )
        .bind(alice_session_id)
        .fetch_one(&pool)
        .await
        .expect("session idle time should be readable"),
        "periodic socket checks must not advance last_used_at"
    );
    assert!(
        timeout(Duration::from_millis(20), alice_socket.next())
            .await
            .is_err(),
        "a still-valid idle session should keep its socket open"
    );
    sqlx::query("UPDATE account_sessions SET revoked_at = now() WHERE id = $1")
        .bind(alice_session_id)
        .execute(&pool)
        .await
        .expect("session should be revocable");
    hub.publish(alice.id);
    assert_socket_terminated(&mut alice_socket, "revoked session").await;

    let bob_url = format!("ws://{address}/v1/personas/{}/sync/live", bob.id);
    let mut bob_socket = open_owned_socket(&bob_url, &bob.token).await;
    let (_, bob_session_id) = session_identity(&pool, bob.id).await;
    sqlx::query(
        "UPDATE account_sessions SET expires_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(bob_session_id)
    .execute(&pool)
    .await
    .expect("session expiry should be adjustable");
    assert_socket_terminated(&mut bob_socket, "expired session").await;

    let carol_url = format!("ws://{address}/v1/personas/{}/sync/live", carol.id);
    let mut carol_socket = open_owned_socket(&carol_url, &carol.token).await;
    let (carol_account_id, _) = session_identity(&pool, carol.id).await;
    sqlx::query("UPDATE accounts SET status = 'disabled' WHERE id = $1")
        .bind(carol_account_id)
        .execute(&pool)
        .await
        .expect("account status should be adjustable");
    assert_socket_terminated(&mut carol_socket, "disabled account").await;

    assert_ne!(alice_account_id, carol_account_id);
    server.abort();
}

async fn create_test_persona(pool: &PgPool, username: &str, handle: &str) -> TestPersona {
    let password = "TEST-ONLY-sync-passphrase";
    assert_eq!(
        request(
            pool,
            Method::POST,
            "/v1/accounts",
            "",
            Some(json!({"username": username, "password": password})),
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let session = request(
        pool,
        Method::POST,
        "/v1/sessions",
        "",
        Some(json!({
            "username": username.to_ascii_lowercase(),
            "password": password,
            "device_name": "Sync API test"
        })),
    )
    .await;
    assert_eq!(session.status, StatusCode::CREATED);
    let token = session.json()["token"]
        .as_str()
        .expect("session should expose a token")
        .to_owned();
    let persona = request(
        pool,
        Method::POST,
        "/v1/personas",
        &token,
        Some(json!({"handle": handle, "display_name": handle})),
    )
    .await;
    assert_eq!(persona.status, StatusCode::CREATED);
    TestPersona {
        token,
        id: Uuid::try_parse(persona.json()["id"].as_str().expect("persona ID"))
            .expect("persona ID should be a UUID"),
    }
}

async fn create_owned_test_persona(pool: &PgPool, token: &str, handle: &str) -> Uuid {
    let persona = request(
        pool,
        Method::POST,
        "/v1/personas",
        token,
        Some(json!({"handle": handle, "display_name": handle})),
    )
    .await;
    assert_eq!(persona.status, StatusCode::CREATED);
    Uuid::try_parse(persona.json()["id"].as_str().expect("persona ID"))
        .expect("persona ID should be a UUID")
}

async fn sync_page(
    pool: &PgPool,
    persona: &TestPersona,
    after: Option<i64>,
    limit: Option<u16>,
) -> TestResponse {
    let mut path = format!("/v1/personas/{}/sync", persona.id);
    let mut query = Vec::new();
    if let Some(after) = after {
        query.push(format!("after={after}"));
    }
    if let Some(limit) = limit {
        query.push(format!("limit={limit}"));
    }
    if !query.is_empty() {
        path.push('?');
        path.push_str(&query.join("&"));
    }
    request(pool, Method::GET, &path, &persona.token, None).await
}

async fn request(
    pool: &PgPool,
    method: Method,
    uri: &str,
    token: &str,
    payload: Option<Value>,
) -> TestResponse {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = if let Some(payload) = payload {
        builder = builder.header(CONTENT_TYPE, "application/json");
        Body::from(payload.to_string())
    } else {
        Body::empty()
    };
    if !token.is_empty() {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router(pool.clone(), MfaCipher::test_cipher())
        .oneshot(builder.body(body).expect("request should be valid"))
        .await
        .expect("router should return a response");
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();
    TestResponse {
        status,
        headers,
        body: String::from_utf8(body.to_vec()).expect("response should be UTF-8"),
    }
}

fn assert_event_types(document: &Value, expected: &[&str]) {
    let actual = document["events"]
        .as_array()
        .expect("events should be an array")
        .iter()
        .map(|event| event["type"].as_str().expect("event should have a type"))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
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

fn assert_http_error(error: WebSocketError, expected: StatusCode) {
    match error {
        WebSocketError::Http(response) => assert_eq!(response.status(), expected),
        other => panic!("expected an HTTP WebSocket error, got {other:?}"),
    }
}

fn owned_socket_request(
    socket_url: &str,
    token: &str,
) -> tokio_tungstenite::tungstenite::http::Request<()> {
    let mut request = socket_url
        .into_client_request()
        .expect("WebSocket request should build");
    request.headers_mut().insert(
        "authorization",
        format!("Bearer {token}")
            .parse()
            .expect("authorization header should parse"),
    );
    request
}

async fn open_owned_socket(
    socket_url: &str,
    token: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let (mut socket, response) = connect_async(owned_socket_request(socket_url, token))
        .await
        .expect("owned persona socket should upgrade");
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert_eq!(
        next_socket_json(&mut socket).await,
        json!({"type": "ready", "cursor": 0})
    );
    socket
}

async fn assert_socket_terminated<S>(
    socket: &mut tokio_tungstenite::WebSocketStream<S>,
    reason: &str,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    match timeout(Duration::from_secs(2), socket.next()).await {
        Ok(Some(Ok(ClientMessage::Close(_)))) | Ok(Some(Err(_))) | Ok(None) => {}
        Ok(Some(Ok(other))) => {
            panic!("{reason} must terminate the socket instead of delivering {other:?}")
        }
        Err(_) => panic!("{reason} should terminate the socket promptly"),
    }
}

async fn session_identity(pool: &PgPool, persona_id: Uuid) -> (Uuid, Uuid) {
    sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT persona.account_id, session.id
        FROM personas AS persona
        JOIN account_sessions AS session ON session.account_id = persona.account_id
        WHERE persona.id = $1
        ORDER BY session.created_at, session.id
        LIMIT 1
        "#,
    )
    .bind(persona_id)
    .fetch_one(pool)
    .await
    .expect("test persona should own one session")
}

async fn next_socket_json<S>(socket: &mut tokio_tungstenite::WebSocketStream<S>) -> Value
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let message = timeout(Duration::from_secs(2), socket.next())
        .await
        .expect("socket message should arrive")
        .expect("socket should stay open")
        .expect("socket message should be valid");
    serde_json::from_str(
        message
            .to_text()
            .expect("socket message should contain text"),
    )
    .expect("socket message should contain JSON")
}
