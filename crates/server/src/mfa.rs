use std::sync::Arc;

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit as AeadKeyInit, Payload},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, Mac};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rand_core::{OsRng, RngCore};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool};
use tracing::{error, info};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{credentials, sessions};

const TOTP_SECRET_BYTES: usize = 20;
const TOTP_STEP_SECONDS: i64 = 30;
const TOTP_DIGITS: u32 = 6;
const ENROLLMENT_LIFETIME_MINUTES: i64 = 10;
const CHALLENGE_PREFIX: &str = "ogm1_";
const CHALLENGE_RANDOM_BYTES: usize = 32;
const MAX_ACTIVE_CHALLENGES: i64 = 10;
const RECOVERY_RANDOM_BYTES: usize = 15;
const RECOVERY_CODE_COUNT: usize = 10;
const MAX_FAILED_ATTEMPTS: i16 = 5;

/// Installation-level key used to protect recoverable TOTP secrets.
#[derive(Clone)]
pub struct MfaCipher {
    key: Arc<Zeroizing<[u8; 32]>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MfaKeyError {
    InvalidEncoding,
    InvalidLength,
}

pub struct BeginEnrollmentInput {
    pub session_token: String,
    pub password: String,
}

pub struct Enrollment {
    pub secret: String,
    pub provisioning_uri: String,
}

pub struct ConfirmEnrollmentInput {
    pub session_token: String,
    pub code: String,
}

pub struct ConfirmedEnrollment {
    pub recovery_codes: Vec<String>,
}

pub struct DisableMfaInput {
    pub session_token: String,
    pub password: String,
    pub code: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MfaStatus {
    pub enabled: bool,
    pub recovery_codes_remaining: i64,
}

pub struct MfaChallenge {
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MfaError {
    Unauthorized,
    InvalidCredentials,
    AlreadyEnabled,
    NotEnabled,
    EnrollmentNotFound,
    InvalidCode,
    RateLimited,
    InvalidChallenge,
    Internal,
}

enum VerifiedFactor {
    Totp(i64),
    Recovery(Uuid),
}

struct LockedAuthenticator {
    encrypted_secret: Vec<u8>,
    nonce: Vec<u8>,
    last_used_step: Option<i64>,
    failed_attempts: i16,
    locked: bool,
    unix_time: i64,
}

impl MfaCipher {
    pub fn from_base64url(encoded: &str) -> Result<Self, MfaKeyError> {
        let decoded = URL_SAFE_NO_PAD
            .decode(encoded.as_bytes())
            .map_err(|_| MfaKeyError::InvalidEncoding)?;
        let key: [u8; 32] = decoded.try_into().map_err(|_| MfaKeyError::InvalidLength)?;

        Ok(Self {
            key: Arc::new(Zeroizing::new(key)),
        })
    }

    fn encrypt_secret(
        &self,
        account_id: Uuid,
        secret: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), MfaError> {
        let cipher = Aes256Gcm::new_from_slice(self.key.as_ref().as_ref())
            .map_err(|_| MfaError::Internal)?;
        let mut nonce_bytes = [0_u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::try_from(nonce_bytes.as_slice()).map_err(|_| MfaError::Internal)?;
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: secret,
                    aad: account_id.as_bytes(),
                },
            )
            .map_err(|_| MfaError::Internal)?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    fn decrypt_secret(
        &self,
        account_id: Uuid,
        ciphertext: &[u8],
        nonce: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, MfaError> {
        let cipher = Aes256Gcm::new_from_slice(self.key.as_ref().as_ref())
            .map_err(|_| MfaError::Internal)?;
        let nonce = Nonce::try_from(nonce).map_err(|_| MfaError::Internal)?;
        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: ciphertext,
                    aad: account_id.as_bytes(),
                },
            )
            .map_err(|decrypt_error| {
                error!(error = %decrypt_error, "TOTP secret decryption failed");
                MfaError::Internal
            })?;
        if plaintext.len() != TOTP_SECRET_BYTES {
            error!(
                length = plaintext.len(),
                "decrypted TOTP secret has invalid length"
            );
            return Err(MfaError::Internal);
        }

