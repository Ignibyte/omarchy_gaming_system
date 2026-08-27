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
    personas::{self, CreatePersonaInput},
    sessions::{self, CreateSessionInput, SessionCreation},
    sync::SyncHub,
};
use omarchy_game_runtime::GameRegistry;
use omarchy_gaming_system_server::{
    cartridge_catalog::{
        CatalogCommand, CatalogSelection, ReviewedReleaseInput, SnapshotPublication,
        apply_catalog_command, publish_snapshot, snapshot_sha256,
    },
    cartridge_distribution::CartridgeDistributionRuntime,
    session_cartridges,
};
use omarchygs_game_cartridge::{
    AcquisitionServerAdmission, CatalogStatus, MarketplaceReleaseEntry, MarketplaceSnapshotPayload,
    SecureCartridgeStore, SignedCatalogPolicy, create_release, export_sdk,
    generate_catalog_keypair, generate_keypair, rich_2d_host_profile, sign_catalog_policy,
    sign_marketplace_snapshot, supported_sdk_identity, verify_acquisition_bytes,
    verify_catalog_policy, verify_release_directory,
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
            policy_snapshot_version = policy_snapshot_version + 1,
            last_seen_snapshot_version = last_seen_snapshot_version + 1,
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
            policy_snapshot_version = policy_snapshot_version + 1,
            last_seen_snapshot_version = last_seen_snapshot_version + 1,
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
    let mut mismatch = pool
        .begin()
        .await
        .expect("mismatch transaction should begin");
    let mismatch_session: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO game_sessions (game_key, game_version, state, authority)
        VALUES ('door-legends', 1, '{}'::jsonb, 'platform_compiled')
        RETURNING id
        "#,
    )
    .fetch_one(&mut *mismatch)
    .await
    .expect("mismatch session should insert");
    let mismatched_digest = "f".repeat(64);
    assert!(
        !session_cartridges::pin_new_session(
            &mut mismatch,
            &fixture.runtime,
            mismatch_session,
            "door-legends",
            1,
            Some(&mismatched_digest),
        )
        .await
        .expect("provider digest mismatch should be an honest unbound session")
    );
    mismatch
        .commit()
        .await
        .expect("unbound mismatch session should commit");
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM game_session_cartridge_presentations WHERE game_session_id = $1",
        )
        .bind(mismatch_session)
        .fetch_one(&pool)
        .await
        .expect("mismatch binding count should read"),
        0
    );
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
            policy_snapshot_version = policy_snapshot_version + 1,
            last_seen_snapshot_version = last_seen_snapshot_version + 1,
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

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn participant_acquires_the_historical_exact_session_pin_without_current_selection(
    pool: PgPool,
) {
    let fixture = acquisition_fixture(&pool).await;
    let token = create_session(&pool).await;
    let persona = personas::create_persona(
        &pool,
        &token,
        CreatePersonaInput {
            handle: "historical_pin_player".to_owned(),
            display_name: "Historical Pin Player".to_owned(),
            bio: String::new(),
            status_message: String::new(),
        },
    )
    .await
    .expect("participant persona should create");
    let session_id = seed_cartridge_action_session(&pool, &fixture, persona.id).await;
    sqlx::query(
        r#"
        UPDATE server_cartridge_catalogs
        SET active_release_id = NULL,
            admission_revision = admission_revision + 1,
            updated_at = clock_timestamp()
        WHERE game_key = 'door-legends'
        "#,
    )
    .execute(&pool)
    .await
    .expect("current catalog selection should advance away from the pinned release");

    let app = router_with_runtimes(
        pool.clone(),
        MfaCipher::test_cipher(),
        SyncHub::new(),
        GameRegistry::empty(),
        None,
        Some(fixture.runtime.clone()),
        Arc::from("Historical Acquisition Test"),
    );
    let current_route = format!(
        "/v1/cartridges/{}/{}/acquisition",
        fixture.admission.game_key, fixture.admission.archive_sha256
    );
    assert_eq!(
        get_uri(app.clone(), &current_route, Some(&token))
            .await
            .status,
        StatusCode::NOT_FOUND,
        "the old release must no longer be a current catalog selection"
    );
    let session_route = format!(
        "/v1/personas/{}/game-sessions/{}/cartridge-acquisition",
        persona.id, session_id
    );
    let unauthorized = get_uri(app.clone(), &session_route, None).await;
    assert_eq!(unauthorized.status, StatusCode::UNAUTHORIZED);
    assert_no_store(&unauthorized);
    let response = get_uri(app, &session_route, Some(&token)).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_no_store(&response);
    let verified = verify_acquisition_bytes(
        response.body.as_bytes(),
        &fixture.admission,
        &fixture.marketplace_public,
        &supported_sdk_identity().expect("supported SDK should load"),
        &rich_2d_host_profile(),
    )
    .expect("historical session acquisition should independently verify");
    assert_eq!(
        verified.release().payload().archive_sha256,
        fixture.admission.archive_sha256
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM marketplace_release_acquisition_evidence",
        )
        .fetch_one(&pool)
        .await
        .expect("retained evidence count should read"),
        1
    );
    assert!(
        sqlx::query("DELETE FROM marketplace_snapshot_acquisition_evidence")
            .execute(&pool)
            .await
            .is_err(),
        "retained signed acquisition evidence must be immutable"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn cartridge_action_admission_is_exact_immutable_and_survives_later_revocation(pool: PgPool) {
    let fixture = acquisition_fixture(&pool).await;
    let token = create_session(&pool).await;
    let persona = personas::create_persona(
        &pool,
        &token,
        CreatePersonaInput {
            handle: "catalog_action_player".to_owned(),
            display_name: "Catalog Action Player".to_owned(),
            bio: String::new(),
            status_message: String::new(),
        },
    )
    .await
    .expect("test persona should create");
    let session_id = seed_cartridge_action_session(&pool, &fixture, persona.id).await;
    let idempotency_key = uuid::Uuid::new_v4();
    let admitted = session_cartridges::admit_session_action(
        &pool,
        &fixture.runtime,
        persona.id,
        session_id,
        idempotency_key,
        0,
        &fixture.admission.archive_sha256,
        Some("lobby"),
        "enter",
        &json!({}),
    )
    .await
    .expect("active signed action should be admitted");
    assert_eq!(admitted.authority, "platform_compiled");
    assert_eq!(admitted.command, json!({"action": "enter"}));
    let secondary_key = uuid::Uuid::new_v4();
    let secondary = session_cartridges::admit_session_action(
        &pool,
        &fixture.runtime,
        persona.id,
        session_id,
        secondary_key,
        0,
        &fixture.admission.archive_sha256,
        Some("chronicle"),
        "enter",
        &json!({}),
    )
    .await
    .expect("gameplay emitted by the signed secondary screen should be admitted");
    assert_eq!(secondary, admitted);
    assert_eq!(
        session_cartridges::admit_session_action(
            &pool,
            &fixture.runtime,
            persona.id,
            session_id,
            secondary_key,
            0,
            &fixture.admission.archive_sha256,
            Some("lobby"),
            "enter",
            &json!({}),
        )
        .await,
        Err(session_cartridges::SessionCartridgeError::IdempotencyConflict),
        "a replay cannot move an admitted action across screens"
    );
    for (screen, action) in [
        ("unknown", "enter"),
        ("lobby", "navigate.chronicle"),
        ("chronicle", "navigate.lobby"),
    ] {
        assert_eq!(
            session_cartridges::admit_session_action(
                &pool,
                &fixture.runtime,
                persona.id,
                session_id,
                uuid::Uuid::new_v4(),
                0,
                &fixture.admission.archive_sha256,
                Some(screen),
                action,
                &json!({}),
            )
            .await,
            Err(session_cartridges::SessionCartridgeError::Denied),
            "unknown screens and host navigation must not enter gameplay"
        );
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM game_session_cartridge_action_admissions WHERE game_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("admission count should read"),
        2
    );
    assert!(
        sqlx::query(
            "UPDATE game_session_cartridge_action_admissions SET action = 'other' WHERE game_session_id = $1",
        )
        .bind(session_id)
        .execute(&pool)
        .await
        .is_err(),
        "durable action admissions must be immutable"
    );

    publish_fixture_policy(&pool, &fixture, CatalogStatus::Suspended).await;
    let replay = session_cartridges::admit_session_action(
        &pool,
        &fixture.runtime,
        persona.id,
        session_id,
        idempotency_key,
        0,
        &fixture.admission.archive_sha256,
        Some("lobby"),
        "enter",
        &json!({}),
    )
    .await
    .expect("exact pre-transition admission should remain recoverable");
    assert_eq!(replay, admitted);
    assert_eq!(
        session_cartridges::admit_session_action(
            &pool,
            &fixture.runtime,
            persona.id,
            session_id,
            idempotency_key,
            0,
            &fixture.admission.archive_sha256,
            Some("lobby"),
            "inspect",
            &json!({}),
        )
        .await,
        Err(session_cartridges::SessionCartridgeError::IdempotencyConflict)
    );
    assert_eq!(
        session_cartridges::admit_session_action(
            &pool,
            &fixture.runtime,
            persona.id,
            session_id,
            uuid::Uuid::new_v4(),
            0,
            &fixture.admission.archive_sha256,
            Some("lobby"),
            "enter",
            &json!({}),
        )
        .await,
        Err(session_cartridges::SessionCartridgeError::Denied)
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn snapshot_writer_wins_before_fresh_cartridge_action_admission(pool: PgPool) {
    let fixture = acquisition_fixture(&pool).await;
    let token = create_session(&pool).await;
    let persona = personas::create_persona(
        &pool,
        &token,
        CreatePersonaInput {
            handle: "catalog_writer_first".to_owned(),
            display_name: "Catalog Writer First".to_owned(),
            bio: String::new(),
            status_message: String::new(),
        },
    )
    .await
    .expect("test persona should create");
    let session_id = seed_cartridge_action_session(&pool, &fixture, persona.id).await;

    let mut writer = pool.begin().await.expect("writer transaction should begin");
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(omarchy_gaming_system_server::cartridge_catalog::SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *writer)
        .await
        .expect("snapshot writer lock should be acquired");
    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET signed_policy = $1,
            policy_version = 2,
            policy_status = 'suspended',
            policy_reason = 'Review paused.',
            policy_snapshot_version = policy_snapshot_version + 1,
            last_seen_snapshot_version = last_seen_snapshot_version + 1,
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $2
        "#,
    )
    .bind(sqlx::types::Json(&fixture.suspended_policy))
    .bind(&fixture.admission.archive_sha256)
    .execute(&mut *writer)
    .await
    .expect("writer should stage the suspended policy");

    let action_pool = pool.clone();
    let runtime = fixture.runtime.clone();
    let digest = fixture.admission.archive_sha256.clone();
    let actor_id = persona.id;
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let action = tokio::spawn(async move {
        let _ = started_tx.send(());
        session_cartridges::admit_session_action(
            &action_pool,
            &runtime,
            actor_id,
            session_id,
            uuid::Uuid::new_v4(),
            0,
            &digest,
            Some("lobby"),
            "enter",
            &json!({}),
        )
        .await
    });
    started_rx.await.expect("action task should start");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        !action.is_finished(),
        "fresh admission must wait behind the exclusive snapshot writer"
    );
    writer.commit().await.expect("writer should commit");
    assert_eq!(
        action.await.expect("action task should join"),
        Err(session_cartridges::SessionCartridgeError::Denied)
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM game_session_cartridge_action_admissions WHERE game_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("admission count should read"),
        0
    );
}

pub(crate) struct AcquisitionFixture {
    _temp: tempfile::TempDir,
    store_root: std::path::PathBuf,
    pub(crate) runtime: CartridgeDistributionRuntime,
    pub(crate) marketplace_public: omarchygs_game_cartridge::CatalogPublicKey,
    pub(crate) admission: AcquisitionServerAdmission,
    pub(crate) suspended_policy: SignedCatalogPolicy,
}

pub(crate) async fn acquisition_fixture(pool: &PgPool) -> AcquisitionFixture {
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
    let suspended_policy = sign_catalog_policy(
        &release,
        &marketplace_private,
        2,
        CatalogStatus::Suspended,
        "Review paused.",
    )
    .expect("suspended policy should sign");
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
        SnapshotPublication {
            origin: "https://market.example.test",
            key: &marketplace_public,
            payload: &payload,
            digest: &digest,
            signed_snapshot: &signed_snapshot_bytes,
            releases: &[ReviewedReleaseInput {
                entry: entry.clone(),
                policy,
                display_name: release.cartridge().manifest().display_name.clone(),
                compatible: true,
                imported: true,
            }],
            marketplace_trust: None,
        },
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
        suspended_policy,
    }
}

