//! PostgreSQL-backed challenge API integration tests.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

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
    app::router_with_game_registry,
    connections,
    mfa::MfaCipher,
    personas::{self, CreatePersonaInput},
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

struct FixtureGame {
    key: &'static str,
    version: u32,
    fail: bool,
}

impl GameDefinition for FixtureGame {
    fn manifest(&self) -> GameManifest {
        GameManifest {
            key: self.key.to_owned(),
            version: self.version,
            display_name: format!("Fixture {}", self.version),
            min_human_players: 2,
            max_human_players: 2,
        }
    }

    fn initial_state(&self, human_players: u8) -> Result<Value, GameInitializationError> {
        if self.fail {
            Err(GameInitializationError)
        } else {
            Ok(json!({
                "rules_version": self.version,
                "human_players": human_players,
                "turn": 0
            }))
        }
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

fn registry(definitions: &[(&'static str, u32, bool)]) -> GameRegistry {
    GameRegistry::new(definitions.iter().map(|(key, version, fail)| {
        Arc::new(FixtureGame {
            key,
            version: *version,
            fail: *fail,
        }) as Arc<dyn GameDefinition>
    }))
    .expect("fixture registry should be valid")
}

#[tokio::test]
async fn challenge_route_rejects_oversized_bodies_before_database_work() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:5432/test")
        .expect("test database URL should parse without connecting");
    let response = request_json(
        router_with_game_registry(
            pool,
            MfaCipher::test_cipher(),
            registry(&[("fixture", 1, false)]),
        ),
        Method::POST,
        &format!("/v1/personas/{}/game-challenges", Uuid::nil()),
        "not-consulted-before-the-body-limit",
        json!({
            "idempotency_key": test_uuid(1).to_string(),
            "challenged_persona_id": test_uuid(2).to_string(),
            "game_key": "fixture",
            "game_version": 1,
            "padding": "x".repeat(9 * 1024)
        }),
    )
    .await;
    assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_no_store(&response);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn creation_is_private_idempotent_and_atomically_notified(pool: PgPool) {
    let alice = create_test_persona(&pool, "Challenge_Alice", "challenge_alice").await;
    let bob = create_test_persona(&pool, "Challenge_Bob", "challenge_bob").await;
    let carol = create_test_persona(&pool, "Challenge_Carol", "challenge_carol").await;
    connect(&pool, &alice, &bob).await;
    let games = registry(&[("fixture", 1, false), ("fixture", 2, false)]);
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), games);
    let alice_cursor = current_cursor(&pool, alice.id).await;
    let bob_cursor = current_cursor(&pool, bob.id).await;
    let idempotency_key = test_uuid(10);
    let path = format!("/v1/personas/{}/game-challenges", alice.id);
    let body = json!({
        "idempotency_key": idempotency_key.to_string(),
        "challenged_persona_id": bob.id.to_string(),
        "game_key": "fixture",
        "game_version": 1
    });

    let created = request_json(app.clone(), Method::POST, &path, &alice.token, body.clone()).await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_no_store(&created);
    let document = created.json();
    assert_eq!(document["direction"], "outgoing");
    assert_eq!(document["status"], "pending");
    assert_eq!(document["game_key"], "fixture");
    assert_eq!(document["game_version"], 1);
    assert_eq!(document["challenger"]["id"], alice.id.to_string());
    assert_eq!(document["challenged"]["id"], bob.id.to_string());
    assert_private_fields_absent(&created.body);
    let challenge_id = Uuid::parse_str(document["id"].as_str().unwrap()).unwrap();

    let replay = request_json(app.clone(), Method::POST, &path, &alice.token, body.clone()).await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.json(), document);

    let collision = request_json(
        app.clone(),
        Method::POST,
        &path,
        &alice.token,
        json!({
            "idempotency_key": idempotency_key.to_string(),
            "challenged_persona_id": bob.id.to_string(),
            "game_key": "fixture",
            "game_version": 2
        }),
    )
    .await;
    assert_eq!(collision.status, StatusCode::CONFLICT);
    assert_eq!(
        collision.json()["error"]["code"],
        "game_challenge_idempotency_conflict"
    );

    let unavailable_target = request_json(
        app.clone(),
        Method::POST,
        &path,
        &alice.token,
        json!({
            "idempotency_key": test_uuid(11).to_string(),
            "challenged_persona_id": carol.id.to_string(),
            "game_key": "fixture",
            "game_version": 1
        }),
    )
    .await;
    assert_eq!(unavailable_target.status, StatusCode::CONFLICT);
    assert_eq!(
        unavailable_target.json()["error"]["code"],
        "challenge_target_unavailable"
    );

    assert_eq!(game_challenge_count(&pool).await, 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM inbox_messages WHERE system_game_challenge_id = $1",
        )
        .bind(challenge_id)
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    assert_eq!(challenge_event_count(&pool, challenge_id).await, 2);

    for (persona, cursor) in [(&alice, alice_cursor), (&bob, bob_cursor)] {
        let sync = request(
            app.clone(),
            Method::GET,
            &format!("/v1/personas/{}/sync?after={cursor}", persona.id),
            Some(&persona.token),
        )
        .await;
        assert_eq!(sync.status, StatusCode::OK);
        let events = sync.json()["events"].as_array().cloned().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["type"], "game_challenge_changed");
        assert_eq!(events[0]["game_challenge_id"], challenge_id.to_string());
        assert_eq!(events[1]["type"], "conversation_changed");
        assert!(events[0].get("game_session_id").is_none());
        assert!(events[0].get("conversation_id").is_none());
    }

