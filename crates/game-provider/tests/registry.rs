use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::SigningKey;
use omarchy_game_provider::{
    ProviderError,
    model::{
        AchievementDefinitionInput, ActivatePilotInput, ActiveSessionPolicy, LifecycleStatus,
        OperationalKeyInput, OperationalKeyKind, OperatorCommand, PilotStatus, ProviderEndpoint,
        ProviderQuotas, ProviderScope, RegisterReleaseInput, SessionAdmission,
    },
    protocol::GrantIssuer,
    registry::{IssueGrantRequest, ProviderRegistry},
};
use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::Barrier;
use uuid::Uuid;

const PROVIDER_ID: &str = "fixture-provider";

fn release_id() -> Uuid {
    Uuid::from_u128(0x18181818181818181818181818181818)
}

async fn database_now(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT EXTRACT(EPOCH FROM clock_timestamp())::BIGINT")
        .fetch_one(pool)
        .await
        .expect("database clock should be readable")
}

async fn register_fixture(pool: &PgPool, quotas: ProviderQuotas) -> ProviderRegistry {
    let now = database_now(pool).await;
    let provider_signing = SigningKey::from_bytes(&[9; 32]);
    let command = OperatorCommand::RegisterRelease {
        actor: "integration-operator".to_owned(),
        reason: "exercise the exact provider registry contract".to_owned(),
        registration: RegisterReleaseInput {
            provider_id: PROVIDER_ID.to_owned(),
            display_name: "Fixture Provider".to_owned(),
            release_id: release_id(),
            game_key: "signal_siege".to_owned(),
            rules_version: 1,
            cartridge_digest: "a".repeat(64),
            endpoint: ProviderEndpoint {
                host: "provider.example.test".to_owned(),
                port: 443,
                base_path: "/omarchygs/provider/v1/".to_owned(),
            },
            active_session_policy: ActiveSessionPolicy::Continue,
            scopes: vec![
                ProviderScope::Launch,
                ProviderScope::Command,
                ProviderScope::Reconcile,
                ProviderScope::Event,
            ],
            message_keys: vec![OperationalKeyInput {
                key_id: "provider-key-1".to_owned(),
                public_material_base64: STANDARD
                    .encode(provider_signing.verifying_key().as_bytes()),
                valid_from: now - 60,
                valid_until: None,
            }],
            tls_roots: vec![OperationalKeyInput {
                key_id: "provider-tls-1".to_owned(),
                public_material_base64: STANDARD.encode([0x30_u8; 128]),
                valid_from: now - 60,
                valid_until: None,
            }],
            quotas,
        },
    };
    let registry = ProviderRegistry::new(pool.clone());
    let receipt = registry
        .apply_operator_command(&command)
        .await
        .expect("fixture release should register");
    assert_eq!(receipt.provider_id, PROVIDER_ID);
    assert_eq!(receipt.release_id, Some(release_id()));
    registry
}

