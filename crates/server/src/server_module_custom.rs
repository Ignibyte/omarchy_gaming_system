//! Local-only administrator custody and lifecycle for operator-custom modules.

use std::{
    collections::BTreeMap,
    env,
    fs::{File, Metadata},
    io::Read as _,
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::SigningKey;
use omarchygs_server_module_runtime::{
    AdmissionCoordinates, Capability, ExecutionTrust, HOOK_FORMAT, HookKind, HookPayload,
    HostRequest, HostResult, MAX_ARTIFACT_BYTES, ModuleHookEvent, ModuleSubject, ProcessSupervisor,
    RESPONSE_FORMAT, ReviewedRelease, SignedEnvelope, canonical_json, decode_verifying_key,
    host_request, sha256_hex, sign_active_admission_with_grants, sign_operator_custom_provenance,
    verify_release_material, verifying_key_sha256,
};
use rustix::{
    fs::{CWD, Mode, OFlags, ResolveFlags, openat2},
    process::geteuid,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, Transaction, types::Json};
use uuid::Uuid;

use crate::server_modules::ModuleError;

/// Exact acknowledgement required before unreviewed executable code is staged.
pub const UNREVIEWED_ACKNOWLEDGEMENT: &str =
    "I understand this module is unreviewed and unsupported by OmarchyGS.";
/// Maximum canonical command descriptor bytes.
pub const MAX_CUSTOM_MODULE_COMMAND_BYTES: usize = 64 * 1024;
/// Hard ceiling for operator-custom module identities on one server.
pub const MAX_CUSTOM_MODULES: i64 = 8;

const PUBLIC_KEY_FORMAT: &str = "omarchygs.server-module-public-key/v1";
const PRIVATE_KEY_FORMAT: &str = "omarchygs.server-module-private-key/v1";
const IMPORT_FORMAT: &str = "omarchygs.operator-custom-module-import-command/v1";
const LIFECYCLE_FORMAT: &str = "omarchygs.operator-custom-module-lifecycle-command/v1";
const REGISTRY_ADVISORY_LOCK: i64 = 0x4f47_534d_4f44_3031;
const MAX_KEY_DOCUMENT_BYTES: usize = 4096;
const MAX_STATE_ENTRIES: usize = 32;
const MAX_STATE_BYTES: usize = 4096;
const MAX_STATE_VALUE_BYTES: usize = 512;

/// Exact public-key document supplied by the publisher.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModulePublicKeyDocument {
    pub format: String,
    pub algorithm: String,
    pub key_id: String,
    pub verifying_key: String,
}

/// Owner-private provenance signing key used only by the local CLI.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModulePrivateKeyDocument {
    pub format: String,
    pub algorithm: String,
    pub key_id: String,
    pub signing_seed: String,
}

/// Canonical database-local import descriptor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomModuleImportCommand {
    pub format: String,
    pub operation_id: Uuid,
    pub server_id: Uuid,
    pub signed_release_path: PathBuf,
    pub component_path: PathBuf,
    pub publisher_public_key_path: PathBuf,
    pub provenance_private_key_path: PathBuf,
    pub publisher_key_sha256: String,
    pub provenance_key_sha256: String,
    pub granted_capabilities: Vec<Capability>,
    #[serde(default)]
    pub initial_config: BTreeMap<String, String>,
    #[serde(default)]
    pub initial_state: BTreeMap<String, String>,
    pub acknowledgement: String,
    pub actor: String,
    pub reason: String,
}

/// Custom lifecycle mutation selected by an exact command descriptor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomModuleLifecycleAction {
    Enable,
    Disable,
    Suspend,
    Recover,
    Upgrade,
    Rollback,
    Remove,
}

impl CustomModuleLifecycleAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Enable => "enable",
            Self::Disable => "disable",
            Self::Suspend => "suspend",
            Self::Recover => "recover",
            Self::Upgrade => "upgrade",
            Self::Rollback => "rollback",
            Self::Remove => "remove",
        }
    }
}

/// Canonical local lifecycle descriptor. Candidate state is accepted only by upgrade.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomModuleLifecycleCommand {
    pub format: String,
    pub action: CustomModuleLifecycleAction,
    pub operation_id: Uuid,
    pub instance_id: Uuid,
    pub expected_lifecycle_revision: u64,
    pub expected_config_revision: u64,
    pub expected_state_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_release_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_state: Option<BTreeMap<String, String>>,
    pub actor: String,
    pub reason: String,
}

/// Stable non-secret receipt for an idempotent custom-module operation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomModuleReceipt {
    pub format: String,
    pub operation_id: Uuid,
    pub action: String,
    pub instance_id: Uuid,
    pub module_id: String,
    pub release_id: Uuid,
    pub lifecycle: String,
    pub lifecycle_revision: u64,
    pub state_revision: u64,
    pub replayed: bool,
}

/// Runtime secrets required by local custom-module admission operations.
#[derive(Clone)]
pub struct CustomModuleAdminConfig {
    admission_signing_seed: [u8; 32],
    _pairwise_secret: [u8; 32],
}

impl CustomModuleAdminConfig {
    /// Parse the same all-or-none runtime keys required by the server worker.
    pub fn from_environment() -> Result<Self, ModuleError> {
        let admission = env::var("OGS_MODULE_ADMISSION_SIGNING_SEED")
            .map_err(|_| ModuleError::InvalidConfig)?;
        let pairwise =
            env::var("OGS_MODULE_PAIRWISE_SECRET").map_err(|_| ModuleError::InvalidConfig)?;
        Ok(Self {
            admission_signing_seed: decode_secret(&admission)?,
            _pairwise_secret: decode_secret(&pairwise)?,
        })
    }

    fn signer(&self) -> SigningKey {
        SigningKey::from_bytes(&self.admission_signing_seed)
    }

    #[cfg(test)]
    pub(crate) const fn for_test(admission_signing_seed: [u8; 32]) -> Self {
        Self {
            admission_signing_seed,
            _pairwise_secret: [22_u8; 32],
        }
    }
}

pub(crate) trait ReleaseProbe {
    fn probe(
        &self,
        release: &ReviewedRelease,
        request: &HostRequest,
        core_key: &ed25519_dalek::VerifyingKey,
    ) -> Result<omarchygs_server_module_runtime::HostResponse, ModuleError>;
}

struct ContainedReleaseProbe;

impl ReleaseProbe for ContainedReleaseProbe {
    fn probe(
        &self,
        release: &ReviewedRelease,
        request: &HostRequest,
        core_key: &ed25519_dalek::VerifyingKey,
    ) -> Result<omarchygs_server_module_runtime::HostResponse, ModuleError> {
        ProcessSupervisor::packaged_sibling()
            .map_err(|_| ModuleError::Unavailable)?
            .execute_release(request, core_key, release)
            .map(|report| report.response)
            .map_err(|_| ModuleError::Unavailable)
    }
}

#[cfg(test)]
pub(crate) struct LocalReleaseProbe;

#[cfg(test)]
impl ReleaseProbe for LocalReleaseProbe {
    fn probe(
        &self,
        release: &ReviewedRelease,
        request: &HostRequest,
        core_key: &ed25519_dalek::VerifyingKey,
    ) -> Result<omarchygs_server_module_runtime::HostResponse, ModuleError> {
        let runtime =
            omarchygs_server_module_runtime::ModuleRuntime::compile_bytes(&release.component_bytes)
                .map_err(contract_failure)?;
        runtime.readiness().map_err(contract_failure)?;
        Ok(runtime.execute_release(request, core_key, release))
    }
}

/// Decode a strict canonical import command before following any referenced path.
pub fn decode_import_command(bytes: &[u8]) -> Result<CustomModuleImportCommand, ModuleError> {
    decode_canonical(bytes, MAX_CUSTOM_MODULE_COMMAND_BYTES)
}

/// Decode a strict canonical lifecycle command.
pub fn decode_lifecycle_command(bytes: &[u8]) -> Result<CustomModuleLifecycleCommand, ModuleError> {
    decode_canonical(bytes, MAX_CUSTOM_MODULE_COMMAND_BYTES)
}

/// Import one exact release and create a disabled instance when the module is new.
pub async fn import_custom_module(
    pool: &PgPool,
    config: &CustomModuleAdminConfig,
    command: &CustomModuleImportCommand,
) -> Result<CustomModuleReceipt, ModuleError> {
    import_custom_module_with_probe(pool, config, command, &ContainedReleaseProbe).await
}