        Ok(Zeroizing::new(plaintext))
    }

    #[cfg(test)]
    pub(crate) fn test_cipher() -> Self {
        Self {
            key: Arc::new(Zeroizing::new([0x42; 32])),
        }
    }
}

pub async fn begin_enrollment(
    pool: &PgPool,
    cipher: &MfaCipher,
    input: BeginEnrollmentInput,
) -> Result<Enrollment, MfaError> {
    let authenticated = sessions::authenticate(pool, &input.session_token)
        .await
        .map_err(session_auth_error)?;
    let username = verify_current_password(pool, authenticated.account_id, input.password).await?;

    let mut secret = Zeroizing::new([0_u8; TOTP_SECRET_BYTES]);
    OsRng.fill_bytes(secret.as_mut());
    let (encrypted_secret, nonce) =
        cipher.encrypt_secret(authenticated.account_id, secret.as_ref())?;

    let stored = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account_totp_authenticators (
            account_id,
            encrypted_secret,
            secret_nonce
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (account_id) DO UPDATE
        SET encrypted_secret = EXCLUDED.encrypted_secret,
            secret_nonce = EXCLUDED.secret_nonce,
            enabled_at = NULL,
            last_used_step = NULL,
            failed_attempts = 0,
            locked_until = NULL,
            created_at = now(),
            updated_at = now()
        WHERE account_totp_authenticators.enabled_at IS NULL
        RETURNING account_id
        "#,
    )
    .bind(authenticated.account_id)
    .bind(encrypted_secret)
    .bind(nonce)
    .fetch_optional(pool)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "TOTP enrollment storage failed");
        MfaError::Internal
    })?;

    if stored.is_none() {
        return Err(MfaError::AlreadyEnabled);
    }

    let encoded_secret = BASE32_NOPAD.encode(secret.as_ref());
    let provisioning_uri = provisioning_uri(&username, &encoded_secret);
    info!(account_id = %authenticated.account_id, "TOTP enrollment started");

    Ok(Enrollment {
        secret: encoded_secret,
        provisioning_uri,
    })
}

