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

struct TestPersona {
    token: String,
    id: Uuid,
    handle: String,
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn acceptance_creates_one_private_conversation_and_one_typed_event(pool: PgPool) {
    let alice = create_test_persona(&pool, "Inbox_Alice", "inbox_alice").await;
    let bob = create_test_persona(&pool, "Inbox_Bob", "inbox_bob").await;
    let carol = create_test_persona(&pool, "Inbox_Carol", "inbox_carol").await;

    request_connection(&pool, &alice, &bob).await;
    let acceptance_path = connection_path(bob.id, alice.id);
    for _ in 0..2 {
        assert_eq!(
            request(&pool, Method::PUT, &acceptance_path, &bob.token, None)
                .await
                .status,
            StatusCode::OK
        );
    }

    let (conversation_count, event_count) = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            (SELECT count(*) FROM inbox_conversations),
            (SELECT count(*) FROM inbox_messages
             WHERE message_type = 'system' AND system_type = 'connection_accepted')
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("inbox rows should be countable");
    assert_eq!((conversation_count, event_count), (1, 1));

    let alice_inventory = inventory(&pool, &alice).await;
    let bob_inventory = inventory(&pool, &bob).await;
    assert_eq!(alice_inventory.status, StatusCode::OK);
    assert_eq!(bob_inventory.status, StatusCode::OK);
    assert_no_store(&alice_inventory);
    assert_no_store(&bob_inventory);

    let alice_conversation = &alice_inventory.json()["conversations"][0];
    let bob_conversation = &bob_inventory.json()["conversations"][0];
    assert_conversation(alice_conversation, &bob.handle, 1);
    assert_conversation(bob_conversation, &alice.handle, 0);
    assert_system_message(&alice_conversation["latest_message"], &bob.handle);
    assert_private_fields_absent(&alice_inventory.body);
    assert_private_fields_absent(&bob_inventory.body);

    let foreign_actor = request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/conversations", alice.id),
        &carol.token,
        None,
    )
    .await;
    let absent_actor = request(
        &pool,
        Method::GET,
        &format!("/v1/personas/{}/conversations", Uuid::nil()),
        &carol.token,
        None,
    )
    .await;
    assert_eq!(foreign_actor.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign_actor.body, absent_actor.body);
    assert_eq!(foreign_actor.json()["error"]["code"], "persona_not_found");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn user_messages_are_body_only_and_drive_private_monotonic_unread_state(pool: PgPool) {
    let alice = create_test_persona(&pool, "Unread_Alice", "unread_alice").await;
    let bob = create_test_persona(&pool, "Unread_Bob", "unread_bob").await;
    let conversation_id = create_and_accept(&pool, &alice, &bob).await;

    let created = request(
        &pool,
        Method::POST,
        &messages_path(bob.id, conversation_id),
        &bob.token,
        Some(json!({"body": "  ready\nnext turn  "})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_no_store(&created);
    assert_user_message(&created.json(), &bob.handle, "ready\nnext turn");
    assert_private_fields_absent(&created.body);
    let message_id = created.json()["id"]
        .as_str()
        .expect("created message should expose an ID")
        .to_owned();

    let forged_system = request(
        &pool,
        Method::POST,
        &messages_path(bob.id, conversation_id),
        &bob.token,
        Some(json!({"body": "forged", "type": "system"})),
    )
    .await;
    assert_eq!(forged_system.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_no_store(&forged_system);

    let invalid_body = request(
        &pool,
        Method::POST,
        &messages_path(bob.id, conversation_id),
        &bob.token,
        Some(json!({"body": " \t "})),
    )
    .await;
    assert_eq!(invalid_body.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_no_store(&invalid_body);
    assert_eq!(invalid_body.json()["error"]["code"], "invalid_message_body");

    assert_eq!(
        inventory(&pool, &alice).await.json()["conversations"][0]["unread_count"],
        2
    );
    assert_eq!(
        inventory(&pool, &bob).await.json()["conversations"][0]["unread_count"],
        0
    );

    let read_path = format!(
        "/v1/personas/{}/conversations/{conversation_id}/read/{message_id}",
        alice.id
    );
    for _ in 0..2 {
        let read = request(&pool, Method::PUT, &read_path, &alice.token, None).await;
        assert_eq!(read.status, StatusCode::OK);
        assert_no_store(&read);
        assert_eq!(
            read.json(),
            json!({"through_message_id": message_id, "unread_count": 0})
        );
    }
    assert_eq!(
        inventory(&pool, &alice).await.json()["conversations"][0]["unread_count"],
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn message_sequences_are_local_to_each_conversation(pool: PgPool) {
    let alice = create_test_persona(&pool, "Local_Alice", "local_alice").await;
    let bob = create_test_persona(&pool, "Local_Bob", "local_bob").await;
    let carol = create_test_persona(&pool, "Local_Carol", "local_carol").await;
    let dave = create_test_persona(&pool, "Local_Dave", "local_dave").await;
    let first_conversation = create_and_accept(&pool, &alice, &bob).await;
    let second_conversation = create_and_accept(&pool, &carol, &dave).await;

    let first_message = request(
        &pool,
        Method::POST,
        &messages_path(alice.id, first_conversation),
        &alice.token,
        Some(json!({"body": "first thread"})),
    )
    .await;
    let unrelated_message = request(
        &pool,
        Method::POST,
        &messages_path(carol.id, second_conversation),
        &carol.token,
        Some(json!({"body": "unrelated thread"})),
    )
    .await;
    let second_message = request(
        &pool,
        Method::POST,
        &messages_path(alice.id, first_conversation),
        &alice.token,
        Some(json!({"body": "first thread again"})),
    )
    .await;

    assert_eq!(first_message.status, StatusCode::CREATED);
    assert_eq!(unrelated_message.status, StatusCode::CREATED);
    assert_eq!(second_message.status, StatusCode::CREATED);
    assert_eq!(first_message.json()["sequence"], 2);
    assert_eq!(unrelated_message.json()["sequence"], 2);
    assert_eq!(second_message.json()["sequence"], 3);

    let sequences = sqlx::query_as::<_, (Uuid, Vec<i64>)>(
        r#"
        SELECT conversation_id, array_agg(message_sequence ORDER BY message_sequence)
        FROM inbox_messages
        GROUP BY conversation_id
        ORDER BY conversation_id
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("conversation-local message sequences should be readable");
    assert_eq!(sequences.len(), 2);
    assert!(
        sequences
            .iter()
            .all(|row| row.1 == vec![1, 2] || row.1 == vec![1, 2, 3])
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn history_is_bounded_private_and_survives_disconnect_and_block(pool: PgPool) {
    let alice = create_test_persona(&pool, "History_Alice", "history_alice").await;
    let bob = create_test_persona(&pool, "History_Bob", "history_bob").await;
    let carol = create_test_persona(&pool, "History_Carol", "history_carol").await;
    let conversation_id = create_and_accept(&pool, &alice, &bob).await;

    for body in ["one", "two", "three"] {
        let response = request(
            &pool,
            Method::POST,
            &messages_path(alice.id, conversation_id),
            &alice.token,
            Some(json!({"body": body})),
        )
        .await;
        assert_eq!(response.status, StatusCode::CREATED);
    }

    let newest = request(
        &pool,
        Method::GET,
        &format!("{}?limit=2", messages_path(alice.id, conversation_id)),
        &alice.token,
        None,
    )
    .await;
    assert_eq!(newest.status, StatusCode::OK);
    assert_no_store(&newest);
    let malformed_query = request(
        &pool,
        Method::GET,
        &format!(
            "{}?limit=not-a-number",
            messages_path(alice.id, conversation_id)
        ),
        &alice.token,
        None,
    )
    .await;
    assert_eq!(malformed_query.status, StatusCode::BAD_REQUEST);
    assert_no_store(&malformed_query);
    let newest_json = newest.json();
    let newest_messages = newest_json["messages"]
        .as_array()
        .expect("messages should be an array");
    assert_eq!(newest_messages.len(), 2);
    assert!(newest_messages[0]["sequence"].as_i64() < newest_messages[1]["sequence"].as_i64());
    assert_eq!(newest_messages[0]["body"], "two");
    assert_eq!(newest_messages[1]["body"], "three");
    let next_before = newest_json["next_before"]
        .as_i64()
        .expect("a bounded newest page should expose its older cursor");

    let older = request(
        &pool,
        Method::GET,
        &format!(
            "{}?limit=2&before={next_before}",
            messages_path(alice.id, conversation_id)
        ),
        &alice.token,
        None,
    )
    .await;
    assert_eq!(older.status, StatusCode::OK);
    let older_json = older.json();
    assert_eq!(older_json["messages"].as_array().map(Vec::len), Some(2));
    assert_system_message(&older_json["messages"][0], &bob.handle);
    assert_eq!(older_json["messages"][1]["body"], "one");
    assert!(older_json["next_before"].is_null());

    let foreign = request(
        &pool,
        Method::GET,
        &messages_path(carol.id, conversation_id),
        &carol.token,
        None,
    )
    .await;
    let absent = request(
        &pool,
        Method::GET,
        &messages_path(carol.id, Uuid::nil()),
        &carol.token,
        None,
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(foreign.body, absent.body);
    assert_eq!(foreign.json()["error"]["code"], "conversation_not_found");

    assert_eq!(
        request(
            &pool,
            Method::DELETE,
            &connection_path(alice.id, bob.id),
            &alice.token,
            None,
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_history_readable_but_send_denied(&pool, &alice, conversation_id).await;

    assert_eq!(
        request(
            &pool,
            Method::PUT,
            &block_path(bob.id, alice.id),
            &bob.token,
            None,
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_history_readable_but_send_denied(&pool, &alice, conversation_id).await;
    assert_eq!(
        request(
            &pool,
            Method::DELETE,
            &block_path(bob.id, alice.id),
            &bob.token,
            None,
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_history_readable_but_send_denied(&pool, &alice, conversation_id).await;

    request_connection(&pool, &alice, &bob).await;
    assert_eq!(
        request(
            &pool,
            Method::PUT,
            &connection_path(bob.id, alice.id),
            &bob.token,
            None,
        )
        .await
        .status,
        StatusCode::OK
    );
    let resumed = request(
        &pool,
        Method::POST,
        &messages_path(alice.id, conversation_id),
        &alice.token,
        Some(json!({"body": "reconnected"})),
    )
    .await;
    assert_eq!(resumed.status, StatusCode::CREATED);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM inbox_conversations")
            .fetch_one(&pool)
            .await
            .expect("conversation count should be readable"),
        1
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn concurrent_sends_serialize_and_concurrent_reads_never_move_backward(pool: PgPool) {
    let alice = create_test_persona(&pool, "Race_Inbox_Alice", "race_inbox_alice").await;
    let bob = create_test_persona(&pool, "Race_Inbox_Bob", "race_inbox_bob").await;
    let conversation_id = create_and_accept(&pool, &alice, &bob).await;

    let alice_path = messages_path(alice.id, conversation_id);
    let bob_path = messages_path(bob.id, conversation_id);
    let alice_send = request(
        &pool,
        Method::POST,
        &alice_path,
        &alice.token,
        Some(json!({"body": "from alice"})),
    );
    let bob_send = request(
        &pool,
        Method::POST,
        &bob_path,
        &bob.token,
        Some(json!({"body": "from bob"})),
    );
    let (alice_result, bob_result) = tokio::join!(alice_send, bob_send);
    assert_eq!(alice_result.status, StatusCode::CREATED);
    assert_eq!(bob_result.status, StatusCode::CREATED);

    let first = request(
        &pool,
        Method::POST,
        &bob_path,
        &bob.token,
        Some(json!({"body": "read first"})),
    )
    .await;
    let second = request(
        &pool,
        Method::POST,
        &bob_path,
        &bob.token,
        Some(json!({"body": "read second"})),
    )
    .await;
    let first_id = first.json()["id"]
        .as_str()
        .expect("first message ID")
        .to_owned();
    let second_id = second.json()["id"]
        .as_str()
        .expect("second message ID")
        .to_owned();
    let first_read_path = read_path(alice.id, conversation_id, &first_id);
    let second_read_path = read_path(alice.id, conversation_id, &second_id);
    let first_read = request(&pool, Method::PUT, &first_read_path, &alice.token, None);
    let second_read = request(&pool, Method::PUT, &second_read_path, &alice.token, None);
    let (first_result, second_result) = tokio::join!(first_read, second_read);
    assert_eq!(first_result.status, StatusCode::OK);
    assert_eq!(second_result.status, StatusCode::OK);

    let sequences = sqlx::query_scalar::<_, i64>(
        "SELECT message_sequence FROM inbox_messages ORDER BY message_sequence",
    )
    .fetch_all(&pool)
    .await
    .expect("message sequences should be readable");
    assert_eq!(sequences.len(), 5);
    assert!(sequences.windows(2).all(|window| window[0] < window[1]));
    assert_eq!(
        inventory(&pool, &alice).await.json()["conversations"][0]["unread_count"],
        0
    );

    let (low_id, low_read, high_read, latest) = sqlx::query_as::<_, (Uuid, i64, i64, i64)>(
        r#"
        SELECT persona_low_id, low_last_read_sequence, high_last_read_sequence,
               last_message_sequence
        FROM inbox_conversations
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("conversation read positions should be readable");
    let alice_read = if alice.id == low_id {
        low_read
    } else {
        high_read
    };
    assert_eq!(alice_read, latest);
}

async fn create_and_accept(
    pool: &PgPool,
    requester: &TestPersona,
    addressee: &TestPersona,
) -> Uuid {
    request_connection(pool, requester, addressee).await;
    let accepted = request(
        pool,
        Method::PUT,
        &connection_path(addressee.id, requester.id),
        &addressee.token,
        None,
    )
    .await;
    assert_eq!(accepted.status, StatusCode::OK);
    let inventory = inventory(pool, requester).await;
    Uuid::try_parse(
        inventory.json()["conversations"][0]["id"]
            .as_str()
            .expect("accepted pair should expose a conversation ID"),
    )
    .expect("conversation ID should be a UUID")
}

async fn request_connection(pool: &PgPool, requester: &TestPersona, addressee: &TestPersona) {
    assert_eq!(
        request(
            pool,
            Method::PUT,
            &format!(
                "/v1/personas/{}/connection-requests/{}",
                requester.id, addressee.id
            ),
            &requester.token,
            None,
        )
        .await
        .status,
        StatusCode::CREATED
    );
}

async fn create_test_persona(pool: &PgPool, username: &str, handle: &str) -> TestPersona {
    let password = "TEST-ONLY-inbox-passphrase";
    let invite_code = crate::accounts::create_test_invite(pool).await;
    assert_eq!(
        request(
            pool,
            Method::POST,
            "/v1/accounts",
            "",
            Some(json!({
                "invite_code": invite_code,
                "username": username,
                "password": password
            })),
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
            "device_name": "Inbox API test"
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
    let id = Uuid::try_parse(persona.json()["id"].as_str().expect("persona ID"))
        .expect("persona ID should be a UUID");
    TestPersona {
        token,
        id,
        handle: handle.to_owned(),
    }
}

async fn inventory(pool: &PgPool, persona: &TestPersona) -> TestResponse {
    request(
        pool,
        Method::GET,
        &format!("/v1/personas/{}/conversations", persona.id),
        &persona.token,
        None,
    )
    .await
}

async fn assert_history_readable_but_send_denied(
    pool: &PgPool,
    persona: &TestPersona,
    conversation_id: Uuid,
) {
    assert_eq!(
        request(
            pool,
            Method::GET,
            &messages_path(persona.id, conversation_id),
            &persona.token,
            None,
        )
        .await
        .status,
        StatusCode::OK
    );
    let denied = request(
        pool,
        Method::POST,
        &messages_path(persona.id, conversation_id),
        &persona.token,
        Some(json!({"body": "not allowed"})),
    )
    .await;
    assert_eq!(denied.status, StatusCode::CONFLICT);
    assert_eq!(denied.json()["error"]["code"], "conversation_unavailable");
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

fn connection_path(actor_id: Uuid, other_id: Uuid) -> String {
    format!("/v1/personas/{actor_id}/connections/{other_id}")
}

fn block_path(actor_id: Uuid, other_id: Uuid) -> String {
    format!("/v1/personas/{actor_id}/blocks/{other_id}")
}

fn messages_path(actor_id: Uuid, conversation_id: Uuid) -> String {
    format!("/v1/personas/{actor_id}/conversations/{conversation_id}/messages")
}

fn read_path(actor_id: Uuid, conversation_id: Uuid, message_id: &str) -> String {
    format!("/v1/personas/{actor_id}/conversations/{conversation_id}/read/{message_id}")
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

fn assert_conversation(document: &Value, expected_handle: &str, unread_count: i64) {
    assert_exact_keys(
        document,
        &[
            "created_at",
            "id",
            "latest_message",
            "other_persona",
            "unread_count",
            "updated_at",
        ],
    );
    assert_eq!(document["other_persona"]["handle"], expected_handle);
    assert_eq!(document["unread_count"], unread_count);
    assert_public_persona(&document["other_persona"]);
}

fn assert_user_message(document: &Value, expected_sender: &str, expected_body: &str) {
    assert_exact_keys(
        document,
        &["body", "created_at", "id", "sender", "sequence", "type"],
    );
    assert_eq!(document["type"], "user");
    assert_eq!(document["sender"]["handle"], expected_sender);
    assert_eq!(document["body"], expected_body);
    assert_public_persona(&document["sender"]);
}

fn assert_system_message(document: &Value, expected_actor: &str) {
    assert_exact_keys(
        document,
        &["created_at", "id", "sequence", "system", "type"],
    );
    assert_eq!(document["type"], "system");
    assert_exact_keys(&document["system"], &["actor", "type"]);
    assert_eq!(document["system"]["type"], "connection_accepted");
    assert_eq!(document["system"]["actor"]["handle"], expected_actor);
    assert_public_persona(&document["system"]["actor"]);
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
        "token_hash",
        "password_hash",
        "low_last_read_sequence",
        "high_last_read_sequence",
    ] {
        assert!(
            !body.contains(private_field),
            "inbox response exposed private field {private_field}"
        );
    }
}