pub(crate) async fn import_custom_module_with_probe(
    pool: &PgPool,
    config: &CustomModuleAdminConfig,
    command: &CustomModuleImportCommand,
    probe: &dyn ReleaseProbe,
) -> Result<CustomModuleReceipt, ModuleError> {
    validate_import_command(command)?;
    let command_bytes = canonical_json(command).map_err(contract_failure)?;
    let command_sha256 = sha256_hex(&command_bytes);
    if let Some(receipt) =
        load_replay(pool, command.operation_id, "import", &command_sha256).await?
    {
        return Ok(receipt);
    }

    let database_server_id: Uuid =
        sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton")
            .fetch_one(pool)
            .await
            .map_err(database_error)?;
    if database_server_id != command.server_id {
        return Err(ModuleError::Denied);
    }

    let release: SignedEnvelope = decode_canonical(
        &read_private_file(&command.signed_release_path, MAX_ARTIFACT_BYTES)?,
        MAX_ARTIFACT_BYTES,
    )?;
    let component = read_private_file(&command.component_path, MAX_ARTIFACT_BYTES)?;
    let publisher_document: ModulePublicKeyDocument = decode_canonical(
        &read_private_file(&command.publisher_public_key_path, MAX_KEY_DOCUMENT_BYTES)?,
        MAX_KEY_DOCUMENT_BYTES,
    )?;
    let provenance_document: ModulePrivateKeyDocument = decode_canonical(
        &read_private_file(&command.provenance_private_key_path, MAX_KEY_DOCUMENT_BYTES)?,
        MAX_KEY_DOCUMENT_BYTES,
    )?;
    let publisher_key = validate_public_key(&publisher_document)?;
    let provenance_key = validate_private_key(&provenance_document)?;
    let publisher_fingerprint = verifying_key_sha256(&publisher_key);
    let provenance_fingerprint = verifying_key_sha256(&provenance_key.verifying_key());
    if publisher_fingerprint != command.publisher_key_sha256
        || provenance_fingerprint != command.provenance_key_sha256
    {
        return Err(ModuleError::Denied);
    }
    let (_, provenance) = sign_operator_custom_provenance(
        &release,
        command.server_id,
        &provenance_document.key_id,
        &provenance_key,
    )
    .map_err(contract_failure)?;
    let reviewed = verify_release_material(
        release,
        provenance,
        &ExecutionTrust {
            publisher_key_id: publisher_document.key_id.clone(),
            publisher_public_key: publisher_key,
            provenance_key_id: provenance_document.key_id.clone(),
            provenance_public_key: provenance_key.verifying_key(),
            provenance_class: "operator_custom".into(),
            provenance_server_id: Some(command.server_id),
        },
        component,
    )
    .map_err(contract_failure)?;
    validate_grant(
        &command.granted_capabilities,
        &reviewed.manifest.requested_capabilities,
    )?;
    validate_snapshot(&command.initial_config)?;
    validate_snapshot(&command.initial_state)?;
    let signer = config.signer();
    probe_release(
        probe,
        ReleaseProbeInput {
            release: &reviewed,
            signer: &signer,
            grants: &command.granted_capabilities,
            config: &command.initial_config,
            state: &command.initial_state,
            coordinates: AdmissionCoordinates {
                server_id: command.server_id,
                admission_id: Uuid::new_v4(),
                lifecycle_revision: 1,
                config_revision: 1,
                state_revision: 0,
            },
        },
    )?;

    persist_import(
        pool,
        command,
        &command_sha256,
        &reviewed,
        &publisher_fingerprint,
        &provenance_fingerprint,
    )
    .await
}

/// Apply one expected-revision custom lifecycle operation.
pub async fn apply_custom_lifecycle(
    pool: &PgPool,
    config: &CustomModuleAdminConfig,
    command: &CustomModuleLifecycleCommand,
) -> Result<CustomModuleReceipt, ModuleError> {
    apply_custom_lifecycle_with_probe(pool, config, command, &ContainedReleaseProbe).await
}

pub(crate) async fn apply_custom_lifecycle_with_probe(
    pool: &PgPool,
    config: &CustomModuleAdminConfig,
    command: &CustomModuleLifecycleCommand,
    probe: &dyn ReleaseProbe,
) -> Result<CustomModuleReceipt, ModuleError> {
    validate_lifecycle_command(command)?;
    let command_sha256 = sha256_hex(&canonical_json(command).map_err(contract_failure)?);
    let action = command.action.as_str();
    if let Some(receipt) = load_replay(pool, command.operation_id, action, &command_sha256).await? {
        return Ok(receipt);
    }
    let candidate = load_lifecycle_candidate(pool, command).await?;
    let signer = config.signer();
    let prepared = prepare_lifecycle(pool, command, candidate, &signer, probe).await?;
    finalize_lifecycle(pool, command, &command_sha256, prepared).await
}

#[derive(FromRow)]
struct LifecycleCandidateRow {
    instance_id: Uuid,
    module_id: String,
    release_id: Uuid,
    lifecycle: String,
    lifecycle_revision: i64,
    config: Json<Value>,
    config_revision: i64,
    state_schema: String,
    state_revision: i64,
    activation_allowed: bool,
    restored_pending_review: bool,
    previous_release_id: Option<Uuid>,
    rollback_snapshot_id: Option<Uuid>,
    state: Json<Value>,
    state_byte_size: i32,
}

struct PreparedLifecycle {
    candidate: LifecycleCandidateRow,
    target_release: ReviewedRelease,
    target_release_id: Uuid,
    target_state: BTreeMap<String, String>,
    target_state_schema: String,
    admission: Option<(
        omarchygs_server_module_runtime::ModuleAdmission,
        SignedEnvelope,
    )>,
    publisher_key_sha256: String,
    provenance_key_sha256: String,
    granted_capabilities: Vec<Capability>,
    previous_snapshot: Option<(Uuid, String, i64, Json<Value>, i32)>,
}

#[derive(FromRow)]
struct ExistingReleaseRow {
    component_sha256: String,
    signed_release: Vec<u8>,
    signed_provenance: Vec<u8>,
    component_bytes: Option<Vec<u8>>,
}

struct ReleaseProbeInput<'a> {
    release: &'a ReviewedRelease,
    signer: &'a SigningKey,
    grants: &'a [Capability],
    config: &'a BTreeMap<String, String>,
    state: &'a BTreeMap<String, String>,
    coordinates: AdmissionCoordinates,
}

#[derive(FromRow)]
struct CustomOperationReplayRow {
    action: String,
    command_sha256: String,
    instance_id: Uuid,
    release_id: Uuid,
    resulting_lifecycle: String,
    resulting_lifecycle_revision: i64,
    resulting_state_revision: i64,
    module_id: String,
}

