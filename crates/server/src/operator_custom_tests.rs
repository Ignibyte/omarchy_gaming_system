use std::{fs, os::unix::fs::PermissionsExt as _, path::Path};

use omarchygs_game_cartridge::{
    CatalogStatus, OPERATOR_CUSTOM_WARNING, OperatorCustomAcquisition, create_release, export_sdk,
    generate_catalog_keypair, generate_keypair, operator_custom_key_sha256, rich_2d_host_profile,
    supported_sdk_identity, verify_operator_custom_acquisition_bytes,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    cartridge_catalog::{
        CatalogCommand, CatalogSelection, apply_catalog_command_with_sources, list_player_catalog,
    },
    cartridge_distribution::{self, CartridgeDistributionRuntime, DistributionError},
    operator_custom::{
        CustomImportCommand, CustomPolicyCommand, OperatorCustomAdminConfig, apply_custom_policy,
        import_custom_release,
    },
};

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn custom_release_is_distinct_idempotent_acquirable_and_denied_by_policy(pool: PgPool) {
    let fixture = CustomFixture::create();
    let import = fixture.import_command;
    let first = import_custom_release(&pool, &fixture.admin, &import)
        .await
        .expect("custom release should import");
    assert!(first.imported);
    assert!(!first.replayed);
    let replay = import_custom_release(&pool, &fixture.admin, &import)
        .await
        .expect("exact custom import should replay");
    assert_eq!(replay.release_id, first.release_id);
    assert!(replay.replayed);

    let store = fixture.admin.open_store().expect("store should reopen");
    let catalog = CatalogCommand {
        idempotency_key: Uuid::new_v4(),
        game_key: first.game_key.clone(),
        expected: CatalogSelection::Inactive,
        desired: CatalogSelection::CustomRelease {
            archive_sha256: first.archive_sha256.clone(),
        },
        actor: "test-operator".to_owned(),
        reason: "Enable a local custom game.".to_owned(),
    };
    apply_catalog_command_with_sources(
        &pool,
        &store,
        None,
        Some(&fixture.admin.public_key),
        &rich_2d_host_profile(),
        &catalog,
    )
    .await
    .expect("custom release should activate");

    let listed = list_player_catalog(&pool)
        .await
        .expect("player catalog should list");
    assert_eq!(listed.len(), 1);
    let custom = listed[0]
        .operator_custom
        .as_ref()
        .expect("custom provenance should be explicit");
    assert!(listed[0].marketplace.is_none());
    assert_eq!(custom.warning, OPERATOR_CUSTOM_WARNING);
    assert_eq!(listed[0].warning.as_deref(), Some(OPERATOR_CUSTOM_WARNING));
    let value = serde_json::to_value(&listed[0]).expect("catalog release should serialize");
    assert!(value.get("marketplace").is_none());
    assert!(value.to_string().find("reviewed_by").is_none());

    let public = fixture.admin.public_config();
    let runtime = CartridgeDistributionRuntime::from_configs(&pool, None, Some(&public))
        .await
        .expect("runtime should validate")
        .expect("custom runtime should enable");
    let bytes = cartridge_distribution::acquire_exact(
        &pool,
        &runtime,
        &first.game_key,
        &first.archive_sha256,
    )
    .await
    .expect("custom release should acquire");
    let document: OperatorCustomAcquisition =
        serde_json::from_slice(&bytes).expect("custom acquisition should parse");
    let admission = document.server_admission;
    assert_eq!(admission.game_key, first.game_key);
    assert_eq!(admission.archive_sha256, first.archive_sha256);
    assert_eq!(admission.admission_revision, 1);
    let verified = verify_operator_custom_acquisition_bytes(
        &bytes,
        &admission,
        &fixture.admin.public_key,
        &supported_sdk_identity().expect("SDK identity should load"),
        &rich_2d_host_profile(),
    )
    .expect("custom acquisition should verify");
    assert_eq!(verified.attestation().warning, OPERATOR_CUSTOM_WARNING);

    let account_id: Uuid = sqlx::query_scalar(
        "INSERT INTO accounts (username, password_hash) VALUES ('custom_player', 'test') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("account should seed");
    let persona_id: Uuid = sqlx::query_scalar(
        "INSERT INTO personas (account_id, handle, display_name) VALUES ($1, 'custom_player', 'Custom Player') RETURNING id",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("persona should seed");
    let mut transaction = pool
        .begin()
        .await
        .expect("session transaction should start");
    let game_session_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO game_sessions (game_key, game_version, state, authority)
        VALUES ($1, 1, '{}'::jsonb, 'platform_compiled')
        RETURNING id
        "#,
    )
    .bind(&first.game_key)
    .fetch_one(&mut *transaction)
    .await
    .expect("game session should seed");
    sqlx::query(
        "INSERT INTO game_session_participants (game_session_id, persona_id, seat) VALUES ($1, $2, 0)",
    )
    .bind(game_session_id)
    .bind(persona_id)
    .execute(&mut *transaction)
    .await
    .expect("participant should seed");
    assert!(
        crate::session_cartridges::pin_new_session(
            &mut transaction,
            &runtime,
            game_session_id,
            &first.game_key,
            1,
            Some(&first.archive_sha256),
        )
        .await
        .expect("custom presentation should pin")
    );
    transaction
        .commit()
        .await
        .expect("session pin should commit");
    apply_catalog_command_with_sources(
        &pool,
        &store,
        None,
        Some(&fixture.admin.public_key),
        &rich_2d_host_profile(),
        &CatalogCommand {
            idempotency_key: Uuid::new_v4(),
            game_key: first.game_key.clone(),
            expected: CatalogSelection::CustomRelease {
                archive_sha256: first.archive_sha256.clone(),
            },
            desired: CatalogSelection::Inactive,
            actor: "test-operator".to_owned(),
            reason: "Stop new launches while keeping historical evidence.".to_owned(),
        },
    )
    .await
    .expect("custom selection should deactivate");
    let historical =
        cartridge_distribution::acquire_session_exact(&pool, &runtime, persona_id, game_session_id)
            .await
            .expect("historical custom pin should remain acquirable");
    let historical_document: OperatorCustomAcquisition =
        serde_json::from_slice(&historical).expect("historical acquisition should parse");
    assert_eq!(
        historical_document.server_admission.archive_sha256,
        first.archive_sha256
    );
    apply_catalog_command_with_sources(
        &pool,
        &store,
        None,
        Some(&fixture.admin.public_key),
        &rich_2d_host_profile(),
        &CatalogCommand {
            idempotency_key: Uuid::new_v4(),
            game_key: first.game_key.clone(),
            expected: CatalogSelection::Inactive,
            desired: CatalogSelection::CustomRelease {
                archive_sha256: first.archive_sha256.clone(),
            },
            actor: "test-operator".to_owned(),
            reason: "Re-enable before testing lifecycle denial.".to_owned(),
        },
    )
    .await
    .expect("custom selection should reactivate");

    let action_id = Uuid::new_v4();
    let admitted = crate::session_cartridges::admit_session_action(
        &pool,
        &runtime,
        persona_id,
        game_session_id,
        action_id,
        0,
        &first.archive_sha256,
        Some("lobby"),
        "enter",
        &serde_json::json!({}),
    )
    .await
    .expect("active custom release action should admit");
    assert_eq!(admitted.command, serde_json::json!({"action": "enter"}));

    let policy_command = CustomPolicyCommand {
        idempotency_key: Uuid::new_v4(),
        game_key: first.game_key.clone(),
        archive_sha256: first.archive_sha256.clone(),
        release_directory: fixture.release_root.clone(),
        publisher_public_key_file: fixture.publisher_public_path.clone(),
        policy_version: 2,
        lifecycle_status: CatalogStatus::Suspended,
        actor: "test-operator".to_owned(),
        reason: "Pause this custom release.".to_owned(),
    };
    let mut action_blocker = pool
        .begin()
        .await
        .expect("action lifecycle transaction should start");
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, $2))")
        .bind(&first.game_key)
        .bind(crate::cartridge_catalog::SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *action_blocker)
        .await
        .expect("same-game policy lock should hold");
    let policy_future = apply_custom_policy(&pool, &fixture.admin, &policy_command);
    tokio::pin!(policy_future);
    tokio::select! {
        result = &mut policy_future => {
            panic!("custom policy must wait for admitted actions: {result:?}");
        }
        () = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
    }
    let fresh_payload = serde_json::json!({});
    let fresh_action_future = crate::session_cartridges::admit_session_action(
        &pool,
        &runtime,
        persona_id,
        game_session_id,
        Uuid::new_v4(),
        0,
        &first.archive_sha256,
        Some("lobby"),
        "enter",
        &fresh_payload,
    );
    tokio::pin!(fresh_action_future);
    tokio::select! {
        result = &mut policy_future => {
            panic!("custom policy must still wait for its same-game lock: {result:?}");
        }
        result = &mut fresh_action_future => {
            panic!("fresh action must wait behind the queued custom policy: {result:?}");
        }
        () = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
    }
    action_blocker
        .commit()
        .await
        .expect("same-game policy blocker should commit");
    let denied = policy_future
        .await
        .expect("newer denial should persist after admitted actions drain");
    assert!(denied.imported, "retained immutable bytes remain imported");
    assert_eq!(
        fresh_action_future.await,
        Err(crate::session_cartridges::SessionCartridgeError::Denied)
    );
    let replay = crate::session_cartridges::admit_session_action(
        &pool,
        &runtime,
        persona_id,
        game_session_id,
        action_id,
        0,
        &first.archive_sha256,
        Some("lobby"),
        "enter",
        &serde_json::json!({}),
    )
    .await
    .expect("exact pre-transition action should remain replayable");
    assert_eq!(replay, admitted);
    assert_eq!(
        crate::session_cartridges::admit_session_action(
            &pool,
            &runtime,
            persona_id,
            game_session_id,
            action_id,
            0,
            &first.archive_sha256,
            Some("lobby"),
            "inspect",
            &serde_json::json!({}),
        )
        .await,
        Err(crate::session_cartridges::SessionCartridgeError::IdempotencyConflict)
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM game_session_cartridge_action_admissions"
        )
        .fetch_one(&pool)
        .await
        .expect("action admission count should read"),
        1
    );
    assert!(
        list_player_catalog(&pool)
            .await
            .expect("catalog should list")
            .is_empty()
    );
    assert_eq!(
        cartridge_distribution::acquire_exact(
            &pool,
            &runtime,
            &first.game_key,
            &first.archive_sha256,
        )
        .await,
        Err(DistributionError::Denied)
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM operator_custom_audit_events")
            .fetch_one(&pool)
            .await
            .expect("audit count should read"),
        2
    );
}

