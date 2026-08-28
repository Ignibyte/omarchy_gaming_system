//! PostgreSQL-backed production server-module registry and observation dispatcher.
//!
//! Ticket 040 intentionally admits only the compiled-in Sentinel fixture and
//! one post-commit report observation. No caller can supply executable bytes,
//! a host path, an arbitrary hook/capability, SQL, or a network destination.

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{SigningKey, VerifyingKey};
use hmac::{Hmac, KeyInit, Mac};
use omarchygs_server_module_runtime::{
    BUILTIN_MODULE_ID, BUILTIN_RELEASE_ID, Capability, FixtureKind, HOOK_FORMAT, HookKind,
    HookPayload, HostRequest, HostResponse, HostResult, MAX_FRAME_BYTES, ModuleAdmission,
    ModuleHookEvent, ModuleIntent, ModuleSubject, PRIORITY_REVIEW_LABEL, ProcessSupervisor,
    RESPONSE_FORMAT, ReviewedRelease, SignedEnvelope, canonical_json, host_request,
    reviewed_release, sha256_hex, sign_active_admission, verify_host_request,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use sqlx::{FromRow, PgPool, Postgres, Transaction, types::Json};
use thiserror::Error;
use tokio::{sync::watch, task::JoinHandle};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Stable singleton instance identity for the only reviewed production module.
pub const BUILTIN_INSTANCE_ID: Uuid = Uuid::from_u128(0x12000000000040008000000000000001);
/// Hard ceiling for durable nonterminal observation work.
pub const MAX_UNDELIVERED_EVENTS: i64 = 1024;
/// Maximum delivery attempts before durable dead letter.
pub const MAX_DELIVERY_ATTEMPTS: i32 = 3;
const CLAIM_LEASE_SECONDS: i32 = 5;
const CIRCUIT_FAILURE_THRESHOLD: i32 = 3;
const DISPATCH_POLL: Duration = Duration::from_millis(50);
const MODULE_REGISTRY_ADVISORY_LOCK: i64 = 0x4f47_534d_4f44_3031;
const MAX_STATE_ENTRIES: usize = 32;
const MAX_STATE_BYTES: usize = 4096;
const MAX_STATE_VALUE_BYTES: usize = 512;

/// Exact secrets needed only when the reviewed production module is enabled.
#[derive(Clone)]
pub struct ModuleConfig {
    /// Server-specific admission signer seed.
    pub admission_signing_seed: [u8; 32],
    /// Purpose-specific pairwise persona derivation secret.
    pub pairwise_secret: [u8; 32],
}

/// Stable module-control errors safe for local adapters and transport mapping.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ModuleError {
    /// Configuration is absent, partial, or malformed.
    #[error("invalid module configuration")]
    InvalidConfig,
    /// A bounded command, state value, or contract is malformed.
    #[error("invalid module input")]
    InvalidInput,
    /// Expected revision, idempotency body, or immutable inventory conflicts.
    #[error("module state conflict")]
    Conflict,
    /// Operator/lifecycle policy denies the requested action.
    #[error("module action denied")]
    Denied,
    /// Host readiness or execution is temporarily unavailable.
    #[error("module host unavailable")]
    Unavailable,
    /// PostgreSQL or an internal invariant failed.
    #[error("module internal failure")]
    Internal,
}

impl ModuleError {
    /// Stable non-secret error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidConfig => "server_module_invalid_config",
            Self::InvalidInput => "server_module_invalid_input",
            Self::Conflict => "server_module_conflict",
            Self::Denied => "server_module_denied",
            Self::Unavailable => "server_module_unavailable",
            Self::Internal => "server_module_internal",
        }
    }
}

/// Optional same-transaction observation producer held by application state.
#[derive(Clone)]
pub struct ModuleEmitter {
    instance_id: Uuid,
    pairwise_secret: Arc<[u8; 32]>,
}

/// One running database dispatcher plus its cloneable event emitter.
pub struct ServerModuleService {
    emitter: ModuleEmitter,
    shutdown: watch::Sender<bool>,
    worker: JoinHandle<()>,
}

/// Cloneable shutdown edge used to stop fresh module work before HTTP drain.
#[derive(Clone)]
pub struct ModuleShutdownTrigger {
    shutdown: watch::Sender<bool>,
}

impl ModuleShutdownTrigger {
    /// Notify the dispatcher synchronously so it cannot claim work during drain.
    pub fn request(&self) {
        let _ = self.shutdown.send(true);
    }
}

impl ServerModuleService {
    /// Verify, register, probe, admit, and start the compiled production module.
    pub async fn production(pool: PgPool, config: ModuleConfig) -> Result<Self, ModuleError> {
        let supervisor = ProcessSupervisor::packaged_sibling().map_err(|runtime_error| {
            warn!(error = %runtime_error, "packaged module host unavailable");
            ModuleError::InvalidConfig
        })?;
        Self::start_with_executor(pool, config, Arc::new(ProcessExecutor { supervisor })).await
    }

    /// Start with an injected executor used by the PostgreSQL fault corpus.
    pub(crate) async fn start_with_executor(
        pool: PgPool,
        config: ModuleConfig,
        executor: Arc<dyn ModuleExecutor>,
    ) -> Result<Self, ModuleError> {
        let core_signer = SigningKey::from_bytes(&config.admission_signing_seed);
        let enabled = register_and_enable(&pool, &core_signer, Arc::clone(&executor)).await?;
        let emitter = ModuleEmitter {
            instance_id: enabled.instance_id,
            pairwise_secret: Arc::new(config.pairwise_secret),
        };
        let (shutdown, receiver) = watch::channel(false);
        let worker_pool = pool.clone();
        let worker = tokio::spawn(dispatch_loop(
            worker_pool,
            executor,
            core_signer.verifying_key(),
            Arc::clone(&emitter.pairwise_secret),
            receiver,
        ));
        Ok(Self {
            emitter,
            shutdown,
            worker,
        })
    }

    /// Clone the restricted same-transaction emitter for application state.
    #[must_use]
    pub fn emitter(&self) -> ModuleEmitter {
        self.emitter.clone()
    }

    /// Clone the restricted trigger used by the HTTP graceful-shutdown edge.
    #[must_use]
    pub fn shutdown_trigger(&self) -> ModuleShutdownTrigger {
        ModuleShutdownTrigger {
            shutdown: self.shutdown.clone(),
        }
    }

    /// Stop claiming fresh work and await the bounded worker shutdown.
    pub async fn shutdown(self) {
        self.shutdown_trigger().request();
        let mut worker = self.worker;
        if tokio::time::timeout(Duration::from_secs(32), &mut worker)
            .await
            .is_err()
        {
            worker.abort();
            let _ = worker.await;
            warn!("server module dispatcher did not stop before shutdown deadline");
        }
    }
}

pub(crate) trait ModuleExecutor: Send + Sync {
    fn execute(
        &self,
        request: HostRequest,
        core_key: VerifyingKey,
    ) -> Result<HostResponse, ModuleError>;
}

struct ProcessExecutor {
    supervisor: ProcessSupervisor,
}

impl ModuleExecutor for ProcessExecutor {
    fn execute(
        &self,
        request: HostRequest,
        core_key: VerifyingKey,
    ) -> Result<HostResponse, ModuleError> {
        self.supervisor
            .execute(&request, &core_key)
            .map(|report| report.response)
            .map_err(|runtime_error| {
                warn!(error = %runtime_error, "contained module host invocation failed");
                ModuleError::Unavailable
            })
    }
}

#[derive(FromRow)]
struct InstanceRow {
    instance_id: Uuid,
    lifecycle: String,
    lifecycle_revision: i64,
    current_admission_id: Option<Uuid>,
    current_admission_revision: Option<i64>,
    config: Json<Value>,
    config_revision: i64,
    state_revision: i64,
    activation_allowed: bool,
    restored_pending_review: bool,
}

struct EnabledInstance {
    instance_id: Uuid,
}

async fn register_and_enable(
    pool: &PgPool,
    core_signer: &SigningKey,
    executor: Arc<dyn ModuleExecutor>,
) -> Result<EnabledInstance, ModuleError> {
    let reviewed = reviewed_release().map_err(contract_failure)?;
    let server_id = register_release_and_instance(pool, &reviewed).await?;
    let instance = load_instance(pool).await?;
    if !instance.activation_allowed
        || instance.restored_pending_review
        || matches!(instance.lifecycle.as_str(), "suspended" | "retired")
    {
        return Err(ModuleError::Denied);
    }

    if instance.lifecycle == "active" {
        let admission = load_current_admission(pool, &instance).await?;
        let request = readiness_request(&reviewed, admission, &instance, server_id)?;
        let response =
            execute_without_transaction(executor, request.clone(), core_signer.verifying_key())
                .await?;
        verify_readiness_response(&request, &response)?;
        return Ok(EnabledInstance {
            instance_id: instance.instance_id,
        });
    }

    let next_revision = u64::try_from(instance.lifecycle_revision)
        .ok()
        .and_then(|revision| revision.checked_add(1))
        .ok_or(ModuleError::Internal)?;
    let admission_id = Uuid::new_v4();
    let (admission, signed_admission) = sign_active_admission(
        &reviewed,
        server_id,
        admission_id,
        next_revision,
        u64::try_from(instance.config_revision).map_err(|_| ModuleError::Internal)?,
        u64::try_from(instance.state_revision).map_err(|_| ModuleError::Internal)?,
        core_signer,
    )
    .map_err(contract_failure)?;
    let request = readiness_request(&reviewed, signed_admission.clone(), &instance, server_id)?;
    let response =
        execute_without_transaction(executor, request.clone(), core_signer.verifying_key()).await?;
    verify_readiness_response(&request, &response)?;
    finalize_enable(pool, &instance, &admission, &signed_admission).await?;
    Ok(EnabledInstance {
        instance_id: instance.instance_id,
    })
}

