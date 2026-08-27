use std::io::Cursor;
use std::sync::OnceLock;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use omarchygs_server_module_spike::{
    ADMISSION_FORMAT, HostReady, HostResponse, HostResult, MAX_ARTIFACT_BYTES, MAX_FRAME_BYTES,
    ModuleAdmission, ModuleProvenance, ModuleReleaseManifest, PROVENANCE_FORMAT, ProofCore,
    ProofError, ProvenanceClass, RELEASE_FORMAT, RESPONSE_FORMAT, SignedEnvelope,
    build_fixture_component, fixture_request, read_bounded_artifact, read_frame,
    verify_host_request, write_frame,
};
use uuid::Uuid;

const VALID_SOURCE: &[u8] = include_bytes!("../fixtures/components/valid.wat");

fn valid() -> &'static [u8] {
    static COMPONENT: OnceLock<Vec<u8>> = OnceLock::new();
    COMPONENT
        .get_or_init(|| build_fixture_component(VALID_SOURCE).expect("valid component builds"))
        .as_slice()
}

fn operator_provenance() -> ProvenanceClass {
    ProvenanceClass::OperatorCustom {
        server_id: Uuid::parse_str("20000000-0000-4000-8000-000000000002")
            .expect("fixture UUID is valid"),
    }
}

#[test]
fn separate_provenance_classes_share_one_runtime_contract() {
    let operator = fixture_request(valid(), operator_provenance()).expect("operator fixture");
    let marketplace = fixture_request(
        valid(),
        ProvenanceClass::MarketplaceVetted {
            review_id: Uuid::parse_str("60000000-0000-4000-8000-000000000006")
                .expect("fixture UUID is valid"),
        },
    )
    .expect("marketplace fixture");

    let operator_verified = verify_host_request(&operator, valid()).expect("operator verifies");
    let marketplace_verified =
        verify_host_request(&marketplace, valid()).expect("marketplace verifies");
    assert_eq!(
        operator_verified.admission.granted_capabilities,
        marketplace_verified.admission.granted_capabilities
    );
    assert_eq!(
        operator_verified.admission.budgets,
        marketplace_verified.admission.budgets
    );
}

#[test]
fn tampered_component_signature_and_context_are_rejected() {
    let mut request = fixture_request(valid(), operator_provenance()).expect("fixture");
    assert!(matches!(
        verify_host_request(&request, b"changed"),
        Err(ProofError::Integrity(_))
    ));

    request.release.signature.replace_range(0..1, "A");
    assert!(matches!(
        verify_host_request(&request, valid()),
        Err(ProofError::Integrity(_)) | Err(ProofError::Contract(_))
    ));

    let mut request = fixture_request(valid(), operator_provenance()).expect("fixture");
    request.event.module_id = "attacker.module".into();
    assert!(matches!(
        verify_host_request(&request, valid()),
        Err(ProofError::Integrity(_))
    ));

    let trusted_publisher = SigningKey::from_bytes(&[7_u8; 32]);
    let attacker = SigningKey::from_bytes(&[13_u8; 32]);
    let mut request = fixture_request(valid(), operator_provenance()).expect("fixture");
    let manifest: ModuleReleaseManifest = request
        .release
        .verify(RELEASE_FORMAT, &trusted_publisher.verifying_key())
        .expect("trusted release verifies");
    request.release =
        SignedEnvelope::sign(RELEASE_FORMAT, "publisher-ignibyte-1", &manifest, &attacker)
            .expect("attacker can self-sign");
    request.publisher_public_key = URL_SAFE_NO_PAD.encode(attacker.verifying_key().as_bytes());
    assert!(matches!(
        verify_host_request(&request, valid()),
        Err(ProofError::Integrity(_))
    ));
}

