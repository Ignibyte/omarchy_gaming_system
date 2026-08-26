use std::{
    collections::HashMap,
    fs,
    net::{Ipv4Addr, SocketAddr},
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{Response, StatusCode, header::LOCATION},
    routing::get,
};
use axum_server::tls_rustls::RustlsConfig;
use omarchygs_game_cartridge::{
    CatalogPrivateKey, CatalogPublicKey, CatalogStatus, MarketplaceReleaseEntry,
    MarketplaceSnapshotPayload, PublisherPublicKey, VerifiedRelease, create_release, export_sdk,
    generate_catalog_keypair, generate_keypair, rich_2d_host_profile, sign_catalog_policy,
    sign_marketplace_snapshot, verify_release_directory,
};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use sqlx::PgPool;
use tokio::{net::TcpStream, sync::RwLock, task::JoinHandle};
use uuid::Uuid;

use crate::{
    cartridge_catalog::{
        CatalogCommand, CatalogError, CatalogSelection, apply_catalog_command, list_inventory,
        list_player_catalog, snapshot_sha256,
    },
    marketplace_egress::{GuardedMarketplaceClient, MarketplaceOrigin},
    marketplace_sync::{
        LocalCatalogConfig, MarketplaceSyncConfig, MarketplaceSyncError, synchronize_with_client,
    },
};