async fn register_release_and_instance(
    pool: &PgPool,
    reviewed: &ReviewedRelease,
) -> Result<Uuid, ModuleError> {
    let release_bytes = canonical_json(&reviewed.release).map_err(contract_failure)?;
    let provenance_bytes = canonical_json(&reviewed.provenance).map_err(contract_failure)?;
    let mut transaction = pool.begin().await.map_err(database_error)?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(MODULE_REGISTRY_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    let server_id: Uuid =
        sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton FOR SHARE")
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?;
    let inserted = sqlx::query(
        r#"
        INSERT INTO server_module_releases (
            release_id, module_id, publisher_id, version, release_format,
            signed_release, release_sha256, signed_provenance, provenance_sha256,
            provenance_class, review_id, component_sha256, wit_package, wit_world,
            wit_major, wit_sha256, requested_capabilities, subscribed_hooks,
            frame_bytes, memory_bytes, fuel, execution_ms, config_schema, state_schema
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
            $15, $16, $17, $18, $19, $20, $21, $22, $23, $24
        )
        ON CONFLICT (release_id) DO NOTHING
        "#,
    )
    .bind(reviewed.manifest.release_id)
    .bind(&reviewed.manifest.module_id)
    .bind(&reviewed.manifest.publisher_id)
    .bind(&reviewed.manifest.version)
    .bind(&reviewed.manifest.format)
    .bind(&release_bytes)
    .bind(
        reviewed
            .release
            .payload_sha256()
            .map_err(contract_failure)?,
    )
    .bind(&provenance_bytes)
    .bind(
        reviewed
            .provenance
            .payload_sha256()
            .map_err(contract_failure)?,
    )
    .bind(&reviewed.provenance_statement.class)
    .bind(reviewed.provenance_statement.review_id)
    .bind(&reviewed.manifest.component_sha256)
    .bind(&reviewed.manifest.wit.package)
    .bind(&reviewed.manifest.wit.world)
    .bind(i32::from(reviewed.manifest.wit.major))
    .bind(&reviewed.manifest.wit.sha256)
    .bind(vec![Capability::ModerationAddLabel.to_string()])
    .bind(vec![HookKind::PersonaReported.to_string()])
    .bind(i32::try_from(reviewed.manifest.budgets.frame_bytes).map_err(|_| ModuleError::Internal)?)
    .bind(i32::try_from(reviewed.manifest.budgets.memory_bytes).map_err(|_| ModuleError::Internal)?)
    .bind(i64::try_from(reviewed.manifest.budgets.fuel).map_err(|_| ModuleError::Internal)?)
    .bind(i32::try_from(reviewed.manifest.budgets.execution_ms).map_err(|_| ModuleError::Internal)?)
    .bind(&reviewed.manifest.config_schema)
    .bind(&reviewed.manifest.state_schema)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?
    .rows_affected();
    if inserted == 0 {
        let existing = sqlx::query_as::<_, (Vec<u8>, String, Vec<u8>, String, String, String)>(
            r#"
            SELECT signed_release, release_sha256, signed_provenance,
                   provenance_sha256, component_sha256, wit_sha256
            FROM server_module_releases
            WHERE release_id = $1
            FOR SHARE
            "#,
        )
        .bind(reviewed.manifest.release_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(database_error)?;
        if existing.0 != release_bytes
            || existing.1
                != reviewed
                    .release
                    .payload_sha256()
                    .map_err(contract_failure)?
            || existing.2 != provenance_bytes
            || existing.3
                != reviewed
                    .provenance
                    .payload_sha256()
                    .map_err(contract_failure)?
            || existing.4 != reviewed.manifest.component_sha256
            || existing.5 != reviewed.manifest.wit.sha256
        {
            return Err(ModuleError::Conflict);
        }
    }

    let instance_inserted = sqlx::query(
        r#"
        INSERT INTO server_module_instances (
            instance_id, module_id, release_id, lifecycle, lifecycle_revision
        ) VALUES ($1, $2, $3, 'disabled', 1)
        ON CONFLICT (instance_id) DO NOTHING
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(BUILTIN_MODULE_ID)
    .bind(BUILTIN_RELEASE_ID)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?
    .rows_affected();
    if instance_inserted == 1 {
        sqlx::query(
            r#"
            INSERT INTO server_module_state_namespaces (
                instance_id, state_schema, revision, entries, byte_size
            ) VALUES ($1, 'ignibyte.sentinel.state/v1', 0, '{}'::JSONB, 2)
            "#,
        )
        .bind(BUILTIN_INSTANCE_ID)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        sqlx::query(
            r#"
            INSERT INTO server_module_lifecycle_audit (
                operation_id, instance_id, action, expected_revision,
                previous_state, resulting_state, resulting_revision, actor, reason
            ) VALUES ($1, $2, 'register', 0, 'absent', 'disabled', 1,
                      'omarchygs-core', 'Register exact reviewed first-party module release')
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(BUILTIN_INSTANCE_ID)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }
    transaction.commit().await.map_err(database_error)?;
    Ok(server_id)
}

async fn load_instance(pool: &PgPool) -> Result<InstanceRow, ModuleError> {
    sqlx::query_as::<_, InstanceRow>(
        r#"
        SELECT instance_id, lifecycle, lifecycle_revision, current_admission_id,
               current_admission_revision, config, config_revision, state_revision,
               activation_allowed, restored_pending_review
        FROM server_module_instances
        WHERE instance_id = $1 AND release_id = $2 AND module_id = $3
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(BUILTIN_RELEASE_ID)
    .bind(BUILTIN_MODULE_ID)
    .fetch_one(pool)
    .await
    .map_err(database_error)
}

async fn load_current_admission(
    pool: &PgPool,
    instance: &InstanceRow,
) -> Result<SignedEnvelope, ModuleError> {
    let admission_id = instance.current_admission_id.ok_or(ModuleError::Conflict)?;
    let admission_revision = instance
        .current_admission_revision
        .ok_or(ModuleError::Conflict)?;
    let bytes: Vec<u8> = sqlx::query_scalar(
        r#"
        SELECT signed_admission
        FROM server_module_admissions
        WHERE admission_id = $1 AND lifecycle_revision = $2
        "#,
    )
    .bind(admission_id)
    .bind(admission_revision)
    .fetch_one(pool)
    .await
    .map_err(database_error)?;
    decode_canonical(&bytes)
}

async fn finalize_enable(
    pool: &PgPool,
    instance: &InstanceRow,
    admission: &ModuleAdmission,
    signed_admission: &SignedEnvelope,
) -> Result<(), ModuleError> {
    let signed_bytes = canonical_json(signed_admission).map_err(contract_failure)?;
    let mut transaction = pool.begin().await.map_err(database_error)?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(MODULE_REGISTRY_ADVISORY_LOCK)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    let current: (String, i64, i64, i64, i64, bool, bool) = sqlx::query_as(
        r#"
        SELECT lifecycle, lifecycle_revision, config_revision, state_revision,
               n.revision AS namespace_revision, activation_allowed,
               restored_pending_review
        FROM server_module_instances i
        JOIN server_module_state_namespaces n ON n.instance_id = i.instance_id
        WHERE i.instance_id = $1
        FOR UPDATE OF i, n
        "#,
    )
    .bind(instance.instance_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(database_error)?;
    if current.1 != instance.lifecycle_revision
        || current.2 != instance.config_revision
        || current.3 != instance.state_revision
        || current.4 != instance.state_revision
        || u64::try_from(current.2).map_err(|_| ModuleError::Internal)? != admission.config_revision
        || u64::try_from(current.3).map_err(|_| ModuleError::Internal)? != admission.state_revision
        || !current.5
        || current.6
    {
        return Err(ModuleError::Conflict);
    }
    if matches!(current.0.as_str(), "suspended" | "retired") {
        return Err(ModuleError::Denied);
    }
    sqlx::query(
        r#"
        INSERT INTO server_module_admissions (
            admission_id, lifecycle_revision, release_id, server_id,
            admission_format, signed_admission, admission_sha256, lifecycle,
            granted_capabilities, subscribed_hooks, config_revision,
            state_schema, state_revision
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, $9, $10, $11, $12)
        "#,
    )
    .bind(admission.admission_id)
    .bind(i64::try_from(admission.lifecycle_revision).map_err(|_| ModuleError::Internal)?)
    .bind(admission.release_id)
    .bind(admission.server_id)
    .bind(&admission.format)
    .bind(&signed_bytes)
    .bind(sha256_hex(&signed_bytes))
    .bind(vec![Capability::ModerationAddLabel.to_string()])
    .bind(vec![HookKind::PersonaReported.to_string()])
    .bind(i64::try_from(admission.config_revision).map_err(|_| ModuleError::Internal)?)
    .bind(&admission.state_schema)
    .bind(i64::try_from(admission.state_revision).map_err(|_| ModuleError::Internal)?)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    let next_revision =
        i64::try_from(admission.lifecycle_revision).map_err(|_| ModuleError::Internal)?;
    sqlx::query(
        r#"
        UPDATE server_module_instances
        SET current_admission_id = $2,
            current_admission_revision = $3,
            lifecycle = 'active',
            lifecycle_revision = $3,
            consecutive_failures = 0,
            updated_at = clock_timestamp()
        WHERE instance_id = $1
        "#,
    )
    .bind(instance.instance_id)
    .bind(admission.admission_id)
    .bind(next_revision)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    sqlx::query(
        r#"
        INSERT INTO server_module_lifecycle_audit (
            operation_id, instance_id, action, expected_revision, previous_state,
            resulting_state, resulting_revision, actor, reason
        ) VALUES ($1, $2, 'enable', $3, $4, 'active', $5,
                  'omarchygs-core', 'Host readiness and exact admission verified')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(instance.instance_id)
    .bind(instance.lifecycle_revision)
    .bind(&instance.lifecycle)
    .bind(next_revision)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    transaction.commit().await.map_err(database_error)
}

fn readiness_request(
    reviewed: &ReviewedRelease,
    signed_admission: SignedEnvelope,
    instance: &InstanceRow,
    server_id: Uuid,
) -> Result<HostRequest, ModuleError> {
    let admission_payload = decode_signed_payload::<ModuleAdmission>(&signed_admission)?;
    Ok(host_request(
        reviewed,
        signed_admission,
        ModuleHookEvent {
            format: HOOK_FORMAT.into(),
            event_id: Uuid::new_v4(),
            attempt: 1,
            server_id,
            module_id: BUILTIN_MODULE_ID.into(),
            release_id: BUILTIN_RELEASE_ID,
            admission_id: admission_payload.admission_id,
            admission_revision: admission_payload.lifecycle_revision,
            hook: HookKind::PersonaReported,
            causal_revision: 0,
            deadline_ms: admission_payload.budgets.execution_ms,
            subject: ModuleSubject::Pairwise("startup-readiness-probe".into()),
            config: json_object_to_string_map(&instance.config.0)?,
            config_revision: u64::try_from(instance.config_revision)
                .map_err(|_| ModuleError::Internal)?,
            state: BTreeMap::new(),
            state_revision: u64::try_from(instance.state_revision)
                .map_err(|_| ModuleError::Internal)?,
            payload: HookPayload::PersonaReported {
                report_id: Uuid::new_v4(),
                category: "other".into(),
            },
        },
    ))
}

fn verify_readiness_response(
    request: &HostRequest,
    response: &HostResponse,
) -> Result<(), ModuleError> {
    if response.format != RESPONSE_FORMAT
        || response.event_id != request.event.event_id
        || response.release_id != request.event.release_id
        || response.admission_id != request.event.admission_id
        || response.admission_revision != request.event.admission_revision
        || !matches!(response.outcome, HostResult::Proposed { .. })
    {
        return Err(ModuleError::Unavailable);
    }
    Ok(())
}

async fn execute_without_transaction(
    executor: Arc<dyn ModuleExecutor>,
    request: HostRequest,
    core_key: VerifyingKey,
) -> Result<HostResponse, ModuleError> {
    tokio::task::spawn_blocking(move || executor.execute(request, core_key))
        .await
        .map_err(|join_error| {
            error!(error = %join_error, "module executor task failed");
            ModuleError::Unavailable
        })?
}

impl ModuleEmitter {
    /// Build the restricted emitter for a configured but inactive module.
    #[must_use]
    pub fn configured(config: &ModuleConfig) -> Self {
        Self {
            instance_id: BUILTIN_INSTANCE_ID,
            pairwise_secret: Arc::new(config.pairwise_secret),
        }
    }

    /// Append one report observation in the caller's authoritative transaction.
    pub async fn append_persona_reported(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        report_id: Uuid,
        subject_persona_id: Uuid,
        category: &str,
    ) -> Result<Option<Uuid>, ModuleError> {
        if report_id.is_nil()
            || subject_persona_id.is_nil()
            || !matches!(category, "harassment" | "spam" | "cheating" | "other")
        {
            return Err(ModuleError::InvalidInput);
        }
        let row = sqlx::query_as::<_, EmitContextRow>(
            r#"
            SELECT i.lifecycle, i.release_id, i.current_admission_id,
                   i.current_admission_revision, i.config, i.config_revision,
                   n.entries AS state, n.revision AS state_revision,
                   a.server_id
            FROM server_module_instances AS i
            JOIN server_module_state_namespaces AS n ON n.instance_id = i.instance_id
            LEFT JOIN server_module_admissions AS a
              ON a.admission_id = i.current_admission_id
             AND a.lifecycle_revision = i.current_admission_revision
            WHERE i.instance_id = $1
            FOR UPDATE OF i
            "#,
        )
        .bind(self.instance_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?
        .ok_or(ModuleError::Conflict)?;
        if row.lifecycle != "active" {
            record_observation_gap(transaction, self.instance_id, "module_inactive").await?;
            return Ok(None);
        }
        let admission_id = row.current_admission_id.ok_or(ModuleError::Conflict)?;
        let admission_revision = row
            .current_admission_revision
            .ok_or(ModuleError::Conflict)?;
        let server_id = row.server_id.ok_or(ModuleError::Conflict)?;
        let outstanding: i64 = sqlx::query_scalar(
            r#"
            SELECT count(*)
            FROM server_module_outbox
            WHERE instance_id = $1
              AND status <> 'delivered'
            "#,
        )
        .bind(self.instance_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(database_error)?;
        if outstanding >= MAX_UNDELIVERED_EVENTS {
            record_observation_gap(transaction, self.instance_id, "queue_saturated").await?;
            return Ok(None);
        }
        let pairwise_subject = derive_pairwise_subject(
            self.pairwise_secret.as_ref(),
            BUILTIN_MODULE_ID,
            subject_persona_id,
        )?;
        let payload = json!({
            "kind": "persona_reported",
            "report_id": report_id,
            "category": category,
        });
        let payload_sha256 = sha256_hex(&canonical_json(&payload).map_err(contract_failure)?);
        let event_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO server_module_outbox (
                event_id, instance_id, release_id, admission_id,
                admission_revision, hook, partition_subject, subject_persona_id,
                target_report_id, causal_revision, payload, payload_sha256,
                config_snapshot, config_revision, state_snapshot, state_revision
            ) VALUES (
                $1, $2, $3, $4, $5, 'persona_reported', $6, $7,
                $8, 0, $9, $10, $11, $12, $13, $14
            )
            "#,
        )
        .bind(event_id)
        .bind(self.instance_id)
        .bind(row.release_id)
        .bind(admission_id)
        .bind(admission_revision)
        .bind(pairwise_subject)
        .bind(subject_persona_id)
        .bind(report_id)
        .bind(Json(payload))
        .bind(payload_sha256)
        .bind(row.config)
        .bind(row.config_revision)
        .bind(row.state)
        .bind(row.state_revision)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
        let _ = server_id;
        Ok(Some(event_id))
    }
}

async fn record_observation_gap(
    transaction: &mut Transaction<'_, Postgres>,
    instance_id: Uuid,
    reason: &'static str,
) -> Result<(), ModuleError> {
    let updated = sqlx::query(
        r#"
        UPDATE server_module_instances
        SET observation_gap_count = CASE
                WHEN observation_gap_count < 9223372036854775807
                    THEN observation_gap_count + 1
                ELSE observation_gap_count
            END,
            last_observation_gap_reason = $2,
            last_observation_gap_at = clock_timestamp(),
            updated_at = clock_timestamp()
        WHERE instance_id = $1
        "#,
    )
    .bind(instance_id)
    .bind(reason)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?
    .rows_affected();
    if updated != 1 {
        return Err(ModuleError::Conflict);
    }
    Ok(())
}

#[derive(FromRow)]
struct EmitContextRow {
    lifecycle: String,
    release_id: Uuid,
    current_admission_id: Option<Uuid>,
    current_admission_revision: Option<i64>,
    config: Json<Value>,
    config_revision: i64,
    state: Json<Value>,
    state_revision: i64,
    server_id: Option<Uuid>,
}

fn derive_pairwise_subject(
    secret: &[u8; 32],
    module_id: &str,
    persona_id: Uuid,
) -> Result<String, ModuleError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| ModuleError::InvalidConfig)?;
    mac.update(b"OmarchyGS module pairwise subject\0");
    mac.update(module_id.as_bytes());
    mac.update(&[0]);
    mac.update(persona_id.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

#[derive(FromRow)]
struct ClaimedEvent {
    event_id: Uuid,
    instance_id: Uuid,
    release_id: Uuid,
    admission_id: Uuid,
    admission_revision: i64,
    hook: String,
    partition_subject: String,
    subject_persona_id: Uuid,
    target_report_id: Uuid,
    causal_revision: i64,
    payload: Json<Value>,
    payload_sha256: String,
    config_snapshot: Json<Value>,
    config_revision: i64,
    state_snapshot: Json<Value>,
    state_revision: i64,
    attempt_count: i32,
    lease_id: Uuid,
    server_id: Uuid,
    signed_admission: Vec<u8>,
}

async fn dispatch_loop(
    pool: PgPool,
    executor: Arc<dyn ModuleExecutor>,
    core_key: VerifyingKey,
    pairwise_secret: Arc<[u8; 32]>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        if *shutdown.borrow() {
            break;
        }
        match dispatch_once(
            &pool,
            Arc::clone(&executor),
            core_key,
            pairwise_secret.as_ref(),
        )
        .await
        {
            Ok(true) => continue,
            Ok(false) => {}
            Err(module_error) => {
                error!(
                    code = module_error.code(),
                    "server module dispatch iteration failed"
                );
            }
        }
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            () = tokio::time::sleep(DISPATCH_POLL) => {}
        }
    }
}

async fn dispatch_once(
    pool: &PgPool,
    executor: Arc<dyn ModuleExecutor>,
    core_key: VerifyingKey,
    pairwise_secret: &[u8; 32],
) -> Result<bool, ModuleError> {
    let Some(claimed) = claim_next_event(pool).await? else {
        return Ok(false);
    };
    let request = match request_from_claim(&claimed) {
        Ok(request) => request,
        Err(module_error) => {
            record_delivery_failure(pool, &claimed, module_error.code()).await?;
            return Ok(true);
        }
    };
    let request_receipt = match encode_request_receipt(&request) {
        Ok(receipt) => receipt,
        Err(module_error) => {
            record_delivery_failure(pool, &claimed, module_error.code()).await?;
            return Ok(true);
        }
    };
    if reconcile_existing_delivery(pool, &claimed, &request_receipt.sha256).await? {
        return Ok(true);
    }
    let response = match execute_without_transaction(executor, request.clone(), core_key).await {
        Ok(response) => response,
        Err(module_error) => {
            record_delivery_failure(pool, &claimed, module_error.code()).await?;
            return Ok(true);
        }
    };
    if let HostResult::Rejected { code } = &response.outcome {
        let stable = match code.as_str() {
            "request_rejected" => "host_request_rejected",
            "runtime_limit_rejected" => "host_runtime_limit",
            "module_instantiation_failed" => "host_instantiation_failed",
            "module_execution_failed" => "host_execution_failed",
            "intent_outside_policy" => "host_intent_outside_policy",
            "intent_not_granted" => "host_intent_not_granted",
            _ => "host_response_rejected",
        };
        record_delivery_failure(pool, &claimed, stable).await?;
        return Ok(true);
    }
    if let Err(module_error) = apply_host_response(
        pool,
        &claimed,
        &request,
        &request_receipt,
        &response,
        &core_key,
        pairwise_secret,
    )
    .await
    {
        record_delivery_failure(pool, &claimed, module_error.code()).await?;
    }
    Ok(true)
}

struct EncodedRequestReceipt {
    body: Vec<u8>,
    sha256: String,
}

fn encode_request_receipt(request: &HostRequest) -> Result<EncodedRequestReceipt, ModuleError> {
    let mut stable_request = request.clone();
    stable_request.event.attempt = 0;
    let body = canonical_json(&stable_request).map_err(contract_failure)?;
    if body.is_empty() || body.len() > MAX_FRAME_BYTES {
        return Err(ModuleError::InvalidInput);
    }
    Ok(EncodedRequestReceipt {
        sha256: sha256_hex(&body),
        body,
    })
}

async fn claim_next_event(pool: &PgPool) -> Result<Option<ClaimedEvent>, ModuleError> {
    let mut transaction = pool.begin().await.map_err(database_error)?;
    sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = 'retry',
            lease_id = NULL,
            lease_expires_at = NULL,
            next_attempt_at = clock_timestamp(),
            last_error_code = 'lease_expired',
            updated_at = clock_timestamp()
        WHERE status = 'in_flight' AND lease_expires_at <= clock_timestamp()
        "#,
    )
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    let sequence = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT o.sequence
        FROM server_module_outbox AS o
        JOIN server_module_instances AS i ON i.instance_id = o.instance_id
        WHERE i.lifecycle = 'active'
          AND o.status IN ('pending', 'retry')
          AND o.next_attempt_at <= clock_timestamp()
          AND NOT EXISTS (
              SELECT 1
              FROM server_module_outbox AS earlier
              WHERE earlier.release_id = o.release_id
                AND earlier.hook = o.hook
                AND earlier.partition_subject = o.partition_subject
                AND earlier.sequence < o.sequence
                AND earlier.status NOT IN ('delivered', 'dead_letter')
          )
        ORDER BY o.sequence
        FOR UPDATE OF o SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?;
    let Some(sequence) = sequence else {
        transaction.commit().await.map_err(database_error)?;
        return Ok(None);
    };
    let lease_id = Uuid::new_v4();
    sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = 'in_flight',
            attempt_count = attempt_count + 1,
            lease_id = $2,
            lease_expires_at = clock_timestamp() + make_interval(secs => $3),
            last_error_code = NULL,
            updated_at = clock_timestamp()
        WHERE sequence = $1
        "#,
    )
    .bind(sequence)
    .bind(lease_id)
    .bind(CLAIM_LEASE_SECONDS)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    let claimed = sqlx::query_as::<_, ClaimedEvent>(
        r#"
        SELECT o.event_id, o.instance_id, o.release_id,
               o.admission_id, o.admission_revision, o.hook,
               o.partition_subject, o.subject_persona_id, o.target_report_id,
               o.causal_revision, o.payload, o.payload_sha256,
               o.config_snapshot, o.config_revision, o.state_snapshot,
               o.state_revision, o.attempt_count, o.lease_id,
               a.server_id, a.signed_admission
        FROM server_module_outbox AS o
        JOIN server_module_admissions AS a
          ON a.admission_id = o.admission_id
         AND a.lifecycle_revision = o.admission_revision
        WHERE o.sequence = $1 AND o.lease_id = $2
        "#,
    )
    .bind(sequence)
    .bind(lease_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(database_error)?;
    transaction.commit().await.map_err(database_error)?;
    Ok(Some(claimed))
}

fn request_from_claim(claimed: &ClaimedEvent) -> Result<HostRequest, ModuleError> {
    if claimed.hook != HookKind::PersonaReported.to_string()
        || claimed.release_id != BUILTIN_RELEASE_ID
        || claimed.attempt_count <= 0
        || claimed.attempt_count > MAX_DELIVERY_ATTEMPTS
    {
        return Err(ModuleError::Conflict);
    }
    let reviewed = reviewed_release().map_err(contract_failure)?;
    let signed_admission: SignedEnvelope = decode_canonical(&claimed.signed_admission)?;
    let (report_id, category) = parse_report_payload(&claimed.payload.0)?;
    if report_id != claimed.target_report_id
        || sha256_hex(&canonical_json(&claimed.payload.0).map_err(contract_failure)?)
            != claimed.payload_sha256
    {
        return Err(ModuleError::Conflict);
    }
    Ok(host_request(
        &reviewed,
        signed_admission,
        ModuleHookEvent {
            format: HOOK_FORMAT.into(),
            event_id: claimed.event_id,
            attempt: u16::try_from(claimed.attempt_count).map_err(|_| ModuleError::Internal)?,
            server_id: claimed.server_id,
            module_id: BUILTIN_MODULE_ID.into(),
            release_id: claimed.release_id,
            admission_id: claimed.admission_id,
            admission_revision: u64::try_from(claimed.admission_revision)
                .map_err(|_| ModuleError::Internal)?,
            hook: HookKind::PersonaReported,
            causal_revision: u64::try_from(claimed.causal_revision)
                .map_err(|_| ModuleError::Internal)?,
            deadline_ms: omarchygs_server_module_runtime::MAX_EXECUTION_MS,
            subject: ModuleSubject::Pairwise(claimed.partition_subject.clone()),
            config: json_object_to_string_map(&claimed.config_snapshot.0)?,
            config_revision: u64::try_from(claimed.config_revision)
                .map_err(|_| ModuleError::Internal)?,
            state: json_object_to_string_map(&claimed.state_snapshot.0)?,
            state_revision: u64::try_from(claimed.state_revision)
                .map_err(|_| ModuleError::Internal)?,
            payload: HookPayload::PersonaReported {
                report_id,
                category,
            },
        },
    ))
}

async fn reconcile_existing_delivery(
    pool: &PgPool,
    claimed: &ClaimedEvent,
    request_sha: &str,
) -> Result<bool, ModuleError> {
    let existing = sqlx::query_as::<_, (String,)>(
        "SELECT request_sha256 FROM server_module_delivery_receipts WHERE event_id = $1",
    )
    .bind(claimed.event_id)
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;
    let Some((stored_sha,)) = existing else {
        return Ok(false);
    };
    if stored_sha != request_sha {
        record_delivery_failure(pool, claimed, "receipt_request_conflict").await?;
        return Ok(true);
    }
    let updated = sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = 'delivered', lease_id = NULL, lease_expires_at = NULL,
            delivered_at = COALESCE(delivered_at, clock_timestamp()),
            updated_at = clock_timestamp()
        WHERE event_id = $1 AND status = 'in_flight' AND lease_id = $2
        "#,
    )
    .bind(claimed.event_id)
    .bind(claimed.lease_id)
    .execute(pool)
    .await
    .map_err(database_error)?
    .rows_affected();
    if updated != 1 {
        return Err(ModuleError::Conflict);
    }
    Ok(true)
}

#[derive(FromRow)]
struct ApplyRoot {
    lifecycle: String,
    lifecycle_revision: i64,
    current_admission_id: Option<Uuid>,
    current_admission_revision: Option<i64>,
    outbox_status: String,
    outbox_lease_id: Option<Uuid>,
    report_subject_persona_id: Uuid,
    report_status: String,
}

async fn apply_host_response(
    pool: &PgPool,
    claimed: &ClaimedEvent,
    request: &HostRequest,
    request_receipt: &EncodedRequestReceipt,
    response: &HostResponse,
    core_key: &VerifyingKey,
    pairwise_secret: &[u8; 32],
) -> Result<(), ModuleError> {
    verify_host_request(request, core_key, FixtureKind::Valid).map_err(contract_failure)?;
    if response.format != RESPONSE_FORMAT
        || response.event_id != claimed.event_id
        || response.release_id != claimed.release_id
        || response.admission_id != claimed.admission_id
        || response.admission_revision
            != u64::try_from(claimed.admission_revision).map_err(|_| ModuleError::Internal)?
    {
        record_delivery_failure(pool, claimed, "response_context_mismatch").await?;
        return Ok(());
    }
    let response_body = canonical_json(response).map_err(contract_failure)?;
    let response_sha = sha256_hex(&response_body);
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let root = sqlx::query_as::<_, ApplyRoot>(
        r#"
        SELECT i.lifecycle, i.lifecycle_revision, i.current_admission_id,
               i.current_admission_revision, o.status AS outbox_status,
               o.lease_id AS outbox_lease_id,
               r.subject_persona_id AS report_subject_persona_id,
               r.status AS report_status
        FROM server_module_instances AS i
        JOIN server_module_outbox AS o ON o.instance_id = i.instance_id
        JOIN persona_reports AS r ON r.id = o.target_report_id
        WHERE i.instance_id = $1 AND o.event_id = $2
        FOR UPDATE OF i, o, r
        "#,
    )
    .bind(claimed.instance_id)
    .bind(claimed.event_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(database_error)?;
    if root.outbox_status != "in_flight"
        || root.outbox_lease_id != Some(claimed.lease_id)
        || root.lifecycle != "active"
        || root.current_admission_id != Some(claimed.admission_id)
        || root.current_admission_revision != Some(claimed.admission_revision)
    {
        transaction.rollback().await.map_err(database_error)?;
        record_delivery_failure(pool, claimed, "current_admission_changed").await?;
        return Ok(());
    }
    if root.report_subject_persona_id != claimed.subject_persona_id
        || derive_pairwise_subject(
            pairwise_secret,
            BUILTIN_MODULE_ID,
            root.report_subject_persona_id,
        )? != claimed.partition_subject
    {
        return Err(ModuleError::Conflict);
    }
    let receipt_exists: Option<String> = sqlx::query_scalar(
        "SELECT request_sha256 FROM server_module_delivery_receipts WHERE event_id = $1",
    )
    .bind(claimed.event_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?;
    if let Some(stored_sha) = receipt_exists {
        if stored_sha != request_receipt.sha256.as_str() {
            return Err(ModuleError::Conflict);
        }
        mark_delivered(&mut transaction, claimed, root.lifecycle_revision).await?;
        transaction.commit().await.map_err(database_error)?;
        return Ok(());
    }

    let outcome_code = match &response.outcome {
        HostResult::Noop => "noop",
        HostResult::Proposed { intent } => {
            apply_intent(
                &mut transaction,
                claimed,
                &request_receipt.sha256,
                intent,
                &root.report_status,
            )
            .await?
        }
        HostResult::Rejected { .. } => return Err(ModuleError::Conflict),
    };
    sqlx::query(
        r#"
        INSERT INTO server_module_delivery_receipts (
            event_id, release_id, request_sha256, response_sha256,
            response_body, outcome_code, attempt_count, request_body,
            target_report_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(claimed.event_id)
    .bind(claimed.release_id)
    .bind(&request_receipt.sha256)
    .bind(response_sha)
    .bind(response_body)
    .bind(outcome_code)
    .bind(claimed.attempt_count)
    .bind(&request_receipt.body)
    .bind(claimed.target_report_id)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    mark_delivered(&mut transaction, claimed, root.lifecycle_revision).await?;
    transaction.commit().await.map_err(database_error)?;
    prune_delivered(pool, claimed.instance_id).await?;
    info!(
        event_id = %claimed.event_id,
        release_id = %claimed.release_id,
        outcome = outcome_code,
        "server module observation delivered"
    );
    Ok(())
}

async fn apply_intent(
    transaction: &mut Transaction<'_, Postgres>,
    claimed: &ClaimedEvent,
    request_sha: &str,
    intent: &ModuleIntent,
    report_status: &str,
) -> Result<&'static str, ModuleError> {
    if intent.capability() != Capability::ModerationAddLabel {
        return Err(ModuleError::Denied);
    }
    let intent_body = canonical_json(intent).map_err(contract_failure)?;
    let intent_sha = sha256_hex(&intent_body);
    let ModuleIntent::ModerationAddLabel {
        expected_revision,
        label,
    } = intent;
    let current_revision: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(max(revision), 0)
        FROM server_module_report_labels
        WHERE instance_id = $1 AND report_id = $2
        "#,
    )
    .bind(claimed.instance_id)
    .bind(claimed.target_report_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(database_error)?;
    let expected_revision_i64 =
        i64::try_from(*expected_revision).map_err(|_| ModuleError::InvalidInput)?;
    let (outcome, resulting_revision, committed, stored_label) = if report_status != "open" {
        ("report_not_open", current_revision, false, Some(*label))
    } else if *label != PRIORITY_REVIEW_LABEL {
        ("label_denied", current_revision, false, Some(*label))
    } else if expected_revision_i64 != current_revision {
        ("revision_conflict", current_revision, false, Some(*label))
    } else {
        let resulting_revision = current_revision
            .checked_add(1)
            .ok_or(ModuleError::Internal)?;
        sqlx::query(
            r#"
            INSERT INTO server_module_report_labels (
                instance_id, report_id, label, revision, source_event_id
            ) VALUES ($1, $2, 'priority_review', $3, $4)
            "#,
        )
        .bind(claimed.instance_id)
        .bind(claimed.target_report_id)
        .bind(resulting_revision)
        .bind(claimed.event_id)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
        (
            "moderation_label_added",
            resulting_revision,
            true,
            Some(*label),
        )
    };
    sqlx::query(
        r#"
        INSERT INTO server_module_intent_receipts (
            release_id, event_id, ordinal, request_sha256, intent_sha256,
            outcome_code, target_report_id, expected_revision,
            resulting_revision, label, committed
        ) VALUES ($1, $2, 0, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(claimed.release_id)
    .bind(claimed.event_id)
    .bind(request_sha)
    .bind(intent_sha)
    .bind(outcome)
    .bind(claimed.target_report_id)
    .bind(expected_revision_i64)
    .bind(resulting_revision)
    .bind(stored_label.map(|value| i64::try_from(value).unwrap_or(i64::MAX)))
    .bind(committed)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(outcome)
}

async fn mark_delivered(
    transaction: &mut Transaction<'_, Postgres>,
    claimed: &ClaimedEvent,
    lifecycle_revision: i64,
) -> Result<(), ModuleError> {
    let updated = sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = 'delivered', lease_id = NULL, lease_expires_at = NULL,
            delivered_at = clock_timestamp(), last_error_code = NULL,
            updated_at = clock_timestamp()
        WHERE event_id = $1 AND status = 'in_flight' AND lease_id = $2
        "#,
    )
    .bind(claimed.event_id)
    .bind(claimed.lease_id)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?
    .rows_affected();
    if updated != 1 {
        return Err(ModuleError::Conflict);
    }
    sqlx::query(
        r#"
        UPDATE server_module_instances
        SET consecutive_failures = 0, updated_at = clock_timestamp()
        WHERE instance_id = $1 AND lifecycle_revision = $2
        "#,
    )
    .bind(claimed.instance_id)
    .bind(lifecycle_revision)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn record_delivery_failure(
    pool: &PgPool,
    claimed: &ClaimedEvent,
    error_code: &str,
) -> Result<(), ModuleError> {
    if error_code.len() < 3
        || error_code.len() > 64
        || !error_code
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(ModuleError::Internal);
    }
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let instance: (String, i64, i32) = sqlx::query_as(
        r#"
        SELECT lifecycle, lifecycle_revision, consecutive_failures
        FROM server_module_instances
        WHERE instance_id = $1
        FOR UPDATE
        "#,
    )
    .bind(claimed.instance_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(database_error)?;
    let current: Option<(String, Option<Uuid>, i32)> = sqlx::query_as(
        r#"
        SELECT status, lease_id, attempt_count
        FROM server_module_outbox
        WHERE event_id = $1
        FOR UPDATE
        "#,
    )
    .bind(claimed.event_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?;
    let Some((status, lease_id, attempts)) = current else {
        transaction.commit().await.map_err(database_error)?;
        return Ok(());
    };
    if status != "in_flight" || lease_id != Some(claimed.lease_id) {
        transaction.commit().await.map_err(database_error)?;
        return Ok(());
    }
    let terminal = attempts >= MAX_DELIVERY_ATTEMPTS;
    let backoff_seconds = 1_i32 << u32::try_from(attempts.saturating_sub(1).min(2)).unwrap_or(0);
    sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = CASE WHEN $3 THEN 'dead_letter' ELSE 'retry' END,
            lease_id = NULL,
            lease_expires_at = NULL,
            next_attempt_at = CASE
                WHEN $3 THEN next_attempt_at
                ELSE clock_timestamp() + make_interval(secs => $4)
            END,
            last_error_code = $2,
            dead_lettered_at = CASE WHEN $3 THEN clock_timestamp() ELSE NULL END,
            updated_at = clock_timestamp()
        WHERE event_id = $1
        "#,
    )
    .bind(claimed.event_id)
    .bind(error_code)
    .bind(terminal)
    .bind(backoff_seconds)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    let failures = instance.2.saturating_add(1);
    if failures >= CIRCUIT_FAILURE_THRESHOLD && instance.0 == "active" {
        let next_revision = instance.1.checked_add(1).ok_or(ModuleError::Internal)?;
        sqlx::query(
            r#"
            UPDATE server_module_instances
            SET lifecycle = 'degraded', lifecycle_revision = $2,
                consecutive_failures = $3, activation_allowed = FALSE,
                updated_at = clock_timestamp()
            WHERE instance_id = $1
            "#,
        )
        .bind(claimed.instance_id)
        .bind(next_revision)
        .bind(failures)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        sqlx::query(
            r#"
            INSERT INTO server_module_lifecycle_audit (
                operation_id, instance_id, action, expected_revision,
                previous_state, resulting_state, resulting_revision, actor, reason
            ) VALUES ($1, $2, 'degrade', $3, 'active', 'degraded', $4,
                      'omarchygs-core', 'Circuit breaker threshold reached')
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(claimed.instance_id)
        .bind(instance.1)
        .bind(next_revision)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    } else {
        sqlx::query(
            r#"
            UPDATE server_module_instances
            SET consecutive_failures = $2, updated_at = clock_timestamp()
            WHERE instance_id = $1
            "#,
        )
        .bind(claimed.instance_id)
        .bind(failures)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }
    transaction.commit().await.map_err(database_error)?;
    warn!(
        event_id = %claimed.event_id,
        attempt = attempts,
        terminal,
        error_code,
        "server module delivery failed"
    );
    Ok(())
}

pub(crate) async fn prune_delivered(pool: &PgPool, instance_id: Uuid) -> Result<(), ModuleError> {
    sqlx::query(
        r#"
        DELETE FROM server_module_outbox
        WHERE sequence IN (
            SELECT sequence
            FROM server_module_outbox
            WHERE instance_id = $1 AND status = 'delivered'
            ORDER BY sequence DESC
            OFFSET 4096
        )
        "#,
    )
    .bind(instance_id)
    .execute(pool)
    .await
    .map_err(database_error)?;
    Ok(())
}

/// Operator-selectable lifecycle actions that never install or grant code.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleLifecycleAction {
    /// Stop subscription and prevent automatic activation.
    Disable,
    /// Emergency stop with a distinct player/operator-visible state.
    Suspend,
    /// Clear restore/circuit policy into disabled; startup must probe to activate.
    Recover,
    /// Terminally retain evidence and prohibit future activation.
    Retire,
}

impl ModuleLifecycleAction {
    const fn audit_name(self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::Suspend => "suspend",
            Self::Recover => "recover",
            Self::Retire => "retire",
        }
    }

    const fn resulting_state(self) -> &'static str {
        match self {
            Self::Disable | Self::Recover => "disabled",
            Self::Suspend => "suspended",
            Self::Retire => "retired",
        }
    }
}

/// Expected-revision database-local lifecycle command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleLifecycleCommand {
    /// Exact command schema.
    pub format: String,
    /// Idempotency identity.
    pub operation_id: Uuid,
    /// Only currently installable module identity.
    pub module_id: String,
    /// Required current lifecycle revision.
    pub expected_revision: i64,
    /// Bounded action.
    pub action: ModuleLifecycleAction,
    /// Local operator identity for audit.
    pub actor: String,
    /// Bounded human reason.
    pub reason: String,
}

/// Immutable lifecycle receipt safe for local JSON output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleLifecycleReceipt {
    /// Idempotency identity.
    pub operation_id: Uuid,
    /// Stable module identity.
    pub module_id: String,
    /// Prior lifecycle state.
    pub previous_state: String,
    /// Resulting lifecycle state.
    pub resulting_state: String,
    /// Monotonic lifecycle revision.
    pub resulting_revision: i64,
}

/// Apply disable, suspension, recovery preparation, or retirement locally.
pub async fn apply_lifecycle_command(
    pool: &PgPool,
    command: &ModuleLifecycleCommand,
) -> Result<ModuleLifecycleReceipt, ModuleError> {
    validate_lifecycle_command(command)?;
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let existing = load_lifecycle_replay(&mut transaction, command.operation_id).await?;
    if let Some(existing) = existing {
        if existing.expected_revision != command.expected_revision
            || existing.action != command.action.audit_name()
            || existing.actor != command.actor
            || existing.reason != command.reason
        {
            return Err(ModuleError::Conflict);
        }
        transaction.commit().await.map_err(database_error)?;
        return Ok(existing.receipt());
    }
    let current: (String, i64, bool) = sqlx::query_as(
        r#"
        SELECT lifecycle, lifecycle_revision, restored_pending_review
        FROM server_module_instances
        WHERE instance_id = $1 AND module_id = $2
        FOR UPDATE
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(&command.module_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?
    .ok_or(ModuleError::Denied)?;
    if current.1 != command.expected_revision {
        return Err(ModuleError::Conflict);
    }
    validate_lifecycle_transition(command.action, &current.0, current.2)?;
    let next_revision = current.1.checked_add(1).ok_or(ModuleError::Internal)?;
    let activation_allowed = command.action == ModuleLifecycleAction::Recover;
    let restored_pending_review = if command.action == ModuleLifecycleAction::Recover {
        false
    } else {
        current.2
    };
    sqlx::query(
        r#"
        UPDATE server_module_instances
        SET lifecycle = $2,
            lifecycle_revision = $3,
            activation_allowed = $4,
            restored_pending_review = $5,
            consecutive_failures = CASE WHEN $6 THEN 0 ELSE consecutive_failures END,
            updated_at = clock_timestamp()
        WHERE instance_id = $1
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(command.action.resulting_state())
    .bind(next_revision)
    .bind(activation_allowed)
    .bind(restored_pending_review)
    .bind(command.action == ModuleLifecycleAction::Recover)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    release_in_flight_for_pause(&mut transaction, BUILTIN_INSTANCE_ID).await?;
    sqlx::query(
        r#"
        INSERT INTO server_module_lifecycle_audit (
            operation_id, instance_id, action, expected_revision, previous_state,
            resulting_state, resulting_revision, actor, reason
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(command.operation_id)
    .bind(BUILTIN_INSTANCE_ID)
    .bind(command.action.audit_name())
    .bind(command.expected_revision)
    .bind(&current.0)
    .bind(command.action.resulting_state())
    .bind(next_revision)
    .bind(&command.actor)
    .bind(&command.reason)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    transaction.commit().await.map_err(database_error)?;
    Ok(ModuleLifecycleReceipt {
        operation_id: command.operation_id,
        module_id: command.module_id.clone(),
        previous_state: current.0,
        resulting_state: command.action.resulting_state().into(),
        resulting_revision: next_revision,
    })
}

#[derive(FromRow)]
struct LifecycleReplayRow {
    operation_id: Uuid,
    action: String,
    expected_revision: i64,
    previous_state: String,
    resulting_state: String,
    resulting_revision: i64,
    actor: String,
    reason: String,
}

impl LifecycleReplayRow {
    fn receipt(&self) -> ModuleLifecycleReceipt {
        ModuleLifecycleReceipt {
            operation_id: self.operation_id,
            module_id: BUILTIN_MODULE_ID.into(),
            previous_state: self.previous_state.clone(),
            resulting_state: self.resulting_state.clone(),
            resulting_revision: self.resulting_revision,
        }
    }
}

async fn load_lifecycle_replay(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
) -> Result<Option<LifecycleReplayRow>, ModuleError> {
    sqlx::query_as::<_, LifecycleReplayRow>(
        r#"
        SELECT operation_id, action, expected_revision, previous_state,
               resulting_state, resulting_revision, actor, reason
        FROM server_module_lifecycle_audit
        WHERE instance_id = $1 AND operation_id = $2
        FOR UPDATE
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)
}

fn validate_lifecycle_command(command: &ModuleLifecycleCommand) -> Result<(), ModuleError> {
    if command.format != "omarchygs.server-module-lifecycle-command/v1"
        || command.operation_id.is_nil()
        || command.module_id != BUILTIN_MODULE_ID
        || command.expected_revision <= 0
    {
        return Err(ModuleError::InvalidInput);
    }
    validate_actor_reason(&command.actor, &command.reason)
}

fn validate_lifecycle_transition(
    action: ModuleLifecycleAction,
    current: &str,
    restored_pending_review: bool,
) -> Result<(), ModuleError> {
    let valid = match action {
        ModuleLifecycleAction::Disable => matches!(current, "active" | "degraded"),
        ModuleLifecycleAction::Suspend => !matches!(current, "retired"),
        ModuleLifecycleAction::Recover => {
            restored_pending_review || matches!(current, "disabled" | "degraded" | "suspended")
        }
        ModuleLifecycleAction::Retire => matches!(current, "disabled" | "suspended"),
    };
    if valid {
        Ok(())
    } else {
        Err(ModuleError::Denied)
    }
}

async fn release_in_flight_for_pause(
    transaction: &mut Transaction<'_, Postgres>,
    instance_id: Uuid,
) -> Result<(), ModuleError> {
    sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = 'retry', lease_id = NULL, lease_expires_at = NULL,
            next_attempt_at = clock_timestamp(), last_error_code = 'lifecycle_paused',
            updated_at = clock_timestamp()
        WHERE instance_id = $1 AND status = 'in_flight'
        "#,
    )
    .bind(instance_id)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

/// Post-restore reconciliation command. Every restored module remains disabled.
pub async fn prepare_restored_modules(
    pool: &PgPool,
    operation_id: Uuid,
    actor: &str,
    reason: &str,
) -> Result<Vec<ModuleLifecycleReceipt>, ModuleError> {
    if operation_id.is_nil() {
        return Err(ModuleError::InvalidInput);
    }
    validate_actor_reason(actor, reason)?;
    let existing = sqlx::query_as::<_, LifecycleReplayRow>(
        r#"
        SELECT operation_id, action, expected_revision, previous_state,
               resulting_state, resulting_revision, actor, reason
        FROM server_module_lifecycle_audit
        WHERE operation_id = $1 AND action = 'restore'
        ORDER BY instance_id
        "#,
    )
    .bind(operation_id)
    .fetch_all(pool)
    .await
    .map_err(database_error)?;
    if !existing.is_empty() {
        if existing
            .iter()
            .any(|row| row.actor != actor || row.reason != reason)
        {
            return Err(ModuleError::Conflict);
        }
        return Ok(existing.iter().map(LifecycleReplayRow::receipt).collect());
    }
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let instances = sqlx::query_as::<_, (Uuid, String, i64)>(
        r#"
        SELECT instance_id, lifecycle, lifecycle_revision
        FROM server_module_instances
        ORDER BY instance_id
        FOR UPDATE
        "#,
    )
    .fetch_all(&mut *transaction)
    .await
    .map_err(database_error)?;
    let mut receipts = Vec::with_capacity(instances.len());
    for (instance_id, previous, revision) in instances {
        let next = revision.checked_add(1).ok_or(ModuleError::Internal)?;
        sqlx::query(
            r#"
            UPDATE server_module_instances
            SET lifecycle = 'disabled', lifecycle_revision = $2,
                activation_allowed = FALSE, restored_pending_review = TRUE,
                consecutive_failures = 0, updated_at = clock_timestamp()
            WHERE instance_id = $1
            "#,
        )
        .bind(instance_id)
        .bind(next)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        release_in_flight_for_pause(&mut transaction, instance_id).await?;
        sqlx::query(
            r#"
            INSERT INTO server_module_lifecycle_audit (
                operation_id, instance_id, action, expected_revision,
                previous_state, resulting_state, resulting_revision, actor, reason
            ) VALUES ($1, $2, 'restore', $3, $4, 'disabled', $5, $6, $7)
            "#,
        )
        .bind(operation_id)
        .bind(instance_id)
        .bind(revision)
        .bind(&previous)
        .bind(next)
        .bind(actor)
        .bind(reason)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        receipts.push(ModuleLifecycleReceipt {
            operation_id,
            module_id: BUILTIN_MODULE_ID.into(),
            previous_state: previous,
            resulting_state: "disabled".into(),
            resulting_revision: next,
        });
    }
    transaction.commit().await.map_err(database_error)?;
    Ok(receipts)
}

/// One bounded typed state mutation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModuleStateOperation {
    /// Insert or replace one string entry.
    Set { key: String, value: String },
    /// Remove one string entry.
    Remove { key: String },
}

/// CAS state/config operation receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleDataReceipt {
    /// Operation identity.
    pub operation_id: Uuid,
    /// Stable module identity.
    pub module_id: String,
    /// Data operation kind.
    pub action: String,
    /// Monotonic resulting revision.
    pub resulting_revision: i64,
    /// Retained rollback snapshot when applicable.
    pub snapshot_id: Option<Uuid>,
}

#[derive(Clone, Copy)]
struct DataMutationContext<'a> {
    operation_id: Uuid,
    expected_revision: i64,
    actor: &'a str,
    reason: &'a str,
}

struct DataAuditEntry<'a> {
    context: DataMutationContext<'a>,
    action: &'a str,
    command_sha: &'a str,
    resulting_revision: i64,
    snapshot_id: Option<Uuid>,
}

/// Compare-and-set a bounded module state namespace while delivery is disabled.
pub async fn update_state(
    pool: &PgPool,
    operation_id: Uuid,
    expected_revision: i64,
    operations: &[ModuleStateOperation],
    actor: &str,
    reason: &str,
) -> Result<ModuleDataReceipt, ModuleError> {
    mutate_state(
        pool,
        DataMutationContext {
            operation_id,
            expected_revision,
            actor,
            reason,
        },
        operations,
        "state_update",
        false,
    )
    .await
}

/// Apply an explicit atomic state migration with a retained pre-migration snapshot.
pub async fn migrate_state(
    pool: &PgPool,
    operation_id: Uuid,
    expected_revision: i64,
    operations: &[ModuleStateOperation],
    actor: &str,
    reason: &str,
) -> Result<ModuleDataReceipt, ModuleError> {
    mutate_state(
        pool,
        DataMutationContext {
            operation_id,
            expected_revision,
            actor,
            reason,
        },
        operations,
        "state_migrate",
        true,
    )
    .await
}

async fn mutate_state(
    pool: &PgPool,
    context: DataMutationContext<'_>,
    operations: &[ModuleStateOperation],
    action: &'static str,
    snapshot: bool,
) -> Result<ModuleDataReceipt, ModuleError> {
    if context.operation_id.is_nil()
        || context.expected_revision < 0
        || operations.is_empty()
        || operations.len() > MAX_STATE_ENTRIES
    {
        return Err(ModuleError::InvalidInput);
    }
    validate_actor_reason(context.actor, context.reason)?;
    let command_sha = data_command_sha(
        action,
        context.operation_id,
        context.expected_revision,
        &json!({
            "operations": operations,
            "actor": context.actor,
            "reason": context.reason,
        }),
    )?;
    if let Some(receipt) =
        load_data_replay(pool, context.operation_id, action, &command_sha).await?
    {
        return Ok(receipt);
    }
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let (lifecycle, revision, entries, byte_size): (String, i64, Json<Value>, i32) =
        sqlx::query_as(
            r#"
            SELECT i.lifecycle, n.revision, n.entries, n.byte_size
            FROM server_module_instances AS i
            JOIN server_module_state_namespaces AS n ON n.instance_id = i.instance_id
            WHERE i.instance_id = $1
            FOR UPDATE OF i, n
            "#,
        )
        .bind(BUILTIN_INSTANCE_ID)
        .fetch_one(&mut *transaction)
        .await
        .map_err(database_error)?;
    ensure_data_mutable(&mut transaction, lifecycle.as_str()).await?;
    if revision != context.expected_revision {
        return Err(ModuleError::Conflict);
    }
    let mut state = json_object_to_string_map(&entries.0)?;
    apply_state_operations(&mut state, operations)?;
    let next_value = string_map_json(&state)?;
    let next_revision = revision.checked_add(1).ok_or(ModuleError::Internal)?;
    let snapshot_id = if snapshot {
        let snapshot_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO server_module_state_snapshots (
                snapshot_id, instance_id, source_schema, source_revision,
                entries, byte_size, reason
            ) VALUES ($1, $2, 'ignibyte.sentinel.state/v1', $3, $4, $5, $6)
            "#,
        )
        .bind(snapshot_id)
        .bind(BUILTIN_INSTANCE_ID)
        .bind(revision)
        .bind(entries)
        .bind(byte_size)
        .bind(context.reason)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        Some(snapshot_id)
    } else {
        None
    };
    update_state_rows(&mut transaction, next_revision, &next_value).await?;
    insert_data_audit(
        &mut transaction,
        DataAuditEntry {
            context: DataMutationContext {
                expected_revision: revision,
                ..context
            },
            action,
            command_sha: &command_sha,
            resulting_revision: next_revision,
            snapshot_id,
        },
    )
    .await?;
    transaction.commit().await.map_err(database_error)?;
    Ok(ModuleDataReceipt {
        operation_id: context.operation_id,
        module_id: BUILTIN_MODULE_ID.into(),
        action: action.into(),
        resulting_revision: next_revision,
        snapshot_id,
    })
}

/// Restore one retained snapshot as a new monotonic state revision.
pub async fn rollback_state(
    pool: &PgPool,
    operation_id: Uuid,
    snapshot_id: Uuid,
    expected_revision: i64,
    actor: &str,
    reason: &str,
) -> Result<ModuleDataReceipt, ModuleError> {
    if operation_id.is_nil() || snapshot_id.is_nil() || expected_revision < 0 {
        return Err(ModuleError::InvalidInput);
    }
    validate_actor_reason(actor, reason)?;
    let command_sha = data_command_sha(
        "state_rollback",
        operation_id,
        expected_revision,
        &json!({
            "snapshot_id": snapshot_id,
            "actor": actor,
            "reason": reason,
        }),
    )?;
    if let Some(receipt) =
        load_data_replay(pool, operation_id, "state_rollback", &command_sha).await?
    {
        return Ok(receipt);
    }
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let (lifecycle, revision): (String, i64) = sqlx::query_as(
        r#"
        SELECT i.lifecycle, n.revision
        FROM server_module_instances AS i
        JOIN server_module_state_namespaces AS n ON n.instance_id = i.instance_id
        WHERE i.instance_id = $1
        FOR UPDATE OF i, n
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(&mut *transaction)
    .await
    .map_err(database_error)?;
    ensure_data_mutable(&mut transaction, lifecycle.as_str()).await?;
    if revision != expected_revision {
        return Err(ModuleError::Conflict);
    }
    let entries: Json<Value> = sqlx::query_scalar(
        r#"
        SELECT entries
        FROM server_module_state_snapshots
        WHERE snapshot_id = $1 AND instance_id = $2
        FOR SHARE
        "#,
    )
    .bind(snapshot_id)
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?
    .ok_or(ModuleError::Denied)?;
    let restored = json_object_to_string_map(&entries.0)?;
    let restored_value = string_map_json(&restored)?;
    let next_revision = revision.checked_add(1).ok_or(ModuleError::Internal)?;
    update_state_rows(&mut transaction, next_revision, &restored_value).await?;
    insert_data_audit(
        &mut transaction,
        DataAuditEntry {
            context: DataMutationContext {
                operation_id,
                expected_revision: revision,
                actor,
                reason,
            },
            action: "state_rollback",
            command_sha: &command_sha,
            resulting_revision: next_revision,
            snapshot_id: Some(snapshot_id),
        },
    )
    .await?;
    transaction.commit().await.map_err(database_error)?;
    Ok(ModuleDataReceipt {
        operation_id,
        module_id: BUILTIN_MODULE_ID.into(),
        action: "state_rollback".into(),
        resulting_revision: next_revision,
        snapshot_id: Some(snapshot_id),
    })
}

/// Compare-and-set the bounded configuration object while delivery is disabled.
pub async fn update_configuration(
    pool: &PgPool,
    operation_id: Uuid,
    expected_revision: i64,
    configuration: &BTreeMap<String, String>,
    actor: &str,
    reason: &str,
) -> Result<ModuleDataReceipt, ModuleError> {
    if operation_id.is_nil() || expected_revision <= 0 {
        return Err(ModuleError::InvalidInput);
    }
    validate_actor_reason(actor, reason)?;
    validate_string_map(configuration)?;
    let command_sha = data_command_sha(
        "configure",
        operation_id,
        expected_revision,
        &json!({
            "configuration": configuration,
            "actor": actor,
            "reason": reason,
        }),
    )?;
    if let Some(receipt) = load_data_replay(pool, operation_id, "configure", &command_sha).await? {
        return Ok(receipt);
    }
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let (lifecycle, revision): (String, i64) = sqlx::query_as(
        r#"
        SELECT lifecycle, config_revision
        FROM server_module_instances
        WHERE instance_id = $1
        FOR UPDATE
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(&mut *transaction)
    .await
    .map_err(database_error)?;
    ensure_data_mutable(&mut transaction, lifecycle.as_str()).await?;
    if revision != expected_revision {
        return Err(ModuleError::Conflict);
    }
    let next_revision = revision.checked_add(1).ok_or(ModuleError::Internal)?;
    sqlx::query(
        r#"
        UPDATE server_module_instances
        SET config = $2, config_revision = $3, updated_at = clock_timestamp()
        WHERE instance_id = $1
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(Json(string_map_json(configuration)?))
    .bind(next_revision)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?;
    insert_data_audit(
        &mut transaction,
        DataAuditEntry {
            context: DataMutationContext {
                operation_id,
                expected_revision: revision,
                actor,
                reason,
            },
            action: "configure",
            command_sha: &command_sha,
            resulting_revision: next_revision,
            snapshot_id: None,
        },
    )
    .await?;
    transaction.commit().await.map_err(database_error)?;
    Ok(ModuleDataReceipt {
        operation_id,
        module_id: BUILTIN_MODULE_ID.into(),
        action: "configure".into(),
        resulting_revision: next_revision,
        snapshot_id: None,
    })
}

async fn ensure_data_mutable(
    transaction: &mut Transaction<'_, Postgres>,
    lifecycle: &str,
) -> Result<(), ModuleError> {
    if !matches!(lifecycle, "disabled" | "suspended") {
        return Err(ModuleError::Denied);
    }
    let outstanding: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM server_module_outbox
        WHERE instance_id = $1 AND status <> 'delivered'
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .fetch_one(&mut **transaction)
    .await
    .map_err(database_error)?;
    if outstanding != 0 {
        return Err(ModuleError::Denied);
    }
    Ok(())
}

fn apply_state_operations(
    state: &mut BTreeMap<String, String>,
    operations: &[ModuleStateOperation],
) -> Result<(), ModuleError> {
    for operation in operations {
        match operation {
            ModuleStateOperation::Set { key, value } => {
                validate_state_key(key)?;
                validate_state_value(value)?;
                state.insert(key.clone(), value.clone());
            }
            ModuleStateOperation::Remove { key } => {
                validate_state_key(key)?;
                state.remove(key);
            }
        }
    }
    validate_string_map(state)
}

fn validate_string_map(values: &BTreeMap<String, String>) -> Result<(), ModuleError> {
    if values.len() > MAX_STATE_ENTRIES {
        return Err(ModuleError::InvalidInput);
    }
    for (key, value) in values {
        validate_state_key(key)?;
        validate_state_value(value)?;
    }
    let encoded = canonical_json(values).map_err(contract_failure)?;
    if encoded.len() > MAX_STATE_BYTES.saturating_sub(MAX_STATE_ENTRIES * 2) {
        return Err(ModuleError::InvalidInput);
    }
    Ok(())
}

fn validate_state_key(value: &str) -> Result<(), ModuleError> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(ModuleError::InvalidInput);
    }
    Ok(())
}

fn validate_state_value(value: &str) -> Result<(), ModuleError> {
    if value.len() > MAX_STATE_VALUE_BYTES || value.chars().any(char::is_control) {
        return Err(ModuleError::InvalidInput);
    }
    Ok(())
}

fn string_map_json(values: &BTreeMap<String, String>) -> Result<Value, ModuleError> {
    validate_string_map(values)?;
    serde_json::to_value(values).map_err(|_| ModuleError::Internal)
}

async fn update_state_rows(
    transaction: &mut Transaction<'_, Postgres>,
    next_revision: i64,
    next_value: &Value,
) -> Result<(), ModuleError> {
    sqlx::query(
        r#"
        UPDATE server_module_state_namespaces
        SET revision = $2, entries = $3,
            byte_size = octet_length(($3::JSONB)::TEXT),
            updated_at = clock_timestamp()
        WHERE instance_id = $1
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(next_revision)
    .bind(Json(next_value.clone()))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    sqlx::query(
        r#"
        UPDATE server_module_instances
        SET state_revision = $2, updated_at = clock_timestamp()
        WHERE instance_id = $1
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(next_revision)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn insert_data_audit(
    transaction: &mut Transaction<'_, Postgres>,
    entry: DataAuditEntry<'_>,
) -> Result<(), ModuleError> {
    sqlx::query(
        r#"
        INSERT INTO server_module_data_audit (
            operation_id, instance_id, action, command_sha256,
            expected_revision, resulting_revision, snapshot_id, actor, reason
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(entry.context.operation_id)
    .bind(BUILTIN_INSTANCE_ID)
    .bind(entry.action)
    .bind(entry.command_sha)
    .bind(entry.context.expected_revision)
    .bind(entry.resulting_revision)
    .bind(entry.snapshot_id)
    .bind(entry.context.actor)
    .bind(entry.context.reason)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

fn data_command_sha(
    action: &str,
    operation_id: Uuid,
    expected_revision: i64,
    body: &Value,
) -> Result<String, ModuleError> {
    Ok(sha256_hex(
        &canonical_json(&json!({
            "format": "omarchygs.server-module-data-command/v1",
            "operation_id": operation_id,
            "action": action,
            "expected_revision": expected_revision,
            "body": body,
        }))
        .map_err(contract_failure)?,
    ))
}

async fn load_data_replay(
    pool: &PgPool,
    operation_id: Uuid,
    action: &str,
    command_sha: &str,
) -> Result<Option<ModuleDataReceipt>, ModuleError> {
    let existing = sqlx::query_as::<_, (String, String, i64, Option<Uuid>)>(
        r#"
        SELECT action, command_sha256, resulting_revision, snapshot_id
        FROM server_module_data_audit
        WHERE instance_id = $1 AND operation_id = $2
        "#,
    )
    .bind(BUILTIN_INSTANCE_ID)
    .bind(operation_id)
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;
    match existing {
        Some((stored_action, stored_sha, revision, snapshot_id))
            if stored_action == action && stored_sha == command_sha =>
        {
            Ok(Some(ModuleDataReceipt {
                operation_id,
                module_id: BUILTIN_MODULE_ID.into(),
                action: stored_action,
                resulting_revision: revision,
                snapshot_id,
            }))
        }
        Some(_) => Err(ModuleError::Conflict),
        None => Ok(None),
    }
}

/// Safe module inventory entry; no signed bodies, secrets, event payloads, or state values.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInventoryEntry {
    /// Stable module identity.
    pub module_id: String,
    /// Exact immutable release identity.
    pub release_id: Uuid,
    /// Exact component digest.
    pub component_sha256: String,
    /// Current lifecycle.
    pub lifecycle: String,
    /// Current lifecycle revision.
    pub lifecycle_revision: i64,
    /// Current configuration revision.
    pub config_revision: i64,
    /// Current state revision.
    pub state_revision: i64,
    /// Durable nonterminal queue rows, including dead letters.
    pub outstanding_events: i64,
    /// Durable dead-letter rows.
    pub dead_letter_events: i64,
    /// Core observations not queued because the optional module was inactive or saturated.
    pub observation_gap_count: i64,
    /// Stable reason for the most recent observation gap.
    pub last_observation_gap_reason: Option<String>,
    /// UTC timestamp for the most recent observation gap.
    pub last_observation_gap_at: Option<String>,
    /// Immutable delivery receipts.
    pub delivery_receipts: i64,
    /// Upgrade-era delivery receipts that predate retained request evidence.
    pub legacy_delivery_receipts: i64,
    /// Immutable intent receipts.
    pub intent_receipts: i64,
    /// Restore review gate.
    pub restored_pending_review: bool,
}

/// Bounded module inventory response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInventory {
    /// Exact inventory schema.
    pub format: String,
    /// Current reviewed modules.
    pub modules: Vec<ModuleInventoryEntry>,
}

/// List non-secret module status and durable queue/receipt counts.
pub async fn list_module_inventory(pool: &PgPool) -> Result<ModuleInventory, ModuleError> {
    let rows = sqlx::query_as::<_, ModuleInventoryRow>(
        r#"
        SELECT i.module_id, i.release_id, r.component_sha256, i.lifecycle,
               i.lifecycle_revision, i.config_revision, i.state_revision,
               i.restored_pending_review, i.observation_gap_count,
               i.last_observation_gap_reason,
               to_char(i.last_observation_gap_at AT TIME ZONE 'UTC',
                       'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')
                   AS last_observation_gap_at,
               (SELECT count(*) FROM server_module_outbox o
                 WHERE o.instance_id = i.instance_id AND o.status <> 'delivered')
                   AS outstanding_events,
               (SELECT count(*) FROM server_module_outbox o
                 WHERE o.instance_id = i.instance_id AND o.status = 'dead_letter')
                   AS dead_letter_events,
               (SELECT count(*) FROM server_module_delivery_receipts d
                 WHERE d.release_id = i.release_id) AS delivery_receipts,
               (SELECT count(*) FROM server_module_delivery_receipts d
                 WHERE d.release_id = i.release_id
                   AND (d.request_body IS NULL OR d.target_report_id IS NULL))
                   AS legacy_delivery_receipts,
               (SELECT count(*) FROM server_module_intent_receipts x
                 WHERE x.release_id = i.release_id) AS intent_receipts
        FROM server_module_instances i
        JOIN server_module_releases r ON r.release_id = i.release_id
        ORDER BY i.module_id
        LIMIT 16
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(database_error)?;
    Ok(ModuleInventory {
        format: "omarchygs.server-module-inventory/v1".into(),
        modules: rows.into_iter().map(ModuleInventoryEntry::from).collect(),
    })
}

#[derive(FromRow)]
struct ModuleInventoryRow {
    module_id: String,
    release_id: Uuid,
    component_sha256: String,
    lifecycle: String,
    lifecycle_revision: i64,
    config_revision: i64,
    state_revision: i64,
    restored_pending_review: bool,
    observation_gap_count: i64,
    last_observation_gap_reason: Option<String>,
    last_observation_gap_at: Option<String>,
    outstanding_events: i64,
    dead_letter_events: i64,
    delivery_receipts: i64,
    legacy_delivery_receipts: i64,
    intent_receipts: i64,
}

impl From<ModuleInventoryRow> for ModuleInventoryEntry {
    fn from(row: ModuleInventoryRow) -> Self {
        Self {
            module_id: row.module_id,
            release_id: row.release_id,
            component_sha256: row.component_sha256,
            lifecycle: row.lifecycle,
            lifecycle_revision: row.lifecycle_revision,
            config_revision: row.config_revision,
            state_revision: row.state_revision,
            outstanding_events: row.outstanding_events,
            dead_letter_events: row.dead_letter_events,
            observation_gap_count: row.observation_gap_count,
            last_observation_gap_reason: row.last_observation_gap_reason,
            last_observation_gap_at: row.last_observation_gap_at,
            delivery_receipts: row.delivery_receipts,
            legacy_delivery_receipts: row.legacy_delivery_receipts,
            intent_receipts: row.intent_receipts,
            restored_pending_review: row.restored_pending_review,
        }
    }
}

fn validate_actor_reason(actor: &str, reason: &str) -> Result<(), ModuleError> {
    if actor.is_empty()
        || actor.len() > 64
        || actor.trim() != actor
        || actor.chars().any(char::is_control)
        || reason.is_empty()
        || reason.len() > 500
        || reason.trim() != reason
        || reason.chars().any(char::is_control)
    {
        return Err(ModuleError::InvalidInput);
    }
    Ok(())
}

fn parse_report_payload(value: &Value) -> Result<(Uuid, String), ModuleError> {
    let object = value.as_object().ok_or(ModuleError::InvalidInput)?;
    if object.len() != 3 || object.get("kind").and_then(Value::as_str) != Some("persona_reported") {
        return Err(ModuleError::InvalidInput);
    }
    let report_id = object
        .get("report_id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(ModuleError::InvalidInput)?;
    let category = object
        .get("category")
        .and_then(Value::as_str)
        .filter(|category| matches!(*category, "harassment" | "spam" | "cheating" | "other"))
        .ok_or(ModuleError::InvalidInput)?;
    Ok((report_id, category.into()))
}

fn json_object_to_string_map(value: &Value) -> Result<BTreeMap<String, String>, ModuleError> {
    let object = value.as_object().ok_or(ModuleError::InvalidInput)?;
    let mut values = BTreeMap::new();
    for (key, value) in object {
        let value = value.as_str().ok_or(ModuleError::InvalidInput)?;
        values.insert(key.clone(), value.to_owned());
    }
    validate_string_map(&values)?;
    Ok(values)
}

fn decode_canonical<T: for<'de> Deserialize<'de> + Serialize>(
    bytes: &[u8],
) -> Result<T, ModuleError> {
    if bytes.is_empty() || bytes.len() > omarchygs_server_module_runtime::MAX_ARTIFACT_BYTES {
        return Err(ModuleError::InvalidInput);
    }
    let value: T = serde_json::from_slice(bytes).map_err(|_| ModuleError::InvalidInput)?;
    if canonical_json(&value).map_err(contract_failure)? != bytes {
        return Err(ModuleError::InvalidInput);
    }
    Ok(value)
}

fn decode_signed_payload<T: for<'de> Deserialize<'de>>(
    envelope: &SignedEnvelope,
) -> Result<T, ModuleError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(&envelope.payload)
        .map_err(|_| ModuleError::InvalidInput)?;
    if bytes.len() > omarchygs_server_module_runtime::MAX_ARTIFACT_BYTES
        || URL_SAFE_NO_PAD.encode(&bytes) != envelope.payload
    {
        return Err(ModuleError::InvalidInput);
    }
    serde_json::from_slice(&bytes).map_err(|_| ModuleError::InvalidInput)
}

fn contract_failure(error: impl std::fmt::Display) -> ModuleError {
    error!(error = %error, "server module contract failed");
    ModuleError::InvalidInput
}

fn database_error(error: sqlx::Error) -> ModuleError {
    error!(error = %error, "server module database operation failed");
    ModuleError::Internal
}
