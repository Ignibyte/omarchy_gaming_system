use sqlx::{PgPool, Postgres, Transaction};
use tracing::{error, info};
use uuid::Uuid;

use crate::sessions;

const MAX_OPEN_REPORTS_PER_PERSONA: i64 = 25;
const MAX_REPORT_DETAIL_CHARACTERS: usize = 1000;

#[derive(Debug, PartialEq, Eq)]
pub struct PlayerReportReceipt {
    pub id: Uuid,
    pub idempotency_key: Uuid,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReportOutcome {
    Created(PlayerReportReceipt),
    Existing(PlayerReportReceipt),
}

pub struct CreateReportInput {
    pub idempotency_key: String,
    pub subject_persona_id: String,
    pub category: String,
    pub detail: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReportError {
    Unauthorized,
    PersonaNotFound,
    InvalidReport,
    IdempotencyConflict,
    OpenLimitReached,
    Internal,
}

type ReplayRow = (Uuid, Uuid, String, String, String);

pub async fn create_report(
    pool: &PgPool,
    token: &str,
    reporter_persona_id: &str,
    input: CreateReportInput,
) -> Result<ReportOutcome, ReportError> {
    let authenticated = sessions::authenticate(pool, token)
        .await
        .map_err(map_session_error)?;
    let reporter_id =
        Uuid::parse_str(reporter_persona_id).map_err(|_| ReportError::PersonaNotFound)?;
    let subject_id =
        Uuid::parse_str(&input.subject_persona_id).map_err(|_| ReportError::InvalidReport)?;
    let idempotency_key =
        Uuid::parse_str(&input.idempotency_key).map_err(|_| ReportError::InvalidReport)?;
    let category = validate_category(&input.category)?;
    let detail = validate_detail(&input.detail)?;
    if reporter_id == subject_id {
        return Err(ReportError::InvalidReport);
    }

    let mut transaction = pool.begin().await.map_err(|database_error| {
        error!(error = %database_error, "report transaction failed");
        ReportError::Internal
    })?;

    let owned_reporter = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM personas WHERE id = $1 AND account_id = $2 FOR UPDATE",
    )
    .bind(reporter_id)
    .bind(authenticated.account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|database_error| database_failure(database_error, "reporter lock"))?;
    if owned_reporter.is_none() {
        return Err(ReportError::PersonaNotFound);
    }

    if let Some(existing) = load_replay(&mut transaction, reporter_id, idempotency_key).await? {
        if existing.1 != subject_id || existing.2 != category || existing.3 != detail {
            return Err(ReportError::IdempotencyConflict);
        }
        let receipt = receipt_from_replay(existing, idempotency_key);
        transaction
            .commit()
            .await
            .map_err(|database_error| database_failure(database_error, "report replay commit"))?;
        return Ok(ReportOutcome::Existing(receipt));
    }

    let subject_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM personas WHERE id = $1)")
            .bind(subject_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|database_error| database_failure(database_error, "report subject lookup"))?;
    if !subject_exists {
        return Err(ReportError::PersonaNotFound);
    }

    let open_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM persona_reports WHERE reporter_persona_id = $1 AND status = 'open'",
    )
    .bind(reporter_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|database_error| database_failure(database_error, "open report count"))?;
    if open_count >= MAX_OPEN_REPORTS_PER_PERSONA {
        return Err(ReportError::OpenLimitReached);
    }

    let row = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        INSERT INTO persona_reports (
            reporter_persona_id,
            subject_persona_id,
            idempotency_key,
            category,
            detail
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        "#,
    )
    .bind(reporter_id)
    .bind(subject_id)
    .bind(idempotency_key)
    .bind(&category)
    .bind(&detail)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|database_error| database_failure(database_error, "report insertion"))?;

    transaction
        .commit()
        .await
        .map_err(|database_error| database_failure(database_error, "report commit"))?;
    info!(report_id = %row.0, reporter_persona_id = %reporter_id, "persona report created");
    Ok(ReportOutcome::Created(PlayerReportReceipt {
        id: row.0,
        idempotency_key,
        status: "open".to_owned(),
        created_at: row.1,
    }))
}

async fn load_replay(
    transaction: &mut Transaction<'_, Postgres>,
    reporter_id: Uuid,
    idempotency_key: Uuid,
) -> Result<Option<ReplayRow>, ReportError> {
    sqlx::query_as::<_, ReplayRow>(
        r#"
        SELECT
            id,
            subject_persona_id,
            category,
            detail,
            to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
        FROM persona_reports
        WHERE reporter_persona_id = $1 AND idempotency_key = $2
        FOR UPDATE
        "#,
    )
    .bind(reporter_id)
    .bind(idempotency_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|database_error| database_failure(database_error, "report replay lookup"))
}

fn receipt_from_replay(row: ReplayRow, idempotency_key: Uuid) -> PlayerReportReceipt {
    PlayerReportReceipt {
        id: row.0,
        idempotency_key,
        status: "open".to_owned(),
        created_at: row.4,
    }
}

fn validate_category(category: &str) -> Result<String, ReportError> {
    match category {
        "harassment" | "spam" | "cheating" | "other" => Ok(category.to_owned()),
        _ => Err(ReportError::InvalidReport),
    }
}

fn validate_detail(detail: &str) -> Result<String, ReportError> {
    let detail = detail.trim();
    let character_count = detail.chars().count();
    if !(1..=MAX_REPORT_DETAIL_CHARACTERS).contains(&character_count)
        || detail
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\t' | '\n' | '\r'))
    {
        return Err(ReportError::InvalidReport);
    }
    Ok(detail.to_owned())
}

fn map_session_error(error: sessions::SessionError) -> ReportError {
    match error {
        sessions::SessionError::Unauthorized => ReportError::Unauthorized,
        _ => ReportError::Internal,
    }
}

fn database_failure(database_error: sqlx::Error, operation: &'static str) -> ReportError {
    error!(error = %database_error, operation, "report database operation failed");
    ReportError::Internal
}