const MARKETPLACE_HOST: &str = "market.example.test";
const REVISION_ONE: &str = "1111111111111111111111111111111111111111";
const REVISION_TWO: &str = "2222222222222222222222222222222222222222";
const BUILDER_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL and the separately spawned TLS marketplace fixture"]
async fn tls_sync_activation_race_rollback_lifecycle_and_audit_are_exact(pool: PgPool) {
    let generated = GeneratedMarketplace::new();
    let snapshot_one = generated.snapshot(1, CatalogStatus::Active, CatalogStatus::Active, 1);
    generated
        .files
        .write()
        .await
        .insert("snapshot.signed.json".to_owned(), snapshot_one.clone());
    let (address, certificate, server) = spawn_marketplace(Arc::clone(&generated.files)).await;
    let origin = MarketplaceOrigin::parse(&format!(
        "https://{MARKETPLACE_HOST}:{}/v1/",
        address.port()
    ))
    .expect("fixture origin should parse");
    let local = LocalCatalogConfig {
        marketplace_key: generated.catalog_public.clone(),
        store_root: generated.store_root.clone(),
    };
    let config = MarketplaceSyncConfig::for_test(local, origin.clone(), certificate.clone());
    let client = GuardedMarketplaceClient::conformance_loopback(origin, address, &certificate)
        .expect("exact loopback client should construct");

    let first = synchronize_with_client(&pool, &config, &client)
        .await
        .expect("first snapshot should synchronize");
    assert_eq!(first.snapshot_version, 1);
    assert_eq!(first.releases, 2);
    assert_eq!(first.imported, 2);
    assert!(!first.replayed);
    assert_eq!(
        fs::read_dir(generated.store_root.join("active"))
            .expect("active directory should exist")
            .count(),
        0,
        "marketplace synchronization must not grant local activation"
    );

    let replay = synchronize_with_client(&pool, &config, &client)
        .await
        .expect("exact snapshot replay should pass");
    assert!(replay.replayed);
    let inventory = list_inventory(&pool)
        .await
        .expect("inventory should be readable");
    assert_eq!(inventory.releases.len(), 2);
    assert!(inventory.releases.iter().all(|release| {
        release.present && release.imported && !release.selected && !release.effective
    }));

    let store = config.local.open_store().expect("store should reopen");
    let host = rich_2d_host_profile();
    let release_one = &generated.releases[0];
    let release_two = &generated.releases[1];
    let activate_two = command(
        1,
        CatalogSelection::Inactive,
        selection(release_two),
        "activate current release",
    );
    let activated = apply_catalog_command(
        &pool,
        &store,
        &generated.catalog_public,
        &host,
        &activate_two,
    )
    .await
    .expect("exact release should activate");
    assert_eq!(activated.action, "activate_cartridge");
    assert_eq!(activated.admission_revision, 1);
    assert_eq!(
        apply_catalog_command(
            &pool,
            &store,
            &generated.catalog_public,
            &host,
            &activate_two,
        )
        .await,
        Ok(activated.clone())
    );
    let mut collision = activate_two.clone();
    collision.reason = "different intent".to_owned();
    assert_eq!(
        apply_catalog_command(&pool, &store, &generated.catalog_public, &host, &collision,).await,
        Err(CatalogError::Conflict)
    );
    let player = list_player_catalog(&pool)
        .await
        .expect("effective player catalog should list");
    assert_eq!(player.len(), 1);
    assert_eq!(player[0].archive_sha256, release_two.archive_sha256());

    let deactivate = command(
        2,
        selection(release_two),
        CatalogSelection::Inactive,
        "prepare race",
    );
    apply_catalog_command(&pool, &store, &generated.catalog_public, &host, &deactivate)
        .await
        .expect("catalog should deactivate");
    let race_one = command(
        3,
        CatalogSelection::Inactive,
        selection(release_one),
        "race one",
    );
    let race_two = command(
        4,
        CatalogSelection::Inactive,
        selection(release_two),
        "race two",
    );
    let (first_race, second_race) = tokio::join!(
        apply_catalog_command(&pool, &store, &generated.catalog_public, &host, &race_one,),
        apply_catalog_command(&pool, &store, &generated.catalog_public, &host, &race_two,)
    );
    assert_eq!(
        [first_race.is_ok(), second_race.is_ok()]
            .into_iter()
            .filter(|ok| *ok)
            .count(),
        1
    );
    assert!(matches!(
        (&first_race, &second_race),
        (Ok(_), Err(CatalogError::Conflict)) | (Err(CatalogError::Conflict), Ok(_))
    ));
    let raced_digest = first_race
        .as_ref()
        .ok()
        .or_else(|| second_race.as_ref().ok())
        .and_then(|receipt| receipt.resulting_archive_sha256.clone())
        .expect("one race winner should select a digest");
    let raced_release = if raced_digest == release_one.archive_sha256() {
        release_one
    } else {
        release_two
    };
    apply_catalog_command(
        &pool,
        &store,
        &generated.catalog_public,
        &host,
        &command(
            5,
            selection(raced_release),
            CatalogSelection::Inactive,
            "clear race winner",
        ),
    )
    .await
    .expect("race winner should deactivate");
    apply_catalog_command(
        &pool,
        &store,
        &generated.catalog_public,
        &host,
        &command(
            6,
            CatalogSelection::Inactive,
            selection(release_two),
            "activate newest",
        ),
    )
    .await
    .expect("newest should activate");
    let rollback = apply_catalog_command(
        &pool,
        &store,
        &generated.catalog_public,
        &host,
        &command(
            7,
            selection(release_two),
            selection(release_one),
            "rollback exact release",
        ),
    )
    .await
    .expect("older permitted release should roll back");
    assert_eq!(rollback.action, "rollback_cartridge");

    let snapshot_two = generated.snapshot(2, CatalogStatus::Suspended, CatalogStatus::Active, 2);
    generated
        .files
        .write()
        .await
        .insert("snapshot.signed.json".to_owned(), snapshot_two);
    let lifecycle = synchronize_with_client(&pool, &config, &client)
        .await
        .expect("newer lifecycle snapshot should synchronize");
    assert_eq!(lifecycle.snapshot_version, 2);
    assert_eq!(lifecycle.imported, 1);
    assert!(
        list_player_catalog(&pool)
            .await
            .expect("player catalog should read")
            .is_empty(),
        "suspended selected release must become ineffective without fallback"
    );
    assert_eq!(
        apply_catalog_command(
            &pool,
            &store,
            &generated.catalog_public,
            &host,
            &command(
                8,
                selection(release_one),
                selection(release_two),
                "explicit recovery",
            ),
        )
        .await
        .expect("operator may explicitly select the permitted release")
        .action,
        "upgrade_cartridge"
    );
    assert_eq!(list_player_catalog(&pool).await.unwrap().len(), 1);
    assert_eq!(
        apply_catalog_command(
            &pool,
            &store,
            &generated.catalog_public,
            &host,
            &command(
                9,
                selection(release_two),
                selection(release_one),
                "denied rollback",
            ),
        )
        .await,
        Err(CatalogError::Denied)
    );

    generated
        .files
        .write()
        .await
        .insert("snapshot.signed.json".to_owned(), snapshot_one.clone());
    assert_eq!(
        synchronize_with_client(&pool, &config, &client).await,
        Err(MarketplaceSyncError::Conflict)
    );
    let snapshot_three = generated.snapshot(3, CatalogStatus::Active, CatalogStatus::Active, 3);
    let archive_path = format!("{}cartridge.ogsc", release_two.release_path);
    let original_archive = generated
        .files
        .write()
        .await
        .insert(archive_path.clone(), b"tampered".to_vec())
        .expect("fixture archive should exist");
    generated
        .files
        .write()
        .await
        .insert("snapshot.signed.json".to_owned(), snapshot_three);
    assert_eq!(
        synchronize_with_client(&pool, &config, &client).await,
        Err(MarketplaceSyncError::Rejected)
    );
    generated
        .files
        .write()
        .await
        .insert(archive_path, original_archive);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT snapshot_version FROM marketplace_sync_state WHERE singleton",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        2,
        "failed snapshot must not publish partial inventory"
    );

    let audit_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM cartridge_catalog_audit_events ORDER BY created_at LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("audit row should exist");
    assert!(
        sqlx::query("UPDATE cartridge_catalog_audit_events SET reason = 'changed' WHERE id = $1")
            .bind(audit_id)
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("DELETE FROM cartridge_catalog_audit_events WHERE id = $1")
            .bind(audit_id)
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("TRUNCATE cartridge_catalog_audit_events")
            .execute(&pool)
            .await
            .is_err()
    );

    server.abort();
}

