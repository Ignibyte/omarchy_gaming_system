//! PostgreSQL production server-module integration tests.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::Duration,
};

use ed25519_dalek::VerifyingKey;
use omarchygs_server_module_runtime::{
    FixtureKind, HookPayload, HostRequest, HostResponse, ModuleRuntime, PRIORITY_REVIEW_LABEL,
    canonical_json, sha256_hex,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    accounts::{self, RegistrationInput},
    personas::{self, CreatePersonaInput},
    reports::{self, CreateReportInput, ReportError, ReportOutcome},
    server_modules::{
        BUILTIN_INSTANCE_ID, MAX_UNDELIVERED_EVENTS, ModuleConfig, ModuleError, ModuleExecutor,
        ModuleLifecycleAction, ModuleLifecycleCommand, ModuleStateOperation, ServerModuleService,
        apply_lifecycle_command, list_module_inventory, migrate_state, prepare_restored_modules,
        prune_delivered, rollback_state, update_configuration, update_state,
    },
    sessions::{self, CreateSessionInput, SessionCreation},
};

const MODE_VALID: u8 = 0;
const MODE_UNAVAILABLE: u8 = 1;
const MODE_SLOW: u8 = 2;

struct ControlledExecutor {
    runtime: ModuleRuntime,
    mode: AtomicU8,
    executing: AtomicBool,
    calls: Mutex<Vec<Uuid>>,
}

