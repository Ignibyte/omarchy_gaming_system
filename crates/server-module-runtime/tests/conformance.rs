use std::{collections::BTreeMap, io::Cursor, path::Path};

use ed25519_dalek::SigningKey;
use omarchygs_server_module_runtime::{
    ADMISSION_FORMAT, AdmissionCoordinates, BUILTIN_MODULE_ID, BUILTIN_RELEASE_ID,
    BUILTIN_SUCCESSOR_RELEASE_ID, Capability, ExecutionTrust, FixtureKind, HOOK_FORMAT, HookKind,
    HookPayload, HostRequest, HostResult, MAX_EXECUTION_MS, MAX_FRAME_BYTES, MAX_FUEL,
    MAX_LINEAR_MEMORY_BYTES, ModuleHookEvent, ModuleReleaseManifest, ModuleRuntime, ModuleSubject,
    PACKAGED_REVIEWED_CATALOG, PRIORITY_REVIEW_LABEL, PackagedReviewedRelease, ProcessSupervisor,
    RELEASE_FORMAT, RESPONSE_FORMAT, ResourceBudgets, ReviewedRelease, SignedEnvelope, WIT_PACKAGE,
    WIT_WORLD, WitIdentity, host_request, packaged_reviewed_release,
    packaged_reviewed_release_by_id, packaged_reviewed_releases, read_frame, reviewed_release,
    reviewed_release_for, sha256_hex, sign_active_admission, sign_active_admission_for,
    sign_active_admission_with_grants, sign_operator_custom_provenance, verify_host_request,
    verify_release_material, wit_sha256, write_frame,
};
use uuid::Uuid;

const SERVER_ID: Uuid = Uuid::from_u128(0x20000000000040008000000000000002);
const ADMISSION_ID: Uuid = Uuid::from_u128(0x30000000000040008000000000000003);
const EVENT_ID: Uuid = Uuid::from_u128(0x40000000000040008000000000000004);
const REPORT_ID: Uuid = Uuid::from_u128(0x50000000000040008000000000000005);

#[test]
fn production_release_is_exact_deterministic_and_separately_admitted() {
    let first = reviewed_release().expect("reviewed release should build");
    let second = reviewed_release().expect("reviewed release should be deterministic");
    assert_eq!(first.release, second.release);
    assert_eq!(first.provenance, second.provenance);
    assert_eq!(first.manifest.module_id, BUILTIN_MODULE_ID);
    assert_eq!(first.manifest.release_id, BUILTIN_RELEASE_ID);
    assert_eq!(first.manifest.wit.sha256, wit_sha256());
    assert_eq!(
        first.manifest.component_sha256,
        sha256_hex(FixtureKind::Valid.component_bytes())
    );
    assert_eq!(
        first.manifest.component_sha256,
        "74eb3f982cbda8448214899be25eb6fbe1708353502fcc0024727a7faffb78e0"
    );

    let core = core_key();
    let (admission, envelope) =
        sign_active_admission(&first, SERVER_ID, ADMISSION_ID, 1, 1, 0, &core)
            .expect("explicit admission should sign");
    assert_eq!(admission.granted_capabilities.len(), 1);
    assert_eq!(admission.subscribed_hooks.len(), 1);
    assert_eq!(envelope.document_format, ADMISSION_FORMAT);
    assert_ne!(first.release.payload, envelope.payload);
}

