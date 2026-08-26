use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CACHE_CONTROL},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{app::router_with_server_name, mfa::MfaCipher};

async fn get_discovery(app: axum::Router) -> (StatusCode, String, Value) {
    let response = app
        .oneshot(
            Request::get("/.well-known/omarchygs")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    let status = response.status();
    let cache_control = response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should be readable")
        .to_bytes();
    let document = serde_json::from_slice(&body).expect("body should be JSON");
    (status, cache_control, document)
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn discovery_is_exact_stable_public_and_immutable(pool: PgPool) {
    let persisted_id =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM server_identity WHERE singleton = TRUE")
            .fetch_one(&pool)
            .await
            .expect("migration should create the identity");
    let count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM server_identity")
        .fetch_one(&pool)
        .await
        .expect("identity count should be readable");
    assert_eq!(count, 1);

    let (status, cache_control, first) = get_discovery(router_with_server_name(
        pool.clone(),
        MfaCipher::test_cipher(),
        Arc::from("Arcade Friends"),
    ))
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cache_control, "no-store");
    assert_eq!(
        first,
        json!({
            "service": "omarchy-gaming-system",
            "server_id": persisted_id.to_string(),
            "server_name": "Arcade Friends",
            "protocol_version": 1,
            "capabilities": [
                "accounts.invite-registration.v1",
                "auth.device-sessions.v1",
                "auth.totp.v1",
                "games.challenges.v1",
                "games.sessions.v1",
                "identity.personas.v1",
                "social.connections.v1",
                "social.private-inbox.v1",
                "social.reporting.v1",
                "sync.cursor.v1",
                "sync.websocket-hints.v1"
            ]
        })
    );

    let (_, _, renamed) = get_discovery(router_with_server_name(
        pool.clone(),
        MfaCipher::test_cipher(),
        Arc::from("Renamed Community"),
    ))
    .await;
    assert_eq!(renamed["server_id"], first["server_id"]);
    assert_eq!(renamed["server_name"], "Renamed Community");

    assert!(
        sqlx::query("UPDATE server_identity SET id = gen_random_uuid()")
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("DELETE FROM server_identity")
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("TRUNCATE server_identity")
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("INSERT INTO server_identity DEFAULT VALUES")
            .execute(&pool)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn discovery_fails_closed_when_identity_storage_is_unavailable() {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(100))
        .connect_lazy("postgres://test:test@127.0.0.1:1/unavailable")
        .expect("unavailable test URL should parse");
    let app = router_with_server_name(pool, MfaCipher::test_cipher(), Arc::from("Arcade Friends"));

    let (status, cache_control, document) = get_discovery(app).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(cache_control, "no-store");
    assert_eq!(
        document,
        json!({
            "error": {
                "code": "server_discovery_unavailable",
                "message": "server identity is temporarily unavailable"
            }
        })
    );
}
