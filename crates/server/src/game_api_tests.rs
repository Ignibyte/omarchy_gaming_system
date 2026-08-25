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
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    accounts::{self, RegistrationInput},
    app::{router, router_with_game_registry},
    games::{self, GameError},
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
            min_human_players: 1,
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
        state: &Value,
        actor_seat: u8,
        command: &Value,
    ) -> Result<Value, GameCommandRejection> {
        match command.get("kind").and_then(Value::as_str) {
            Some("advance") => {
                let mut next_state = state.clone();
                let object = next_state.as_object_mut().ok_or(GameCommandRejection)?;
                let turn = object
                    .get("turn")
                    .and_then(Value::as_i64)
                    .ok_or(GameCommandRejection)?;
                object.insert("turn".to_owned(), json!(turn + 1));
                object.insert("last_actor_seat".to_owned(), json!(actor_seat));
                Ok(next_state)
            }
            Some("invalid_output") => Ok(json!([])),
            _ => Err(GameCommandRejection),
        }
    }
}

fn registry(definitions: &[(&'static str, u32)]) -> GameRegistry {
    GameRegistry::new(definitions.iter().map(|(key, version)| {
        Arc::new(FixtureGame {
            key,
            version: *version,
            fail: false,
        }) as Arc<dyn GameDefinition>
    }))
    .expect("fixture registry should be valid")
}

fn command_key(sequence: u128) -> String {
    Uuid::from_u128(0x8f5d_8f1d_48df_4f5a_b6e7_ad26_eb30_0000 + sequence).to_string()
}

#[tokio::test]
async fn public_catalog_is_stable_and_production_is_honestly_empty() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:5432/test")
        .expect("test database URL should parse without connecting");
    let empty = request(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::GET,
        "/v1/games",
        None,
    )
    .await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.json(), json!({"games": []}));

    let catalog = request(
        router_with_game_registry(
            pool,
            MfaCipher::test_cipher(),
            registry(&[("zeta", 2), ("alpha", 2), ("alpha", 1)]),
        ),
        Method::GET,
        "/v1/games",
        None,
    )
    .await;
    assert_eq!(catalog.status, StatusCode::OK);
    assert_eq!(
        catalog.json(),
        json!({
            "games": [
                {
                    "key": "alpha",
                    "version": 1,
                    "display_name": "Fixture 1",
                    "min_human_players": 1,
                    "max_human_players": 2
                },
                {
                    "key": "alpha",
                    "version": 2,
                    "display_name": "Fixture 2",
                    "min_human_players": 1,
                    "max_human_players": 2
                },
                {
                    "key": "zeta",
                    "version": 2,
                    "display_name": "Fixture 2",
                    "min_human_players": 1,
                    "max_human_players": 2
                }
            ]
        })
    );
    assert!(!catalog.body.contains("account_id"));
}