#[test]
fn packaged_catalog_is_bounded_exact_compatible_and_executable() {
    let catalog = packaged_reviewed_releases().expect("packaged catalog should build");
    assert_eq!(PACKAGED_REVIEWED_CATALOG.len(), 2);
    assert_eq!(catalog.len(), 2);
    let initial = &catalog[0];
    let successor = &catalog[1];
    assert_eq!(initial.manifest.release_id, BUILTIN_RELEASE_ID);
    assert_eq!(successor.manifest.release_id, BUILTIN_SUCCESSOR_RELEASE_ID);
    assert_eq!(successor.manifest.version, "1.1.0");
    assert_eq!(
        successor.manifest.state_schema,
        "ignibyte.sentinel.state/v2"
    );
    assert_eq!(
        successor.manifest.config_schema,
        initial.manifest.config_schema
    );
    assert_eq!(successor.manifest.wit, initial.manifest.wit);
    assert_eq!(
        successor.manifest.requested_capabilities,
        initial.manifest.requested_capabilities
    );
    assert_eq!(
        successor.manifest.subscribed_hooks,
        initial.manifest.subscribed_hooks
    );
    assert_eq!(successor.manifest.budgets, initial.manifest.budgets);
    assert_ne!(successor.manifest.release_id, initial.manifest.release_id);
    assert_ne!(
        successor.manifest.component_sha256,
        initial.manifest.component_sha256
    );
    assert_ne!(
        successor.provenance_statement.review_id,
        initial.provenance_statement.review_id
    );
    let repeated = packaged_reviewed_release(PackagedReviewedRelease::Successor)
        .expect("successor should be deterministic");
    assert_eq!(repeated.release, successor.release);
    assert_eq!(repeated.provenance, successor.provenance);
    assert_eq!(repeated.component_bytes, successor.component_bytes);
    let resolved = packaged_reviewed_release_by_id(BUILTIN_SUCCESSOR_RELEASE_ID)
        .expect("catalog lookup should succeed")
        .expect("successor should be present");
    assert_eq!(resolved.release, successor.release);
    assert_eq!(resolved.provenance, successor.provenance);
    assert_eq!(resolved.component_bytes, successor.component_bytes);
    assert!(
        packaged_reviewed_release_by_id(Uuid::new_v4())
            .expect("unknown lookup should be bounded")
            .is_none()
    );

    let core = core_key();
    let (_, admission) = sign_active_admission(successor, SERVER_ID, ADMISSION_ID, 2, 1, 1, &core)
        .expect("successor admission should sign");
    let request = host_request(
        successor,
        admission,
        ModuleHookEvent {
            format: HOOK_FORMAT.into(),
            event_id: EVENT_ID,
            attempt: 1,
            server_id: SERVER_ID,
            module_id: BUILTIN_MODULE_ID.into(),
            release_id: BUILTIN_SUCCESSOR_RELEASE_ID,
            admission_id: ADMISSION_ID,
            admission_revision: 2,
            hook: HookKind::PersonaReported,
            causal_revision: 0,
            deadline_ms: MAX_EXECUTION_MS,
            subject: ModuleSubject::Pairwise("pairwise-successor-persona-7".into()),
            config: BTreeMap::from([("policy".into(), "strict".into())]),
            config_revision: 1,
            state: BTreeMap::from([("schema".into(), "v2".into())]),
            state_revision: 1,
            payload: HookPayload::PersonaReported {
                report_id: REPORT_ID,
                category: "cheating".into(),
            },
        },
    );
    let runtime = ModuleRuntime::compile_bytes(&successor.component_bytes)
        .expect("successor should compile under the shared runtime");
    runtime.readiness().expect("successor WIT should be ready");
    assert!(matches!(
        runtime
            .execute_release(&request, &core.verifying_key(), successor)
            .outcome,
        HostResult::Proposed { .. }
    ));
}

