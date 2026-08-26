use std::{
    collections::BTreeSet,
    os::unix::fs::{PermissionsExt as _, symlink},
    process::Command,
};

use serde_json::{Value, json};
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn local_operator_cli_lists_and_dispositions_reports_without_secret_output(pool: PgPool) {
    let database_url = isolated_database_url(&pool).await;
    let reporter_account = seed_account(&pool, "cli_reporter").await;
    let subject_account = seed_account(&pool, "cli_subject").await;
    let reporter = seed_persona(&pool, reporter_account, "cli_reporter", "CLI Reporter").await;
    let subject = seed_persona(&pool, subject_account, "cli_subject", "CLI Subject").await;
    let report_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO persona_reports (
            reporter_persona_id, subject_persona_id, idempotency_key, category, detail
        )
        VALUES ($1, $2, gen_random_uuid(), 'harassment', 'Bounded operator CLI fixture')
        RETURNING id
        "#,
    )
    .bind(reporter)
    .bind(subject)
    .fetch_one(&pool)
    .await
    .expect("report should seed");

    let listed = Command::new(env!("CARGO_BIN_EXE_omarchygs-admin"))
        .args(["reports", "open", "10"])
        .env("DATABASE_URL", database_url.as_str())
        .output()
        .expect("operator report inventory should execute");
    assert!(
        listed.status.success(),
        "inventory failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    assert!(listed.stderr.is_empty());
    let inventory: Value =
        serde_json::from_slice(&listed.stdout).expect("inventory should be JSON");
    exact_keys(&inventory, &["reports"]);
    let reports = inventory["reports"]
        .as_array()
        .expect("reports should be an array");
    assert_eq!(reports.len(), 1);
    exact_keys(
        &reports[0],
        &[
            "category",
            "closed_at",
            "created_at",
            "detail",
            "id",
            "reporter",
            "status",
            "subject",
            "subject_account_id",
            "updated_at",
        ],
    );
    exact_keys(
        &reports[0]["reporter"],
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
    exact_keys(
        &reports[0]["subject"],
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
    assert_eq!(
        reports[0]["subject_account_id"],
        subject_account.to_string()
    );
    let listed_text = String::from_utf8(listed.stdout).expect("inventory should be UTF-8");
    for secret in [
        "password_hash",
        "token_hash",
        "account_sessions",
        "DATABASE_URL",
        "test-only-password-hash",
    ] {
        assert!(!listed_text.contains(secret));
    }

    let temp = TempDir::new().expect("private command directory should create");
    let operation_id = Uuid::new_v4();
    let command_path = temp.path().join("resolve.json");
    std::fs::write(
        &command_path,
        serde_json::to_vec(&json!({
            "command": "set_report_status",
            "idempotency_key": operation_id,
            "report_id": report_id,
            "status": "resolved",
            "actor": "cli-smoke-sysop",
            "reason": "Resolve the CLI smoke report"
        }))
        .expect("command should serialize"),
    )
    .expect("command should write");
    std::fs::set_permissions(&command_path, std::fs::Permissions::from_mode(0o600))
        .expect("command permissions should restrict");
    let applied = Command::new(env!("CARGO_BIN_EXE_omarchygs-admin"))
        .arg("apply")
        .arg(&command_path)
        .env("DATABASE_URL", database_url.as_str())
        .output()
        .expect("operator action should execute");
    assert!(
        applied.status.success(),
        "action failed: {}",
        String::from_utf8_lossy(&applied.stderr)
    );
    let receipt: Value = serde_json::from_slice(&applied.stdout).expect("receipt should be JSON");
    exact_keys(
        &receipt,
        &[
            "action",
            "created_at",
            "id",
            "operation_id",
            "previous_state",
            "resulting_state",
            "target_id",
            "target_kind",
        ],
    );
    assert_eq!(receipt["operation_id"], operation_id.to_string());
    assert_eq!(receipt["target_id"], report_id.to_string());
    assert_eq!(receipt["previous_state"], "open");
    assert_eq!(receipt["resulting_state"], "resolved");

    let link_path = temp.path().join("symlink.json");
    symlink(&command_path, &link_path).expect("test symlink should create");
    let rejected = Command::new(env!("CARGO_BIN_EXE_omarchygs-admin"))
        .arg("apply")
        .arg(&link_path)
        .env("DATABASE_URL", database_url.as_str())
        .output()
        .expect("symlink rejection should execute");
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    assert_eq!(rejected.stderr, b"operator_invalid_input\n");
}

fn exact_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<&str> = value
        .as_object()
        .expect("value should be an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(actual, expected.iter().copied().collect());
}

async fn isolated_database_url(pool: &PgPool) -> url::Url {
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .expect("test database name should resolve");
    let mut database_url = url::Url::parse(
        &std::env::var("DATABASE_URL").expect("DATABASE_URL should be supplied by test script"),
    )
    .expect("DATABASE_URL should parse");
    database_url.set_path(&format!("/{database_name}"));
    database_url
}

async fn seed_account(pool: &PgPool, username: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO accounts (username, password_hash) VALUES ($1, 'test-only-password-hash') RETURNING id",
    )
    .bind(username)
    .fetch_one(pool)
    .await
    .expect("account should seed")
}

async fn seed_persona(pool: &PgPool, account_id: Uuid, handle: &str, display_name: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO personas (account_id, handle, display_name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(account_id)
    .bind(handle)
    .bind(display_name)
    .fetch_one(pool)
    .await
    .expect("persona should seed")
}
