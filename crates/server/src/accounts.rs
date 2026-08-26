use sqlx::{FromRow, PgPool};
use tracing::error;
use uuid::Uuid;

use crate::{credentials, registration_invites};

const MIN_USERNAME_BYTES: usize = 3;
const MAX_USERNAME_BYTES: usize = 32;
const MIN_PASSWORD_BYTES: usize = 12;
const MAX_PASSWORD_BYTES: usize = 128;

pub struct RegistrationInput {
    pub invite_code: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisteredAccount {
    pub id: String,
    pub username: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistrationOutcome {
    Created(RegisteredAccount),
    Existing(RegisteredAccount),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistrationError {
    InvalidUsername,
    InvalidPassword,
    InvalidInvitation,
    UsernameTaken,
    Internal,
}

#[derive(FromRow)]
struct InvitationRow {
    id: Uuid,
    revoked: bool,
    expired: bool,
    used_account_id: Option<Uuid>,
    used_username: Option<String>,
    used_password_hash: Option<String>,
}

pub async fn register_account(
    pool: &PgPool,
    input: RegistrationInput,
) -> Result<RegistrationOutcome, RegistrationError> {
    let username = canonical_username(&input.username)?;
    validate_password(&input.password)?;
    let code_hash = registration_invites::digest(&input.invite_code)
        .ok_or(RegistrationError::InvalidInvitation)?;

    let invitation = load_invitation(pool, &code_hash, false).await?;
    let Some(invitation) = invitation else {
        return Err(RegistrationError::InvalidInvitation);
    };
    if invitation.used_account_id.is_some() {
        return exact_replay(invitation, &username, input.password).await;
    }
    if invitation.revoked || invitation.expired {
        return Err(RegistrationError::InvalidInvitation);
    }

    let password_hash = credentials::hash_password(input.password.clone())
        .await
        .map_err(|_| RegistrationError::Internal)?;

    let mut transaction = pool.begin().await.map_err(|database_error| {
        error!(error = %database_error, "registration transaction failed");
        RegistrationError::Internal
    })?;
    let invitation = load_invitation(&mut *transaction, &code_hash, true).await?;
    let Some(invitation) = invitation else {
        return Err(RegistrationError::InvalidInvitation);
    };
    if invitation.used_account_id.is_some() {
        transaction.rollback().await.map_err(|database_error| {
            error!(error = %database_error, "registration replay rollback failed");
            RegistrationError::Internal
        })?;
        let refreshed = load_invitation(pool, &code_hash, false)
            .await?
            .ok_or(RegistrationError::Internal)?;
        return exact_replay(refreshed, &username, input.password).await;
    }
    if invitation.revoked || invitation.expired {
        return Err(RegistrationError::InvalidInvitation);
    }

    let inserted = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        INSERT INTO accounts (username, password_hash)
        VALUES ($1, $2)
        RETURNING id, username
        "#,
    )
    .bind(&username)
    .bind(password_hash)
    .fetch_one(&mut *transaction)
    .await;

    let (account_id, username) = match inserted {
        Ok(account) => account,
        Err(database_error) if is_username_conflict(&database_error) => {
            return Err(RegistrationError::UsernameTaken);
        }
        Err(database_error) => {
            error!(error = %database_error, "account insertion failed");
            return Err(RegistrationError::Internal);
        }
    };
    let consumed = sqlx::query(
        r#"
        UPDATE registration_invites
        SET used_at = clock_timestamp(), used_by_account_id = $2
        WHERE id = $1
          AND used_at IS NULL
          AND revoked_at IS NULL
          AND expires_at > clock_timestamp()
        "#,
    )
    .bind(invitation.id)
    .bind(account_id)
    .execute(&mut *transaction)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "registration invitation consumption failed");
        RegistrationError::Internal
    })?;
    if consumed.rows_affected() != 1 {
        return Err(RegistrationError::InvalidInvitation);
    }
    transaction.commit().await.map_err(|database_error| {
        error!(error = %database_error, "registration commit failed");
        RegistrationError::Internal
    })?;

    Ok(RegistrationOutcome::Created(RegisteredAccount {
        id: account_id.to_string(),
        username,
    }))
}