#[test]
fn exact_request_executes_valid_noop_and_bounded_rejection_components() {
    let core = core_key();
    for (kind, expected) in [
        (FixtureKind::Valid, "proposed"),
        (FixtureKind::Noop, "noop"),
        (FixtureKind::Unauthorized, "rejected"),
        (FixtureKind::Trap, "rejected"),
        (FixtureKind::Loop, "rejected"),
    ] {
        let request = request(kind, &core);
        let runtime = ModuleRuntime::compile(kind).expect("fixture should compile");
        runtime.readiness().expect("fixture WIT should be ready");
        let response = runtime.execute(&request, &core.verifying_key());
        assert_eq!(response.format, RESPONSE_FORMAT);
        match (expected, response.outcome) {
            (
                "proposed",
                HostResult::Proposed {
                    intent:
                        omarchygs_server_module_runtime::ModuleIntent::ModerationAddLabel {
                            expected_revision,
                            label,
                        },
                },
            ) => {
                assert_eq!(expected_revision, 0);
                assert_eq!(label, PRIORITY_REVIEW_LABEL);
            }
            ("noop", HostResult::Noop) => {}
            ("rejected", HostResult::Rejected { code }) => {
                assert!(
                    matches!(
                        code.as_str(),
                        "intent_outside_policy" | "module_execution_failed"
                    ),
                    "unexpected rejection: {code}"
                );
            }
            (expected, actual) => panic!("expected {expected}, got {actual:?}"),
        }
    }
}

#[test]
fn operator_custom_release_uses_the_same_runtime_and_distinct_trust_claims() {
    let core = core_key();
    let reviewed = reviewed_release().expect("reviewed release should build");
    let reviewed_request = request(FixtureKind::Valid, &core);
    let runtime = ModuleRuntime::compile_bytes(FixtureKind::Valid.component_bytes())
        .expect("shared component should compile");
    runtime.readiness().expect("shared WIT should be ready");
    assert!(matches!(
        runtime
            .execute_release(&reviewed_request, &core.verifying_key(), &reviewed)
            .outcome,
        HostResult::Proposed { .. }
    ));

    let (custom, custom_request) = custom_request(&core);
    assert_eq!(custom.provenance_statement.class, "operator_custom");
    assert_eq!(custom.provenance_statement.review_id, None);
    assert_eq!(custom.provenance_statement.server_id, Some(SERVER_ID));
    assert!(matches!(
        runtime
            .execute_release(&custom_request, &core.verifying_key(), &custom)
            .outcome,
        HostResult::Proposed { .. }
    ));

    let wrong_server = Uuid::from_u128(0x90000000000040008000000000000009);
    let mut wrong_server_trust = custom.execution_trust();
    wrong_server_trust.provenance_server_id = Some(wrong_server);
    assert!(
        verify_release_material(
            custom.release.clone(),
            custom.provenance.clone(),
            &wrong_server_trust,
            custom.component_bytes.clone(),
        )
        .is_err()
    );
    let mut tampered = custom.component_bytes.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert!(
        verify_release_material(
            custom.release.clone(),
            custom.provenance.clone(),
            &custom.execution_trust(),
            tampered,
        )
        .is_err()
    );
}

#[test]
fn wrong_component_interface_import_memory_and_authority_are_rejected() {
    for kind in [
        FixtureKind::MemoryHog,
        FixtureKind::ForbiddenImport,
        FixtureKind::WrongInterface,
    ] {
        let runtime = ModuleRuntime::compile(kind);
        assert!(
            runtime.is_err() || runtime.and_then(|runtime| runtime.readiness()).is_err(),
            "{kind:?} must fail compile/readiness"
        );
    }

    let core = core_key();
    let request = request(FixtureKind::Valid, &core);
    let wrong_core = SigningKey::from_bytes(&[41; 32]);
    assert!(
        verify_host_request(&request, &wrong_core.verifying_key(), FixtureKind::Valid).is_err()
    );

    let mut tampered = request;
    tampered.event.module_id = "attacker.module".into();
    assert!(verify_host_request(&tampered, &core.verifying_key(), FixtureKind::Valid).is_err());
}

