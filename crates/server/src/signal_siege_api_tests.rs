use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{
        HeaderMap, Method, Request, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
};
use http_body_util::BodyExt;
use omarchy_game_runtime::{
    GameCommandRejection, GameDefinition, GameInitializationError, GameManifest, GameRegistry,
    GameTransition,
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    accounts::{self, RegistrationInput},
    app::{router, router_with_game_registry},
    games::MAX_ACTIVE_SOLO_SESSIONS_PER_PERSONA,
    mfa::MfaCipher,
    personas::{self, CreatePersonaInput},
    production_game_registry,
    sessions::{self, CreateSessionInput, SessionCreation},
};

struct TestPersona {
    token: String,
    id: Uuid,
}

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

struct TwoHumanGame;

impl GameDefinition for TwoHumanGame {
    fn manifest(&self) -> GameManifest {
        GameManifest {
            key: "two_human".to_owned(),
            version: 1,
            display_name: "Two Human Fixture".to_owned(),
            min_human_players: 2,
            max_human_players: 2,
        }
    }

    fn initial_state(&self, _human_players: u8) -> Result<Value, GameInitializationError> {
        Ok(json!({"turn": 0}))
    }

    fn apply_command(
        &self,
        _state: &Value,
        _actor_seat: u8,
        _command: &Value,
    ) -> Result<GameTransition, GameCommandRejection> {
        Err(GameCommandRejection)
    }
}

