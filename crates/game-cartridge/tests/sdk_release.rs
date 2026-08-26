use std::{
    fs,
    path::{Path, PathBuf},
};

use omarchygs_game_cartridge::{
    ActiveSessionDecision, CartridgeError, CatalogStatus, LifecycleUse, NewLaunchDecision,
    SecureCartridgeStore, create_release, export_sdk, generate_catalog_keypair, generate_keypair,
    lifecycle_decision, sign_catalog_policy, verify_catalog_policy, verify_release_directory,
    verify_sdk_directory,
};
use tempfile::TempDir;

const REVISION: &str = "1111111111111111111111111111111111111111";
const BUILDER_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct Fixture {
    _temp: TempDir,
    sdk: PathBuf,
    release: PathBuf,
    private: omarchygs_game_cartridge::PublisherPrivateKey,
    public: omarchygs_game_cartridge::PublisherPublicKey,
}

fn example_source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/first-party-door-legends/cartridge")
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let sdk = temp.path().join("sdk");
    let release = temp.path().join("release");
    fs::create_dir(&sdk).unwrap();
    fs::create_dir(&release).unwrap();
    export_sdk(&sdk).unwrap();
    let (private, public) = generate_keypair("ignibyte-primary-v1", "ignibyte").unwrap();
    create_release(
        &example_source(),
        &private,
        &sdk,
        REVISION,
        BUILDER_DIGEST,
        &omarchygs_game_cartridge::core_host_profile(),
        &release,
    )
    .unwrap();
    Fixture {
        _temp: temp,
        sdk,
        release,
        private,
        public,
    }
}

fn policy_bytes(
    release: &omarchygs_game_cartridge::VerifiedRelease,
    key: &omarchygs_game_cartridge::CatalogPrivateKey,
    version: u64,
    status: CatalogStatus,
) -> Vec<u8> {
    serde_json::to_vec(
        &sign_catalog_policy(release, key, version, status, "test policy transition").unwrap(),
    )
    .unwrap()
}

#[test]
fn sdk_exports_are_exact_reproducible_and_drift_rejects() {
    let temp = tempfile::tempdir().unwrap();
    let one = temp.path().join("one");
    let two = temp.path().join("two");
    fs::create_dir(&one).unwrap();
    fs::create_dir(&two).unwrap();
    let first = export_sdk(&one).unwrap();
    let second = export_sdk(&two).unwrap();
    assert_eq!(first, second);
    assert_eq!(snapshot(&one), snapshot(&two));
    assert_eq!(verify_sdk_directory(&one).unwrap(), first);

    let manifest_schema = one.join("schemas/cartridge-manifest.schema.json");
    make_writable(&manifest_schema);
    fs::write(&manifest_schema, b"{}\n").unwrap();
    assert!(matches!(
        verify_sdk_directory(&one),
        Err(CartridgeError::InvalidSdk)
    ));

    fs::write(two.join("unexpected.json"), b"{}\n").unwrap();
    assert!(matches!(
        verify_sdk_directory(&two),
        Err(CartridgeError::InvalidSdk)
    ));
}

#[test]
fn release_attestation_is_reproducible_and_tamper_evident() {
    let fixture = fixture();
    let second = fixture._temp.path().join("release-two");
    fs::create_dir(&second).unwrap();
    create_release(
        &example_source(),
        &fixture.private,
        &fixture.sdk,
        REVISION,
        BUILDER_DIGEST,
        &omarchygs_game_cartridge::core_host_profile(),
        &second,
    )
    .unwrap();
    assert_eq!(snapshot(&fixture.release), snapshot(&second));

    let verified = verify_release_directory(
        &fixture.release,
        &fixture.public,
        &fixture.sdk,
        &omarchygs_game_cartridge::core_host_profile(),
    )
    .unwrap();
    assert_eq!(verified.payload().source_revision, REVISION);
    assert_eq!(verified.payload().builder.binary_sha256, BUILDER_DIGEST);
    assert_eq!(
        verified.payload().archive_sha256,
        verified.cartridge().archive_sha256()
    );

    let conformance = fixture.release.join("conformance.json");
    make_writable(&conformance);
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&conformance).unwrap()).unwrap();
    value["source_tree_read"] = serde_json::Value::Bool(true);
    fs::write(&conformance, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
        verify_release_directory(
            &fixture.release,
            &fixture.public,
            &fixture.sdk,
            &omarchygs_game_cartridge::core_host_profile(),
        )
        .is_err()
    );

    let (_, wrong_key) = generate_keypair("other-key-v1", "ignibyte").unwrap();
    assert!(
        verify_release_directory(
            &second,
            &wrong_key,
            &fixture.sdk,
            &omarchygs_game_cartridge::core_host_profile(),
        )
        .is_err()
    );
}

