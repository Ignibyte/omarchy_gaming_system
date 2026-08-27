use std::{
    collections::HashMap,
    fs,
    net::{Ipv4Addr, SocketAddr},
    os::unix::fs::{PermissionsExt as _, symlink},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{Response, StatusCode},
    routing::get,
};
use axum_server::tls_rustls::RustlsConfig;
use omarchygs_game_cartridge::{
    CatalogStatus, create_release, export_sdk, generate_catalog_keypair, generate_keypair,
    rich_2d_host_profile,
};
use omarchygs_marketplace_publisher::{
    PrepareOptions, ProbeFloor, PublicationPackagePlan, PublicationPlan, PublicationReleasePlan,
    activate_publication, finalize_publication, offline_sign, prepare_publication,
    probe_mirrors_with_clients, publication_id, verify_current, verify_version,
};
use omarchygs_marketplace_trust::{
    ChannelOrigin, GuardedChannelClient, MarketplaceKeyStatus, MarketplaceTrustKey,
    catalog_key_sha256, generate_trust_root_keypair, read_trust_root_public_key,
    write_new_private_key, write_new_public_key,
};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use sha2::{Digest as _, Sha256};
use tokio::{net::TcpStream, sync::RwLock, task::JoinHandle};

const NOW: u64 = 2_000_000_000;
const REVISION: &str = "1111111111111111111111111111111111111111";
const BUILDER_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn complete_publication_is_deterministic_atomic_and_fail_closed() {
    let fixture = Fixture::new();
    let first = fixture.prepare("prepared-one");
    let second = fixture.prepare("prepared-two");
    assert_eq!(first.1.evidence_sha256, second.1.evidence_sha256);
    assert_eq!(
        fs::read(first.0.join("offline-request.json")).unwrap(),
        fs::read(second.0.join("offline-request.json")).unwrap()
    );

    let first_response = fixture.root.join("response-one.json");
    let second_response = fixture.root.join("response-two.json");
    let signed_one = offline_sign(
        &first.0.join("offline-request.json"),
        &fixture.root_private,
        &first_response,
    )
    .expect("first offline request should sign");
    let signed_two = offline_sign(
        &second.0.join("offline-request.json"),
        &fixture.root_private,
        &second_response,
    )
    .expect("second offline request should sign");
    assert_eq!(signed_one.evidence_sha256, signed_two.evidence_sha256);
    assert_eq!(
        fs::read(&first_response).unwrap(),
        fs::read(&second_response).unwrap()
    );

    let store_one = fixture.root.join("store-one");
    let store_two = fixture.root.join("store-two");
    let finalized_one = finalize_publication(&first.0, &first_response, &store_one, NOW)
        .expect("first publication should finalize");
    let finalized_two = finalize_publication(&second.0, &second_response, &store_two, NOW)
        .expect("second publication should finalize");
    assert_eq!(finalized_one, finalized_two);
    assert_eq!(finalized_one.publication_id, publication_id(1));

    let version = format!(
        "{:020}-{}",
        finalized_one.bundle_version,
        finalized_one.publication_sha256.as_deref().unwrap()
    );
    let activated = activate_publication(&store_one, &version, &fixture.root_public, NOW)
        .expect("publication should activate");
    assert_eq!(
        activated.publication_sha256.as_deref(),
        finalized_one.publication_sha256.as_deref()
    );
    assert_eq!(
        verify_current(&store_one, &fixture.root_public, NOW)
            .expect("current publication should verify")
            .publication_sha256
            .as_deref(),
        finalized_one.publication_sha256.as_deref()
    );
    assert_eq!(
        verify_version(&store_two, &version, &fixture.root_public, NOW)
            .expect("copied publication identity should verify")
            .publication_sha256
            .as_deref(),
        finalized_one.publication_sha256.as_deref()
    );

    let first_manifest = store_one
        .join("versions")
        .join(&version)
        .join("channel/publication.json");
    let second_manifest = store_two
        .join("versions")
        .join(&version)
        .join("channel/publication.json");
    assert_eq!(
        fs::read(first_manifest).unwrap(),
        fs::read(second_manifest).unwrap()
    );

    let extra = store_one
        .join("versions")
        .join(&version)
        .join("marketplace/unexpected.json");
    fs::write(&extra, b"{}").unwrap();
    fs::set_permissions(&extra, fs::Permissions::from_mode(0o444)).unwrap();
    assert!(verify_current(&store_one, &fixture.root_public, NOW).is_err());
    fs::remove_file(extra).unwrap();

    let archive = store_one
        .join("versions")
        .join(&version)
        .join("marketplace/releases/door-legends/1/cartridge.ogsc");
    fs::set_permissions(&archive, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(verify_current(&store_one, &fixture.root_public, NOW).is_err());
}

#[test]
fn concurrent_finalization_converges_on_one_immutable_version() {
    let fixture = Fixture::new();
    let (prepared, _) = fixture.prepare("prepared-concurrent");
    let response = fixture.root.join("response-concurrent.json");
    offline_sign(
        &prepared.join("offline-request.json"),
        &fixture.root_private,
        &response,
    )
    .unwrap();
    let store = fixture.root.join("store-concurrent");
    let first = std::thread::spawn({
        let prepared = prepared.clone();
        let response = response.clone();
        let store = store.clone();
        move || finalize_publication(&prepared, &response, &store, NOW)
    });
    let second = std::thread::spawn({
        let prepared = prepared.clone();
        let response = response.clone();
        let store = store.clone();
        move || finalize_publication(&prepared, &response, &store, NOW)
    });
    assert_eq!(
        first.join().unwrap().unwrap(),
        second.join().unwrap().unwrap()
    );
    assert_eq!(fs::read_dir(store.join("versions")).unwrap().count(), 1);
}

#[test]
fn private_key_modes_paths_and_offline_response_identity_fail_closed() {
    let fixture = Fixture::new();
    let (prepared, _) = fixture.prepare("prepared-hostile");
    fs::set_permissions(&fixture.root_private, fs::Permissions::from_mode(0o640)).unwrap();
    assert!(
        offline_sign(
            &prepared.join("offline-request.json"),
            &fixture.root_private,
            &fixture.root.join("mode-response.json"),
        )
        .is_err()
    );
    fs::set_permissions(&fixture.root_private, fs::Permissions::from_mode(0o600)).unwrap();

    let response = fixture.root.join("hostile-response.json");
    offline_sign(
        &prepared.join("offline-request.json"),
        &fixture.root_private,
        &response,
    )
    .unwrap();
    let snapshot = prepared.join("public/marketplace/snapshot.signed.json");
    fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o644)).unwrap();
    fs::write(&snapshot, b"tampered").unwrap();
    assert!(
        finalize_publication(
            &prepared,
            &response,
            &fixture.root.join("tampered-store"),
            NOW,
        )
        .is_err()
    );

    let symlink_fixture = Fixture::new();
    let package = symlink_fixture
        .input
        .join("packages/omarchygs-client.pkg.tar.zst");
    fs::remove_file(&package).unwrap();
    symlink("/etc/passwd", &package).unwrap();
    assert!(
        prepare_publication(PrepareOptions {
            plan_path: &symlink_fixture.plan,
            input_root: &symlink_fixture.input,
            sdk_root: &symlink_fixture.sdk,
            catalog_private_key_path: &symlink_fixture.catalog_private,
            root_public_key_path: &symlink_fixture.root_public,
            previous_trust_path: None,
            output_root: &symlink_fixture.root.join("symlink-prepared"),
        })
        .is_err()
    );

    let foreign = Fixture::new();
    let (foreign_prepared, _) = foreign.prepare("foreign-prepared");
    let foreign_response = foreign.root.join("foreign-response.json");
    offline_sign(
        &foreign_prepared.join("offline-request.json"),
        &foreign.root_private,
        &foreign_response,
    )
    .unwrap();
    let clean = fixture.prepare("prepared-clean").0;
    assert!(
        finalize_publication(
            &clean,
            &foreign_response,
            &fixture.root.join("foreign-store"),
            NOW,
        )
        .is_err()
    );
}

