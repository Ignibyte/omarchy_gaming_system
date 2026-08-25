use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{error, info};
use uuid::Uuid;

use crate::{accounts, credentials, mfa};

const TOKEN_PREFIX: &str = "ogs1_";
const LEGACY_TOKEN_PREFIX: &str = "bbs1_";
const TOKEN_RANDOM_BYTES: usize = 32;
const MAX_DEVICE_NAME_CHARACTERS: usize = 64;

pub struct CreateSessionInput {
    pub username: String,
    pub password: String,
    pub device_name: String,
}

pub struct CreatedSession {
    pub token: String,
    pub session: DeviceSession,
}

pub enum SessionCreation {
    Created(CreatedSession),
    MfaRequired(mfa::MfaChallenge),
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeviceSession {
    pub id: Uuid,
    pub device_name: String,
    pub created_at: String,
    pub last_used_at: String,
    pub expires_at: String,
    pub revoked_at: Option<String>,
    pub current: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionError {
    InvalidDeviceName,
    InvalidCredentials,
    RateLimited,
    Unauthorized,
    SessionNotFound,
    Internal,
}

pub(crate) struct AuthenticatedSession {
    pub(crate) account_id: Uuid,
    pub(crate) session_id: Uuid,
}

type SessionRow = (Uuid, String, String, String, String, Option<String>);

pub async fn create_session(
    pool: &PgPool,
    input: CreateSessionInput,
) -> Result<SessionCreation, SessionError> {
    let device_name = validate_device_name(&input.device_name)?;
    let canonical_username = accounts::canonical_username(&input.username).ok();

    let account = match canonical_username {
        Some(username) => sqlx::query_as::<_, (Uuid, String, String)>(
            "SELECT id, password_hash, status FROM accounts WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|database_error| {
            error!(error = %database_error, "account lookup for session creation failed");
            SessionError::Internal
        })?,
        None => None,
    };

    let stored_hash = account
        .as_ref()
        .map(|(_, password_hash, _)| password_hash.clone());
    let password_valid = credentials::verify_password(input.password, stored_hash)
        .await
        .map_err(|_| SessionError::Internal)?;

    let Some((account_id, _, account_status)) = account else {
        return Err(SessionError::InvalidCredentials);
    };
    if !password_valid || account_status != "active" {
        return Err(SessionError::InvalidCredentials);
    }

    let mut transaction = pool.begin().await.map_err(|database_error| {
        error!(error = %database_error, "session creation transaction failed");
        SessionError::Internal
    })?;
    let account_still_active = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM accounts WHERE id = $1 AND status = 'active' FOR UPDATE",
    )
    .bind(account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "active account lock for session creation failed");
        SessionError::Internal
    })?;
    if account_still_active.is_none() {
        return Err(SessionError::InvalidCredentials);
    }

    if let Some(challenge) =
        mfa::create_challenge_if_enabled(&mut transaction, account_id, &device_name)
            .await
            .map_err(|mfa_error| {
                if mfa_error == mfa::MfaError::RateLimited {
                    SessionError::RateLimited
                } else {
                    error!(?mfa_error, "MFA challenge creation failed");
                    SessionError::Internal
                }
            })?
    {
        transaction.commit().await.map_err(|database_error| {
            error!(error = %database_error, "MFA challenge commit failed");
            SessionError::Internal
        })?;
        info!(account_id = %account_id, "device login requires MFA");
        return Ok(SessionCreation::MfaRequired(challenge));
    }

    let created = issue_session_in_transaction(&mut transaction, account_id, device_name).await?;
    transaction.commit().await.map_err(|database_error| {
        error!(error = %database_error, "device session commit failed");
        SessionError::Internal
    })?;
    info!(session_id = %created.session.id, "device session created");

    Ok(SessionCreation::Created(created))
}

pub(crate) async fn issue_session_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    device_name: String,
) -> Result<CreatedSession, SessionError> {
    let token = generate_token();
    let token_hash = token_digest(&token).ok_or(SessionError::Internal)?;
    let inserted = sqlx::query_as::<_, SessionRow>(
        r#"
        INSERT INTO account_sessions (
            account_id,
            token_hash,
            device_name,
            expires_at
        )
        SELECT id, $2, $3, now() + interval '30 days'
        FROM accounts
        WHERE id = $1 AND status = 'active'
        RETURNING
            id,
            device_name,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(last_used_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            NULL::text
        "#,
    )
    .bind(account_id)
    .bind(token_hash)
    .bind(device_name)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "device session insertion failed");
        SessionError::Internal
    })?;

    let Some(session_row) = inserted else {
        return Err(SessionError::InvalidCredentials);
    };
    let session = session_from_row(session_row, true);
    Ok(CreatedSession { token, session })
}

pub async fn list_sessions(pool: &PgPool, token: &str) -> Result<Vec<DeviceSession>, SessionError> {
    let authenticated = authenticate(pool, token).await?;
    let rows = sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT
            id,
            device_name,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(last_used_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            CASE
                WHEN revoked_at IS NULL THEN NULL
                ELSE to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            END
        FROM account_sessions
        WHERE account_id = $1
        ORDER BY created_at DESC, id
        "#,
    )
    .bind(authenticated.account_id)
    .fetch_all(pool)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "device session listing failed");
        SessionError::Internal
    })?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let current = row.0 == authenticated.session_id;
            session_from_row(row, current)
        })
        .collect())
}