struct CustomFixture {
    _temp: tempfile::TempDir,
    release_root: std::path::PathBuf,
    publisher_public_path: std::path::PathBuf,
    admin: OperatorCustomAdminConfig,
    import_command: CustomImportCommand,
}

impl CustomFixture {
    fn create() -> Self {
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
        let publisher_public_path = temp.path().join("publisher.public.json");
        fs::write(
            &publisher_public_path,
            serde_json::to_vec(&publisher_public).expect("publisher key should serialize"),
        )
        .expect("publisher key should write");
        let (private_key, public_key) =
            generate_catalog_keypair("server-custom-v1", "local-community")
                .expect("operator key should generate");
        let key_sha256 = operator_custom_key_sha256(&public_key).expect("operator key should hash");
        let admin = OperatorCustomAdminConfig {
            operator_name: "Test Community Operator".to_owned(),
            private_key,
            public_key: public_key.clone(),
            key_sha256: key_sha256.clone(),
            store_root: store_root.clone(),
        };
        let import_command = CustomImportCommand {
            idempotency_key: Uuid::new_v4(),
            release_directory: release_root.clone(),
            publisher_public_key_file: publisher_public_path.clone(),
            policy_version: 1,
            lifecycle_status: CatalogStatus::Active,
            actor: "test-operator".to_owned(),
            reason: "Install a locally reviewed custom game.".to_owned(),
            acknowledge_marketplace_warning: true,
        };
        Self {
            _temp: temp,
            release_root,
            publisher_public_path,
            admin,
            import_command,
        }
    }
}