#[test]
fn operator_custom_provenance_is_bound_to_the_admitted_server() {
    let mut request = fixture_request(valid(), operator_provenance()).expect("fixture");
    let provenance_key = SigningKey::from_bytes(&[9_u8; 32]);
    let mut provenance: ModuleProvenance = request
        .provenance
        .verify(PROVENANCE_FORMAT, &provenance_key.verifying_key())
        .expect("trusted provenance verifies");
    provenance.provenance = ProvenanceClass::OperatorCustom {
        server_id: Uuid::parse_str("70000000-0000-4000-8000-000000000007").expect("fixture UUID"),
    };
    request.provenance = SignedEnvelope::sign(
        PROVENANCE_FORMAT,
        "provenance-authority-1",
        &provenance,
        &provenance_key,
    )
    .expect("provenance re-signs");

    let core_key = SigningKey::from_bytes(&[11_u8; 32]);
    let mut admission: ModuleAdmission = request
        .admission
        .verify(ADMISSION_FORMAT, &core_key.verifying_key())
        .expect("trusted admission verifies");
    admission.provenance_sha256 = request
        .provenance
        .payload_sha256()
        .expect("provenance hash");
    request.admission =
        SignedEnvelope::sign(ADMISSION_FORMAT, "server-core-1", &admission, &core_key)
            .expect("admission re-signs");

    assert!(matches!(
        verify_host_request(&request, valid()),
        Err(ProofError::Integrity(_))
    ));
}

#[test]
fn duplicate_capabilities_and_wit_downgrade_are_rejected_even_when_resigned() {
    let request = fixture_request(valid(), operator_provenance()).expect("fixture");
    let publisher = SigningKey::from_bytes(&[7_u8; 32]);
    let mut manifest: ModuleReleaseManifest = request
        .release
        .verify(RELEASE_FORMAT, &publisher.verifying_key())
        .expect("release verifies");
    manifest
        .requested_capabilities
        .push(manifest.requested_capabilities[0]);
    manifest
        .validate()
        .expect_err("duplicate capability rejected");
    manifest.requested_capabilities.pop();
    manifest.wit.major = 0;
    manifest.validate().expect_err("WIT downgrade rejected");
}

#[test]
fn strict_signed_payload_rejects_unknown_fields() {
    let key = SigningKey::from_bytes(&[7_u8; 32]);
    let unknown = serde_json::json!({
        "format": RELEASE_FORMAT,
        "unknown": true
    });
    let envelope = SignedEnvelope::sign(RELEASE_FORMAT, "publisher-1", &unknown, &key)
        .expect("generic document signs");
    let decoded = envelope.verify::<ModuleReleaseManifest>(RELEASE_FORMAT, &key.verifying_key());
    assert!(matches!(decoded, Err(ProofError::Contract(_))));
}

#[test]
fn frame_reader_rejects_oversized_malformed_and_noncanonical_input() {
    let mut oversized = Cursor::new(((MAX_FRAME_BYTES as u32) + 1).to_be_bytes().to_vec());
    assert!(matches!(
        read_frame::<HostReady, _>(&mut oversized),
        Err(ProofError::Frame(_))
    ));

    let malformed_payload = b"{";
    let mut malformed = (malformed_payload.len() as u32).to_be_bytes().to_vec();
    malformed.extend_from_slice(malformed_payload);
    assert!(matches!(
        read_frame::<HostReady, _>(&mut Cursor::new(malformed)),
        Err(ProofError::Frame(_))
    ));

    let noncanonical_payload = br#"{ "format": "x", "component_ready": true, "home_absent": true, "passwd_absent": true, "server_environment_absent": true, "loopback_only": true }"#;
    let mut noncanonical = (noncanonical_payload.len() as u32).to_be_bytes().to_vec();
    noncanonical.extend_from_slice(noncanonical_payload);
    assert!(matches!(
        read_frame::<HostReady, _>(&mut Cursor::new(noncanonical)),
        Err(ProofError::Frame(_))
    ));
}

