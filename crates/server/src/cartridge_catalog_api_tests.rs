use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt as _;

use crate::{
    accounts::{self, RegistrationInput},
    app::{router, router_with_runtimes},
    mfa::MfaCipher,
    sessions::{self, CreateSessionInput, SessionCreation},
    sync::SyncHub,
};
use omarchy_game_runtime::GameRegistry;
use omarchy_gaming_system_server::{
    cartridge_catalog::{
        CatalogCommand, CatalogSelection, ReviewedReleaseInput, apply_catalog_command,
        publish_snapshot, snapshot_sha256,
    },
    cartridge_distribution::CartridgeDistributionRuntime,
};
use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CatalogStatus, MarketplaceReleaseEntry, MarketplaceSnapshotPayload,
    SecureCartridgeStore, create_release, export_sdk, generate_catalog_keypair, generate_keypair,
    rich_2d_host_profile, sign_catalog_policy, sign_marketplace_snapshot, supported_sdk_identity,
    verify_acquisition_bytes, verify_catalog_policy, verify_release_directory,
};
use std::{fs, os::unix::fs::PermissionsExt as _, path::Path, sync::Arc};

const ARCHIVE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SIGNED_IDENTITY: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn authenticated_catalog_is_exact_no_store_and_lifecycle_filtered(pool: PgPool) {
    seed_deprecated_catalog(&pool).await;
    let token = create_session(&pool).await;

    let missing = get(router(pool.clone(), MfaCipher::test_cipher()), None).await;
    assert_eq!(missing.status, StatusCode::UNAUTHORIZED);
    assert_no_store(&missing);

    let invalid = get(
        router(pool.clone(), MfaCipher::test_cipher()),
        Some("invalid-token"),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::UNAUTHORIZED);
    assert_no_store(&invalid);

    let listed = get(router(pool.clone(), MfaCipher::test_cipher()), Some(&token)).await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_no_store(&listed);
    assert_eq!(
        listed.json(),
        json!({
            "cartridges": [{
                "game_key": "door-legends",
                "publisher_id": "ignibyte",
                "rules_version": 1,
                "cartridge_version": 2,
                "display_name": "Door Legends",
                "archive_sha256": ARCHIVE,
                "signed_identity_sha256": SIGNED_IDENTITY,
                "marketplace": {
                    "provenance_class": "marketplace_vetted",
                    "marketplace_id": "omarchygs-marketplace",
                    "marketplace_name": "OmarchyGS Marketplace",
                    "reviewed_by": "review-team",
                    "review_summary": "Bounded first-party review passed.",
                    "policy_version": 1,
                    "lifecycle_status": "deprecated"
                },
                "server_admission": {"revision": 4},
                "warning": "Upgrade when practical."
            }]
        })
    );
    for forbidden in [
        "release_path",
        "verifying_key",
        "PUBLIC-KEY-MATERIAL",
        "market.example.test",
        "/srv/cartridges",
        "javascript",
        "qml",
    ] {
        assert!(!listed.body.contains(forbidden), "leaked {forbidden}");
    }

    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET policy_version = 2,
            policy_status = 'suspended',
            policy_reason = 'Review paused.',
            signed_policy = '{"version":2}'::jsonb,
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $1
        "#,
    )
    .bind(ARCHIVE)
    .execute(&pool)
    .await
    .expect("newer suspended policy should apply");
    let hidden = get(router(pool.clone(), MfaCipher::test_cipher()), Some(&token)).await;
    assert_eq!(hidden.status, StatusCode::OK);
    assert_eq!(hidden.json(), json!({"cartridges": []}));

    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET policy_version = 3,
            policy_status = 'active',
            policy_reason = 'Review restored.',
            signed_policy = '{"version":3}'::jsonb,
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $1
        "#,
    )
    .bind(ARCHIVE)
    .execute(&pool)
    .await
    .expect("newer active policy should apply");
    sqlx::query(
        r#"
        UPDATE marketplace_sync_state
        SET snapshot_version = 2,
            snapshot_sha256 = $1,
            synchronized_at = clock_timestamp()
        WHERE singleton
        "#,
    )
    .bind("c".repeat(64))
    .execute(&pool)
    .await
    .expect("new snapshot should apply");
    let omitted = get(router(pool, MfaCipher::test_cipher()), Some(&token)).await;
    assert_eq!(omitted.status, StatusCode::OK);
    assert_eq!(omitted.json(), json!({"cartridges": []}));
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn exact_acquisition_is_authenticated_verified_and_current(pool: PgPool) {
    let fixture = acquisition_fixture(&pool).await;
    let token = create_session(&pool).await;
    let route = format!(
        "/v1/cartridges/{}/{}/acquisition",
        fixture.admission.game_key, fixture.admission.archive_sha256
    );
    let metadata_only = get_uri(
        router(pool.clone(), MfaCipher::test_cipher()),
        &route,
        Some(&token),
    )
    .await;
    assert_eq!(metadata_only.status, StatusCode::NOT_FOUND);

    let app = router_with_runtimes(
        pool.clone(),
        MfaCipher::test_cipher(),
        SyncHub::new(),
        GameRegistry::empty(),
        None,
        Some(fixture.runtime),
        Arc::from("Acquisition Test"),
    );
    let missing = get_uri(app.clone(), &route, None).await;
    assert_eq!(missing.status, StatusCode::UNAUTHORIZED);
    assert_no_store(&missing);
    let response = get_uri(app.clone(), &route, Some(&token)).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_no_store(&response);
    assert_eq!(
        response
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    let sdk = supported_sdk_identity().expect("supported SDK should load");
    let verified = verify_acquisition_bytes(
        response.body.as_bytes(),
        &fixture.admission,
        &fixture.marketplace_public,
        &sdk,
        &rich_2d_host_profile(),
    )
    .expect("acquisition should independently verify");
    assert_eq!(
        verified.release().payload().archive_sha256,
        fixture.admission.archive_sha256
    );
    for forbidden in [
        fixture.store_root.to_string_lossy().as_ref(),
        "release_path",
        "https://market.example.test",
        "private_key",
        "javascript",
        "qml",
    ] {
        assert!(!response.body.contains(forbidden), "leaked {forbidden}");
    }

    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET policy_version = policy_version + 1,
            policy_status = 'suspended',
            policy_reason = 'Review paused.',
            signed_policy = '{"version":999}'::jsonb,
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $1
        "#,
    )
    .bind(&fixture.admission.archive_sha256)
    .execute(&pool)
    .await
    .expect("new denial should apply");
    let denied = get_uri(app, &route, Some(&token)).await;
    assert_eq!(denied.status, StatusCode::NOT_FOUND);
    assert_no_store(&denied);
}

