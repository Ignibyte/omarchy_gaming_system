use axum::{
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{app::router, connections::MAX_PENDING_REQUESTS_PER_DIRECTION, mfa::MfaCipher};

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
    handle: String,
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn connection_requests_are_idempotent_directional_private_and_owner_scoped(pool: PgPool) {
    let alice = create_test_persona(
        &pool,
        "Connect_Alice",
        "TEST-ONLY-alice-passphrase",
        "connect_alice",
    )
    .await;
    let alice_alt = create_persona(&pool, &alice.token, "connect_alice_alt").await;
    let bob = create_test_persona(
        &pool,
        "Connect_Bob",
        "TEST-ONLY-bob-passphrase",
        "connect_bob",
    )
    .await;
    let carol = create_test_persona(
        &pool,
        "Connect_Carol",
        "TEST-ONLY-carol-passphrase",
        "connect_carol",
    )
    .await;

    let created = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(alice.id, bob.id),
        &alice.token,
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_no_store(&created);
    assert_request_entry(&created.json(), &bob.handle);
    assert_private_fields_absent(&created.body);

    let retried = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(alice.id, bob.id),
        &alice.token,
    )
    .await;
    assert_eq!(retried.status, StatusCode::OK);
    assert_eq!(retried.json(), created.json());

    let reverse = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(bob.id, alice.id),
        &bob.token,
    )
    .await;
    assert_eq!(reverse.status, StatusCode::CONFLICT);
    assert_eq!(
        reverse.json()["error"]["code"],
        "connection_request_pending"
    );

    let carol_request = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(carol.id, alice.id),
        &carol.token,
    )
    .await;
    assert_eq!(carol_request.status, StatusCode::CREATED);

    let alice_inventory = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/connection-requests", alice.id),
        &alice.token,
    )
    .await;
    assert_eq!(alice_inventory.status, StatusCode::OK);
    assert_no_store(&alice_inventory);
    assert_eq!(
        alice_inventory.json()["incoming"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        alice_inventory.json()["outgoing"].as_array().map(Vec::len),
        Some(1)
    );
    assert_request_entry(&alice_inventory.json()["incoming"][0], &carol.handle);
    assert_request_entry(&alice_inventory.json()["outgoing"][0], &bob.handle);
    assert_private_fields_absent(&alice_inventory.body);

    let bob_inventory = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/connection-requests", bob.id),
        &bob.token,
    )
    .await;
    assert_eq!(bob_inventory.status, StatusCode::OK);
    assert_request_entry(&bob_inventory.json()["incoming"][0], &alice.handle);
    assert_eq!(bob_inventory.json()["outgoing"], json!([]));

    let alt_inventory = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{alice_alt}/connection-requests"),
        &alice.token,
    )
    .await;
    assert_eq!(alt_inventory.status, StatusCode::OK);
    assert_eq!(
        alt_inventory.json(),
        json!({"incoming": [], "outgoing": []})
    );

    let foreign_actor = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/connection-requests", bob.id),
        &alice.token,
    )
    .await;
    let absent_actor = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/connection-requests", Uuid::nil()),
        &alice.token,
    )
    .await;
    let malformed_actor = social_request(
        &pool,
        Method::GET,
        "/v1/personas/not-a-uuid/connection-requests",
        &alice.token,
    )
    .await;
    for rejected in [&foreign_actor, &absent_actor, &malformed_actor] {
        assert_eq!(rejected.status, StatusCode::NOT_FOUND);
        assert_eq!(rejected.json()["error"]["code"], "persona_not_found");
    }
    assert_eq!(foreign_actor.body, absent_actor.body);
    assert_eq!(absent_actor.body, malformed_actor.body);

    let unauthenticated_malformed = social_request(
        &pool,
        Method::GET,
        "/v1/personas/not-a-uuid/connection-requests",
        "not-a-valid-session",
    )
    .await;
    assert_eq!(unauthenticated_malformed.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        unauthenticated_malformed.json()["error"]["code"],
        "invalid_session"
    );

    let same_account = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(alice.id, alice_alt),
        &alice.token,
    )
    .await;
    let absent_target = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(alice.id, Uuid::nil()),
        &alice.token,
    )
    .await;
    let malformed_target = social_request(
        &pool,
        Method::PUT,
        &format!("/v1/personas/{}/connection-requests/not-a-uuid", alice.id),
        &alice.token,
    )
    .await;
    for rejected in [&same_account, &absent_target, &malformed_target] {
        assert_eq!(rejected.status, StatusCode::CONFLICT);
        assert_eq!(rejected.json()["error"]["code"], "connection_unavailable");
    }
    assert_eq!(same_account.body, absent_target.body);
    assert_eq!(absent_target.body, malformed_target.body);

    let rows = sqlx::query_as::<_, (Uuid, Uuid, Uuid, Uuid, String)>(
        r#"
        SELECT persona_low_id, persona_high_id, requester_id, addressee_id, status
        FROM persona_connections
        ORDER BY requester_id
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("connection requests should be readable");
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|row| row.0 < row.1 && row.2 != row.3 && row.4 == "pending")
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn pending_request_limits_are_bounded_and_race_safe(pool: PgPool) {
    let attacker = create_test_persona(
        &pool,
        "Cap_Attacker",
        "TEST-ONLY-attacker-passphrase",
        "cap_attacker",
    )
    .await;
    let victim = create_test_persona(
        &pool,
        "Cap_Victim",
        "TEST-ONLY-victim-passphrase",
        "cap_victim",
    )
    .await;
    let incoming_actors = insert_test_personas(
        &pool,
        attacker.id,
        "cap_in",
        MAX_PENDING_REQUESTS_PER_DIRECTION as usize + 1,
    )
    .await;
    for actor_id in incoming_actors
        .iter()
        .take(MAX_PENDING_REQUESTS_PER_DIRECTION as usize - 1)
    {
        insert_pending_request(&pool, *actor_id, victim.id).await;
    }

    let incoming_first_path = connection_request_path(
        incoming_actors[MAX_PENDING_REQUESTS_PER_DIRECTION as usize - 1],
        victim.id,
    );
    let incoming_second_path = connection_request_path(
        incoming_actors[MAX_PENDING_REQUESTS_PER_DIRECTION as usize],
        victim.id,
    );
    let incoming_first = social_request(&pool, Method::PUT, &incoming_first_path, &attacker.token);
    let incoming_second =
        social_request(&pool, Method::PUT, &incoming_second_path, &attacker.token);
    let (incoming_first, incoming_second) = tokio::join!(incoming_first, incoming_second);
    assert_one_created_and_one_limited(&incoming_first, &incoming_second);
    let winning_incoming_path = if incoming_first.status == StatusCode::CREATED {
        incoming_first_path
    } else {
        incoming_second_path
    };
    assert_eq!(
        social_request(&pool, Method::PUT, &winning_incoming_path, &attacker.token,)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        social_request(
            &pool,
            Method::GET,
            &format!("/v1/personas/{}/connection-requests", victim.id),
            &victim.token,
        )
        .await
        .json()["incoming"]
            .as_array()
            .map(Vec::len),
        Some(MAX_PENDING_REQUESTS_PER_DIRECTION as usize)
    );

    let requester = create_test_persona(
        &pool,
        "Cap_Requester",
        "TEST-ONLY-requester-passphrase",
        "cap_requester",
    )
    .await;
    let target_owner = create_test_persona(
        &pool,
        "Cap_Targets",
        "TEST-ONLY-targets-passphrase",
        "cap_targets",
    )
    .await;
    let outgoing_targets = insert_test_personas(
        &pool,
        target_owner.id,
        "cap_out",
        MAX_PENDING_REQUESTS_PER_DIRECTION as usize + 1,
    )
    .await;
    for target_id in outgoing_targets
        .iter()
        .take(MAX_PENDING_REQUESTS_PER_DIRECTION as usize - 1)
    {
        insert_pending_request(&pool, requester.id, *target_id).await;
    }

    let outgoing_first_path = connection_request_path(
        requester.id,
        outgoing_targets[MAX_PENDING_REQUESTS_PER_DIRECTION as usize - 1],
    );
    let outgoing_second_path = connection_request_path(
        requester.id,
        outgoing_targets[MAX_PENDING_REQUESTS_PER_DIRECTION as usize],
    );
    let outgoing_first = social_request(&pool, Method::PUT, &outgoing_first_path, &requester.token);
    let outgoing_second =
        social_request(&pool, Method::PUT, &outgoing_second_path, &requester.token);
    let (outgoing_first, outgoing_second) = tokio::join!(outgoing_first, outgoing_second);
    assert_one_created_and_one_limited(&outgoing_first, &outgoing_second);
    assert_eq!(
        social_request(
            &pool,
            Method::GET,
            &format!("/v1/personas/{}/connection-requests", requester.id),
            &requester.token,
        )
        .await
        .json()["outgoing"]
            .as_array()
            .map(Vec::len),
        Some(MAX_PENDING_REQUESTS_PER_DIRECTION as usize)
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn acceptance_and_removal_are_participant_scoped_mutual_and_idempotent(pool: PgPool) {
    let alice = create_test_persona(
        &pool,
        "Accept_Alice",
        "TEST-ONLY-alice-passphrase",
        "accept_alice",
    )
    .await;
    let bob = create_test_persona(
        &pool,
        "Accept_Bob",
        "TEST-ONLY-bob-passphrase",
        "accept_bob",
    )
    .await;
    let carol = create_test_persona(
        &pool,
        "Accept_Carol",
        "TEST-ONLY-carol-passphrase",
        "accept_carol",
    )
    .await;

    assert_eq!(
        social_request(
            &pool,
            Method::PUT,
            &connection_request_path(alice.id, bob.id),
            &alice.token,
        )
        .await
        .status,
        StatusCode::CREATED
    );

    let requester_self_accept = social_request(
        &pool,
        Method::PUT,
        &connection_path(alice.id, bob.id),
        &alice.token,
    )
    .await;
    assert_eq!(requester_self_accept.status, StatusCode::NOT_FOUND);
    assert_eq!(
        requester_self_accept.json()["error"]["code"],
        "connection_request_not_found"
    );

    let foreign_actor = social_request(
        &pool,
        Method::PUT,
        &connection_path(bob.id, alice.id),
        &carol.token,
    )
    .await;
    assert_eq!(foreign_actor.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign_actor.json()["error"]["code"], "persona_not_found");

    let accepted = social_request(
        &pool,
        Method::PUT,
        &connection_path(bob.id, alice.id),
        &bob.token,
    )
    .await;
    assert_eq!(accepted.status, StatusCode::OK);
    assert_no_store(&accepted);
    assert_connection_entry(&accepted.json(), &alice.handle);
    assert_private_fields_absent(&accepted.body);

    let acceptance_retry = social_request(
        &pool,
        Method::PUT,
        &connection_path(bob.id, alice.id),
        &bob.token,
    )
    .await;
    assert_eq!(acceptance_retry.status, StatusCode::OK);
    assert_eq!(acceptance_retry.json(), accepted.json());

    for (persona, expected_handle) in [(&alice, &bob.handle), (&bob, &alice.handle)] {
        let inventory = social_request(
            &pool,
            Method::GET,
            &format!("/v1/personas/{}/connections", persona.id),
            &persona.token,
        )
        .await;
        assert_eq!(inventory.status, StatusCode::OK);
        assert_no_store(&inventory);
        let connections = inventory.json()["connections"]
            .as_array()
            .cloned()
            .expect("connection inventory should be an array");
        assert_eq!(connections.len(), 1);
        assert_connection_entry(&connections[0], expected_handle);
        assert_private_fields_absent(&inventory.body);
    }

    let duplicate_request = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(alice.id, bob.id),
        &alice.token,
    )
    .await;
    assert_eq!(duplicate_request.status, StatusCode::CONFLICT);
    assert_eq!(
        duplicate_request.json()["error"]["code"],
        "connection_already_exists"
    );

    for _ in 0..2 {
        let removal = social_request(
            &pool,
            Method::DELETE,
            &connection_path(alice.id, bob.id),
            &alice.token,
        )
        .await;
        assert_eq!(removal.status, StatusCode::NO_CONTENT);
        assert!(removal.body.is_empty());
    }

    for persona in [&alice, &bob] {
        let inventory = social_request(
            &pool,
            Method::GET,
            &format!("/v1/personas/{}/connections", persona.id),
            &persona.token,
        )
        .await;
        assert_eq!(inventory.json(), json!({"connections": []}));
    }

    assert_eq!(
        social_request(
            &pool,
            Method::PUT,
            &connection_request_path(alice.id, bob.id),
            &alice.token,
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        social_request(
            &pool,
            Method::DELETE,
            &connection_path(bob.id, alice.id),
            &bob.token,
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM persona_connections")
            .fetch_one(&pool)
            .await
            .expect("relationship count should be readable"),
        0
    );

    let malformed_delete = social_request(
        &pool,
        Method::DELETE,
        &format!("/v1/personas/{}/connections/not-a-uuid", alice.id),
        &alice.token,
    )
    .await;
    assert_eq!(malformed_delete.status, StatusCode::NO_CONTENT);

    let missing_accept = social_request(
        &pool,
        Method::PUT,
        &connection_path(bob.id, carol.id),
        &bob.token,
    )
    .await;
    assert_eq!(missing_accept.status, StatusCode::NOT_FOUND);
    assert_eq!(
        missing_accept.json()["error"]["code"],
        "connection_request_not_found"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn blocks_are_private_atomic_retryable_and_win_request_races(pool: PgPool) {
    let alice = create_test_persona(
        &pool,
        "Block_Alice",
        "TEST-ONLY-alice-passphrase",
        "block_alice",
    )
    .await;
    let bob =
        create_test_persona(&pool, "Block_Bob", "TEST-ONLY-bob-passphrase", "block_bob").await;
    let carol = create_test_persona(
        &pool,
        "Block_Carol",
        "TEST-ONLY-carol-passphrase",
        "block_carol",
    )
    .await;

    create_and_accept(&pool, &alice, &bob).await;
    let blocked = social_request(
        &pool,
        Method::PUT,
        &block_path(alice.id, bob.id),
        &alice.token,
    )
    .await;
    assert_eq!(blocked.status, StatusCode::CREATED);
    assert_no_store(&blocked);
    assert_block_entry(&blocked.json(), &bob.handle);
    assert_private_fields_absent(&blocked.body);

    let block_retry = social_request(
        &pool,
        Method::PUT,
        &block_path(alice.id, bob.id),
        &alice.token,
    )
    .await;
    assert_eq!(block_retry.status, StatusCode::OK);
    assert_eq!(block_retry.json(), blocked.json());

    let alice_connections = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/connections", alice.id),
        &alice.token,
    )
    .await;
    assert_eq!(alice_connections.json(), json!({"connections": []}));

    let alice_blocks = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/blocks", alice.id),
        &alice.token,
    )
    .await;
    assert_eq!(alice_blocks.status, StatusCode::OK);
    assert_no_store(&alice_blocks);
    assert_eq!(
        alice_blocks.json()["blocks"].as_array().map(Vec::len),
        Some(1)
    );
    assert_block_entry(&alice_blocks.json()["blocks"][0], &bob.handle);

    let bob_blocks = social_request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/blocks", bob.id),
        &bob.token,
    )
    .await;
    assert_eq!(bob_blocks.json(), json!({"blocks": []}));

    let alice_to_bob = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(alice.id, bob.id),
        &alice.token,
    )
    .await;
    let bob_to_alice = social_request(
        &pool,
        Method::PUT,
        &connection_request_path(bob.id, alice.id),
        &bob.token,
    )
    .await;
    assert_eq!(alice_to_bob.status, StatusCode::CONFLICT);
    assert_eq!(alice_to_bob.body, bob_to_alice.body);
    assert_eq!(
        alice_to_bob.json()["error"]["code"],
        "connection_unavailable"
    );

    for _ in 0..2 {
        assert_eq!(
            social_request(
                &pool,
                Method::DELETE,
                &block_path(alice.id, bob.id),
                &alice.token,
            )
            .await
            .status,
            StatusCode::NO_CONTENT
        );
    }
    assert_eq!(
        social_request(
            &pool,
            Method::GET,
            &format!("/v1/personas/{}/connections", alice.id),
            &alice.token,
        )
        .await
        .json(),
        json!({"connections": []})
    );
    assert_eq!(
        social_request(
            &pool,
            Method::PUT,
            &connection_request_path(alice.id, bob.id),
            &alice.token,
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        social_request(
            &pool,
            Method::DELETE,
            &connection_path(alice.id, bob.id),
            &alice.token,
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );

    let raced_request_path = connection_request_path(carol.id, alice.id);
    let raced_block_path = block_path(alice.id, carol.id);
    let request_future = social_request(&pool, Method::PUT, &raced_request_path, &carol.token);
    let block_future = social_request(&pool, Method::PUT, &raced_block_path, &alice.token);
    let (request_result, block_result) = tokio::join!(request_future, block_future);
    assert!(matches!(
        request_result.status,
        StatusCode::CREATED | StatusCode::CONFLICT
    ));
    assert_eq!(block_result.status, StatusCode::CREATED);

    let pair_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM persona_connections
        WHERE (persona_low_id = LEAST($1, $2) AND persona_high_id = GREATEST($1, $2))
        "#,
    )
    .bind(alice.id)
    .bind(carol.id)
    .fetch_one(&pool)
    .await
    .expect("raced relationship count should be readable");
    let block_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM persona_blocks WHERE blocker_id = $1 AND blocked_id = $2",
    )
    .bind(alice.id)
    .bind(carol.id)
    .fetch_one(&pool)
    .await
    .expect("raced block count should be readable");
    assert_eq!(pair_count, 0);
    assert_eq!(block_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn opposite_requests_and_acceptance_are_serialized_to_one_pair(pool: PgPool) {
    let alice = create_test_persona(
        &pool,
        "Race_Alice",
        "TEST-ONLY-alice-passphrase",
        "race_alice",
    )
    .await;
    let bob = create_test_persona(&pool, "Race_Bob", "TEST-ONLY-bob-passphrase", "race_bob").await;

    let alice_request_path = connection_request_path(alice.id, bob.id);
    let bob_request_path = connection_request_path(bob.id, alice.id);
    let alice_future = social_request(&pool, Method::PUT, &alice_request_path, &alice.token);
    let bob_future = social_request(&pool, Method::PUT, &bob_request_path, &bob.token);
    let (alice_result, bob_result) = tokio::join!(alice_future, bob_future);
    let statuses = [alice_result.status, bob_result.status];
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
            .filter(|status| **status == StatusCode::CONFLICT)
            .count(),
        1
    );

    let (requester_id, addressee_id, status) = sqlx::query_as::<_, (Uuid, Uuid, String)>(
        "SELECT requester_id, addressee_id, status FROM persona_connections",
    )
    .fetch_one(&pool)
    .await
    .expect("one serialized request should exist");
    assert_eq!(status, "pending");
    let (addressee, requester) = if addressee_id == alice.id {
        (&alice, &bob)
    } else {
        (&bob, &alice)
    };
    assert_eq!(requester.id, requester_id);

    let acceptance_path = connection_path(addressee.id, requester.id);
    let first_accept = social_request(&pool, Method::PUT, &acceptance_path, &addressee.token);
    let second_accept = social_request(&pool, Method::PUT, &acceptance_path, &addressee.token);
    let (first_result, second_result) = tokio::join!(first_accept, second_accept);
    assert_eq!(first_result.status, StatusCode::OK);
    assert_eq!(second_result.status, StatusCode::OK);
    assert_eq!(first_result.json(), second_result.json());

    let (pair_count, accepted_count) = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            count(*),
            count(*) FILTER (WHERE status = 'accepted' AND accepted_at IS NOT NULL)
        FROM persona_connections
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("serialized connection should be readable");
    assert_eq!((pair_count, accepted_count), (1, 1));
}

async fn create_and_accept(pool: &PgPool, requester: &TestPersona, addressee: &TestPersona) {
    assert_eq!(
        social_request(
            pool,
            Method::PUT,
            &connection_request_path(requester.id, addressee.id),
            &requester.token,
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        social_request(
            pool,
            Method::PUT,
            &connection_path(addressee.id, requester.id),
            &addressee.token,
        )
        .await
        .status,
        StatusCode::OK
    );
}

async fn create_test_persona(
    pool: &PgPool,
    username: &str,
    password: &str,
    handle: &str,
) -> TestPersona {
    register_account(pool, username, password).await;
    let token = create_session(pool, &username.to_ascii_lowercase(), password).await;
    let id = create_persona(pool, &token, handle).await;
    TestPersona {
        token,
        id,
        handle: handle.to_owned(),
    }
}

async fn insert_test_personas(
    pool: &PgPool,
    owner_persona_id: Uuid,
    handle_prefix: &str,
    count: usize,
) -> Vec<Uuid> {
    let account_id = sqlx::query_scalar::<_, Uuid>("SELECT account_id FROM personas WHERE id = $1")
        .bind(owner_persona_id)
        .fetch_one(pool)
        .await
        .expect("test persona owner should exist");
    let mut persona_ids = Vec::with_capacity(count);
    for index in 0..count {
        let handle = format!("{handle_prefix}_{index:03}");
        let persona_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO personas (account_id, handle, display_name) VALUES ($1, $2, $2) RETURNING id",
        )
        .bind(account_id)
        .bind(handle)
        .fetch_one(pool)
        .await
        .expect("test persona should be inserted");
        persona_ids.push(persona_id);
    }
    persona_ids
}

async fn insert_pending_request(pool: &PgPool, requester_id: Uuid, addressee_id: Uuid) {
    let (low_id, high_id) = if requester_id < addressee_id {
        (requester_id, addressee_id)
    } else {
        (addressee_id, requester_id)
    };
    sqlx::query(
        r#"
        INSERT INTO persona_connections (
            persona_low_id,
            persona_high_id,
            requester_id,
            addressee_id
        )
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(low_id)
    .bind(high_id)
    .bind(requester_id)
    .bind(addressee_id)
    .execute(pool)
    .await
    .expect("pending request fixture should be inserted");
}

fn assert_one_created_and_one_limited(first: &TestResponse, second: &TestResponse) {
    let statuses = [first.status, second.status];
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::CREATED)
            .count(),
        1
    );
    let limited = if first.status == StatusCode::CONFLICT {
        first
    } else {
        second
    };
    assert_eq!(limited.status, StatusCode::CONFLICT);
    assert_eq!(limited.json()["error"]["code"], "connection_unavailable");
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

async fn create_session(pool: &PgPool, username: &str, password: &str) -> String {
    let response = request(
        pool,
        Method::POST,
        "/v1/sessions",
        Some(json!({
            "username": username,
            "password": password,
            "device_name": "Connection API test"
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

async fn create_persona(pool: &PgPool, token: &str, handle: &str) -> Uuid {
    let response = social_request_with_payload(
        pool,
        Method::POST,
        "/v1/personas",
        token,
        Some(json!({"handle": handle, "display_name": handle})),
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED);
    Uuid::try_parse(
        response.json()["id"]
            .as_str()
            .expect("persona creation should return an ID"),
    )
    .expect("persona ID should be a UUID")
}

async fn social_request(pool: &PgPool, method: Method, uri: &str, token: &str) -> TestResponse {
    social_request_with_payload(pool, method, uri, token, None).await
}

async fn social_request_with_payload(
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

fn connection_request_path(actor_id: Uuid, target_id: Uuid) -> String {
    format!("/v1/personas/{actor_id}/connection-requests/{target_id}")
}

fn connection_path(actor_id: Uuid, other_id: Uuid) -> String {
    format!("/v1/personas/{actor_id}/connections/{other_id}")
}

fn block_path(actor_id: Uuid, target_id: Uuid) -> String {
    format!("/v1/personas/{actor_id}/blocks/{target_id}")
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

fn assert_request_entry(document: &Value, expected_handle: &str) {
    assert_exact_keys(document, &["created_at", "persona"]);
    assert_eq!(document["persona"]["handle"], expected_handle);
    assert_public_persona(&document["persona"]);
}

fn assert_connection_entry(document: &Value, expected_handle: &str) {
    assert_exact_keys(document, &["connected_at", "persona"]);
    assert_eq!(document["persona"]["handle"], expected_handle);
    assert_public_persona(&document["persona"]);
}

fn assert_block_entry(document: &Value, expected_handle: &str) {
    assert_exact_keys(document, &["created_at", "persona"]);
    assert_eq!(document["persona"]["handle"], expected_handle);
    assert_public_persona(&document["persona"]);
}

fn assert_public_persona(document: &Value) {
    assert_exact_keys(
        document,
        &[
            "bio",
            "created_at",
            "display_name",
            "handle",
            "id",
            "status_message",
            "updated_at",
        ],
    );
}

fn assert_exact_keys(document: &Value, expected: &[&str]) {
    let mut keys = document
        .as_object()
        .expect("response value should be a JSON object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    keys.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(keys, expected);
}

fn assert_private_fields_absent(body: &str) {
    for private_field in [
        "account_id",
        "token",
        "token_hash",
        "password",
        "session_id",
        "blocker_id",
        "blocked_id",
    ] {
        assert!(
            !body.contains(private_field),
            "social response exposed private field {private_field}"
        );
    }
}
