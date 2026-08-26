use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

pub const MAX_OPERATOR_DOCUMENT_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorError {
    InvalidInput,
    NotFound,
    Conflict,
    Denied,
    Internal,
}

impl OperatorError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "operator_invalid_input",
            Self::NotFound => "operator_not_found",
            Self::Conflict => "operator_conflict",
            Self::Denied => "operator_denied",
            Self::Internal => "operator_internal",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Suspended,
}

impl AccountStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Resolved,
    Dismissed,
}

impl ReportStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Dismissed => "dismissed",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum OperatorCommand {
    SetAccountStatus {
        idempotency_key: Uuid,
        account_id: Uuid,
        status: AccountStatus,
        actor: String,
        reason: String,
    },
    SetReportStatus {
        idempotency_key: Uuid,
        report_id: Uuid,
        status: ReportStatus,
        actor: String,
        reason: String,
    },
}

impl OperatorCommand {
    pub fn validate(&self) -> Result<(), OperatorError> {
        let (operation_id, target_id, actor, reason) = match self {
            Self::SetAccountStatus {
                idempotency_key,
                account_id,
                actor,
                reason,
                ..
            } => (idempotency_key, account_id, actor, reason),
            Self::SetReportStatus {
                idempotency_key,
                report_id,
                actor,
                reason,
                ..
            } => (idempotency_key, report_id, actor, reason),
        };
        if operation_id.is_nil()
            || target_id.is_nil()
            || !valid_text(actor, 64)
            || !valid_text(reason, 500)
        {
            Err(OperatorError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuditReceipt {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub target_kind: String,
    pub target_id: Uuid,
    pub action: String,
    pub previous_state: String,
    pub resulting_state: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PublicPersona {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    pub bio: String,
    pub status_message: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OperatorReport {
    pub id: Uuid,
    pub reporter: PublicPersona,
    pub subject: PublicPersona,
    pub subject_account_id: Uuid,
    pub category: String,
    pub detail: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReportInventory {
    pub reports: Vec<OperatorReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFilter {
    Open,
    Resolved,
    Dismissed,
    All,
}

impl ReportFilter {
    pub fn parse(value: &str) -> Result<Self, OperatorError> {
        match value {
            "open" => Ok(Self::Open),
            "resolved" => Ok(Self::Resolved),
            "dismissed" => Ok(Self::Dismissed),
            "all" => Ok(Self::All),
            _ => Err(OperatorError::InvalidInput),
        }
    }

    const fn query_value(self) -> Option<&'static str> {
        match self {
            Self::Open => Some("open"),
            Self::Resolved => Some("resolved"),
            Self::Dismissed => Some("dismissed"),
            Self::All => None,
        }
    }
}

#[derive(FromRow)]
struct ReportRow {
    id: Uuid,
    reporter_id: Uuid,
    reporter_handle: String,
    reporter_display_name: String,
    reporter_bio: String,
    reporter_status_message: String,
    reporter_created_at: String,
    reporter_updated_at: String,
    subject_id: Uuid,
    subject_account_id: Uuid,
    subject_handle: String,
    subject_display_name: String,
    subject_bio: String,
    subject_status_message: String,
    subject_created_at: String,
    subject_updated_at: String,
    category: String,
    detail: String,
    status: String,
    created_at: String,
    updated_at: String,
    closed_at: Option<String>,
}

#[derive(FromRow)]
struct AuditRow {
    id: Uuid,
    operation_id: Uuid,
    target_kind: String,
    target_id: Uuid,
    action: String,
    actor: String,
    reason: String,
    previous_state: String,
    resulting_state: String,
    created_at: String,
}

pub async fn list_reports(
    pool: &PgPool,
    filter: ReportFilter,
    limit: u16,
) -> Result<ReportInventory, OperatorError> {
    if !(1..=100).contains(&limit) {
        return Err(OperatorError::InvalidInput);
    }
    let rows = sqlx::query_as::<_, ReportRow>(
        r#"
        SELECT
            report.id,
            reporter.id AS reporter_id,
            reporter.handle AS reporter_handle,
            reporter.display_name AS reporter_display_name,
            reporter.bio AS reporter_bio,
            reporter.status_message AS reporter_status_message,
            to_char(reporter.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS reporter_created_at,
            to_char(reporter.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS reporter_updated_at,
            subject.id AS subject_id,
            subject.account_id AS subject_account_id,
            subject.handle AS subject_handle,
            subject.display_name AS subject_display_name,
            subject.bio AS subject_bio,
            subject.status_message AS subject_status_message,
            to_char(subject.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS subject_created_at,
            to_char(subject.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS subject_updated_at,
            report.category,
            report.detail,
            report.status,
            to_char(report.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(report.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
            CASE WHEN report.closed_at IS NULL THEN NULL
                 ELSE to_char(report.closed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
            END AS closed_at
        FROM persona_reports AS report
        JOIN personas AS reporter ON reporter.id = report.reporter_persona_id
        JOIN personas AS subject ON subject.id = report.subject_persona_id
        WHERE ($1::text IS NULL OR report.status = $1)
        ORDER BY report.created_at DESC, report.id DESC
        LIMIT $2
        "#,
    )
    .bind(filter.query_value())
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(|_| OperatorError::Internal)?;

    Ok(ReportInventory {
        reports: rows.into_iter().map(report_from_row).collect(),
    })
}

pub async fn apply_command(
    pool: &PgPool,
    command: &OperatorCommand,
) -> Result<AuditReceipt, OperatorError> {
    command.validate()?;
    match command {
        OperatorCommand::SetAccountStatus {
            idempotency_key,
            account_id,
            status,
            actor,
            reason,
        } => set_account_status(pool, *idempotency_key, *account_id, *status, actor, reason).await,
        OperatorCommand::SetReportStatus {
            idempotency_key,
            report_id,
            status,
            actor,
            reason,
        } => set_report_status(pool, *idempotency_key, *report_id, *status, actor, reason).await,
    }
}

async fn set_account_status(
    pool: &PgPool,
    operation_id: Uuid,
    account_id: Uuid,
    requested: AccountStatus,
    actor: &str,
    reason: &str,
) -> Result<AuditReceipt, OperatorError> {
    let mut transaction = begin(pool).await?;
    let current =
        sqlx::query_scalar::<_, String>("SELECT status FROM accounts WHERE id = $1 FOR UPDATE")
            .bind(account_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| OperatorError::Internal)?
            .ok_or(OperatorError::NotFound)?;

    if let Some(replay) = account_replay(&mut transaction, account_id, operation_id).await? {
        let result = exact_replay(replay, requested.as_str(), actor, reason)?;
        transaction
            .commit()
            .await
            .map_err(|_| OperatorError::Internal)?;
        return Ok(result);
    }

    let resulting = requested.as_str();
    if current == "disabled" || current == resulting {
        return Err(OperatorError::Denied);
    }
    if !matches!(
        (current.as_str(), resulting),
        ("active", "suspended") | ("suspended", "active")
    ) {
        return Err(OperatorError::Denied);
    }

    sqlx::query("UPDATE accounts SET status = $2, updated_at = now() WHERE id = $1")
        .bind(account_id)
        .bind(resulting)
        .execute(&mut *transaction)
        .await
        .map_err(|_| OperatorError::Internal)?;
    if requested == AccountStatus::Suspended {
        sqlx::query(
            "UPDATE account_sessions SET revoked_at = now() WHERE account_id = $1 AND revoked_at IS NULL",
        )
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| OperatorError::Internal)?;
    }
    let receipt = insert_audit(
        &mut transaction,
        operation_id,
        "account",
        account_id,
        "set_account_status",
        actor,
        reason,
        &current,
        resulting,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(|_| OperatorError::Internal)?;
    Ok(receipt)
}

async fn set_report_status(
    pool: &PgPool,
    operation_id: Uuid,
    report_id: Uuid,
    requested: ReportStatus,
    actor: &str,
    reason: &str,
) -> Result<AuditReceipt, OperatorError> {
    let mut transaction = begin(pool).await?;
    let current = sqlx::query_scalar::<_, String>(
        "SELECT status FROM persona_reports WHERE id = $1 FOR UPDATE",
    )
    .bind(report_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| OperatorError::Internal)?
    .ok_or(OperatorError::NotFound)?;

    if let Some(replay) = report_replay(&mut transaction, report_id, operation_id).await? {
        let result = exact_replay(replay, requested.as_str(), actor, reason)?;
        transaction
            .commit()
            .await
            .map_err(|_| OperatorError::Internal)?;
        return Ok(result);
    }
    if current != "open" {
        return Err(OperatorError::Denied);
    }

    let resulting = requested.as_str();
    sqlx::query(
        "UPDATE persona_reports SET status = $2, updated_at = now(), closed_at = now() WHERE id = $1",
    )
    .bind(report_id)
    .bind(resulting)
    .execute(&mut *transaction)
    .await
    .map_err(|_| OperatorError::Internal)?;
    let receipt = insert_audit(
        &mut transaction,
        operation_id,
        "report",
        report_id,
        "set_report_status",
        actor,
        reason,
        &current,
        resulting,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(|_| OperatorError::Internal)?;
    Ok(receipt)
}

async fn begin(pool: &PgPool) -> Result<Transaction<'_, Postgres>, OperatorError> {
    pool.begin().await.map_err(|_| OperatorError::Internal)
}

async fn account_replay(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
    operation_id: Uuid,
) -> Result<Option<AuditRow>, OperatorError> {
    sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT id, operation_id, target_kind, target_account_id AS target_id,
               action, actor, reason, previous_state, resulting_state,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
        FROM operator_audit_events
        WHERE target_kind = 'account' AND target_account_id = $1 AND operation_id = $2
        "#,
    )
    .bind(target_id)
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| OperatorError::Internal)
}

async fn report_replay(
    transaction: &mut Transaction<'_, Postgres>,
    target_id: Uuid,
    operation_id: Uuid,
) -> Result<Option<AuditRow>, OperatorError> {
    sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT id, operation_id, target_kind, target_report_id AS target_id,
               action, actor, reason, previous_state, resulting_state,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
        FROM operator_audit_events
        WHERE target_kind = 'report' AND target_report_id = $1 AND operation_id = $2
        "#,
    )
    .bind(target_id)
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| OperatorError::Internal)
}

fn exact_replay(
    row: AuditRow,
    resulting_state: &str,
    actor: &str,
    reason: &str,
) -> Result<AuditReceipt, OperatorError> {
    if row.resulting_state != resulting_state || row.actor != actor || row.reason != reason {
        return Err(OperatorError::Conflict);
    }
    Ok(receipt_from_audit(row))
}

#[allow(clippy::too_many_arguments)]
async fn insert_audit(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
    target_kind: &str,
    target_id: Uuid,
    action: &str,
    actor: &str,
    reason: &str,
    previous_state: &str,
    resulting_state: &str,
) -> Result<AuditReceipt, OperatorError> {
    let (account_id, report_id) = if target_kind == "account" {
        (Some(target_id), None)
    } else {
        (None, Some(target_id))
    };
    let row = sqlx::query_as::<_, AuditRow>(
        r#"
        INSERT INTO operator_audit_events (
            operation_id, target_kind, target_account_id, target_report_id,
            action, actor, reason, previous_state, resulting_state, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
        RETURNING id, operation_id, target_kind,
                  COALESCE(target_account_id, target_report_id) AS target_id,
                  action, actor, reason, previous_state, resulting_state,
                  to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
        "#,
    )
    .bind(operation_id)
    .bind(target_kind)
    .bind(account_id)
    .bind(report_id)
    .bind(action)
    .bind(actor)
    .bind(reason)
    .bind(previous_state)
    .bind(resulting_state)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| OperatorError::Internal)?;
    Ok(receipt_from_audit(row))
}

fn receipt_from_audit(row: AuditRow) -> AuditReceipt {
    AuditReceipt {
        id: row.id,
        operation_id: row.operation_id,
        target_kind: row.target_kind,
        target_id: row.target_id,
        action: row.action,
        previous_state: row.previous_state,
        resulting_state: row.resulting_state,
        created_at: row.created_at,
    }
}

fn report_from_row(row: ReportRow) -> OperatorReport {
    OperatorReport {
        id: row.id,
        reporter: PublicPersona {
            id: row.reporter_id,
            handle: row.reporter_handle,
            display_name: row.reporter_display_name,
            bio: row.reporter_bio,
            status_message: row.reporter_status_message,
            created_at: row.reporter_created_at,
            updated_at: row.reporter_updated_at,
        },
        subject: PublicPersona {
            id: row.subject_id,
            handle: row.subject_handle,
            display_name: row.subject_display_name,
            bio: row.subject_bio,
            status_message: row.subject_status_message,
            created_at: row.subject_created_at,
            updated_at: row.subject_updated_at,
        },
        subject_account_id: row.subject_account_id,
        category: row.category,
        detail: row.detail,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        closed_at: row.closed_at,
    }
}

fn valid_text(value: &str, maximum: usize) -> bool {
    value == value.trim()
        && (1..=maximum).contains(&value.chars().count())
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use serde_json::to_value;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::{
        AccountStatus, OperatorCommand, OperatorError, ReportFilter, ReportStatus, apply_command,
        list_reports,
    };

    #[test]
    fn commands_reject_nil_untrimmed_control_and_oversized_values() {
        let operation_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let valid = OperatorCommand::SetAccountStatus {
            idempotency_key: operation_id,
            account_id,
            status: AccountStatus::Suspended,
            actor: "local-sysop".to_owned(),
            reason: "Private-alpha safety review".to_owned(),
        };
        assert_eq!(valid.validate(), Ok(()));
        for invalid in [
            OperatorCommand::SetAccountStatus {
                idempotency_key: Uuid::nil(),
                account_id,
                status: AccountStatus::Suspended,
                actor: "local-sysop".to_owned(),
                reason: "Private-alpha safety review".to_owned(),
            },
            OperatorCommand::SetAccountStatus {
                idempotency_key: operation_id,
                account_id,
                status: AccountStatus::Suspended,
                actor: " local-sysop".to_owned(),
                reason: "Private-alpha safety review".to_owned(),
            },
            OperatorCommand::SetAccountStatus {
                idempotency_key: operation_id,
                account_id,
                status: AccountStatus::Suspended,
                actor: "local-sysop".to_owned(),
                reason: "unsafe\nreason".to_owned(),
            },
            OperatorCommand::SetAccountStatus {
                idempotency_key: operation_id,
                account_id,
                status: AccountStatus::Suspended,
                actor: "local-sysop".to_owned(),
                reason: "x".repeat(501),
            },
        ] {
            assert_eq!(invalid.validate(), Err(OperatorError::InvalidInput));
        }
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
    async fn report_inventory_is_bounded_ordered_public_and_secret_free(pool: PgPool) {
        let (reporter_account, reporter) = seed_persona(&pool, "reporter", "Reporter").await;
        let (subject_account, subject) = seed_persona(&pool, "subject", "Subject").await;
        let old_report = seed_report(&pool, reporter, subject, "spam", "Older report").await;
        sqlx::query(
            "UPDATE persona_reports SET created_at = now() - interval '1 hour', updated_at = now() - interval '1 hour' WHERE id = $1",
        )
        .bind(old_report)
        .execute(&pool)
        .await
        .expect("older report timestamp should set");
        let newest = seed_report(&pool, reporter, subject, "cheating", "Newest report").await;

        let inventory = list_reports(&pool, ReportFilter::Open, 1)
            .await
            .expect("report inventory should load");
        assert_eq!(inventory.reports.len(), 1);
        let report = &inventory.reports[0];
        assert_eq!(report.id, newest);
        assert_eq!(report.reporter.id, reporter);
        assert_eq!(report.subject.id, subject);
        assert_eq!(report.subject_account_id, subject_account);
        assert_eq!(report.detail, "Newest report");
        let document = to_value(&inventory).expect("inventory should serialize");
        assert_eq!(document.as_object().expect("object").len(), 1);
        let encoded = document.to_string();
        for secret in [
            "password_hash",
            "token_hash",
            "account_sessions",
            "idempotency_key",
        ] {
            assert!(!encoded.contains(secret));
        }
        assert!(!encoded.contains(&reporter_account.to_string()));
        assert_eq!(
            list_reports(&pool, ReportFilter::All, 0).await,
            Err(OperatorError::InvalidInput)
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
    async fn account_actions_revoke_replay_reactivate_and_serialize(pool: PgPool) {
        let account_id = seed_account(&pool, "account_target", "active").await;
        let live_one = seed_session(&pool, account_id, 1, false).await;
        let live_two = seed_session(&pool, account_id, 2, false).await;
        let earlier = seed_session(&pool, account_id, 3, true).await;
        let earlier_timestamp: String =
            sqlx::query_scalar("SELECT revoked_at::text FROM account_sessions WHERE id = $1")
                .bind(earlier)
                .fetch_one(&pool)
                .await
                .expect("earlier revocation should read");
        let suspend = OperatorCommand::SetAccountStatus {
            idempotency_key: Uuid::new_v4(),
            account_id,
            status: AccountStatus::Suspended,
            actor: "local-sysop".to_owned(),
            reason: "Contain reported behavior".to_owned(),
        };
        let receipt = apply_command(&pool, &suspend)
            .await
            .expect("suspension should apply");
        assert_eq!(receipt.previous_state, "active");
        assert_eq!(receipt.resulting_state, "suspended");
        assert_eq!(
            apply_command(&pool, &suspend).await,
            Ok(receipt.clone()),
            "exact replay should return the original audit receipt"
        );
        let collision = OperatorCommand::SetAccountStatus {
            idempotency_key: match &suspend {
                OperatorCommand::SetAccountStatus {
                    idempotency_key, ..
                } => *idempotency_key,
                OperatorCommand::SetReportStatus { .. } => unreachable!(),
            },
            account_id,
            status: AccountStatus::Suspended,
            actor: "local-sysop".to_owned(),
            reason: "Different intent".to_owned(),
        };
        assert_eq!(
            apply_command(&pool, &collision).await,
            Err(OperatorError::Conflict)
        );
        let live_revocations: Vec<String> = sqlx::query_scalar(
            "SELECT revoked_at::text FROM account_sessions WHERE id = ANY($1) ORDER BY id",
        )
        .bind(vec![live_one, live_two])
        .fetch_all(&pool)
        .await
        .expect("live revocations should read");
        assert_eq!(live_revocations.len(), 2);
        assert_eq!(live_revocations[0], live_revocations[1]);
        let preserved: String =
            sqlx::query_scalar("SELECT revoked_at::text FROM account_sessions WHERE id = $1")
                .bind(earlier)
                .fetch_one(&pool)
                .await
                .expect("preserved revocation should read");
        assert_eq!(preserved, earlier_timestamp);

        let reactivate = OperatorCommand::SetAccountStatus {
            idempotency_key: Uuid::new_v4(),
            account_id,
            status: AccountStatus::Active,
            actor: "local-sysop".to_owned(),
            reason: "Review completed".to_owned(),
        };
        apply_command(&pool, &reactivate)
            .await
            .expect("reactivation should apply");
        let (status, still_revoked): (String, i64) = sqlx::query_as(
            "SELECT status, (SELECT count(*) FROM account_sessions WHERE account_id = $1 AND revoked_at IS NOT NULL) FROM accounts WHERE id = $1",
        )
        .bind(account_id)
        .fetch_one(&pool)
        .await
        .expect("reactivated state should read");
        assert_eq!(status, "active");
        assert_eq!(still_revoked, 3);

        let disabled = seed_account(&pool, "disabled_target", "disabled").await;
        let denied = OperatorCommand::SetAccountStatus {
            idempotency_key: Uuid::new_v4(),
            account_id: disabled,
            status: AccountStatus::Active,
            actor: "local-sysop".to_owned(),
            reason: "This must stay denied".to_owned(),
        };
        assert_eq!(
            apply_command(&pool, &denied).await,
            Err(OperatorError::Denied)
        );

        let concurrent = seed_account(&pool, "concurrent_target", "active").await;
        let first = OperatorCommand::SetAccountStatus {
            idempotency_key: Uuid::new_v4(),
            account_id: concurrent,
            status: AccountStatus::Suspended,
            actor: "sysop-a".to_owned(),
            reason: "First concurrent action".to_owned(),
        };
        let second = OperatorCommand::SetAccountStatus {
            idempotency_key: Uuid::new_v4(),
            account_id: concurrent,
            status: AccountStatus::Suspended,
            actor: "sysop-b".to_owned(),
            reason: "Second concurrent action".to_owned(),
        };
        let (first_result, second_result) =
            tokio::join!(apply_command(&pool, &first), apply_command(&pool, &second));
        assert_eq!(
            usize::from(first_result.is_ok()) + usize::from(second_result.is_ok()),
            1
        );
        assert!(matches!(
            (first_result, second_result),
            (Ok(_), Err(OperatorError::Denied)) | (Err(OperatorError::Denied), Ok(_))
        ));
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM operator_audit_events WHERE target_account_id = $1",
        )
        .bind(concurrent)
        .fetch_one(&pool)
        .await
        .expect("concurrent audit should count");
        assert_eq!(audit_count, 1);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "requires PostgreSQL; run scripts/test-database.sh"]
    async fn report_actions_are_terminal_serialized_and_append_only(pool: PgPool) {
        let (_, reporter) = seed_persona(&pool, "report_actioner", "Report Actioner").await;
        let (_, subject) = seed_persona(&pool, "report_subject", "Report Subject").await;
        let report_id = seed_report(&pool, reporter, subject, "other", "Review this report").await;
        let resolve = OperatorCommand::SetReportStatus {
            idempotency_key: Uuid::new_v4(),
            report_id,
            status: ReportStatus::Resolved,
            actor: "local-sysop".to_owned(),
            reason: "Reviewed and resolved".to_owned(),
        };
        let receipt = apply_command(&pool, &resolve)
            .await
            .expect("resolution should apply");
        assert_eq!(apply_command(&pool, &resolve).await, Ok(receipt.clone()));
        assert_eq!(
            apply_command(
                &pool,
                &OperatorCommand::SetReportStatus {
                    idempotency_key: match &resolve {
                        OperatorCommand::SetReportStatus {
                            idempotency_key, ..
                        } => *idempotency_key,
                        OperatorCommand::SetAccountStatus { .. } => unreachable!(),
                    },
                    report_id,
                    status: ReportStatus::Dismissed,
                    actor: "local-sysop".to_owned(),
                    reason: "Reviewed and resolved".to_owned(),
                }
            )
            .await,
            Err(OperatorError::Conflict)
        );
        let terminal_retry = OperatorCommand::SetReportStatus {
            idempotency_key: Uuid::new_v4(),
            report_id,
            status: ReportStatus::Dismissed,
            actor: "local-sysop".to_owned(),
            reason: "Reviewed and resolved".to_owned(),
        };
        assert_eq!(
            apply_command(&pool, &terminal_retry).await,
            Err(OperatorError::Denied)
        );
        assert!(
            sqlx::query("UPDATE operator_audit_events SET reason = 'changed' WHERE id = $1")
                .bind(receipt.id)
                .execute(&pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("DELETE FROM operator_audit_events WHERE id = $1")
                .bind(receipt.id)
                .execute(&pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("DELETE FROM persona_reports WHERE id = $1")
                .bind(report_id)
                .execute(&pool)
                .await
                .is_err()
        );

        let race_report = seed_report(&pool, reporter, subject, "spam", "Competing review").await;
        let resolved = OperatorCommand::SetReportStatus {
            idempotency_key: Uuid::new_v4(),
            report_id: race_report,
            status: ReportStatus::Resolved,
            actor: "sysop-a".to_owned(),
            reason: "Resolve competing review".to_owned(),
        };
        let dismissed = OperatorCommand::SetReportStatus {
            idempotency_key: Uuid::new_v4(),
            report_id: race_report,
            status: ReportStatus::Dismissed,
            actor: "sysop-b".to_owned(),
            reason: "Dismiss competing review".to_owned(),
        };
        let (resolved_result, dismissed_result) = tokio::join!(
            apply_command(&pool, &resolved),
            apply_command(&pool, &dismissed)
        );
        assert_eq!(
            usize::from(resolved_result.is_ok()) + usize::from(dismissed_result.is_ok()),
            1
        );
        let (status, audits): (String, i64) = sqlx::query_as(
            "SELECT status, (SELECT count(*) FROM operator_audit_events WHERE target_report_id = $1) FROM persona_reports WHERE id = $1",
        )
        .bind(race_report)
        .fetch_one(&pool)
        .await
        .expect("race outcome should read");
        assert!(status == "resolved" || status == "dismissed");
        assert_eq!(audits, 1);
    }

    async fn seed_account(pool: &PgPool, username: &str, status: &str) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO accounts (username, password_hash, status) VALUES ($1, 'test-only-hash', $2) RETURNING id",
        )
        .bind(username)
        .bind(status)
        .fetch_one(pool)
        .await
        .expect("account should seed")
    }

    async fn seed_persona(pool: &PgPool, handle: &str, display_name: &str) -> (Uuid, Uuid) {
        let account_id = seed_account(pool, &format!("{handle}_acct"), "active").await;
        let persona_id = sqlx::query_scalar(
            "INSERT INTO personas (account_id, handle, display_name) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(account_id)
        .bind(handle)
        .bind(display_name)
        .fetch_one(pool)
        .await
        .expect("persona should seed");
        (account_id, persona_id)
    }

    async fn seed_session(pool: &PgPool, account_id: Uuid, marker: u8, revoked: bool) -> Uuid {
        sqlx::query_scalar(
            r#"
            INSERT INTO account_sessions (
                account_id, token_hash, device_name, expires_at, revoked_at
            )
            VALUES ($1, $2, $3, now() + interval '30 days',
                    CASE WHEN $4 THEN now() - interval '1 hour' ELSE NULL END)
            RETURNING id
            "#,
        )
        .bind(account_id)
        .bind(vec![marker; 32])
        .bind(format!("Fixture device {marker}"))
        .bind(revoked)
        .fetch_one(pool)
        .await
        .expect("session should seed")
    }

    async fn seed_report(
        pool: &PgPool,
        reporter: Uuid,
        subject: Uuid,
        category: &str,
        detail: &str,
    ) -> Uuid {
        sqlx::query_scalar(
            r#"
            INSERT INTO persona_reports (
                reporter_persona_id, subject_persona_id, idempotency_key, category, detail
            )
            VALUES ($1, $2, gen_random_uuid(), $3, $4)
            RETURNING id
            "#,
        )
        .bind(reporter)
        .bind(subject)
        .bind(category)
        .bind(detail)
        .fetch_one(pool)
        .await
        .expect("report should seed")
    }
}