struct AcquisitionFixture {
    _temp: tempfile::TempDir,
    store_root: std::path::PathBuf,
    runtime: CartridgeDistributionRuntime,
    marketplace_public: omarchygs_game_cartridge::CatalogPublicKey,
    admission: AcquisitionServerAdmission,
}

async fn acquisition_fixture(pool: &PgPool) -> AcquisitionFixture {
    let temp = tempfile::tempdir().expect("fixture temp should create");
    let sdk_root = temp.path().join("sdk");
    let release_root = temp.path().join("release");
    let store_root = temp.path().join("store");
    for path in [&sdk_root, &release_root, &store_root] {
        fs::create_dir(path).expect("fixture directory should create");
    }
    fs::set_permissions(&store_root, fs::Permissions::from_mode(0o700))
        .expect("store should be private");
    export_sdk(&sdk_root).expect("SDK should export");
    let (publisher_private, publisher_public) =
        generate_keypair("publisher-primary-v1", "ignibyte")
            .expect("publisher key should generate");
    create_release(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/first-party-door-legends/cartridge"),
        &publisher_private,
        &sdk_root,
        "1111111111111111111111111111111111111111",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        &rich_2d_host_profile(),
        &release_root,
    )
    .expect("release should create");
    let release = verify_release_directory(
        &release_root,
        &publisher_public,
        &sdk_root,
        &rich_2d_host_profile(),
    )
    .expect("release should verify");
    let (marketplace_private, marketplace_public) =
        generate_catalog_keypair("marketplace-primary-v1", "omarchygs-marketplace")
            .expect("marketplace key should generate");
    let signed_policy = sign_catalog_policy(
        &release,
        &marketplace_private,
        1,
        CatalogStatus::Active,
        "Reviewed exact release.",
    )
    .expect("policy should sign");
    let policy = verify_catalog_policy(&signed_policy, &marketplace_public, &release)
        .expect("policy should verify");
    let entry = MarketplaceReleaseEntry {
        release_path: "releases/door-legends/1/".to_owned(),
        game_key: release.payload().game_key.clone(),
        publisher_id: release.payload().publisher_id.clone(),
        rules_version: release.payload().rules_version,
        cartridge_version: release.payload().cartridge_version,
        archive_sha256: release.payload().archive_sha256.clone(),
        signed_identity_sha256: release.payload().signed_identity_sha256.clone(),
        publisher_key: publisher_public,
        reviewed_by: "review-team".to_owned(),
        review_summary: "Bounded first-party review passed.".to_owned(),
        policy: signed_policy,
    };
    let payload = MarketplaceSnapshotPayload {
        format: "omarchygs.marketplace-snapshot/v1".to_owned(),
        snapshot_version: 1,
        authority_id: marketplace_public.authority_id.clone(),
        marketplace_name: "OmarchyGS Marketplace".to_owned(),
        releases: vec![entry.clone()],
    };
    let signed_snapshot =
        sign_marketplace_snapshot(&payload, &marketplace_private).expect("snapshot should sign");
    let signed_snapshot_bytes =
        serde_json::to_vec(&signed_snapshot).expect("snapshot should serialize");
    let store = SecureCartridgeStore::open_existing(&store_root).expect("store should open");
    let staged = store
        .stage_reviewed_release(
            &release,
            &entry.policy_bytes().expect("policy should serialize"),
            &marketplace_public,
        )
        .expect("release should stage");
    assert!(staged.installed);
    let digest = snapshot_sha256(&signed_snapshot_bytes);
    publish_snapshot(
        pool,
        "https://market.example.test",
        &marketplace_public,
        &payload,
        &digest,
        &signed_snapshot_bytes,
        &[ReviewedReleaseInput {
            entry: entry.clone(),
            policy,
            display_name: release.cartridge().manifest().display_name.clone(),
            compatible: true,
            imported: true,
        }],
    )
    .await
    .expect("snapshot should publish");
    let command = CatalogCommand {
        idempotency_key: uuid::Uuid::new_v4(),
        game_key: entry.game_key.clone(),
        expected: CatalogSelection::Inactive,
        desired: CatalogSelection::Release {
            archive_sha256: entry.archive_sha256.clone(),
        },
        actor: "test-operator".to_owned(),
        reason: "Exercise authenticated acquisition.".to_owned(),
    };
    let receipt = apply_catalog_command(
        pool,
        &store,
        &marketplace_public,
        &rich_2d_host_profile(),
        &command,
    )
    .await
    .expect("release should activate");
    let server_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton")
            .fetch_one(pool)
            .await
            .expect("server identity should exist");
    let admission = AcquisitionServerAdmission {
        server_id: server_id.to_string(),
        game_key: entry.game_key,
        publisher_id: entry.publisher_id,
        rules_version: entry.rules_version,
        cartridge_version: entry.cartridge_version,
        archive_sha256: entry.archive_sha256,
        signed_identity_sha256: entry.signed_identity_sha256,
        admission_revision: receipt.admission_revision,
    };
    let runtime = CartridgeDistributionRuntime::from_verified_store(
        SecureCartridgeStore::open_existing(&store_root).expect("runtime store should open"),
        marketplace_public.clone(),
    );
    AcquisitionFixture {
        _temp: temp,
        store_root,
        runtime,
        marketplace_public,
        admission,
    }
}