#[test]
fn frames_reject_oversize_noncanonical_and_changed_schema() {
    let core = core_key();
    let request = request(FixtureKind::Valid, &core);
    let mut encoded = Vec::new();
    write_frame(&mut encoded, &request).expect("valid request should frame");
    let decoded: HostRequest =
        read_frame(&mut Cursor::new(&encoded)).expect("valid frame should decode");
    assert_eq!(decoded, request);

    let mut oversized = Cursor::new(((MAX_FRAME_BYTES as u32) + 1).to_be_bytes().to_vec());
    assert!(read_frame::<HostRequest, _>(&mut oversized).is_err());

    let body = serde_json::to_vec(&request).expect("request should serialize");
    let pretty = serde_json::to_vec_pretty(&request).expect("request should pretty serialize");
    assert_ne!(body, pretty);
    let mut noncanonical = Vec::new();
    noncanonical.extend_from_slice(&(pretty.len() as u32).to_be_bytes());
    noncanonical.extend_from_slice(&pretty);
    assert!(read_frame::<HostRequest, _>(&mut Cursor::new(noncanonical)).is_err());

    let mut document = serde_json::to_value(&request).expect("request should be JSON");
    document
        .as_object_mut()
        .expect("request should be an object")
        .insert("unknown".into(), serde_json::json!(true));
    let changed = serde_json::to_vec(&document).expect("changed request should encode");
    let mut frame = Vec::new();
    frame.extend_from_slice(&(changed.len() as u32).to_be_bytes());
    frame.extend_from_slice(&changed);
    assert!(read_frame::<HostRequest, _>(&mut Cursor::new(frame)).is_err());
}

#[test]
#[ignore = "requires built host binary, systemd user scope, and Bubblewrap; run scripts/test-server-modules.sh"]
fn real_process_is_contained_and_recovers_after_failure() {
    let host = std::env::var("OGS_MODULE_HOST_TEST_BINARY")
        .expect("test script must provide the reviewed host binary");
    let supervisor =
        ProcessSupervisor::reviewed_path(Path::new(&host)).expect("host path should resolve");
    let core = core_key();
    let valid = request(FixtureKind::Valid, &core);
    let report = supervisor
        .execute(&valid, &core.verifying_key())
        .expect("valid host flow should complete");
    assert_eq!(report.containment, "systemd-user-scope+bubblewrap+prlimit");
    assert!(report.ready.server_environment_absent);
    assert!(report.ready.loopback_only);
    assert!(matches!(
        report.response.outcome,
        HostResult::Proposed { .. }
    ));

    let (custom, custom_request) = custom_request(&core);
    let custom_report = supervisor
        .execute_release(&custom_request, &core.verifying_key(), &custom)
        .expect("operator-custom artifact should use the same containment boundary");
    assert_eq!(
        custom_report.containment,
        "systemd-user-scope+bubblewrap+prlimit"
    );
    assert!(custom_report.ready.server_environment_absent);
    assert!(custom_report.ready.loopback_only);
    assert!(matches!(
        custom_report.response.outcome,
        HostResult::Proposed { .. }
    ));

    for (kind, failure) in [
        (FixtureKind::Trap, None),
        (FixtureKind::Loop, None),
        (FixtureKind::Valid, Some("exit")),
        (FixtureKind::Valid, Some("hang")),
    ] {
        let request = request(kind, &core);
        let outcome = supervisor.execute_fixture(&request, &core.verifying_key(), kind, failure);
        if failure.is_some() {
            assert!(outcome.is_err());
        } else {
            assert!(matches!(
                outcome
                    .expect("runtime fault should be a bounded response")
                    .response
                    .outcome,
                HostResult::Rejected { .. }
            ));
        }
    }

    let restarted = supervisor
        .execute(&valid, &core.verifying_key())
        .expect("fresh host should work after failures");
    assert!(matches!(
        restarted.response.outcome,
        HostResult::Proposed { .. }
    ));
}