#[test]
fn frame_reader_rejects_unknown_hook_and_writer_rejects_oversized_response() {
    let request = fixture_request(valid(), operator_provenance()).expect("fixture");
    let mut value = serde_json::to_value(request).expect("request value");
    value["event"]["hook"] = serde_json::Value::String("future_unknown_hook".into());
    let payload = serde_json::to_vec(&value).expect("payload");
    let mut frame = (payload.len() as u32).to_be_bytes().to_vec();
    frame.extend_from_slice(&payload);
    assert!(matches!(
        read_frame::<omarchygs_server_module_spike::HostRequest, _>(&mut Cursor::new(frame)),
        Err(ProofError::Frame(_))
    ));

    let response = HostResponse {
        format: RESPONSE_FORMAT.into(),
        event_id: Uuid::new_v4(),
        release_id: Uuid::new_v4(),
        admission_id: Uuid::new_v4(),
        outcome: HostResult::Rejected {
            code: "x".repeat(MAX_FRAME_BYTES),
        },
    };
    assert!(matches!(
        write_frame(&mut Vec::new(), &response),
        Err(ProofError::Frame(_))
    ));
}

#[test]
fn frame_round_trip_is_exact_and_sensitive_fields_are_absent() {
    let request = fixture_request(valid(), operator_provenance()).expect("fixture");
    let mut encoded = Vec::new();
    write_frame(&mut encoded, &request).expect("frame writes");
    let decoded = read_frame(&mut Cursor::new(encoded)).expect("frame reads");
    verify_host_request(&decoded, valid()).expect("decoded request verifies");

    let json = serde_json::to_string(&request).expect("serialize request");
    for forbidden in [
        "account_id",
        "password",
        "session_token",
        "database_url",
        "destination_url",
    ] {
        assert!(!json.to_ascii_lowercase().contains(forbidden));
    }
}

#[test]
fn core_reauthorizes_exact_context_and_replays_only_identical_requests() {
    let request = fixture_request(valid(), operator_provenance()).expect("fixture");
    let response = HostResponse {
        format: RESPONSE_FORMAT.into(),
        event_id: request.event.event_id,
        release_id: request.event.release_id,
        admission_id: request.event.admission_id,
        outcome: HostResult::Proposed {
            intent: omarchygs_server_module_spike::ModuleIntent::ModerationAddLabel {
                expected_revision: 42,
                label: 7,
            },
        },
    };
    let mut core = ProofCore::at_revision(42);
    let first = core
        .apply(&request, &response, valid())
        .expect("first commit");
    let replay = core
        .apply(&request, &response, valid())
        .expect("exact replay returns receipt");
    assert_eq!(first, replay);
    assert_eq!(core.labels(), [7]);

    let mut changed = request.clone();
    changed.event.attempt = 2;
    assert!(matches!(
        core.apply(&changed, &response, valid()),
        Err(ProofError::ReplayConflict)
    ));

    let mut forged = response;
    forged.event_id = Uuid::new_v4();
    let mut fresh_core = ProofCore::at_revision(42);
    assert!(matches!(
        fresh_core.apply(&request, &forged, valid()),
        Err(ProofError::Integrity(_))
    ));
}

#[test]
fn public_key_encoding_is_canonical_base64url() {
    let request = fixture_request(valid(), operator_provenance()).expect("fixture");
    let bytes = URL_SAFE_NO_PAD
        .decode(&request.publisher_public_key)
        .expect("public key decodes");
    assert_eq!(bytes.len(), 32);
    assert_eq!(URL_SAFE_NO_PAD.encode(bytes), request.publisher_public_key);
}

#[test]
fn component_file_reader_enforces_the_artifact_limit_before_compilation() {
    let file = tempfile::NamedTempFile::new().expect("temporary component");
    file.as_file()
        .set_len((MAX_ARTIFACT_BYTES + 1) as u64)
        .expect("oversized sparse component");
    assert!(matches!(
        read_bounded_artifact(file.path()),
        Err(ProofError::Contract(_))
    ));
}
