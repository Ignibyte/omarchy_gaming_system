//! PostgreSQL tests for operator-custom server-module custody and lifecycle.

use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{SigningKey, VerifyingKey};
use omarchygs_server_module_runtime::{
    Capability, FixtureKind, HookKind, HostRequest, HostResponse, MAX_EXECUTION_MS,
    MAX_FRAME_BYTES, MAX_FUEL, MAX_LINEAR_MEMORY_BYTES, ModuleReleaseManifest, ModuleRuntime,
    RELEASE_FORMAT, ResourceBudgets, ReviewedRelease, SignedEnvelope, WIT_PACKAGE, WIT_WORLD,
    WitIdentity, canonical_json, encode_verifying_key, sha256_hex, verifying_key_sha256,
    wit_sha256,
};
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

use crate::{
    server_module_custom::{
        CustomModuleAdminConfig, CustomModuleImportCommand, CustomModuleLifecycleAction,
        CustomModuleLifecycleCommand, LocalReleaseProbe, ModulePrivateKeyDocument,
        ModulePublicKeyDocument, UNREVIEWED_ACKNOWLEDGEMENT, apply_custom_lifecycle_with_probe,
        import_custom_module_with_probe,
    },
    server_modules::{
        ModuleConfig, ModuleError, ModuleExecutor, ServerModuleService, prepare_restored_modules,
    },
};

struct CustomFixture {
    _directory: TempDir,
    command: CustomModuleImportCommand,
}

struct LocalExecutor {
    runtime: ModuleRuntime,
}