async fn persist_import(
    pool: &PgPool,
    command: &CustomModuleImportCommand,
    command_sha256: &str,
    reviewed: &ReviewedRelease,
    publisher_fingerprint: &str,
    provenance_fingerprint: &str,
) -> Result<CustomModuleReceipt, ModuleError> {
    let signed_release = canonical_json(&reviewed.release).map_err(contract_failure)?;
    let signed_provenance = canonical_json(&reviewed.provenance).map_err(contract_failure)?;
    let config_value = map_value(&command.initial_config);
    let state_value = map_value(&command.initial_state);
    let mut transaction = pool.begin().await.map_err(database_error)?;
    registry_lock(&mut transaction).await?;
    if let Some(receipt) = load_replay_tx(
        &mut transaction,
        command.operation_id,
        "import",
        command_sha256,
    )
    .await?
    {
        transaction.commit().await.map_err(database_error)?;
        return Ok(receipt);
    }
    let server_id: Uuid =
        sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton FOR SHARE")
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?;
    if server_id != command.server_id {
        return Err(ModuleError::Denied);
    }
    let existing_release = sqlx::query_as::<_, ExistingReleaseRow>(
        r#"
        SELECT component_sha256, signed_release, signed_provenance, component_bytes
        FROM server_module_releases WHERE release_id = $1 FOR SHARE
        "#,
    )
    .bind(reviewed.manifest.release_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?;
    if let Some(existing) = existing_release {
        if existing.component_sha256 != reviewed.manifest.component_sha256
            || existing.signed_release != signed_release
            || existing.signed_provenance != signed_provenance
            || existing.component_bytes.as_deref() != Some(reviewed.component_bytes.as_slice())
        {
            return Err(ModuleError::Conflict);
        }
        let existing_grants: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT granted_capabilities
            FROM server_module_custom_operations
            WHERE release_id = $1 AND action = 'import'
            ORDER BY created_at, operation_id
            LIMIT 1
            "#,
        )
        .bind(reviewed.manifest.release_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .ok_or(ModuleError::Conflict)?;
        if parse_capabilities(&existing_grants)? != command.granted_capabilities {
            return Err(ModuleError::Conflict);
        }
    } else {
        sqlx::query(
            r#"
            INSERT INTO server_module_releases (
                release_id, module_id, publisher_id, version, release_format,
                signed_release, release_sha256, signed_provenance,
                provenance_sha256, provenance_class, review_id, component_sha256,
                wit_package, wit_world, wit_major, wit_sha256,
                requested_capabilities, subscribed_hooks, frame_bytes,
                memory_bytes, fuel, execution_ms, config_schema, state_schema,
                component_bytes, artifact_custody, publisher_key_id,
                publisher_public_key, publisher_key_sha256, provenance_key_id,
                provenance_public_key, provenance_key_sha256, provenance_server_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, 'operator_custom', NULL,
                $10, $11, $12, $13, $14, $15, ARRAY['persona_reported']::TEXT[],
                $16, $17, $18, $19, $20, $21, $22, 'database_immutable',
                $23, $24, $25, $26, $27, $28, $29
            )
            "#,
        )
        .bind(reviewed.manifest.release_id)
        .bind(&reviewed.manifest.module_id)
        .bind(&reviewed.manifest.publisher_id)
        .bind(&reviewed.manifest.version)
        .bind(&reviewed.manifest.format)
        .bind(&signed_release)
        .bind(
            reviewed
                .release
                .payload_sha256()
                .map_err(contract_failure)?,
        )
        .bind(&signed_provenance)
        .bind(
            reviewed
                .provenance
                .payload_sha256()
                .map_err(contract_failure)?,
        )
        .bind(&reviewed.manifest.component_sha256)
        .bind(&reviewed.manifest.wit.package)
        .bind(&reviewed.manifest.wit.world)
        .bind(i32::from(reviewed.manifest.wit.major))
        .bind(&reviewed.manifest.wit.sha256)
        .bind(capability_strings(
            &reviewed.manifest.requested_capabilities,
        ))
        .bind(
            i32::try_from(reviewed.manifest.budgets.frame_bytes)
                .map_err(|_| ModuleError::Internal)?,
        )
        .bind(
            i32::try_from(reviewed.manifest.budgets.memory_bytes)
                .map_err(|_| ModuleError::Internal)?,
        )
        .bind(i64::try_from(reviewed.manifest.budgets.fuel).map_err(|_| ModuleError::Internal)?)
        .bind(
            i32::try_from(reviewed.manifest.budgets.execution_ms)
                .map_err(|_| ModuleError::Internal)?,
        )
        .bind(&reviewed.manifest.config_schema)
        .bind(&reviewed.manifest.state_schema)
        .bind(&reviewed.component_bytes)
        .bind(&reviewed.publisher_key_id)
        .bind(omarchygs_server_module_runtime::encode_verifying_key(
            &reviewed.publisher_public_key,
        ))
        .bind(publisher_fingerprint)
        .bind(&reviewed.provenance_key_id)
        .bind(omarchygs_server_module_runtime::encode_verifying_key(
            &reviewed.provenance_public_key,
        ))
        .bind(provenance_fingerprint)
        .bind(command.server_id)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }

    let existing_instance: Option<(Uuid, String, i64, Uuid, i64, String)> = sqlx::query_as(
        r#"
        SELECT i.instance_id, i.lifecycle, i.lifecycle_revision, i.release_id,
               i.state_revision, r.provenance_class
        FROM server_module_instances i
        JOIN server_module_releases r ON r.release_id = i.release_id
        WHERE i.module_id = $1
        FOR UPDATE OF i
        "#,
    )
    .bind(&reviewed.manifest.module_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?;
    let (instance_id, lifecycle, lifecycle_revision, state_revision) =
        if let Some(existing) = existing_instance {
            if existing.5 != "operator_custom" || existing.1 == "retired" {
                return Err(ModuleError::Denied);
            }
            (existing.0, existing.1, existing.2, existing.4)
        } else {
            let custom_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*) FROM server_module_instances i
                JOIN server_module_releases r ON r.release_id = i.release_id
                WHERE r.provenance_class = 'operator_custom'
                "#,
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?;
            if custom_count >= MAX_CUSTOM_MODULES {
                return Err(ModuleError::Denied);
            }
            let instance_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO server_module_instances (
                    instance_id, module_id, release_id, lifecycle,
                    lifecycle_revision, config, config_revision, state_schema,
                    state_revision
                ) VALUES ($1, $2, $3, 'disabled', 1, $4, 1, $5, 0)
                "#,
            )
            .bind(instance_id)
            .bind(&reviewed.manifest.module_id)
            .bind(reviewed.manifest.release_id)
            .bind(Json(config_value))
            .bind(&reviewed.manifest.state_schema)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
            sqlx::query(
                r#"
                INSERT INTO server_module_state_namespaces (
                    instance_id, state_schema, revision, entries, byte_size
                ) VALUES ($1, $2, 0, $3, octet_length(($3::JSONB)::TEXT))
                "#,
            )
            .bind(instance_id)
            .bind(&reviewed.manifest.state_schema)
            .bind(Json(state_value))
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
            insert_lifecycle_audit(
                &mut transaction,
                command.operation_id,
                instance_id,
                "import",
                0,
                "absent",
                "disabled",
                1,
                &command.actor,
                &command.reason,
            )
            .await?;
            (instance_id, "disabled".to_owned(), 1, 0)
        };
    insert_custom_operation(
        &mut transaction,
        command.operation_id,
        "import",
        command_sha256,
        instance_id,
        reviewed.manifest.release_id,
        publisher_fingerprint,
        provenance_fingerprint,
        &reviewed.manifest.requested_capabilities,
        &command.granted_capabilities,
        0,
        &lifecycle,
        lifecycle_revision,
        state_revision,
        &command.actor,
        &command.reason,
    )
    .await?;
    transaction.commit().await.map_err(database_error)?;
    Ok(CustomModuleReceipt {
        format: "omarchygs.operator-custom-module-receipt/v1".into(),
        operation_id: command.operation_id,
        action: "import".into(),
        instance_id,
        module_id: reviewed.manifest.module_id.clone(),
        release_id: reviewed.manifest.release_id,
        lifecycle,
        lifecycle_revision: u64::try_from(lifecycle_revision).map_err(|_| ModuleError::Internal)?,
        state_revision: u64::try_from(state_revision).map_err(|_| ModuleError::Internal)?,
        replayed: false,
    })
}

#[derive(FromRow)]
struct StoredReleaseRow {
    release_id: Uuid,
    module_id: String,
    signed_release: Vec<u8>,
    signed_provenance: Vec<u8>,
    provenance_class: String,
    provenance_server_id: Option<Uuid>,
    component_bytes: Option<Vec<u8>>,
    publisher_key_id: Option<String>,
    publisher_public_key: Option<String>,
    publisher_key_sha256: Option<String>,
    provenance_key_id: Option<String>,
    provenance_public_key: Option<String>,
    provenance_key_sha256: Option<String>,
}

async fn load_lifecycle_candidate(
    pool: &PgPool,
    command: &CustomModuleLifecycleCommand,
) -> Result<LifecycleCandidateRow, ModuleError> {
    let row = sqlx::query_as::<_, LifecycleCandidateRow>(
        r#"
        SELECT i.instance_id, i.module_id, i.release_id, i.lifecycle,
               i.lifecycle_revision, i.config, i.config_revision,
               i.state_schema, i.state_revision, i.activation_allowed,
               i.restored_pending_review, i.previous_release_id,
               i.rollback_snapshot_id, n.entries AS state,
               n.byte_size AS state_byte_size
        FROM server_module_instances i
        JOIN server_module_releases r ON r.release_id = i.release_id
        JOIN server_module_state_namespaces n ON n.instance_id = i.instance_id
        WHERE i.instance_id = $1 AND r.provenance_class = 'operator_custom'
        "#,
    )
    .bind(command.instance_id)
    .fetch_optional(pool)
    .await
    .map_err(database_error)?
    .ok_or(ModuleError::Denied)?;
    if row.lifecycle_revision
        != i64::try_from(command.expected_lifecycle_revision)
            .map_err(|_| ModuleError::InvalidInput)?
        || row.config_revision
            != i64::try_from(command.expected_config_revision)
                .map_err(|_| ModuleError::InvalidInput)?
        || row.state_revision
            != i64::try_from(command.expected_state_revision)
                .map_err(|_| ModuleError::InvalidInput)?
    {
        return Err(ModuleError::Conflict);
    }
    Ok(row)
}

