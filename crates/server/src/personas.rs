use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::sessions::{self, SessionError};

const MIN_HANDLE_BYTES: usize = 3;
const MAX_HANDLE_BYTES: usize = 24;
const MAX_DISPLAY_NAME_CHARACTERS: usize = 64;
const MAX_BIO_CHARACTERS: usize = 1_000;
const MAX_STATUS_MESSAGE_CHARACTERS: usize = 160;
const HANDLE_UNIQUE_CONSTRAINT: &str = "personas_handle_unique_ci";

pub struct CreatePersonaInput {
    pub handle: String,
    pub display_name: String,
    pub bio: String,
    pub status_message: String,
}

pub struct UpdatePersonaInput {
    pub handle: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub status_message: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Persona {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub bio: String,
    pub status_message: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PersonaError {
    InvalidHandle,
    InvalidDisplayName,
    InvalidBio,
    InvalidStatusMessage,
    EmptyPatch,
    HandleTaken,
    Unauthorized,
    PersonaNotFound,
    Internal,
}

type PersonaRow = (Uuid, String, String, String, String, String, String);

pub async fn create_persona(
    pool: &PgPool,
    token: &str,
    input: CreatePersonaInput,
) -> Result<Persona, PersonaError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let handle = canonical_handle(&input.handle)?;
    let display_name = validate_display_name(&input.display_name)?;
    let bio = validate_bio(&input.bio)?;
    let status_message = validate_status_message(&input.status_message)?;

    let row = sqlx::query_as::<_, PersonaRow>(
        r#"
        INSERT INTO personas (account_id, handle, display_name, bio, status_message)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            handle,
            display_name,
            bio,
            status_message,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        "#,
    )
    .bind(account_id)
    .bind(handle)
    .bind(display_name)
    .bind(bio)
    .bind(status_message)
    .fetch_one(pool)
    .await
    .map_err(|database_error| map_write_error(database_error, "persona creation"))?;

    let persona = persona_from_row(row);
    info!(persona_id = %persona.id, "persona created");
    Ok(persona)
}

pub async fn list_personas(pool: &PgPool, token: &str) -> Result<Vec<Persona>, PersonaError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let rows = sqlx::query_as::<_, PersonaRow>(
        r#"
        SELECT
            id,
            handle,
            display_name,
            bio,
            status_message,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        FROM personas
        WHERE account_id = $1
        ORDER BY created_at, id
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "persona listing failed");
        PersonaError::Internal
    })?;

    Ok(rows.into_iter().map(persona_from_row).collect())
}

pub async fn get_persona_by_handle(pool: &PgPool, handle: &str) -> Result<Persona, PersonaError> {
    let handle = canonical_handle(handle).map_err(|_| PersonaError::PersonaNotFound)?;
    let row = sqlx::query_as::<_, PersonaRow>(
        r#"
        SELECT
            id,
            handle,
            display_name,
            bio,
            status_message,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        FROM personas
        WHERE handle = $1
        "#,
    )
    .bind(handle)
    .fetch_optional(pool)
    .await
    .map_err(|database_error| {
        error!(error = %database_error, "public persona lookup failed");
        PersonaError::Internal
    })?;

    row.map(persona_from_row)
        .ok_or(PersonaError::PersonaNotFound)
}

pub async fn update_persona(
    pool: &PgPool,
    token: &str,
    persona_id: &str,
    input: UpdatePersonaInput,
) -> Result<Persona, PersonaError> {
    let account_id = authenticated_account_id(pool, token).await?;
    let persona_id = Uuid::try_parse(persona_id).map_err(|_| PersonaError::PersonaNotFound)?;
    let handle = input
        .handle
        .map(|handle| canonical_handle(&handle))
        .transpose()?;
    let display_name = input
        .display_name
        .map(|display_name| validate_display_name(&display_name))
        .transpose()?;
    let bio = input.bio.map(|bio| validate_bio(&bio)).transpose()?;
    let status_message = input
        .status_message
        .map(|status_message| validate_status_message(&status_message))
        .transpose()?;

    if handle.is_none() && display_name.is_none() && bio.is_none() && status_message.is_none() {
        return Err(PersonaError::EmptyPatch);
    }

    let row = sqlx::query_as::<_, PersonaRow>(
        r#"
        UPDATE personas
        SET
            handle = COALESCE($3, handle),
            display_name = COALESCE($4, display_name),
            bio = COALESCE($5, bio),
            status_message = COALESCE($6, status_message),
            updated_at = now()
        WHERE id = $1 AND account_id = $2
        RETURNING
            id,
            handle,
            display_name,
            bio,
            status_message,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
            to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        "#,
    )
    .bind(persona_id)
    .bind(account_id)
    .bind(handle)
    .bind(display_name)
    .bind(bio)
    .bind(status_message)
    .fetch_optional(pool)
    .await
    .map_err(|database_error| map_write_error(database_error, "persona update"))?;

    let persona = row
        .map(persona_from_row)
        .ok_or(PersonaError::PersonaNotFound)?;
    info!(persona_id = %persona.id, "persona updated");
    Ok(persona)
}