#[test]
fn lifecycle_matrix_is_exact_and_catalog_signatures_bind_release() {
    let fixture = fixture();
    let release = verify_release_directory(
        &fixture.release,
        &fixture.public,
        &fixture.sdk,
        &omarchygs_game_cartridge::core_host_profile(),
    )
    .unwrap();
    let (catalog_private, catalog_public) =
        generate_catalog_keypair("catalog-primary-v1", "omarchygs").unwrap();
    let expected = [
        (
            CatalogStatus::Active,
            NewLaunchDecision::Allow,
            ActiveSessionDecision::Continue,
        ),
        (
            CatalogStatus::Deprecated,
            NewLaunchDecision::AllowWithWarning,
            ActiveSessionDecision::Continue,
        ),
        (
            CatalogStatus::Suspended,
            NewLaunchDecision::Deny,
            ActiveSessionDecision::Suspend,
        ),
        (
            CatalogStatus::Revoked,
            NewLaunchDecision::Deny,
            ActiveSessionDecision::Terminate,
        ),
        (
            CatalogStatus::Retired,
            NewLaunchDecision::Deny,
            ActiveSessionDecision::Continue,
        ),
    ];
    for (status, launch, session) in expected {
        let decision = lifecycle_decision(status);
        assert_eq!(decision.new_launch, launch);
        assert_eq!(decision.active_session, session);
    }

    let signed = sign_catalog_policy(
        &release,
        &catalog_private,
        1,
        CatalogStatus::Active,
        "first-party release approved",
    )
    .unwrap();
    let policy = verify_catalog_policy(&signed, &catalog_public, &release).unwrap();
    assert_eq!(policy.archive_sha256, release.payload().archive_sha256);
    let (_, wrong_catalog) = generate_catalog_keypair("catalog-other-v1", "omarchygs").unwrap();
    assert!(verify_catalog_policy(&signed, &wrong_catalog, &release).is_err());
    assert!(
        sign_catalog_policy(
            &release,
            &catalog_private,
            0,
            CatalogStatus::Active,
            "invalid version",
        )
        .is_err()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn secure_store_stays_descriptor_anchored_and_enforces_fresh_policy() {
    use std::os::unix::fs::symlink;

    let fixture = fixture();
    let host = omarchygs_game_cartridge::core_host_profile();
    let release =
        verify_release_directory(&fixture.release, &fixture.public, &fixture.sdk, &host).unwrap();
    let (catalog_private, catalog_public) =
        generate_catalog_keypair("catalog-primary-v1", "omarchygs").unwrap();
    let active = policy_bytes(&release, &catalog_private, 1, CatalogStatus::Active);

    let parent = tempfile::tempdir().unwrap();
    let visible_root = parent.path().join("store");
    let anchored_root = parent.path().join("anchored-store");
    let attacker_root = parent.path().join("attacker-store");
    fs::create_dir(&visible_root).unwrap();
    let store = SecureCartridgeStore::open_existing(&visible_root).unwrap();
    fs::rename(&visible_root, &anchored_root).unwrap();
    fs::create_dir(&attacker_root).unwrap();
    symlink(&attacker_root, &visible_root).unwrap();

    let report = store
        .import_release(&release, &active, &catalog_public)
        .unwrap();
    assert!(report.descriptor_relative);
    assert!(
        anchored_root
            .join(format!(
                "blobs/sha256/{}.ogsc",
                report.activation.archive_sha256
            ))
            .is_file()
    );
    assert_eq!(fs::read_dir(&attacker_root).unwrap().count(), 0);

    let resolved = store
        .resolve_active(
            "door-legends",
            &fixture.public,
            &host,
            &active,
            &catalog_public,
            LifecycleUse::NewLaunch,
        )
        .unwrap();
    assert_eq!(
        resolved.cartridge().archive_sha256(),
        release.cartridge().archive_sha256()
    );

    let suspended = policy_bytes(&release, &catalog_private, 2, CatalogStatus::Suspended);
    assert!(matches!(
        store.resolve_active(
            "door-legends",
            &fixture.public,
            &host,
            &suspended,
            &catalog_public,
            LifecycleUse::NewLaunch,
        ),
        Err(CartridgeError::LifecycleDenied)
    ));
    assert!(matches!(
        store.resolve_active(
            "door-legends",
            &fixture.public,
            &host,
            &active,
            &catalog_public,
            LifecycleUse::NewLaunch,
        ),
        Err(CartridgeError::InvalidCatalogPolicy)
    ));

    let retired = policy_bytes(&release, &catalog_private, 3, CatalogStatus::Retired);
    assert!(
        store
            .resolve_active(
                "door-legends",
                &fixture.public,
                &host,
                &retired,
                &catalog_public,
                LifecycleUse::ActiveSession,
            )
            .is_ok()
    );
    assert!(matches!(
        store.resolve_active(
            "door-legends",
            &fixture.public,
            &host,
            &retired,
            &catalog_public,
            LifecycleUse::NewLaunch,
        ),
        Err(CartridgeError::LifecycleDenied)
    ));

    let revoked = policy_bytes(&release, &catalog_private, 4, CatalogStatus::Revoked);
    assert!(matches!(
        store.resolve_active(
            "door-legends",
            &fixture.public,
            &host,
            &revoked,
            &catalog_public,
            LifecycleUse::ActiveSession,
        ),
        Err(CartridgeError::LifecycleDenied)
    ));
    assert_eq!(fs::read_dir(&attacker_root).unwrap().count(), 0);
}

#[cfg(target_os = "linux")]
#[test]
fn reviewed_staging_never_writes_active_and_resolves_only_the_exact_digest() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = fixture();
    let host = omarchygs_game_cartridge::core_host_profile();
    let release =
        verify_release_directory(&fixture.release, &fixture.public, &fixture.sdk, &host).unwrap();
    let (catalog_private, catalog_public) =
        generate_catalog_keypair("catalog-primary-v1", "omarchygs").unwrap();
    let active = policy_bytes(&release, &catalog_private, 1, CatalogStatus::Active);
    let suspended = policy_bytes(&release, &catalog_private, 2, CatalogStatus::Suspended);
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let store = SecureCartridgeStore::open_existing(&root).unwrap();

    let staged = store
        .stage_reviewed_release(&release, &active, &catalog_public)
        .unwrap();
    assert!(staged.installed);
    assert!(!staged.active_pointer_written);
    assert!(!root.join("active/door-legends.json").exists());
    assert!(
        store
            .resolve_exact(
                "door-legends",
                &release.payload().archive_sha256,
                &fixture.public,
                &host,
                &active,
                &catalog_public,
                LifecycleUse::NewLaunch,
            )
            .is_ok()
    );
    assert!(matches!(
        store.resolve_exact(
            "door-legends",
            &"0".repeat(64),
            &fixture.public,
            &host,
            &active,
            &catalog_public,
            LifecycleUse::NewLaunch,
        ),
        Err(CartridgeError::InvalidCatalogPolicy) | Err(CartridgeError::InvalidActivation)
    ));

    let denied = store
        .stage_reviewed_release(&release, &suspended, &catalog_public)
        .unwrap();
    assert!(!denied.installed);
    assert!(!root.join("active/door-legends.json").exists());
    assert!(matches!(
        store.resolve_exact(
            "door-legends",
            &release.payload().archive_sha256,
            &fixture.public,
            &host,
            &suspended,
            &catalog_public,
            LifecycleUse::NewLaunch,
        ),
        Err(CartridgeError::LifecycleDenied)
    ));
    assert!(matches!(
        store.resolve_exact(
            "door-legends",
            &release.payload().archive_sha256,
            &fixture.public,
            &host,
            &active,
            &catalog_public,
            LifecycleUse::NewLaunch,
        ),
        Err(CartridgeError::InvalidCatalogPolicy)
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn secure_store_rejects_group_or_world_writable_directories() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();

    let writable_root = temp.path().join("writable-root");
    fs::create_dir(&writable_root).unwrap();
    fs::set_permissions(&writable_root, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(matches!(
        SecureCartridgeStore::open_existing(&writable_root),
        Err(CartridgeError::UnsafeFilesystemPath)
    ));

    let writable_child_root = temp.path().join("writable-child-root");
    fs::create_dir(&writable_child_root).unwrap();
    fs::set_permissions(&writable_child_root, fs::Permissions::from_mode(0o755)).unwrap();
    let active = writable_child_root.join("active");
    fs::create_dir(&active).unwrap();
    fs::set_permissions(&active, fs::Permissions::from_mode(0o770)).unwrap();
    assert!(matches!(
        SecureCartridgeStore::open_existing(&writable_child_root),
        Err(CartridgeError::UnsafeFilesystemPath)
    ));

    let ordinary_root = temp.path().join("ordinary-root");
    fs::create_dir(&ordinary_root).unwrap();
    fs::set_permissions(&ordinary_root, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(SecureCartridgeStore::open_existing(&ordinary_root).is_ok());
}

#[cfg(target_os = "linux")]
#[test]
fn denied_import_policy_survives_restart_and_blocks_rollback() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = fixture();
    let host = omarchygs_game_cartridge::core_host_profile();
    let release =
        verify_release_directory(&fixture.release, &fixture.public, &fixture.sdk, &host).unwrap();
    let (catalog_private, catalog_public) =
        generate_catalog_keypair("catalog-primary-v1", "omarchygs").unwrap();
    let active_v1 = policy_bytes(&release, &catalog_private, 1, CatalogStatus::Active);
    let revoked_v2 = policy_bytes(&release, &catalog_private, 2, CatalogStatus::Revoked);
    let active_v3 = policy_bytes(&release, &catalog_private, 3, CatalogStatus::Active);

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    {
        let store = SecureCartridgeStore::open_existing(&root).unwrap();
        assert!(matches!(
            store.import_release(&release, &revoked_v2, &catalog_public),
            Err(CartridgeError::LifecycleDenied)
        ));
        assert!(!root.join("active/door-legends.json").exists());
        assert_eq!(fs::read_dir(root.join("blobs/sha256")).unwrap().count(), 0);
    }

    let reopened = SecureCartridgeStore::open_existing(&root).unwrap();
    assert!(matches!(
        reopened.import_release(&release, &active_v1, &catalog_public),
        Err(CartridgeError::InvalidCatalogPolicy)
    ));
    assert!(
        reopened
            .import_release(&release, &active_v3, &catalog_public)
            .is_ok()
    );
    assert!(root.join("active/door-legends.json").is_file());
}

#[cfg(target_os = "linux")]
#[test]
fn concurrent_policy_transitions_never_replace_a_newer_version() {
    use std::{
        os::unix::fs::PermissionsExt,
        sync::{Arc, Barrier},
        thread,
    };

    let fixture = fixture();
    let host = omarchygs_game_cartridge::core_host_profile();
    let release =
        verify_release_directory(&fixture.release, &fixture.public, &fixture.sdk, &host).unwrap();
    let (catalog_private, catalog_public) =
        generate_catalog_keypair("catalog-primary-v1", "omarchygs").unwrap();
    let active_v1 = policy_bytes(&release, &catalog_private, 1, CatalogStatus::Active);
    let deprecated_v2 = policy_bytes(&release, &catalog_private, 2, CatalogStatus::Deprecated);
    let active_v3 = policy_bytes(&release, &catalog_private, 3, CatalogStatus::Active);
    let temp = tempfile::tempdir().unwrap();

    for trial in 0..64 {
        let root = temp.path().join(format!("store-{trial}"));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        SecureCartridgeStore::open_existing(&root)
            .unwrap()
            .import_release(&release, &active_v1, &catalog_public)
            .unwrap();
        let older_store = SecureCartridgeStore::open_existing(&root).unwrap();
        let newer_store = SecureCartridgeStore::open_existing(&root).unwrap();
        let barrier = Arc::new(Barrier::new(3));

        let older_barrier = Arc::clone(&barrier);
        let older_release = release.clone();
        let older_key = catalog_public.clone();
        let older_policy = deprecated_v2.clone();
        let older = thread::spawn(move || {
            older_barrier.wait();
            older_store.import_release(&older_release, &older_policy, &older_key)
        });

        let newer_barrier = Arc::clone(&barrier);
        let newer_release = release.clone();
        let newer_key = catalog_public.clone();
        let newer_policy = active_v3.clone();
        let newer = thread::spawn(move || {
            newer_barrier.wait();
            newer_store.import_release(&newer_release, &newer_policy, &newer_key)
        });

        barrier.wait();
        let older_result = older.join().unwrap();
        let newer_result = newer.join().unwrap();
        assert!(
            older_result.is_ok()
                || matches!(older_result, Err(CartridgeError::InvalidCatalogPolicy))
        );
        assert!(newer_result.is_ok());

        let final_store = SecureCartridgeStore::open_existing(&root).unwrap();
        assert!(matches!(
            final_store.resolve_active(
                "door-legends",
                &fixture.public,
                &host,
                &deprecated_v2,
                &catalog_public,
                LifecycleUse::NewLaunch,
            ),
            Err(CartridgeError::InvalidCatalogPolicy)
        ));
        assert!(
            final_store
                .resolve_active(
                    "door-legends",
                    &fixture.public,
                    &host,
                    &active_v3,
                    &catalog_public,
                    LifecycleUse::NewLaunch,
                )
                .is_ok()
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn secure_store_rejects_symlinked_fixed_children() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    let outside = temp.path().join("outside");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&outside).unwrap();
    symlink(&outside, root.join("active")).unwrap();
    assert!(SecureCartridgeStore::open_existing(&root).is_err());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
}

fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    collect_snapshot(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn collect_snapshot(root: &Path, path: &Path, files: &mut Vec<(String, Vec<u8>)>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            collect_snapshot(root, &entry.path(), files);
        } else {
            files.push((
                entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                fs::read(entry.path()).unwrap(),
            ));
        }
    }
}

#[cfg(unix)]
fn make_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
}

#[cfg(not(unix))]
fn make_writable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions).unwrap();
}
