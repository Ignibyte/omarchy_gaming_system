use std::{fs, path::Path};

use omarchygs_game_cartridge::{
    CartridgeError, CatalogStatus, MarketplaceReleaseEntry, MarketplaceSnapshotPayload,
    create_release, export_sdk, generate_catalog_keypair, generate_keypair, sign_catalog_policy,
    sign_marketplace_snapshot, verify_marketplace_snapshot_bytes, verify_release_directory,
};

const REVISION: &str = "1111111111111111111111111111111111111111";
const BUILDER_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn signed_snapshot_is_deterministic_exact_and_domain_authenticated() {
    let fixture = fixture();
    let first = sign_marketplace_snapshot(&fixture.payload, &fixture.catalog_private).unwrap();
    let second = sign_marketplace_snapshot(&fixture.payload, &fixture.catalog_private).unwrap();
    assert_eq!(first, second);
    let bytes = serde_json::to_vec(&first).unwrap();
    let verified = verify_marketplace_snapshot_bytes(&bytes, &fixture.catalog_public).unwrap();
    assert_eq!(verified, fixture.payload);

    let (_, wrong_key) = generate_catalog_keypair("other-v1", "marketplace").unwrap();
    assert!(verify_marketplace_snapshot_bytes(&bytes, &wrong_key).is_err());

    let mut tampered = first;
    let replacement = if tampered.signature.starts_with('A') {
        "B"
    } else {
        "A"
    };
    tampered.signature.replace_range(0..1, replacement);
    assert!(matches!(
        verify_marketplace_snapshot_bytes(
            &serde_json::to_vec(&tampered).unwrap(),
            &fixture.catalog_public,
        ),
        Err(CartridgeError::InvalidSignature)
    ));

    let mut extra: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    extra["unexpected"] = serde_json::json!(true);
    assert!(
        verify_marketplace_snapshot_bytes(
            &serde_json::to_vec(&extra).unwrap(),
            &fixture.catalog_public,
        )
        .is_err()
    );
}

#[test]
fn snapshot_signing_rejects_unsafe_or_ambiguous_inventory() {
    let fixture = fixture();
    let mut invalid = fixture.payload.clone();
    invalid.snapshot_version = 0;
    assert!(sign_marketplace_snapshot(&invalid, &fixture.catalog_private).is_err());

    let mut invalid = fixture.payload.clone();
    invalid.authority_id = "other-marketplace".to_owned();
    assert!(sign_marketplace_snapshot(&invalid, &fixture.catalog_private).is_err());

    for release_path in [
        "/absolute/",
        "../escape/",
        "releases//game/",
        "releases/%2e%2e/game/",
        "releases/game?target=x/",
        "releases/game",
    ] {
        let mut invalid = fixture.payload.clone();
        invalid.releases[0].release_path = release_path.to_owned();
        assert!(
            sign_marketplace_snapshot(&invalid, &fixture.catalog_private).is_err(),
            "{release_path} should reject"
        );
    }

    let mut invalid = fixture.payload.clone();
    invalid.releases[0].review_summary = "unsafe\nreview".to_owned();
    assert!(sign_marketplace_snapshot(&invalid, &fixture.catalog_private).is_err());

    let mut invalid = fixture.payload.clone();
    invalid.releases.push(invalid.releases[0].clone());
    assert!(sign_marketplace_snapshot(&invalid, &fixture.catalog_private).is_err());

    let mut invalid = fixture.payload.clone();
    invalid.releases[0].archive_sha256 = "0".repeat(64);
    assert!(sign_marketplace_snapshot(&invalid, &fixture.catalog_private).is_err());
}

struct Fixture {
    _temp: tempfile::TempDir,
    catalog_private: omarchygs_game_cartridge::CatalogPrivateKey,
    catalog_public: omarchygs_game_cartridge::CatalogPublicKey,
    payload: MarketplaceSnapshotPayload,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let sdk = temp.path().join("sdk");
    let release_root = temp.path().join("release");
    fs::create_dir(&sdk).unwrap();
    fs::create_dir(&release_root).unwrap();
    export_sdk(&sdk).unwrap();
    let (publisher_private, publisher_public) =
        generate_keypair("ignibyte-primary-v1", "ignibyte").unwrap();
    create_release(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/first-party-door-legends/cartridge"),
        &publisher_private,
        &sdk,
        REVISION,
        BUILDER_DIGEST,
        &omarchygs_game_cartridge::core_host_profile(),
        &release_root,
    )
    .unwrap();
    let release = verify_release_directory(
        &release_root,
        &publisher_public,
        &sdk,
        &omarchygs_game_cartridge::core_host_profile(),
    )
    .unwrap();
    let (catalog_private, catalog_public) =
        generate_catalog_keypair("marketplace-primary-v1", "marketplace").unwrap();
    let policy = sign_catalog_policy(
        &release,
        &catalog_private,
        1,
        CatalogStatus::Active,
        "reviewed release",
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
        publisher_key: publisher_public,
        reviewed_by: "omarchygs-review".to_owned(),
        review_summary: "First-party conformance and safety review passed.".to_owned(),
        policy,
    };
    Fixture {
        _temp: temp,
        catalog_private,
        catalog_public,
        payload: MarketplaceSnapshotPayload {
            format: "omarchygs.marketplace-snapshot/v1".to_owned(),
            snapshot_version: 1,
            authority_id: "marketplace".to_owned(),
            marketplace_name: "OmarchyGS Marketplace".to_owned(),
            releases: vec![entry],
        },
    }
}
