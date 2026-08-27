//! Immutable Game Cartridge bindings for authoritative game sessions.

use omarchygs_game_cartridge::{
    ActiveSessionDecision, CatalogPublicKey, CatalogStatus, LifecycleUse, PublisherPublicKey,
    SignedCatalogPolicy, lifecycle_decision, validate_screen_action,
};
use serde::Serialize;
use serde_json::{Map, Value, json};
use sqlx::{FromRow, PgPool, Postgres, Transaction, types::Json};
use uuid::Uuid;

use crate::cartridge_distribution::CartridgeDistributionRuntime;

/// Stable non-secret presentation facts exposed with a participant-authorized
/// game session.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionCartridgePresentation {
    pub format: &'static str,
    pub publisher_id: String,
    pub game_key: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub admission_revision: u64,
    pub lifecycle_status: String,
    pub active_session_policy: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// A cartridge action translated by the host after the exact signed screen
/// contract was verified.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedSessionCartridgeAction {
    pub archive_sha256: String,
    pub authority: String,
    pub command: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCartridgeError {
    InvalidInput,
    NotFound,
    Denied,
    RevisionConflict,
    IdempotencyConflict,
    Completed,
    Internal,
}

impl SessionCartridgeError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "session_cartridge_invalid_input",
            Self::NotFound => "session_cartridge_not_found",
            Self::Denied => "session_cartridge_denied",
            Self::RevisionConflict => "session_cartridge_revision_conflict",
            Self::IdempotencyConflict => "session_cartridge_idempotency_conflict",
            Self::Completed => "session_cartridge_completed",
            Self::Internal => "session_cartridge_internal",
        }
    }
}

/// Pin the currently effective exact release to a newly inserted game session.
///
/// Absence or an expected provider digest mismatch is an honest unbound
/// session. Once a candidate exists, trust/key/store corruption is an error so
/// the surrounding session transaction cannot commit a false presentation.
pub async fn pin_new_session(
    transaction: &mut Transaction<'_, Postgres>,
    runtime: &CartridgeDistributionRuntime,
    game_session_id: Uuid,
    game_key: &str,
    rules_version: u32,
    expected_archive_sha256: Option<&str>,
) -> Result<bool, SessionCartridgeError> {
    if game_session_id.is_nil()
        || !valid_identifier(game_key)
        || rules_version == 0
        || expected_archive_sha256.is_some_and(|digest| !valid_sha256(digest))
    {
        return Err(SessionCartridgeError::InvalidInput);
    }
    let row = sqlx::query_as::<_, PinnableReleaseRow>(
        r#"
        SELECT release.id AS release_id,
               release.game_key,
               release.publisher_id,
               release.publisher_key,
               release.rules_version,
               release.cartridge_version,
               release.archive_sha256,
               release.signed_identity_sha256,
               release.signed_policy,
               catalog.admission_revision,
               sync.marketplace_key
        FROM server_cartridge_catalogs AS catalog
        JOIN marketplace_releases AS release
          ON release.id = catalog.active_release_id
        JOIN marketplace_release_acquisition_evidence AS evidence
          ON evidence.marketplace_release_id = release.id
        JOIN marketplace_sync_state AS sync
          ON sync.singleton
        WHERE catalog.game_key = $1
          AND release.rules_version = $2
          AND release.imported
          AND release.compatible
          AND release.last_seen_snapshot_version = sync.snapshot_version
          AND release.policy_status IN ('active', 'deprecated')
        FOR SHARE OF release, catalog, sync
        "#,
    )
    .bind(game_key)
    .bind(i64::from(rules_version))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| SessionCartridgeError::Internal)?;
    let Some(row) = row else {
        return Ok(false);
    };
    if expected_archive_sha256.is_some_and(|digest| digest != row.archive_sha256.as_str()) {
        return Ok(false);
    }
    let database_key = row.marketplace_key.ok_or(SessionCartridgeError::Denied)?.0;
    if &database_key != runtime.marketplace_key() {
        return Err(SessionCartridgeError::Denied);
    }
    let policy_bytes =
        serde_json::to_vec(&row.signed_policy.0).map_err(|_| SessionCartridgeError::Internal)?;
    let resolution = runtime
        .resolve_exact_release(
            &row.game_key,
            &row.archive_sha256,
            &row.publisher_key.0,
            &policy_bytes,
            LifecycleUse::NewLaunch,
        )
        .map_err(|_| SessionCartridgeError::Denied)?;
    let activation = resolution.activation();
    if activation.game_key != row.game_key
        || activation.publisher_id != row.publisher_id
        || activation.cartridge_version
            != u32::try_from(row.cartridge_version).map_err(|_| SessionCartridgeError::Internal)?
        || activation.archive_sha256 != row.archive_sha256
        || activation.signed_identity_sha256 != row.signed_identity_sha256
        || resolution.cartridge().manifest().rules_version != rules_version
    {
        return Err(SessionCartridgeError::Denied);
    }
    sqlx::query(
        r#"
        INSERT INTO game_session_cartridge_presentations (
            game_session_id, marketplace_release_id, admission_revision
        )
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(game_session_id)
    .bind(row.release_id)
    .bind(row.admission_revision)
    .execute(&mut **transaction)
    .await
    .map_err(|_| SessionCartridgeError::Internal)?;
    Ok(true)
}