pub async fn confirm_enrollment(
    pool: &PgPool,
    cipher: &MfaCipher,
    input: ConfirmEnrollmentInput,
) -> Result<ConfirmedEnrollment, MfaError> {
    let authenticated = sessions::authenticate(pool, &input.session_token)
        .await
        .map_err(session_auth_error)?;
    let recovery_codes = generate_recovery_codes();
    let mut transaction = pool.begin().await.map_err(internal_database_error)?;

    let locked = sqlx::query_as::<_, (Vec<u8>, Vec<u8>, Option<i64>, i16, bool, i64)>(
        r#"
        SELECT
            encrypted_secret,
            secret_nonce,
            last_used_step,
            failed_attempts,
            locked_until IS NOT NULL AND locked_until > now(),
            extract(epoch FROM now())::bigint
        FROM account_totp_authenticators
        WHERE account_id = $1
          AND enabled_at IS NULL
          AND created_at > now() - make_interval(mins => $2)
        FOR UPDATE
        "#,
    )
    .bind(authenticated.account_id)
    .bind(ENROLLMENT_LIFETIME_MINUTES as i32)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(internal_database_error)?;

    let Some(row) = locked else {
        return Err(MfaError::EnrollmentNotFound);
    };
    let authenticator = locked_authenticator(row);
    if authenticator.locked {
        return Err(MfaError::RateLimited);
    }

    let secret = cipher.decrypt_secret(
        authenticated.account_id,
        &authenticator.encrypted_secret,
        &authenticator.nonce,
    )?;
    let factor = verify_factor(
        &mut transaction,
        authenticated.account_id,
        &input.code,
        secret.as_ref(),
        authenticator.last_used_step,
        authenticator.unix_time,
    )
    .await?;

    let Some(VerifiedFactor::Totp(step)) = factor else {
        record_failure(
            &mut transaction,
            authenticated.account_id,
            authenticator.failed_attempts,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(internal_database_error)?;
        return Err(MfaError::InvalidCode);
    };

    for code in &recovery_codes {
        sqlx::query(
            "INSERT INTO account_mfa_recovery_codes (account_id, code_hash) VALUES ($1, $2)",
        )
        .bind(authenticated.account_id)
        .bind(recovery_digest(code).ok_or(MfaError::Internal)?)
        .execute(&mut *transaction)
        .await
        .map_err(internal_database_error)?;
    }

    sqlx::query(
        r#"
        UPDATE account_totp_authenticators
        SET enabled_at = now(),
            last_used_step = $2,
            failed_attempts = 0,
            locked_until = NULL,
            updated_at = now()
        WHERE account_id = $1
        "#,
    )
    .bind(authenticated.account_id)
    .bind(step)
    .execute(&mut *transaction)
    .await
    .map_err(internal_database_error)?;

    transaction
        .commit()
        .await
        .map_err(internal_database_error)?;
    info!(account_id = %authenticated.account_id, "TOTP MFA enabled");

    Ok(ConfirmedEnrollment { recovery_codes })
}

pub async fn status(pool: &PgPool, session_token: &str) -> Result<MfaStatus, MfaError> {
    let authenticated = sessions::authenticate(pool, session_token)
        .await
        .map_err(session_auth_error)?;
    let status = sqlx::query_as::<_, (bool, i64)>(
        r#"
        SELECT
            authenticator.enabled_at IS NOT NULL,
            count(recovery.id) FILTER (WHERE recovery.used_at IS NULL)
        FROM account_totp_authenticators AS authenticator
        LEFT JOIN account_mfa_recovery_codes AS recovery
          ON recovery.account_id = authenticator.account_id
        WHERE authenticator.account_id = $1
        GROUP BY authenticator.enabled_at
        "#,
    )
    .bind(authenticated.account_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_database_error)?;

    Ok(match status {
        Some((enabled, recovery_codes_remaining)) if enabled => MfaStatus {
            enabled,
            recovery_codes_remaining,
        },
        _ => MfaStatus {
            enabled: false,
            recovery_codes_remaining: 0,
        },
    })
}

pub async fn disable(
    pool: &PgPool,
    cipher: &MfaCipher,
    input: DisableMfaInput,
) -> Result<(), MfaError> {
    let authenticated = sessions::authenticate(pool, &input.session_token)
        .await
        .map_err(session_auth_error)?;
    verify_current_password(pool, authenticated.account_id, input.password).await?;

    let mut transaction = pool.begin().await.map_err(internal_database_error)?;
    let locked = lock_enabled_authenticator(&mut transaction, authenticated.account_id).await?;
    let Some(authenticator) = locked else {
        return Err(MfaError::NotEnabled);
    };
    if authenticator.locked {
        return Err(MfaError::RateLimited);
    }

    let secret = cipher.decrypt_secret(
        authenticated.account_id,
        &authenticator.encrypted_secret,
        &authenticator.nonce,
    )?;
    let factor = verify_factor(
        &mut transaction,
        authenticated.account_id,
        &input.code,
        secret.as_ref(),
        authenticator.last_used_step,
        authenticator.unix_time,
    )
    .await?;
    if factor.is_none() {
        record_failure(
            &mut transaction,
            authenticated.account_id,
            authenticator.failed_attempts,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(internal_database_error)?;
        return Err(MfaError::InvalidCode);
    }

    sqlx::query("DELETE FROM account_mfa_login_challenges WHERE account_id = $1")
        .bind(authenticated.account_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_database_error)?;
    sqlx::query("DELETE FROM account_totp_authenticators WHERE account_id = $1")
        .bind(authenticated.account_id)
        .execute(&mut *transaction)
        .await
        .map_err(internal_database_error)?;
    transaction
        .commit()
        .await
        .map_err(internal_database_error)?;
    info!(account_id = %authenticated.account_id, "TOTP MFA disabled");

    Ok(())
}

pub(crate) async fn create_challenge_if_enabled(
    connection: &mut PgConnection,
    account_id: Uuid,
    device_name: &str,
) -> Result<Option<MfaChallenge>, MfaError> {
    let enabled = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM account_totp_authenticators WHERE account_id = $1 AND enabled_at IS NOT NULL)",
    )
    .bind(account_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(internal_database_error)?;
    if !enabled {
        return Ok(None);
    }

    sqlx::query(
        r#"
        DELETE FROM account_mfa_login_challenges
        WHERE account_id = $1
          AND (consumed_at IS NOT NULL OR expires_at <= now())
        "#,
    )
    .bind(account_id)
    .execute(&mut *connection)
    .await
    .map_err(internal_database_error)?;
    let active_challenges = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM account_mfa_login_challenges
        WHERE account_id = $1
          AND consumed_at IS NULL
          AND expires_at > now()
        "#,
    )
    .bind(account_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(internal_database_error)?;
    if active_challenges >= MAX_ACTIVE_CHALLENGES {
        return Err(MfaError::RateLimited);
    }

    let token = generate_challenge_token();
    let token_hash = challenge_digest(&token).ok_or(MfaError::Internal)?;
    let expires_at = sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO account_mfa_login_challenges (
            account_id,
            token_hash,
            device_name,
            expires_at
        )
        VALUES ($1, $2, $3, now() + interval '5 minutes')
        RETURNING to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        "#,
    )
    .bind(account_id)
    .bind(token_hash)
    .bind(device_name)
    .fetch_one(&mut *connection)
    .await
    .map_err(internal_database_error)?;

    Ok(Some(MfaChallenge { token, expires_at }))
}

