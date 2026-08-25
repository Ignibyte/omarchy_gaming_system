use std::collections::HashSet;

use axum::{
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header::CONTENT_TYPE},
};
use data_encoding::BASE32_NOPAD;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tower::ServiceExt;

use crate::{
    app::router,
    mfa::{MfaCipher, test_totp_at},
};

struct TestResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

impl TestResponse {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("response should contain valid JSON")
    }

    fn assert_no_store(&self) {
        assert_eq!(
            self.headers
                .get("cache-control")
                .and_then(|value| value.to_str().ok()),
            Some("no-store")
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn enrollment_encrypts_secret_confirms_recovery_and_scopes_status(pool: PgPool) {
    let password = "TEST-ONLY-mfa-enrollment-passphrase";
    let alice_token = register_and_login(&pool, "Mfa_Enroll_Alice", password).await;
    let bob_token = register_and_login(
        &pool,
        "Mfa_Enroll_Bob",
        "TEST-ONLY-bob-enrollment-passphrase",
    )
    .await;

    let initial_status = get(&pool, "/v1/account/mfa", Some(&alice_token)).await;
    assert_eq!(initial_status.status, StatusCode::OK);
    initial_status.assert_no_store();
    assert_eq!(
        initial_status.json(),
        json!({"enabled": false, "recovery_codes_remaining": 0})
    );

    let wrong_password = authenticated_json(
        &pool,
        Method::POST,
        "/v1/account/mfa",
        &alice_token,
        json!({"password": "TEST-ONLY-wrong-enrollment-passphrase"}),
    )
    .await;
    assert_eq!(wrong_password.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        wrong_password.json()["error"]["code"],
        "invalid_credentials"
    );

    let first_enrollment = begin_enrollment(&pool, &alice_token, password).await;
    assert_eq!(first_enrollment.status, StatusCode::CREATED);
    first_enrollment.assert_no_store();
    let first_document = first_enrollment.json();
    let first_secret = first_document["secret"]
        .as_str()
        .expect("enrollment should return a secret");
    assert_eq!(
        BASE32_NOPAD
            .decode(first_secret.as_bytes())
            .expect("secret should be base32")
            .len(),
        20
    );
    assert!(
        first_document["provisioning_uri"]
            .as_str()
            .expect("enrollment should return a URI")
            .starts_with("otpauth://totp/OmarchyGS%3A")
    );

    let (encrypted_secret, nonce, enabled, stored_plaintext_match) =
        sqlx::query_as::<_, (Vec<u8>, Vec<u8>, bool, bool)>(
            r#"
            SELECT
                encrypted_secret,
                secret_nonce,
                enabled_at IS NOT NULL,
                encrypted_secret = $1
            FROM account_totp_authenticators
            WHERE account_id = (SELECT id FROM accounts WHERE username = 'mfa_enroll_alice')
            "#,
        )
        .bind(
            BASE32_NOPAD
                .decode(first_secret.as_bytes())
                .expect("secret should decode"),
        )
        .fetch_one(&pool)
        .await
        .expect("pending authenticator should be stored");
    assert_eq!(encrypted_secret.len(), 36);
    assert_eq!(nonce.len(), 12);
    assert!(!enabled);
    assert!(!stored_plaintext_match);

    sqlx::query(
        r#"
        UPDATE account_totp_authenticators
        SET created_at = now() - interval '11 minutes'
        WHERE account_id = (SELECT id FROM accounts WHERE username = 'mfa_enroll_alice')
        "#,
    )
    .execute(&pool)
    .await
    .expect("test should expire pending enrollment");
    let expired_confirmation = confirm_enrollment(
        &pool,
        &alice_token,
        &test_totp_at(first_secret, database_time(&pool).await),
    )
    .await;
    assert_eq!(expired_confirmation.status, StatusCode::CONFLICT);
    assert_eq!(
        expired_confirmation.json()["error"]["code"],
        "mfa_enrollment_not_found"
    );

    let enrollment = begin_enrollment(&pool, &alice_token, password).await;
    let secret = enrollment.json()["secret"]
        .as_str()
        .expect("replacement enrollment should return a secret")
        .to_owned();
    let unix_time = database_time(&pool).await;
    let previous_code = test_totp_at(&secret, unix_time - 30);
    let invalid_code = different_code(&previous_code);
    let invalid_confirmation = confirm_enrollment(&pool, &alice_token, &invalid_code).await;
    assert_eq!(invalid_confirmation.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        invalid_confirmation.json()["error"]["code"],
        "invalid_mfa_code"
    );
    assert_eq!(
        get(&pool, "/v1/account/mfa", Some(&alice_token))
            .await
            .json()["enabled"],
        false
    );

    let confirmation = confirm_enrollment(&pool, &alice_token, &previous_code).await;
    assert_eq!(confirmation.status, StatusCode::OK);
    confirmation.assert_no_store();
    let recovery_codes = confirmation.json()["recovery_codes"]
        .as_array()
        .expect("confirmation should return recovery codes")
        .iter()
        .map(|code| {
            code.as_str()
                .expect("recovery code should be text")
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(recovery_codes.len(), 10);
    assert_eq!(recovery_codes.iter().collect::<HashSet<_>>().len(), 10);
    assert!(recovery_codes.iter().all(|code| code.starts_with("OGS-")));

    let (enabled, stored_codes, digest_lengths, plaintext_matches) =
        sqlx::query_as::<_, (bool, i64, i64, i64)>(
            r#"
            SELECT
                authenticator.enabled_at IS NOT NULL,
                count(recovery.id),
                count(recovery.id) FILTER (WHERE octet_length(recovery.code_hash) = 32),
                count(recovery.id) FILTER (WHERE encode(recovery.code_hash, 'escape') = ANY($1))
            FROM account_totp_authenticators AS authenticator
            LEFT JOIN account_mfa_recovery_codes AS recovery
              ON recovery.account_id = authenticator.account_id
            WHERE authenticator.account_id = (
                SELECT id FROM accounts WHERE username = 'mfa_enroll_alice'
            )
            GROUP BY authenticator.enabled_at
            "#,
        )
        .bind(&recovery_codes)
        .fetch_one(&pool)
        .await
        .expect("enabled authenticator should be inspectable");
    assert!(enabled);
    assert_eq!(stored_codes, 10);
    assert_eq!(digest_lengths, 10);
    assert_eq!(plaintext_matches, 0);

    let enabled_status = get(&pool, "/v1/account/mfa", Some(&alice_token)).await;
    assert_eq!(
        enabled_status.json(),
        json!({"enabled": true, "recovery_codes_remaining": 10})
    );
    assert!(!enabled_status.body.contains("secret"));
    assert!(!enabled_status.body.contains("account_id"));

    let bob_status = get(&pool, "/v1/account/mfa", Some(&bob_token)).await;
    assert_eq!(bob_status.json()["enabled"], false);
    assert!(!bob_status.body.contains("mfa_enroll_alice"));

    let repeated_confirmation = confirm_enrollment(&pool, &alice_token, &previous_code).await;
    assert_eq!(repeated_confirmation.status, StatusCode::CONFLICT);
    assert_eq!(
        repeated_confirmation.json()["error"]["code"],
        "mfa_enrollment_not_found"
    );
    assert!(!repeated_confirmation.body.contains("OGS-"));
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn mfa_login_consumes_totp_recovery_and_challenge_under_replay(pool: PgPool) {
    let username = "Mfa_Login_Alice";
    let password = "TEST-ONLY-mfa-login-passphrase";
    let (existing_token, secret, recovery_codes) = enabled_account(&pool, username, password).await;
    let unix_time = database_time(&pool).await;
    let current_code = test_totp_at(&secret, unix_time);

    let challenged = password_login(&pool, username, password, "TOTP device").await;
    assert_eq!(challenged.status, StatusCode::ACCEPTED);
    challenged.assert_no_store();
    let challenged_document = challenged.json();
    assert_eq!(challenged_document["mfa_required"], true);
    let challenge_token = challenged_document["challenge_token"]
        .as_str()
        .expect("challenge token should be returned")
        .to_owned();
    assert!(challenge_token.starts_with("ogm1_"));
    assert_eq!(session_count(&pool, username).await, 1);

    let completed = complete_challenge(&pool, &challenge_token, &current_code).await;
    assert_eq!(completed.status, StatusCode::CREATED);
    completed.assert_no_store();
    assert!(
        completed.json()["token"]
            .as_str()
            .expect("created session should include token")
            .starts_with("ogs1_")
    );

    let concurrent_replay = complete_challenge(&pool, &challenge_token, &current_code).await;
    assert_eq!(concurrent_replay.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        concurrent_replay.json()["error"]["code"],
        "invalid_mfa_challenge"
    );

    let replay_challenge = challenge_token_from(
        password_login(&pool, username, password, "Replayed TOTP device").await,
    );
    let replayed_totp = complete_challenge(&pool, &replay_challenge, &current_code).await;
    assert_eq!(replayed_totp.status, StatusCode::UNAUTHORIZED);
    assert_eq!(replayed_totp.json()["error"]["code"], "invalid_mfa_code");

    let recovery_challenge =
        challenge_token_from(password_login(&pool, username, password, "Recovery device").await);
    let recovered = complete_challenge(&pool, &recovery_challenge, &recovery_codes[0]).await;
    assert_eq!(recovered.status, StatusCode::CREATED);
    let status_after_recovery = get(&pool, "/v1/account/mfa", Some(&existing_token)).await;
    assert_eq!(status_after_recovery.json()["recovery_codes_remaining"], 9);

    let reused_recovery_challenge = challenge_token_from(
        password_login(&pool, username, password, "Reused recovery device").await,
    );
    let reused_recovery =
        complete_challenge(&pool, &reused_recovery_challenge, &recovery_codes[0]).await;
    assert_eq!(reused_recovery.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reused_recovery.json()["error"]["code"], "invalid_mfa_code");

    let expired_challenge = challenge_token_from(
        password_login(&pool, username, password, "Expired challenge device").await,
    );
    sqlx::query(
        r#"
        UPDATE account_mfa_login_challenges
        SET created_at = now() - interval '10 minutes',
            expires_at = now() - interval '1 second'
        WHERE token_hash = $1
        "#,
    )
    .bind(Sha256::digest(expired_challenge.as_bytes()).to_vec())
    .execute(&pool)
    .await
    .expect("test challenge should be expirable");
    let expired = complete_challenge(&pool, &expired_challenge, &recovery_codes[1]).await;
    assert_eq!(expired.status, StatusCode::UNAUTHORIZED);
    assert_eq!(expired.json()["error"]["code"], "invalid_mfa_challenge");

    let inactive_challenge = challenge_token_from(
        password_login(&pool, username, password, "Inactive account device").await,
    );
    sqlx::query("UPDATE accounts SET status = 'suspended' WHERE username = $1")
        .bind(username.to_ascii_lowercase())
        .execute(&pool)
        .await
        .expect("test account should be suspendable");
    let inactive = complete_challenge(&pool, &inactive_challenge, &recovery_codes[1]).await;
    assert_eq!(inactive.status, StatusCode::UNAUTHORIZED);
    assert_eq!(inactive.json()["error"]["code"], "invalid_mfa_challenge");
    sqlx::query("UPDATE accounts SET status = 'active' WHERE username = $1")
        .bind(username.to_ascii_lowercase())
        .execute(&pool)
        .await
        .expect("test account should be reactivated");

    let concurrent_challenge = challenge_token_from(
        password_login(&pool, username, password, "Concurrent replay device").await,
    );
    let next_code = test_totp_at(&secret, unix_time + 30);
    let first_attempt = complete_challenge(&pool, &concurrent_challenge, &next_code);
    let second_attempt = complete_challenge(&pool, &concurrent_challenge, &next_code);
    let (first, second) = tokio::join!(first_attempt, second_attempt);
    let statuses = [first.status, second.status];
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::CREATED)
            .count(),
        1
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == StatusCode::UNAUTHORIZED)
            .count(),
        1
    );
    assert_eq!(session_count(&pool, username).await, 4);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn challenge_issuance_is_bounded_without_invalidating_live_challenges(pool: PgPool) {
    let username = "Mfa_Challenge_Budget_Alice";
    let password = "TEST-ONLY-mfa-challenge-budget-passphrase";
    let (_, _, recovery_codes) = enabled_account(&pool, username, password).await;
    let mut challenges = Vec::new();

    for challenge_number in 0..10 {
        let response = password_login(
            &pool,
            username,
            password,
            &format!("Parallel MFA device {challenge_number}"),
        )
        .await;
        assert_eq!(response.status, StatusCode::ACCEPTED);
        response.assert_no_store();
        challenges.push(challenge_token_from(response));
    }
    assert_eq!(challenges.iter().collect::<HashSet<_>>().len(), 10);

    let bounded = password_login(&pool, username, password, "Excess MFA device").await;
    assert_eq!(bounded.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(bounded.json()["error"]["code"], "mfa_rate_limited");

    let first_completed = complete_challenge(&pool, &challenges[0], &recovery_codes[0]).await;
    assert_eq!(first_completed.status, StatusCode::CREATED);

    let replenished = password_login(&pool, username, password, "Replenished MFA device").await;
    assert_eq!(replenished.status, StatusCode::ACCEPTED);

    let last_original_completed =
        complete_challenge(&pool, &challenges[9], &recovery_codes[1]).await;
    assert_eq!(last_original_completed.status, StatusCode::CREATED);
    assert_eq!(session_count(&pool, username).await, 3);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
async fn attempts_span_challenges_and_disable_requires_both_factors(pool: PgPool) {
    let username = "Mfa_Disable_Alice";
    let password = "TEST-ONLY-mfa-disable-passphrase";
    let (session_token, secret, recovery_codes) = enabled_account(&pool, username, password).await;
    let unix_time = database_time(&pool).await;
    let valid_code = test_totp_at(&secret, unix_time);
    let invalid_code = different_code(&valid_code);

    for attempt in 0..5 {
        let challenge = challenge_token_from(
            password_login(
                &pool,
                username,
                password,
                &format!("Failed factor device {attempt}"),
            )
            .await,
        );
        let rejected = complete_challenge(&pool, &challenge, &invalid_code).await;
        assert_eq!(rejected.status, StatusCode::UNAUTHORIZED);
        assert_eq!(rejected.json()["error"]["code"], "invalid_mfa_code");
    }

    let locked_challenge = challenge_token_from(
        password_login(&pool, username, password, "Locked factor device").await,
    );
    let locked = complete_challenge(&pool, &locked_challenge, &valid_code).await;
    assert_eq!(locked.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(locked.json()["error"]["code"], "mfa_rate_limited");

    sqlx::query(
        r#"
        UPDATE account_totp_authenticators
        SET locked_until = now() - interval '1 second'
        WHERE account_id = (SELECT id FROM accounts WHERE username = $1)
        "#,
    )
    .bind(username.to_ascii_lowercase())
    .execute(&pool)
    .await
    .expect("test lock should be expirable");

    let wrong_password = disable_mfa(
        &pool,
        &session_token,
        "TEST-ONLY-wrong-disable-passphrase",
        &valid_code,
    )
    .await;
    assert_eq!(wrong_password.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        wrong_password.json()["error"]["code"],
        "invalid_credentials"
    );

    let wrong_factor = disable_mfa(&pool, &session_token, password, &invalid_code).await;
    assert_eq!(wrong_factor.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_factor.json()["error"]["code"], "invalid_mfa_code");
    assert_eq!(
        get(&pool, "/v1/account/mfa", Some(&session_token))
            .await
            .json()["enabled"],
        true
    );

    let disabled = disable_mfa(&pool, &session_token, password, &recovery_codes[0]).await;
    assert_eq!(disabled.status, StatusCode::NO_CONTENT);
    assert!(disabled.body.is_empty());

    let disabled_status = get(&pool, "/v1/account/mfa", Some(&session_token)).await;
    assert_eq!(disabled_status.json()["enabled"], false);
    let (authenticators, recovery, challenges) = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"
        SELECT
            (SELECT count(*) FROM account_totp_authenticators),
            (SELECT count(*) FROM account_mfa_recovery_codes),
            (SELECT count(*) FROM account_mfa_login_challenges)
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("MFA tables should be countable");
    assert_eq!((authenticators, recovery, challenges), (0, 0, 0));

    let ordinary_login = password_login(&pool, username, password, "Post-MFA device").await;
    assert_eq!(ordinary_login.status, StatusCode::CREATED);
    assert!(ordinary_login.json().get("challenge_token").is_none());
}

async fn enabled_account(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> (String, String, Vec<String>) {
    let session_token = register_and_login(pool, username, password).await;
    let enrollment = begin_enrollment(pool, &session_token, password).await;
    assert_eq!(enrollment.status, StatusCode::CREATED);
    let secret = enrollment.json()["secret"]
        .as_str()
        .expect("enrollment should return secret")
        .to_owned();
    let previous_code = test_totp_at(&secret, database_time(pool).await - 30);
    let confirmation = confirm_enrollment(pool, &session_token, &previous_code).await;
    assert_eq!(confirmation.status, StatusCode::OK);
    let recovery_codes = confirmation.json()["recovery_codes"]
        .as_array()
        .expect("confirmation should return recovery codes")
        .iter()
        .map(|code| {
            code.as_str()
                .expect("recovery code should be text")
                .to_owned()
        })
        .collect();
    (session_token, secret, recovery_codes)
}

async fn register_and_login(pool: &PgPool, username: &str, password: &str) -> String {
    let registration = request(
        pool,
        Method::POST,
        "/v1/accounts",
        Some(json!({"username": username, "password": password})),
        None,
    )
    .await;
    assert_eq!(registration.status, StatusCode::CREATED);
    let login = password_login(pool, username, password, "Enrollment device").await;
    assert_eq!(login.status, StatusCode::CREATED);
    login.json()["token"]
        .as_str()
        .expect("login should return token")
        .to_owned()
}

async fn password_login(
    pool: &PgPool,
    username: &str,
    password: &str,
    device_name: &str,
) -> TestResponse {
    request(
        pool,
        Method::POST,
        "/v1/sessions",
        Some(json!({
            "username": username,
            "password": password,
            "device_name": device_name
        })),
        None,
    )
    .await
}

async fn begin_enrollment(pool: &PgPool, token: &str, password: &str) -> TestResponse {
    authenticated_json(
        pool,
        Method::POST,
        "/v1/account/mfa",
        token,
        json!({"password": password}),
    )
    .await
}

async fn confirm_enrollment(pool: &PgPool, token: &str, code: &str) -> TestResponse {
    authenticated_json(
        pool,
        Method::POST,
        "/v1/account/mfa/confirm",
        token,
        json!({"code": code}),
    )
    .await
}

async fn complete_challenge(pool: &PgPool, challenge_token: &str, code: &str) -> TestResponse {
    request(
        pool,
        Method::POST,
        "/v1/sessions/mfa",
        Some(json!({"challenge_token": challenge_token, "code": code})),
        None,
    )
    .await
}

async fn disable_mfa(pool: &PgPool, token: &str, password: &str, code: &str) -> TestResponse {
    authenticated_json(
        pool,
        Method::DELETE,
        "/v1/account/mfa",
        token,
        json!({"password": password, "code": code}),
    )
    .await
}

async fn authenticated_json(
    pool: &PgPool,
    method: Method,
    uri: &str,
    token: &str,
    payload: Value,
) -> TestResponse {
    request(pool, method, uri, Some(payload), Some(token)).await
}

async fn get(pool: &PgPool, uri: &str, token: Option<&str>) -> TestResponse {
    request(pool, Method::GET, uri, None, token).await
}

async fn request(
    pool: &PgPool,
    method: Method,
    uri: &str,
    payload: Option<Value>,
    token: Option<&str>,
) -> TestResponse {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = if let Some(payload) = payload {
        builder = builder.header(CONTENT_TYPE, "application/json");
        Body::from(payload.to_string())
    } else {
        Body::empty()
    };
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }

    let response = router(pool.clone(), MfaCipher::test_cipher())
        .oneshot(builder.body(body).expect("request should be valid"))
        .await
        .expect("router should return a response");
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();

    TestResponse {
        status,
        headers,
        body: String::from_utf8(body.to_vec()).expect("response body should be UTF-8"),
    }
}

fn challenge_token_from(response: TestResponse) -> String {
    assert_eq!(response.status, StatusCode::ACCEPTED);
    response.json()["challenge_token"]
        .as_str()
        .expect("MFA challenge should include token")
        .to_owned()
}

async fn database_time(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT extract(epoch FROM now())::bigint")
        .fetch_one(pool)
        .await
        .expect("database time should be readable")
}

async fn session_count(pool: &PgPool, username: &str) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM account_sessions AS session
        JOIN accounts AS account ON account.id = session.account_id
        WHERE account.username = $1
        "#,
    )
    .bind(username.to_ascii_lowercase())
    .fetch_one(pool)
    .await
    .expect("session count should be readable")
}

fn different_code(valid_code: &str) -> String {
    if valid_code == "000000" {
        "000001".to_owned()
    } else {
        "000000".to_owned()
    }
}