#[test]
fn signed_snapshot_digest_is_stable() {
    let generated = GeneratedMarketplace::new();
    let first = generated.snapshot(1, CatalogStatus::Active, CatalogStatus::Active, 1);
    let second = generated.snapshot(1, CatalogStatus::Active, CatalogStatus::Active, 1);
    assert_eq!(first, second);
    assert_eq!(snapshot_sha256(&first), snapshot_sha256(&second));
}

#[tokio::test]
async fn guarded_marketplace_tls_rejects_wrong_root_redirect_and_oversized_body() {
    let files = Arc::new(RwLock::new(HashMap::from([
        ("small".to_owned(), b"bounded".to_vec()),
        ("oversized".to_owned(), vec![b'x'; 17]),
    ])));
    let (address, certificate, server) = spawn_marketplace(files).await;
    let origin = MarketplaceOrigin::parse(&format!(
        "https://{MARKETPLACE_HOST}:{}/v1/",
        address.port()
    ))
    .expect("fixture origin should parse");
    let client =
        GuardedMarketplaceClient::conformance_loopback(origin.clone(), address, &certificate)
            .expect("fixture client should construct");

    assert_eq!(client.get("small", 7).await, Ok(b"bounded".to_vec()));
    assert_eq!(
        client.get("oversized", 16).await,
        Err(crate::marketplace_egress::MarketplaceEgressError::Rejected)
    );
    assert_eq!(
        client.get("redirect", 16).await,
        Err(crate::marketplace_egress::MarketplaceEgressError::Rejected)
    );

    let CertifiedKey {
        cert: wrong_cert, ..
    } = generate_simple_self_signed(vec![MARKETPLACE_HOST.to_owned()])
        .expect("wrong trust root should generate");
    let wrong_root =
        GuardedMarketplaceClient::conformance_loopback(origin, address, wrong_cert.der().as_ref())
            .expect("valid but wrong trust root should configure");
    assert_eq!(
        wrong_root.get("small", 7).await,
        Err(crate::marketplace_egress::MarketplaceEgressError::Unavailable)
    );
    server.abort();
}

struct GeneratedRelease {
    verified: VerifiedRelease,
    publisher_key: PublisherPublicKey,
    release_root: PathBuf,
    release_path: String,
}