pub async fn complete_login_challenge(
    pool: &PgPool,
    cipher: &MfaCipher,
    challenge_token: &str,
    code: &str,
) -> Result<sessions::CreatedSession, MfaError> {
    let token_hash = challenge_digest(challenge_token).ok_or(MfaError::InvalidChallenge)?;
    let mut transaction = pool.begin().await.map_err(internal_database_error)?;

    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Vec<u8>,
            Vec<u8>,
            Option<i64>,
            i16,
            bool,
            i64,
        ),
    >(
        r#"
        SELECT
            challenge.id,
            challenge.account_id,
            challenge.device_name,
            authenticator.encrypted_secret,
            authenticator.secret_nonce,
            authenticator.last_used_step,
            authenticator.failed_attempts,
            authenticator.locked_until IS NOT NULL AND authenticator.locked_until > now(),
            extract(epoch FROM now())::bigint
        FROM account_mfa_login_challenges AS challenge
        JOIN accounts AS account ON account.id = challenge.account_id
        JOIN account_totp_authenticators AS authenticator
          ON authenticator.account_id = challenge.account_id
        WHERE challenge.token_hash = $1
          AND challenge.consumed_at IS NULL
          AND challenge.expires_at > now()
          AND challenge.failed_attempts < $2
          AND account.status = 'active'
          AND authenticator.enabled_at IS NOT NULL
        FOR UPDATE OF challenge, account, authenticator
        "#,
    )
    .bind(token_hash)
    .bind(MAX_FAILED_ATTEMPTS)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(internal_database_error)?;

    let Some((
        challenge_id,
        account_id,
        device_name,
        encrypted_secret,
        nonce,
        last_used_step,
        failed_attempts,
        locked,
        unix_time,
    )) = row
    else {
        return Err(MfaError::InvalidChallenge);
    };
    if locked {
        return Err(MfaError::RateLimited);
    }

    let secret = cipher.decrypt_secret(account_id, &encrypted_secret, &nonce)?;
    let factor = verify_factor(
        &mut transaction,
        account_id,
        code,
        secret.as_ref(),
        last_used_step,
        unix_time,
    )
    .await?;
    let Some(factor) = factor else {
        sqlx::query(
            r#"
            UPDATE account_mfa_login_challenges
            SET failed_attempts = LEAST(failed_attempts + 1, $2),
                consumed_at = CASE
                    WHEN failed_attempts + 1 >= $2 THEN now()
                    ELSE consumed_at
                END
            WHERE id = $1
            "#,
        )
        .bind(challenge_id)
        .bind(MAX_FAILED_ATTEMPTS)
        .execute(&mut *transaction)
        .await
        .map_err(internal_database_error)?;
        record_failure(&mut transaction, account_id, failed_attempts).await?;
        transaction
            .commit()
            .await
            .map_err(internal_database_error)?;
        return Err(MfaError::InvalidCode);
    };

    consume_factor(&mut transaction, account_id, factor).await?;
    sqlx::query(
        "UPDATE account_mfa_login_challenges SET consumed_at = now() WHERE id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(&mut *transaction)
    .await
    .map_err(internal_database_error)?;
    let created = sessions::issue_session_in_transaction(&mut transaction, account_id, device_name)
        .await
        .map_err(|session_error| {
            error!(?session_error, "device session issuance after MFA failed");
            MfaError::Internal
        })?;
    transaction
        .commit()
        .await
        .map_err(internal_database_error)?;
    info!(account_id = %account_id, "MFA login challenge completed");

    Ok(created)
}

