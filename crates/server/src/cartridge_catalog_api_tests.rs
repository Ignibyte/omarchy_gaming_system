use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt as _;

use crate::{
    accounts::{self, RegistrationInput},
    app::router,
    mfa::MfaCipher,
    sessions::{self, CreateSessionInput, SessionCreation},
};

const ARCHIVE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SIGNED_IDENTITY: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn authenticated_catalog_is_exact_no_store_and_lifecycle_filtered(pool: PgPool) {
    seed_deprecated_catalog(&pool).await;
    let token = create_session(&pool).await;

    let missing = get(router(pool.clone(), MfaCipher::test_cipher()), None).await;
    assert_eq!(missing.status, StatusCode::UNAUTHORIZED);
    assert_no_store(&missing);

    let invalid = get(
        router(pool.clone(), MfaCipher::test_cipher()),
        Some("invalid-token"),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::UNAUTHORIZED);
    assert_no_store(&invalid);

    let listed = get(router(pool.clone(), MfaCipher::test_cipher()), Some(&token)).await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_no_store(&listed);
    assert_eq!(
        listed.json(),
        json!({
            "cartridges": [{
                "game_key": "door-legends",
                "publisher_id": "ignibyte",
                "rules_version": 1,
                "cartridge_version": 2,
                "display_name": "Door Legends",
                "archive_sha256": ARCHIVE,
                "signed_identity_sha256": SIGNED_IDENTITY,
                "marketplace": {
                    "provenance_class": "marketplace_vetted",
                    "marketplace_id": "omarchygs-marketplace",
                    "marketplace_name": "OmarchyGS Marketplace",
                    "reviewed_by": "review-team",
                    "review_summary": "Bounded first-party review passed.",
                    "policy_version": 1,
                    "lifecycle_status": "deprecated"
                },
                "server_admission": {"revision": 4},
                "warning": "Upgrade when practical."
            }]
        })
    );
    for forbidden in [
        "release_path",
        "verifying_key",
        "PUBLIC-KEY-MATERIAL",
        "market.example.test",
        "/srv/cartridges",
        "javascript",
        "qml",
    ] {
        assert!(!listed.body.contains(forbidden), "leaked {forbidden}");
    }

    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET policy_version = 2,
            policy_status = 'suspended',
            policy_reason = 'Review paused.',
            signed_policy = '{"version":2}'::jsonb,
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $1
        "#,
    )
    .bind(ARCHIVE)
    .execute(&pool)
    .await
    .expect("newer suspended policy should apply");
    let hidden = get(router(pool.clone(), MfaCipher::test_cipher()), Some(&token)).await;
    assert_eq!(hidden.status, StatusCode::OK);
    assert_eq!(hidden.json(), json!({"cartridges": []}));

    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET policy_version = 3,
            policy_status = 'active',
            policy_reason = 'Review restored.',
            signed_policy = '{"version":3}'::jsonb,
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $1
        "#,
    )
    .bind(ARCHIVE)
    .execute(&pool)
    .await
    .expect("newer active policy should apply");
    sqlx::query(
        r#"
        UPDATE marketplace_sync_state
        SET snapshot_version = 2,
            snapshot_sha256 = $1,
            synchronized_at = clock_timestamp()
        WHERE singleton
        "#,
    )
    .bind("c".repeat(64))
    .execute(&pool)
    .await
    .expect("new snapshot should apply");
    let omitted = get(router(pool, MfaCipher::test_cipher()), Some(&token)).await;
    assert_eq!(omitted.status, StatusCode::OK);
    assert_eq!(omitted.json(), json!({"cartridges": []}));
}

async fn seed_deprecated_catalog(pool: &PgPool) {
    sqlx::query(
        r#"
        INSERT INTO marketplace_sync_state (
            marketplace_origin, authority_id, key_id, marketplace_name,
            snapshot_version, snapshot_sha256
        )
        VALUES (
            'https://market.example.test/v1/', 'omarchygs-marketplace',
            'marketplace-primary-v1', 'OmarchyGS Marketplace', 1, $1
        )
        "#,
    )
    .bind("d".repeat(64))
    .execute(pool)
    .await
    .expect("sync state should insert");
    let release_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO marketplace_releases (
            game_key, publisher_id, publisher_key, rules_version,
            cartridge_version, archive_sha256, signed_identity_sha256,
            display_name, release_path, reviewed_by, review_summary,
            signed_policy, policy_version, policy_status, policy_reason,
            compatible, imported, first_seen_snapshot_version,
            last_seen_snapshot_version
        )
        VALUES (
            'door-legends', 'ignibyte', $1, 1, 2, $2, $3,
            'Door Legends', 'releases/door-legends/2/', 'review-team',
            'Bounded first-party review passed.', $4, 1, 'deprecated',
            'Upgrade when practical.', TRUE, TRUE, 1, 1
        )
        RETURNING id
        "#,
    )
    .bind(json!({
        "format_version": 1,
        "algorithm": "ed25519",
        "key_id": "publisher-primary-v1",
        "publisher_id": "ignibyte",
        "verifying_key": "PUBLIC-KEY-MATERIAL"
    }))
    .bind(ARCHIVE)
    .bind(SIGNED_IDENTITY)
    .bind(json!({"version": 1}))
    .fetch_one(pool)
    .await
    .expect("reviewed release should insert");
    sqlx::query(
        r#"
        INSERT INTO server_cartridge_catalogs (
            game_key, active_release_id, admission_revision
        )
        VALUES ('door-legends', $1, 4)
        "#,
    )
    .bind(release_id)
    .execute(pool)
    .await
    .expect("catalog selection should insert");
}

async fn create_session(pool: &PgPool) -> String {
    accounts::register_account(
        pool,
        RegistrationInput {
            invite_code: accounts::create_test_invite(pool).await,
            username: "catalog_player".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await
    .expect("test account should register");
    match sessions::create_session(
        pool,
        CreateSessionInput {
            username: "catalog_player".to_owned(),
            password: "correct horse battery staple".to_owned(),
            device_name: "catalog api test".to_owned(),
        },
    )
    .await
    .expect("test session should create")
    {
        SessionCreation::Created(created) => created.token,
        SessionCreation::MfaRequired(_) => panic!("new test account should not require MFA"),
    }
}

struct TestResponse {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: String,
}

impl TestResponse {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("response should be JSON")
    }
}

async fn get(app: Router, token: Option<&str>) -> TestResponse {
    let mut request = Request::builder().uri("/v1/cartridges");
    if let Some(token) = token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .oneshot(request.body(Body::empty()).expect("request should build"))
        .await
        .expect("router should respond");
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should read")
        .to_bytes();
    TestResponse {
        status,
        headers,
        body: String::from_utf8(body.to_vec()).expect("body should be UTF-8"),
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