#[tokio::test]
async fn production_catalog_and_solo_start_body_bound_are_public_and_exact() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:5432/test")
        .expect("test database URL should parse without connecting");
    let app = router_with_game_registry(
        pool,
        MfaCipher::test_cipher(),
        production_game_registry().expect("production registry should build"),
    );
    let catalog = request(app.clone(), Method::GET, "/v1/games", None).await;
    assert_eq!(catalog.status, StatusCode::OK);
    assert_eq!(
        catalog.json(),
        json!({
            "games": [{
                "key": "signal_siege",
                "version": 1,
                "display_name": "Signal Siege",
                "min_human_players": 1,
                "max_human_players": 1
            }]
        })
    );

    let oversized = request_json(
        app,
        &format!("/v1/personas/{}/game-sessions", Uuid::nil()),
        "not-consulted-before-body-limit",
        json!({
            "idempotency_key": Uuid::nil().to_string(),
            "game_key": "signal_siege",
            "game_version": 1,
            "padding": "x".repeat(9 * 1024)
        }),
    )
    .await;
    assert_eq!(oversized.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_no_store(&oversized);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn solo_start_is_owner_scoped_atomic_idempotent_and_registry_independent(pool: PgPool) {
    let alice = create_test_persona(&pool, "Siege_Start_Alice", "siege_start_alice").await;
    let bob = create_test_persona(&pool, "Siege_Start_Bob", "siege_start_bob").await;
    let path = format!("/v1/personas/{}/game-sessions", alice.id);
    let body = start_body(start_key(1));
    let app = production_router(pool.clone());

    let first_request = request_json(app.clone(), &path, &alice.token, body.clone());
    let raced_retry = request_json(app.clone(), &path, &alice.token, body.clone());
    let (first, second) = tokio::join!(first_request, raced_retry);
    let statuses = [first.status, second.status];
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::CREATED)
            .count(),
        1
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::OK)
            .count(),
        1
    );
    assert_eq!(first.body, second.body);
    assert_no_store(&first);
    let document = first.json();
    let session_id = Uuid::try_parse(
        document["id"]
            .as_str()
            .expect("session ID should be a string"),
    )
    .expect("session ID should be a UUID");
    assert_eq!(document["game_key"], "signal_siege");
    assert_eq!(document["game_version"], 1);
    assert_eq!(document["revision"], 0);
    assert_eq!(document["status"], "active");
    assert!(document["completed_at"].is_null());
    assert_eq!(document["participants"].as_array().map(Vec::len), Some(1));
    assert_eq!(document["participants"][0]["seat"], 0);
    assert_eq!(
        document["participants"][0]["persona"]["id"],
        alice.id.to_string()
    );
    assert_eq!(
        sorted_keys(&document),
        vec![
            "completed_at",
            "created_at",
            "game_key",
            "game_version",
            "id",
            "participants",
            "revision",
            "state",
            "status",
            "updated_at"
        ]
    );
    assert!(!first.body.contains("idempotency_key"));
    assert!(!first.body.contains("account_id"));
    assert_eq!(table_count(&pool, "game_sessions").await, 1);
    assert_eq!(table_count(&pool, "game_session_starts").await, 1);
    assert_eq!(game_event_count(&pool, session_id).await, 1);

    let collision = request_json(
        app.clone(),
        &path,
        &alice.token,
        json!({
            "idempotency_key": start_key(1),
            "game_key": "signal_siege",
            "game_version": 2
        }),
    )
    .await;
    assert_eq!(collision.status, StatusCode::CONFLICT);
    assert_eq!(
        collision.json()["error"]["code"],
        "game_idempotency_conflict"
    );

    let foreign_actor = request_json(app, &path, &bob.token, start_body(start_key(2))).await;
    assert_eq!(foreign_actor.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign_actor.json()["error"]["code"], "persona_not_found");

    let replay = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        &path,
        &alice.token,
        body,
    )
    .await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.body, first.body);
    assert_eq!(table_count(&pool, "game_sessions").await, 1);
    assert_eq!(game_event_count(&pool, session_id).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn solo_start_rejects_invalid_multiplayer_and_over_cap_without_partial_state(pool: PgPool) {
    let alice = create_test_persona(&pool, "Siege_Cap_Alice", "siege_cap_alice").await;
    let path = format!("/v1/personas/{}/game-sessions", alice.id);
    let app = production_router(pool.clone());

    let invalid = request_json(
        app.clone(),
        &path,
        &alice.token,
        json!({
            "idempotency_key": "not-a-uuid",
            "game_key": "signal_siege",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid.json()["error"]["code"], "invalid_game_start");

    let unavailable = request_json(
        app.clone(),
        &path,
        &alice.token,
        json!({
            "idempotency_key": start_key(2),
            "game_key": "missing_game",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(unavailable.status, StatusCode::CONFLICT);
    assert_eq!(unavailable.json()["error"]["code"], "game_unavailable");

    let two_human_registry = GameRegistry::new([Arc::new(TwoHumanGame) as Arc<dyn GameDefinition>])
        .expect("two-human fixture should register");
    let multiplayer = request_json(
        router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), two_human_registry),
        &path,
        &alice.token,
        json!({
            "idempotency_key": start_key(3),
            "game_key": "two_human",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(multiplayer.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        multiplayer.json()["error"]["code"],
        "invalid_game_participants"
    );
    assert_eq!(table_count(&pool, "game_sessions").await, 0);

    for sequence in 0..(MAX_ACTIVE_SOLO_SESSIONS_PER_PERSONA - 1) {
        let created = request_json(
            app.clone(),
            &path,
            &alice.token,
            start_body(start_key(
                100 + u128::try_from(sequence).expect("sequence should fit"),
            )),
        )
        .await;
        assert_eq!(created.status, StatusCode::CREATED);
    }

    let final_slot_a = request_json(app.clone(), &path, &alice.token, start_body(start_key(124)));
    let final_slot_b = request_json(app.clone(), &path, &alice.token, start_body(start_key(125)));
    let (final_a, final_b) = tokio::join!(final_slot_a, final_slot_b);
    let final_statuses = [final_a.status, final_b.status];
    assert_eq!(
        final_statuses
            .iter()
            .filter(|status| **status == StatusCode::CREATED)
            .count(),
        1
    );
    assert_eq!(
        final_statuses
            .iter()
            .filter(|status| **status == StatusCode::TOO_MANY_REQUESTS)
            .count(),
        1
    );
    let rejected_final_slot = if final_a.status == StatusCode::TOO_MANY_REQUESTS {
        &final_a
    } else {
        &final_b
    };
    assert_eq!(
        rejected_final_slot.json()["error"]["code"],
        "too_many_active_game_sessions"
    );

    let capped = request_json(app, &path, &alice.token, start_body(start_key(999))).await;
    assert_eq!(capped.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        capped.json()["error"]["code"],
        "too_many_active_game_sessions"
    );
    assert_eq!(table_count(&pool, "game_sessions").await, 25);
    assert_eq!(table_count(&pool, "game_session_starts").await, 25);

    let replay_at_cap = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        &path,
        &alice.token,
        start_body(start_key(100)),
    )
    .await;
    assert_eq!(replay_at_cap.status, StatusCode::OK);
    assert_eq!(table_count(&pool, "game_sessions").await, 25);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn signal_siege_completes_replays_and_recovers_without_bot_identity(pool: PgPool) {
    let alice = create_test_persona(&pool, "Siege_Play_Alice", "siege_play_alice").await;
    let path = format!("/v1/personas/{}/game-sessions", alice.id);
    let start_request = start_body(start_key(1));
    let app = production_router(pool.clone());
    let started = request_json(app.clone(), &path, &alice.token, start_request.clone()).await;
    assert_eq!(started.status, StatusCode::CREATED);
    let session_id = started.json()["id"]
        .as_str()
        .expect("session ID should be a string")
        .to_owned();
    let command_path = format!("{path}/{session_id}/commands");
    let mut revision = 0_i64;
    let mut state = started.json()["state"].clone();
    let mut status = "active".to_owned();
    let mut final_request = Value::Null;
    let mut final_response = String::new();
    while status == "active" {
        let energy = state["human"]["energy"].as_u64().unwrap_or(0);
        let action = if energy == 0 { "charge" } else { "strike" };
        final_request = json!({
            "idempotency_key": command_key(u128::try_from(revision + 1).expect("revision should fit")),
            "expected_revision": revision,
            "command": {"kind": "play", "action": action}
        });
        let applied = request_json(
            app.clone(),
            &command_path,
            &alice.token,
            final_request.clone(),
        )
        .await;
        assert_eq!(applied.status, StatusCode::OK);
        assert_no_store(&applied);
        revision = applied.json()["revision"]
            .as_i64()
            .expect("revision should be an integer");
        status = applied.json()["status"]
            .as_str()
            .expect("status should be a string")
            .to_owned();
        state = applied.json()["state"].clone();
        final_response = applied.body;
        assert!(
            revision <= 12,
            "fixed round limit should terminate the game"
        );
    }
    assert_eq!(status, "completed");
    assert_eq!(state["phase"], "completed");
    assert!(state["outcome"].is_object());
    assert_eq!(state["outcome"]["rounds_played"], revision);

    let empty_app = router(pool.clone(), MfaCipher::test_cipher());
    let final_replay = request_json(
        empty_app.clone(),
        &command_path,
        &alice.token,
        final_request,
    )
    .await;
    assert_eq!(final_replay.status, StatusCode::OK);
    assert_eq!(final_replay.body, final_response);

    let after_completion = request_json(
        app,
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": command_key(99),
            "expected_revision": revision,
            "command": {"kind": "play", "action": "charge"}
        }),
    )
    .await;
    assert_eq!(after_completion.status, StatusCode::CONFLICT);
    assert_eq!(after_completion.json()["error"]["code"], "game_completed");

    let replayed_start = request_json(empty_app.clone(), &path, &alice.token, start_request).await;
    assert_eq!(replayed_start.status, StatusCode::OK);
    assert_eq!(replayed_start.json()["id"], session_id);
    assert_eq!(replayed_start.json()["status"], "completed");
    assert!(replayed_start.json()["completed_at"].is_string());

    let detail = request(
        empty_app.clone(),
        Method::GET,
        &format!("{path}/{session_id}"),
        Some(&alice.token),
    )
    .await;
    assert_eq!(detail.status, StatusCode::OK);
    assert_eq!(detail.json()["status"], "completed");
    assert_eq!(detail.json()["state"], state);
    assert!(detail.json()["completed_at"].is_string());

    let inventory = request(empty_app.clone(), Method::GET, &path, Some(&alice.token)).await;
    assert_eq!(inventory.status, StatusCode::OK);
    assert_eq!(inventory.json()["sessions"][0]["id"], session_id);
    assert_eq!(inventory.json()["sessions"][0]["status"], "completed");

    let sync = request(
        empty_app,
        Method::GET,
        &format!("/v1/personas/{}/sync?after=0", alice.id),
        Some(&alice.token),
    )
    .await;
    assert_eq!(sync.status, StatusCode::OK);
    assert_eq!(
        sync.json()["events"].as_array().map(Vec::len),
        usize::try_from(revision + 1).ok()
    );
    assert!(sync.json()["events"].as_array().is_some_and(|events| {
        events.iter().all(|event| {
            event["type"] == "game_session_changed"
                && event["game_session_id"] == session_id
                && sorted_keys(event) == vec!["created_at", "cursor", "game_session_id", "type"]
        })
    }));
    assert!(!sync.body.contains("outcome"));
    assert!(!sync.body.contains("human_core"));

    let session_uuid = Uuid::try_parse(&session_id).expect("session ID should parse");
    let stored = sqlx::query_as::<_, (String, bool, i64)>(
        "SELECT status, completed_at IS NOT NULL, revision FROM game_sessions WHERE id = $1",
    )
    .bind(session_uuid)
    .fetch_one(&pool)
    .await
    .expect("completed session should be stored");
    assert_eq!(stored, ("completed".to_owned(), true, revision));
    assert_eq!(command_receipt_count(&pool, session_uuid).await, revision);
    assert_eq!(game_event_count(&pool, session_uuid).await, revision + 1);
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT session_status FROM game_session_commands WHERE game_session_id = $1 ORDER BY applied_revision DESC LIMIT 1",
        )
        .bind(session_uuid)
        .fetch_one(&pool)
        .await
        .expect("final receipt should be readable"),
        "completed"
    );
    assert_eq!(table_count(&pool, "accounts").await, 1);
    assert_eq!(table_count(&pool, "personas").await, 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM game_session_participants WHERE game_session_id = $1",
        )
        .bind(session_uuid)
        .fetch_one(&pool)
        .await
        .expect("participants should be countable"),
        1
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn failed_solo_start_rolls_back_session_receipt_and_sync(pool: PgPool) {
    let alice = create_test_persona(&pool, "Siege_Rollback_Alice", "siege_rollback_alice").await;
    sqlx::query(
        r#"
        CREATE FUNCTION reject_test_solo_start() RETURNS trigger
        LANGUAGE plpgsql AS $$
        BEGIN
            RAISE EXCEPTION 'test-only solo start rejection';
        END;
        $$
        "#,
    )
    .execute(&pool)
    .await
    .expect("test rejection function should install");
    sqlx::query(
        r#"
        CREATE TRIGGER reject_test_solo_start
        BEFORE INSERT ON game_session_starts
        FOR EACH ROW EXECUTE FUNCTION reject_test_solo_start()
        "#,
    )
    .execute(&pool)
    .await
    .expect("test rejection trigger should install");

    let failed = request_json(
        production_router(pool.clone()),
        &format!("/v1/personas/{}/game-sessions", alice.id),
        &alice.token,
        start_body(start_key(1)),
    )
    .await;
    assert_eq!(failed.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(failed.json()["error"]["code"], "internal_error");
    assert_eq!(table_count(&pool, "game_sessions").await, 0);
    assert_eq!(table_count(&pool, "game_session_starts").await, 0);
    assert_eq!(table_count(&pool, "persona_sync_events").await, 0);
}

fn production_router(pool: PgPool) -> Router {
    router_with_game_registry(
        pool,
        MfaCipher::test_cipher(),
        production_game_registry().expect("production registry should build"),
    )
}

fn start_body(idempotency_key: String) -> Value {
    json!({
        "idempotency_key": idempotency_key,
        "game_key": "signal_siege",
        "game_version": 1
    })
}

fn start_key(sequence: u128) -> String {
    Uuid::from_u128(0x4f60_6586_b2b2_43fa_8c48_5c0d_0f20_0000 + sequence).to_string()
}

fn command_key(sequence: u128) -> String {
    Uuid::from_u128(0x237b_d7e3_9475_428e_9ff2_1a8f_27d0_0000 + sequence).to_string()
}

async fn create_test_persona(pool: &PgPool, username: &str, handle: &str) -> TestPersona {
    accounts::register_account(
        pool,
        RegistrationInput {
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
            device_name: "signal siege test".to_owned(),
        },
    )
    .await
    .expect("test session should be created")
    {
        SessionCreation::Created(created) => created.token,
        SessionCreation::MfaRequired(_) => panic!("new test account should not require MFA"),
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
    .expect("test persona should be created");
    TestPersona {
        token,
        id: persona.id,
    }
}

async fn request(app: Router, method: Method, path: &str, token: Option<&str>) -> TestResponse {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    response(
        app,
        builder.body(Body::empty()).expect("request should build"),
    )
    .await
}

async fn request_json(app: Router, path: &str, token: &str, body: Value) -> TestResponse {
    response(
        app,
        Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .expect("JSON request should build"),
    )
    .await
}

async fn response(app: Router, request: Request<Body>) -> TestResponse {
    let response = app.oneshot(request).await.expect("router should respond");
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

async fn table_count(pool: &PgPool, table: &str) -> i64 {
    match table {
        "accounts" => {
            sqlx::query_scalar("SELECT count(*) FROM accounts")
                .fetch_one(pool)
                .await
        }
        "personas" => {
            sqlx::query_scalar("SELECT count(*) FROM personas")
                .fetch_one(pool)
                .await
        }
        "game_sessions" => {
            sqlx::query_scalar("SELECT count(*) FROM game_sessions")
                .fetch_one(pool)
                .await
        }
        "game_session_starts" => {
            sqlx::query_scalar("SELECT count(*) FROM game_session_starts")
                .fetch_one(pool)
                .await
        }
        "persona_sync_events" => {
            sqlx::query_scalar("SELECT count(*) FROM persona_sync_events")
                .fetch_one(pool)
                .await
        }
        _ => panic!("unsupported test table"),
    }
    .expect("table should be countable")
}

async fn command_receipt_count(pool: &PgPool, session_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM game_session_commands WHERE game_session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("command receipts should be countable")
}

async fn game_event_count(pool: &PgPool, session_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM persona_sync_events WHERE game_session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("game events should be countable")
}

fn sorted_keys(value: &Value) -> Vec<&str> {
    let mut keys = value
        .as_object()
        .expect("value should be an object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    keys.sort_unstable();
    keys
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