async fn verify_current_password(
    pool: &PgPool,
    account_id: Uuid,
    password: String,
) -> Result<String, MfaError> {
    let account = sqlx::query_as::<_, (String, String)>(
        "SELECT username, password_hash FROM accounts WHERE id = $1 AND status = 'active'",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_database_error)?;
    let stored_hash = account
        .as_ref()
        .map(|(_, password_hash)| password_hash.clone());
    let password_valid = credentials::verify_password(password, stored_hash)
        .await
        .map_err(|_| MfaError::Internal)?;
    match account {
        Some((username, _)) if password_valid => Ok(username),
        _ => Err(MfaError::InvalidCredentials),
    }
}

async fn lock_enabled_authenticator(
    connection: &mut PgConnection,
    account_id: Uuid,
) -> Result<Option<LockedAuthenticator>, MfaError> {
    let row = sqlx::query_as::<_, (Vec<u8>, Vec<u8>, Option<i64>, i16, bool, i64)>(
        r#"
        SELECT
            encrypted_secret,
            secret_nonce,
            last_used_step,
            failed_attempts,
            locked_until IS NOT NULL AND locked_until > now(),
            extract(epoch FROM now())::bigint
        FROM account_totp_authenticators
        WHERE account_id = $1 AND enabled_at IS NOT NULL
        FOR UPDATE
        "#,
    )
    .bind(account_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(internal_database_error)?;

    Ok(row.map(locked_authenticator))
}

fn locked_authenticator(
    row: (Vec<u8>, Vec<u8>, Option<i64>, i16, bool, i64),
) -> LockedAuthenticator {
    LockedAuthenticator {
        encrypted_secret: row.0,
        nonce: row.1,
        last_used_step: row.2,
        failed_attempts: row.3,
        locked: row.4,
        unix_time: row.5,
    }
}

async fn verify_factor(
    connection: &mut PgConnection,
    account_id: Uuid,
    code: &str,
    secret: &[u8],
    last_used_step: Option<i64>,
    unix_time: i64,
) -> Result<Option<VerifiedFactor>, MfaError> {
    if let Some(digest) = recovery_digest(code) {
        let recovery_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM account_mfa_recovery_codes
            WHERE account_id = $1 AND code_hash = $2 AND used_at IS NULL
            FOR UPDATE
            "#,
        )
        .bind(account_id)
        .bind(digest)
        .fetch_optional(&mut *connection)
        .await
        .map_err(internal_database_error)?;
        return Ok(recovery_id.map(VerifiedFactor::Recovery));
    }

    Ok(matching_totp_step(secret, code, unix_time, last_used_step).map(VerifiedFactor::Totp))
}