impl ControlledExecutor {
    fn new() -> Self {
        Self {
            runtime: ModuleRuntime::compile(FixtureKind::Valid)
                .expect("reviewed fixture should compile"),
            mode: AtomicU8::new(MODE_VALID),
            executing: AtomicBool::new(false),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn set_mode(&self, mode: u8) {
        self.mode.store(mode, Ordering::SeqCst);
    }

    fn clear_calls(&self) {
        self.calls.lock().expect("call log should lock").clear();
    }

    fn calls(&self) -> Vec<Uuid> {
        self.calls.lock().expect("call log should lock").clone()
    }
}

impl ModuleExecutor for ControlledExecutor {
    fn execute(
        &self,
        request: HostRequest,
        core_key: VerifyingKey,
    ) -> Result<HostResponse, ModuleError> {
        self.calls
            .lock()
            .expect("call log should lock")
            .push(request.event.event_id);
        match self.mode.load(Ordering::SeqCst) {
            MODE_UNAVAILABLE => Err(ModuleError::Unavailable),
            MODE_SLOW => {
                self.executing.store(true, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(750));
                self.executing.store(false, Ordering::SeqCst);
                Ok(self.runtime.execute(&request, &core_key))
            }
            _ => Ok(self.runtime.execute(&request, &core_key)),
        }
    }
}

struct TestPersona {
    id: Uuid,
    token: String,
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn report_observation_is_atomic_private_idempotent_and_core_applied(pool: PgPool) {
    let executor = Arc::new(ControlledExecutor::new());
    let service =
        ServerModuleService::start_with_executor(pool.clone(), module_config(), executor.clone())
            .await
            .expect("reviewed module should start");
    executor.clear_calls();
    let reporter = create_test_persona(&pool, "module_reporter", "module_reporter").await;
    let subject = create_test_persona(&pool, "module_subject", "module_subject").await;
    let operation_id = Uuid::new_v4();

    let outcome = create_report(
        &pool,
        &reporter,
        subject.id,
        operation_id,
        Some(&service.emitter()),
    )
    .await
    .expect("report and observation should commit");
    assert!(matches!(outcome, ReportOutcome::Created(_)));
    wait_for_outbox_status(&pool, "delivered", 1).await;

    let (payload, partition_subject): (Value, String) =
        sqlx::query_as("SELECT payload, partition_subject FROM server_module_outbox LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("observation should remain inspectable");
    assert_eq!(
        payload
            .as_object()
            .expect("payload should be an object")
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        ["category", "kind", "report_id"].into_iter().collect()
    );
    let payload_text = payload.to_string();
    for private in [
        reporter.id.to_string(),
        subject.id.to_string(),
        reporter.token.clone(),
        "module_reporter".to_owned(),
        "module_subject".to_owned(),
        "private report detail".to_owned(),
    ] {
        assert!(!payload_text.contains(&private));
    }
    assert!(!partition_subject.contains(&subject.id.to_string()));

    let label: (String, i64) =
        sqlx::query_as("SELECT label, revision FROM server_module_report_labels LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("core should apply the typed label intent");
    assert_eq!(label, ("priority_review".to_owned(), 1));
    assert_eq!(PRIORITY_REVIEW_LABEL, 7);
    assert_eq!(count(&pool, "server_module_delivery_receipts").await, 1);
    assert_eq!(count(&pool, "server_module_intent_receipts").await, 1);
    let (request_body, request_sha, target_report_id): (Vec<u8>, String, Uuid) = sqlx::query_as(
        r#"
        SELECT request_body, request_sha256, target_report_id
        FROM server_module_delivery_receipts
        LIMIT 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("complete delivery request evidence should load");
    assert_eq!(sha256_hex(&request_body), request_sha);
    let stored_request: HostRequest =
        serde_json::from_slice(&request_body).expect("request evidence should be canonical JSON");
    assert_eq!(stored_request.event.attempt, 0);
    let HookPayload::PersonaReported { report_id, .. } = stored_request.event.payload;
    assert_eq!(report_id, target_report_id);
    let request_text = String::from_utf8(request_body).expect("request evidence should be UTF-8");
    for private in [
        reporter.id.to_string(),
        subject.id.to_string(),
        reporter.token.clone(),
        "module_reporter".to_owned(),
        "module_subject".to_owned(),
        "private report detail".to_owned(),
    ] {
        assert!(!request_text.contains(&private));
    }

    let replay = create_report(
        &pool,
        &reporter,
        subject.id,
        operation_id,
        Some(&service.emitter()),
    )
    .await
    .expect("report replay should succeed");
    assert!(matches!(replay, ReportOutcome::Existing(_)));
    assert_eq!(count(&pool, "server_module_outbox").await, 1);

    create_report(
        &pool,
        &reporter,
        subject.id,
        Uuid::new_v4(),
        Some(&service.emitter()),
    )
    .await
    .expect("second partition event should enqueue");
    wait_for_outbox_status(&pool, "delivered", 2).await;
    let ordered_events: Vec<Uuid> =
        sqlx::query_scalar("SELECT event_id FROM server_module_outbox ORDER BY sequence")
            .fetch_all(&pool)
            .await
            .expect("ordered events should load");
    assert_eq!(executor.calls(), ordered_events);

    executor.clear_calls();
    sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = 'retry', delivered_at = NULL,
            next_attempt_at = clock_timestamp(), updated_at = clock_timestamp()
        WHERE event_id = $1
        "#,
    )
    .bind(ordered_events[0])
    .execute(&pool)
    .await
    .expect("at-least-once replay fixture should reset delivery marker");
    wait_for_outbox_status(&pool, "delivered", 2).await;
    assert!(
        executor.calls().is_empty(),
        "immutable delivery receipt should reconcile without re-execution"
    );

    let report_id: Uuid =
        sqlx::query_scalar("SELECT id FROM persona_reports WHERE idempotency_key = $1")
            .bind(operation_id)
            .fetch_one(&pool)
            .await
            .expect("report should exist");
    let before = count(&pool, "server_module_outbox").await;
    let mut transaction = pool.begin().await.expect("rollback fixture should begin");
    service
        .emitter()
        .append_persona_reported(&mut transaction, report_id, subject.id, "harassment")
        .await
        .expect("observation should append inside transaction");
    transaction
        .rollback()
        .await
        .expect("fixture transaction should roll back");
    assert_eq!(count(&pool, "server_module_outbox").await, before);
    service.shutdown().await;
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn dispatcher_releases_database_locks_and_degrades_after_bounded_failures(pool: PgPool) {
    let executor = Arc::new(ControlledExecutor::new());
    let service =
        ServerModuleService::start_with_executor(pool.clone(), module_config(), executor.clone())
            .await
            .expect("reviewed module should start");
    let reporter = create_test_persona(&pool, "module_slow_actor", "module_slow_actor").await;
    let subject = create_test_persona(&pool, "module_slow_peer", "module_slow_peer").await;

    executor.set_mode(MODE_SLOW);
    create_report(
        &pool,
        &reporter,
        subject.id,
        Uuid::new_v4(),
        Some(&service.emitter()),
    )
    .await
    .expect("slow observation should enqueue");
    wait_for_flag(&executor.executing, true).await;
    tokio::time::timeout(
        Duration::from_millis(250),
        sqlx::query("UPDATE persona_reports SET category = category").execute(&pool),
    )
    .await
    .expect("domain update must not wait for module execution")
    .expect("domain update should succeed");
    wait_for_outbox_status(&pool, "delivered", 1).await;

    executor.set_mode(MODE_UNAVAILABLE);
    create_report(
        &pool,
        &reporter,
        subject.id,
        Uuid::new_v4(),
        Some(&service.emitter()),
    )
    .await
    .expect("failing observation should enqueue");
    wait_for_outbox_status(&pool, "dead_letter", 1).await;
    wait_for_lifecycle(&pool, "degraded").await;
    let attempts: i32 = sqlx::query_scalar(
        "SELECT attempt_count FROM server_module_outbox WHERE status = 'dead_letter'",
    )
    .fetch_one(&pool)
    .await
    .expect("dead letter should retain attempts");
    assert_eq!(attempts, 3);
    let outbox_before_gap = count(&pool, "server_module_outbox").await;
    let calls_before_gap = executor.calls().len();
    let degraded_report = create_report(
        &pool,
        &reporter,
        subject.id,
        Uuid::new_v4(),
        Some(&service.emitter()),
    )
    .await
    .expect("a degraded optional module must not reject the core report");
    assert!(matches!(degraded_report, ReportOutcome::Created(_)));
    assert_eq!(
        count(&pool, "server_module_outbox").await,
        outbox_before_gap
    );
    assert_eq!(executor.calls().len(), calls_before_gap);
    let degraded_gap: (i64, Option<String>) = sqlx::query_as(
        r#"
        SELECT observation_gap_count, last_observation_gap_reason
        FROM server_module_instances
        WHERE instance_id = $1
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(&pool)
    .await
    .expect("degraded observation gap should load");
    assert_eq!(degraded_gap, (1, Some("module_inactive".into())));
    service.shutdown().await;

    executor.set_mode(MODE_VALID);
    assert!(matches!(
        ServerModuleService::start_with_executor(pool.clone(), module_config(), executor.clone(),)
            .await,
        Err(ModuleError::Denied)
    ));
    let revision: i64 = sqlx::query_scalar(
        "SELECT lifecycle_revision FROM server_module_instances WHERE instance_id = $1",
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(&pool)
    .await
    .expect("module instance should exist");
    apply_lifecycle_command(
        &pool,
        &ModuleLifecycleCommand {
            format: "omarchygs.server-module-lifecycle-command/v1".into(),
            operation_id: Uuid::new_v4(),
            module_id: "ignibyte.sentinel".into(),
            expected_revision: revision,
            action: ModuleLifecycleAction::Recover,
            actor: "module-test".into(),
            reason: "Recover after the bounded failure drill".into(),
        },
    )
    .await
    .expect("operator recovery should return to disabled");
    let restarted =
        ServerModuleService::start_with_executor(pool.clone(), module_config(), executor)
            .await
            .expect("readiness should reactivate after recovery");
    wait_for_lifecycle(&pool, "active").await;
    restarted.shutdown().await;
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn queue_cap_records_gap_and_state_lifecycle_restore_are_cas(pool: PgPool) {
    let executor = Arc::new(ControlledExecutor::new());
    let service = ServerModuleService::start_with_executor(pool.clone(), module_config(), executor)
        .await
        .expect("reviewed module should start");
    let reporter = create_test_persona(&pool, "module_cap_actor", "module_cap_actor").await;
    let subject = create_test_persona(&pool, "module_cap_peer", "module_cap_peer").await;
    let base_operation = Uuid::new_v4();
    create_report(
        &pool,
        &reporter,
        subject.id,
        base_operation,
        Some(&service.emitter()),
    )
    .await
    .expect("base report should enqueue");
    wait_for_outbox_status(&pool, "delivered", 1).await;
    let report_id: Uuid =
        sqlx::query_scalar("SELECT id FROM persona_reports WHERE idempotency_key = $1")
            .bind(base_operation)
            .fetch_one(&pool)
            .await
            .expect("base report should exist");
    seed_dead_letters(&pool, subject.id, report_id, MAX_UNDELIVERED_EVENTS).await;
    let outbox_before_gap = count(&pool, "server_module_outbox").await;
    let saturated_operation = Uuid::new_v4();
    let saturated = create_report(
        &pool,
        &reporter,
        subject.id,
        saturated_operation,
        Some(&service.emitter()),
    )
    .await
    .expect("a saturated optional module must not reject the core report");
    assert!(matches!(saturated, ReportOutcome::Created(_)));
    let committed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM persona_reports WHERE idempotency_key = $1)",
    )
    .bind(saturated_operation)
    .fetch_one(&pool)
    .await
    .expect("committed report should be queryable");
    assert!(committed);
    assert_eq!(
        count(&pool, "server_module_outbox").await,
        outbox_before_gap
    );
    let saturated_gap: (i64, Option<String>, bool) = sqlx::query_as(
        r#"
        SELECT observation_gap_count, last_observation_gap_reason,
               last_observation_gap_at IS NOT NULL
        FROM server_module_instances
        WHERE instance_id = $1
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(&pool)
    .await
    .expect("saturated observation gap should load");
    assert_eq!(saturated_gap, (1, Some("queue_saturated".into()), true));
    service.shutdown().await;
    sqlx::query("DELETE FROM server_module_outbox WHERE status = 'dead_letter'")
        .execute(&pool)
        .await
        .expect("saturation-only dead letters should be removed for state fixture");

    let active_revision = module_revision(&pool).await;
    let disable_operation = Uuid::new_v4();
    let disable = lifecycle_command(
        disable_operation,
        active_revision,
        ModuleLifecycleAction::Disable,
        "Disable for state maintenance",
    );
    let disabled = apply_lifecycle_command(&pool, &disable)
        .await
        .expect("active module should disable");
    assert_eq!(disabled.resulting_state, "disabled");
    assert_eq!(
        apply_lifecycle_command(&pool, &disable)
            .await
            .expect("exact lifecycle replay should succeed"),
        disabled
    );
    assert_eq!(
        apply_lifecycle_command(
            &pool,
            &ModuleLifecycleCommand {
                reason: "Changed replay body".into(),
                ..disable.clone()
            }
        )
        .await,
        Err(ModuleError::Conflict)
    );

    let state_operation = Uuid::new_v4();
    let state = update_state(
        &pool,
        state_operation,
        0,
        &[ModuleStateOperation::Set {
            key: "review_count".into(),
            value: "1".into(),
        }],
        "module-test",
        "Create bounded state",
    )
    .await
    .expect("disabled module state should update");
    assert_eq!(state.resulting_revision, 1);
    assert_eq!(
        migrate_state(
            &pool,
            state_operation,
            1,
            &[ModuleStateOperation::Set {
                key: "review_count".into(),
                value: "2".into(),
            }],
            "module-test",
            "Reject operation identity reuse",
        )
        .await,
        Err(ModuleError::Conflict)
    );
    assert_eq!(
        update_state(
            &pool,
            Uuid::new_v4(),
            0,
            &[ModuleStateOperation::Remove {
                key: "review_count".into(),
            }],
            "module-test",
            "Reject stale state",
        )
        .await,
        Err(ModuleError::Conflict)
    );
    let migration = migrate_state(
        &pool,
        Uuid::new_v4(),
        1,
        &[ModuleStateOperation::Set {
            key: "review_count".into(),
            value: "2".into(),
        }],
        "module-test",
        "Migrate bounded state",
    )
    .await
    .expect("state migration should retain a snapshot");
    let snapshot = migration.snapshot_id.expect("migration needs snapshot");
    let rollback = rollback_state(
        &pool,
        Uuid::new_v4(),
        snapshot,
        2,
        "module-test",
        "Roll back retained state",
    )
    .await
    .expect("snapshot rollback should be monotonic");
    assert_eq!(rollback.resulting_revision, 3);
    let configuration = std::collections::BTreeMap::from([("policy".into(), "strict".into())]);
    let configured = update_configuration(
        &pool,
        Uuid::new_v4(),
        1,
        &configuration,
        "module-test",
        "Confirm exact configuration",
    )
    .await
    .expect("configuration CAS should succeed");
    assert_eq!(configured.resulting_revision, 2);

    let restore_operation = Uuid::new_v4();
    let restored = prepare_restored_modules(
        &pool,
        restore_operation,
        "module-test",
        "Reconcile isolated restore",
    )
    .await
    .expect("restore should force disabled review");
    assert_eq!(restored.len(), 1);
    let inventory = list_module_inventory(&pool)
        .await
        .expect("safe inventory should load");
    assert_eq!(inventory.modules.len(), 1);
    assert_eq!(inventory.modules[0].lifecycle, "disabled");
    assert!(inventory.modules[0].restored_pending_review);
    assert_eq!(
        prepare_restored_modules(
            &pool,
            restore_operation,
            "module-test",
            "Reconcile isolated restore",
        )
        .await
        .expect("exact restore replay should succeed"),
        restored
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn readiness_finalization_rejects_changed_configuration_and_state(pool: PgPool) {
    let executor = Arc::new(ControlledExecutor::new());
    executor.set_mode(MODE_SLOW);
    let start_pool = pool.clone();
    let start_executor = executor.clone();
    let configuration_start = tokio::spawn(async move {
        ServerModuleService::start_with_executor(start_pool, module_config(), start_executor).await
    });
    wait_for_flag(&executor.executing, true).await;
    update_configuration(
        &pool,
        Uuid::new_v4(),
        1,
        &std::collections::BTreeMap::from([("policy".into(), "review".into())]),
        "module-test",
        "Change configuration while readiness is executing",
    )
    .await
    .expect("disabled configuration should change concurrently");
    let configuration_result = configuration_start
        .await
        .expect("configuration readiness task should join");
    assert!(matches!(configuration_result, Err(ModuleError::Conflict)));

    let start_pool = pool.clone();
    let start_executor = executor.clone();
    let state_start = tokio::spawn(async move {
        ServerModuleService::start_with_executor(start_pool, module_config(), start_executor).await
    });
    wait_for_flag(&executor.executing, true).await;
    update_state(
        &pool,
        Uuid::new_v4(),
        0,
        &[ModuleStateOperation::Set {
            key: "review_count".into(),
            value: "1".into(),
        }],
        "module-test",
        "Change state while readiness is executing",
    )
    .await
    .expect("disabled state should change concurrently");
    let state_result = state_start.await.expect("state readiness task should join");
    assert!(matches!(state_result, Err(ModuleError::Conflict)));

    let (lifecycle, current_admission_id, admissions, enable_audits): (
        String,
        Option<Uuid>,
        i64,
        i64,
    ) = sqlx::query_as(
        r#"
        SELECT i.lifecycle, i.current_admission_id,
               (SELECT count(*) FROM server_module_admissions),
               (SELECT count(*) FROM server_module_lifecycle_audit WHERE action = 'enable')
        FROM server_module_instances i
        WHERE i.instance_id = $1
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(&pool)
    .await
    .expect("failed readiness state should load");
    assert_eq!(lifecycle, "disabled");
    assert_eq!(current_admission_id, None);
    assert_eq!(admissions, 0);
    assert_eq!(enable_audits, 0);

    executor.set_mode(MODE_VALID);
    let service = ServerModuleService::start_with_executor(pool.clone(), module_config(), executor)
        .await
        .expect("unchanged current revisions should pass a fresh readiness probe");
    wait_for_lifecycle(&pool, "active").await;
    service.shutdown().await;
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn delivery_request_evidence_survives_outbox_pruning(pool: PgPool) {
    let executor = Arc::new(ControlledExecutor::new());
    let service = ServerModuleService::start_with_executor(pool.clone(), module_config(), executor)
        .await
        .expect("reviewed module should start");
    service.shutdown().await;
    let reporter = create_test_persona(&pool, "module_receipt_actor", "module_receipt_actor").await;
    let subject = create_test_persona(&pool, "module_receipt_peer", "module_receipt_peer").await;
    let report = create_report(&pool, &reporter, subject.id, Uuid::new_v4(), None)
        .await
        .expect("receipt target report should commit");
    let report_id = match report {
        ReportOutcome::Created(receipt) | ReportOutcome::Existing(receipt) => receipt.id,
    };
    let payload = json!({
        "kind": "persona_reported",
        "report_id": report_id,
        "category": "other"
    });
    let payload_sha = sha256_hex(&canonical_json(&payload).expect("payload should encode"));
    let request_body = br#"{"retained":"request"}"#.to_vec();
    let request_sha = sha256_hex(&request_body);
    let response_body = br#"{"retained":"response"}"#.to_vec();
    let response_sha = sha256_hex(&response_body);
    let inserted: Vec<Uuid> = sqlx::query_scalar(
        r#"
        WITH seeded AS (
            INSERT INTO server_module_outbox (
                event_id, instance_id, release_id, admission_id, admission_revision,
                hook, partition_subject, subject_persona_id, target_report_id,
                causal_revision, payload, payload_sha256, config_snapshot,
                config_revision, state_snapshot, state_revision, status,
                attempt_count, delivered_at
            )
            SELECT gen_random_uuid(), i.instance_id, i.release_id,
                   i.current_admission_id, i.current_admission_revision,
                   'persona_reported', 'receipt-retention-subject', $1, $2, 0,
                   $3, $4, i.config, i.config_revision,
                   n.entries, n.revision, 'delivered', 1, clock_timestamp()
            FROM server_module_instances i
            JOIN server_module_state_namespaces n ON n.instance_id = i.instance_id
            CROSS JOIN generate_series(1, 4097)
            WHERE i.instance_id = $5
            RETURNING event_id, release_id, target_report_id
        ), receipts AS (
            INSERT INTO server_module_delivery_receipts (
                event_id, release_id, request_sha256, response_sha256,
                response_body, outcome_code, attempt_count, request_body,
                target_report_id
            )
            SELECT event_id, release_id, $6, $7, $8, 'noop', 1, $9,
                   target_report_id
            FROM seeded
            RETURNING event_id
        )
        SELECT event_id FROM receipts
        "#,
    )
    .bind(subject.id)
    .bind(report_id)
    .bind(payload)
    .bind(payload_sha)
    .bind(BUILTIN_INSTANCE_ID)
    .bind(&request_sha)
    .bind(response_sha)
    .bind(response_body)
    .bind(&request_body)
    .fetch_all(&pool)
    .await
    .expect("retention fixture should seed");
    assert_eq!(inserted.len(), 4097);

    prune_delivered(&pool, BUILTIN_INSTANCE_ID)
        .await
        .expect("delivered outbox pruning should succeed");
    assert_eq!(count(&pool, "server_module_outbox").await, 4096);
    assert_eq!(count(&pool, "server_module_delivery_receipts").await, 4097);
    let (retained_body, retained_sha, retained_target): (Vec<u8>, String, Uuid) = sqlx::query_as(
        r#"
        SELECT d.request_body, d.request_sha256, d.target_report_id
        FROM server_module_delivery_receipts d
        LEFT JOIN server_module_outbox o ON o.event_id = d.event_id
        WHERE o.event_id IS NULL
        LIMIT 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("pruned delivery evidence should remain");
    assert_eq!(retained_target, report_id);
    assert_eq!(retained_body, request_body);
    assert_eq!(retained_sha, request_sha);
    assert_eq!(sha256_hex(&retained_body), retained_sha);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn upgraded_receipts_distinguish_legacy_evidence_and_require_new_preimages(pool: PgPool) {
    let executor = Arc::new(ControlledExecutor::new());
    let service = ServerModuleService::start_with_executor(pool.clone(), module_config(), executor)
        .await
        .expect("reviewed module should start");
    service.shutdown().await;

    sqlx::query(
        "ALTER TABLE server_module_delivery_receipts DROP CONSTRAINT server_module_delivery_receipts_request_evidence",
    )
    .execute(&pool)
    .await
    .expect("upgrade fixture should temporarily remove the new-write constraint");
    sqlx::query(
        r#"
        INSERT INTO server_module_delivery_receipts (
            event_id, release_id, request_sha256, response_sha256,
            response_body, outcome_code, attempt_count
        )
        SELECT $1, release_id, repeat('a', 64), repeat('b', 64),
               '{}'::BYTEA, 'noop', 1
        FROM server_module_instances
        WHERE instance_id = $2
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(BUILTIN_INSTANCE_ID)
    .execute(&pool)
    .await
    .expect("legacy digest-only receipt should remain representable");
    sqlx::query(
        r#"
        ALTER TABLE server_module_delivery_receipts
        ADD CONSTRAINT server_module_delivery_receipts_request_evidence CHECK (
            request_body IS NOT NULL
            AND octet_length(request_body) BETWEEN 1 AND 65536
            AND target_report_id IS NOT NULL
        ) NOT VALID
        "#,
    )
    .execute(&pool)
    .await
    .expect("upgrade constraint should tolerate existing legacy evidence");

    let rejected = sqlx::query(
        r#"
        INSERT INTO server_module_delivery_receipts (
            event_id, release_id, request_sha256, response_sha256,
            response_body, outcome_code, attempt_count
        )
        SELECT $1, release_id, repeat('c', 64), repeat('d', 64),
               '{}'::BYTEA, 'noop', 1
        FROM server_module_instances
        WHERE instance_id = $2
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(BUILTIN_INSTANCE_ID)
    .execute(&pool)
    .await
    .expect_err("new receipt without request evidence must be rejected");
    assert_eq!(
        rejected
            .as_database_error()
            .and_then(|error| error.code().map(|code| code.into_owned())),
        Some("23514".into())
    );

    let inventory = list_module_inventory(&pool)
        .await
        .expect("legacy receipt inventory should load");
    assert_eq!(inventory.modules[0].delivery_receipts, 1);
    assert_eq!(inventory.modules[0].legacy_delivery_receipts, 1);
}

fn module_config() -> ModuleConfig {
    ModuleConfig {
        admission_signing_seed: [11_u8; 32],
        pairwise_secret: [22_u8; 32],
    }
}

async fn create_test_persona(pool: &PgPool, username: &str, handle: &str) -> TestPersona {
    accounts::register_account(
        pool,
        RegistrationInput {
            invite_code: accounts::create_test_invite(pool).await,
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
            device_name: "server module test".to_owned(),
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

async fn create_report(
    pool: &PgPool,
    reporter: &TestPersona,
    subject_id: Uuid,
    operation_id: Uuid,
    emitter: Option<&crate::server_modules::ModuleEmitter>,
) -> Result<ReportOutcome, ReportError> {
    reports::create_report_with_emitter(
        pool,
        &reporter.token,
        &reporter.id.to_string(),
        CreateReportInput {
            idempotency_key: operation_id.to_string(),
            subject_persona_id: subject_id.to_string(),
            category: "harassment".into(),
            detail: "private report detail".into(),
        },
        emitter,
    )
    .await
}

async fn wait_for_outbox_status(pool: &PgPool, status: &str, minimum: i64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
    loop {
        let found: i64 =
            sqlx::query_scalar("SELECT count(*) FROM server_module_outbox WHERE status = $1")
                .bind(status)
                .fetch_one(pool)
                .await
                .expect("outbox should be queryable");
        if found >= minimum {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "outbox did not reach {status}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_lifecycle(pool: &PgPool, lifecycle: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
    loop {
        let found: String = sqlx::query_scalar(
            "SELECT lifecycle FROM server_module_instances WHERE instance_id = $1",
        )
        .bind(BUILTIN_INSTANCE_ID)
        .fetch_one(pool)
        .await
        .expect("module lifecycle should be queryable");
        if found == lifecycle {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "module did not reach {lifecycle}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_for_flag(flag: &AtomicBool, expected: bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while flag.load(Ordering::SeqCst) != expected {
        assert!(
            tokio::time::Instant::now() < deadline,
            "executor flag did not change"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn count(pool: &PgPool, table: &str) -> i64 {
    let query = match table {
        "server_module_outbox" => "SELECT count(*) FROM server_module_outbox",
        "server_module_delivery_receipts" => "SELECT count(*) FROM server_module_delivery_receipts",
        "server_module_intent_receipts" => "SELECT count(*) FROM server_module_intent_receipts",
        _ => panic!("unaudited fixture table"),
    };
    sqlx::query_scalar(query)
        .fetch_one(pool)
        .await
        .expect("fixture table should be queryable")
}

async fn module_revision(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        "SELECT lifecycle_revision FROM server_module_instances WHERE instance_id = $1",
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(pool)
    .await
    .expect("module revision should be queryable")
}

fn lifecycle_command(
    operation_id: Uuid,
    expected_revision: i64,
    action: ModuleLifecycleAction,
    reason: &str,
) -> ModuleLifecycleCommand {
    ModuleLifecycleCommand {
        format: "omarchygs.server-module-lifecycle-command/v1".into(),
        operation_id,
        module_id: "ignibyte.sentinel".into(),
        expected_revision,
        action,
        actor: "module-test".into(),
        reason: reason.into(),
    }
}

async fn seed_dead_letters(pool: &PgPool, subject_id: Uuid, report_id: Uuid, count: i64) {
    sqlx::query(
        r#"
        INSERT INTO server_module_outbox (
            event_id, instance_id, release_id, admission_id, admission_revision,
            hook, partition_subject, subject_persona_id, target_report_id,
            causal_revision, payload, payload_sha256, config_snapshot,
            config_revision, state_snapshot, state_revision, status,
            attempt_count, last_error_code, dead_lettered_at
        )
        SELECT gen_random_uuid(), i.instance_id, i.release_id,
               i.current_admission_id, i.current_admission_revision,
               'persona_reported', 'queue-cap-fixture-' || series::TEXT,
               $1, $2, 0,
               $3, repeat('a', 64), i.config, i.config_revision,
               n.entries, n.revision, 'dead_letter', 3,
               'fixture_failure', clock_timestamp()
        FROM server_module_instances i
        JOIN server_module_state_namespaces n ON n.instance_id = i.instance_id
        CROSS JOIN generate_series(1, $4) AS series
        WHERE i.instance_id = $5
        "#,
    )
    .bind(subject_id)
    .bind(report_id)
    .bind(json!({
        "kind": "persona_reported",
        "report_id": report_id,
        "category": "harassment"
    }))
    .bind(count)
    .bind(BUILTIN_INSTANCE_ID)
    .execute(pool)
    .await
    .expect("dead-letter saturation fixture should seed");
}