pub async fn revoke_session(
    pool: &PgPool,
    token: &str,
    session_id: Uuid,
) -> Result<(), SessionError> {
    let authenticated = authenticate(pool, token).await?;
    let revoked_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE account_sessions
        SET revoked_at = COALESCE(revoked_at, now())
        WHERE id = $1 AND account_id = $2
        RETURNING id
        "#,
    )
    .bind(session_id)
    .bind(authenticated.account_id)
    .fetch_optional(pool)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "device session revocation failed");
        SessionError::Internal
    })?;

    if revoked_id.is_none() {
        return Err(SessionError::SessionNotFound);
    }

    info!(session_id = %session_id, "device session revoked");
    Ok(())
}

pub(crate) async fn authenticate(
    pool: &PgPool,
    token: &str,
) -> Result<AuthenticatedSession, SessionError> {
    let token_hash = token_digest(token).ok_or(SessionError::Unauthorized)?;
    let authenticated = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        UPDATE account_sessions AS session
        SET last_used_at = now()
        FROM accounts AS account
        WHERE session.account_id = account.id
          AND session.token_hash = $1
          AND session.revoked_at IS NULL
          AND session.expires_at > now()
          AND session.last_used_at > now() - interval '7 days'
          AND account.status = 'active'
        RETURNING session.account_id, session.id
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "device session authentication failed");
        SessionError::Internal
    })?;

    authenticated
        .map(|(account_id, session_id)| AuthenticatedSession {
            account_id,
            session_id,
        })
        .ok_or(SessionError::Unauthorized)
}

pub(crate) async fn remains_authorized(
    pool: &PgPool,
    account_id: Uuid,
    session_id: Uuid,
) -> Result<bool, SessionError> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM account_sessions AS session
            JOIN accounts AS account ON account.id = session.account_id
            WHERE session.id = $1
              AND session.account_id = $2
              AND session.revoked_at IS NULL
              AND session.expires_at > now()
              AND session.last_used_at > now() - interval '7 days'
              AND account.status = 'active'
        )
        "#,
    )
    .bind(session_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "device session reauthorization failed");
        SessionError::Internal
    })
}

fn validate_device_name(device_name: &str) -> Result<String, SessionError> {
    let device_name = device_name.trim();
    let character_count = device_name.chars().count();

    if !(1..=MAX_DEVICE_NAME_CHARACTERS).contains(&character_count)
        || device_name.chars().any(char::is_control)
    {
        return Err(SessionError::InvalidDeviceName);
    }

    Ok(device_name.to_owned())
}

fn generate_token() -> String {
    let mut random_bytes = [0_u8; TOKEN_RANDOM_BYTES];
    OsRng.fill_bytes(&mut random_bytes);
    format!("{TOKEN_PREFIX}{}", URL_SAFE_NO_PAD.encode(random_bytes))
}

fn token_digest(token: &str) -> Option<Vec<u8>> {
    let encoded_token = token
        .strip_prefix(TOKEN_PREFIX)
        .or_else(|| token.strip_prefix(LEGACY_TOKEN_PREFIX))?;
    let random_bytes = URL_SAFE_NO_PAD.decode(encoded_token).ok()?;
    if random_bytes.len() != TOKEN_RANDOM_BYTES {
        return None;
    }

    Some(Sha256::digest(token.as_bytes()).to_vec())
}

fn session_from_row(row: SessionRow, current: bool) -> DeviceSession {
    DeviceSession {
        id: row.0,
        device_name: row.1,
        created_at: row.2,
        last_used_at: row.3,
        expires_at: row.4,
        revoked_at: row.5,
        current,
    }
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use sha2::{Digest, Sha256};

    use super::{
        LEGACY_TOKEN_PREFIX, TOKEN_PREFIX, TOKEN_RANDOM_BYTES, generate_token, token_digest,
        validate_device_name,
    };

    #[test]
    fn device_names_are_trimmed_bounded_and_control_free() {
        assert_eq!(
            validate_device_name("  Omarchy laptop  "),
            Ok("Omarchy laptop".to_owned())
        );
        assert!(validate_device_name("").is_err());
        assert!(validate_device_name("bad\ndevice").is_err());
        assert!(validate_device_name(&"x".repeat(65)).is_err());
    }

    #[test]
    fn tokens_have_256_random_bits_and_stable_digests() {
        let first = generate_token();
        let second = generate_token();

        assert!(first.starts_with(TOKEN_PREFIX));
        assert_eq!(first.len(), 48);
        assert_ne!(first, second);
        let random_bytes = URL_SAFE_NO_PAD
            .decode(first.trim_start_matches(TOKEN_PREFIX))
            .expect("random token component should be base64url");
        assert_eq!(random_bytes.len(), TOKEN_RANDOM_BYTES);

        let first_digest = token_digest(&first).expect("generated token should hash");
        assert_eq!(first_digest.len(), 32);
        assert_eq!(first_digest, Sha256::digest(first.as_bytes()).to_vec());
        assert_eq!(token_digest(&first), Some(first_digest));

        let legacy_token = format!(
            "{LEGACY_TOKEN_PREFIX}{}",
            URL_SAFE_NO_PAD.encode([7_u8; TOKEN_RANDOM_BYTES])
        );
        assert_eq!(
            token_digest(&legacy_token),
            Some(Sha256::digest(legacy_token.as_bytes()).to_vec())
        );
        assert!(token_digest("not-a-session-token").is_none());
    }
}
