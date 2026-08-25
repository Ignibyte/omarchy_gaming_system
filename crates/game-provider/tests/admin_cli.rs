use std::{os::unix::fs::PermissionsExt as _, process::Command};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::SigningKey;
use serde_json::{Value, json};
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-provider-conformance.sh"]
async fn operator_cli_applies_one_bounded_command_and_emits_only_a_safe_receipt(pool: PgPool) {
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("test database name should resolve");
    let mut database_url = url::Url::parse(
        &std::env::var("DATABASE_URL")
            .expect("DATABASE_URL should be supplied by canonical script"),
    )
    .expect("DATABASE_URL should parse");
    database_url.set_path(&format!("/{database_name}"));
    let now: i64 = sqlx::query_scalar("SELECT EXTRACT(EPOCH FROM clock_timestamp())::BIGINT")
        .fetch_one(&pool)
        .await
        .expect("database clock should read");
    let release_id = Uuid::new_v4();
    let signing_key = SigningKey::from_bytes(&[51; 32]);
    let command = json!({
        "command": "register_release",
        "actor": "cli-smoke-operator",
        "reason": "prove the bounded operator adapter",
        "registration": {
            "provider_id": "cli-fixture-provider",
            "display_name": "CLI Fixture Provider",
            "release_id": release_id,
            "game_key": "signal_siege",
            "rules_version": 1,
            "cartridge_digest": "e".repeat(64),
            "endpoint": {
                "host": "provider.example.test",
                "port": 443,
                "base_path": "/omarchygs/provider/v1/"
            },
            "active_session_policy": "continue",
            "scopes": ["game.launch", "game.command", "game.reconcile", "game.event"],
            "message_keys": [{
                "key_id": "provider-message-1",
                "public_material_base64": STANDARD.encode(signing_key.verifying_key().as_bytes()),
                "valid_from": now - 60,
                "valid_until": null
            }],
            "tls_roots": [{
                "key_id": "provider-tls-1",
                "public_material_base64": STANDARD.encode([0x30_u8; 128]),
                "valid_from": now - 60,
                "valid_until": null
            }],
            "quotas": {
                "grants_per_minute": 10,
                "requests_per_minute": 10,
                "callbacks_per_minute": 10,
                "max_concurrent_requests": 2,
                "request_body_bytes": 8192,
                "response_body_bytes": 8192,
                "connect_timeout_ms": 500,
                "total_timeout_ms": 2000
            }
        }
    });
    let temp = TempDir::new().expect("private command directory should create");
    let command_path = temp.path().join("command.json");
    std::fs::write(
        &command_path,
        serde_json::to_vec(&command).expect("operator command should serialize"),
    )
    .expect("operator command should write");
    std::fs::set_permissions(&command_path, std::fs::Permissions::from_mode(0o600))
        .expect("operator command permissions should restrict");
    let output = Command::new(env!("CARGO_BIN_EXE_omarchygs-provider-admin"))
        .arg("apply")
        .arg(&command_path)
        .env("DATABASE_URL", database_url.as_str())
        .output()
        .expect("operator adapter should execute");
    assert!(
        output.status.success(),
        "operator adapter failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value =
        serde_json::from_slice(&output.stdout).expect("operator receipt should parse");
    assert_eq!(receipt["command"], "register_release");
    assert_eq!(receipt["provider_id"], "cli-fixture-provider");
    assert_eq!(receipt["release_id"], release_id.to_string());
    let receipt_text = receipt.to_string();
    assert!(!receipt_text.contains("public_material_base64"));
    assert!(!receipt_text.contains("DATABASE_URL"));
    assert!(!receipt_text.contains(&STANDARD.encode([51_u8; 32])));
}