    let bob_detail = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-challenges/{challenge_id}", bob.id),
        Some(&bob.token),
    )
    .await;
    assert_eq!(bob_detail.status, StatusCode::OK);
    assert_eq!(bob_detail.json()["direction"], "incoming");
    assert_private_fields_absent(&bob_detail.body);

    let foreign = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-challenges/{challenge_id}", carol.id),
        Some(&carol.token),
    )
    .await;
    let absent = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-challenges/{}", carol.id, Uuid::nil()),
        Some(&carol.token),
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign.body, absent.body);

    connections::block_persona(
        &pool,
        &bob.token,
        &bob.id.to_string(),
        &alice.id.to_string(),
    )
    .await
    .expect("block should commit");
    let replay_after_policy_change = request_json(
        router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), registry(&[])),
        Method::POST,
        &path,
        &alice.token,
        body,
    )
    .await;
    assert_eq!(replay_after_policy_change.status, StatusCode::OK);
    assert_eq!(replay_after_policy_change.json(), document);
    assert_eq!(game_challenge_count(&pool).await, 1);
    assert_eq!(challenge_message_count(&pool, challenge_id).await, 1);
    assert_eq!(challenge_event_count(&pool, challenge_id).await, 2);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn acceptance_creates_one_exact_session_and_retry_has_no_effects(pool: PgPool) {
    let alice = create_test_persona(&pool, "Accept_Alice", "accept_alice").await;
    let bob = create_test_persona(&pool, "Accept_Bob", "accept_bob").await;
    connect(&pool, &alice, &bob).await;
    let games = registry(&[("fixture", 1, false), ("fixture", 2, false)]);
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), games);
    let challenge_id = create_challenge(&app, &alice, bob.id, "fixture", 2).await;
    let path = format!(
        "/v1/personas/{}/game-challenges/{challenge_id}/accept",
        bob.id
    );

    let accepted = request(app.clone(), Method::PUT, &path, Some(&bob.token)).await;
    assert_eq!(accepted.status, StatusCode::OK);
    assert_no_store(&accepted);
    assert_eq!(accepted.json()["status"], "accepted");
    assert_eq!(accepted.json()["game_version"], 2);
    let session_id = Uuid::parse_str(
        accepted.json()["game_session_id"]
            .as_str()
            .expect("accepted challenge should link a session"),
    )
    .unwrap();
    let stored = sqlx::query_as::<_, (String, i64, Value)>(
        "SELECT game_key, game_version, state FROM game_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored.0, "fixture");
    assert_eq!(stored.1, 2);
    assert_eq!(stored.2["human_players"], 2);
    assert_eq!(
        sqlx::query_as::<_, (Uuid, i16)>(
            "SELECT persona_id, seat FROM game_session_participants WHERE game_session_id = $1 ORDER BY seat",
        )
        .bind(session_id)
        .fetch_all(&pool)
        .await
        .unwrap(),
        vec![(alice.id, 0), (bob.id, 1)]
    );
    assert_eq!(challenge_event_count(&pool, challenge_id).await, 4);
    assert_eq!(game_session_event_count(&pool, session_id).await, 2);
    assert_eq!(challenge_message_count(&pool, challenge_id).await, 2);

    let conversation_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM inbox_conversations WHERE (persona_low_id = $1 AND persona_high_id = $2) OR (persona_low_id = $2 AND persona_high_id = $1)",
    )
    .bind(alice.id)
    .bind(bob.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let messages = request(
        app.clone(),
        Method::GET,
        &format!(
            "/v1/personas/{}/conversations/{conversation_id}/messages",
            bob.id
        ),
        Some(&bob.token),
    )
    .await;
    assert_eq!(messages.status, StatusCode::OK);
    let messages = messages.json()["messages"].as_array().cloned().unwrap();
    assert_eq!(messages[0]["system"]["type"], "connection_accepted");
    assert_eq!(messages[1]["system"]["type"], "game_challenge_created");
    assert_eq!(
        messages[1]["system"]["challenge_id"],
        challenge_id.to_string()
    );
    assert_eq!(messages[2]["system"]["type"], "game_challenge_accepted");
    assert_eq!(
        messages[2]["system"]["challenge_id"],
        challenge_id.to_string()
    );
    assert_eq!(
        messages[2]["system"]["game_session_id"],
        session_id.to_string()
    );

    let retry = request(app.clone(), Method::PUT, &path, Some(&bob.token)).await;
    assert_eq!(retry.status, StatusCode::OK);
    assert_eq!(retry.json(), accepted.json());
    assert_eq!(challenge_event_count(&pool, challenge_id).await, 4);
    assert_eq!(game_session_event_count(&pool, session_id).await, 2);
    assert_eq!(challenge_message_count(&pool, challenge_id).await, 2);

    let wrong_direction = request(
        app,
        Method::PUT,
        &format!(
            "/v1/personas/{}/game-challenges/{challenge_id}/accept",
            alice.id
        ),
        Some(&alice.token),
    )
    .await;
    assert_eq!(wrong_direction.status, StatusCode::CONFLICT);
    assert_eq!(
        wrong_direction.json()["error"]["code"],
        "game_challenge_transition_unavailable"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn terminal_history_paginates_and_expiry_prevents_acceptance(pool: PgPool) {
    let alice = create_test_persona(&pool, "History_Alice", "history_alice").await;
    let bob = create_test_persona(&pool, "History_Bob", "history_bob").await;
    connect(&pool, &alice, &bob).await;
    let games = registry(&[
        ("fixture", 1, false),
        ("fixture", 2, false),
        ("fixture", 3, false),
    ]);
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), games);

    let declined_id = create_challenge(&app, &alice, bob.id, "fixture", 1).await;
    let declined = request(
        app.clone(),
        Method::PUT,
        &format!(
            "/v1/personas/{}/game-challenges/{declined_id}/decline",
            bob.id
        ),
        Some(&bob.token),
    )
    .await;
    assert_eq!(declined.status, StatusCode::OK);
    assert_eq!(declined.json()["status"], "declined");
    assert!(declined.json()["game_session_id"].is_null());

    let cancelled_id = create_challenge(&app, &alice, bob.id, "fixture", 2).await;
    let cancelled = request(
        app.clone(),
        Method::DELETE,
        &format!("/v1/personas/{}/game-challenges/{cancelled_id}", alice.id),
        Some(&alice.token),
    )
    .await;
    assert_eq!(cancelled.status, StatusCode::OK);
    assert_eq!(cancelled.json()["status"], "cancelled");
    assert!(cancelled.json()["game_session_id"].is_null());

    let conversation_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM inbox_conversations WHERE (persona_low_id = $1 AND persona_high_id = $2) OR (persona_low_id = $2 AND persona_high_id = $1)",
    )
    .bind(alice.id)
    .bind(bob.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let messages = request(
        app.clone(),
        Method::GET,
        &format!(
            "/v1/personas/{}/conversations/{conversation_id}/messages",
            alice.id
        ),
        Some(&alice.token),
    )
    .await;
    assert_eq!(messages.status, StatusCode::OK);
    let messages = messages.json()["messages"].as_array().cloned().unwrap();
    assert!(messages.iter().any(|message| {
        message["system"]["type"] == "game_challenge_declined"
            && message["system"]["challenge_id"] == declined_id.to_string()
            && message["system"].get("game_session_id").is_none()
    }));
    assert!(messages.iter().any(|message| {
        message["system"]["type"] == "game_challenge_cancelled"
            && message["system"]["challenge_id"] == cancelled_id.to_string()
            && message["system"].get("game_session_id").is_none()
    }));

    let expired_id = create_challenge(&app, &alice, bob.id, "fixture", 3).await;
    sqlx::query(
        r#"
        UPDATE game_challenges
        SET created_at = now() - interval '2 days',
            expires_at = now() - interval '1 day'
        WHERE id = $1
        "#,
    )
    .bind(expired_id)
    .execute(&pool)
    .await
    .unwrap();
    let expired_accept = request(
        app.clone(),
        Method::PUT,
        &format!(
            "/v1/personas/{}/game-challenges/{expired_id}/accept",
            bob.id
        ),
        Some(&bob.token),
    )
    .await;
    assert_eq!(expired_accept.status, StatusCode::CONFLICT);
    assert_eq!(
        expired_accept.json()["error"]["code"],
        "game_challenge_expired"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT status FROM game_challenges WHERE id = $1")
            .bind(expired_id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        "expired"
    );
    assert_eq!(game_session_count(&pool).await, 0);

    let first_page = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-challenges?limit=2", alice.id),
        Some(&alice.token),
    )
    .await;
    assert_eq!(first_page.status, StatusCode::OK);
    assert_eq!(first_page.json()["challenges"].as_array().unwrap().len(), 2);
    let first_document = first_page.json();
    let before = first_document["next_before"].as_str().unwrap().to_owned();
    let second_page = request(
        app,
        Method::GET,
        &format!(
            "/v1/personas/{}/game-challenges?limit=2&before={before}",
            alice.id
        ),
        Some(&alice.token),
    )
    .await;
    assert_eq!(second_page.status, StatusCode::OK);
    assert_eq!(
        second_page.json()["challenges"].as_array().unwrap().len(),
        1
    );
    let second_document = second_page.json();
    let statuses = first_document["challenges"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second_document["challenges"].as_array().unwrap())
        .map(|challenge| challenge["status"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert!(statuses.contains(&"declined".to_owned()));
    assert!(statuses.contains(&"cancelled".to_owned()));
    assert!(statuses.contains(&"expired".to_owned()));
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn acceptance_failures_roll_back_and_blocked_pairs_cannot_start_sessions(pool: PgPool) {
    let alice = create_test_persona(&pool, "Failure_Alice", "failure_alice").await;
    let bob = create_test_persona(&pool, "Failure_Bob", "failure_bob").await;
    connect(&pool, &alice, &bob).await;
    let games = registry(&[("failure", 1, true), ("fixture", 1, false)]);
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), games);

    let failing_id = create_challenge(&app, &alice, bob.id, "failure", 1).await;
    let failed = request(
        app.clone(),
        Method::PUT,
        &format!(
            "/v1/personas/{}/game-challenges/{failing_id}/accept",
            bob.id
        ),
        Some(&bob.token),
    )
    .await;
    assert_eq!(failed.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(failed.json()["error"]["code"], "internal_error");
    assert_eq!(challenge_status(&pool, failing_id).await, "pending");
    assert_eq!(challenge_message_count(&pool, failing_id).await, 1);
    assert_eq!(challenge_event_count(&pool, failing_id).await, 2);
    assert_eq!(game_session_count(&pool).await, 0);

    let blocked_id = create_challenge(&app, &alice, bob.id, "fixture", 1).await;
    connections::block_persona(
        &pool,
        &bob.token,
        &bob.id.to_string(),
        &alice.id.to_string(),
    )
    .await
    .expect("block should commit");
    let blocked = request(
        app,
        Method::PUT,
        &format!(
            "/v1/personas/{}/game-challenges/{blocked_id}/accept",
            bob.id
        ),
        Some(&bob.token),
    )
    .await;
    assert_eq!(blocked.status, StatusCode::CONFLICT);
    assert_eq!(
        blocked.json()["error"]["code"],
        "challenge_target_unavailable"
    );
    assert_eq!(challenge_status(&pool, blocked_id).await, "pending");
    assert_eq!(challenge_message_count(&pool, blocked_id).await, 1);
    assert_eq!(game_session_count(&pool).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn outgoing_and_incoming_pending_limits_are_server_enforced(pool: PgPool) {
    let alice = create_test_persona(&pool, "Limit_Alice", "limit_alice").await;
    let bob = create_test_persona(&pool, "Limit_Bob", "limit_bob").await;
    let carol = create_test_persona(&pool, "Limit_Carol", "limit_carol").await;
    connect(&pool, &alice, &bob).await;
    connect(&pool, &alice, &carol).await;
    connect(&pool, &carol, &bob).await;
    sqlx::query(
        r#"
        INSERT INTO game_challenges (
            idempotency_key,
            challenger_persona_id,
            challenged_persona_id,
            game_key,
            game_version,
            expires_at
        )
        SELECT
            gen_random_uuid(),
            $1,
            $2,
            'cap' || lpad(sequence::text, 3, '0'),
            1,
            now() + interval '7 days'
        FROM generate_series(1, 100) AS sequence
        "#,
    )
    .bind(alice.id)
    .bind(bob.id)
    .execute(&pool)
    .await
    .unwrap();
    let app = router_with_game_registry(
        pool.clone(),
        MfaCipher::test_cipher(),
        registry(&[("fixture", 1, false)]),
    );

    let outgoing = request_json(
        app.clone(),
        Method::POST,
        &format!("/v1/personas/{}/game-challenges", alice.id),
        &alice.token,
        json!({
            "idempotency_key": test_uuid(800).to_string(),
            "challenged_persona_id": carol.id.to_string(),
            "game_key": "fixture",
            "game_version": 1
        }),
    )
    .await;
    let incoming = request_json(
        app,
        Method::POST,
        &format!("/v1/personas/{}/game-challenges", carol.id),
        &carol.token,
        json!({
            "idempotency_key": test_uuid(801).to_string(),
            "challenged_persona_id": bob.id.to_string(),
            "game_key": "fixture",
            "game_version": 1
        }),
    )
    .await;
    for rejected in [outgoing, incoming] {
        assert_eq!(rejected.status, StatusCode::CONFLICT);
        assert_eq!(
            rejected.json()["error"]["code"],
            "game_challenge_limit_reached"
        );
    }
    assert_eq!(game_challenge_count(&pool).await, 100);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn concurrent_accept_and_decline_have_one_terminal_winner(pool: PgPool) {
    let alice = create_test_persona(&pool, "Race_Challenge_Alice", "race_challenge_alice").await;
    let bob = create_test_persona(&pool, "Race_Challenge_Bob", "race_challenge_bob").await;
    connect(&pool, &alice, &bob).await;
    let games = registry(&[("fixture", 1, false)]);
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), games);
    let challenge_id = create_challenge(&app, &alice, bob.id, "fixture", 1).await;
    let accept_path = format!(
        "/v1/personas/{}/game-challenges/{challenge_id}/accept",
        bob.id
    );
    let decline_path = format!(
        "/v1/personas/{}/game-challenges/{challenge_id}/decline",
        bob.id
    );
    let accept = request(app.clone(), Method::PUT, &accept_path, Some(&bob.token));
    let decline = request(app, Method::PUT, &decline_path, Some(&bob.token));
    let (accept, decline) = tokio::join!(accept, decline);
    assert_eq!(
        [accept.status, decline.status]
            .into_iter()
            .filter(|status| *status == StatusCode::OK)
            .count(),
        1
    );
    assert_eq!(
        [accept.status, decline.status]
            .into_iter()
            .filter(|status| *status == StatusCode::CONFLICT)
            .count(),
        1
    );
    let status =
        sqlx::query_scalar::<_, String>("SELECT status FROM game_challenges WHERE id = $1")
            .bind(challenge_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(matches!(status.as_str(), "accepted" | "declined"));
    assert_eq!(challenge_message_count(&pool, challenge_id).await, 2);
    assert_eq!(challenge_event_count(&pool, challenge_id).await, 4);
    assert_eq!(
        game_session_count(&pool).await,
        i64::from(status == "accepted")
    );
}

async fn create_challenge(
    app: &Router,
    challenger: &TestPersona,
    challenged_id: Uuid,
    game_key: &str,
    game_version: u32,
) -> Uuid {
    let response = request_json(
        app.clone(),
        Method::POST,
        &format!("/v1/personas/{}/game-challenges", challenger.id),
        &challenger.token,
        json!({
            "idempotency_key": next_test_uuid().to_string(),
            "challenged_persona_id": challenged_id.to_string(),
            "game_key": game_key,
            "game_version": game_version
        }),
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
    Uuid::parse_str(response.json()["id"].as_str().unwrap()).unwrap()
}

async fn connect(pool: &PgPool, requester: &TestPersona, addressee: &TestPersona) {
    connections::request_connection(
        pool,
        &requester.token,
        &requester.id.to_string(),
        &addressee.id.to_string(),
    )
    .await
    .expect("connection request should succeed");
    connections::accept_connection(
        pool,
        &addressee.token,
        &addressee.id.to_string(),
        &requester.id.to_string(),
    )
    .await
    .expect("connection acceptance should succeed");
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
            device_name: "challenge test".to_owned(),
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
    response(app, builder.body(Body::empty()).unwrap()).await
}

async fn request_json(
    app: Router,
    method: Method,
    path: &str,
    token: &str,
    body: Value,
) -> TestResponse {
    response(
        app,
        Request::builder()
            .method(method)
            .uri(path)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
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

async fn challenge_event_count(pool: &PgPool, challenge_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM persona_sync_events WHERE game_challenge_id = $1")
        .bind(challenge_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn game_session_event_count(pool: &PgPool, session_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM persona_sync_events WHERE game_session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn challenge_message_count(pool: &PgPool, challenge_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM inbox_messages WHERE system_game_challenge_id = $1")
        .bind(challenge_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn game_challenge_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM game_challenges")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn game_session_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM game_sessions")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn challenge_status(pool: &PgPool, challenge_id: Uuid) -> String {
    sqlx::query_scalar("SELECT status FROM game_challenges WHERE id = $1")
        .bind(challenge_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn current_cursor(pool: &PgPool, persona_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COALESCE((SELECT last_event_sequence FROM persona_sync_state WHERE persona_id = $1), 0)",
    )
    .bind(persona_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn test_uuid(sequence: u128) -> Uuid {
    Uuid::from_u128(0x91cc_0000_0000_4000_8000_0000_0000_0000 + sequence)
}

fn next_test_uuid() -> Uuid {
    static SEQUENCE: AtomicU64 = AtomicU64::new(1_000);
    test_uuid(u128::from(SEQUENCE.fetch_add(1, Ordering::Relaxed)))
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

fn assert_private_fields_absent(body: &str) {
    for private_field in [
        "account_id",
        "idempotency_key",
        "token",
        "token_hash",
        "password",
        "blocker_id",
        "blocked_id",
        "state",
        "snapshot",
    ] {
        assert!(
            !body.contains(private_field),
            "challenge response exposed private field {private_field}"
        );
    }
}