#[tokio::test]
async fn guarded_tls_mirrors_require_one_exact_authenticated_publication() {
    let fixture = Fixture::new();
    let (store, version) = fixture.finalize("probe");
    let tree = load_hosted_tree(&store.join("versions").join(version));
    let first_files = Arc::new(RwLock::new(tree.clone()));
    let second_files = Arc::new(RwLock::new(tree));
    let (first_address, first_certificate, first_server) = spawn_mirror(first_files).await;
    let (second_address, second_certificate, second_server) =
        spawn_mirror(Arc::clone(&second_files)).await;
    let (first_channel, first_marketplace) = clients(first_address, &first_certificate);
    let (second_channel, second_marketplace) = clients(second_address, &second_certificate);
    let root = read_trust_root_public_key(&fixture.root_public).unwrap();
    let floor = ProbeFloor {
        minimum_bundle_version: 1,
        minimum_snapshot_version: 1,
        expected_publication_sha256: None,
    };

    let receipt = probe_mirrors_with_clients(
        &[
            (&first_channel, &first_marketplace),
            (&second_channel, &second_marketplace),
        ],
        &root,
        &floor,
        NOW,
    )
    .await
    .expect("identical mirrors should verify");
    assert_eq!(receipt.mirrors, 2);
    assert_eq!(receipt.operation, "probe_mirrors");
    let stale_floor = ProbeFloor {
        minimum_bundle_version: 2,
        minimum_snapshot_version: 2,
        expected_publication_sha256: None,
    };
    assert!(
        probe_mirrors_with_clients(
            &[
                (&first_channel, &first_marketplace),
                (&second_channel, &second_marketplace),
            ],
            &root,
            &stale_floor,
            NOW,
        )
        .await
        .is_err()
    );

    let archive = "marketplace/releases/door-legends/1/cartridge.ogsc";
    second_files
        .write()
        .await
        .insert(archive.to_owned(), b"tampered".to_vec());
    assert!(
        probe_mirrors_with_clients(
            &[
                (&first_channel, &first_marketplace),
                (&second_channel, &second_marketplace),
            ],
            &root,
            &floor,
            NOW,
        )
        .await
        .is_err()
    );
    first_server.abort();
    second_server.abort();
}