async fn consume_factor(
    connection: &mut PgConnection,
    account_id: Uuid,
    factor: VerifiedFactor,
) -> Result<(), MfaError> {
    match factor {
        VerifiedFactor::Totp(step) => {
            sqlx::query(
                r#"
                UPDATE account_totp_authenticators
                SET last_used_step = $2,
                    failed_attempts = 0,
                    locked_until = NULL,
                    updated_at = now()
                WHERE account_id = $1
                "#,
            )
            .bind(account_id)
            .bind(step)
            .execute(&mut *connection)
            .await
            .map_err(internal_database_error)?;
        }
        VerifiedFactor::Recovery(recovery_id) => {
            sqlx::query(
                "UPDATE account_mfa_recovery_codes SET used_at = now() WHERE id = $1 AND account_id = $2 AND used_at IS NULL",
            )
            .bind(recovery_id)
            .bind(account_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_database_error)?;
            sqlx::query(
                r#"
                UPDATE account_totp_authenticators
                SET failed_attempts = 0,
                    locked_until = NULL,
                    updated_at = now()
                WHERE account_id = $1
                "#,
            )
            .bind(account_id)
            .execute(&mut *connection)
            .await
            .map_err(internal_database_error)?;
        }
    }

    Ok(())
}

async fn record_failure(
    connection: &mut PgConnection,
    account_id: Uuid,
    prior_failures: i16,
) -> Result<(), MfaError> {
    let expired_lock_reset = if prior_failures >= MAX_FAILED_ATTEMPTS {
        0
    } else {
        prior_failures
    };
    let next_failures = (expired_lock_reset + 1).min(MAX_FAILED_ATTEMPTS);
    sqlx::query(
        r#"
        UPDATE account_totp_authenticators
        SET failed_attempts = $2,
            locked_until = CASE
                WHEN $2 >= $3 THEN now() + interval '5 minutes'
                ELSE NULL
            END,
            updated_at = now()
        WHERE account_id = $1
        "#,
    )
    .bind(account_id)
    .bind(next_failures)
    .bind(MAX_FAILED_ATTEMPTS)
    .execute(&mut *connection)
    .await
    .map_err(internal_database_error)?;
    Ok(())
}

fn provisioning_uri(username: &str, secret: &str) -> String {
    let issuer = "OmarchyGS";
    let label = format!("{issuer}:{username}");
    format!(
        "otpauth://totp/{}?secret={secret}&issuer={}&algorithm=SHA1&digits=6&period=30",
        utf8_percent_encode(&label, NON_ALPHANUMERIC),
        utf8_percent_encode(issuer, NON_ALPHANUMERIC),
    )
}

fn generate_challenge_token() -> String {
    let mut random_bytes = [0_u8; CHALLENGE_RANDOM_BYTES];
    OsRng.fill_bytes(&mut random_bytes);
    format!("{CHALLENGE_PREFIX}{}", URL_SAFE_NO_PAD.encode(random_bytes))
}

fn challenge_digest(token: &str) -> Option<Vec<u8>> {
    let encoded = token.strip_prefix(CHALLENGE_PREFIX)?;
    let random_bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
    if random_bytes.len() != CHALLENGE_RANDOM_BYTES {
        return None;
    }
    Some(Sha256::digest(token.as_bytes()).to_vec())
}

fn generate_recovery_codes() -> Vec<String> {
    (0..RECOVERY_CODE_COUNT)
        .map(|_| {
            let mut random_bytes = [0_u8; RECOVERY_RANDOM_BYTES];
            OsRng.fill_bytes(&mut random_bytes);
            let encoded = BASE32_NOPAD.encode(&random_bytes);
            let groups = encoded
                .as_bytes()
                .chunks(4)
                .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("-");
            format!("OGS-{groups}")
        })
        .collect()
}

