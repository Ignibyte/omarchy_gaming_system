use std::{fs, path::Path};

use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CatalogStatus, OPERATOR_CUSTOM_WARNING, OperatorCustomAcquisition,
    RELEASE_ARCHIVE_PATH, RELEASE_ATTESTATION_PATH, RELEASE_CONFORMANCE_PATH, create_release,
    export_sdk, generate_catalog_keypair, generate_keypair, operator_custom_key_sha256,
    rich_2d_host_profile, sign_catalog_policy, sign_operator_custom_release,
    signed_operator_custom_release_bytes, supported_sdk_identity,
    verify_operator_custom_acquisition_bytes, verify_release_directory,
};

const REVISION: &str = "2222222222222222222222222222222222222222";
const BUILDER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn operator_custom_acquisition_is_deterministic_distinct_and_fully_verified() {
    let fixture = fixture();
    let first = sign_operator_custom_release(
        &fixture.verified,
        &fixture.publisher_public,
        &fixture.operator_private,
        &fixture.admission.server_id,
        "Test Community Operator",
    )
    .unwrap();
    let second = sign_operator_custom_release(
        &fixture.verified,
        &fixture.publisher_public,
        &fixture.operator_private,
        &fixture.admission.server_id,
        "Test Community Operator",
    )
    .unwrap();
    assert_eq!(first, second);
    let signed = signed_operator_custom_release_bytes(&first).unwrap();
    let document = acquisition(&fixture, &signed);
    let bytes = document.to_bounded_json().unwrap();
    let verified = verify_operator_custom_acquisition_bytes(
        &bytes,
        &fixture.admission,
        &fixture.operator_public,
        &supported_sdk_identity().unwrap(),
        &rich_2d_host_profile(),
    )
    .unwrap();
    assert_eq!(verified.attestation().warning, OPERATOR_CUSTOM_WARNING);
    assert_eq!(
        verified.attestation().operator_name,
        "Test Community Operator"
    );
    assert_eq!(verified.policy().policy_version, 1);
    assert_eq!(verified.release().payload(), fixture.verified.payload());
    assert_eq!(
        verified.attestation().operator_key_sha256,
        operator_custom_key_sha256(&fixture.operator_public).unwrap()
    );

    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("marketplace_key").is_none());
    assert!(json.get("signed_marketplace_snapshot").is_none());
    assert!(!String::from_utf8(bytes).unwrap().contains("reviewed_by"));
}

#[test]
fn operator_custom_acquisition_rejects_substitution_tamper_and_fake_review_fields() {
    let fixture = fixture();
    let signed = signed_operator_custom_release_bytes(
        &sign_operator_custom_release(
            &fixture.verified,
            &fixture.publisher_public,
            &fixture.operator_private,
            &fixture.admission.server_id,
            "Test Community Operator",
        )
        .unwrap(),
    )
    .unwrap();
    let bytes = acquisition(&fixture, &signed).to_bounded_json().unwrap();
    let sdk = supported_sdk_identity().unwrap();

    let (_, wrong_key) = generate_catalog_keypair("custom-v1", "test-community-custom").unwrap();
    assert_ne!(wrong_key, fixture.operator_public);
    assert!(
        verify_operator_custom_acquisition_bytes(
            &bytes,
            &fixture.admission,
            &wrong_key,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err()
    );

    let mut wrong_server = fixture.admission.clone();
    wrong_server.server_id = "33333333-3333-4333-8333-333333333333".to_owned();
    assert!(
        verify_operator_custom_acquisition_bytes(
            &bytes,
            &wrong_server,
            &fixture.operator_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err()
    );

    let mut tampered: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let archive = tampered["archive"].as_str().unwrap().to_owned();
    tampered["archive"] = serde_json::Value::String(format!("A{}", &archive[1..]));
    assert!(
        verify_operator_custom_acquisition_bytes(
            &serde_json::to_vec(&tampered).unwrap(),
            &fixture.admission,
            &fixture.operator_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err()
    );

    let mut fake_review: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    fake_review["reviewed_by"] = serde_json::json!("marketplace");
    assert!(
        verify_operator_custom_acquisition_bytes(
            &serde_json::to_vec(&fake_review).unwrap(),
            &fixture.admission,
            &fixture.operator_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err()
    );
}

struct Fixture {
    _temp: tempfile::TempDir,
    release: std::path::PathBuf,
    publisher_public: omarchygs_game_cartridge::PublisherPublicKey,
    operator_private: omarchygs_game_cartridge::CatalogPrivateKey,
    operator_public: omarchygs_game_cartridge::CatalogPublicKey,
    verified: omarchygs_game_cartridge::VerifiedRelease,
    policy: Vec<u8>,
    admission: AcquisitionServerAdmission,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let sdk = temp.path().join("sdk");
    let release = temp.path().join("release");
    fs::create_dir(&sdk).unwrap();
    fs::create_dir(&release).unwrap();
    export_sdk(&sdk).unwrap();
    let (publisher_private, publisher_public) =
        generate_keypair("publisher-custom-v1", "ignibyte").unwrap();
    create_release(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/first-party-door-legends/cartridge"),
        &publisher_private,
        &sdk,
        REVISION,
        BUILDER_DIGEST,
        &rich_2d_host_profile(),
        &release,
    )
    .unwrap();
    let verified =
        verify_release_directory(&release, &publisher_public, &sdk, &rich_2d_host_profile())
            .unwrap();
    let (operator_private, operator_public) =
        generate_catalog_keypair("custom-v1", "test-community-custom").unwrap();
    let policy = serde_json::to_vec(
        &sign_catalog_policy(
            &verified,
            &operator_private,
            1,
            CatalogStatus::Active,
            "Enabled by this server operator as unvetted custom content.",
        )
        .unwrap(),
    )
    .unwrap();
    let admission = AcquisitionServerAdmission {
        server_id: "11111111-1111-4111-8111-111111111111".to_owned(),
        game_key: verified.payload().game_key.clone(),
        publisher_id: verified.payload().publisher_id.clone(),
        rules_version: verified.payload().rules_version,
        cartridge_version: verified.payload().cartridge_version,
        archive_sha256: verified.payload().archive_sha256.clone(),
        signed_identity_sha256: verified.payload().signed_identity_sha256.clone(),
        admission_revision: 1,
    };
    Fixture {
        _temp: temp,
        release,
        publisher_public,
        operator_private,
        operator_public,
        verified,
        policy,
        admission,
    }
}

fn acquisition(fixture: &Fixture, signed: &[u8]) -> OperatorCustomAcquisition {
    OperatorCustomAcquisition::from_verified_bytes(
        fixture.admission.clone(),
        fixture.operator_public.clone(),
        signed,
        &fixture.policy,
        &fs::read(fixture.release.join(RELEASE_ARCHIVE_PATH)).unwrap(),
        &fs::read(fixture.release.join(RELEASE_CONFORMANCE_PATH)).unwrap(),
        &fs::read(fixture.release.join(RELEASE_ATTESTATION_PATH)).unwrap(),
    )
    .unwrap()
}
