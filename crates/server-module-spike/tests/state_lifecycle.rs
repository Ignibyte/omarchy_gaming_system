use omarchygs_server_module_spike::{
    DispatchQueue, LifecycleStatus, MigrationPlan, ModuleLifecycle, NamespaceState, ProofError,
    ProvenanceClass, StateOperation, build_fixture_component, fixture_request,
};
use uuid::Uuid;

const VALID: &[u8] = include_bytes!("../fixtures/components/valid.wat");

fn fixture_event(attempt: u16) -> omarchygs_server_module_spike::ModuleHookEvent {
    let component = build_fixture_component(VALID).expect("fixture component builds");
    let mut request = fixture_request(
        &component,
        ProvenanceClass::OperatorCustom {
            server_id: Uuid::parse_str("20000000-0000-4000-8000-000000000002")
                .expect("fixture UUID"),
        },
    )
    .expect("fixture");
    request.event.attempt = attempt;
    request.event
}

#[test]
fn bounded_queue_preserves_partition_order_and_applies_backpressure() {
    let mut queue = DispatchQueue::new(2).expect("queue");
    queue
        .enqueue("release:subject-a", fixture_event(1))
        .expect("first event");
    queue
        .enqueue("release:subject-a", fixture_event(2))
        .expect("second event");
    assert!(matches!(
        queue.enqueue("release:subject-b", fixture_event(3)),
        Err(ProofError::QueueFull)
    ));
    assert_eq!(queue.pop("release:subject-a").expect("first").attempt, 1);
    assert_eq!(queue.pop("release:subject-a").expect("second").attempt, 2);
    assert_eq!(queue.active_partition_count(), 0);
}

#[test]
fn state_is_namespaced_revisioned_bounded_and_recoverable() {
    let mut state = NamespaceState::empty("ignibyte.sentinel", "state/v1").expect("state");
    state
        .compare_and_set(
            0,
            &[StateOperation::Set {
                key: "count".into(),
                value: "1".into(),
            }],
        )
        .expect("CAS succeeds");
    assert!(matches!(
        state.compare_and_set(
            0,
            &[StateOperation::Set {
                key: "count".into(),
                value: "2".into()
            }]
        ),
        Err(ProofError::RevisionConflict)
    ));

    let backup = state.backup().expect("backup");
    let restored = NamespaceState::restore(&backup, "ignibyte.sentinel").expect("restore");
    assert_eq!(state, restored);
    assert!(NamespaceState::restore(&backup, "another.module").is_err());

    let oversized = "x".repeat(513);
    let original = state.clone();
    assert!(
        state
            .compare_and_set(
                1,
                &[StateOperation::Set {
                    key: "oversized".into(),
                    value: oversized
                }]
            )
            .is_err()
    );
    assert_eq!(state, original);
}

#[test]
fn migration_is_atomic_and_retains_an_explicit_rollback_snapshot() {
    let mut state = NamespaceState::empty("ignibyte.sentinel", "state/v1").expect("state");
    state
        .compare_and_set(
            0,
            &[StateOperation::Set {
                key: "count".into(),
                value: "1".into(),
            }],
        )
        .expect("seed");
    let rollback = state
        .migrate(
            1,
            &MigrationPlan {
                from_schema: "state/v1".into(),
                to_schema: "state/v2".into(),
                operations: vec![StateOperation::Set {
                    key: "migrated".into(),
                    value: "yes".into(),
                }],
            },
        )
        .expect("migration");
    assert_eq!(state.schema, "state/v2");
    assert_eq!(rollback.schema, "state/v1");

    let before_failure = state.clone();
    assert!(
        state
            .migrate(
                2,
                &MigrationPlan {
                    from_schema: "state/v2".into(),
                    to_schema: "state/v3".into(),
                    operations: vec![StateOperation::Set {
                        key: "bad".into(),
                        value: "x".repeat(513),
                    }],
                }
            )
            .is_err()
    );
    assert_eq!(state, before_failure);
}

#[test]
fn lifecycle_requires_expected_revisions_supports_replay_and_retirement_is_terminal() {
    let release = Uuid::new_v4();
    let mut lifecycle = ModuleLifecycle::staged(release);
    let install = Uuid::new_v4();
    let first = lifecycle
        .transition(install, 0, LifecycleStatus::Disabled, "install verified")
        .expect("stage to disabled");
    let replay = lifecycle
        .transition(install, 0, LifecycleStatus::Disabled, "install verified")
        .expect("idempotent replay");
    assert_eq!(first, replay);
    assert!(matches!(
        lifecycle.transition(install, 0, LifecycleStatus::Enabling, "changed replay"),
        Err(ProofError::ReplayConflict)
    ));
    assert!(matches!(
        lifecycle.transition(Uuid::new_v4(), 0, LifecycleStatus::Enabling, "stale"),
        Err(ProofError::RevisionConflict)
    ));

    lifecycle
        .transition(Uuid::new_v4(), 1, LifecycleStatus::Enabling, "enable")
        .expect("enabling");
    lifecycle
        .transition(Uuid::new_v4(), 2, LifecycleStatus::Active, "ready")
        .expect("active");
    lifecycle
        .transition(Uuid::new_v4(), 3, LifecycleStatus::Disabled, "disable")
        .expect("disabled");
    lifecycle
        .transition(Uuid::new_v4(), 4, LifecycleStatus::Retired, "remove")
        .expect("retired");
    assert!(
        lifecycle
            .transition(Uuid::new_v4(), 5, LifecycleStatus::Disabled, "resurrect")
            .is_err()
    );
    assert_eq!(lifecycle.audit.len(), 5);
}