fn recovery_digest(code: &str) -> Option<Vec<u8>> {
    let compact = code
        .trim()
        .chars()
        .filter(|character| *character != '-')
        .flat_map(char::to_uppercase)
        .collect::<String>();
    let payload = compact.strip_prefix("OGS")?;
    if payload.len() != 24
        || !payload
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || matches!(byte, b'2'..=b'7'))
    {
        return None;
    }
    let canonical_groups = payload
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).ok())
        .collect::<Option<Vec<_>>>()?
        .join("-");
    let canonical = format!("OGS-{canonical_groups}");
    Some(Sha256::digest(canonical.as_bytes()).to_vec())
}

fn matching_totp_step(
    secret: &[u8],
    raw_code: &str,
    unix_time: i64,
    last_used_step: Option<i64>,
) -> Option<i64> {
    let code = parse_totp_code(raw_code)?;
    let current_step = unix_time.checked_div(TOTP_STEP_SECONDS)?;
    if current_step < 0 {
        return None;
    }
    let candidates = [
        current_step,
        current_step.saturating_sub(1),
        current_step.saturating_add(1),
    ];

    if let Some(last_step) = last_used_step
        && candidates.contains(&last_step)
        && totp_code(secret, last_step as u64) == code
    {
        return None;
    }

    candidates.into_iter().find(|step| {
        *step >= 0
            && last_used_step.is_none_or(|last_step| *step > last_step)
            && totp_code(secret, *step as u64) == code
    })
}

fn parse_totp_code(raw_code: &str) -> Option<u32> {
    let code = raw_code.trim().as_bytes();
    if code.len() != TOTP_DIGITS as usize || !code.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(code).ok()?.parse().ok()
}

fn totp_code(secret: &[u8], step: u64) -> u32 {
    let mut hmac = <Hmac<Sha1> as hmac::KeyInit>::new_from_slice(secret)
        .expect("HMAC accepts TOTP secrets of any length");
    hmac.update(&step.to_be_bytes());
    let digest = hmac.finalize().into_bytes();
    let offset = usize::from(digest[digest.len() - 1] & 0x0f);
    let binary = (u32::from(digest[offset] & 0x7f) << 24)
        | (u32::from(digest[offset + 1]) << 16)
        | (u32::from(digest[offset + 2]) << 8)
        | u32::from(digest[offset + 3]);
    binary % 10_u32.pow(TOTP_DIGITS)
}

#[cfg(test)]
pub(crate) fn test_totp_at(encoded_secret: &str, unix_time: i64) -> String {
    let secret = BASE32_NOPAD
        .decode(encoded_secret.as_bytes())
        .expect("server-issued TOTP secret should decode");
    let step = unix_time
        .checked_div(TOTP_STEP_SECONDS)
        .and_then(|step| u64::try_from(step).ok())
        .expect("test time should be after the Unix epoch");
    format!("{:06}", totp_code(&secret, step))
}

fn session_auth_error(error: sessions::SessionError) -> MfaError {
    match error {
        sessions::SessionError::Unauthorized => MfaError::Unauthorized,
        _ => MfaError::Internal,
    }
}