async fn load_invitation<'e, E>(
    executor: E,
    code_hash: &[u8; 32],
    lock: bool,
) -> Result<Option<InvitationRow>, RegistrationError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let query = if lock {
        r#"
        SELECT invitation.id,
               invitation.revoked_at IS NOT NULL AS revoked,
               invitation.expires_at <= clock_timestamp() AS expired,
               invitation.used_by_account_id AS used_account_id,
               account.username AS used_username,
               account.password_hash AS used_password_hash
        FROM registration_invites AS invitation
        LEFT JOIN accounts AS account ON account.id = invitation.used_by_account_id
        WHERE invitation.code_hash = $1
        FOR UPDATE OF invitation
        "#
    } else {
        r#"
        SELECT invitation.id,
               invitation.revoked_at IS NOT NULL AS revoked,
               invitation.expires_at <= clock_timestamp() AS expired,
               invitation.used_by_account_id AS used_account_id,
               account.username AS used_username,
               account.password_hash AS used_password_hash
        FROM registration_invites AS invitation
        LEFT JOIN accounts AS account ON account.id = invitation.used_by_account_id
        WHERE invitation.code_hash = $1
        "#
    };
    sqlx::query_as::<_, InvitationRow>(query)
        .bind(code_hash.as_slice())
        .fetch_optional(executor)
        .await
        .map_err(|database_error| {
            error!(error = %database_error, "registration invitation lookup failed");
            RegistrationError::Internal
        })
}

async fn exact_replay(
    invitation: InvitationRow,
    username: &str,
    password: String,
) -> Result<RegistrationOutcome, RegistrationError> {
    let (Some(account_id), Some(stored_username), Some(password_hash)) = (
        invitation.used_account_id,
        invitation.used_username,
        invitation.used_password_hash,
    ) else {
        return Err(RegistrationError::Internal);
    };
    let username_matches = stored_username == username;
    // A resolved invitation is a bearer secret. Keep every credential mismatch on
    // the same Argon2 path so replay responses do not reveal its linked username.
    let verified = credentials::verify_password(password, Some(password_hash))
        .await
        .map_err(|_| RegistrationError::Internal)?;
    if !username_matches || !verified {
        return Err(RegistrationError::InvalidInvitation);
    }
    Ok(RegistrationOutcome::Existing(RegisteredAccount {
        id: account_id.to_string(),
        username: stored_username,
    }))
}

pub(crate) fn canonical_username(raw_username: &str) -> Result<String, RegistrationError> {
    let username = raw_username.trim().to_ascii_lowercase();
    let bytes = username.as_bytes();

    if !(MIN_USERNAME_BYTES..=MAX_USERNAME_BYTES).contains(&bytes.len())
        || !bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(RegistrationError::InvalidUsername);
    }

    Ok(username)
}

fn validate_password(password: &str) -> Result<(), RegistrationError> {
    if (MIN_PASSWORD_BYTES..=MAX_PASSWORD_BYTES).contains(&password.len()) {
        Ok(())
    } else {
        Err(RegistrationError::InvalidPassword)
    }
}

fn is_username_conflict(error: &sqlx::Error) -> bool {
    let Some(database_error) = error.as_database_error() else {
        return false;
    };

    database_error.code().as_deref() == Some("23505")
        && database_error.constraint() == Some("accounts_username_unique_ci")
}

#[cfg(test)]
pub(crate) async fn create_test_invite(pool: &PgPool) -> String {
    let (code, code_hash) = registration_invites::generate().expect("test invite should generate");
    sqlx::query(
        r#"
        WITH moment AS (SELECT clock_timestamp() AS created_at)
        INSERT INTO registration_invites (
            code_hash, label, valid_for_hours, issued_operation_id,
            issued_by, issued_reason, created_at, expires_at
        )
        SELECT $1, 'test fixture', 24, $2,
               'test-suite', 'Create isolated registration fixture',
               created_at, created_at + interval '24 hours'
        FROM moment
        "#,
    )
    .bind(code_hash.as_slice())
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("test invite should persist");
    code
}

#[cfg(test)]
mod tests {
    use super::{RegistrationError, canonical_username, validate_password};

    #[test]
    fn username_is_trimmed_lowercased_and_validated() {
        assert_eq!(
            canonical_username("  Player_One  "),
            Ok("player_one".to_owned())
        );

        for invalid in [
            "ab",
            "-player",
            "player one",
            "pláyer",
            "player!",
            "abcdefghijklmnopqrstuvwxyz1234567",
        ] {
            assert_eq!(
                canonical_username(invalid),
                Err(RegistrationError::InvalidUsername),
                "expected {invalid:?} to be rejected"
            );
        }
    }

    #[test]
    fn password_length_is_bounded_without_mutating_input() {
        assert_eq!(
            validate_password("eleven-byte"),
            Err(RegistrationError::InvalidPassword)
        );
        assert_eq!(validate_password("twelve-bytes"), Ok(()));
        assert_eq!(validate_password(&"x".repeat(128)), Ok(()));
        assert_eq!(
            validate_password(&"x".repeat(129)),
            Err(RegistrationError::InvalidPassword)
        );
    }
}