impl GeneratedRelease {
    fn archive_sha256(&self) -> String {
        self.verified.payload().archive_sha256.clone()
    }
}

struct GeneratedMarketplace {
    _temp: tempfile::TempDir,
    store_root: PathBuf,
    catalog_private: CatalogPrivateKey,
    catalog_public: CatalogPublicKey,
    releases: Vec<GeneratedRelease>,
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl GeneratedMarketplace {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("fixture temp directory should create");
        let sdk = temp.path().join("sdk");
        let source_one = temp.path().join("source-one");
        let source_two = temp.path().join("source-two");
        let release_one = temp.path().join("release-one");
        let release_two = temp.path().join("release-two");
        let store_root = temp.path().join("store");
        for path in [&sdk, &release_one, &release_two, &store_root] {
            fs::create_dir(path).expect("fixture directory should create");
        }
        fs::set_permissions(&store_root, fs::Permissions::from_mode(0o700))
            .expect("store permissions should set");
        export_sdk(&sdk).expect("SDK should export");
        let example = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/first-party-door-legends/cartridge");
        copy_tree(&example, &source_one);
        copy_tree(&example, &source_two);
        set_release_manifest(&source_one, 1, "Door Legends Classic");
        set_release_manifest(&source_two, 2, "Door Legends");
        let (publisher_private, publisher_public) =
            generate_keypair("ignibyte-primary-v1", "ignibyte")
                .expect("publisher key should generate");
        create_release(
            &source_one,
            &publisher_private,
            &sdk,
            REVISION_ONE,
            BUILDER_DIGEST,
            &rich_2d_host_profile(),
            &release_one,
        )
        .expect("release one should build");
        create_release(
            &source_two,
            &publisher_private,
            &sdk,
            REVISION_TWO,
            BUILDER_DIGEST,
            &rich_2d_host_profile(),
            &release_two,
        )
        .expect("release two should build");
        let verified_one = verify_release_directory(
            &release_one,
            &publisher_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .expect("release one should verify");
        let verified_two = verify_release_directory(
            &release_two,
            &publisher_public,
            &sdk,
            &rich_2d_host_profile(),
        )
        .expect("release two should verify");
        let releases = vec![
            GeneratedRelease {
                verified: verified_one,
                publisher_key: publisher_public.clone(),
                release_root: release_one,
                release_path: "releases/door-legends/1/".to_owned(),
            },
            GeneratedRelease {
                verified: verified_two,
                publisher_key: publisher_public,
                release_root: release_two,
                release_path: "releases/door-legends/2/".to_owned(),
            },
        ];
        let mut files = HashMap::new();
        for release in &releases {
            for name in ["cartridge.ogsc", "conformance.json", "release.signed.json"] {
                files.insert(
                    format!("{}{name}", release.release_path),
                    fs::read(release.release_root.join(name))
                        .expect("release component should read"),
                );
            }
        }
        let (catalog_private, catalog_public) =
            generate_catalog_keypair("marketplace-primary-v1", "omarchygs-marketplace")
                .expect("catalog key should generate");
        Self {
            _temp: temp,
            store_root,
            catalog_private,
            catalog_public,
            releases,
            files: Arc::new(RwLock::new(files)),
        }
    }

    fn snapshot(
        &self,
        snapshot_version: u64,
        first_status: CatalogStatus,
        second_status: CatalogStatus,
        policy_version: u64,
    ) -> Vec<u8> {
        let statuses = [first_status, second_status];
        let entries = self
            .releases
            .iter()
            .zip(statuses)
            .map(|(release, status)| {
                let policy = sign_catalog_policy(
                    &release.verified,
                    &self.catalog_private,
                    policy_version,
                    status,
                    match status {
                        CatalogStatus::Active => "Marketplace review active.",
                        CatalogStatus::Deprecated => "Marketplace release deprecated.",
                        CatalogStatus::Suspended => "Marketplace review suspended.",
                        CatalogStatus::Revoked => "Marketplace release revoked.",
                        CatalogStatus::Retired => "Marketplace release retired.",
                    },
                )
                .expect("policy should sign");
                MarketplaceReleaseEntry {
                    release_path: release.release_path.clone(),
                    game_key: release.verified.payload().game_key.clone(),
                    publisher_id: release.verified.payload().publisher_id.clone(),
                    rules_version: release.verified.payload().rules_version,
                    cartridge_version: release.verified.payload().cartridge_version,
                    archive_sha256: release.verified.payload().archive_sha256.clone(),
                    signed_identity_sha256: release
                        .verified
                        .payload()
                        .signed_identity_sha256
                        .clone(),
                    publisher_key: release.publisher_key.clone(),
                    reviewed_by: "omarchygs-review".to_owned(),
                    review_summary: "Bounded first-party review passed.".to_owned(),
                    policy,
                }
            })
            .collect();
        let payload = MarketplaceSnapshotPayload {
            format: "omarchygs.marketplace-snapshot/v1".to_owned(),
            snapshot_version,
            authority_id: self.catalog_public.authority_id.clone(),
            marketplace_name: "OmarchyGS Marketplace".to_owned(),
            releases: entries,
        };
        serde_json::to_vec(
            &sign_marketplace_snapshot(&payload, &self.catalog_private)
                .expect("snapshot should sign"),
        )
        .expect("snapshot should serialize")
    }
}

fn command(
    sequence: u128,
    expected: CatalogSelection,
    desired: CatalogSelection,
    reason: &str,
) -> CatalogCommand {
    CatalogCommand {
        idempotency_key: Uuid::from_u128(0x0320_0000_0000_0000_0000_0000_0000_0000 + sequence),
        game_key: "door-legends".to_owned(),
        expected,
        desired,
        actor: "fixture-admin".to_owned(),
        reason: reason.to_owned(),
    }
}

fn selection(release: &GeneratedRelease) -> CatalogSelection {
    CatalogSelection::Release {
        archive_sha256: release.archive_sha256(),
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir(destination).expect("destination should create");
    for entry in fs::read_dir(source).expect("source should read") {
        let entry = entry.expect("source entry should read");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("entry type should read").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("source file should copy");
        }
    }
}

fn set_release_manifest(source: &Path, version: u32, display_name: &str) {
    let path = source.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("manifest should read"))
            .expect("manifest should parse");
    manifest["cartridge_version"] = serde_json::json!(version);
    manifest["display_name"] = serde_json::json!(display_name);
    fs::write(
        path,
        serde_json::to_vec(&manifest).expect("manifest should serialize"),
    )
    .expect("manifest should write");
}