#[tokio::test]
async fn command_route_rejects_oversized_bodies_before_database_work() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:5432/test")
        .expect("test database URL should parse without connecting");
    let path = format!(
        "/v1/personas/{}/game-sessions/{}/commands",
        Uuid::nil(),
        Uuid::nil()
    );
    let oversized = request_json(
        router(pool, MfaCipher::test_cipher()),
        &path,
        "not-consulted-before-the-body-limit",
        json!({
            "idempotency_key": command_key(1),
            "expected_revision": 0,
            "command": {
                "kind": "advance",
                "padding": "x".repeat(33 * 1024)
            }
        }),
    )
    .await;

    assert_eq!(oversized.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_no_store(&oversized);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn creation_is_atomic_version_pinned_and_syncs_every_participant(pool: PgPool) {
    let alice = create_test_persona(&pool, "Game_Alice", "game_alice").await;
    let bob = create_test_persona(&pool, "Game_Bob", "game_bob").await;
    let fixtures = registry(&[("fixture", 1), ("fixture", 2)]);

    let mut transaction = pool.begin().await.expect("transaction should start");
    let session_id = games::create_session(
        &mut transaction,
        &fixtures,
        "fixture",
        1,
        &[alice.id, bob.id],
    )
    .await
    .expect("registered version should initialize");
    transaction.commit().await.expect("session should commit");

    let stored = sqlx::query_as::<_, (String, i64, i64, String, Value)>(
        "SELECT game_key, game_version, revision, status, state FROM game_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .expect("session should be readable");
    assert_eq!(stored.0, "fixture");
    assert_eq!(stored.1, 1);
    assert_eq!(stored.2, 0);
    assert_eq!(stored.3, "active");
    assert_eq!(
        stored.4,
        json!({"rules_version": 1, "human_players": 2, "turn": 0})
    );
    let seats = sqlx::query_as::<_, (Uuid, i16)>(
        "SELECT persona_id, seat FROM game_session_participants WHERE game_session_id = $1 ORDER BY seat",
    )
    .bind(session_id)
    .fetch_all(&pool)
    .await
    .expect("participants should be readable");
    assert_eq!(seats, vec![(alice.id, 0), (bob.id, 1)]);

    for persona in [&alice, &bob] {
        let sync = request(
            router(pool.clone(), MfaCipher::test_cipher()),
            Method::GET,
            &format!("/v1/personas/{}/sync?after=0", persona.id),
            Some(&persona.token),
        )
        .await;
        assert_eq!(sync.status, StatusCode::OK);
        assert_eq!(
            sync.json()["events"],
            json!([{
                "cursor": 1,
                "type": "game_session_changed",
                "game_session_id": session_id.to_string(),
                "created_at": sync.json()["events"][0]["created_at"]
            }])
        );
        assert!(!sync.body.contains("rules_version"));
        assert!(!sync.body.contains("account_id"));
    }

    for (game_key, version, participants, expected) in [
        (
            "fixture",
            3,
            vec![alice.id, bob.id],
            GameError::GameUnavailable,
        ),
        (
            "fixture",
            1,
            vec![alice.id, alice.id],
            GameError::InvalidParticipants,
        ),
        (
            "fixture",
            1,
            vec![alice.id, Uuid::nil()],
            GameError::InvalidParticipants,
        ),
    ] {
        let mut rejected = pool.begin().await.expect("transaction should start");
        assert_eq!(
            games::create_session(&mut rejected, &fixtures, game_key, version, &participants).await,
            Err(expected)
        );
        rejected
            .rollback()
            .await
            .expect("rejected transaction should roll back");
    }

    let failing_registry = GameRegistry::new([Arc::new(FixtureGame {
        key: "failure",
        version: 1,
        fail: true,
    }) as Arc<dyn GameDefinition>])
    .expect("failing fixture manifest should register");
    let mut failed_initialization = pool.begin().await.expect("transaction should start");
    assert_eq!(
        games::create_session(
            &mut failed_initialization,
            &failing_registry,
            "failure",
            1,
            &[alice.id]
        )
        .await,
        Err(GameError::InitializationFailed)
    );
    failed_initialization
        .rollback()
        .await
        .expect("failed initialization should roll back");

    let mut explicit_rollback = pool.begin().await.expect("transaction should start");
    games::create_session(
        &mut explicit_rollback,
        &fixtures,
        "fixture",
        2,
        &[alice.id, bob.id],
    )
    .await
    .expect("fixture should initialize before rollback");
    explicit_rollback
        .rollback()
        .await
        .expect("test transaction should roll back");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM game_sessions")
            .fetch_one(&pool)
            .await
            .expect("sessions should be countable"),
        1
    );

    let detail = request(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions/{session_id}", alice.id),
        Some(&alice.token),
    )
    .await;
    assert_eq!(detail.status, StatusCode::OK);
    assert_eq!(detail.json()["game_version"], 1);
    assert_eq!(detail.json()["state"]["rules_version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn session_queries_are_bounded_participant_private_and_registry_independent(pool: PgPool) {
    let alice = create_test_persona(&pool, "Query_Alice", "query_alice").await;
    let bob = create_test_persona(&pool, "Query_Bob", "query_bob").await;
    let carol = create_test_persona(&pool, "Query_Carol", "query_carol").await;
    let fixtures = registry(&[("fixture", 1)]);

    let mut first = pool.begin().await.expect("transaction should start");
    let first_id = games::create_session(&mut first, &fixtures, "fixture", 1, &[alice.id, bob.id])
        .await
        .expect("first session should initialize");
    first.commit().await.expect("first session should commit");
    sqlx::query(
        "UPDATE game_sessions SET created_at = created_at - interval '1 minute' WHERE id = $1",
    )
    .bind(first_id)
    .execute(&pool)
    .await
    .expect("first session should be ageable");

    let mut second = pool.begin().await.expect("transaction should start");
    let second_id =
        games::create_session(&mut second, &fixtures, "fixture", 1, &[bob.id, alice.id])
            .await
            .expect("second session should initialize");
    second.commit().await.expect("second session should commit");

    let app = router_with_game_registry(
        pool.clone(),
        MfaCipher::test_cipher(),
        registry(&[("fixture", 2)]),
    );
    let inventory = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions?limit=1", alice.id),
        Some(&alice.token),
    )
    .await;
    assert_eq!(inventory.status, StatusCode::OK);
    assert_no_store(&inventory);
    assert_eq!(
        inventory.json()["sessions"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(inventory.json()["sessions"][0]["id"], second_id.to_string());
    assert_eq!(inventory.json()["sessions"][0]["game_version"], 1);
    assert_eq!(
        inventory.json()["sessions"][0]["participants"][0]["seat"],
        0
    );
    assert_eq!(
        inventory.json()["sessions"][0]["participants"][0]["persona"]["id"],
        bob.id.to_string()
    );
    assert!(!inventory.body.contains("account_id"));
    assert!(!inventory.body.contains("token"));

    let bob_detail = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions/{first_id}", bob.id),
        Some(&bob.token),
    )
    .await;
    assert_eq!(bob_detail.status, StatusCode::OK);
    assert_no_store(&bob_detail);
    assert_eq!(bob_detail.json()["game_version"], 1);
    assert_eq!(
        bob_detail.json()["participants"].as_array().map(Vec::len),
        Some(2)
    );

    let foreign = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions/{first_id}", carol.id),
        Some(&carol.token),
    )
    .await;
    let absent = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions/{}", carol.id, Uuid::nil()),
        Some(&carol.token),
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign.body, absent.body);
    assert_eq!(foreign.json()["error"]["code"], "game_session_not_found");
    assert_no_store(&foreign);

    let foreign_actor = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions", alice.id),
        Some(&carol.token),
    )
    .await;
    let absent_actor = request(
        app.clone(),
        Method::GET,
        &format!("/v1/personas/{}/game-sessions", Uuid::nil()),
        Some(&carol.token),
    )
    .await;
    assert_eq!(foreign_actor.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign_actor.body, absent_actor.body);
    assert_eq!(foreign_actor.json()["error"]["code"], "persona_not_found");
    assert_no_store(&foreign_actor);

    for query in ["limit=0", "limit=101"] {
        let invalid = request(
            app.clone(),
            Method::GET,
            &format!("/v1/personas/{}/game-sessions?{query}", alice.id),
            Some(&alice.token),
        )
        .await;
        assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(invalid.json()["error"]["code"], "invalid_pagination");
        assert_no_store(&invalid);
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn commands_commit_atomically_replay_semantic_json_and_reject_conflicts(pool: PgPool) {
    let alice = create_test_persona(&pool, "Command_Alice", "command_alice").await;
    let bob = create_test_persona(&pool, "Command_Bob", "command_bob").await;
    let fixtures = registry(&[("fixture", 1), ("fixture", 2)]);
    let session_id = create_game_session(&pool, &fixtures, &[alice.id, bob.id], "fixture", 1).await;
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), fixtures.clone());
    let command_path = format!(
        "/v1/personas/{}/game-sessions/{session_id}/commands",
        alice.id
    );
    let idempotency_key = command_key(1);
    let first_body = format!(
        r#"{{"idempotency_key":"{idempotency_key}","expected_revision":0,"command":{{"kind":"advance","amount":1,"metadata":{{"left":true,"right":false}}}}}}"#
    );

    let first = request_json_text(app.clone(), &command_path, &alice.token, &first_body).await;
    assert_eq!(first.status, StatusCode::OK);
    assert_no_store(&first);
    assert_eq!(
        first.json(),
        json!({
            "game_session_id": session_id.to_string(),
            "revision": 1,
            "state": {
                "rules_version": 1,
                "human_players": 2,
                "turn": 1,
                "last_actor_seat": 0
            }
        })
    );
    assert!(!first.body.contains("account_id"));
    assert!(!first.body.contains("idempotency_key"));
    assert!(!first.body.contains("command"));

    let stored = sqlx::query_as::<_, (i64, Value, bool)>(
        "SELECT revision, state, updated_at > created_at FROM game_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .expect("committed game state should be readable");
    assert_eq!(stored.0, 1);
    assert_eq!(stored.1, first.json()["state"]);
    assert!(stored.2, "the committed command should advance updated_at");
    assert_eq!(command_receipt_count(&pool, session_id).await, 1);
    assert_eq!(game_sync_event_count(&pool, session_id).await, 4);

    let semantic_retry_body = format!(
        r#"{{"command":{{"metadata":{{"right":false,"left":true}},"amount":1.0,"kind":"advance"}},"expected_revision":0,"idempotency_key":"{idempotency_key}"}}"#
    );
    let replay = request_json_text(
        app.clone(),
        &command_path,
        &alice.token,
        &semantic_retry_body,
    )
    .await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.body, first.body);
    assert_eq!(command_receipt_count(&pool, session_id).await, 1);
    assert_eq!(game_sync_event_count(&pool, session_id).await, 4);

    let collision_cases = [
        (
            format!(
                "/v1/personas/{}/game-sessions/{session_id}/commands",
                bob.id
            ),
            &bob.token,
            json!({
                "idempotency_key": idempotency_key,
                "expected_revision": 0,
                "command": {
                    "kind": "advance",
                    "amount": 1,
                    "metadata": {"left": true, "right": false}
                }
            }),
        ),
        (
            command_path.clone(),
            &alice.token,
            json!({
                "idempotency_key": idempotency_key,
                "expected_revision": 1,
                "command": {
                    "kind": "advance",
                    "amount": 1,
                    "metadata": {"left": true, "right": false}
                }
            }),
        ),
        (
            command_path.clone(),
            &alice.token,
            json!({
                "idempotency_key": idempotency_key,
                "expected_revision": 0,
                "command": {
                    "kind": "advance",
                    "amount": 2,
                    "metadata": {"left": true, "right": false}
                }
            }),
        ),
    ];
    for (path, token, body) in collision_cases {
        let collision = request_json(app.clone(), &path, token, body).await;
        assert_eq!(collision.status, StatusCode::CONFLICT);
        assert_eq!(
            collision.json()["error"]["code"],
            "game_idempotency_conflict"
        );
        assert_no_store(&collision);
    }

    for (key, expected_revision) in [(command_key(2), 0), (command_key(3), 2)] {
        let conflict = request_json(
            app.clone(),
            &command_path,
            &alice.token,
            json!({
                "idempotency_key": key,
                "expected_revision": expected_revision,
                "command": {"kind": "advance"}
            }),
        )
        .await;
        assert_eq!(conflict.status, StatusCode::CONFLICT);
        assert_eq!(conflict.json()["error"]["code"], "game_revision_conflict");
        assert!(!conflict.body.contains("\"revision\":"));
    }

    for (key, command, status, code) in [
        (
            command_key(4),
            json!({"kind": "reject"}),
            StatusCode::UNPROCESSABLE_ENTITY,
            "game_command_rejected",
        ),
        (
            command_key(5),
            json!([]),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_game_command",
        ),
        (
            command_key(6),
            json!({"kind": "invalid_output"}),
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        ),
    ] {
        let rejected = request_json(
            app.clone(),
            &command_path,
            &alice.token,
            json!({
                "idempotency_key": key,
                "expected_revision": 1,
                "command": command
            }),
        )
        .await;
        assert_eq!(rejected.status, status);
        assert_eq!(rejected.json()["error"]["code"], code);
        assert_no_store(&rejected);
    }

    let unavailable = request_json(
        router_with_game_registry(
            pool.clone(),
            MfaCipher::test_cipher(),
            registry(&[("fixture", 2)]),
        ),
        &command_path,
        &alice.token,
        json!({
            "idempotency_key": command_key(7),
            "expected_revision": 1,
            "command": {"kind": "advance"}
        }),
    )
    .await;
    assert_eq!(unavailable.status, StatusCode::CONFLICT);
    assert_eq!(unavailable.json()["error"]["code"], "game_unavailable");
    assert_eq!(command_receipt_count(&pool, session_id).await, 1);
    assert_eq!(game_sync_event_count(&pool, session_id).await, 4);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT revision FROM game_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .expect("session revision should be readable"),
        1
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn command_authorization_hides_absent_malformed_and_nonparticipant_sessions(pool: PgPool) {
    let alice = create_test_persona(&pool, "Privacy_Alice", "privacy_alice").await;
    let bob = create_test_persona(&pool, "Privacy_Bob", "privacy_bob").await;
    let carol = create_test_persona(&pool, "Privacy_Carol", "privacy_carol").await;
    let fixtures = registry(&[("fixture", 1)]);
    let session_id = create_game_session(&pool, &fixtures, &[alice.id, bob.id], "fixture", 1).await;
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), fixtures);
    let body = json!({
        "idempotency_key": command_key(1),
        "expected_revision": 0,
        "command": {"kind": "advance"}
    });

    let mut hidden_responses = Vec::new();
    for hidden_session in [
        session_id.to_string(),
        Uuid::nil().to_string(),
        "not-a-uuid".to_owned(),
    ] {
        hidden_responses.push(
            request_json(
                app.clone(),
                &format!(
                    "/v1/personas/{}/game-sessions/{hidden_session}/commands",
                    carol.id
                ),
                &carol.token,
                body.clone(),
            )
            .await,
        );
    }
    for hidden in &hidden_responses {
        assert_eq!(hidden.status, StatusCode::NOT_FOUND);
        assert_eq!(hidden.body, hidden_responses[0].body);
        assert_eq!(hidden.json()["error"]["code"], "game_session_not_found");
        assert_no_store(hidden);
    }

    let foreign_actor = request_json(
        app.clone(),
        &format!(
            "/v1/personas/{}/game-sessions/{session_id}/commands",
            alice.id
        ),
        &carol.token,
        body.clone(),
    )
    .await;
    assert_eq!(foreign_actor.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign_actor.json()["error"]["code"], "persona_not_found");

    let unauthorized = request_json(
        app.clone(),
        &format!(
            "/v1/personas/{}/game-sessions/{session_id}/commands",
            alice.id
        ),
        "invalid-token",
        body.clone(),
    )
    .await;
    assert_eq!(unauthorized.status, StatusCode::UNAUTHORIZED);

    for invalid_body in [
        json!({
            "idempotency_key": "not-a-uuid",
            "expected_revision": 0,
            "command": {"kind": "advance"}
        }),
        json!({
            "idempotency_key": command_key(2),
            "expected_revision": -1,
            "command": {"kind": "advance"}
        }),
    ] {
        let invalid = request_json(
            app.clone(),
            &format!(
                "/v1/personas/{}/game-sessions/{session_id}/commands",
                alice.id
            ),
            &alice.token,
            invalid_body,
        )
        .await;
        assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(invalid.json()["error"]["code"], "invalid_game_command");
        assert_no_store(&invalid);
    }
    assert_eq!(command_receipt_count(&pool, session_id).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn concurrent_distinct_commands_from_one_revision_have_one_winner(pool: PgPool) {
    let alice = create_test_persona(&pool, "Race_Alice", "race_alice").await;
    let bob = create_test_persona(&pool, "Race_Bob", "race_bob").await;
    let fixtures = registry(&[("fixture", 1)]);
    let session_id = create_game_session(&pool, &fixtures, &[alice.id, bob.id], "fixture", 1).await;
    let app = router_with_game_registry(pool.clone(), MfaCipher::test_cipher(), fixtures);
    let alice_path = format!(
        "/v1/personas/{}/game-sessions/{session_id}/commands",
        alice.id
    );
    let bob_path = format!(
        "/v1/personas/{}/game-sessions/{session_id}/commands",
        bob.id
    );

    let alice_request = request_json(
        app.clone(),
        &alice_path,
        &alice.token,
        json!({
            "idempotency_key": command_key(1),
            "expected_revision": 0,
            "command": {"kind": "advance", "racer": "alice"}
        }),
    );
    let bob_request = request_json(
        app,
        &bob_path,
        &bob.token,
        json!({
            "idempotency_key": command_key(2),
            "expected_revision": 0,
            "command": {"kind": "advance", "racer": "bob"}
        }),
    );
    let (alice_response, bob_response) = tokio::join!(alice_request, bob_request);
    let statuses = [alice_response.status, bob_response.status];
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::OK)
            .count(),
        1
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::CONFLICT)
            .count(),
        1
    );
    let loser = if alice_response.status == StatusCode::CONFLICT {
        &alice_response
    } else {
        &bob_response
    };
    assert_eq!(loser.json()["error"]["code"], "game_revision_conflict");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT revision FROM game_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .expect("raced session should be readable"),
        1
    );
    assert_eq!(command_receipt_count(&pool, session_id).await, 1);
    assert_eq!(game_sync_event_count(&pool, session_id).await, 4);
}