async fn seed_cartridge_action_session(
    pool: &PgPool,
    fixture: &AcquisitionFixture,
    persona_id: uuid::Uuid,
) -> uuid::Uuid {
    let mut transaction = pool
        .begin()
        .await
        .expect("session transaction should begin");
    let session_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO game_sessions (
            game_key, game_version, revision, status, state, authority
        )
        VALUES ('door-legends', 1, 0, 'active', '{}'::jsonb, 'platform_compiled')
        RETURNING id
        "#,
    )
    .fetch_one(&mut *transaction)
    .await
    .expect("action session should insert");
    sqlx::query(
        r#"
        INSERT INTO game_session_participants (game_session_id, persona_id, seat)
        VALUES ($1, $2, 0)
        "#,
    )
    .bind(session_id)
    .bind(persona_id)
    .execute(&mut *transaction)
    .await
    .expect("action participant should insert");
    assert!(
        session_cartridges::pin_new_session(
            &mut transaction,
            &fixture.runtime,
            session_id,
            "door-legends",
            1,
            Some(&fixture.admission.archive_sha256),
        )
        .await
        .expect("exact cartridge should pin")
    );
    transaction
        .commit()
        .await
        .expect("action session should commit");
    session_id
}

pub(crate) async fn publish_fixture_policy(
    pool: &PgPool,
    fixture: &AcquisitionFixture,
    status: CatalogStatus,
) {
    assert_eq!(status, CatalogStatus::Suspended);
    let mut transaction = pool.begin().await.expect("policy transaction should begin");
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(omarchy_gaming_system_server::cartridge_catalog::SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .expect("snapshot writer lock should be acquired");
    sqlx::query(
        r#"
        UPDATE marketplace_releases
        SET signed_policy = $1,
            policy_version = 2,
            policy_status = 'suspended',
            policy_reason = 'Review paused.',
            policy_snapshot_version = policy_snapshot_version + 1,
            last_seen_snapshot_version = last_seen_snapshot_version + 1,
            updated_at = clock_timestamp()
        WHERE archive_sha256 = $2
        "#,
    )
    .bind(sqlx::types::Json(&fixture.suspended_policy))
    .bind(&fixture.admission.archive_sha256)
    .execute(&mut *transaction)
    .await
    .expect("valid suspended policy should publish");
    transaction
        .commit()
        .await
        .expect("policy transition should commit");
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
