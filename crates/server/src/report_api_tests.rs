use axum::{
    Router,
    body::Body,
    http::{
        HeaderMap, Method, Request, StatusCode,
        header::{AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE},
    },
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    accounts::{self, RegistrationInput},
    app::router,
    mfa::MfaCipher,
    personas::{self, CreatePersonaInput},
    reports::{self, CreateReportInput, ReportOutcome},
    sessions::{self, CreateSessionInput, SessionCreation},
};

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
        serde_json::from_str(&self.body).expect("response should contain JSON")
    }
}

#[tokio::test]
async fn report_route_rejects_oversized_bodies_before_database_work() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://test:test@127.0.0.1:5432/test")
        .expect("test database URL should parse without connecting");
    let response = request_json(
        router(pool, MfaCipher::test_cipher()),
        Method::POST,
        &format!("/v1/personas/{}/reports", Uuid::nil()),
        "not-consulted-before-body-limit",
        json!({
            "idempotency_key": Uuid::nil(),
            "subject_persona_id": Uuid::nil(),
            "category": "other",
            "detail": "x".repeat(9 * 1024)
        }),
    )
    .await;
    assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_no_store(&response);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn report_creation_is_owner_scoped_private_and_exactly_idempotent(pool: PgPool) {
    let alice = create_test_persona(&pool, "report_alice", "report_alice").await;
    let bob = create_test_persona(&pool, "report_bob", "report_bob").await;
    let operation_id = Uuid::new_v4();
    let path = format!("/v1/personas/{}/reports", alice.id);
    let document = json!({
        "idempotency_key": operation_id,
        "subject_persona_id": bob.id,
        "category": "harassment",
        "detail": "  Repeated hostile private messages.\nPlease review.  "
    });

    let created = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &path,
        &alice.token,
        document.clone(),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert_no_store(&created);
    assert_exact_keys(
        &created.json(),
        &["created_at", "id", "idempotency_key", "status"],
    );
    assert_eq!(created.json()["idempotency_key"], operation_id.to_string());
    assert_eq!(created.json()["status"], "open");
    assert!(!created.body.contains("report_bob"));
    assert!(!created.body.contains("hostile"));
    assert!(!created.body.contains("account"));

    let replay = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &path,
        &alice.token,
        document.clone(),
    )
    .await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.json(), created.json());

    let collision = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &path,
        &alice.token,
        json!({
            "idempotency_key": operation_id,
            "subject_persona_id": bob.id,
            "category": "spam",
            "detail": "Repeated hostile private messages.\nPlease review."
        }),
    )
    .await;
    assert_error(
        &collision,
        StatusCode::CONFLICT,
        "report_idempotency_conflict",
    );

    let self_report = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &path,
        &alice.token,
        report_document(Uuid::new_v4(), alice.id),
    )
    .await;
    assert_error(
        &self_report,
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalid_report",
    );

    let foreign_reporter = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &format!("/v1/personas/{}/reports", bob.id),
        &alice.token,
        report_document(Uuid::new_v4(), alice.id),
    )
    .await;
    assert_error(
        &foreign_reporter,
        StatusCode::NOT_FOUND,
        "persona_not_found",
    );

    let absent_subject = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &path,
        &alice.token,
        report_document(Uuid::new_v4(), Uuid::new_v4()),
    )
    .await;
    assert_error(&absent_subject, StatusCode::NOT_FOUND, "persona_not_found");

    let invalid_category = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &path,
        &alice.token,
        json!({
            "idempotency_key": Uuid::new_v4(),
            "subject_persona_id": bob.id,
            "category": "credential_theft",
            "detail": "not an allowed category"
        }),
    )
    .await;
    assert_error(
        &invalid_category,
        StatusCode::UNPROCESSABLE_ENTITY,
        "invalid_report",
    );

    let invalid_session_precedes_path_validation = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        "/v1/personas/not-a-uuid/reports",
        "invalid-session",
        json!({
            "idempotency_key": "not-a-uuid",
            "subject_persona_id": "not-a-uuid",
            "category": "not-a-category",
            "detail": ""
        }),
    )
    .await;
    assert_error(
        &invalid_session_precedes_path_validation,
        StatusCode::UNAUTHORIZED,
        "invalid_session",
    );

    let stored = sqlx::query_as::<_, (Uuid, Uuid, String, String, String)>(
        r#"
        SELECT reporter_persona_id, subject_persona_id, category, detail, status
        FROM persona_reports
        WHERE idempotency_key = $1
        "#,
    )
    .bind(operation_id)
    .fetch_one(&pool)
    .await
    .expect("report should be stored once");
    assert_eq!(stored.0, alice.id);
    assert_eq!(stored.1, bob.id);
    assert_eq!(stored.2, "harassment");
    assert_eq!(
        stored.3,
        "Repeated hostile private messages.\nPlease review."
    );
    assert_eq!(stored.4, "open");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM persona_reports")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    sqlx::query(
        "UPDATE persona_reports SET status = 'resolved', updated_at = now(), closed_at = now() WHERE idempotency_key = $1",
    )
    .bind(operation_id)
    .execute(&pool)
    .await
    .expect("operator fixture should resolve the report");
    let replay_after_disposition = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &path,
        &alice.token,
        document,
    )
    .await;
    assert_eq!(replay_after_disposition.status, StatusCode::OK);
    assert_eq!(replay_after_disposition.json(), created.json());
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn report_open_cap_and_simultaneous_first_delivery_are_serialized(pool: PgPool) {
    let reporter = create_test_persona(&pool, "report_cap_actor", "report_cap_actor").await;
    let subject = create_test_persona(&pool, "report_cap_peer", "report_cap_peer").await;

    for index in 0..25 {
        let response = request_json(
            router(pool.clone(), MfaCipher::test_cipher()),
            Method::POST,
            &format!("/v1/personas/{}/reports", reporter.id),
            &reporter.token,
            json!({
                "idempotency_key": deterministic_uuid(index),
                "subject_persona_id": subject.id,
                "category": "other",
                "detail": format!("bounded report {index}")
            }),
        )
        .await;
        assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
    }

    let over_cap = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &format!("/v1/personas/{}/reports", reporter.id),
        &reporter.token,
        report_document(Uuid::new_v4(), subject.id),
    )
    .await;
    assert_error(
        &over_cap,
        StatusCode::TOO_MANY_REQUESTS,
        "report_limit_reached",
    );

    let replay_at_cap = request_json(
        router(pool.clone(), MfaCipher::test_cipher()),
        Method::POST,
        &format!("/v1/personas/{}/reports", reporter.id),
        &reporter.token,
        json!({
            "idempotency_key": deterministic_uuid(0),
            "subject_persona_id": subject.id,
            "category": "other",
            "detail": "bounded report 0"
        }),
    )
    .await;
    assert_eq!(replay_at_cap.status, StatusCode::OK);

    sqlx::query("UPDATE persona_reports SET status = 'resolved', updated_at = clock_timestamp(), closed_at = clock_timestamp()")
        .execute(&pool)
        .await
        .expect("fixture reports should close");
    let concurrent_key = Uuid::new_v4();
    let reporter_id = reporter.id.to_string();
    let first = reports::create_report(
        &pool,
        &reporter.token,
        &reporter_id,
        domain_input(concurrent_key, subject.id),
    );
    let second = reports::create_report(
        &pool,
        &reporter.token,
        &reporter_id,
        domain_input(concurrent_key, subject.id),
    );
    let (first, second) = tokio::join!(first, second);
    let outcomes = [first.unwrap(), second.unwrap()];
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, ReportOutcome::Created(_)))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, ReportOutcome::Existing(_)))
            .count(),
        1
    );
    let concurrent_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM persona_reports WHERE reporter_persona_id = $1 AND idempotency_key = $2",
    )
    .bind(reporter.id)
    .bind(concurrent_key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(concurrent_count, 1);

    let delete_error = sqlx::query("DELETE FROM persona_reports WHERE idempotency_key = $1")
        .bind(concurrent_key)
        .execute(&pool)
        .await
        .expect_err("reports must reject deletion");
    assert!(delete_error.to_string().contains("cannot be deleted"));
}

