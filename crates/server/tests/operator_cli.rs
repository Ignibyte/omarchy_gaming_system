use std::{
    collections::BTreeSet,
    os::unix::fs::{PermissionsExt as _, symlink},
    process::Command,
};

use serde_json::{Value, json};
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

use omarchygs_game_cartridge::generate_catalog_keypair;

const CATALOG_ARCHIVE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CATALOG_IDENTITY: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

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

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn local_operator_cli_issues_lists_replays_and_revokes_registration_invites(pool: PgPool) {
    let database_url = isolated_database_url(&pool).await;
    let temp = TempDir::new().expect("private invite command directory should create");
    let operation_id = Uuid::new_v4();
    let issue_path = temp.path().join("issue-invite.json");
    write_private_json(
        &issue_path,
        &json!({
            "command": "issue_registration_invite",
            "idempotency_key": operation_id,
            "label": "CLI alpha tester",
            "valid_for_hours": 72,
            "actor": "cli-smoke-sysop",
            "reason": "Admit one CLI alpha fixture"
        }),
    );
    let issued = run_admin(
        &database_url,
        &["apply", issue_path.to_str().expect("path is UTF-8")],
    );
    assert!(
        issued.status.success(),
        "invite issue failed: {}",
        String::from_utf8_lossy(&issued.stderr)
    );
    assert!(issued.stderr.is_empty());
    let issue_receipt: Value =
        serde_json::from_slice(&issued.stdout).expect("issue receipt should be JSON");
    exact_keys(
        &issue_receipt,
        &[
            "action",
            "created_at",
            "expires_at",
            "first_delivery",
            "id",
            "invite_code",
            "label",
            "operation_id",
            "previous_state",
            "resulting_state",
            "target_id",
            "target_kind",
        ],
    );
    assert_eq!(issue_receipt["target_kind"], "registration_invite");
    assert_eq!(issue_receipt["first_delivery"], true);
    let invite_id = Uuid::parse_str(
        issue_receipt["target_id"]
            .as_str()
            .expect("invite ID should be a string"),
    )
    .expect("invite ID should be a UUID");
    let invite_code = issue_receipt["invite_code"]
        .as_str()
        .expect("first issue should include code")
        .to_owned();
    assert!(invite_code.starts_with("ogsi_"));
    assert_eq!(invite_code.len(), 48);

    let replayed = run_admin(
        &database_url,
        &["apply", issue_path.to_str().expect("path is UTF-8")],
    );
    assert!(replayed.status.success());
    let replay: Value = serde_json::from_slice(&replayed.stdout).expect("replay should be JSON");
    exact_keys(
        &replay,
        &[
            "action",
            "created_at",
            "expires_at",
            "first_delivery",
            "id",
            "label",
            "operation_id",
            "previous_state",
            "resulting_state",
            "target_id",
            "target_kind",
        ],
    );
    assert_eq!(replay["id"], issue_receipt["id"]);
    assert_eq!(replay["first_delivery"], false);

    let listed = run_admin(&database_url, &["invites", "issued", "10"]);
    assert!(listed.status.success());
    assert!(listed.stderr.is_empty());
    let inventory: Value =
        serde_json::from_slice(&listed.stdout).expect("invite inventory should be JSON");
    exact_keys(&inventory, &["invitations"]);
    assert_eq!(inventory["invitations"].as_array().map(Vec::len), Some(1));
    exact_keys(
        &inventory["invitations"][0],
        &[
            "created_at",
            "expires_at",
            "id",
            "label",
            "redeemed_username",
            "revoked_at",
            "state",
            "used_at",
        ],
    );
    assert_eq!(inventory["invitations"][0]["id"], invite_id.to_string());
    assert_eq!(inventory["invitations"][0]["state"], "issued");
    let inventory_text = String::from_utf8(listed.stdout).expect("inventory should be UTF-8");
    for forbidden in [
        invite_code.as_str(),
        "code_hash",
        "issued_reason",
        "password_hash",
        "account_sessions",
    ] {
        assert!(!inventory_text.contains(forbidden));
    }

    let revoke_path = temp.path().join("revoke-invite.json");
    write_private_json(
        &revoke_path,
        &json!({
            "command": "revoke_registration_invite",
            "idempotency_key": Uuid::new_v4(),
            "invite_id": invite_id,
            "actor": "cli-smoke-sysop",
            "reason": "Invitation delivery was canceled"
        }),
    );
    let revoked = run_admin(
        &database_url,
        &["apply", revoke_path.to_str().expect("path is UTF-8")],
    );
    assert!(revoked.status.success());
    let receipt: Value =
        serde_json::from_slice(&revoked.stdout).expect("revoke receipt should be JSON");
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
    assert_eq!(receipt["previous_state"], "issued");
    assert_eq!(receipt["resulting_state"], "revoked");
    let revoked_list = run_admin(&database_url, &["invites", "revoked", "10"]);
    assert!(revoked_list.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&revoked_list.stdout)
            .expect("revoked inventory should be JSON")["invitations"][0]["state"],
        "revoked"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn local_operator_cli_lists_public_catalog_facts_and_deactivates_exact_release(pool: PgPool) {
    let database_url = isolated_database_url(&pool).await;
    let temp = TempDir::new().expect("catalog CLI directory should create");
    let store_root = temp.path().join("store");
    std::fs::create_dir(&store_root).expect("store root should create");
    std::fs::set_permissions(&store_root, std::fs::Permissions::from_mode(0o700))
        .expect("store permissions should restrict");
    let (_, marketplace_public) =
        generate_catalog_keypair("marketplace-primary-v1", "omarchygs-marketplace")
            .expect("marketplace key should generate");
    let key_path = temp.path().join("marketplace-public.json");
    write_private_json(
        &key_path,
        &serde_json::to_value(&marketplace_public).expect("key should serialize"),
    );

    sqlx::query(
        r#"
        INSERT INTO marketplace_sync_state (
            marketplace_origin, authority_id, key_id, marketplace_name,
            snapshot_version, snapshot_sha256, signed_snapshot, marketplace_key
        ) VALUES (
            'https://market.example.test/v1/', $1, $2,
            'OmarchyGS Marketplace', 1, $3, $4, $5
        )
        "#,
    )
    .bind(&marketplace_public.authority_id)
    .bind(&marketplace_public.key_id)
    .bind("c".repeat(64))
    .bind(vec![1_u8])
    .bind(serde_json::to_value(&marketplace_public).expect("key should serialize"))
    .execute(&pool)
    .await
    .expect("sync state should seed");
    let release_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO marketplace_releases (
            game_key, publisher_id, publisher_key, rules_version,
            cartridge_version, archive_sha256, signed_identity_sha256,
            display_name, release_path, reviewed_by, review_summary,
            signed_policy, policy_marketplace_key, policy_snapshot_version,
            policy_version, policy_status, policy_reason,
            compatible, imported, first_seen_snapshot_version,
            last_seen_snapshot_version
        ) VALUES (
            'door-legends', 'ignibyte', $1, 1, 2, $2, $3,
            'Door Legends', 'releases/door-legends/2/', 'review-team',
            'Bounded review passed.', $4, $5, 1, 1, 'active', 'Current release.',
            TRUE, TRUE, 1, 1
        ) RETURNING id
        "#,
    )
    .bind(json!({"key_id": "publisher-primary-v1"}))
    .bind(CATALOG_ARCHIVE)
    .bind(CATALOG_IDENTITY)
    .bind(json!({"policy": "public-but-not-returned"}))
    .bind(serde_json::to_value(&marketplace_public).expect("key should serialize"))
    .fetch_one(&pool)
    .await
    .expect("release should seed");
    sqlx::query(
        "INSERT INTO server_cartridge_catalogs (game_key, active_release_id, admission_revision) VALUES ('door-legends', $1, 1)",
    )
    .bind(release_id)
    .execute(&pool)
    .await
    .expect("catalog should seed");

    let listed = run_catalog_admin(&database_url, &["cartridges"], &key_path, &store_root);
    assert!(
        listed.status.success(),
        "catalog inventory failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    assert!(listed.stderr.is_empty());
    let inventory: Value =
        serde_json::from_slice(&listed.stdout).expect("catalog inventory should be JSON");
    exact_keys(&inventory, &["releases", "snapshot"]);
    exact_keys(
        &inventory["snapshot"],
        &[
            "key_id",
            "marketplace_id",
            "marketplace_name",
            "marketplace_origin",
            "snapshot_sha256",
            "snapshot_version",
            "synchronized_at",
        ],
    );
    assert_eq!(inventory["releases"].as_array().map(Vec::len), Some(1));
    exact_keys(
        &inventory["releases"][0],
        &[
            "admission_revision",
            "archive_sha256",
            "cartridge_version",
            "compatible",
            "display_name",
            "effective",
            "game_key",
            "imported",
            "policy_reason",
            "policy_status",
            "policy_version",
            "present",
            "publisher_id",
            "publisher_key_id",
            "review_summary",
            "reviewed_by",
            "rules_version",
            "selected",
            "signed_identity_sha256",
        ],
    );
    assert_eq!(inventory["releases"][0]["effective"], true);
    let inventory_text = String::from_utf8(listed.stdout).expect("inventory should be UTF-8");
    for forbidden in [
        "release_path",
        "signed_policy",
        "verifying_key",
        "OGS_MARKETPLACE_PUBLIC_KEY",
        store_root.to_str().expect("store path is UTF-8"),
    ] {
        assert!(!inventory_text.contains(forbidden), "leaked {forbidden}");
    }

    let operation_id = Uuid::new_v4();
    let command_path = temp.path().join("deactivate.json");
    write_private_json(
        &command_path,
        &json!({
            "idempotency_key": operation_id,
            "game_key": "door-legends",
            "expected": {"state": "release", "archive_sha256": CATALOG_ARCHIVE},
            "desired": {"state": "inactive"},
            "actor": "cli-sysop",
            "reason": "Temporarily remove this cartridge"
        }),
    );
    let applied = run_catalog_admin(
        &database_url,
        &[
            "catalog-apply",
            command_path.to_str().expect("command path is UTF-8"),
        ],
        &key_path,
        &store_root,
    );
    assert!(
        applied.status.success(),
        "catalog apply failed: {}",
        String::from_utf8_lossy(&applied.stderr)
    );
    let receipt: Value =
        serde_json::from_slice(&applied.stdout).expect("catalog receipt should be JSON");
    exact_keys(
        &receipt,
        &[
            "action",
            "admission_revision",
            "created_at",
            "game_key",
            "id",
            "operation_id",
            "previous_archive_sha256",
            "previous_provenance_class",
            "resulting_archive_sha256",
        ],
    );
    assert_eq!(receipt["operation_id"], operation_id.to_string());
    assert_eq!(receipt["action"], "deactivate_cartridge");
    assert_eq!(receipt["previous_archive_sha256"], CATALOG_ARCHIVE);
    assert!(receipt["resulting_archive_sha256"].is_null());

    let replay = run_catalog_admin(
        &database_url,
        &[
            "catalog-apply",
            command_path.to_str().expect("command path is UTF-8"),
        ],
        &key_path,
        &store_root,
    );
    assert!(replay.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&replay.stdout).unwrap()["id"],
        receipt["id"]
    );

    let link_path = temp.path().join("catalog-link.json");
    symlink(&command_path, &link_path).expect("catalog command symlink should create");
    let rejected = run_catalog_admin(
        &database_url,
        &[
            "catalog-apply",
            link_path.to_str().expect("link path is UTF-8"),
        ],
        &key_path,
        &store_root,
    );
    assert!(!rejected.status.success());
    assert_eq!(rejected.stderr, b"operator_invalid_input\n");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn custom_cartridge_cli_requires_matching_owner_private_signing_key(pool: PgPool) {
    let database_url = isolated_database_url(&pool).await;
    let temp = TempDir::new().expect("custom CLI directory should create");
    let store_root = temp.path().join("store");
    std::fs::create_dir(&store_root).expect("store root should create");
    std::fs::set_permissions(&store_root, std::fs::Permissions::from_mode(0o700))
        .expect("store permissions should restrict");
    let (private_key, public_key) =
        generate_catalog_keypair("custom-primary-v1", "test-community").expect("custom key");
    let (_, substituted_public) =
        generate_catalog_keypair("custom-primary-v2", "test-community").expect("other key");
    let private_path = temp.path().join("custom.private.json");
    let public_path = temp.path().join("custom.public.json");
    let substituted_path = temp.path().join("substituted.public.json");
    write_private_json(
        &private_path,
        &serde_json::to_value(&private_key).expect("private key should serialize"),
    );
    write_private_json(
        &public_path,
        &serde_json::to_value(&public_key).expect("public key should serialize"),
    );
    write_private_json(
        &substituted_path,
        &serde_json::to_value(&substituted_public).expect("other key should serialize"),
    );
    let command_path = temp.path().join("import.json");
    write_private_json(
        &command_path,
        &json!({
            "idempotency_key": Uuid::new_v4(),
            "release_directory": temp.path().join("missing-release"),
            "publisher_public_key_file": temp.path().join("missing-publisher.json"),
            "policy_version": 1,
            "lifecycle_status": "active",
            "actor": "cli-sysop",
            "reason": "Prove signing key configuration fails before import",
            "acknowledge_marketplace_warning": true
        }),
    );

    let mismatched = run_custom_admin(
        &database_url,
        &command_path,
        &private_path,
        &substituted_path,
        &store_root,
    );
    assert!(!mismatched.status.success());
    assert!(mismatched.stdout.is_empty());
    assert_eq!(mismatched.stderr, b"operator_custom_invalid_config\n");

    std::fs::set_permissions(&private_path, std::fs::Permissions::from_mode(0o644))
        .expect("private key mode should change");
    let public_private = run_custom_admin(
        &database_url,
        &command_path,
        &private_path,
        &public_path,
        &store_root,
    );
    assert!(!public_private.status.success());
    assert!(public_private.stdout.is_empty());
    assert_eq!(public_private.stderr, b"operator_custom_invalid_config\n");
    let authority_count: i64 = sqlx::query_scalar("SELECT count(*) FROM operator_custom_authority")
        .fetch_one(&pool)
        .await
        .expect("authority count should query");
    assert_eq!(authority_count, 0);
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

fn write_private_json(path: &std::path::Path, document: &Value) {
    std::fs::write(
        path,
        serde_json::to_vec(document).expect("command should serialize"),
    )
    .expect("command should write");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .expect("command permissions should restrict");
}

fn run_admin(database_url: &url::Url, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_omarchygs-admin"))
        .args(arguments)
        .env("DATABASE_URL", database_url.as_str())
        .output()
        .expect("operator CLI should execute")
}

fn run_catalog_admin(
    database_url: &url::Url,
    arguments: &[&str],
    public_key: &std::path::Path,
    store_root: &std::path::Path,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_omarchygs-admin"))
        .args(arguments)
        .env("DATABASE_URL", database_url.as_str())
        .env("OGS_MARKETPLACE_PUBLIC_KEY", public_key)
        .env("OGS_CARTRIDGE_STORE_ROOT", store_root)
        .output()
        .expect("catalog CLI should execute")
}

fn run_custom_admin(
    database_url: &url::Url,
    command_path: &std::path::Path,
    private_key: &std::path::Path,
    public_key: &std::path::Path,
    store_root: &std::path::Path,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_omarchygs-admin"))
        .args([
            "custom-cartridge-import",
            command_path.to_str().expect("UTF-8 path"),
        ])
        .env("DATABASE_URL", database_url.as_str())
        .env(
            "OGS_CUSTOM_CARTRIDGE_OPERATOR_NAME",
            "Test Community Operator",
        )
        .env("OGS_CUSTOM_CARTRIDGE_PRIVATE_KEY", private_key)
        .env("OGS_CUSTOM_CARTRIDGE_PUBLIC_KEY", public_key)
        .env("OGS_CARTRIDGE_STORE_ROOT", store_root)
        .output()
        .expect("custom CLI should execute")
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
