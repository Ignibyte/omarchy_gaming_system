use std::collections::BTreeSet;

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CACHE_CONTROL, header::CONTENT_TYPE},
};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt as _;

use crate::{
    accounts::{self, RegistrationError, RegistrationInput, RegistrationOutcome},
    app,
    mfa::MfaCipher,
    registration_invites,
};

const PASSWORD: &str = "TEST-ONLY-invited-registration-passphrase";

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn invited_registration_is_atomic_digest_only_and_exactly_replayable(pool: PgPool) {
    let invite_code = accounts::create_test_invite(&pool).await;
    let payload = json!({
        "invite_code": invite_code,
        "username": "  Invited_Player  ",
        "password": PASSWORD
    });
    let created = post_registration(&pool, payload.clone()).await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_eq!(created.cache_control.as_deref(), Some("no-store"));
    exact_keys(&created.document, &["id", "username"]);
    assert_eq!(created.document["username"], "invited_player");

    let (account_count, used_count, raw_code_count, digest_length): (i64, i64, i64, i32) =
        sqlx::query_as(
            r#"
            SELECT
                (SELECT count(*) FROM accounts),
                (SELECT count(*) FROM registration_invites
                    WHERE used_by_account_id IS NOT NULL AND used_at IS NOT NULL),
                (SELECT count(*) FROM registration_invites
                    WHERE encode(code_hash, 'escape') = $1),
                (SELECT octet_length(code_hash) FROM registration_invites)
            "#,
        )
        .bind(&invite_code)
        .fetch_one(&pool)
        .await
        .expect("registration persistence should be readable");
    assert_eq!(
        (account_count, used_count, raw_code_count, digest_length),
        (1, 1, 0, 32)
    );

    let replay = post_registration(
        &pool,
        json!({
            "invite_code": invite_code,
            "username": "INVITED_PLAYER",
            "password": PASSWORD
        }),
    )
    .await;
    assert_eq!(replay.status, StatusCode::OK);
    assert_eq!(replay.cache_control.as_deref(), Some("no-store"));
    assert_eq!(replay.document, created.document);

    for changed in [
        json!({
            "invite_code": invite_code,
            "username": "different_player",
            "password": PASSWORD
        }),
        json!({
            "invite_code": invite_code,
            "username": "invited_player",
            "password": "TEST-ONLY-a-different-passphrase"
        }),
    ] {
        assert_invalid_invitation(post_registration(&pool, changed).await);
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .expect("account count should read"),
        1
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn unavailable_invitations_are_uniform_and_conflicts_do_not_consume(pool: PgPool) {
    let (absent_code, _) = registration_invites::generate().expect("absent code should generate");
    assert_invalid_invitation(
        post_registration(
            &pool,
            json!({
                "invite_code": absent_code,
                "username": "absent_invite",
                "password": PASSWORD
            }),
        )
        .await,
    );
    assert_invalid_invitation(
        post_registration(
            &pool,
            json!({
                "invite_code": "ogsi_malformed",
                "username": "malformed_invite",
                "password": PASSWORD
            }),
        )
        .await,
    );

    let expired_code = accounts::create_test_invite(&pool).await;
    sqlx::query(
        r#"
        UPDATE registration_invites
        SET created_at = created_at - interval '25 hours',
            expires_at = expires_at - interval '25 hours'
        WHERE code_hash = $1
        "#,
    )
    .bind(
        registration_invites::digest(&expired_code)
            .expect("code should digest")
            .as_slice(),
    )
    .execute(&pool)
    .await
    .expect("invite should expire");
    assert_invalid_invitation(
        post_registration(
            &pool,
            json!({
                "invite_code": expired_code,
                "username": "expired_invite",
                "password": PASSWORD
            }),
        )
        .await,
    );

    let revoked_code = accounts::create_test_invite(&pool).await;
    sqlx::query(
        r#"
        UPDATE registration_invites
        SET revoked_at = clock_timestamp(),
            revoked_by = 'test-suite',
            revoked_reason = 'Revoke unavailable fixture',
            revoked_operation_id = gen_random_uuid()
        WHERE code_hash = $1
        "#,
    )
    .bind(
        registration_invites::digest(&revoked_code)
            .expect("code should digest")
            .as_slice(),
    )
    .execute(&pool)
    .await
    .expect("invite should revoke");
    assert_invalid_invitation(
        post_registration(
            &pool,
            json!({
                "invite_code": revoked_code,
                "username": "revoked_invite",
                "password": PASSWORD
            }),
        )
        .await,
    );

    let first_code = accounts::create_test_invite(&pool).await;
    assert_eq!(
        post_registration(
            &pool,
            json!({
                "invite_code": first_code,
                "username": "conflict_target",
                "password": PASSWORD
            }),
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let reusable_code = accounts::create_test_invite(&pool).await;
    let conflict = post_registration(
        &pool,
        json!({
            "invite_code": reusable_code,
            "username": "CONFLICT_TARGET",
            "password": "TEST-ONLY-another-registration-passphrase"
        }),
    )
    .await;
    assert_eq!(conflict.status, StatusCode::CONFLICT);
    assert_eq!(conflict.document["error"]["code"], "username_taken");
    assert_eq!(conflict.cache_control.as_deref(), Some("no-store"));
    assert_eq!(
        post_registration(
            &pool,
            json!({
                "invite_code": reusable_code,
                "username": "conflict_recovered",
                "password": PASSWORD
            }),
        )
        .await
        .status,
        StatusCode::CREATED,
        "username conflict must leave the invitation usable"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn simultaneous_consumers_create_exactly_one_account(pool: PgPool) {
    let invite_code = accounts::create_test_invite(&pool).await;
    let first = RegistrationInput {
        invite_code: invite_code.clone(),
        username: "concurrent_first".to_owned(),
        password: PASSWORD.to_owned(),
    };
    let second = RegistrationInput {
        invite_code,
        username: "concurrent_second".to_owned(),
        password: PASSWORD.to_owned(),
    };
    let (first_result, second_result) = tokio::join!(
        accounts::register_account(&pool, first),
        accounts::register_account(&pool, second)
    );
    assert_eq!(
        usize::from(matches!(first_result, Ok(RegistrationOutcome::Created(_))))
            + usize::from(matches!(second_result, Ok(RegistrationOutcome::Created(_)))),
        1
    );
    assert!(matches!(
        (&first_result, &second_result),
        (
            Ok(RegistrationOutcome::Created(_)),
            Err(RegistrationError::InvalidInvitation)
        ) | (
            Err(RegistrationError::InvalidInvitation),
            Ok(RegistrationOutcome::Created(_))
        )
    ));
    let (accounts, used): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM accounts), (SELECT count(*) FROM registration_invites WHERE used_at IS NOT NULL)",
    )
    .fetch_one(&pool)
    .await
    .expect("concurrent outcome should read");
    assert_eq!((accounts, used), (1, 1));
}

struct TestResponse {
    status: StatusCode,
    cache_control: Option<String>,
    document: Value,
}

async fn post_registration(pool: &PgPool, payload: Value) -> TestResponse {
    let response = app::router(pool.clone(), MfaCipher::test_cipher())
        .oneshot(
            Request::post("/v1/accounts")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    let status = response.status();
    let cache_control = response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should read")
        .to_bytes();
    let document = serde_json::from_slice(&body).expect("body should be JSON");
    TestResponse {
        status,
        cache_control,
        document,
    }
}

fn assert_invalid_invitation(response: TestResponse) {
    assert_eq!(response.status, StatusCode::FORBIDDEN);
    assert_eq!(response.cache_control.as_deref(), Some("no-store"));
    exact_keys(&response.document, &["error"]);
    exact_keys(&response.document["error"], &["code", "message"]);
    assert_eq!(response.document["error"]["code"], "invalid_invitation");
    assert_eq!(
        response.document["error"]["message"],
        "registration invitation is invalid"
    );
}

fn exact_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<&str> = value
        .as_object()
        .expect("value should be an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(actual, expected.iter().copied().collect());
}