impl ModuleExecutor for LocalExecutor {
    fn execute(
        &self,
        request: HostRequest,
        core_key: VerifyingKey,
        release: ReviewedRelease,
    ) -> Result<HostResponse, ModuleError> {
        Ok(self.runtime.execute_release(&request, &core_key, &release))
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn custom_import_is_signed_idempotent_private_and_immutable(pool: PgPool) {
    let server_id = server_id(&pool).await;
    let fixture = custom_fixture(
        server_id,
        "community.report-helper",
        Uuid::new_v4(),
        "1.0.0",
        "community.report-helper.state/v1",
        BTreeMap::from([("review_count".into(), "1".into())]),
    );
    let config = CustomModuleAdminConfig::for_test([11_u8; 32]);
    let probe = LocalReleaseProbe;

    let imported = import_custom_module_with_probe(&pool, &config, &fixture.command, &probe)
        .await
        .expect("valid custom release should import");
    assert_eq!(imported.module_id, "community.report-helper");
    assert_eq!(imported.lifecycle, "disabled");
    assert_eq!(imported.lifecycle_revision, 1);
    assert!(!imported.replayed);

    let replay = import_custom_module_with_probe(&pool, &config, &fixture.command, &probe)
        .await
        .expect("exact import replay should succeed");
    assert!(replay.replayed);
    assert_eq!(replay.instance_id, imported.instance_id);
    assert_eq!(replay.release_id, imported.release_id);

    let changed_replay = CustomModuleImportCommand {
        reason: "Changed replay body".into(),
        ..fixture.command.clone()
    };
    assert_eq!(
        import_custom_module_with_probe(&pool, &config, &changed_replay, &probe).await,
        Err(ModuleError::Conflict)
    );

    let stored: (String, Option<Uuid>, Option<Uuid>, String, String, bool) = sqlx::query_as(
        r#"
        SELECT provenance_class, review_id, provenance_server_id,
               artifact_custody, publisher_key_sha256,
               component_bytes = $2
        FROM server_module_releases
        WHERE release_id = $1
        "#,
    )
    .bind(imported.release_id)
    .bind(FixtureKind::Valid.component_bytes())
    .fetch_one(&pool)
    .await
    .expect("immutable release evidence should load");
    assert_eq!(stored.0, "operator_custom");
    assert_eq!(stored.1, None);
    assert_eq!(stored.2, Some(server_id));
    assert_eq!(stored.3, "database_immutable");
    assert_eq!(stored.4, fixture.command.publisher_key_sha256);
    assert!(stored.5);

    let namespace: (serde_json::Value, i32, i32) = sqlx::query_as(
        r#"
        SELECT entries, byte_size, octet_length(entries::TEXT)
        FROM server_module_state_namespaces
        WHERE instance_id = $1
        "#,
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("initial namespace should load");
    assert_eq!(namespace.0["review_count"], "1");
    assert_eq!(namespace.1, namespace.2);

    let immutable =
        sqlx::query("UPDATE server_module_releases SET version = '9.9.9' WHERE release_id = $1")
            .bind(imported.release_id)
            .execute(&pool)
            .await;
    assert!(immutable.is_err(), "release evidence must reject mutation");

    let same_release = CustomModuleImportCommand {
        operation_id: Uuid::new_v4(),
        reason: "Reconfirm exact immutable release".into(),
        ..fixture.command.clone()
    };
    let confirmed = import_custom_module_with_probe(&pool, &config, &same_release, &probe)
        .await
        .expect("same release and grant should be idempotently reconfirmable");
    assert_eq!(confirmed.instance_id, imported.instance_id);
    assert_eq!(confirmed.release_id, imported.release_id);

    let changed_grant = CustomModuleImportCommand {
        operation_id: Uuid::new_v4(),
        granted_capabilities: Vec::new(),
        reason: "Attempt to replace immutable grant review".into(),
        ..fixture.command.clone()
    };
    assert_eq!(
        import_custom_module_with_probe(&pool, &config, &changed_grant, &probe).await,
        Err(ModuleError::Conflict)
    );

    let wrong_fingerprint = CustomModuleImportCommand {
        operation_id: Uuid::new_v4(),
        publisher_key_sha256: "0".repeat(64),
        reason: "Reject an unconfirmed publisher key".into(),
        ..fixture.command.clone()
    };
    assert_eq!(
        import_custom_module_with_probe(&pool, &config, &wrong_fingerprint, &probe).await,
        Err(ModuleError::Denied)
    );

    fs::set_permissions(
        &fixture.command.component_path,
        fs::Permissions::from_mode(0o644),
    )
    .expect("fixture mode should change");
    let public_component = CustomModuleImportCommand {
        operation_id: Uuid::new_v4(),
        reason: "Reject a non-private component path".into(),
        ..fixture.command.clone()
    };
    assert_eq!(
        import_custom_module_with_probe(&pool, &config, &public_component, &probe).await,
        Err(ModuleError::InvalidInput)
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn custom_lifecycle_upgrades_rolls_back_once_and_removes_terminally(pool: PgPool) {
    let server_id = server_id(&pool).await;
    let config = CustomModuleAdminConfig::for_test([11_u8; 32]);
    let probe = LocalReleaseProbe;
    let first = custom_fixture(
        server_id,
        "community.lifecycle-helper",
        Uuid::new_v4(),
        "1.0.0",
        "community.lifecycle-helper.state/v1",
        BTreeMap::from([("counter".into(), "1".into())]),
    );
    let imported = import_custom_module_with_probe(&pool, &config, &first.command, &probe)
        .await
        .expect("first release should import");

    let enabled = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Enable,
            imported.instance_id,
            1,
            0,
            None,
            None,
        ),
    )
    .await;
    assert_eq!(enabled.lifecycle, "active");
    assert_eq!(enabled.lifecycle_revision, 2);

    let suspended = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Suspend,
            imported.instance_id,
            2,
            0,
            None,
            None,
        ),
    )
    .await;
    assert_eq!(suspended.lifecycle, "suspended");
    let recovered = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Recover,
            imported.instance_id,
            3,
            0,
            None,
            None,
        ),
    )
    .await;
    assert_eq!(recovered.lifecycle, "active");
    let disabled = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Disable,
            imported.instance_id,
            4,
            0,
            None,
            None,
        ),
    )
    .await;
    assert_eq!(disabled.lifecycle, "disabled");

    let second = custom_fixture(
        server_id,
        "community.lifecycle-helper",
        Uuid::new_v4(),
        "2.0.0",
        "community.lifecycle-helper.state/v2",
        BTreeMap::new(),
    );
    let staged = import_custom_module_with_probe(&pool, &config, &second.command, &probe)
        .await
        .expect("second release should stage without selection");
    assert_eq!(staged.instance_id, imported.instance_id);
    assert_eq!(staged.lifecycle, "disabled");

    let upgraded = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Upgrade,
            imported.instance_id,
            5,
            0,
            Some(staged.release_id),
            Some(BTreeMap::from([("counter".into(), "2".into())])),
        ),
    )
    .await;
    assert_eq!(upgraded.release_id, staged.release_id);
    assert_eq!(upgraded.lifecycle, "active");
    assert_eq!(upgraded.state_revision, 1);
    let upgraded_root: (
        Uuid,
        Option<Uuid>,
        Option<Uuid>,
        String,
        serde_json::Value,
        i32,
        i32,
    ) = sqlx::query_as(
        r#"
            SELECT i.release_id, i.previous_release_id, i.rollback_snapshot_id,
                   i.state_schema, n.entries, n.byte_size,
                   octet_length(n.entries::TEXT)
            FROM server_module_instances i
            JOIN server_module_state_namespaces n ON n.instance_id = i.instance_id
            WHERE i.instance_id = $1
            "#,
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("upgraded root should load");
    assert_eq!(upgraded_root.0, staged.release_id);
    assert_eq!(upgraded_root.1, Some(imported.release_id));
    assert!(upgraded_root.2.is_some());
    assert_eq!(upgraded_root.3, "community.lifecycle-helper.state/v2");
    assert_eq!(upgraded_root.4["counter"], "2");
    assert_eq!(upgraded_root.5, upgraded_root.6);

    let rolled_back = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Rollback,
            imported.instance_id,
            6,
            1,
            None,
            None,
        ),
    )
    .await;
    assert_eq!(rolled_back.release_id, imported.release_id);
    assert_eq!(rolled_back.state_revision, 2);
    let rollback_root: (Option<Uuid>, Option<Uuid>, String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT i.previous_release_id, i.rollback_snapshot_id, i.state_schema, n.entries
        FROM server_module_instances i
        JOIN server_module_state_namespaces n ON n.instance_id = i.instance_id
        WHERE i.instance_id = $1
        "#,
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("rolled-back root should load");
    assert_eq!(rollback_root.0, None);
    assert_eq!(rollback_root.1, None);
    assert_eq!(rollback_root.2, "community.lifecycle-helper.state/v1");
    assert_eq!(rollback_root.3["counter"], "1");

    let second_rollback = lifecycle(
        CustomModuleLifecycleAction::Rollback,
        imported.instance_id,
        7,
        2,
        None,
        None,
    );
    assert_eq!(
        apply_custom_lifecycle_with_probe(&pool, &config, &second_rollback, &probe).await,
        Err(ModuleError::Denied)
    );

    let removed = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Remove,
            imported.instance_id,
            7,
            2,
            None,
            None,
        ),
    )
    .await;
    assert_eq!(removed.lifecycle, "retired");
    let disposition: (String, String) = sqlx::query_as(
        "SELECT lifecycle, state_disposition FROM server_module_instances WHERE instance_id = $1",
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("terminal disposition should load");
    assert_eq!(disposition, ("retired".into(), "retain_for_audit".into()));

    let restore = prepare_restored_modules(
        &pool,
        Uuid::new_v4(),
        "restore-test",
        "Retired modules must remain terminal after restore",
    )
    .await
    .expect("restore reconciliation should ignore retired modules");
    assert!(restore.is_empty());
    let lifecycle: String =
        sqlx::query_scalar("SELECT lifecycle FROM server_module_instances WHERE instance_id = $1")
            .bind(imported.instance_id)
            .fetch_one(&pool)
            .await
            .expect("retired lifecycle should remain queryable");
    assert_eq!(lifecycle, "retired");

    let retained_grants: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT granted_capabilities
        FROM server_module_custom_operations
        WHERE instance_id = $1 AND action = 'disable'
        "#,
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("disabled operation should retain reviewed grant");
    assert_eq!(retained_grants, vec!["moderation_add_label"]);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn restored_custom_module_requires_readiness_and_concurrent_cas_is_atomic(pool: PgPool) {
    let server_id = server_id(&pool).await;
    let config = CustomModuleAdminConfig::for_test([11_u8; 32]);
    let probe = LocalReleaseProbe;
    let fixture = custom_fixture(
        server_id,
        "community.restore-helper",
        Uuid::new_v4(),
        "1.0.0",
        "community.restore-helper.state/v1",
        BTreeMap::new(),
    );
    let imported = import_custom_module_with_probe(&pool, &config, &fixture.command, &probe)
        .await
        .expect("custom release should import");
    apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Enable,
            imported.instance_id,
            1,
            0,
            None,
            None,
        ),
    )
    .await;

    let restore_operation = Uuid::new_v4();
    let restored = prepare_restored_modules(
        &pool,
        restore_operation,
        "restore-test",
        "Require explicit custom-module review after restore",
    )
    .await
    .expect("restore reconciliation should disable custom code");
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].module_id, "community.restore-helper");
    let replay = prepare_restored_modules(
        &pool,
        restore_operation,
        "restore-test",
        "Require explicit custom-module review after restore",
    )
    .await
    .expect("restore reconciliation should replay exactly");
    assert_eq!(replay, restored);

    let restored_root: (String, i64, bool, bool, Option<Uuid>) = sqlx::query_as(
        r#"
        SELECT lifecycle, lifecycle_revision, activation_allowed,
               restored_pending_review, current_admission_id
        FROM server_module_instances WHERE instance_id = $1
        "#,
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("restored root should load");
    assert_eq!(restored_root, ("disabled".into(), 3, false, true, None));

    let enable = lifecycle(
        CustomModuleLifecycleAction::Enable,
        imported.instance_id,
        3,
        0,
        None,
        None,
    );
    assert_eq!(
        apply_custom_lifecycle_with_probe(&pool, &config, &enable, &probe).await,
        Err(ModuleError::Denied)
    );
    let recovered = apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Recover,
            imported.instance_id,
            3,
            0,
            None,
            None,
        ),
    )
    .await;
    assert_eq!(recovered.lifecycle, "active");
    let recovered_root: (bool, bool, bool) = sqlx::query_as(
        r#"
        SELECT activation_allowed, restored_pending_review,
               current_admission_id IS NOT NULL
        FROM server_module_instances WHERE instance_id = $1
        "#,
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("recovered root should load");
    assert_eq!(recovered_root, (true, false, true));

    let first = lifecycle(
        CustomModuleLifecycleAction::Suspend,
        imported.instance_id,
        4,
        0,
        None,
        None,
    );
    let second = CustomModuleLifecycleCommand {
        operation_id: Uuid::new_v4(),
        reason: "Competing expected-revision suspension".into(),
        ..first.clone()
    };
    let (first_result, second_result) = tokio::join!(
        apply_custom_lifecycle_with_probe(&pool, &config, &first, &probe),
        apply_custom_lifecycle_with_probe(&pool, &config, &second, &probe)
    );
    let outcomes = [first_result, second_result];
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|result| matches!(result, Err(ModuleError::Conflict)))
            .count(),
        1
    );
    let root: (String, i64) = sqlx::query_as(
        "SELECT lifecycle, lifecycle_revision FROM server_module_instances WHERE instance_id = $1",
    )
    .bind(imported.instance_id)
    .fetch_one(&pool)
    .await
    .expect("CAS result should load");
    assert_eq!(root, ("suspended".into(), 5));
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn custom_release_dispatches_through_shared_private_receipt_and_effect_path(pool: PgPool) {
    let server_id = server_id(&pool).await;
    let config = CustomModuleAdminConfig::for_test([11_u8; 32]);
    let probe = LocalReleaseProbe;
    let fixture = custom_fixture(
        server_id,
        "community.dispatch-helper",
        Uuid::new_v4(),
        "1.0.0",
        "community.dispatch-helper.state/v1",
        BTreeMap::new(),
    );
    let imported = import_custom_module_with_probe(&pool, &config, &fixture.command, &probe)
        .await
        .expect("custom release should import");
    apply(
        &pool,
        &config,
        &probe,
        lifecycle(
            CustomModuleLifecycleAction::Enable,
            imported.instance_id,
            1,
            0,
            None,
            None,
        ),
    )
    .await;

    let executor = Arc::new(LocalExecutor {
        runtime: ModuleRuntime::compile_bytes(FixtureKind::Valid.component_bytes())
            .expect("custom component should compile"),
    });
    let service = ServerModuleService::start_with_executor(
        pool.clone(),
        ModuleConfig {
            enable_first_party_report: false,
            admission_signing_seed: [11_u8; 32],
            pairwise_secret: [22_u8; 32],
        },
        executor,
    )
    .await
    .expect("custom-only dispatcher should start");

    let reporter_account: Uuid = sqlx::query_scalar(
        "INSERT INTO accounts (username, password_hash) VALUES ('custom_dispatch_reporter', 'test-hash') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("reporter account should seed");
    let subject_account: Uuid = sqlx::query_scalar(
        "INSERT INTO accounts (username, password_hash) VALUES ('custom_dispatch_subject', 'test-hash') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("subject account should seed");
    let reporter: Uuid = sqlx::query_scalar(
        "INSERT INTO personas (account_id, handle, display_name) VALUES ($1, 'custom_dispatch_reporter', 'Reporter') RETURNING id",
    )
    .bind(reporter_account)
    .fetch_one(&pool)
    .await
    .expect("reporter persona should seed");
    let subject: Uuid = sqlx::query_scalar(
        "INSERT INTO personas (account_id, handle, display_name) VALUES ($1, 'custom_dispatch_subject', 'Subject') RETURNING id",
    )
    .bind(subject_account)
    .fetch_one(&pool)
    .await
    .expect("subject persona should seed");

    let mut transaction = pool.begin().await.expect("report transaction should begin");
    let report_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO persona_reports (
            reporter_persona_id, subject_persona_id, idempotency_key, category, detail
        ) VALUES ($1, $2, $3, 'other', 'Custom dispatcher private fixture')
        RETURNING id
        "#,
    )
    .bind(reporter)
    .bind(subject)
    .bind(Uuid::new_v4())
    .fetch_one(&mut *transaction)
    .await
    .expect("report should stage");
    let event_id = service
        .emitter()
        .append_persona_reported(&mut transaction, report_id, subject, "other")
        .await
        .expect("custom observation should stage")
        .expect("custom observation should return an event");
    transaction
        .commit()
        .await
        .expect("report and observation should commit atomically");

    wait_for_custom_delivery(&pool, event_id).await;
    service.shutdown().await;
    let effect: (String, i64, Uuid) = sqlx::query_as(
        r#"
        SELECT label, revision, source_event_id
        FROM server_module_report_labels
        WHERE instance_id = $1 AND report_id = $2
        "#,
    )
    .bind(imported.instance_id)
    .bind(report_id)
    .fetch_one(&pool)
    .await
    .expect("custom typed effect should be core-applied");
    assert_eq!(effect, ("priority_review".into(), 1, event_id));
    let receipt: (Uuid, Vec<u8>, String) = sqlx::query_as(
        r#"
        SELECT release_id, request_body, request_sha256
        FROM server_module_delivery_receipts WHERE event_id = $1
        "#,
    )
    .bind(event_id)
    .fetch_one(&pool)
    .await
    .expect("custom delivery receipt should persist");
    assert_eq!(receipt.0, imported.release_id);
    assert_eq!(sha256_hex(&receipt.1), receipt.2);
    let request_text = String::from_utf8(receipt.1).expect("request should be canonical UTF-8");
    assert!(!request_text.contains(&reporter.to_string()));
    assert!(!request_text.contains(&subject.to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn custom_identity_ceiling_and_reviewed_identity_collision_fail_closed(pool: PgPool) {
    let server_id = server_id(&pool).await;
    let config = CustomModuleAdminConfig::for_test([11_u8; 32]);
    let probe = LocalReleaseProbe;
    let executor = Arc::new(LocalExecutor {
        runtime: ModuleRuntime::compile_bytes(FixtureKind::Valid.component_bytes())
            .expect("reviewed fixture should compile"),
    });
    let service = ServerModuleService::start_with_executor(
        pool.clone(),
        ModuleConfig {
            enable_first_party_report: true,
            admission_signing_seed: [11_u8; 32],
            pairwise_secret: [22_u8; 32],
        },
        executor,
    )
    .await
    .expect("reviewed identity should register");
    service.shutdown().await;

    let collision = custom_fixture(
        server_id,
        "ignibyte.sentinel",
        Uuid::new_v4(),
        "2.0.0",
        "ignibyte.sentinel.state/v2",
        BTreeMap::new(),
    );
    assert_eq!(
        import_custom_module_with_probe(&pool, &config, &collision.command, &probe).await,
        Err(ModuleError::Denied)
    );

    let mut fixtures = Vec::new();
    for index in 0..8 {
        let fixture = custom_fixture(
            server_id,
            &format!("community.limit-helper-{index}"),
            Uuid::new_v4(),
            "1.0.0",
            &format!("community.limit-helper-{index}.state/v1"),
            BTreeMap::new(),
        );
        import_custom_module_with_probe(&pool, &config, &fixture.command, &probe)
            .await
            .expect("each identity within the ceiling should import");
        fixtures.push(fixture);
    }
    let overflow = custom_fixture(
        server_id,
        "community.limit-helper-overflow",
        Uuid::new_v4(),
        "1.0.0",
        "community.limit-helper-overflow.state/v1",
        BTreeMap::new(),
    );
    assert_eq!(
        import_custom_module_with_probe(&pool, &config, &overflow.command, &probe).await,
        Err(ModuleError::Denied)
    );
    let counts: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
          (SELECT count(*) FROM server_module_instances i
             JOIN server_module_releases r ON r.release_id = i.release_id
            WHERE r.provenance_class = 'operator_custom'),
          (SELECT count(*) FROM server_module_releases
            WHERE provenance_class = 'operator_custom')
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("custom identity counts should load");
    assert_eq!(counts, (8, 8));
    assert_eq!(fixtures.len(), 8);
}

async fn wait_for_custom_delivery(pool: &PgPool, event_id: Uuid) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let delivered: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM server_module_delivery_receipts WHERE event_id = $1)",
        )
        .bind(event_id)
        .fetch_one(pool)
        .await
        .expect("delivery receipt should be queryable");
        if delivered {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "custom event did not reach a delivery receipt"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn apply(
    pool: &PgPool,
    config: &CustomModuleAdminConfig,
    probe: &LocalReleaseProbe,
    command: CustomModuleLifecycleCommand,
) -> crate::server_module_custom::CustomModuleReceipt {
    apply_custom_lifecycle_with_probe(pool, config, &command, probe)
        .await
        .expect("custom lifecycle action should succeed")
}

fn lifecycle(
    action: CustomModuleLifecycleAction,
    instance_id: Uuid,
    lifecycle_revision: u64,
    state_revision: u64,
    target_release_id: Option<Uuid>,
    candidate_state: Option<BTreeMap<String, String>>,
) -> CustomModuleLifecycleCommand {
    CustomModuleLifecycleCommand {
        format: "omarchygs.operator-custom-module-lifecycle-command/v1".into(),
        action,
        operation_id: Uuid::new_v4(),
        instance_id,
        expected_lifecycle_revision: lifecycle_revision,
        expected_config_revision: 1,
        expected_state_revision: state_revision,
        target_release_id,
        candidate_state,
        actor: "server-owner".into(),
        reason: format!("Exercise {} lifecycle", action_name(action)),
    }
}

const fn action_name(action: CustomModuleLifecycleAction) -> &'static str {
    match action {
        CustomModuleLifecycleAction::Enable => "enable",
        CustomModuleLifecycleAction::Disable => "disable",
        CustomModuleLifecycleAction::Suspend => "suspend",
        CustomModuleLifecycleAction::Recover => "recover",
        CustomModuleLifecycleAction::Upgrade => "upgrade",
        CustomModuleLifecycleAction::Rollback => "rollback",
        CustomModuleLifecycleAction::Remove => "remove",
    }
}

async fn server_id(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton")
        .fetch_one(pool)
        .await
        .expect("server identity should exist")
}

fn custom_fixture(
    server_id: Uuid,
    module_id: &str,
    release_id: Uuid,
    version: &str,
    state_schema: &str,
    initial_state: BTreeMap<String, String>,
) -> CustomFixture {
    let directory = tempfile::tempdir().expect("custom fixture directory should create");
    let publisher = SigningKey::from_bytes(&[31_u8; 32]);
    let provenance = SigningKey::from_bytes(&[41_u8; 32]);
    let component = FixtureKind::Valid.component_bytes();
    let publisher_key_id = "community-publisher-v1";
    let provenance_key_id = "server-custom-root-v1";
    let manifest = ModuleReleaseManifest {
        format: RELEASE_FORMAT.into(),
        module_id: module_id.into(),
        publisher_id: "community".into(),
        release_id,
        version: version.into(),
        component_sha256: sha256_hex(component),
        wit: WitIdentity {
            package: WIT_PACKAGE.into(),
            world: WIT_WORLD.into(),
            major: 1,
            sha256: wit_sha256(),
        },
        requested_capabilities: vec![Capability::ModerationAddLabel],
        subscribed_hooks: vec![HookKind::PersonaReported],
        budgets: ResourceBudgets {
            frame_bytes: MAX_FRAME_BYTES as u32,
            memory_bytes: MAX_LINEAR_MEMORY_BYTES as u32,
            fuel: MAX_FUEL,
            execution_ms: MAX_EXECUTION_MS,
        },
        config_schema: format!("{module_id}.config/v1"),
        state_schema: state_schema.into(),
        entrypoint: "handle".into(),
    };
    manifest
        .validate()
        .expect("fixture manifest should validate");
    let release = SignedEnvelope::sign(RELEASE_FORMAT, publisher_key_id, &manifest, &publisher)
        .expect("fixture release should sign");
    let publisher_document = ModulePublicKeyDocument {
        format: "omarchygs.server-module-public-key/v1".into(),
        algorithm: "ed25519".into(),
        key_id: publisher_key_id.into(),
        verifying_key: encode_verifying_key(&publisher.verifying_key()),
    };
    let provenance_document = ModulePrivateKeyDocument {
        format: "omarchygs.server-module-private-key/v1".into(),
        algorithm: "ed25519".into(),
        key_id: provenance_key_id.into(),
        signing_seed: URL_SAFE_NO_PAD.encode(provenance.to_bytes()),
    };

    let release_path = directory.path().join("release.json");
    let component_path = directory.path().join("component.wasm");
    let publisher_path = directory.path().join("publisher-public.json");
    let provenance_path = directory.path().join("operator-private.json");
    write_private(
        &release_path,
        &canonical_json(&release).expect("release should encode"),
    );
    write_private(&component_path, component);
    write_private(
        &publisher_path,
        &canonical_json(&publisher_document).expect("publisher key should encode"),
    );
    write_private(
        &provenance_path,
        &canonical_json(&provenance_document).expect("provenance key should encode"),
    );

    CustomFixture {
        command: CustomModuleImportCommand {
            format: "omarchygs.operator-custom-module-import-command/v1".into(),
            operation_id: Uuid::new_v4(),
            server_id,
            signed_release_path: absolute(&release_path),
            component_path: absolute(&component_path),
            publisher_public_key_path: absolute(&publisher_path),
            provenance_private_key_path: absolute(&provenance_path),
            publisher_key_sha256: verifying_key_sha256(&publisher.verifying_key()),
            provenance_key_sha256: verifying_key_sha256(&provenance.verifying_key()),
            granted_capabilities: vec![Capability::ModerationAddLabel],
            initial_config: BTreeMap::from([("policy".into(), "strict".into())]),
            initial_state,
            acknowledgement: UNREVIEWED_ACKNOWLEDGEMENT.into(),
            actor: "server-owner".into(),
            reason: format!("Import {module_id} {version}"),
        },
        _directory: directory,
    }
}

fn absolute(path: &Path) -> PathBuf {
    path.canonicalize()
        .expect("fixture path should canonicalize")
}

fn write_private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("private fixture should write");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .expect("private fixture mode should set");
}