async fn seed_deprecated_catalog(pool: &PgPool) {
    sqlx::query(
        r#"
        INSERT INTO marketplace_sync_state (
            marketplace_origin, authority_id, key_id, marketplace_name,
            snapshot_version, snapshot_sha256
        )
        VALUES (
            'https://market.example.test/v1/', 'omarchygs-marketplace',
            'marketplace-primary-v1', 'OmarchyGS Marketplace', 1, $1
        )
        "#,
    )
    .bind("d".repeat(64))
    .execute(pool)
    .await
    .expect("sync state should insert");
    let release_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO marketplace_releases (
            game_key, publisher_id, publisher_key, rules_version,
            cartridge_version, archive_sha256, signed_identity_sha256,
            display_name, release_path, reviewed_by, review_summary,
            signed_policy, policy_version, policy_status, policy_reason,
            compatible, imported, first_seen_snapshot_version,
            last_seen_snapshot_version
        )
        VALUES (
            'door-legends', 'ignibyte', $1, 1, 2, $2, $3,
            'Door Legends', 'releases/door-legends/2/', 'review-team',
            'Bounded first-party review passed.', $4, 1, 'deprecated',
            'Upgrade when practical.', TRUE, TRUE, 1, 1
        )
        RETURNING id
        "#,
    )
    .bind(json!({
        "format_version": 1,
        "algorithm": "ed25519",
        "key_id": "publisher-primary-v1",
        "publisher_id": "ignibyte",
        "verifying_key": "PUBLIC-KEY-MATERIAL"
    }))
    .bind(ARCHIVE)
    .bind(SIGNED_IDENTITY)
    .bind(json!({"version": 1}))
    .fetch_one(pool)
    .await
    .expect("reviewed release should insert");
    sqlx::query(
        r#"
        INSERT INTO server_cartridge_catalogs (
            game_key, active_release_id, admission_revision
        )
        VALUES ('door-legends', $1, 4)
        "#,
    )
    .bind(release_id)
    .execute(pool)
    .await
    .expect("catalog selection should insert");
}