type FixtureFiles = Arc<RwLock<HashMap<String, Vec<u8>>>>;

async fn spawn_marketplace(files: FixtureFiles) -> (SocketAddr, Vec<u8>, JoinHandle<()>) {
    let address = unused_loopback_address();
    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(vec![MARKETPLACE_HOST.to_owned()])
            .expect("TLS fixture certificate should generate");
    let certificate_der = cert.der().to_vec();
    let tls = RustlsConfig::from_der(vec![certificate_der.clone()], signing_key.serialize_der())
        .await
        .expect("TLS config should load");
    let app = Router::new()
        .route("/v1/{*path}", get(serve_fixture_file))
        .with_state(files);
    let task = tokio::spawn(async move {
        axum_server::bind_rustls(address, tls)
            .serve(app.into_make_service())
            .await
            .expect("fixture marketplace should run");
    });
    wait_for_listener(address).await;
    (address, certificate_der, task)
}

async fn serve_fixture_file(
    State(files): State<FixtureFiles>,
    AxumPath(path): AxumPath<String>,
) -> Response<Body> {
    if path == "redirect" {
        return Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "https://attacker.example.invalid/escape")
            .body(Body::empty())
            .expect("redirect response should build");
    }
    match files.read().await.get(&path).cloned() {
        Some(bytes) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .expect("fixture response should build"),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found"))
            .expect("fixture response should build"),
    }
}

fn unused_loopback_address() -> SocketAddr {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("ephemeral listener should bind");
    let address = listener.local_addr().expect("listener address should read");
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
    panic!("TLS fixture failed to listen at {address}");
}
