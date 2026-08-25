use sqlx::PgPool;
use tracing::error;

use crate::credentials;

const MIN_USERNAME_BYTES: usize = 3;
const MAX_USERNAME_BYTES: usize = 32;
const MIN_PASSWORD_BYTES: usize = 12;
const MAX_PASSWORD_BYTES: usize = 128;

pub struct RegistrationInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisteredAccount {
    pub id: String,
    pub username: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistrationError {
    InvalidUsername,
    InvalidPassword,
    UsernameTaken,
    Internal,
}

pub async fn register_account(
    pool: &PgPool,
    input: RegistrationInput,
) -> Result<RegisteredAccount, RegistrationError> {
    let username = canonical_username(&input.username)?;
    validate_password(&input.password)?;

    let password_hash = credentials::hash_password(input.password)
        .await
        .map_err(|_| RegistrationError::Internal)?;

    let inserted = sqlx::query_as::<_, (String, String)>(
        r#"
        INSERT INTO accounts (username, password_hash)
        VALUES ($1, $2)
        RETURNING id::text, username
        "#,
    )
    .bind(&username)
    .bind(password_hash)
    .fetch_one(pool)
    .await;

    match inserted {
        Ok((id, username)) => Ok(RegisteredAccount { id, username }),
        Err(database_error) if is_username_conflict(&database_error) => {
            Err(RegistrationError::UsernameTaken)
        }
        Err(database_error) => {
            error!(error = %database_error, "account insertion failed");
            Err(RegistrationError::Internal)
        }
    }
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