async fn prepare_lifecycle(
    pool: &PgPool,
    command: &CustomModuleLifecycleCommand,
    candidate: LifecycleCandidateRow,
    signer: &SigningKey,
    probe: &dyn ReleaseProbe,
) -> Result<PreparedLifecycle, ModuleError> {
    if candidate.lifecycle == "retired" {
        return Err(ModuleError::Denied);
    }
    let restored_recovery = command.action == CustomModuleLifecycleAction::Recover
        && candidate.lifecycle == "disabled"
        && candidate.restored_pending_review;
    if !candidate.activation_allowed
        && !restored_recovery
        && !matches!(
            command.action,
            CustomModuleLifecycleAction::Disable
                | CustomModuleLifecycleAction::Suspend
                | CustomModuleLifecycleAction::Remove
        )
    {
        return Err(ModuleError::Denied);
    }
    let (target_release_id, target_state, target_state_schema, previous_snapshot) =
        match command.action {
            CustomModuleLifecycleAction::Enable => {
                if candidate.lifecycle != "disabled" || candidate.restored_pending_review {
                    return Err(ModuleError::Denied);
                }
                (
                    candidate.release_id,
                    value_map(&candidate.state.0)?,
                    candidate.state_schema.clone(),
                    None,
                )
            }
            CustomModuleLifecycleAction::Recover => {
                if !restored_recovery
                    && !matches!(candidate.lifecycle.as_str(), "degraded" | "suspended")
                {
                    return Err(ModuleError::Denied);
                }
                (
                    candidate.release_id,
                    value_map(&candidate.state.0)?,
                    candidate.state_schema.clone(),
                    None,
                )
            }
            CustomModuleLifecycleAction::Upgrade => {
                if !matches!(candidate.lifecycle.as_str(), "active" | "disabled") {
                    return Err(ModuleError::Denied);
                }
                let release_id = command.target_release_id.ok_or(ModuleError::InvalidInput)?;
                if release_id == candidate.release_id {
                    return Err(ModuleError::Conflict);
                }
                let state = command
                    .candidate_state
                    .clone()
                    .ok_or(ModuleError::InvalidInput)?;
                validate_snapshot(&state)?;
                let snapshot_id = Uuid::new_v4();
                (
                    release_id,
                    state,
                    String::new(),
                    Some((
                        snapshot_id,
                        candidate.state_schema.clone(),
                        candidate.state_revision,
                        candidate.state.clone(),
                        candidate.state_byte_size,
                    )),
                )
            }
            CustomModuleLifecycleAction::Rollback => {
                if !matches!(candidate.lifecycle.as_str(), "active" | "disabled") {
                    return Err(ModuleError::Denied);
                }
                let release_id = candidate.previous_release_id.ok_or(ModuleError::Denied)?;
                let snapshot_id = candidate.rollback_snapshot_id.ok_or(ModuleError::Denied)?;
                let snapshot: (String, Json<Value>) = sqlx::query_as(
                    r#"
                    SELECT source_schema, entries FROM server_module_state_snapshots
                    WHERE snapshot_id = $1 AND instance_id = $2
                    "#,
                )
                .bind(snapshot_id)
                .bind(candidate.instance_id)
                .fetch_optional(pool)
                .await
                .map_err(database_error)?
                .ok_or(ModuleError::Conflict)?;
                (release_id, value_map(&snapshot.1.0)?, snapshot.0, None)
            }
            CustomModuleLifecycleAction::Disable
            | CustomModuleLifecycleAction::Suspend
            | CustomModuleLifecycleAction::Remove => {
                let allowed = match command.action {
                    CustomModuleLifecycleAction::Disable => {
                        matches!(candidate.lifecycle.as_str(), "active" | "degraded")
                    }
                    CustomModuleLifecycleAction::Suspend => candidate.lifecycle != "retired",
                    CustomModuleLifecycleAction::Remove => candidate.lifecycle != "retired",
                    _ => false,
                };
                if !allowed {
                    return Err(ModuleError::Denied);
                }
                (
                    candidate.release_id,
                    value_map(&candidate.state.0)?,
                    candidate.state_schema.clone(),
                    None,
                )
            }
        };

    let target_release = load_stored_release(pool, target_release_id).await?;
    if target_release.manifest.module_id != candidate.module_id {
        return Err(ModuleError::Denied);
    }
    let target_state_schema = if target_state_schema.is_empty() {
        target_release.manifest.state_schema.clone()
    } else {
        target_state_schema
    };
    if target_state_schema != target_release.manifest.state_schema {
        return Err(ModuleError::InvalidInput);
    }
    let database_server_id: Uuid =
        sqlx::query_scalar("SELECT id FROM server_identity WHERE singleton")
            .fetch_one(pool)
            .await
            .map_err(database_error)?;
    if target_release.provenance_statement.server_id != Some(database_server_id) {
        return Err(ModuleError::Denied);
    }
    let grants = load_release_grants(pool, target_release_id).await?;
    let next_lifecycle_revision = command
        .expected_lifecycle_revision
        .checked_add(1)
        .ok_or(ModuleError::Internal)?;
    let next_state_revision = if matches!(
        command.action,
        CustomModuleLifecycleAction::Upgrade | CustomModuleLifecycleAction::Rollback
    ) {
        command
            .expected_state_revision
            .checked_add(1)
            .ok_or(ModuleError::Internal)?
    } else {
        command.expected_state_revision
    };
    let admission = if matches!(
        command.action,
        CustomModuleLifecycleAction::Enable
            | CustomModuleLifecycleAction::Recover
            | CustomModuleLifecycleAction::Upgrade
            | CustomModuleLifecycleAction::Rollback
    ) {
        let admission = sign_active_admission_with_grants(
            &target_release,
            AdmissionCoordinates {
                server_id: target_release
                    .provenance_statement
                    .server_id
                    .ok_or(ModuleError::Conflict)?,
                admission_id: Uuid::new_v4(),
                lifecycle_revision: next_lifecycle_revision,
                config_revision: command.expected_config_revision,
                state_revision: next_state_revision,
            },
            grants.clone(),
            vec![HookKind::PersonaReported],
            signer,
        )
        .map_err(contract_failure)?;
        let candidate_config = value_map(&candidate.config.0)?;
        probe_release(
            probe,
            ReleaseProbeInput {
                release: &target_release,
                signer,
                grants: &admission.0.granted_capabilities,
                config: &candidate_config,
                state: &target_state,
                coordinates: AdmissionCoordinates {
                    server_id: admission.0.server_id,
                    admission_id: admission.0.admission_id,
                    lifecycle_revision: next_lifecycle_revision,
                    config_revision: command.expected_config_revision,
                    state_revision: next_state_revision,
                },
            },
        )?;
        Some(admission)
    } else {
        None
    };
    Ok(PreparedLifecycle {
        candidate,
        target_release_id,
        target_state,
        target_state_schema,
        admission,
        publisher_key_sha256: verifying_key_sha256(&target_release.publisher_public_key),
        provenance_key_sha256: verifying_key_sha256(&target_release.provenance_public_key),
        granted_capabilities: grants,
        target_release,
        previous_snapshot,
    })
}