#[test]
fn catalog_compromise_drill_rotates_key_and_denies_publication_rollback() {
    let fixture = Fixture::new();
    let (store, first_version) = fixture.finalize("initial");
    activate_publication(&store, &first_version, &fixture.root_public, NOW).unwrap();
    let previous_trust = store
        .join("versions")
        .join(&first_version)
        .join("channel/trust.signed.json");

    let (successor_private, successor_public) =
        generate_catalog_keypair("catalog-successor-v2", "official-marketplace").unwrap();
    let successor_private_path = fixture.root.join("catalog-successor.private.json");
    write_json(&successor_private_path, &successor_private, 0o600);
    let mut plan: PublicationPlan =
        serde_json::from_slice(&fs::read(&fixture.plan).unwrap()).unwrap();
    plan.publication_id = publication_id(2);
    plan.created_at_unix = NOW;
    plan.ceremony_unix = NOW + 1;
    plan.bundle_version = 2;
    plan.snapshot_version = 2;
    plan.not_before_unix = NOW;
    plan.expires_at_unix = NOW + 3_600;
    plan.previous_trust_sha256 = Some(sha256(&fs::read(&previous_trust).unwrap()));
    plan.keys[0].status = MarketplaceKeyStatus::Revoked;
    plan.keys[0].last_snapshot_version = Some(1);
    plan.keys.push(MarketplaceTrustKey {
        key_sha256: catalog_key_sha256(&successor_public).unwrap(),
        key: successor_public,
        status: MarketplaceKeyStatus::Active,
        first_snapshot_version: 2,
        last_snapshot_version: None,
    });
    plan.releases[0].policy_version = 2;
    plan.packages[0].package_version = "0.1.0-2".to_owned();
    let plan_path = fixture.root.join("rotation-plan.json");
    write_json(&plan_path, &plan, 0o644);
    let prepared = fixture.root.join("prepared-rotation");
    prepare_publication(PrepareOptions {
        plan_path: &plan_path,
        input_root: &fixture.input,
        sdk_root: &fixture.sdk,
        catalog_private_key_path: &successor_private_path,
        root_public_key_path: &fixture.root_public,
        previous_trust_path: Some(&previous_trust),
        output_root: &prepared,
    })
    .expect("root-compatible successor should prepare");
    let response = fixture.root.join("rotation-response.json");
    offline_sign(
        &prepared.join("offline-request.json"),
        &fixture.root_private,
        &response,
    )
    .expect("rotation should sign offline");
    let second = finalize_publication(&prepared, &response, &store, NOW + 1)
        .expect("rotation should finalize");
    let second_version = format!(
        "{:020}-{}",
        second.bundle_version,
        second.publication_sha256.as_deref().unwrap()
    );
    activate_publication(&store, &second_version, &fixture.root_public, NOW + 1)
        .expect("higher root-authorized publication should activate");
    assert!(activate_publication(&store, &first_version, &fixture.root_public, NOW + 1).is_err());
    assert!(verify_version(&store, &first_version, &fixture.root_public, NOW + 1).is_ok());
}