async fn create_game_session(
    pool: &PgPool,
    fixtures: &GameRegistry,
    participant_ids: &[Uuid],
    game_key: &str,
    game_version: u32,
) -> Uuid {
    let mut transaction = pool.begin().await.expect("transaction should start");
    let session_id = games::create_session(
        &mut transaction,
        fixtures,
        game_key,
        game_version,
        participant_ids,
    )
    .await
    .expect("fixture game session should initialize");
    transaction
        .commit()
        .await
        .expect("fixture game session should commit");
    session_id
}

async fn command_receipt_count(pool: &PgPool, session_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM game_session_commands WHERE game_session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("game command receipts should be countable")
}

async fn game_sync_event_count(pool: &PgPool, session_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM persona_sync_events WHERE game_session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("game sync events should be countable")
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
            device_name: "game test".to_owned(),
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
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("router should respond");
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

async fn request_json(app: Router, path: &str, token: &str, body: Value) -> TestResponse {
    request_json_text(app, path, token, &body.to_string()).await
}

async fn request_json_text(app: Router, path: &str, token: &str, body: &str) -> TestResponse {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_owned()))
                .expect("JSON request should build"),
        )
        .await
        .expect("router should respond");
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

fn assert_no_store(response: &TestResponse) {
    assert_eq!(
        response
            .headers
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
}