async fn authenticated_account_id(pool: &PgPool, token: &str) -> Result<Uuid, PersonaError> {
    sessions::authenticate(pool, token)
        .await
        .map(|authenticated| authenticated.account_id)
        .map_err(|session_error| match session_error {
            SessionError::Unauthorized => PersonaError::Unauthorized,
            _ => PersonaError::Internal,
        })
}

fn canonical_handle(handle: &str) -> Result<String, PersonaError> {
    let handle = handle.trim().to_ascii_lowercase();
    let bytes = handle.as_bytes();

    if !(MIN_HANDLE_BYTES..=MAX_HANDLE_BYTES).contains(&bytes.len())
        || !bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        || !bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
    {
        return Err(PersonaError::InvalidHandle);
    }

    Ok(handle)
}

fn validate_display_name(display_name: &str) -> Result<String, PersonaError> {
    let display_name = display_name.trim();
    let character_count = display_name.chars().count();

    if !(1..=MAX_DISPLAY_NAME_CHARACTERS).contains(&character_count)
        || display_name.chars().any(char::is_control)
    {
        return Err(PersonaError::InvalidDisplayName);
    }

    Ok(display_name.to_owned())
}

fn validate_bio(bio: &str) -> Result<String, PersonaError> {
    if bio.chars().count() > MAX_BIO_CHARACTERS
        || bio
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
    {
        return Err(PersonaError::InvalidBio);
    }

    Ok(bio.to_owned())
}

fn validate_status_message(status_message: &str) -> Result<String, PersonaError> {
    let status_message = status_message.trim();

    if status_message.chars().count() > MAX_STATUS_MESSAGE_CHARACTERS
        || status_message.chars().any(char::is_control)
    {
        return Err(PersonaError::InvalidStatusMessage);
    }

    Ok(status_message.to_owned())
}

fn map_write_error(database_error: sqlx::Error, operation: &'static str) -> PersonaError {
    if database_error
        .as_database_error()
        .and_then(|error| error.constraint())
        == Some(HANDLE_UNIQUE_CONSTRAINT)
    {
        return PersonaError::HandleTaken;
    }

    error!(error = %database_error, operation, "persona database write failed");
    PersonaError::Internal
}

fn persona_from_row(row: PersonaRow) -> Persona {
    Persona {
        id: row.0,
        handle: row.1,
        display_name: row.2,
        bio: row.3,
        status_message: row.4,
        created_at: row.5,
        updated_at: row.6,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PersonaError, canonical_handle, validate_bio, validate_display_name,
        validate_status_message,
    };

    #[test]
    fn handle_is_canonical_and_ascii_only() {
        assert_eq!(
            canonical_handle("  Player_One  "),
            Ok("player_one".to_owned())
        );
        assert_eq!(canonical_handle("ab"), Err(PersonaError::InvalidHandle));
        assert_eq!(
            canonical_handle("-player"),
            Err(PersonaError::InvalidHandle)
        );
        assert_eq!(
            canonical_handle("player.name"),
            Err(PersonaError::InvalidHandle)
        );
        assert_eq!(canonical_handle("pláyer"), Err(PersonaError::InvalidHandle));
        assert!(canonical_handle(&"a".repeat(24)).is_ok());
        assert_eq!(
            canonical_handle(&"a".repeat(25)),
            Err(PersonaError::InvalidHandle)
        );
    }

    #[test]
    fn public_profile_text_is_bounded_and_control_safe() {
        assert_eq!(
            validate_display_name("  Player One  "),
            Ok("Player One".to_owned())
        );
        assert_eq!(
            validate_display_name("\n"),
            Err(PersonaError::InvalidDisplayName)
        );
        assert!(validate_display_name(&"é".repeat(64)).is_ok());
        assert_eq!(
            validate_display_name(&"é".repeat(65)),
            Err(PersonaError::InvalidDisplayName)
        );

        assert_eq!(
            validate_bio("line one\n\tline two"),
            Ok("line one\n\tline two".to_owned())
        );
        assert_eq!(validate_bio("bad\rline"), Err(PersonaError::InvalidBio));
        assert!(validate_bio(&"界".repeat(1_000)).is_ok());
        assert_eq!(
            validate_bio(&"界".repeat(1_001)),
            Err(PersonaError::InvalidBio)
        );

        assert_eq!(validate_status_message("  Ready  "), Ok("Ready".to_owned()));
        assert_eq!(
            validate_status_message("not\tready"),
            Err(PersonaError::InvalidStatusMessage)
        );
        assert!(validate_status_message(&"x".repeat(160)).is_ok());
        assert_eq!(
            validate_status_message(&"x".repeat(161)),
            Err(PersonaError::InvalidStatusMessage)
        );
    }
}
