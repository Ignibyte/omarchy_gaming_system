use axum::{
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
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

#[tokio::test]
async fn persona_writes_reject_oversized_request_bodies_before_database_work() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:5432/test")
        .expect("test database URL should parse without connecting");
    let response = request(
        &pool,
        Method::POST,
        "/v1/personas",
        Some(json!({
            "handle": "bounded_player",
            "display_name": "Bounded Player",
            "bio": "x".repeat(9 * 1024)
        })),
        Some("ogs1_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
    )
    .await;

    assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn persona_create_and_public_lookup_are_canonical_unique_and_private(pool: PgPool) {
    register_account(&pool, "Persona_Alice", "TEST-ONLY-alice-passphrase").await;
    register_account(&pool, "Persona_Bob", "TEST-ONLY-bob-passphrase").await;
    let alice_token = create_session(
        &pool,
        "persona_alice",
        "TEST-ONLY-alice-passphrase",
        "Alice persona test",
    )
    .await;
    let bob_token = create_session(
        &pool,
        "persona_bob",
        "TEST-ONLY-bob-passphrase",
        "Bob persona test",
    )
    .await;

    let created = authenticated_request(
        &pool,
        Method::POST,
        "/v1/personas",
        &alice_token,
        Some(json!({
            "handle": "  Player_One  ",
            "display_name": "  Player One  ",
            "bio": "first line\n\tsecond line",
            "status_message": "  Ready to play  "
        })),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_no_store(&created);
    assert_public_persona(&created.json());
    assert_eq!(created.json()["handle"], "player_one");
    assert_eq!(created.json()["display_name"], "Player One");
    assert_eq!(created.json()["status_message"], "Ready to play");
    assert_private_fields_absent(&created.body);

    let persona_id = Uuid::try_parse(created.json()["id"].as_str().expect("persona ID"))
        .expect("persona ID should be a UUID");
    let (stored_handle, stored_account_username) = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT persona.handle, account.username
        FROM personas AS persona
        JOIN accounts AS account ON account.id = persona.account_id
        WHERE persona.id = $1
        "#,
    )
    .bind(persona_id)
    .fetch_one(&pool)
    .await
    .expect("created persona should be stored");
    assert_eq!(stored_handle, "player_one");
    assert_eq!(stored_account_username, "persona_alice");

    let duplicate = authenticated_request(
        &pool,
        Method::POST,
        "/v1/personas",
        &bob_token,
        Some(json!({
            "handle": "PLAYER_ONE",
            "display_name": "Other owner"
        })),
    )
    .await;
    assert_eq!(duplicate.status, StatusCode::CONFLICT);
    assert_eq!(duplicate.json()["error"]["code"], "handle_taken");

    let public = request(
        &pool,
        Method::GET,
        "/v1/personas/by-handle/PLAYER_ONE",
        None,
        None,
    )
    .await;
    assert_eq!(public.status, StatusCode::OK);
    assert_eq!(public.json(), created.json());
    assert_public_persona(&public.json());
    assert_private_fields_absent(&public.body);

    let invalid = request(&pool, Method::GET, "/v1/personas/by-handle/xx", None, None).await;
    let absent = request(
        &pool,
        Method::GET,
        "/v1/personas/by-handle/missing_persona",
        None,
        None,
    )
    .await;
    assert_eq!(invalid.status, StatusCode::NOT_FOUND);
    assert_eq!(absent.status, StatusCode::NOT_FOUND);
    assert_eq!(invalid.body, absent.body);
    assert_eq!(invalid.json()["error"]["code"], "persona_not_found");

    let persona_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM personas")
        .fetch_one(&pool)
        .await
        .expect("persona count should be readable");
    assert_eq!(persona_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn persona_inventory_and_updates_are_owner_scoped_allowlisted_and_atomic(pool: PgPool) {
    register_account(&pool, "Owner_Alice", "TEST-ONLY-alice-passphrase").await;
    register_account(&pool, "Owner_Bob", "TEST-ONLY-bob-passphrase").await;
    let alice_token = create_session(
        &pool,
        "owner_alice",
        "TEST-ONLY-alice-passphrase",
        "Alice owner test",
    )
    .await;
    let bob_token = create_session(
        &pool,
        "owner_bob",
        "TEST-ONLY-bob-passphrase",
        "Bob owner test",
    )
    .await;

    let alice_first = create_persona(&pool, &alice_token, "alice_first", "Alice First").await;
    let alice_second = create_persona(&pool, &alice_token, "alice_second", "Alice Second").await;
    let bob_persona = create_persona(&pool, &bob_token, "bob_player", "Bob Player").await;
    let alice_first_id = persona_uuid(&alice_first);
    let alice_second_id = persona_uuid(&alice_second);
    let bob_id = persona_uuid(&bob_persona);

    let inventory =
        authenticated_request(&pool, Method::GET, "/v1/personas", &alice_token, None).await;
    assert_eq!(inventory.status, StatusCode::OK);
    assert_no_store(&inventory);
    let inventory_document = inventory.json();
    let owned = inventory_document["personas"]
        .as_array()
        .expect("inventory should contain personas");
    assert_eq!(owned.len(), 2);
    assert!(owned.iter().all(|persona| {
        persona["handle"] == "alice_first" || persona["handle"] == "alice_second"
    }));
    assert!(
        owned
            .iter()
            .all(|persona| persona["handle"] != "bob_player")
    );
    for persona in owned {
        assert_public_persona(persona);
    }
    assert_private_fields_absent(&inventory.body);

    let unauthenticated = request(&pool, Method::GET, "/v1/personas", None, None).await;
    assert_eq!(unauthenticated.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unauthenticated.json()["error"]["code"], "invalid_session");

    let foreign = authenticated_request(
        &pool,
        Method::PATCH,
        &format!("/v1/personas/{bob_id}"),
        &alice_token,
        Some(json!({"display_name": "Stolen"})),
    )
    .await;
    let absent = authenticated_request(
        &pool,
        Method::PATCH,
        &format!("/v1/personas/{}", Uuid::nil()),
        &alice_token,
        Some(json!({"display_name": "Missing"})),
    )
    .await;
    let malformed = authenticated_request(
        &pool,
        Method::PATCH,
        "/v1/personas/not-a-uuid",
        &alice_token,
        Some(json!({"display_name": "Malformed"})),
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(absent.status, StatusCode::NOT_FOUND);
    assert_eq!(malformed.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign.body, absent.body);
    assert_eq!(absent.body, malformed.body);

    let unauthenticated_malformed = authenticated_request(
        &pool,
        Method::PATCH,
        "/v1/personas/not-a-uuid",
        "not-a-valid-session",
        Some(json!({"display_name": "No bypass"})),
    )
    .await;
    assert_eq!(unauthenticated_malformed.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        unauthenticated_malformed.json()["error"]["code"],
        "invalid_session"
    );

    let unchanged_bob =
        sqlx::query_scalar::<_, String>("SELECT display_name FROM personas WHERE id = $1")
            .bind(bob_id)
            .fetch_one(&pool)
            .await
            .expect("foreign persona should remain readable");
    assert_eq!(unchanged_bob, "Bob Player");

    sqlx::query("SELECT pg_sleep(0.01)")
        .execute(&pool)
        .await
        .expect("test clock should advance before the owner update");

    let updated = authenticated_request(
        &pool,
        Method::PATCH,
        &format!("/v1/personas/{alice_first_id}"),
        &alice_token,
        Some(json!({
            "handle": "  Alice_Prime  ",
            "display_name": "  Alice Prime  ",
            "bio": "",
            "status_message": "  In a match  "
        })),
    )
    .await;
    assert_eq!(updated.status, StatusCode::OK);
    assert_no_store(&updated);
    assert_public_persona(&updated.json());
    assert_eq!(updated.json()["handle"], "alice_prime");
    assert_eq!(updated.json()["display_name"], "Alice Prime");
    assert_eq!(updated.json()["bio"], "");
    assert_eq!(updated.json()["status_message"], "In a match");
    let timestamp_advanced =
        sqlx::query_scalar::<_, bool>("SELECT updated_at > created_at FROM personas WHERE id = $1")
            .bind(alice_first_id)
            .fetch_one(&pool)
            .await
            .expect("persona timestamps should be readable");
    assert!(timestamp_advanced);

    let old_lookup = request(
        &pool,
        Method::GET,
        "/v1/personas/by-handle/alice_first",
        None,
        None,
    )
    .await;
    let new_lookup = request(
        &pool,
        Method::GET,
        "/v1/personas/by-handle/ALICE_PRIME",
        None,
        None,
    )
    .await;
    assert_eq!(old_lookup.status, StatusCode::NOT_FOUND);
    assert_eq!(new_lookup.status, StatusCode::OK);
    assert_eq!(new_lookup.json(), updated.json());

    let conflict = authenticated_request(
        &pool,
        Method::PATCH,
        &format!("/v1/personas/{alice_second_id}"),
        &alice_token,
        Some(json!({"handle": "ALICE_PRIME"})),
    )
    .await;
    assert_eq!(conflict.status, StatusCode::CONFLICT);
    assert_eq!(conflict.json()["error"]["code"], "handle_taken");

    let empty = authenticated_request(
        &pool,
        Method::PATCH,
        &format!("/v1/personas/{alice_second_id}"),
        &alice_token,
        Some(json!({})),
    )
    .await;
    assert_eq!(empty.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(empty.json()["error"]["code"], "empty_persona_patch");

    let preserved_second =
        sqlx::query_scalar::<_, String>("SELECT handle FROM personas WHERE id = $1")
            .bind(alice_second_id)
            .fetch_one(&pool)
            .await
            .expect("conflicting persona should remain readable");
    assert_eq!(preserved_second, "alice_second");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn persona_validation_and_input_allowlists_preserve_storage(pool: PgPool) {
    register_account(&pool, "Valid_Alice", "TEST-ONLY-alice-passphrase").await;
    let token = create_session(
        &pool,
        "valid_alice",
        "TEST-ONLY-alice-passphrase",
        "Validation test",
    )
    .await;

    for (payload, code) in [
        (
            json!({"handle": "-bad", "display_name": "Valid"}),
            "invalid_handle",
        ),
        (
            json!({"handle": "valid_handle", "display_name": "\n"}),
            "invalid_display_name",
        ),
        (
            json!({"handle": "valid_handle", "display_name": "Valid", "bio": "bad\rline"}),
            "invalid_bio",
        ),
        (
            json!({"handle": "valid_handle", "display_name": "Valid", "status_message": "bad\tstatus"}),
            "invalid_status_message",
        ),
    ] {
        let rejected =
            authenticated_request(&pool, Method::POST, "/v1/personas", &token, Some(payload)).await;
        assert_eq!(rejected.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(rejected.json()["error"]["code"], code);
    }

    let owner_injection = authenticated_request(
        &pool,
        Method::POST,
        "/v1/personas",
        &token,
        Some(json!({
            "handle": "injected_owner",
            "display_name": "Injection attempt",
            "account_id": Uuid::nil().to_string()
        })),
    )
    .await;
    assert_eq!(owner_injection.status, StatusCode::UNPROCESSABLE_ENTITY);

    let persona_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM personas")
        .fetch_one(&pool)
        .await
        .expect("persona count should be readable");
    assert_eq!(persona_count, 0);
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
) -> String {
    let response = request(
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
    .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.json()["token"]
        .as_str()
        .expect("session creation should return a token")
        .to_owned()
}

async fn create_persona(pool: &PgPool, token: &str, handle: &str, display_name: &str) -> Value {
    let response = authenticated_request(
        pool,
        Method::POST,
        "/v1/personas",
        token,
        Some(json!({"handle": handle, "display_name": display_name})),
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED);
    response.json()
}

async fn authenticated_request(
    pool: &PgPool,
    method: Method,
    uri: &str,
    token: &str,
    payload: Option<Value>,
) -> TestResponse {
    request(pool, method, uri, payload, Some(token)).await
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

fn assert_no_store(response: &TestResponse) {
    assert_eq!(
        response
            .headers
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
}

fn assert_public_persona(document: &Value) {
    let mut keys = document
        .as_object()
        .expect("persona should be a JSON object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "bio",
            "created_at",
            "display_name",
            "handle",
            "id",
            "status_message",
            "updated_at",
        ]
    );
}

fn assert_private_fields_absent(body: &str) {
    for private_field in [
        "account_id",
        "token",
        "token_hash",
        "password",
        "session_id",
    ] {
        assert!(
            !body.contains(private_field),
            "persona response exposed private field {private_field}"
        );
    }
}

fn persona_uuid(document: &Value) -> Uuid {
    Uuid::try_parse(document["id"].as_str().expect("persona response ID"))
        .expect("persona ID should be a UUID")
}
