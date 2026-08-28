use std::{collections::BTreeMap, io::Cursor, path::Path};

use ed25519_dalek::SigningKey;
use omarchygs_server_module_runtime::{
    ADMISSION_FORMAT, AdmissionCoordinates, BUILTIN_MODULE_ID, BUILTIN_RELEASE_ID, FixtureKind,
    HOOK_FORMAT, HookKind, HookPayload, HostRequest, HostResult, MAX_FRAME_BYTES, ModuleHookEvent,
    ModuleRuntime, ModuleSubject, PRIORITY_REVIEW_LABEL, ProcessSupervisor, RESPONSE_FORMAT,
    host_request, read_frame, reviewed_release, reviewed_release_for, sha256_hex,
    sign_active_admission, sign_active_admission_for, verify_host_request, wit_sha256, write_frame,
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