fn request(kind: FixtureKind, core: &SigningKey) -> HostRequest {
    let reviewed = reviewed_release_for(kind).expect("fixture release should build");
    let (_, admission) = sign_active_admission_for(
        &reviewed,
        kind,
        AdmissionCoordinates {
            server_id: SERVER_ID,
            admission_id: ADMISSION_ID,
            lifecycle_revision: 1,
            config_revision: 1,
            state_revision: 0,
        },
        core,
    )
    .expect("fixture admission should sign");
    host_request(
        &reviewed,
        admission,
        ModuleHookEvent {
            format: HOOK_FORMAT.into(),
            event_id: EVENT_ID,
            attempt: 1,
            server_id: SERVER_ID,
            module_id: BUILTIN_MODULE_ID.into(),
            release_id: BUILTIN_RELEASE_ID,
            admission_id: ADMISSION_ID,
            admission_revision: 1,
            hook: HookKind::PersonaReported,
            causal_revision: 0,
            deadline_ms: 500,
            subject: ModuleSubject::Pairwise("pairwise-persona-7".into()),
            config: BTreeMap::from([("policy".into(), "strict".into())]),
            config_revision: 1,
            state: BTreeMap::new(),
            state_revision: 0,
            payload: HookPayload::PersonaReported {
                report_id: REPORT_ID,
                category: "cheating".into(),
            },
        },
    )
}

fn core_key() -> SigningKey {
    SigningKey::from_bytes(&[11; 32])
}

fn custom_request(core: &SigningKey) -> (ReviewedRelease, HostRequest) {
    let publisher = SigningKey::from_bytes(&[31; 32]);
    let operator = SigningKey::from_bytes(&[41; 32]);
    let release_id = Uuid::from_u128(0x60000000000040008000000000000006);
    let admission_id = Uuid::from_u128(0x70000000000040008000000000000007);
    let manifest = ModuleReleaseManifest {
        format: RELEASE_FORMAT.into(),
        module_id: "community.report-helper".into(),
        publisher_id: "community".into(),
        release_id,
        version: "1.0.0".into(),
        component_sha256: sha256_hex(FixtureKind::Valid.component_bytes()),
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
        config_schema: "community.report-helper.config/v1".into(),
        state_schema: "community.report-helper.state/v1".into(),
        entrypoint: "handle".into(),
    };
    let release = SignedEnvelope::sign(
        RELEASE_FORMAT,
        "community-publisher-v1",
        &manifest,
        &publisher,
    )
    .expect("custom release should sign");
    let (_, provenance) =
        sign_operator_custom_provenance(&release, SERVER_ID, "server-custom-root-v1", &operator)
            .expect("custom provenance should sign");
    let reviewed = verify_release_material(
        release,
        provenance,
        &ExecutionTrust {
            publisher_key_id: "community-publisher-v1".into(),
            publisher_public_key: publisher.verifying_key(),
            provenance_key_id: "server-custom-root-v1".into(),
            provenance_public_key: operator.verifying_key(),
            provenance_class: "operator_custom".into(),
            provenance_server_id: Some(SERVER_ID),
        },
        FixtureKind::Valid.component_bytes().to_vec(),
    )
    .expect("custom release should verify");
    let (_, admission) = sign_active_admission_with_grants(
        &reviewed,
        AdmissionCoordinates {
            server_id: SERVER_ID,
            admission_id,
            lifecycle_revision: 1,
            config_revision: 1,
            state_revision: 0,
        },
        vec![Capability::ModerationAddLabel],
        vec![HookKind::PersonaReported],
        core,
    )
    .expect("custom admission should sign");
    let request = host_request(
        &reviewed,
        admission,
        ModuleHookEvent {
            format: HOOK_FORMAT.into(),
            event_id: Uuid::from_u128(0x80000000000040008000000000000008),
            attempt: 1,
            server_id: SERVER_ID,
            module_id: manifest.module_id,
            release_id,
            admission_id,
            admission_revision: 1,
            hook: HookKind::PersonaReported,
            causal_revision: 0,
            deadline_ms: MAX_EXECUTION_MS,
            subject: ModuleSubject::Pairwise("pairwise-custom-persona-7".into()),
            config: BTreeMap::from([("policy".into(), "strict".into())]),
            config_revision: 1,
            state: BTreeMap::new(),
            state_revision: 0,
            payload: HookPayload::PersonaReported {
                report_id: REPORT_ID,
                category: "cheating".into(),
            },
        },
    );
    (reviewed, request)
}