fn quotas() -> ProviderQuotas {
    ProviderQuotas {
        grants_per_minute: 10,
        requests_per_minute: 10,
        callbacks_per_minute: 10,
        max_concurrent_requests: 2,
        request_body_bytes: 16 * 1024,
        response_body_bytes: 64 * 1024,
        connect_timeout_ms: 500,
        total_timeout_ms: 2_000,
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-provider-conformance.sh"]
async fn pilot_activation_pins_public_policy_and_retirement_is_terminal(pool: PgPool) {
    let registry = register_fixture(&pool, quotas()).await;
    let activation = OperatorCommand::ActivatePilot {
        actor: "integration-operator".to_owned(),
        reason: "enable one exact first-party provider pilot".to_owned(),
        pilot: ActivatePilotInput {
            release_id: release_id(),
            display_name: "Signal Siege Remote Fixture".to_owned(),
            min_human_players: 1,
            max_human_players: 1,
            achievements: vec![AchievementDefinitionInput {
                key: "first_win".to_owned(),
                display_name: "First Win".to_owned(),
                description: "Win the exact registered fixture once.".to_owned(),
            }],
        },
    };
    registry
        .apply_operator_command(&activation)
        .await
        .expect("first pilot activation should pass");
    registry
        .apply_operator_command(&activation)
        .await
        .expect("exact activation replay should be stable");
    let policy: (String, i16, i16, String) = sqlx::query_as(
        r#"
        SELECT display_name, min_human_players, max_human_players, status
        FROM provider_game_pilots
        WHERE release_id = $1
        "#,
    )
    .bind(release_id())
    .fetch_one(&pool)
    .await
    .expect("pilot policy should persist");
    assert_eq!(
        policy,
        (
            "Signal Siege Remote Fixture".to_owned(),
            1,
            1,
            "active".to_owned()
        )
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM provider_achievement_definitions WHERE release_id = $1",
        )
        .bind(release_id())
        .fetch_one(&pool)
        .await
        .expect("achievement definitions should count"),
        1
    );
    registry
        .apply_operator_command(&OperatorCommand::SetPilotStatus {
            actor: "integration-operator".to_owned(),
            reason: "exercise read-only provider suspension".to_owned(),
            release_id: release_id(),
            status: PilotStatus::Suspended,
        })
        .await
        .expect("pilot should suspend");
    registry
        .apply_operator_command(&OperatorCommand::SetPilotStatus {
            actor: "integration-operator".to_owned(),
            reason: "restore the exact provider pilot".to_owned(),
            release_id: release_id(),
            status: PilotStatus::Active,
        })
        .await
        .expect("pilot should restore");
    registry
        .apply_operator_command(&OperatorCommand::SetPilotStatus {
            actor: "integration-operator".to_owned(),
            reason: "permanently retire the provider pilot".to_owned(),
            release_id: release_id(),
            status: PilotStatus::Retired,
        })
        .await
        .expect("pilot should retire");
    let revival = registry
        .apply_operator_command(&OperatorCommand::SetPilotStatus {
            actor: "integration-operator".to_owned(),
            reason: "attempt a forbidden pilot revival".to_owned(),
            release_id: release_id(),
            status: PilotStatus::Active,
        })
        .await;
    assert!(matches!(revival, Err(ProviderError::Denied)));
    assert!(
        sqlx::query(
            "DELETE FROM provider_achievement_definitions WHERE release_id = $1 AND achievement_key = 'first_win'",
        )
        .bind(release_id())
        .execute(&pool)
        .await
        .is_err(),
        "platform achievement policy must be immutable"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-provider-conformance.sh"]
async fn registration_pins_identity_and_preserves_append_only_audit(pool: PgPool) {
    let registry = register_fixture(&pool, quotas()).await;
    let policy = registry
        .load_policy(release_id())
        .await
        .expect("policy should load");
    assert_eq!(policy.provider_id, PROVIDER_ID);
    assert_eq!(policy.game_key, "signal_siege");
    assert_eq!(policy.rules_version, 1);
    assert_eq!(policy.config_revision, 1);
    assert_eq!(policy.endpoint.host, "provider.example.test");

    let identity_mutation = sqlx::query(
        "UPDATE provider_releases SET endpoint_host = 'other.example.test' WHERE release_id = $1",
    )
    .bind(release_id())
    .execute(&pool)
    .await;
    assert!(identity_mutation.is_err());
    let key_mutation = sqlx::query(
        "UPDATE provider_release_keys SET public_material = $2 WHERE release_id = $1 AND key_kind = 'message_ed25519'",
    )
    .bind(release_id())
    .bind(vec![7_u8; 32])
    .execute(&pool)
    .await;
    assert!(key_mutation.is_err());
    let audit_mutation = sqlx::query(
        "UPDATE provider_security_audit_events SET reason_code = 'rewritten' WHERE release_id = $1",
    )
    .bind(release_id())
    .execute(&pool)
    .await;
    assert!(audit_mutation.is_err());

    let audit: Value = sqlx::query_scalar(
        "SELECT safe_details FROM provider_security_audit_events WHERE release_id = $1",
    )
    .bind(release_id())
    .fetch_one(&pool)
    .await
    .expect("registration audit should exist");
    let audit_text = audit.to_string();
    assert!(audit_text.contains("exercise the exact provider registry contract"));
    assert!(!audit_text.contains(&STANDARD.encode([9_u8; 32])));
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-provider-conformance.sh"]
async fn key_rotation_scope_and_lifecycle_revocation_fail_closed(pool: PgPool) {
    let registry = register_fixture(&pool, quotas()).await;
    let now = database_now(&pool).await;
    let rotated = SigningKey::from_bytes(&[10; 32]);
    registry
        .apply_operator_command(&OperatorCommand::RotateKey {
            actor: "integration-operator".to_owned(),
            reason: "overlap a replacement provider message key".to_owned(),
            release_id: release_id(),
            key_kind: OperationalKeyKind::MessageEd25519,
            key: OperationalKeyInput {
                key_id: "provider-key-2".to_owned(),
                public_material_base64: STANDARD.encode(rotated.verifying_key().as_bytes()),
                valid_from: now - 1,
                valid_until: None,
            },
        })
        .await
        .expect("overlapping key should rotate");
    let material = registry
        .admit(
            release_id(),
            ProviderScope::Command,
            SessionAdmission::Existing,
        )
        .await
        .expect("both active keys should admit");
    assert_eq!(material.message_keys.len(), 2);

    registry
        .apply_operator_command(&OperatorCommand::SetKeyStatus {
            actor: "integration-operator".to_owned(),
            reason: "retire the replaced provider message key".to_owned(),
            release_id: release_id(),
            key_kind: OperationalKeyKind::MessageEd25519,
            key_id: "provider-key-1".to_owned(),
            status: LifecycleStatus::Revoked,
        })
        .await
        .expect("old key should revoke");
    let material = registry
        .admit(
            release_id(),
            ProviderScope::Command,
            SessionAdmission::Existing,
        )
        .await
        .expect("replacement key should keep release active");
    assert_eq!(
        material
            .message_keys
            .iter()
            .map(|key| key.key_id.as_str())
            .collect::<Vec<_>>(),
        vec!["provider-key-2"]
    );

    registry
        .apply_operator_command(&OperatorCommand::SetReleaseStatus {
            actor: "integration-operator".to_owned(),
            reason: "exercise suspended existing-session policy".to_owned(),
            release_id: release_id(),
            status: LifecycleStatus::Suspended,
        })
        .await
        .expect("release should suspend");
    assert!(
        registry
            .admit(release_id(), ProviderScope::Launch, SessionAdmission::New,)
            .await
            .is_err()
    );
    assert!(
        registry
            .admit(
                release_id(),
                ProviderScope::Command,
                SessionAdmission::Existing,
            )
            .await
            .is_ok()
    );
    registry
        .apply_operator_command(&OperatorCommand::SetScopeStatus {
            actor: "integration-operator".to_owned(),
            reason: "disable command capability immediately".to_owned(),
            release_id: release_id(),
            scope: ProviderScope::Command,
            status: LifecycleStatus::Revoked,
        })
        .await
        .expect("command scope should revoke");
    assert!(
        registry
            .admit(
                release_id(),
                ProviderScope::Command,
                SessionAdmission::Existing,
            )
            .await
            .is_err()
    );
    registry
        .apply_operator_command(&OperatorCommand::SetReleaseStatus {
            actor: "integration-operator".to_owned(),
            reason: "terminate this exact release".to_owned(),
            release_id: release_id(),
            status: LifecycleStatus::Revoked,
        })
        .await
        .expect("release should revoke");
    let revival = registry
        .apply_operator_command(&OperatorCommand::SetReleaseStatus {
            actor: "integration-operator".to_owned(),
            reason: "attempt an invalid terminal-state reversal".to_owned(),
            release_id: release_id(),
            status: LifecycleStatus::Active,
        })
        .await;
    assert!(matches!(revival, Err(ProviderError::Conflict)));
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-provider-conformance.sh"]
async fn grant_quota_pairwise_privacy_and_concurrency_are_durable(pool: PgPool) {
    let mut limits = quotas();
    limits.grants_per_minute = 1;
    limits.max_concurrent_requests = 1;
    let registry = register_fixture(&pool, limits).await;
    let issuer = GrantIssuer::new("platform-key-1", [11; 32], vec![12; 32])
        .expect("grant issuer should construct");
    let persona_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    let platform_session_id = Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb);
    let grant = registry
        .issue_grant(
            &issuer,
            &IssueGrantRequest {
                release_id: release_id(),
                persona_id,
                platform_session_id,
                scope: ProviderScope::Command,
                session: SessionAdmission::Existing,
            },
        )
        .await
        .expect("first grant should issue");
    assert_eq!(grant.claims.subject.len(), 43);
    let stored: String = sqlx::query_scalar(
        r#"
        SELECT row_to_json(g)::text
        FROM provider_grants g
        WHERE token_id = $1
        "#,
    )
    .bind(grant.claims.token_id)
    .fetch_one(&pool)
    .await
    .expect("grant should persist");
    assert!(!stored.contains(&persona_id.to_string()));
    assert!(!stored.contains("account_id"));
    assert!(!stored.contains("device_token"));
    let second = registry
        .issue_grant(
            &issuer,
            &IssueGrantRequest {
                release_id: release_id(),
                persona_id,
                platform_session_id,
                scope: ProviderScope::Command,
                session: SessionAdmission::Existing,
            },
        )
        .await;
    assert!(matches!(second, Err(ProviderError::QuotaExceeded)));

    let first_lease = registry
        .begin_request(
            release_id(),
            ProviderScope::Command,
            SessionAdmission::Existing,
            Uuid::from_u128(1),
        )
        .await
        .expect("first request lease should admit")
        .1;
    let second_lease = registry
        .begin_request(
            release_id(),
            ProviderScope::Command,
            SessionAdmission::Existing,
            Uuid::from_u128(2),
        )
        .await;
    assert!(matches!(second_lease, Err(ProviderError::QuotaExceeded)));
    registry
        .release_request(first_lease)
        .await
        .expect("lease should release");
    let replacement = registry
        .begin_request(
            release_id(),
            ProviderScope::Command,
            SessionAdmission::Existing,
            Uuid::from_u128(3),
        )
        .await
        .expect("released capacity should admit")
        .1;
    registry
        .release_request(replacement)
        .await
        .expect("replacement lease should release");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-provider-conformance.sh"]
async fn concurrent_request_admission_never_exceeds_the_registered_ceiling(pool: PgPool) {
    let mut limits = quotas();
    limits.max_concurrent_requests = 1;
    let registry = register_fixture(&pool, limits).await;
    let barrier = std::sync::Arc::new(Barrier::new(3));
    let mut tasks = Vec::new();
    for correlation_id in [Uuid::from_u128(21), Uuid::from_u128(22)] {
        let registry = registry.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            registry
                .begin_request(
                    release_id(),
                    ProviderScope::Command,
                    SessionAdmission::Existing,
                    correlation_id,
                )
                .await
        }));
    }
    barrier.wait().await;
    let mut admitted = Vec::new();
    let mut denied = 0;
    for task in tasks {
        match task.await.expect("admission task should join") {
            Ok((_, lease)) => admitted.push(lease),
            Err(ProviderError::QuotaExceeded) => denied += 1,
            Err(error) => panic!("unexpected admission error: {error:?}"),
        }
    }
    assert_eq!(admitted.len(), 1);
    assert_eq!(denied, 1);
    let active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM provider_concurrency_leases WHERE release_id = $1",
    )
    .bind(release_id())
    .fetch_one(&pool)
    .await
    .expect("active leases should count");
    assert_eq!(active, 1);
    registry
        .release_request(admitted.pop().expect("one lease should exist"))
        .await
        .expect("winning lease should release");
}
