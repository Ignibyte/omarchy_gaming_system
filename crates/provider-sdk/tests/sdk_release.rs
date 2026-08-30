use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use ed25519_dalek::SigningKey;
use omarchygs_provider_sdk::{
    ProviderError,
    protocol::{ProviderCompatibilityOffer, ProviderCompatibilitySelection},
    release::{ProviderSdkReleaseSigner, export_sdk, verify_sdk_directory},
};

const REVISION: &str = "1111111111111111111111111111111111111111";
const BUILDER: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn exact_exports_and_signed_provenance_are_reproducible_and_tamper_evident() {
    let temp = tempfile::tempdir().expect("temporary directory should create");
    let one = temp.path().join("one");
    let two = temp.path().join("two");
    fs::create_dir(&one).expect("first output should create");
    fs::create_dir(&two).expect("second output should create");
    let signer = ProviderSdkReleaseSigner::new("omarchygs", "provider-sdk-preview-v1", [7; 32])
        .expect("release signer should construct");
    let first = export_sdk(&one, &signer, REVISION, BUILDER).expect("first export should succeed");
    let second =
        export_sdk(&two, &signer, REVISION, BUILDER).expect("second export should succeed");
    assert_eq!(first, second);
    assert_eq!(snapshot(&one), snapshot(&two));
    assert_eq!(
        verify_sdk_directory(
            &one,
            &signer.verifying_key(),
            "omarchygs",
            "provider-sdk-preview-v1",
        )
        .expect("exact export should verify"),
        first
    );

    let manifest = one.join("Cargo.toml");
    make_writable(&manifest);
    fs::write(&manifest, b"[package]\nname = \"changed\"\n")
        .expect("tampered manifest should write");
    assert!(matches!(
        verify_sdk_directory(
            &one,
            &signer.verifying_key(),
            "omarchygs",
            "provider-sdk-preview-v1",
        ),
        Err(ProviderError::ProtocolRejected)
    ));

    let wrong_key = SigningKey::from_bytes(&[8; 32]).verifying_key();
    assert!(
        verify_sdk_directory(&two, &wrong_key, "omarchygs", "provider-sdk-preview-v1",).is_err()
    );
    fs::write(two.join("unexpected.json"), b"{}").expect("unexpected file should write");
    assert!(matches!(
        verify_sdk_directory(
            &two,
            &signer.verifying_key(),
            "omarchygs",
            "provider-sdk-preview-v1",
        ),
        Err(ProviderError::ProtocolRejected)
    ));
}

#[test]
fn sdk_inventory_rejects_native_aliases_directories_and_broad_trees() {
    let temp = tempfile::tempdir().expect("temporary directory should create");
    let signer = ProviderSdkReleaseSigner::new("omarchygs", "provider-sdk-preview-v1", [7; 32])
        .expect("release signer should construct");

    #[cfg(unix)]
    {
        let alias = temp.path().join("alias");
        fs::create_dir(&alias).expect("alias output should create");
        export_sdk(&alias, &signer, REVISION, BUILDER).expect("alias export should succeed");
        fs::write(alias.join(r"src\lib.rs"), b"unsigned alias")
            .expect("literal backslash filename should write");
        assert!(matches!(
            verify_sdk_directory(
                &alias,
                &signer.verifying_key(),
                "omarchygs",
                "provider-sdk-preview-v1",
            ),
            Err(ProviderError::ProtocolRejected)
        ));
    }

    let empty_directory = temp.path().join("empty-directory");
    fs::create_dir(&empty_directory).expect("empty-directory output should create");
    export_sdk(&empty_directory, &signer, REVISION, BUILDER)
        .expect("empty-directory export should succeed");
    fs::create_dir(empty_directory.join("unsigned"))
        .expect("unexpected empty directory should create");
    assert!(matches!(
        verify_sdk_directory(
            &empty_directory,
            &signer.verifying_key(),
            "omarchygs",
            "provider-sdk-preview-v1",
        ),
        Err(ProviderError::ProtocolRejected)
    ));

    let broad = temp.path().join("broad");
    fs::create_dir(&broad).expect("broad output should create");
    export_sdk(&broad, &signer, REVISION, BUILDER).expect("broad export should succeed");
    for index in 0..=64 {
        fs::create_dir(broad.join(format!("unsigned-{index:02}")))
            .expect("unexpected broad entry should create");
    }
    assert!(matches!(
        verify_sdk_directory(
            &broad,
            &signer.verifying_key(),
            "omarchygs",
            "provider-sdk-preview-v1",
        ),
        Err(ProviderError::ProtocolRejected)
    ));
}

#[test]
fn shipped_compatibility_fixtures_accept_exact_v1_and_reject_downgrade() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sdk/v1/fixtures");
    let offer: ProviderCompatibilityOffer = serde_json::from_slice(
        &fs::read(root.join("compatibility-offer.json")).expect("offer fixture should read"),
    )
    .expect("offer fixture should parse");
    offer.validate().expect("exact offer should validate");
    let selection: ProviderCompatibilitySelection = serde_json::from_slice(
        &fs::read(root.join("compatibility-selection.json"))
            .expect("selection fixture should read"),
    )
    .expect("selection fixture should parse");
    selection
        .validate_for(&offer)
        .expect("selection fixture should bind offer");

    let downgrade: ProviderCompatibilityOffer = serde_json::from_slice(
        &fs::read(root.join("reject-downgrade-offer.json")).expect("downgrade fixture should read"),
    )
    .expect("downgrade fixture should parse structurally");
    assert!(downgrade.validate().is_err());
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).expect("snapshot directory should read") {
            let entry = entry.expect("snapshot entry should read");
            if entry.file_type().expect("file type should read").is_dir() {
                pending.push(entry.path());
            } else {
                files.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .expect("path should be below root")
                        .to_path_buf(),
                    fs::read(entry.path()).expect("snapshot file should read"),
                );
            }
        }
    }
    files
}

fn make_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt as _;

    let mut permissions = fs::metadata(path)
        .expect("metadata should read")
        .permissions();
    permissions.set_mode(permissions.mode() | 0o200);
    fs::set_permissions(path, permissions).expect("permissions should update");
}