/// Durably admit one participant-scoped action against the exact cartridge
/// pinned to the session.
///
/// The shared marketplace-snapshot advisory lock is the lifecycle
/// linearization point. The lock and database transaction end before compiled
/// execution or provider I/O; an exact idempotent replay can recover the
/// admitted intent even after a later suspension or revocation.
#[allow(clippy::too_many_arguments)]
pub async fn admit_session_action(
    pool: &PgPool,
    runtime: &CartridgeDistributionRuntime,
    actor_id: Uuid,
    game_session_id: Uuid,
    idempotency_key: Uuid,
    expected_revision: i64,
    archive_sha256: &str,
    screen_id: Option<&str>,
    action: &str,
    payload: &Value,
) -> Result<ValidatedSessionCartridgeAction, SessionCartridgeError> {
    if actor_id.is_nil()
        || game_session_id.is_nil()
        || idempotency_key.is_nil()
        || expected_revision < 0
        || !valid_sha256(archive_sha256)
        || screen_id.is_some_and(|screen| !valid_identifier(screen))
        || !valid_identifier(action)
        || !payload.is_object()
    {
        return Err(SessionCartridgeError::InvalidInput);
    }
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| SessionCartridgeError::Internal)?;
    sqlx::query("SELECT pg_advisory_xact_lock_shared($1)")
        .bind(crate::cartridge_catalog::SNAPSHOT_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(|_| SessionCartridgeError::Internal)?;
    let row = sqlx::query_as::<_, ActionReleaseRow>(
        r#"
        SELECT session.revision,
               session.status,
               session.authority,
               session.game_key,
               session.game_version,
               presentation.marketplace_release_id,
               presentation.admission_revision,
               release.publisher_id,
               release.publisher_key,
               release.cartridge_version,
               release.archive_sha256,
               release.signed_identity_sha256,
               release.signed_policy,
               release.policy_version,
               release.policy_status,
               sync.marketplace_key
        FROM game_sessions AS session
        JOIN game_session_participants AS participant
          ON participant.game_session_id = session.id
         AND participant.persona_id = $1
        JOIN game_session_cartridge_presentations AS presentation
          ON presentation.game_session_id = session.id
        JOIN marketplace_releases AS release
          ON release.id = presentation.marketplace_release_id
        JOIN marketplace_sync_state AS sync
          ON sync.singleton
        WHERE session.id = $2
        FOR UPDATE OF session
        "#,
    )
    .bind(actor_id)
    .bind(game_session_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| SessionCartridgeError::Internal)?
    .ok_or(SessionCartridgeError::NotFound)?;
    let replay = sqlx::query_as::<_, ActionAdmissionReplayRow>(
        r#"
        SELECT actor_persona_id = $3 AS actor_matches,
               expected_revision = $4 AS revision_matches,
               archive_sha256 = $5 AS archive_matches,
               action = $6 AS action_matches,
               payload = $7 AS payload_matches,
               screen_id IS NULL OR (
                   screen_explicit = ($8::text IS NOT NULL)
                   AND (NOT screen_explicit OR screen_id = $8)
               ) AS screen_matches,
               authority,
               archive_sha256,
               translated_command
        FROM game_session_cartridge_action_admissions
        WHERE game_session_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(game_session_id)
    .bind(idempotency_key)
    .bind(actor_id)
    .bind(expected_revision)
    .bind(archive_sha256)
    .bind(action)
    .bind(Json(payload))
    .bind(screen_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| SessionCartridgeError::Internal)?;
    if let Some(replay) = replay {
        if !replay.actor_matches
            || !replay.revision_matches
            || !replay.archive_matches
            || !replay.action_matches
            || !replay.payload_matches
            || !replay.screen_matches
        {
            return Err(SessionCartridgeError::IdempotencyConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| SessionCartridgeError::Internal)?;
        return Ok(ValidatedSessionCartridgeAction {
            archive_sha256: replay.archive_sha256,
            authority: replay.authority,
            command: replay.translated_command.0,
        });
    }
    if row.status == "completed" {
        return Err(SessionCartridgeError::Completed);
    }
    if row.status != "active" {
        return Err(SessionCartridgeError::Denied);
    }
    if row.revision != expected_revision {
        return Err(SessionCartridgeError::RevisionConflict);
    }
    if row.archive_sha256 != archive_sha256 {
        return Err(SessionCartridgeError::Denied);
    }
    let database_key = row.marketplace_key.ok_or(SessionCartridgeError::Denied)?.0;
    if &database_key != runtime.marketplace_key() {
        return Err(SessionCartridgeError::Denied);
    }
    let policy_bytes =
        serde_json::to_vec(&row.signed_policy.0).map_err(|_| SessionCartridgeError::Internal)?;
    let resolution = runtime
        .resolve_exact_release(
            &row.game_key,
            &row.archive_sha256,
            &row.publisher_key.0,
            &policy_bytes,
            LifecycleUse::ActiveSession,
        )
        .map_err(|_| SessionCartridgeError::Denied)?;
    let manifest = resolution.cartridge().manifest();
    if manifest.game_key != row.game_key
        || i64::from(manifest.rules_version) != row.game_version
        || manifest.publisher_id != row.publisher_id
        || i64::from(manifest.cartridge_version) != row.cartridge_version
        || resolution.cartridge().archive_sha256() != row.archive_sha256
        || resolution.cartridge().signed_identity_sha256() != row.signed_identity_sha256
        || row.admission_revision <= 0
    {
        return Err(SessionCartridgeError::Denied);
    }
    let effective_screen = screen_id.unwrap_or(&manifest.entry_screen);
    validate_screen_action(resolution.cartridge(), effective_screen, action, payload)
        .map_err(|_| SessionCartridgeError::Denied)?;
    let command = translate_command(&row.authority, &row.game_key, action, payload)?;
    sqlx::query(
        r#"
        INSERT INTO game_session_cartridge_action_admissions (
            game_session_id,
            idempotency_key,
            actor_persona_id,
            marketplace_release_id,
            admission_revision,
            authority,
            expected_revision,
            archive_sha256,
            signed_identity_sha256,
            policy_version,
            lifecycle_status,
            screen_id,
            screen_explicit,
            action,
            payload,
            translated_command
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8,
            $9, $10, $11, $12, $13, $14, $15, $16
        )
        "#,
    )
    .bind(game_session_id)
    .bind(idempotency_key)
    .bind(actor_id)
    .bind(row.marketplace_release_id)
    .bind(row.admission_revision)
    .bind(&row.authority)
    .bind(expected_revision)
    .bind(&row.archive_sha256)
    .bind(&row.signed_identity_sha256)
    .bind(row.policy_version)
    .bind(&row.policy_status)
    .bind(effective_screen)
    .bind(screen_id.is_some())
    .bind(action)
    .bind(Json(payload))
    .bind(Json(&command))
    .execute(&mut *transaction)
    .await
    .map_err(|_| SessionCartridgeError::Internal)?;
    transaction
        .commit()
        .await
        .map_err(|_| SessionCartridgeError::Internal)?;
    Ok(ValidatedSessionCartridgeAction {
        archive_sha256: row.archive_sha256,
        authority: row.authority,
        command,
    })
}

/// Convert nullable joined database fields into the exact public projection.
#[allow(clippy::too_many_arguments)]
pub fn project_presentation(
    publisher_id: String,
    game_key: String,
    rules_version: i64,
    cartridge_version: i64,
    archive_sha256: String,
    signed_identity_sha256: String,
    admission_revision: i64,
    lifecycle_status: String,
    lifecycle_reason: String,
) -> Result<SessionCartridgePresentation, SessionCartridgeError> {
    let status = parse_catalog_status(&lifecycle_status)?;
    let active_session_policy = match lifecycle_decision(status).active_session {
        ActiveSessionDecision::Continue => "continue",
        ActiveSessionDecision::Suspend => "suspend",
        ActiveSessionDecision::Terminate => "terminate",
    };
    if !valid_identifier(&publisher_id)
        || !valid_identifier(&game_key)
        || !valid_sha256(&archive_sha256)
        || !valid_sha256(&signed_identity_sha256)
        || admission_revision <= 0
        || !valid_plain_text(&lifecycle_reason, 512)
    {
        return Err(SessionCartridgeError::Internal);
    }
    Ok(SessionCartridgePresentation {
        format: "omarchygs.session-cartridge/v1",
        publisher_id,
        game_key,
        rules_version: u32::try_from(rules_version).map_err(|_| SessionCartridgeError::Internal)?,
        cartridge_version: u32::try_from(cartridge_version)
            .map_err(|_| SessionCartridgeError::Internal)?,
        archive_sha256,
        signed_identity_sha256,
        admission_revision: u64::try_from(admission_revision)
            .map_err(|_| SessionCartridgeError::Internal)?,
        lifecycle_status,
        active_session_policy,
        warning: (status == CatalogStatus::Deprecated).then_some(lifecycle_reason),
    })
}

fn translate_command(
    authority: &str,
    game_key: &str,
    action: &str,
    payload: &Value,
) -> Result<Value, SessionCartridgeError> {
    let object = payload
        .as_object()
        .ok_or(SessionCartridgeError::InvalidInput)?;
    if authority == "platform_compiled" && game_key == "signal_siege" {
        if !object.is_empty() || !matches!(action, "strike" | "guard" | "charge") {
            return Err(SessionCartridgeError::Denied);
        }
        return Ok(json!({"kind": "play", "action": action}));
    }
    if !matches!(authority, "platform_compiled" | "registered_provider") {
        return Err(SessionCartridgeError::Denied);
    }
    let mut command = Map::new();
    command.insert("action".to_owned(), Value::String(action.to_owned()));
    for (key, value) in object {
        if key == "action" || command.insert(key.clone(), value.clone()).is_some() {
            return Err(SessionCartridgeError::Denied);
        }
    }
    Ok(Value::Object(command))
}

fn parse_catalog_status(value: &str) -> Result<CatalogStatus, SessionCartridgeError> {
    match value {
        "active" => Ok(CatalogStatus::Active),
        "deprecated" => Ok(CatalogStatus::Deprecated),
        "suspended" => Ok(CatalogStatus::Suspended),
        "revoked" => Ok(CatalogStatus::Revoked),
        "retired" => Ok(CatalogStatus::Retired),
        _ => Err(SessionCartridgeError::Internal),
    }
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_plain_text(value: &str, max_chars: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

#[derive(FromRow)]
struct PinnableReleaseRow {
    release_id: Uuid,
    game_key: String,
    publisher_id: String,
    publisher_key: Json<PublisherPublicKey>,
    #[allow(dead_code)]
    rules_version: i64,
    cartridge_version: i64,
    archive_sha256: String,
    signed_identity_sha256: String,
    signed_policy: Json<SignedCatalogPolicy>,
    admission_revision: i64,
    marketplace_key: Option<Json<CatalogPublicKey>>,
}

#[derive(FromRow)]
struct ActionReleaseRow {
    revision: i64,
    status: String,
    authority: String,
    game_key: String,
    game_version: i64,
    marketplace_release_id: Uuid,
    admission_revision: i64,
    publisher_id: String,
    publisher_key: Json<PublisherPublicKey>,
    cartridge_version: i64,
    archive_sha256: String,
    signed_identity_sha256: String,
    signed_policy: Json<SignedCatalogPolicy>,
    policy_version: i64,
    policy_status: String,
    marketplace_key: Option<Json<CatalogPublicKey>>,
}

#[derive(FromRow)]
struct ActionAdmissionReplayRow {
    actor_matches: bool,
    revision_matches: bool,
    archive_matches: bool,
    action_matches: bool,
    payload_matches: bool,
    screen_matches: bool,
    authority: String,
    archive_sha256: String,
    translated_command: Json<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_translation_keeps_the_host_in_control() {
        assert_eq!(
            translate_command("registered_provider", "door-legends", "enter", &json!({})),
            Ok(json!({"action": "enter"}))
        );
        assert_eq!(
            translate_command("platform_compiled", "signal_siege", "strike", &json!({})),
            Ok(json!({"kind": "play", "action": "strike"}))
        );
        assert_eq!(
            translate_command(
                "registered_provider",
                "grid-game",
                "move",
                &json!({"column": 2, "row": 3})
            ),
            Ok(json!({"action": "move", "column": 2, "row": 3}))
        );
        assert_eq!(
            translate_command("platform_compiled", "signal_siege", "move", &json!({})),
            Err(SessionCartridgeError::Denied)
        );
    }
}