#[test]
fn cli_runs_complete_ceremony_and_emits_stable_secret_free_json() {
    let fixture = Fixture::new();
    let binary = env!("CARGO_BIN_EXE_omarchygs-marketplace-publisher");
    let prepared = fixture.root.join("cli-prepared");
    let prepare = Command::new(binary)
        .args([
            "prepare",
            fixture.plan.to_str().unwrap(),
            fixture.input.to_str().unwrap(),
            fixture.sdk.to_str().unwrap(),
            fixture.catalog_private.to_str().unwrap(),
            fixture.root_public.to_str().unwrap(),
            "-",
            prepared.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(prepare.status.success());
    let prepare_receipt: serde_json::Value = serde_json::from_slice(&prepare.stdout).unwrap();
    assert_eq!(prepare_receipt["operation"], "prepare");
    assert!(!String::from_utf8_lossy(&prepare.stdout).contains(fixture.root.to_str().unwrap()));

    let response = fixture.root.join("cli-response.json");
    let signing = Command::new("bwrap")
        .args([
            "--die-with-parent",
            "--unshare-net",
            "--ro-bind",
            "/",
            "/",
            "--bind",
            fixture.root.to_str().unwrap(),
            fixture.root.to_str().unwrap(),
            "--dev-bind",
            "/dev",
            "/dev",
            "--proc",
            "/proc",
            binary,
            "offline-sign",
            prepared.join("offline-request.json").to_str().unwrap(),
            fixture.root_private.to_str().unwrap(),
            response.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(signing.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&signing.stdout).unwrap()["operation"],
        "offline_sign"
    );
    assert!(!String::from_utf8_lossy(&signing.stdout).contains(fixture.root.to_str().unwrap()));

    let store = fixture.root.join("cli-store");
    let finalized = Command::new(binary)
        .args([
            "finalize",
            prepared.to_str().unwrap(),
            response.to_str().unwrap(),
            store.to_str().unwrap(),
            &NOW.to_string(),
        ])
        .output()
        .unwrap();
    assert!(finalized.status.success());
    let finalized: serde_json::Value = serde_json::from_slice(&finalized.stdout).unwrap();
    let version = format!(
        "{:020}-{}",
        finalized["bundle_version"].as_u64().unwrap(),
        finalized["publication_sha256"].as_str().unwrap()
    );
    for (command, target) in [("activate", version.as_str()), ("verify", "current")] {
        let output = Command::new(binary)
            .args([
                command,
                store.to_str().unwrap(),
                target,
                fixture.root_public.to_str().unwrap(),
                &NOW.to_string(),
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{command} should succeed");
        assert!(!String::from_utf8_lossy(&output.stdout).contains(fixture.root.to_str().unwrap()));
    }

    let invalid = Command::new(binary).output().unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&invalid.stderr).unwrap()["code"],
        "marketplace_publication_invalid_input"
    );
}

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    input: PathBuf,
    sdk: PathBuf,
    catalog_private: PathBuf,
    root_private: PathBuf,
    root_public: PathBuf,
    plan: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temporary root should create");
        let root = temp.path().to_path_buf();
        let input = root.join("input");
        let sdk = root.join("sdk");
        let release = input.join("release");
        for directory in [&input, &sdk, &release] {
            fs::create_dir_all(directory).unwrap();
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
        }
        export_sdk(&sdk).expect("SDK should export");

        let source = root.join("source");
        copy_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/first-party-door-legends/cartridge"),
            &source,
        );
        let (publisher_private, publisher_public) =
            generate_keypair("publisher-primary-v1", "ignibyte").unwrap();
        create_release(
            &source,
            &publisher_private,
            &sdk,
            REVISION,
            BUILDER_DIGEST,
            &rich_2d_host_profile(),
            &release,
        )
        .expect("release should build");
        write_json(
            &input.join("publisher.public.json"),
            &publisher_public,
            0o644,
        );

        let package_path = input.join("packages/omarchygs-client.pkg.tar.zst");
        fs::create_dir_all(package_path.parent().unwrap()).unwrap();
        fs::write(&package_path, b"native-package-fixture").unwrap();

        let (catalog_private_value, catalog_public) =
            generate_catalog_keypair("catalog-primary-v1", "official-marketplace").unwrap();
        let catalog_private = root.join("catalog.private.json");
        write_json(&catalog_private, &catalog_private_value, 0o600);
        let (root_private_value, root_public_value) =
            generate_trust_root_keypair("root-primary-v1", "official").unwrap();
        let root_private = root.join("root.private.json");
        let root_public = root.join("root.public.json");
        write_new_private_key(&root_private, &root_private_value).unwrap();
        write_new_public_key(&root_public, &root_public_value).unwrap();

        let plan = PublicationPlan {
            format: "omarchygs.marketplace-publication-plan/v1".to_owned(),
            publication_id: publication_id(1),
            created_at_unix: NOW - 10,
            ceremony_unix: NOW,
            channel_id: "official".to_owned(),
            channel_name: "Official OmarchyGS".to_owned(),
            channel_origin: "https://packages.example.test/v1/".to_owned(),
            marketplace_origin: "https://market.example.test/v1/".to_owned(),
            marketplace_authority_id: "official-marketplace".to_owned(),
            marketplace_name: "OmarchyGS Marketplace".to_owned(),
            bundle_version: 1,
            snapshot_version: 1,
            not_before_unix: NOW - 10,
            expires_at_unix: NOW + 3_600,
            keys: vec![MarketplaceTrustKey {
                key_sha256: catalog_key_sha256(&catalog_public).unwrap(),
                key: catalog_public,
                status: MarketplaceKeyStatus::Active,
                first_snapshot_version: 1,
                last_snapshot_version: None,
            }],
            releases: vec![PublicationReleasePlan {
                input_directory: "release".to_owned(),
                publisher_key_path: "publisher.public.json".to_owned(),
                release_path: "releases/door-legends/1/".to_owned(),
                policy_version: 1,
                status: CatalogStatus::Active,
                reason: "Reviewed first-party release.".to_owned(),
                reviewed_by: "omarchygs-review".to_owned(),
                review_summary: "Publisher and conformance review passed.".to_owned(),
            }],
            packages: vec![PublicationPackagePlan {
                input_path: "packages/omarchygs-client.pkg.tar.zst".to_owned(),
                relative_path: "packages/omarchygs-client.pkg.tar.zst".to_owned(),
                platform: "arch-linux".to_owned(),
                architecture: "x86_64".to_owned(),
                package_version: "0.1.0-1".to_owned(),
                filename: "omarchygs-client.pkg.tar.zst".to_owned(),
                source_revision: REVISION.to_owned(),
                source_sha256: "b".repeat(64),
                build_provenance_sha256: "c".repeat(64),
            }],
            previous_trust_sha256: None,
        };
        let plan_path = root.join("plan.json");
        write_json(&plan_path, &plan, 0o644);
        Self {
            _temp: temp,
            root,
            input,
            sdk,
            catalog_private,
            root_private,
            root_public,
            plan: plan_path,
        }
    }

    fn prepare(&self, name: &str) -> (PathBuf, omarchygs_marketplace_publisher::OperationReceipt) {
        let output = self.root.join(name);
        let receipt = prepare_publication(PrepareOptions {
            plan_path: &self.plan,
            input_root: &self.input,
            sdk_root: &self.sdk,
            catalog_private_key_path: &self.catalog_private,
            root_public_key_path: &self.root_public,
            previous_trust_path: None,
            output_root: &output,
        })
        .expect("publication should prepare");
        (output, receipt)
    }

    fn finalize(&self, name: &str) -> (PathBuf, String) {
        let (prepared, _) = self.prepare(&format!("prepared-{name}"));
        let response = self.root.join(format!("response-{name}.json"));
        offline_sign(
            &prepared.join("offline-request.json"),
            &self.root_private,
            &response,
        )
        .unwrap();
        let store = self.root.join(format!("store-{name}"));
        let receipt = finalize_publication(&prepared, &response, &store, NOW).unwrap();
        let version = format!(
            "{:020}-{}",
            receipt.bundle_version,
            receipt.publication_sha256.as_deref().unwrap()
        );
        (store, version)
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize, mode: u32) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

type HostedFiles = Arc<RwLock<HashMap<String, Vec<u8>>>>;

fn load_hosted_tree(root: &Path) -> HashMap<String, Vec<u8>> {
    fn walk(root: &Path, current: &Path, files: &mut HashMap<String, Vec<u8>>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                walk(root, &entry.path(), files);
            } else {
                files.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut files = HashMap::new();
    walk(root, root, &mut files);
    files
}

async fn spawn_mirror(files: HostedFiles) -> (SocketAddr, Vec<u8>, JoinHandle<()>) {
    let address = unused_loopback_address();
    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(vec!["mirror.example.test".to_owned()]).unwrap();
    let certificate = cert.der().to_vec();
    let tls = RustlsConfig::from_der(vec![certificate.clone()], signing_key.serialize_der())
        .await
        .unwrap();
    let app = Router::new()
        .route("/{*path}", get(serve_file))
        .with_state(files);
    let task = tokio::spawn(async move {
        axum_server::bind_rustls(address, tls)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
    wait_for_listener(address).await;
    (address, certificate, task)
}

async fn serve_file(
    State(files): State<HostedFiles>,
    AxumPath(path): AxumPath<String>,
) -> Response<Body> {
    match files.read().await.get(&path).cloned() {
        Some(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(
                "content-type",
                if path.ends_with(".json") {
                    "application/json"
                } else if path.ends_with(".pkg.tar.zst") {
                    "application/vnd.archlinux.package"
                } else {
                    "application/octet-stream"
                },
            )
            .body(Body::from(bytes))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

fn clients(
    address: SocketAddr,
    certificate: &[u8],
) -> (GuardedChannelClient, GuardedChannelClient) {
    let channel = ChannelOrigin::parse(&format!(
        "https://mirror.example.test:{}/channel/",
        address.port()
    ))
    .unwrap();
    let marketplace = ChannelOrigin::parse(&format!(
        "https://mirror.example.test:{}/marketplace/",
        address.port()
    ))
    .unwrap();
    (
        GuardedChannelClient::conformance_loopback(channel, address, certificate).unwrap(),
        GuardedChannelClient::conformance_loopback(marketplace, address, certificate).unwrap(),
    )
}

fn unused_loopback_address() -> SocketAddr {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    address
}

async fn wait_for_listener(address: SocketAddr) {
    for _ in 0..100 {
        if TcpStream::connect(address).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("TLS mirror failed to listen at {address}");
}

#[test]
fn public_hash_helper_is_lowercase_and_stable() {
    assert_eq!(publication_id(7), "publication-00000000000000000007");
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