async fn finalize_lifecycle(
    pool: &PgPool,
    command: &CustomModuleLifecycleCommand,
    command_sha256: &str,
    prepared: PreparedLifecycle,
) -> Result<CustomModuleReceipt, ModuleError> {
    let action = command.action.as_str();
    let mut transaction = pool.begin().await.map_err(database_error)?;
    registry_lock(&mut transaction).await?;
    if let Some(receipt) = load_replay_tx(
        &mut transaction,
        command.operation_id,
        action,
        command_sha256,
    )
    .await?
    {
        transaction.commit().await.map_err(database_error)?;
        return Ok(receipt);
    }
    let current: (
        String,
        i64,
        i64,
        i64,
        Uuid,
        bool,
        bool,
        Option<Uuid>,
        Option<Uuid>,
    ) = sqlx::query_as(
        r#"
            SELECT lifecycle, lifecycle_revision, config_revision, state_revision,
                   release_id, activation_allowed, restored_pending_review,
                   previous_release_id, rollback_snapshot_id
            FROM server_module_instances
            WHERE instance_id = $1 AND module_id = $2
            FOR UPDATE
            "#,
    )
    .bind(command.instance_id)
    .bind(&prepared.candidate.module_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(database_error)?
    .ok_or(ModuleError::Denied)?;
    if current.0 != prepared.candidate.lifecycle
        || current.1
            != i64::try_from(command.expected_lifecycle_revision)
                .map_err(|_| ModuleError::InvalidInput)?
        || current.2
            != i64::try_from(command.expected_config_revision)
                .map_err(|_| ModuleError::InvalidInput)?
        || current.3
            != i64::try_from(command.expected_state_revision)
                .map_err(|_| ModuleError::InvalidInput)?
        || current.4 != prepared.candidate.release_id
        || current.5 != prepared.candidate.activation_allowed
        || current.6 != prepared.candidate.restored_pending_review
        || current.7 != prepared.candidate.previous_release_id
        || current.8 != prepared.candidate.rollback_snapshot_id
    {
        return Err(ModuleError::Conflict);
    }

    let resulting_lifecycle = match command.action {
        CustomModuleLifecycleAction::Enable
        | CustomModuleLifecycleAction::Recover
        | CustomModuleLifecycleAction::Upgrade
        | CustomModuleLifecycleAction::Rollback => "active",
        CustomModuleLifecycleAction::Disable => "disabled",
        CustomModuleLifecycleAction::Suspend => "suspended",
        CustomModuleLifecycleAction::Remove => "retired",
    };
    let resulting_lifecycle_revision = command
        .expected_lifecycle_revision
        .checked_add(1)
        .ok_or(ModuleError::Internal)?;
    let state_changes = matches!(
        command.action,
        CustomModuleLifecycleAction::Upgrade | CustomModuleLifecycleAction::Rollback
    );
    let resulting_state_revision = if state_changes {
        command
            .expected_state_revision
            .checked_add(1)
            .ok_or(ModuleError::Internal)?
    } else {
        command.expected_state_revision
    };

    let (admission_id, admission_revision) = if let Some((admission, envelope)) =
        &prepared.admission
    {
        let envelope_bytes = canonical_json(envelope).map_err(contract_failure)?;
        sqlx::query(
            r#"
            INSERT INTO server_module_admissions (
                admission_id, lifecycle_revision, release_id, server_id,
                admission_format, signed_admission, admission_sha256, lifecycle,
                granted_capabilities, subscribed_hooks, config_revision,
                state_schema, state_revision
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8,
                      ARRAY['persona_reported']::TEXT[], $9, $10, $11)
            "#,
        )
        .bind(admission.admission_id)
        .bind(i64::try_from(admission.lifecycle_revision).map_err(|_| ModuleError::Internal)?)
        .bind(admission.release_id)
        .bind(admission.server_id)
        .bind(&admission.format)
        .bind(&envelope_bytes)
        .bind(envelope.payload_sha256().map_err(contract_failure)?)
        .bind(capability_strings(&admission.granted_capabilities))
        .bind(i64::try_from(admission.config_revision).map_err(|_| ModuleError::Internal)?)
        .bind(&admission.state_schema)
        .bind(i64::try_from(admission.state_revision).map_err(|_| ModuleError::Internal)?)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        (
            Some(admission.admission_id),
            Some(i64::try_from(admission.lifecycle_revision).map_err(|_| ModuleError::Internal)?),
        )
    } else {
        (None, None)
    };

    let mut rollback_release = prepared.candidate.previous_release_id;
    let mut rollback_snapshot = prepared.candidate.rollback_snapshot_id;
    if command.action == CustomModuleLifecycleAction::Upgrade {
        let snapshot = prepared
            .previous_snapshot
            .as_ref()
            .ok_or(ModuleError::Internal)?;
        sqlx::query(
            r#"
            INSERT INTO server_module_state_snapshots (
                snapshot_id, instance_id, source_schema, source_revision,
                entries, byte_size, reason
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(snapshot.0)
        .bind(command.instance_id)
        .bind(&snapshot.1)
        .bind(snapshot.2)
        .bind(&snapshot.3)
        .bind(snapshot.4)
        .bind(format!(
            "Immediate predecessor for upgrade operation {}",
            command.operation_id
        ))
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        rollback_release = Some(prepared.candidate.release_id);
        rollback_snapshot = Some(snapshot.0);
    } else if command.action == CustomModuleLifecycleAction::Rollback {
        rollback_release = None;
        rollback_snapshot = None;
    }

    if state_changes {
        let value = map_value(&prepared.target_state);
        let updated = sqlx::query(
            r#"
            UPDATE server_module_state_namespaces
            SET state_schema = $2, revision = $3, entries = $4,
                byte_size = octet_length(($4::JSONB)::TEXT),
                updated_at = clock_timestamp()
            WHERE instance_id = $1 AND revision = $5
            "#,
        )
        .bind(command.instance_id)
        .bind(&prepared.target_state_schema)
        .bind(i64::try_from(resulting_state_revision).map_err(|_| ModuleError::Internal)?)
        .bind(Json(value))
        .bind(
            i64::try_from(command.expected_state_revision)
                .map_err(|_| ModuleError::InvalidInput)?,
        )
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?
        .rows_affected();
        if updated != 1 {
            return Err(ModuleError::Conflict);
        }
        sqlx::query(
            r#"
            INSERT INTO server_module_data_audit (
                operation_id, instance_id, action, command_sha256,
                expected_revision, resulting_revision, snapshot_id, actor, reason
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(command.operation_id)
        .bind(command.instance_id)
        .bind(if command.action == CustomModuleLifecycleAction::Upgrade {
            "state_migrate"
        } else {
            "state_rollback"
        })
        .bind(command_sha256)
        .bind(
            i64::try_from(command.expected_state_revision)
                .map_err(|_| ModuleError::InvalidInput)?,
        )
        .bind(i64::try_from(resulting_state_revision).map_err(|_| ModuleError::Internal)?)
        .bind(if command.action == CustomModuleLifecycleAction::Upgrade {
            rollback_snapshot
        } else {
            prepared.candidate.rollback_snapshot_id
        })
        .bind(&command.actor)
        .bind(&command.reason)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }

    if matches!(
        command.action,
        CustomModuleLifecycleAction::Disable
            | CustomModuleLifecycleAction::Suspend
            | CustomModuleLifecycleAction::Recover
            | CustomModuleLifecycleAction::Upgrade
            | CustomModuleLifecycleAction::Rollback
            | CustomModuleLifecycleAction::Remove
    ) {
        terminalize_outbox(
            &mut transaction,
            command.instance_id,
            if command.action == CustomModuleLifecycleAction::Remove {
                "module_removed"
            } else {
                "admission_replaced"
            },
        )
        .await?;
    }

    let updated = sqlx::query(
        r#"
        UPDATE server_module_instances
        SET release_id = $2, current_admission_id = $3,
            current_admission_revision = $4, lifecycle = $5,
            lifecycle_revision = $6, state_schema = $7, state_revision = $8,
            consecutive_failures = 0,
            restored_pending_review = CASE WHEN $5 = 'active' THEN FALSE ELSE restored_pending_review END,
            activation_allowed = CASE WHEN $12 THEN TRUE ELSE activation_allowed END,
            previous_release_id = $9, rollback_snapshot_id = $10,
            state_disposition = CASE WHEN $5 = 'retired' THEN 'retain_for_audit' ELSE 'live' END,
            updated_at = clock_timestamp()
        WHERE instance_id = $1 AND lifecycle_revision = $11
        "#,
    )
    .bind(command.instance_id)
    .bind(prepared.target_release_id)
    .bind(admission_id)
    .bind(admission_revision)
    .bind(resulting_lifecycle)
    .bind(i64::try_from(resulting_lifecycle_revision).map_err(|_| ModuleError::Internal)?)
    .bind(&prepared.target_state_schema)
    .bind(i64::try_from(resulting_state_revision).map_err(|_| ModuleError::Internal)?)
    .bind(rollback_release)
    .bind(rollback_snapshot)
    .bind(i64::try_from(command.expected_lifecycle_revision).map_err(|_| ModuleError::InvalidInput)?)
    .bind(command.action == CustomModuleLifecycleAction::Recover)
    .execute(&mut *transaction)
    .await
    .map_err(database_error)?
    .rows_affected();
    if updated != 1 {
        return Err(ModuleError::Conflict);
    }
    insert_lifecycle_audit(
        &mut transaction,
        command.operation_id,
        command.instance_id,
        action,
        i64::try_from(command.expected_lifecycle_revision)
            .map_err(|_| ModuleError::InvalidInput)?,
        &prepared.candidate.lifecycle,
        resulting_lifecycle,
        i64::try_from(resulting_lifecycle_revision).map_err(|_| ModuleError::Internal)?,
        &command.actor,
        &command.reason,
    )
    .await?;
    insert_custom_operation(
        &mut transaction,
        command.operation_id,
        action,
        command_sha256,
        command.instance_id,
        prepared.target_release_id,
        &prepared.publisher_key_sha256,
        &prepared.provenance_key_sha256,
        &prepared.target_release.manifest.requested_capabilities,
        &prepared.granted_capabilities,
        command.expected_lifecycle_revision,
        resulting_lifecycle,
        i64::try_from(resulting_lifecycle_revision).map_err(|_| ModuleError::Internal)?,
        i64::try_from(resulting_state_revision).map_err(|_| ModuleError::Internal)?,
        &command.actor,
        &command.reason,
    )
    .await?;
    transaction.commit().await.map_err(database_error)?;
    Ok(CustomModuleReceipt {
        format: "omarchygs.operator-custom-module-receipt/v1".into(),
        operation_id: command.operation_id,
        action: action.into(),
        instance_id: command.instance_id,
        module_id: prepared.candidate.module_id,
        release_id: prepared.target_release_id,
        lifecycle: resulting_lifecycle.into(),
        lifecycle_revision: resulting_lifecycle_revision,
        state_revision: resulting_state_revision,
        replayed: false,
    })
}

async fn load_stored_release(
    pool: &PgPool,
    release_id: Uuid,
) -> Result<ReviewedRelease, ModuleError> {
    let row = sqlx::query_as::<_, StoredReleaseRow>(
        r#"
        SELECT release_id, module_id, signed_release, signed_provenance,
               provenance_class, provenance_server_id, component_bytes,
               publisher_key_id, publisher_public_key, publisher_key_sha256,
               provenance_key_id, provenance_public_key, provenance_key_sha256
        FROM server_module_releases
        WHERE release_id = $1 AND provenance_class = 'operator_custom'
          AND artifact_custody = 'database_immutable'
        "#,
    )
    .bind(release_id)
    .fetch_optional(pool)
    .await
    .map_err(database_error)?
    .ok_or(ModuleError::Denied)?;
    let publisher_public = row.publisher_public_key.ok_or(ModuleError::Conflict)?;
    let provenance_public = row.provenance_public_key.ok_or(ModuleError::Conflict)?;
    let publisher_key = decode_verifying_key(&publisher_public).map_err(contract_failure)?;
    let provenance_key = decode_verifying_key(&provenance_public).map_err(contract_failure)?;
    if row.publisher_key_sha256.as_deref() != Some(&verifying_key_sha256(&publisher_key))
        || row.provenance_key_sha256.as_deref() != Some(&verifying_key_sha256(&provenance_key))
    {
        return Err(ModuleError::Conflict);
    }
    let release = verify_release_material(
        decode_canonical(&row.signed_release, MAX_ARTIFACT_BYTES)?,
        decode_canonical(&row.signed_provenance, MAX_ARTIFACT_BYTES)?,
        &ExecutionTrust {
            publisher_key_id: row.publisher_key_id.ok_or(ModuleError::Conflict)?,
            publisher_public_key: publisher_key,
            provenance_key_id: row.provenance_key_id.ok_or(ModuleError::Conflict)?,
            provenance_public_key: provenance_key,
            provenance_class: row.provenance_class,
            provenance_server_id: row.provenance_server_id,
        },
        row.component_bytes.ok_or(ModuleError::Conflict)?,
    )
    .map_err(contract_failure)?;
    if release.manifest.release_id != row.release_id || release.manifest.module_id != row.module_id
    {
        return Err(ModuleError::Conflict);
    }
    Ok(release)
}

async fn load_release_grants(
    pool: &PgPool,
    release_id: Uuid,
) -> Result<Vec<Capability>, ModuleError> {
    let grants: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT granted_capabilities FROM server_module_custom_operations
        WHERE release_id = $1 AND action = 'import'
        ORDER BY created_at, operation_id LIMIT 1
        "#,
    )
    .bind(release_id)
    .fetch_optional(pool)
    .await
    .map_err(database_error)?
    .ok_or(ModuleError::Conflict)?;
    parse_capabilities(&grants)
}

fn probe_release(
    probe: &dyn ReleaseProbe,
    input: ReleaseProbeInput<'_>,
) -> Result<(), ModuleError> {
    let AdmissionCoordinates {
        server_id,
        admission_id,
        lifecycle_revision,
        config_revision,
        state_revision,
    } = input.coordinates;
    let ungranted_requested_capability = input
        .release
        .manifest
        .requested_capabilities
        .iter()
        .any(|capability| !input.grants.contains(capability));
    let (_, admission) = sign_active_admission_with_grants(
        input.release,
        AdmissionCoordinates {
            server_id,
            admission_id,
            lifecycle_revision,
            config_revision,
            state_revision,
        },
        input.grants.to_vec(),
        vec![HookKind::PersonaReported],
        input.signer,
    )
    .map_err(contract_failure)?;
    let admission_payload: omarchygs_server_module_runtime::ModuleAdmission =
        decode_signed_payload(&admission)?;
    let request = host_request(
        input.release,
        admission,
        ModuleHookEvent {
            format: HOOK_FORMAT.into(),
            event_id: Uuid::new_v4(),
            attempt: 1,
            server_id,
            module_id: input.release.manifest.module_id.clone(),
            release_id: input.release.manifest.release_id,
            admission_id: admission_payload.admission_id,
            admission_revision: lifecycle_revision,
            hook: HookKind::PersonaReported,
            causal_revision: 0,
            deadline_ms: input.release.manifest.budgets.execution_ms,
            subject: ModuleSubject::Pairwise("administrator-readiness-probe".into()),
            config: input.config.clone(),
            config_revision,
            state: input.state.clone(),
            state_revision,
            payload: HookPayload::PersonaReported {
                report_id: Uuid::new_v4(),
                category: "other".into(),
            },
        },
    );
    let core_key = input.signer.verifying_key();
    let response = probe.probe(input.release, &request, &core_key)?;
    if response.format != RESPONSE_FORMAT
        || response.event_id != request.event.event_id
        || response.release_id != request.event.release_id
        || response.admission_id != request.event.admission_id
        || response.admission_revision != request.event.admission_revision
        || matches!(
            response.outcome,
            HostResult::Rejected { ref code }
                if code != "intent_not_granted" || !ungranted_requested_capability
        )
    {
        return Err(ModuleError::Unavailable);
    }
    Ok(())
}

fn validate_import_command(command: &CustomModuleImportCommand) -> Result<(), ModuleError> {
    if command.format != IMPORT_FORMAT
        || command.operation_id.is_nil()
        || command.server_id.is_nil()
        || command.acknowledgement != UNREVIEWED_ACKNOWLEDGEMENT
        || !valid_actor_reason(&command.actor, &command.reason)
        || !valid_digest(&command.publisher_key_sha256)
        || !valid_digest(&command.provenance_key_sha256)
        || command
            .granted_capabilities
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(ModuleError::InvalidInput);
    }
    let paths = [
        &command.signed_release_path,
        &command.component_path,
        &command.publisher_public_key_path,
        &command.provenance_private_key_path,
    ];
    if paths.iter().any(|path| !path.is_absolute())
        || paths
            .iter()
            .enumerate()
            .any(|(index, left)| paths.iter().skip(index + 1).any(|right| left == right))
    {
        return Err(ModuleError::InvalidInput);
    }
    Ok(())
}

fn validate_lifecycle_command(command: &CustomModuleLifecycleCommand) -> Result<(), ModuleError> {
    if command.format != LIFECYCLE_FORMAT
        || command.operation_id.is_nil()
        || command.instance_id.is_nil()
        || command.expected_lifecycle_revision == 0
        || command.expected_config_revision == 0
        || !valid_actor_reason(&command.actor, &command.reason)
    {
        return Err(ModuleError::InvalidInput);
    }
    match command.action {
        CustomModuleLifecycleAction::Upgrade => {
            if command
                .target_release_id
                .is_none_or(|release_id| release_id.is_nil())
                || command.candidate_state.is_none()
            {
                return Err(ModuleError::InvalidInput);
            }
        }
        _ if command.target_release_id.is_some() || command.candidate_state.is_some() => {
            return Err(ModuleError::InvalidInput);
        }
        _ => {}
    }
    Ok(())
}

fn validate_public_key(
    document: &ModulePublicKeyDocument,
) -> Result<ed25519_dalek::VerifyingKey, ModuleError> {
    if document.format != PUBLIC_KEY_FORMAT
        || document.algorithm != "ed25519"
        || !valid_identifier(&document.key_id, 96)
    {
        return Err(ModuleError::InvalidInput);
    }
    decode_verifying_key(&document.verifying_key).map_err(contract_failure)
}

fn validate_private_key(document: &ModulePrivateKeyDocument) -> Result<SigningKey, ModuleError> {
    if document.format != PRIVATE_KEY_FORMAT
        || document.algorithm != "ed25519"
        || !valid_identifier(&document.key_id, 96)
    {
        return Err(ModuleError::InvalidInput);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(&document.signing_seed)
        .map_err(|_| ModuleError::InvalidInput)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| ModuleError::InvalidInput)?;
    if URL_SAFE_NO_PAD.encode(bytes) != document.signing_seed {
        return Err(ModuleError::InvalidInput);
    }
    Ok(SigningKey::from_bytes(&bytes))
}

fn validate_grant(grants: &[Capability], requested: &[Capability]) -> Result<(), ModuleError> {
    if grants.windows(2).any(|pair| pair[0] >= pair[1])
        || grants
            .iter()
            .any(|grant| requested.binary_search(grant).is_err())
    {
        return Err(ModuleError::Denied);
    }
    Ok(())
}

fn validate_snapshot(values: &BTreeMap<String, String>) -> Result<(), ModuleError> {
    if values.len() > MAX_STATE_ENTRIES {
        return Err(ModuleError::InvalidInput);
    }
    let mut total = 0_usize;
    for (key, value) in values {
        if !valid_identifier(key, 64)
            || value.len() > MAX_STATE_VALUE_BYTES
            || value.chars().any(char::is_control)
        {
            return Err(ModuleError::InvalidInput);
        }
        total = total.saturating_add(key.len()).saturating_add(value.len());
    }
    let compact_bytes = serde_json::to_vec(values).map_err(|_| ModuleError::InvalidInput)?;
    let jsonb_spacing = values.len().saturating_mul(2).saturating_sub(1);
    if total > MAX_STATE_BYTES
        || compact_bytes.len().saturating_add(jsonb_spacing) > MAX_STATE_BYTES
    {
        return Err(ModuleError::InvalidInput);
    }
    Ok(())
}

fn value_map(value: &Value) -> Result<BTreeMap<String, String>, ModuleError> {
    let object = value.as_object().ok_or(ModuleError::Conflict)?;
    let mut result = BTreeMap::new();
    for (key, value) in object {
        result.insert(
            key.clone(),
            value.as_str().ok_or(ModuleError::Conflict)?.to_owned(),
        );
    }
    validate_snapshot(&result)?;
    Ok(result)
}

fn map_value(values: &BTreeMap<String, String>) -> Value {
    Value::Object(
        values
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect(),
    )
}

fn capability_strings(values: &[Capability]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

fn parse_capabilities(values: &[String]) -> Result<Vec<Capability>, ModuleError> {
    values
        .iter()
        .map(|value| match value.as_str() {
            "moderation_add_label" => Ok(Capability::ModerationAddLabel),
            _ => Err(ModuleError::Conflict),
        })
        .collect()
}

async fn load_replay(
    pool: &PgPool,
    operation_id: Uuid,
    action: &str,
    command_sha256: &str,
) -> Result<Option<CustomModuleReceipt>, ModuleError> {
    let row = sqlx::query_as::<_, CustomOperationReplayRow>(
        r#"
        SELECT o.action, o.command_sha256, o.instance_id, o.release_id,
               o.resulting_lifecycle, o.resulting_lifecycle_revision,
               o.resulting_state_revision, i.module_id
        FROM server_module_custom_operations o
        JOIN server_module_instances i ON i.instance_id = o.instance_id
        WHERE o.operation_id = $1
        "#,
    )
    .bind(operation_id)
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;
    replay_receipt(row, operation_id, action, command_sha256)
}

async fn load_replay_tx(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
    action: &str,
    command_sha256: &str,
) -> Result<Option<CustomModuleReceipt>, ModuleError> {
    let row = sqlx::query_as::<_, CustomOperationReplayRow>(
        r#"
        SELECT o.action, o.command_sha256, o.instance_id, o.release_id,
               o.resulting_lifecycle, o.resulting_lifecycle_revision,
               o.resulting_state_revision, i.module_id
        FROM server_module_custom_operations o
        JOIN server_module_instances i ON i.instance_id = o.instance_id
        WHERE o.operation_id = $1
        FOR SHARE OF o, i
        "#,
    )
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    replay_receipt(row, operation_id, action, command_sha256)
}

fn replay_receipt(
    row: Option<CustomOperationReplayRow>,
    operation_id: Uuid,
    action: &str,
    command_sha256: &str,
) -> Result<Option<CustomModuleReceipt>, ModuleError> {
    let Some(row) = row else {
        return Ok(None);
    };
    if row.action != action || row.command_sha256 != command_sha256 {
        return Err(ModuleError::Conflict);
    }
    Ok(Some(CustomModuleReceipt {
        format: "omarchygs.operator-custom-module-receipt/v1".into(),
        operation_id,
        action: row.action,
        instance_id: row.instance_id,
        module_id: row.module_id,
        release_id: row.release_id,
        lifecycle: row.resulting_lifecycle,
        lifecycle_revision: u64::try_from(row.resulting_lifecycle_revision)
            .map_err(|_| ModuleError::Internal)?,
        state_revision: u64::try_from(row.resulting_state_revision)
            .map_err(|_| ModuleError::Internal)?,
        replayed: true,
    }))
}

#[allow(clippy::too_many_arguments)]
async fn insert_custom_operation(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
    action: &str,
    command_sha256: &str,
    instance_id: Uuid,
    release_id: Uuid,
    publisher_key_sha256: &str,
    provenance_key_sha256: &str,
    requested: &[Capability],
    granted: &[Capability],
    expected_lifecycle_revision: u64,
    resulting_lifecycle: &str,
    resulting_lifecycle_revision: i64,
    resulting_state_revision: i64,
    actor: &str,
    reason: &str,
) -> Result<(), ModuleError> {
    sqlx::query(
        r#"
        INSERT INTO server_module_custom_operations (
            operation_id, action, command_sha256, instance_id, release_id,
            publisher_key_sha256, provenance_key_sha256,
            requested_capabilities, granted_capabilities, acknowledgement,
            expected_lifecycle_revision, resulting_lifecycle,
            resulting_lifecycle_revision, resulting_state_revision, actor, reason
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                  $11, $12, $13, $14, $15, $16)
        "#,
    )
    .bind(operation_id)
    .bind(action)
    .bind(command_sha256)
    .bind(instance_id)
    .bind(release_id)
    .bind(publisher_key_sha256)
    .bind(provenance_key_sha256)
    .bind(capability_strings(requested))
    .bind(capability_strings(granted))
    .bind(UNREVIEWED_ACKNOWLEDGEMENT)
    .bind(i64::try_from(expected_lifecycle_revision).map_err(|_| ModuleError::InvalidInput)?)
    .bind(resulting_lifecycle)
    .bind(resulting_lifecycle_revision)
    .bind(resulting_state_revision)
    .bind(actor)
    .bind(reason)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_lifecycle_audit(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: Uuid,
    instance_id: Uuid,
    action: &str,
    expected_revision: i64,
    previous_state: &str,
    resulting_state: &str,
    resulting_revision: i64,
    actor: &str,
    reason: &str,
) -> Result<(), ModuleError> {
    sqlx::query(
        r#"
        INSERT INTO server_module_lifecycle_audit (
            operation_id, instance_id, action, expected_revision,
            previous_state, resulting_state, resulting_revision, actor, reason
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(operation_id)
    .bind(instance_id)
    .bind(action)
    .bind(expected_revision)
    .bind(previous_state)
    .bind(resulting_state)
    .bind(resulting_revision)
    .bind(actor)
    .bind(reason)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn terminalize_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    instance_id: Uuid,
    reason: &'static str,
) -> Result<(), ModuleError> {
    let affected = sqlx::query(
        r#"
        UPDATE server_module_outbox
        SET status = 'dead_letter', lease_id = NULL, lease_expires_at = NULL,
            last_error_code = $2, delivered_at = NULL,
            dead_lettered_at = clock_timestamp(), updated_at = clock_timestamp()
        WHERE instance_id = $1 AND status NOT IN ('delivered', 'dead_letter')
        "#,
    )
    .bind(instance_id)
    .bind(reason)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?
    .rows_affected();
    if affected > 0 {
        sqlx::query(
            r#"
            UPDATE server_module_instances
            SET observation_gap_count = observation_gap_count
                    + LEAST($2, 9223372036854775807 - observation_gap_count),
                last_observation_gap_reason = $3,
                last_observation_gap_at = clock_timestamp(),
                updated_at = clock_timestamp()
            WHERE instance_id = $1
            "#,
        )
        .bind(instance_id)
        .bind(i64::try_from(affected).unwrap_or(i64::MAX))
        .bind(reason)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }
    Ok(())
}

async fn registry_lock(transaction: &mut Transaction<'_, Postgres>) -> Result<(), ModuleError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(REGISTRY_ADVISORY_LOCK)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    Ok(())
}

fn read_private_file(path: &Path, limit: usize) -> Result<Vec<u8>, ModuleError> {
    if !path.is_absolute() {
        return Err(ModuleError::InvalidInput);
    }
    let link = std::fs::symlink_metadata(path).map_err(|_| ModuleError::InvalidInput)?;
    if !trusted_metadata(&link, &link, limit) || link.file_type().is_symlink() {
        return Err(ModuleError::InvalidInput);
    }
    let mut file = File::from(
        openat2(
            CWD,
            path,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        )
        .map_err(|_| ModuleError::InvalidInput)?,
    );
    let opened = file.metadata().map_err(|_| ModuleError::InvalidInput)?;
    if !trusted_metadata(&opened, &link, limit) {
        return Err(ModuleError::InvalidInput);
    }
    let capacity = usize::try_from(opened.len()).map_err(|_| ModuleError::InvalidInput)?;
    let mut bytes = Vec::with_capacity(capacity);
    (&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ModuleError::InvalidInput)?;
    let final_metadata = file.metadata().map_err(|_| ModuleError::InvalidInput)?;
    if bytes.is_empty() || bytes.len() > limit || !trusted_metadata(&final_metadata, &opened, limit)
    {
        return Err(ModuleError::InvalidInput);
    }
    Ok(bytes)
}

fn trusted_metadata(current: &Metadata, expected: &Metadata, limit: usize) -> bool {
    current.is_file()
        && current.len() > 0
        && current.len() <= limit as u64
        && current.dev() == expected.dev()
        && current.ino() == expected.ino()
        && current.len() == expected.len()
        && current.uid() == geteuid().as_raw()
        && current.uid() == expected.uid()
        && current.gid() == expected.gid()
        && current.mode() == expected.mode()
        && current.mode() & 0o777 == 0o600
        && current.nlink() == 1
        && current.nlink() == expected.nlink()
        && current.mtime() == expected.mtime()
        && current.mtime_nsec() == expected.mtime_nsec()
        && current.ctime() == expected.ctime()
        && current.ctime_nsec() == expected.ctime_nsec()
}

fn decode_canonical<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    limit: usize,
) -> Result<T, ModuleError> {
    if bytes.is_empty() || bytes.len() > limit {
        return Err(ModuleError::InvalidInput);
    }
    let value: T = serde_json::from_slice(bytes).map_err(|_| ModuleError::InvalidInput)?;
    if canonical_json(&value).map_err(contract_failure)? != bytes {
        return Err(ModuleError::InvalidInput);
    }
    Ok(value)
}

fn decode_signed_payload<T: DeserializeOwned>(envelope: &SignedEnvelope) -> Result<T, ModuleError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(&envelope.payload)
        .map_err(|_| ModuleError::InvalidInput)?;
    serde_json::from_slice(&bytes).map_err(|_| ModuleError::InvalidInput)
}

fn decode_secret(value: &str) -> Result<[u8; 32], ModuleError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| ModuleError::InvalidConfig)?;
    let secret: [u8; 32] = bytes.try_into().map_err(|_| ModuleError::InvalidConfig)?;
    if URL_SAFE_NO_PAD.encode(secret) != value {
        return Err(ModuleError::InvalidConfig);
    }
    Ok(secret)
}

fn valid_identifier(value: &str, maximum: usize) -> bool {
    let mut bytes = value.bytes();
    !value.is_empty()
        && value.len() <= maximum
        && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_actor_reason(actor: &str, reason: &str) -> bool {
    (1..=64).contains(&actor.len())
        && actor.trim() == actor
        && !actor.chars().any(char::is_control)
        && (1..=500).contains(&reason.len())
        && reason.trim() == reason
        && !reason.chars().any(char::is_control)
}

fn contract_failure(error: impl std::fmt::Display) -> ModuleError {
    tracing::warn!(error = %error, "custom module contract rejected");
    ModuleError::InvalidInput
}

fn database_error(error: sqlx::Error) -> ModuleError {
    tracing::error!(error = %error, "custom module database operation failed");
    ModuleError::Internal
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt as _, symlink},
    };

    use super::*;

    #[test]
    fn commands_are_exact_bounded_and_action_specific() {
        let command = CustomModuleLifecycleCommand {
            format: LIFECYCLE_FORMAT.into(),
            action: CustomModuleLifecycleAction::Enable,
            operation_id: Uuid::new_v4(),
            instance_id: Uuid::new_v4(),
            expected_lifecycle_revision: 1,
            expected_config_revision: 1,
            expected_state_revision: 0,
            target_release_id: None,
            candidate_state: None,
            actor: "server-owner".into(),
            reason: "Enable reviewed local module".into(),
        };
        let bytes = canonical_json(&command).expect("command should encode");
        assert_eq!(decode_lifecycle_command(&bytes).unwrap(), command);
        let mut noncanonical = serde_json::to_vec_pretty(&command).unwrap();
        noncanonical.push(b'\n');
        assert!(decode_lifecycle_command(&noncanonical).is_err());

        let mut invalid = command.clone();
        invalid.target_release_id = Some(Uuid::new_v4());
        assert!(validate_lifecycle_command(&invalid).is_err());
        invalid.action = CustomModuleLifecycleAction::Upgrade;
        invalid.candidate_state = Some(BTreeMap::new());
        assert!(validate_lifecycle_command(&invalid).is_ok());
    }

    #[test]
    fn private_key_documents_require_canonical_exact_ed25519_seed() {
        let document = ModulePrivateKeyDocument {
            format: PRIVATE_KEY_FORMAT.into(),
            algorithm: "ed25519".into(),
            key_id: "community-module-root-v1".into(),
            signing_seed: URL_SAFE_NO_PAD.encode([7_u8; 32]),
        };
        let key = validate_private_key(&document).expect("exact seed should parse");
        assert_eq!(key.to_bytes(), [7_u8; 32]);
        let mut padded = document;
        padded.signing_seed.push('=');
        assert!(validate_private_key(&padded).is_err());
    }

    #[test]
    fn snapshot_and_grant_bounds_fail_closed() {
        assert!(validate_snapshot(&BTreeMap::new()).is_ok());
        assert!(
            validate_snapshot(&BTreeMap::from([(
                "policy".into(),
                "x".repeat(MAX_STATE_VALUE_BYTES + 1)
            )]))
            .is_err()
        );
        assert!(validate_grant(&[], &[Capability::ModerationAddLabel]).is_ok());
        assert!(
            validate_grant(
                &[Capability::ModerationAddLabel],
                &[Capability::ModerationAddLabel]
            )
            .is_ok()
        );
    }

    #[test]
    fn import_commands_require_exact_acknowledgement_and_distinct_absolute_paths() {
        let command = CustomModuleImportCommand {
            format: IMPORT_FORMAT.into(),
            operation_id: Uuid::new_v4(),
            server_id: Uuid::new_v4(),
            signed_release_path: "/private/release.json".into(),
            component_path: "/private/component.wasm".into(),
            publisher_public_key_path: "/private/publisher.json".into(),
            provenance_private_key_path: "/private/provenance.json".into(),
            publisher_key_sha256: "a".repeat(64),
            provenance_key_sha256: "b".repeat(64),
            granted_capabilities: vec![Capability::ModerationAddLabel],
            initial_config: BTreeMap::new(),
            initial_state: BTreeMap::new(),
            acknowledgement: UNREVIEWED_ACKNOWLEDGEMENT.into(),
            actor: "server-owner".into(),
            reason: "Review the exact publisher and requested grant".into(),
        };
        let bytes = canonical_json(&command).expect("import command should encode");
        assert_eq!(decode_import_command(&bytes).unwrap(), command);

        let mut bad_acknowledgement = command.clone();
        bad_acknowledgement.acknowledgement = "I accept".into();
        assert!(validate_import_command(&bad_acknowledgement).is_err());
        let mut relative = command.clone();
        relative.component_path = "component.wasm".into();
        assert!(validate_import_command(&relative).is_err());
        let mut duplicate = command;
        duplicate.component_path = duplicate.signed_release_path.clone();
        assert!(validate_import_command(&duplicate).is_err());
        assert!(decode_import_command(&vec![b'x'; MAX_CUSTOM_MODULE_COMMAND_BYTES + 1]).is_err());
    }

    #[test]
    fn private_artifact_reader_rejects_public_symlink_hardlink_and_oversize_files() {
        let directory = tempfile::tempdir().expect("private reader fixture should create");
        let exact = directory.path().join("exact.bin");
        fs::write(&exact, b"private-bytes").expect("private fixture should write");
        fs::set_permissions(&exact, fs::Permissions::from_mode(0o600))
            .expect("private fixture mode should set");
        assert_eq!(read_private_file(&exact, 64).unwrap(), b"private-bytes");

        let linked = directory.path().join("linked.bin");
        fs::hard_link(&exact, &linked).expect("hard link fixture should create");
        assert!(read_private_file(&exact, 64).is_err());
        fs::remove_file(&linked).expect("hard link fixture should remove");

        let symbolic = directory.path().join("symbolic.bin");
        symlink(&exact, &symbolic).expect("symlink fixture should create");
        assert!(read_private_file(&symbolic, 64).is_err());

        let real_parent = directory.path().join("real-parent");
        fs::create_dir(&real_parent).expect("real parent fixture should create");
        let nested_exact = real_parent.join("nested.bin");
        fs::write(&nested_exact, b"nested-private").expect("nested fixture should write");
        fs::set_permissions(&nested_exact, fs::Permissions::from_mode(0o600))
            .expect("nested fixture mode should set");
        let symbolic_parent = directory.path().join("symbolic-parent");
        symlink(&real_parent, &symbolic_parent).expect("parent symlink fixture should create");
        assert!(read_private_file(&symbolic_parent.join("nested.bin"), 64).is_err());

        fs::set_permissions(&exact, fs::Permissions::from_mode(0o640))
            .expect("public fixture mode should set");
        assert!(read_private_file(&exact, 64).is_err());

        let oversized = directory.path().join("oversized.bin");
        fs::write(&oversized, vec![0_u8; 65]).expect("oversize fixture should write");
        fs::set_permissions(&oversized, fs::Permissions::from_mode(0o600))
            .expect("oversize fixture mode should set");
        assert!(read_private_file(&oversized, 64).is_err());
    }
}
