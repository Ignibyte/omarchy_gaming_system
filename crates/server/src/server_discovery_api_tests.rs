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
                "games.cartridge-catalog.v1",
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

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn discovery_discloses_only_bounded_aggregate_custom_module_behavior(pool: PgPool) {
    let server_id: Uuid =
        sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton = TRUE")
            .fetch_one(&pool)
            .await
            .unwrap();
    let release_id = Uuid::new_v4();
    let admission_id = Uuid::new_v4();
    let instance_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO server_module_releases (
            release_id, module_id, publisher_id, version, release_format,
            signed_release, release_sha256, signed_provenance, provenance_sha256,
            provenance_class, review_id, component_sha256, wit_package, wit_world,
            wit_major, wit_sha256, requested_capabilities, subscribed_hooks,
            frame_bytes, memory_bytes, fuel, execution_ms, config_schema,
            state_schema, component_bytes, artifact_custody, publisher_key_id,
            publisher_public_key, publisher_key_sha256, provenance_key_id,
            provenance_public_key, provenance_key_sha256, provenance_server_id
        ) VALUES (
            $1, 'community.report-helper', 'community', '1.0.0',
            'omarchygs.server-module-release/v1', 'r', repeat('a', 64), 'p',
            repeat('b', 64), 'operator_custom', NULL, repeat('c', 64),
            'ignibyte:omarchygs-server-module@1.0.0', 'module-production', 1,
            repeat('d', 64), ARRAY['moderation_add_label']::TEXT[],
            ARRAY['persona_reported']::TEXT[], 65536, 4194304, 100000, 500,
            'community.report.config/v1', 'community.report.state/v1',
            decode('0061736d01000000', 'hex'), 'database_immutable',
            'community-publisher-v1', repeat('A', 43), repeat('e', 64),
            'community-operator-v1', repeat('B', 43), repeat('f', 64), $2
        )
        "#,
    )
    .bind(release_id)
    .bind(server_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO server_module_admissions (
            admission_id, lifecycle_revision, release_id, server_id,
            admission_format, signed_admission, admission_sha256, lifecycle,
            granted_capabilities, subscribed_hooks, config_revision,
            state_schema, state_revision
        ) VALUES ($1, 1, $2, $3, 'omarchygs.server-module-admission/v1',
                  'a', repeat('1', 64), 'active',
                  ARRAY['moderation_add_label']::TEXT[],
                  ARRAY['persona_reported']::TEXT[], 1,
                  'community.report.state/v1', 0)
        "#,
    )
    .bind(admission_id)
    .bind(release_id)
    .bind(server_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO server_module_instances (
            instance_id, module_id, release_id, current_admission_id,
            current_admission_revision, lifecycle, lifecycle_revision,
            config, config_revision, state_schema, state_revision
        ) VALUES ($1, 'community.report-helper', $2, $3, 1, 'active', 1,
                  '{}'::JSONB, 1, 'community.report.state/v1', 0)
        "#,
    )
    .bind(instance_id)
    .bind(release_id)
    .bind(admission_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        INSERT INTO server_module_state_namespaces (
            instance_id, state_schema, revision, entries, byte_size
        ) VALUES ($1, 'community.report.state/v1', 0, '{}'::JSONB, 2)
        "#,
    )
    .bind(instance_id)
    .execute(&pool)
    .await
    .unwrap();

    let (_, _, document) = get_discovery(router_with_server_name(
        pool,
        MfaCipher::test_cipher(),
        Arc::from("Custom Community"),
    ))
    .await;
    assert_eq!(document["server_id"], server_id.to_string());
    assert!(
        document["capabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("server.operator-custom-modules.v1"))
    );
    assert_eq!(
        document["operator_custom_modules"],
        json!({
            "format": "omarchygs.operator-custom-modules-disclosure/v1",
            "server_id": server_id.to_string(),
            "active_count": 1,
            "behavior_capabilities": ["moderation_labels"],
            "warning": "This server runs operator-custom code not reviewed or supported by OmarchyGS.",
            "support_boundary": "Security, privacy, availability, and support are the server operator's responsibility."
        })
    );
    let serialized = document.to_string();
    let release_id_text = release_id.to_string();
    for private in [
        "community.report-helper",
        release_id_text.as_str(),
        "component_bytes",
        "publisher_public_key",
        "provenance_public_key",
        "signed_release",
    ] {
        assert!(!serialized.contains(private));
    }
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