fn internal_database_error(database_error: sqlx::Error) -> MfaError {
    error!(error = %database_error, "MFA database operation failed");
    MfaError::Internal
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    use super::{
        MfaCipher, MfaKeyError, generate_challenge_token, generate_recovery_codes,
        matching_totp_step, provisioning_uri, recovery_digest, totp_code,
    };

    #[test]
    fn encryption_key_is_exactly_256_bits_of_base64url() {
        let encoded = URL_SAFE_NO_PAD.encode([0x33_u8; 32]);
        assert!(MfaCipher::from_base64url(&encoded).is_ok());
        assert!(matches!(
            MfaCipher::from_base64url("%%%"),
            Err(MfaKeyError::InvalidEncoding)
        ));
        assert!(matches!(
            MfaCipher::from_base64url(&URL_SAFE_NO_PAD.encode([0_u8; 31])),
            Err(MfaKeyError::InvalidLength)
        ));
    }

    #[test]
    fn encrypted_secrets_require_the_same_key_account_and_nonce() {
        let cipher = MfaCipher::test_cipher();
        let account_id = uuid::Uuid::from_u128(1);
        let other_account_id = uuid::Uuid::from_u128(2);
        let secret = [0x77_u8; 20];
        let (ciphertext, nonce) = cipher
            .encrypt_secret(account_id, &secret)
            .expect("secret should encrypt");

        assert_eq!(ciphertext.len(), 36);
        assert_eq!(nonce.len(), 12);
        assert_ne!(ciphertext, secret);
        assert_eq!(
            cipher
                .decrypt_secret(account_id, &ciphertext, &nonce)
                .expect("secret should decrypt")
                .as_slice(),
            secret
        );
        assert!(
            cipher
                .decrypt_secret(other_account_id, &ciphertext, &nonce)
                .is_err()
        );
        let mut tampered = ciphertext;
        tampered[0] ^= 1;
        assert!(
            cipher
                .decrypt_secret(account_id, &tampered, &nonce)
                .is_err()
        );
    }

    #[test]
    fn rfc_6238_sha1_vectors_match() {
        let secret = b"12345678901234567890";
        for (unix_time, expected) in [
            (59, 94_287_082),
            (1_111_111_109, 7_081_804),
            (1_111_111_111, 14_050_471),
            (1_234_567_890, 89_005_924),
            (2_000_000_000, 69_279_037),
            (20_000_000_000, 65_353_130),
        ] {
            assert_eq!(totp_code(secret, unix_time / 30), expected % 1_000_000);
        }
    }

    #[test]
    fn totp_window_is_bounded_and_steps_are_single_use() {
        let secret = b"12345678901234567890";
        let current_step = 1_000_i64;
        let unix_time = current_step * 30;
        let current = format!("{:06}", totp_code(secret, current_step as u64));
        let previous = format!("{:06}", totp_code(secret, (current_step - 1) as u64));
        let next = format!("{:06}", totp_code(secret, (current_step + 1) as u64));
        let too_old = format!("{:06}", totp_code(secret, (current_step - 2) as u64));

        assert_eq!(
            matching_totp_step(secret, &current, unix_time, None),
            Some(current_step)
        );
        assert_eq!(
            matching_totp_step(secret, &previous, unix_time, None),
            Some(current_step - 1)
        );
        assert_eq!(
            matching_totp_step(secret, &next, unix_time, None),
            Some(current_step + 1)
        );
        assert_eq!(matching_totp_step(secret, &too_old, unix_time, None), None);
        assert_eq!(
            matching_totp_step(secret, &current, unix_time, Some(current_step)),
            None
        );
        assert_eq!(matching_totp_step(secret, "12345", unix_time, None), None);
        assert_eq!(matching_totp_step(secret, "12a456", unix_time, None), None);
    }

    #[test]
    fn recovery_and_challenge_tokens_are_random_and_canonical() {
        let first_codes = generate_recovery_codes();
        let second_codes = generate_recovery_codes();
        assert_eq!(first_codes.len(), 10);
        assert_eq!(
            first_codes
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            10
        );
        assert_ne!(first_codes, second_codes);
        for code in first_codes {
            assert_eq!(code.len(), 33);
            assert_eq!(
                recovery_digest(&code),
                recovery_digest(&code.to_ascii_lowercase())
            );
            assert_eq!(recovery_digest(&code).map(|digest| digest.len()), Some(32));
        }

        let first_challenge = generate_challenge_token();
        let second_challenge = generate_challenge_token();
        assert!(first_challenge.starts_with("ogm1_"));
        assert_ne!(first_challenge, second_challenge);
    }

    #[test]
    fn provisioning_uri_uses_the_omarchygs_issuer_and_escaped_account() {
        let uri = provisioning_uri("player_one", "JBSWY3DPEHPK3PXP");
        assert!(uri.starts_with("otpauth://totp/OmarchyGS%3Aplayer%5Fone?"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXP"));
        assert!(uri.contains("issuer=OmarchyGS"));
        assert!(uri.ends_with("algorithm=SHA1&digits=6&period=30"));
    }
}
