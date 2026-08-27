use omarchygs_server_module_spike::{
    HostResult, MAX_ARTIFACT_BYTES, ModuleRuntime, ProofError, ProvenanceClass,
    build_fixture_component, fixture_request,
};
use uuid::Uuid;

const VALID: &[u8] = include_bytes!("../fixtures/components/valid.wat");
const NOOP: &[u8] = include_bytes!("../fixtures/components/noop.wat");
const UNAUTHORIZED: &[u8] = include_bytes!("../fixtures/components/unauthorized.wat");
const TRAP: &[u8] = include_bytes!("../fixtures/components/trap.wat");
const LOOP: &[u8] = include_bytes!("../fixtures/components/loop.wat");
const MEMORY_HOG: &[u8] = include_bytes!("../fixtures/components/memory-hog.wat");
const FORBIDDEN_IMPORT: &[u8] = include_bytes!("../fixtures/components/forbidden-import.wat");
const WRONG_INTERFACE: &[u8] = include_bytes!("../fixtures/components/wrong-interface.wat");

fn provenance() -> ProvenanceClass {
    ProvenanceClass::OperatorCustom {
        server_id: Uuid::parse_str("20000000-0000-4000-8000-000000000002").expect("fixture UUID"),
    }
}

fn component(source: &[u8]) -> Vec<u8> {
    build_fixture_component(source).expect("fixture component builds")
}

#[test]
fn exact_typed_component_proposes_one_allowlisted_intent() {
    let valid = component(VALID);
    let runtime = ModuleRuntime::compile(&valid).expect("component compiles");
    runtime.readiness().expect("component is ready");
    let request = fixture_request(&valid, provenance()).expect("fixture");
    let response = runtime.execute(&request);
    assert!(matches!(response.outcome, HostResult::Proposed { .. }));
}

#[test]
fn no_op_and_undeclared_capability_are_distinct() {
    let noop_component = component(NOOP);
    let noop = ModuleRuntime::compile(&noop_component).expect("noop compiles");
    let noop_response =
        noop.execute(&fixture_request(&noop_component, provenance()).expect("fixture"));
    assert_eq!(noop_response.outcome, HostResult::Noop);

    let unauthorized_component = component(UNAUTHORIZED);
    let unauthorized = ModuleRuntime::compile(&unauthorized_component).expect("component compiles");
    let unauthorized_response = unauthorized
        .execute(&fixture_request(&unauthorized_component, provenance()).expect("fixture"));
    assert_eq!(
        unauthorized_response.outcome,
        HostResult::Rejected {
            code: "intent_not_granted".into()
        }
    );
}

#[test]
fn traps_and_infinite_loops_fail_with_one_stable_code() {
    for source in [TRAP, LOOP] {
        let component = component(source);
        let runtime = ModuleRuntime::compile(&component).expect("component compiles");
        let response =
            runtime.execute(&fixture_request(&component, provenance()).expect("fixture"));
        assert_eq!(
            response.outcome,
            HostResult::Rejected {
                code: "module_execution_failed".into()
            }
        );
    }
}

#[test]
fn forbidden_import_memory_request_and_wrong_world_fail_readiness() {
    for source in [FORBIDDEN_IMPORT, MEMORY_HOG, WRONG_INTERFACE] {
        let component = component(source);
        let runtime = ModuleRuntime::compile(&component).expect("component syntax compiles");
        runtime
            .readiness()
            .expect_err("hostile component must fail readiness");
    }
}

#[test]
fn component_tamper_is_rejected_before_execution() {
    let valid = component(VALID);
    let runtime = ModuleRuntime::compile(&valid).expect("component compiles");
    let request = fixture_request(b"different component", provenance()).expect("fixture");
    assert_eq!(
        runtime.execute(&request).outcome,
        HostResult::Rejected {
            code: "request_rejected".into()
        }
    );
}

#[test]
fn runtime_rejects_text_and_oversized_artifacts_before_compilation() {
    assert!(matches!(
        ModuleRuntime::compile(VALID),
        Err(ProofError::Contract(_))
    ));
    assert!(matches!(
        ModuleRuntime::compile(&vec![0_u8; MAX_ARTIFACT_BYTES + 1]),
        Err(ProofError::Contract(_))
    ));
}
