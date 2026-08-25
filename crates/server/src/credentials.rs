use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
    password_hash::{PasswordHasher, SaltString},
};
use rand_core::OsRng;
use tokio::{sync::Semaphore, task};
use tracing::error;

const ARGON2_MEMORY_KIB: u32 = 19 * 1024;
const ARGON2_ITERATIONS: u32 = 2;
const ARGON2_PARALLELISM: u32 = 1;
const MAX_CONCURRENT_PASSWORD_JOBS: usize = 4;

static PASSWORD_WORK_LIMIT: Semaphore = Semaphore::const_new(MAX_CONCURRENT_PASSWORD_JOBS);

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialError {
    Internal,
}

pub async fn hash_password(password: String) -> Result<String, CredentialError> {
    let _permit = PASSWORD_WORK_LIMIT
        .acquire()
        .await
        .map_err(|acquire_error| {
            error!(error = %acquire_error, "password work limit closed unexpectedly");
            CredentialError::Internal
        })?;
    task::spawn_blocking(move || hash_password_sync(&password))
        .await
        .map_err(|join_error| {
            error!(error = %join_error, "password hashing task failed");
            CredentialError::Internal
        })?
        .map_err(|hash_error| {
            error!(error = %hash_error, "password hashing failed");
            CredentialError::Internal
        })
}

pub async fn verify_password(
    password: String,
    stored_hash: Option<String>,
) -> Result<bool, CredentialError> {
    let _permit = PASSWORD_WORK_LIMIT
        .acquire()
        .await
        .map_err(|acquire_error| {
            error!(error = %acquire_error, "password work limit closed unexpectedly");
            CredentialError::Internal
        })?;
    task::spawn_blocking(move || verify_password_sync(&password, stored_hash.as_deref()))
        .await
        .map_err(|join_error| {
            error!(error = %join_error, "password verification task failed");
            CredentialError::Internal
        })?
        .map_err(|verification_error| {
            error!(error = %verification_error, "password verification failed internally");
            CredentialError::Internal
        })
}

fn password_hasher() -> Argon2<'static> {
    let parameters = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        None,
    )
    .expect("locked Argon2id parameters must remain valid");

    Argon2::new(Algorithm::Argon2id, Version::V0x13, parameters)
}

fn hash_password_sync(password: &str) -> argon2::password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    password_hasher()
        .hash_password(password.as_bytes(), &salt)
        .map(|password_hash| password_hash.to_string())
}

fn verify_password_sync(
    password: &str,
    stored_hash: Option<&str>,
) -> argon2::password_hash::Result<bool> {
    let Some(stored_hash) = stored_hash else {
        hash_password_sync(password)?;
        return Ok(false);
    };

    let parsed_hash = PasswordHash::new(stored_hash)?;
    Ok(password_hasher()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    use super::{hash_password, verify_password};

    #[tokio::test]
    async fn password_hashes_are_salted_parameterized_and_verifiable() {
        let password = "TEST-ONLY-registration-passphrase";
        let first = hash_password(password.to_owned())
            .await
            .expect("first hash should succeed");
        let second = hash_password(password.to_owned())
            .await
            .expect("second hash should succeed");

        assert_ne!(first, second, "each password hash needs a unique salt");
        assert!(first.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert_ne!(first, password);

        let parsed = PasswordHash::new(&first).expect("hash should be PHC encoded");
        assert!(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        );
    }

    #[tokio::test]
    async fn verification_handles_correct_wrong_and_missing_accounts() {
        let password = "TEST-ONLY-device-session-passphrase";
        let password_hash = hash_password(password.to_owned())
            .await
            .expect("hash should succeed");

        assert_eq!(
            verify_password(password.to_owned(), Some(password_hash.clone())).await,
            Ok(true)
        );
        assert_eq!(
            verify_password("TEST-ONLY-wrong-passphrase".to_owned(), Some(password_hash)).await,
            Ok(false)
        );
        assert_eq!(verify_password(password.to_owned(), None).await, Ok(false));
    }
}
