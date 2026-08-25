use axum::{
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{app::router, mfa::MfaCipher};

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

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn session_creation_stores_only_a_digest_and_hides_login_identity(pool: PgPool) {
    register_account(&pool, "Session_Alice", "TEST-ONLY-alice-passphrase").await;

    let created = create_session(
        &pool,
        "  SESSION_ALICE ",
        "TEST-ONLY-alice-passphrase",
        "  Omarchy laptop  ",
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_eq!(
        created
            .headers
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let created_document = created.json();
    let token = created_document["token"]
        .as_str()
        .expect("creation response should contain a token");
    let session_id = Uuid::try_parse(
        created_document["session"]["id"]
            .as_str()
            .expect("creation response should contain a session ID"),
    )
    .expect("session ID should be a UUID");
    assert!(token.starts_with("ogs1_"));
    assert_eq!(created_document["session"]["device_name"], "Omarchy laptop");
    assert_eq!(created_document["session"]["current"], true);
    assert!(!created.body.contains("token_hash"));
    assert!(!created.body.contains("account_id"));

    let (token_hash, device_name, digest_length, long_lived) =
        sqlx::query_as::<_, (Vec<u8>, String, i32, bool)>(
            r#"
            SELECT
                token_hash,
                device_name,
                octet_length(token_hash),
                expires_at > now() + interval '29 days'
            FROM account_sessions
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("created session should be stored");
    assert_eq!(device_name, "Omarchy laptop");
    assert_eq!(digest_length, 32);
    assert!(long_lived);
    assert_ne!(token_hash, token.as_bytes());

    let unknown = create_session(
        &pool,
        "missing_account",
        "TEST-ONLY-alice-passphrase",
        "Unknown account device",
    )
    .await;
    let wrong_password = create_session(
        &pool,
        "session_alice",
        "TEST-ONLY-wrong-passphrase",
        "Wrong password device",
    )
    .await;
    sqlx::query("UPDATE accounts SET status = 'suspended' WHERE username = 'session_alice'")
        .execute(&pool)
        .await
        .expect("test account should be suspendable");
    let suspended = create_session(
        &pool,
        "session_alice",
        "TEST-ONLY-alice-passphrase",
        "Suspended account device",
    )
    .await;
    sqlx::query("UPDATE accounts SET status = 'disabled' WHERE username = 'session_alice'")
        .execute(&pool)
        .await
        .expect("test account should be disableable");
    let disabled = create_session(
        &pool,
        "session_alice",
        "TEST-ONLY-alice-passphrase",
        "Disabled account device",
    )
    .await;

    for failed_login in [&unknown, &wrong_password, &suspended, &disabled] {
        assert_eq!(failed_login.status, StatusCode::UNAUTHORIZED);
        assert_eq!(failed_login.json()["error"]["code"], "invalid_credentials");
        assert_eq!(
            failed_login
                .headers
                .get("www-authenticate")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer")
        );
    }
    assert_eq!(unknown.body, wrong_password.body);
    assert_eq!(wrong_password.body, suspended.body);
    assert_eq!(suspended.body, disabled.body);

    let session_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM account_sessions")
        .fetch_one(&pool)
        .await
        .expect("session count should be readable");
    assert_eq!(session_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn session_authentication_enforces_scope_idle_expiry_and_account_status(pool: PgPool) {
    register_account(&pool, "List_Alice", "TEST-ONLY-alice-passphrase").await;
    register_account(&pool, "List_Bob", "TEST-ONLY-bob-passphrase").await;

    let alice_first = create_session(
        &pool,
        "list_alice",
        "TEST-ONLY-alice-passphrase",
        "Alice first",
    )
    .await
    .json();
    let alice_second = create_session(
        &pool,
        "list_alice",
        "TEST-ONLY-alice-passphrase",
        "Alice idle",
    )
    .await
    .json();
    let bob = create_session(&pool, "list_bob", "TEST-ONLY-bob-passphrase", "Bob expired")
        .await
        .json();

    let alice_first_token = alice_first["token"].as_str().expect("token");
    let alice_first_id = session_uuid(&alice_first);
    let alice_second_token = alice_second["token"].as_str().expect("token");
    let alice_second_id = session_uuid(&alice_second);
    let bob_token = bob["token"].as_str().expect("token");
    let bob_id = session_uuid(&bob);

    sqlx::query(
        "UPDATE account_sessions SET last_used_at = now() - interval '1 hour' WHERE id = $1",
    )
    .bind(alice_first_id)
    .execute(&pool)
    .await
    .expect("last use should be adjustable");
    sqlx::query(
        "UPDATE account_sessions SET last_used_at = now() - interval '8 days' WHERE id = $1",
    )
    .bind(alice_second_id)
    .execute(&pool)
    .await
    .expect("idle timeout should be adjustable");
    sqlx::query(
        "UPDATE account_sessions SET expires_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(bob_id)
    .execute(&pool)
    .await
    .expect("absolute expiry should be adjustable");

    let alice_list =
        authenticated_request(&pool, Method::GET, "/v1/sessions", alice_first_token).await;
    assert_eq!(alice_list.status, StatusCode::OK);
    assert_eq!(
        alice_list
            .headers
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let alice_sessions = alice_list.json()["sessions"]
        .as_array()
        .expect("list response should contain sessions")
        .clone();
    assert_eq!(alice_sessions.len(), 2);
    assert!(
        alice_sessions
            .iter()
            .all(|session| session["device_name"] != "Bob expired")
    );
    assert_eq!(
        alice_sessions
            .iter()
            .find(|session| session["id"] == alice_first_id.to_string())
            .expect("current session should be listed")["current"],
        true
    );

    let last_use_advanced = sqlx::query_scalar::<_, bool>(
        "SELECT last_used_at > now() - interval '1 minute' FROM account_sessions WHERE id = $1",
    )
    .bind(alice_first_id)
    .fetch_one(&pool)
    .await
    .expect("last use should be readable");
    assert!(last_use_advanced);

    for rejected in [
        authenticated_request(&pool, Method::GET, "/v1/sessions", alice_second_token).await,
        authenticated_request(&pool, Method::GET, "/v1/sessions", bob_token).await,
    ] {
        assert_eq!(rejected.status, StatusCode::UNAUTHORIZED);
        assert_eq!(rejected.json()["error"]["code"], "invalid_session");
    }

    sqlx::query("UPDATE accounts SET status = 'disabled' WHERE username = 'list_alice'")
        .execute(&pool)
        .await
        .expect("test account should be disableable");
    let disabled =
        authenticated_request(&pool, Method::GET, "/v1/sessions", alice_first_token).await;
    assert_eq!(disabled.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn session_revocation_is_owner_scoped_idempotent_and_immediate(pool: PgPool) {
    register_account(&pool, "Revoke_Alice", "TEST-ONLY-alice-passphrase").await;
    register_account(&pool, "Revoke_Bob", "TEST-ONLY-bob-passphrase").await;

    let alice_first = create_session(
        &pool,
        "revoke_alice",
        "TEST-ONLY-alice-passphrase",
        "Alice first",
    )
    .await
    .json();
    let alice_second = create_session(
        &pool,
        "revoke_alice",
        "TEST-ONLY-alice-passphrase",
        "Alice second",
    )
    .await
    .json();
    let bob = create_session(
        &pool,
        "revoke_bob",
        "TEST-ONLY-bob-passphrase",
        "Bob device",
    )
    .await
    .json();

    let alice_first_token = alice_first["token"].as_str().expect("token");
    let alice_second_token = alice_second["token"].as_str().expect("token");
    let alice_first_id = session_uuid(&alice_first);
    let alice_second_id = session_uuid(&alice_second);
    let bob_id = session_uuid(&bob);

    let foreign = authenticated_request(
        &pool,
        Method::DELETE,
        &format!("/v1/sessions/{bob_id}"),
        alice_first_token,
    )
    .await;
    let absent = authenticated_request(
        &pool,
        Method::DELETE,
        &format!("/v1/sessions/{}", Uuid::nil()),
        alice_first_token,
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(absent.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign.body, absent.body);

    for _ in 0..2 {
        let revoked = authenticated_request(
            &pool,
            Method::DELETE,
            &format!("/v1/sessions/{alice_second_id}"),
            alice_first_token,
        )
        .await;
        assert_eq!(revoked.status, StatusCode::NO_CONTENT);
    }
    let revoked_reuse =
        authenticated_request(&pool, Method::GET, "/v1/sessions", alice_second_token).await;
    assert_eq!(revoked_reuse.status, StatusCode::UNAUTHORIZED);

    let self_revoked = authenticated_request(
        &pool,
        Method::DELETE,
        &format!("/v1/sessions/{alice_first_id}"),
        alice_first_token,
    )
    .await;
    assert_eq!(self_revoked.status, StatusCode::NO_CONTENT);
    let self_reuse =
        authenticated_request(&pool, Method::GET, "/v1/sessions", alice_first_token).await;
    assert_eq!(self_reuse.status, StatusCode::UNAUTHORIZED);
}

async fn register_account(pool: &PgPool, username: &str, password: &str) {
    let response = request(
        pool,
        Method::POST,
        "/v1/accounts",
        Some(json!({"username": username, "password": password})),
        None,
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED);
}

async fn create_session(
    pool: &PgPool,
    username: &str,
    password: &str,
    device_name: &str,
) -> TestResponse {
    request(
        pool,
        Method::POST,
        "/v1/sessions",
        Some(json!({
            "username": username,
            "password": password,
            "device_name": device_name
        })),
        None,
    )
    .await
}

async fn authenticated_request(
    pool: &PgPool,
    method: Method,
    uri: &str,
    token: &str,
) -> TestResponse {
    request(pool, method, uri, None, Some(token)).await
}

async fn request(
    pool: &PgPool,
    method: Method,
    uri: &str,
    payload: Option<Value>,
    token: Option<&str>,
) -> TestResponse {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = if let Some(payload) = payload {
        builder = builder.header(CONTENT_TYPE, "application/json");
        Body::from(payload.to_string())
    } else {
        Body::empty()
    };
    if let Some(token) = token {
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
        body: String::from_utf8(body.to_vec()).expect("response body should be UTF-8"),
    }
}

fn session_uuid(document: &Value) -> Uuid {
    Uuid::try_parse(
        document["session"]["id"]
            .as_str()
            .expect("session response should contain an ID"),
    )
    .expect("session ID should be a UUID")
}