fn report_document(idempotency_key: Uuid, subject_persona_id: Uuid) -> Value {
    json!({
        "idempotency_key": idempotency_key,
        "subject_persona_id": subject_persona_id,
        "category": "other",
        "detail": "Please review this persona."
    })
}

fn domain_input(idempotency_key: Uuid, subject_persona_id: Uuid) -> CreateReportInput {
    CreateReportInput {
        idempotency_key: idempotency_key.to_string(),
        subject_persona_id: subject_persona_id.to_string(),
        category: "other".to_owned(),
        detail: "simultaneous delivery".to_owned(),
    }
}

fn deterministic_uuid(index: u128) -> Uuid {
    Uuid::from_u128(0x2900_0000_0000_4000_8000_0000_0000_0000 + index)
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
            device_name: "report test".to_owned(),
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

fn assert_error(response: &TestResponse, status: StatusCode, code: &str) {
    assert_eq!(response.status, status, "{}", response.body);
    assert_eq!(response.json()["error"]["code"], code);
    assert_exact_keys(&response.json(), &["error"]);
    assert_exact_keys(&response.json()["error"], &["code", "message"]);
    assert_no_store(response);
}

fn assert_exact_keys(value: &Value, expected: &[&str]) {
    let mut actual = value
        .as_object()
        .expect("value should be an object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

fn assert_no_store(response: &TestResponse) {
    assert_eq!(
        response
            .headers
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
}