async fn create_session(pool: &PgPool) -> String {
    accounts::register_account(
        pool,
        RegistrationInput {
            invite_code: accounts::create_test_invite(pool).await,
            username: "catalog_player".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await
    .expect("test account should register");
    match sessions::create_session(
        pool,
        CreateSessionInput {
            username: "catalog_player".to_owned(),
            password: "correct horse battery staple".to_owned(),
            device_name: "catalog api test".to_owned(),
        },
    )
    .await
    .expect("test session should create")
    {
        SessionCreation::Created(created) => created.token,
        SessionCreation::MfaRequired(_) => panic!("new test account should not require MFA"),
    }
}

struct TestResponse {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: String,
}

impl TestResponse {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("response should be JSON")
    }
}

async fn get(app: Router, token: Option<&str>) -> TestResponse {
    get_uri(app, "/v1/cartridges", token).await
}

async fn get_uri(app: Router, uri: &str, token: Option<&str>) -> TestResponse {
    let mut request = Request::builder().uri(uri);
    if let Some(token) = token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .oneshot(request.body(Body::empty()).expect("request should build"))
        .await
        .expect("router should respond");
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should read")
        .to_bytes();
    TestResponse {
        status,
        headers,
        body: String::from_utf8(body.to_vec()).expect("body should be UTF-8"),
    }
}

fn assert_no_store(response: &TestResponse) {
    assert_eq!(
        response
            .headers
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
}
