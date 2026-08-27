use std::{fs, path::Path};

use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CartridgeAcquisition, CatalogStatus, MarketplaceReleaseEntry,
    MarketplaceSnapshotPayload, RELEASE_ARCHIVE_PATH, RELEASE_ATTESTATION_PATH,
    RELEASE_CONFORMANCE_PATH, create_release, export_sdk, generate_catalog_keypair,
    generate_keypair, rich_2d_host_profile, sign_catalog_policy, sign_marketplace_snapshot,
    supported_sdk_identity, verify_acquisition_bytes, verify_acquisition_bytes_with_policy_key,
    verify_release_directory,
};

const REVISION: &str = "1111111111111111111111111111111111111111";
const BUILDER_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn acquisition_verifies_every_exact_claim_and_rejects_tamper() {
    let fixture = fixture();
    let document = CartridgeAcquisition::from_verified_bytes(
        fixture.admission.clone(),
        fixture.marketplace_public.clone(),
        &fixture.snapshot,
        &fs::read(fixture.release.join(RELEASE_ARCHIVE_PATH)).unwrap(),
        &fs::read(fixture.release.join(RELEASE_CONFORMANCE_PATH)).unwrap(),
        &fs::read(fixture.release.join(RELEASE_ATTESTATION_PATH)).unwrap(),
    )
    .unwrap();
    let bytes = document.to_bounded_json().unwrap();
    let sdk = supported_sdk_identity().unwrap();
    let verified = verify_acquisition_bytes(
        &bytes,
        &fixture.admission,
        &fixture.marketplace_public,
        &sdk,
        &rich_2d_host_profile(),
    )
    .unwrap();
    assert_eq!(verified.entry().reviewed_by, "review-team");
    assert_eq!(verified.policy().policy_version, 1);

    let mut wrong = fixture.admission.clone();
    wrong.admission_revision += 1;
    assert!(
        verify_acquisition_bytes(
            &bytes,
            &wrong,
            &fixture.marketplace_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err()
    );

    let (_, substituted_marketplace_key) =
        generate_catalog_keypair("marketplace-primary-v1", "marketplace").unwrap();
    assert_ne!(substituted_marketplace_key, fixture.marketplace_public);
    assert!(
        verify_acquisition_bytes(
            &bytes,
            &fixture.admission,
            &substituted_marketplace_key,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err(),
        "matching marketplace labels must not authorize a different signing key"
    );

    let mut tampered: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let archive = tampered["archive"].as_str().unwrap().to_owned();
    let replacement = if archive.starts_with('A') { "B" } else { "A" };
    tampered["archive"] = serde_json::Value::String(format!("{replacement}{}", &archive[1..]));
    assert!(
        verify_acquisition_bytes(
            &serde_json::to_vec(&tampered).unwrap(),
            &fixture.admission,
            &fixture.marketplace_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err()
    );

    let mut extra: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    extra["destination"] = serde_json::json!("https://attacker.example.invalid");
    assert!(
        verify_acquisition_bytes(
            &serde_json::to_vec(&extra).unwrap(),
            &fixture.admission,
            &fixture.marketplace_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err()
    );
}

#[test]
fn acquisition_v2_separates_retired_snapshot_evidence_from_current_policy() {
    let fixture = fixture();
    let release = verify_release_directory(
        &fixture.release,
        &fixture.publisher_public,
        &fixture.sdk,
        &rich_2d_host_profile(),
    )
    .unwrap();
    let (policy_private, policy_public) =
        generate_catalog_keypair("marketplace-primary-v2", "marketplace").unwrap();
    let policy = sign_catalog_policy(
        &release,
        &policy_private,
        2,
        CatalogStatus::Active,
        "Re-signed after marketplace rotation.",
    )
    .unwrap();
    let mut policy_entry = fixture.entry.clone();
    policy_entry.policy = policy;
    let policy_snapshot_payload = MarketplaceSnapshotPayload {
        format: "omarchygs.marketplace-snapshot/v1".to_owned(),
        snapshot_version: 2,
        authority_id: policy_public.authority_id.clone(),
        marketplace_name: "Test Marketplace".to_owned(),
        releases: vec![policy_entry],
    };
    let policy_snapshot = serde_json::to_vec(
        &sign_marketplace_snapshot(&policy_snapshot_payload, &policy_private).unwrap(),
    )
    .unwrap();
    let document = CartridgeAcquisition::from_verified_bytes_with_policy(
        fixture.admission.clone(),
        fixture.marketplace_public.clone(),
        policy_public.clone(),
        &fixture.snapshot,
        &policy_snapshot,
        &fs::read(fixture.release.join(RELEASE_ARCHIVE_PATH)).unwrap(),
        &fs::read(fixture.release.join(RELEASE_CONFORMANCE_PATH)).unwrap(),
        &fs::read(fixture.release.join(RELEASE_ATTESTATION_PATH)).unwrap(),
    )
    .unwrap();
    let bytes = document.to_bounded_json().unwrap();
    let sdk = supported_sdk_identity().unwrap();
    let verified = verify_acquisition_bytes_with_policy_key(
        &bytes,
        &fixture.admission,
        &fixture.marketplace_public,
        &policy_public,
        &sdk,
        &rich_2d_host_profile(),
    )
    .unwrap();
    assert_eq!(verified.snapshot().snapshot_version, 1);
    assert_eq!(verified.policy_snapshot_version(), 2);
    assert_eq!(verified.policy_snapshot().snapshot_version, 2);
    assert_eq!(verified.policy_marketplace_key(), &policy_public);
    assert!(
        verify_acquisition_bytes(
            &bytes,
            &fixture.admission,
            &fixture.marketplace_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err(),
        "the legacy one-key verifier must not collapse the two authorities"
    );

    let mut unsigned_version: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    unsigned_version["policy_snapshot_version"] = serde_json::json!(999);
    assert!(
        verify_acquisition_bytes_with_policy_key(
            &serde_json::to_vec(&unsigned_version).unwrap(),
            &fixture.admission,
            &fixture.marketplace_public,
            &policy_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .is_err(),
        "unsigned lifecycle snapshot metadata must not be accepted"
    );
}

struct Fixture {
    _temp: tempfile::TempDir,
    release: std::path::PathBuf,
    sdk: std::path::PathBuf,
    publisher_public: omarchygs_game_cartridge::PublisherPublicKey,
    marketplace_public: omarchygs_game_cartridge::CatalogPublicKey,
    entry: MarketplaceReleaseEntry,
    snapshot: Vec<u8>,
    admission: AcquisitionServerAdmission,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let sdk = temp.path().join("sdk");
    let release_root = temp.path().join("release");
    fs::create_dir(&sdk).unwrap();
    fs::create_dir(&release_root).unwrap();
    export_sdk(&sdk).unwrap();
    let (publisher_private, publisher_public) =
        generate_keypair("publisher-primary-v1", "ignibyte").unwrap();
    create_release(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/first-party-door-legends/cartridge"),
        &publisher_private,
        &sdk,
        REVISION,
        BUILDER_DIGEST,
        &rich_2d_host_profile(),
        &release_root,
    )
    .unwrap();
    let release = verify_release_directory(
        &release_root,
        &publisher_public,
        &sdk,
        &rich_2d_host_profile(),
    )
    .unwrap();
    let (marketplace_private, marketplace_public) =
        generate_catalog_keypair("marketplace-primary-v1", "marketplace").unwrap();
    let policy = sign_catalog_policy(
        &release,
        &marketplace_private,
        1,
        CatalogStatus::Active,
        "Reviewed exact release.",
    )
    .unwrap();
    let entry = MarketplaceReleaseEntry {
        release_path: "releases/door-legends/1/".to_owned(),
        game_key: release.payload().game_key.clone(),
        publisher_id: release.payload().publisher_id.clone(),
        rules_version: release.payload().rules_version,
        cartridge_version: release.payload().cartridge_version,
        archive_sha256: release.payload().archive_sha256.clone(),
        signed_identity_sha256: release.payload().signed_identity_sha256.clone(),
        publisher_key: publisher_public.clone(),
        reviewed_by: "review-team".to_owned(),
        review_summary: "Bounded first-party review passed.".to_owned(),
        policy,
    };
    let payload = MarketplaceSnapshotPayload {
        format: "omarchygs.marketplace-snapshot/v1".to_owned(),
        snapshot_version: 1,
        authority_id: marketplace_public.authority_id.clone(),
        marketplace_name: "Test Marketplace".to_owned(),
        releases: vec![entry.clone()],
    };
    let snapshot =
        serde_json::to_vec(&sign_marketplace_snapshot(&payload, &marketplace_private).unwrap())
            .unwrap();
    let admission = AcquisitionServerAdmission {
        server_id: "11111111-1111-4111-8111-111111111111".to_owned(),
        game_key: release.payload().game_key.clone(),
        publisher_id: release.payload().publisher_id.clone(),
        rules_version: release.payload().rules_version,
        cartridge_version: release.payload().cartridge_version,
        archive_sha256: release.payload().archive_sha256.clone(),
        signed_identity_sha256: release.payload().signed_identity_sha256.clone(),
        admission_revision: 1,
    };
    Fixture {
        _temp: temp,
        release: release_root,
        sdk,
        publisher_public,
        marketplace_public,
        entry,
        snapshot,
        admission,
    }
}
