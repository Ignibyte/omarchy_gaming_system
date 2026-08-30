//! Production contract and isolated execution boundary for OmarchyGS server modules.
//!
//! Releases may be reviewed or explicitly trusted by a server operator, but
//! every artifact crosses the same exact Component Model, capability, resource,
//! and process-containment boundary. This crate exposes no network hostcall,
//! WASI linker, database handle, server secret, or client-code surface.

use std::{
    collections::BTreeMap,
    fmt::Display,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder};

/// Maximum component and signed-document bytes accepted before processing.
pub const MAX_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;
/// Maximum canonical local-control frame accepted before allocation.
pub const MAX_FRAME_BYTES: usize = 64 * 1024;
/// Maximum guest linear memory for one invocation.
pub const MAX_LINEAR_MEMORY_BYTES: usize = 4 * 1024 * 1024;
/// Maximum deterministic Wasmtime fuel for one invocation.
pub const MAX_FUEL: u64 = 100_000;
/// Maximum outer execution deadline admitted by the v1 contract.
pub const MAX_EXECUTION_MS: u32 = 500;
/// Exact WIT package accepted by production v1.
pub const WIT_PACKAGE: &str = "ignibyte:omarchygs-server-module@1.0.0";
/// Exact WIT world accepted by production v1.
pub const WIT_WORLD: &str = "module-production";
/// Canonical release document format.
pub const RELEASE_FORMAT: &str = "omarchygs.server-module-release/v1";
/// Canonical provenance document format.
pub const PROVENANCE_FORMAT: &str = "omarchygs.server-module-provenance/v1";
/// Canonical core-admission document format.
pub const ADMISSION_FORMAT: &str = "omarchygs.server-module-admission/v1";
/// Canonical hook-event format.
pub const HOOK_FORMAT: &str = "omarchygs.server-module-hook/v1";
/// Canonical host-response format.
pub const RESPONSE_FORMAT: &str = "omarchygs.server-module-response/v1";
/// Stable reviewed module identity.
pub const BUILTIN_MODULE_ID: &str = "ignibyte.sentinel";
/// Stable reviewed release identity.
pub const BUILTIN_RELEASE_ID: Uuid = Uuid::from_u128(0x10000000000040008000000000000001);
/// Stable reviewed-fixture provenance identity.
pub const BUILTIN_REVIEW_ID: Uuid = Uuid::from_u128(0x11000000000040008000000000000001);
/// Numeric moderation label admitted by the first production slice.
pub const PRIORITY_REVIEW_LABEL: u64 = 7;

const PUBLISHER_KEY_ID: &str = "ignibyte-fixture-publisher-v1";
const REVIEW_KEY_ID: &str = "ignibyte-fixture-review-v1";
const CORE_KEY_ID: &str = "omarchygs-module-core-v1";
const HOST_READY_FORMAT: &str = "omarchygs.server-module-host-ready/v1";
const PUBLISHER_FIXTURE_SEED: [u8; 32] = [7; 32];
const REVIEW_FIXTURE_SEED: [u8; 32] = [9; 32];
const STARTUP_DEADLINE: Duration = Duration::from_secs(30);
const EXIT_DEADLINE: Duration = Duration::from_millis(150);

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "module-production",
    });
}

/// Stable production module runtime failures.
#[derive(Debug, Error)]
pub enum ModuleRuntimeError {
    /// A bounded contract or schema was invalid.
    #[error("module contract rejected: {0}")]
    Contract(String),
    /// A signature, digest, authority, or cross-document binding failed.
    #[error("module integrity rejected: {0}")]
    Integrity(String),
    /// A local control frame was invalid or exceeded its ceiling.
    #[error("module frame rejected: {0}")]
    Frame(String),
    /// Component compilation, instantiation, or execution failed.
    #[error("module execution rejected: {0}")]
    Execution(String),
    /// The OS containment boundary was unavailable or failed.
    #[error("module containment rejected: {0}")]
    Containment(String),
    /// Local process or pipe I/O failed.
    #[error("module I/O failed: {0}")]
    Io(#[from] std::io::Error),
}

/// One compiled-in component used by production or its hostile conformance corpus.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureKind {
    /// Reviewed production behavior: propose the priority-review label.
    Valid,
    /// Return no effect.
    Noop,
    /// Propose a label outside core policy.
    Unauthorized,
    /// Trap during execution.
    Trap,
    /// Consume fuel in an infinite loop.
    Loop,
    /// Declare memory beyond the admitted Store limit.
    MemoryHog,
    /// Import a forbidden WASI interface.
    ForbiddenImport,
    /// Omit the exact WIT export.
    WrongInterface,
}

impl FixtureKind {
    /// Stable CLI token for the fixed host catalog.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Noop => "noop",
            Self::Unauthorized => "unauthorized",
            Self::Trap => "trap",
            Self::Loop => "loop",
            Self::MemoryHog => "memory-hog",
            Self::ForbiddenImport => "forbidden-import",
            Self::WrongInterface => "wrong-interface",
        }
    }

    /// Parse only the fixed reviewed/conformance catalog.
    pub fn parse(value: &str) -> Result<Self, ModuleRuntimeError> {
        match value {
            "valid" => Ok(Self::Valid),
            "noop" => Ok(Self::Noop),
            "unauthorized" => Ok(Self::Unauthorized),
            "trap" => Ok(Self::Trap),
            "loop" => Ok(Self::Loop),
            "memory-hog" => Ok(Self::MemoryHog),
            "forbidden-import" => Ok(Self::ForbiddenImport),
            "wrong-interface" => Ok(Self::WrongInterface),
            _ => Err(ModuleRuntimeError::Contract(
                "unknown compiled fixture identity".into(),
            )),
        }
    }

    /// Immutable bytes embedded in the reviewed host package.
    #[must_use]
    pub fn component_bytes(self) -> &'static [u8] {
        match self {
            Self::Valid => include_bytes!(concat!(env!("OUT_DIR"), "/valid.component.wasm")),
            Self::Noop => include_bytes!(concat!(env!("OUT_DIR"), "/noop.component.wasm")),
            Self::Unauthorized => {
                include_bytes!(concat!(env!("OUT_DIR"), "/unauthorized.component.wasm"))
            }
            Self::Trap => include_bytes!(concat!(env!("OUT_DIR"), "/trap.component.wasm")),
            Self::Loop => include_bytes!(concat!(env!("OUT_DIR"), "/loop.component.wasm")),
            Self::MemoryHog => {
                include_bytes!(concat!(env!("OUT_DIR"), "/memory-hog.component.wasm"))
            }
            Self::ForbiddenImport => {
                include_bytes!(concat!(env!("OUT_DIR"), "/forbidden-import.component.wasm"))
            }
            Self::WrongInterface => {
                include_bytes!(concat!(env!("OUT_DIR"), "/wrong-interface.component.wasm"))
            }
        }
    }
}

/// Strict domain-separated signed canonical JSON.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedEnvelope {
    /// Signature domain and payload format.
    pub document_format: String,
    /// Stable authority key identity.
    pub key_id: String,
    /// Canonical payload as unpadded base64url.
    pub payload: String,
    /// Ed25519 signature as unpadded base64url.
    pub signature: String,
}

impl SignedEnvelope {
    /// Sign one bounded strict payload.
    pub fn sign<T: Serialize>(
        document_format: &str,
        key_id: &str,
        payload: &T,
        key: &SigningKey,
    ) -> Result<Self, ModuleRuntimeError> {
        validate_identifier("document format", document_format, 96)?;
        validate_identifier("key id", key_id, 96)?;
        let bytes = canonical_json(payload)?;
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ModuleRuntimeError::Contract(
                "signed payload exceeds limit".into(),
            ));
        }
        let signature = key.sign(&signature_message(document_format, &bytes));
        Ok(Self {
            document_format: document_format.to_owned(),
            key_id: key_id.to_owned(),
            payload: URL_SAFE_NO_PAD.encode(bytes),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        })
    }

    /// Verify exact domain, authority, schema, signature, and canonical bytes.
    pub fn verify<T: DeserializeOwned + Serialize>(
        &self,
        expected_format: &str,
        expected_key_id: &str,
        key: &VerifyingKey,
    ) -> Result<T, ModuleRuntimeError> {
        if self.document_format != expected_format || self.key_id != expected_key_id {
            return Err(ModuleRuntimeError::Integrity(
                "signed document authority mismatch".into(),
            ));
        }
        let payload = decode_bounded(&self.payload, MAX_ARTIFACT_BYTES, "signed payload")?;
        let signature_bytes = decode_bounded(&self.signature, 64, "signature")?;
        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|_| ModuleRuntimeError::Integrity("invalid signature length".into()))?;
        key.verify(&signature_message(expected_format, &payload), &signature)
            .map_err(|_| ModuleRuntimeError::Integrity("signature mismatch".into()))?;
        let value: T = serde_json::from_slice(&payload).map_err(|error| {
            ModuleRuntimeError::Contract(format!("invalid signed JSON: {error}"))
        })?;
        if canonical_json(&value)? != payload {
            return Err(ModuleRuntimeError::Contract(
                "signed payload is not canonical JSON".into(),
            ));
        }
        Ok(value)
    }

    /// Digest the decoded canonical payload.
    pub fn payload_sha256(&self) -> Result<String, ModuleRuntimeError> {
        Ok(sha256_hex(&decode_bounded(
            &self.payload,
            MAX_ARTIFACT_BYTES,
            "signed payload",
        )?))
    }
}

/// The one production observation hook.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    /// Observe a committed persona report.
    PersonaReported,
}

impl Display for HookKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("persona_reported")
    }
}

/// The one production module capability.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Propose a fixed moderation label that core independently authorizes.
    ModerationAddLabel,
}

impl Capability {
    const fn bit(self) -> u64 {
        match self {
            Self::ModerationAddLabel => 1,
        }
    }
}

impl Display for Capability {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("moderation_add_label")
    }
}

/// Exact Component Model interface identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WitIdentity {
    /// Versioned WIT package.
    pub package: String,
    /// Exported world.
    pub world: String,
    /// Supported incompatible interface major.
    pub major: u16,
    /// SHA-256 of the exact checked-in WIT source.
    pub sha256: String,
}

/// Core-enforced execution budgets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBudgets {
    /// Maximum local frame bytes.
    pub frame_bytes: u32,
    /// Maximum guest memory bytes.
    pub memory_bytes: u32,
    /// Maximum deterministic fuel.
    pub fuel: u64,
    /// Parent-enforced execution deadline.
    pub execution_ms: u32,
}

impl ResourceBudgets {
    fn validate(&self) -> Result<(), ModuleRuntimeError> {
        if self.frame_bytes == 0
            || self.frame_bytes as usize > MAX_FRAME_BYTES
            || self.memory_bytes == 0
            || self.memory_bytes as usize > MAX_LINEAR_MEMORY_BYTES
            || self.fuel == 0
            || self.fuel > MAX_FUEL
            || self.execution_ms == 0
            || self.execution_ms > MAX_EXECUTION_MS
        {
            return Err(ModuleRuntimeError::Contract(
                "resource budget is outside production limits".into(),
            ));
        }
        Ok(())
    }
}

/// Immutable publisher release statement.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleReleaseManifest {
    /// Exact schema format.
    pub format: String,
    /// Stable module identity.
    pub module_id: String,
    /// Stable publisher identity.
    pub publisher_id: String,
    /// Immutable release identity.
    pub release_id: Uuid,
    /// Semantic release version.
    pub version: String,
    /// Exact component digest.
    pub component_sha256: String,
    /// Exact interface identity.
    pub wit: WitIdentity,
    /// Sorted requested capabilities, not implicit grants.
    pub requested_capabilities: Vec<Capability>,
    /// Sorted requested hooks, not implicit subscriptions.
    pub subscribed_hooks: Vec<HookKind>,
    /// Requested upper execution budgets.
    pub budgets: ResourceBudgets,
    /// Configuration schema identity.
    pub config_schema: String,
    /// State schema identity.
    pub state_schema: String,
    /// Exact WIT export.
    pub entrypoint: String,
}

impl ModuleReleaseManifest {
    /// Validate the generic v1 production release contract without granting it.
    pub fn validate(&self) -> Result<(), ModuleRuntimeError> {
        if self.format != RELEASE_FORMAT || self.release_id.is_nil() || self.entrypoint != "handle"
        {
            return Err(ModuleRuntimeError::Contract(
                "release format, identity, or entrypoint is invalid".into(),
            ));
        }
        validate_lower_identifier("module id", &self.module_id, 96, false)?;
        validate_lower_identifier("publisher id", &self.publisher_id, 96, false)?;
        validate_text("release version", &self.version, 64)?;
        validate_digest(&self.component_sha256)?;
        validate_wit(&self.wit)?;
        validate_sorted_unique_allow_empty("requested capabilities", &self.requested_capabilities)?;
        validate_sorted_unique("subscribed hooks", &self.subscribed_hooks)?;
        if !is_subset(
            &self.requested_capabilities,
            &[Capability::ModerationAddLabel],
        ) || self.subscribed_hooks != [HookKind::PersonaReported]
        {
            return Err(ModuleRuntimeError::Contract(
                "release power or hook is not allowlisted".into(),
            ));
        }
        validate_lower_identifier("configuration schema", &self.config_schema, 128, true)?;
        validate_lower_identifier("state schema", &self.state_schema, 128, true)?;
        self.budgets.validate()
    }
}

/// Independent review/operator statement; it never grants a capability.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleProvenance {
    /// Exact schema format.
    pub format: String,
    /// Digest of the signed release payload.
    pub release_manifest_sha256: String,
    /// Review identity for reviewed provenance; absent for operator-custom trust.
    pub review_id: Option<Uuid>,
    /// Human-independent provenance class.
    pub class: String,
    /// Stable owner server binding required only for operator-custom trust.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_id: Option<Uuid>,
}

impl ModuleProvenance {
    /// Validate the provenance class and its mutually exclusive trust evidence.
    pub fn validate(&self) -> Result<(), ModuleRuntimeError> {
        if self.format != PROVENANCE_FORMAT {
            return Err(ModuleRuntimeError::Contract(
                "provenance format is invalid".into(),
            ));
        }
        validate_digest(&self.release_manifest_sha256)?;
        match self.class.as_str() {
            "first_party_reviewed_fixture" | "marketplace_vetted"
                if self.review_id.is_some_and(|id| !id.is_nil()) && self.server_id.is_none() =>
            {
                Ok(())
            }
            "operator_custom"
                if self.review_id.is_none()
                    && self.server_id.is_some_and(|server_id| !server_id.is_nil()) =>
            {
                Ok(())
            }
            _ => Err(ModuleRuntimeError::Contract(
                "provenance class or authority shape is invalid".into(),
            )),
        }
    }
}

/// Current core lifecycle state encoded into an exact admission.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    /// Installed but not runnable.
    Disabled,
    /// Readiness is being evaluated.
    Enabling,
    /// Exact release may receive events.
    Active,
    /// Circuit breaker has paused work.
    Degraded,
    /// Emergency/operator policy has stopped work.
    Suspended,
    /// Terminal retained tombstone.
    Retired,
}

/// Server-specific exact capability admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleAdmission {
    /// Exact schema format.
    pub format: String,
    /// Stable owner-operated server identity.
    pub server_id: Uuid,
    /// Stable instance admission identity.
    pub admission_id: Uuid,
    /// Monotonic lifecycle/admission revision.
    pub lifecycle_revision: u64,
    /// Admitted state for this signed revision.
    pub lifecycle: LifecycleStatus,
    /// Bound module identity.
    pub module_id: String,
    /// Bound release identity.
    pub release_id: Uuid,
    /// Bound component digest.
    pub component_sha256: String,
    /// Bound release payload digest.
    pub release_manifest_sha256: String,
    /// Bound provenance payload digest.
    pub provenance_sha256: String,
    /// Bound WIT identity.
    pub wit: WitIdentity,
    /// Explicit sorted capability grants.
    pub granted_capabilities: Vec<Capability>,
    /// Explicit sorted hook subscriptions.
    pub subscribed_hooks: Vec<HookKind>,
    /// Granted execution budgets.
    pub budgets: ResourceBudgets,
    /// Immutable configuration revision for one event snapshot.
    pub config_revision: u64,
    /// Bound state schema.
    pub state_schema: String,
    /// Immutable module-state revision for one event snapshot.
    pub state_revision: u64,
}

/// Opaque subject representation crossing the module boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "scope",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ModuleSubject {
    /// Purpose-specific HMAC-derived persona subject.
    Pairwise(String),
}

/// Fixed allowlisted observation payload.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HookPayload {
    /// Report metadata without reporter identity or free-form detail.
    PersonaReported {
        /// Opaque report target retained for event binding.
        report_id: Uuid,
        /// One bounded platform report category.
        category: String,
    },
}

/// Bounded exact event delivered at least once.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleHookEvent {
    /// Exact schema format.
    pub format: String,
    /// Core-owned event identity.
    pub event_id: Uuid,
    /// One-based bounded delivery attempt.
    pub attempt: u16,
    /// Stable server identity.
    pub server_id: Uuid,
    /// Stable module identity.
    pub module_id: String,
    /// Exact release identity.
    pub release_id: Uuid,
    /// Exact signed admission identity.
    pub admission_id: Uuid,
    /// Signed admission revision.
    pub admission_revision: u64,
    /// Typed hook.
    pub hook: HookKind,
    /// Core target revision at event creation.
    pub causal_revision: u64,
    /// Parent-owned deadline, not a guest clock.
    pub deadline_ms: u32,
    /// Opaque module-scoped subject.
    pub subject: ModuleSubject,
    /// Bounded immutable configuration snapshot.
    pub config: BTreeMap<String, String>,
    /// Configuration revision.
    pub config_revision: u64,
    /// Bounded immutable module-state snapshot.
    pub state: BTreeMap<String, String>,
    /// State revision.
    pub state_revision: u64,
    /// Typed payload.
    pub payload: HookPayload,
}

/// Complete local host request. Authorities are provisioned outside this frame.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostRequest {
    /// Publisher-signed release.
    pub release: SignedEnvelope,
    /// Separately signed review provenance.
    pub provenance: SignedEnvelope,
    /// Core-signed exact admission.
    pub admission: SignedEnvelope,
    /// Typed event.
    pub event: ModuleHookEvent,
}

/// The only typed proposal in this production slice.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModuleIntent {
    /// Ask core to attach a bounded label to the source report.
    ModerationAddLabel {
        /// Core target revision expected by the module.
        expected_revision: u64,
        /// Numeric allowlisted label.
        label: u64,
    },
}

impl ModuleIntent {
    /// Capability required for this proposal.
    #[must_use]
    pub const fn capability(&self) -> Capability {
        match self {
            Self::ModerationAddLabel { .. } => Capability::ModerationAddLabel,
        }
    }

    /// Revision that core must re-read and compare.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        match self {
            Self::ModerationAddLabel {
                expected_revision, ..
            } => *expected_revision,
        }
    }
}

/// Stable bounded host result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
pub enum HostResult {
    /// No effect proposed.
    Noop,
    /// One typed effect proposal.
    Proposed { intent: ModuleIntent },
    /// Stable rejection without runtime internals.
    Rejected { code: String },
}

/// Exact response context returned by the isolated host.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostResponse {
    /// Exact schema format.
    pub format: String,
    /// Source event identity.
    pub event_id: Uuid,
    /// Exact release identity.
    pub release_id: Uuid,
    /// Exact admission identity.
    pub admission_id: Uuid,
    /// Exact admission revision.
    pub admission_revision: u64,
    /// Bounded outcome.
    pub outcome: HostResult,
}

fn rejected_response(request: &HostRequest, code: &str) -> HostResponse {
    HostResponse {
        format: RESPONSE_FORMAT.into(),
        event_id: request.event.event_id,
        release_id: request.event.release_id,
        admission_id: request.event.admission_id,
        admission_revision: request.event.admission_revision,
        outcome: HostResult::Rejected { code: code.into() },
    }
}

/// Host readiness evidence emitted before the request is accepted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostReady {
    /// Exact readiness protocol.
    pub format: String,
    /// Component compiled and instantiated against the exact WIT.
    pub component_ready: bool,
    /// Sandbox exposes no home tree.
    pub home_absent: bool,
    /// Sandbox exposes no password database.
    pub passwd_absent: bool,
    /// Server secrets/configuration are absent from the host environment.
    pub server_environment_absent: bool,
    /// Network namespace has no non-loopback interface.
    pub loopback_only: bool,
    /// Host-measured resident memory after compilation.
    pub resident_kib: u64,
}

impl HostReady {
    /// Construct measured host readiness inside the sandbox.
    pub fn measured() -> Result<Self, ModuleRuntimeError> {
        Ok(Self {
            format: HOST_READY_FORMAT.into(),
            component_ready: true,
            home_absent: !Path::new("/home").exists(),
            passwd_absent: !Path::new("/etc/passwd").exists(),
            server_environment_absent: server_environment_absent(),
            loopback_only: loopback_only()?,
            resident_kib: resident_kib()?,
        })
    }

    fn validate(&self) -> Result<(), ModuleRuntimeError> {
        if self.format != HOST_READY_FORMAT
            || !self.component_ready
            || !self.home_absent
            || !self.passwd_absent
            || !self.server_environment_absent
            || !self.loopback_only
            || self.resident_kib == 0
            || self.resident_kib > 256 * 1024
        {
            return Err(ModuleRuntimeError::Containment(
                "host readiness evidence rejected".into(),
            ));
        }
        Ok(())
    }
}

/// Verified immutable release, provenance, authorities, and component material.
#[derive(Clone, Debug)]
pub struct ReviewedRelease {
    /// Signed release envelope.
    pub release: SignedEnvelope,
    /// Strict decoded release.
    pub manifest: ModuleReleaseManifest,
    /// Signed provenance envelope.
    pub provenance: SignedEnvelope,
    /// Strict decoded provenance.
    pub provenance_statement: ModuleProvenance,
    /// Exact publisher key identity provisioned outside guest frames.
    pub publisher_key_id: String,
    /// Exact publisher public key provisioned outside guest frames.
    pub publisher_public_key: VerifyingKey,
    /// Exact provenance key identity provisioned outside guest frames.
    pub provenance_key_id: String,
    /// Exact provenance public key provisioned outside guest frames.
    pub provenance_public_key: VerifyingKey,
    /// Exact immutable binary Component Model artifact.
    pub component_bytes: Vec<u8>,
}

/// Out-of-band public trust roots and provenance expectations supplied to a host.
#[derive(Clone, Debug)]
pub struct ExecutionTrust {
    /// Expected publisher key identity.
    pub publisher_key_id: String,
    /// Explicitly trusted publisher public key.
    pub publisher_public_key: VerifyingKey,
    /// Expected review/operator key identity.
    pub provenance_key_id: String,
    /// Explicitly trusted review/operator public key.
    pub provenance_public_key: VerifyingKey,
    /// Expected provenance class.
    pub provenance_class: String,
    /// Required owner-server binding for custom provenance.
    pub provenance_server_id: Option<Uuid>,
}

impl ReviewedRelease {
    /// Public trust arguments for the isolated host. These are not guest data.
    #[must_use]
    pub fn execution_trust(&self) -> ExecutionTrust {
        ExecutionTrust {
            publisher_key_id: self.publisher_key_id.clone(),
            publisher_public_key: self.publisher_public_key,
            provenance_key_id: self.provenance_key_id.clone(),
            provenance_public_key: self.provenance_public_key,
            provenance_class: self.provenance_statement.class.clone(),
            provenance_server_id: self.provenance_statement.server_id,
        }
    }
}

/// Strict decoded request facts.
#[derive(Clone, Debug)]
pub struct VerifiedRequest {
    /// Release facts.
    pub release: ModuleReleaseManifest,
    /// Review facts.
    pub provenance: ModuleProvenance,
    /// Core grant facts.
    pub admission: ModuleAdmission,
}

/// Return the exact compiled production release and review statements.
pub fn reviewed_release() -> Result<ReviewedRelease, ModuleRuntimeError> {
    reviewed_release_for(FixtureKind::Valid)
}

/// Return a fixed conformance release for one compiled-in fixture.
pub fn reviewed_release_for(kind: FixtureKind) -> Result<ReviewedRelease, ModuleRuntimeError> {
    let manifest = ModuleReleaseManifest {
        format: RELEASE_FORMAT.into(),
        module_id: BUILTIN_MODULE_ID.into(),
        publisher_id: "ignibyte".into(),
        release_id: BUILTIN_RELEASE_ID,
        version: "1.0.0".into(),
        component_sha256: sha256_hex(kind.component_bytes()),
        wit: exact_wit(),
        requested_capabilities: vec![Capability::ModerationAddLabel],
        subscribed_hooks: vec![HookKind::PersonaReported],
        budgets: default_budgets(),
        config_schema: "ignibyte.sentinel.config/v1".into(),
        state_schema: "ignibyte.sentinel.state/v1".into(),
        entrypoint: "handle".into(),
    };
    manifest.validate()?;
    let publisher = SigningKey::from_bytes(&PUBLISHER_FIXTURE_SEED);
    let release = SignedEnvelope::sign(RELEASE_FORMAT, PUBLISHER_KEY_ID, &manifest, &publisher)?;
    let provenance_statement = ModuleProvenance {
        format: PROVENANCE_FORMAT.into(),
        release_manifest_sha256: release.payload_sha256()?,
        review_id: Some(BUILTIN_REVIEW_ID),
        class: "first_party_reviewed_fixture".into(),
        server_id: None,
    };
    let reviewer = SigningKey::from_bytes(&REVIEW_FIXTURE_SEED);
    let provenance = SignedEnvelope::sign(
        PROVENANCE_FORMAT,
        REVIEW_KEY_ID,
        &provenance_statement,
        &reviewer,
    )?;
    Ok(ReviewedRelease {
        release,
        manifest,
        provenance,
        provenance_statement,
        publisher_key_id: PUBLISHER_KEY_ID.into(),
        publisher_public_key: publisher.verifying_key(),
        provenance_key_id: REVIEW_KEY_ID.into(),
        provenance_public_key: reviewer.verifying_key(),
        component_bytes: kind.component_bytes().to_vec(),
    })
}

/// Verify and assemble one exact release from independently supplied trust roots.
pub fn verify_release_material(
    release: SignedEnvelope,
    provenance: SignedEnvelope,
    trust: &ExecutionTrust,
    component_bytes: Vec<u8>,
) -> Result<ReviewedRelease, ModuleRuntimeError> {
    validate_lower_identifier("publisher key id", &trust.publisher_key_id, 96, false)?;
    validate_lower_identifier("provenance key id", &trust.provenance_key_id, 96, false)?;
    if component_bytes.len() < 8
        || component_bytes.len() > MAX_ARTIFACT_BYTES
        || !component_bytes.starts_with(b"\0asm")
    {
        return Err(ModuleRuntimeError::Contract(
            "component bytes are not a bounded binary component".into(),
        ));
    }
    let manifest: ModuleReleaseManifest = release.verify(
        RELEASE_FORMAT,
        &trust.publisher_key_id,
        &trust.publisher_public_key,
    )?;
    manifest.validate()?;
    let provenance_statement: ModuleProvenance = provenance.verify(
        PROVENANCE_FORMAT,
        &trust.provenance_key_id,
        &trust.provenance_public_key,
    )?;
    provenance_statement.validate()?;
    if provenance_statement.class != trust.provenance_class
        || provenance_statement.server_id != trust.provenance_server_id
        || provenance_statement.release_manifest_sha256 != release.payload_sha256()?
        || manifest.component_sha256 != sha256_hex(&component_bytes)
    {
        return Err(ModuleRuntimeError::Integrity(
            "release, provenance, server, or component binding mismatch".into(),
        ));
    }
    Ok(ReviewedRelease {
        release,
        manifest,
        provenance,
        provenance_statement,
        publisher_key_id: trust.publisher_key_id.clone(),
        publisher_public_key: trust.publisher_public_key,
        provenance_key_id: trust.provenance_key_id.clone(),
        provenance_public_key: trust.provenance_public_key,
        component_bytes,
    })
}

/// Sign server-bound operator-custom provenance after publisher verification.
pub fn sign_operator_custom_provenance(
    release: &SignedEnvelope,
    server_id: Uuid,
    provenance_key_id: &str,
    provenance_key: &SigningKey,
) -> Result<(ModuleProvenance, SignedEnvelope), ModuleRuntimeError> {
    if server_id.is_nil() {
        return Err(ModuleRuntimeError::Contract(
            "operator-custom provenance requires a server identity".into(),
        ));
    }
    let statement = ModuleProvenance {
        format: PROVENANCE_FORMAT.into(),
        release_manifest_sha256: release.payload_sha256()?,
        review_id: None,
        class: "operator_custom".into(),
        server_id: Some(server_id),
    };
    statement.validate()?;
    let envelope = SignedEnvelope::sign(
        PROVENANCE_FORMAT,
        provenance_key_id,
        &statement,
        provenance_key,
    )?;
    Ok((statement, envelope))
}

/// Create one server-specific active signed admission for an explicit grant.
pub fn sign_active_admission(
    reviewed: &ReviewedRelease,
    server_id: Uuid,
    admission_id: Uuid,
    lifecycle_revision: u64,
    config_revision: u64,
    state_revision: u64,
    core_key: &SigningKey,
) -> Result<(ModuleAdmission, SignedEnvelope), ModuleRuntimeError> {
    sign_active_admission_with_grants(
        reviewed,
        AdmissionCoordinates {
            server_id,
            admission_id,
            lifecycle_revision,
            config_revision,
            state_revision,
        },
        vec![Capability::ModerationAddLabel],
        vec![HookKind::PersonaReported],
        core_key,
    )
}

/// Create a server-specific admission for an explicit reviewed grant subset.
pub fn sign_active_admission_with_grants(
    reviewed: &ReviewedRelease,
    coordinates: AdmissionCoordinates,
    granted_capabilities: Vec<Capability>,
    subscribed_hooks: Vec<HookKind>,
    core_key: &SigningKey,
) -> Result<(ModuleAdmission, SignedEnvelope), ModuleRuntimeError> {
    let AdmissionCoordinates {
        server_id,
        admission_id,
        lifecycle_revision,
        config_revision,
        state_revision,
    } = coordinates;
    if server_id.is_nil()
        || admission_id.is_nil()
        || lifecycle_revision == 0
        || config_revision == 0
    {
        return Err(ModuleRuntimeError::Contract(
            "invalid admission identity or revision".into(),
        ));
    }
    verify_release_material(
        reviewed.release.clone(),
        reviewed.provenance.clone(),
        &reviewed.execution_trust(),
        reviewed.component_bytes.clone(),
    )?;
    validate_sorted_unique_allow_empty("granted capabilities", &granted_capabilities)?;
    validate_sorted_unique("subscribed hooks", &subscribed_hooks)?;
    if !is_subset(
        &granted_capabilities,
        &reviewed.manifest.requested_capabilities,
    ) || !is_subset(&subscribed_hooks, &reviewed.manifest.subscribed_hooks)
    {
        return Err(ModuleRuntimeError::Contract(
            "admission grant exceeds the release request".into(),
        ));
    }
    let admission = ModuleAdmission {
        format: ADMISSION_FORMAT.into(),
        server_id,
        admission_id,
        lifecycle_revision,
        lifecycle: LifecycleStatus::Active,
        module_id: reviewed.manifest.module_id.clone(),
        release_id: reviewed.manifest.release_id,
        component_sha256: reviewed.manifest.component_sha256.clone(),
        release_manifest_sha256: reviewed.release.payload_sha256()?,
        provenance_sha256: reviewed.provenance.payload_sha256()?,
        wit: reviewed.manifest.wit.clone(),
        granted_capabilities,
        subscribed_hooks,
        budgets: reviewed.manifest.budgets.clone(),
        config_revision,
        state_schema: reviewed.manifest.state_schema.clone(),
        state_revision,
    };
    validate_admission(&admission)?;
    let envelope = SignedEnvelope::sign(ADMISSION_FORMAT, CORE_KEY_ID, &admission, core_key)?;
    Ok((admission, envelope))
}

/// Admission coordinates varied only by the fixed hostile conformance corpus.
#[doc(hidden)]
pub struct AdmissionCoordinates {
    pub server_id: Uuid,
    pub admission_id: Uuid,
    pub lifecycle_revision: u64,
    pub config_revision: u64,
    pub state_revision: u64,
}

/// Create a signed admission for one fixed compiled conformance fixture.
#[doc(hidden)]
pub fn sign_active_admission_for(
    reviewed: &ReviewedRelease,
    kind: FixtureKind,
    coordinates: AdmissionCoordinates,
    core_key: &SigningKey,
) -> Result<(ModuleAdmission, SignedEnvelope), ModuleRuntimeError> {
    verify_reviewed_release(reviewed, kind)?;
    sign_active_admission_with_grants(
        reviewed,
        coordinates,
        vec![Capability::ModerationAddLabel],
        vec![HookKind::PersonaReported],
        core_key,
    )
}

/// Build a complete host request from core-owned persisted facts.
pub fn host_request(
    reviewed: &ReviewedRelease,
    admission: SignedEnvelope,
    event: ModuleHookEvent,
) -> HostRequest {
    HostRequest {
        release: reviewed.release.clone(),
        provenance: reviewed.provenance.clone(),
        admission,
        event,
    }
}

/// Verify the complete request against out-of-band core authority and selected bytes.
pub fn verify_host_request(
    request: &HostRequest,
    core_key: &VerifyingKey,
    kind: FixtureKind,
) -> Result<VerifiedRequest, ModuleRuntimeError> {
    let reviewed = reviewed_release_for(kind)?;
    verify_host_request_with_release(request, core_key, &reviewed)
}

/// Verify a complete request against an exact independently trusted release.
pub fn verify_host_request_with_release(
    request: &HostRequest,
    core_key: &VerifyingKey,
    reviewed: &ReviewedRelease,
) -> Result<VerifiedRequest, ModuleRuntimeError> {
    let release: ModuleReleaseManifest = request.release.verify(
        RELEASE_FORMAT,
        &reviewed.publisher_key_id,
        &reviewed.publisher_public_key,
    )?;
    release.validate()?;
    let provenance: ModuleProvenance = request.provenance.verify(
        PROVENANCE_FORMAT,
        &reviewed.provenance_key_id,
        &reviewed.provenance_public_key,
    )?;
    let admission: ModuleAdmission =
        request
            .admission
            .verify(ADMISSION_FORMAT, CORE_KEY_ID, core_key)?;
    provenance.validate()?;
    validate_admission(&admission)?;

    let release_sha = request.release.payload_sha256()?;
    let provenance_sha = request.provenance.payload_sha256()?;
    let component_sha = sha256_hex(&reviewed.component_bytes);
    if request.release != reviewed.release
        || request.provenance != reviewed.provenance
        || release != reviewed.manifest
        || provenance != reviewed.provenance_statement
        || release.component_sha256 != component_sha
        || provenance.release_manifest_sha256 != release_sha
        || admission.module_id != release.module_id
        || admission.release_id != release.release_id
        || admission.component_sha256 != component_sha
        || admission.release_manifest_sha256 != release_sha
        || admission.provenance_sha256 != provenance_sha
        || admission.wit != release.wit
        || admission.state_schema != release.state_schema
        || !is_subset(
            &admission.granted_capabilities,
            &release.requested_capabilities,
        )
        || !is_subset(&admission.subscribed_hooks, &release.subscribed_hooks)
        || admission.budgets.frame_bytes > release.budgets.frame_bytes
        || admission.budgets.memory_bytes > release.budgets.memory_bytes
        || admission.budgets.fuel > release.budgets.fuel
        || admission.budgets.execution_ms > release.budgets.execution_ms
    {
        return Err(ModuleRuntimeError::Integrity(
            "release, provenance, admission, component, or WIT binding mismatch".into(),
        ));
    }
    validate_event(&request.event, &admission)?;
    Ok(VerifiedRequest {
        release,
        provenance,
        admission,
    })
}

/// Pinned Wasmtime component runtime with no imports/WASI and a fresh Store per call.
pub struct ModuleRuntime {
    engine: Engine,
    component: Component,
    component_sha256: String,
    fixture_kind: Option<FixtureKind>,
}

struct StoreState {
    limits: StoreLimits,
}

impl ModuleRuntime {
    /// Compile one immutable compiled-in component.
    pub fn compile(kind: FixtureKind) -> Result<Self, ModuleRuntimeError> {
        let mut runtime = Self::compile_bytes(kind.component_bytes())?;
        runtime.fixture_kind = Some(kind);
        Ok(runtime)
    }

    /// Compile one exact bounded binary component with no imports linked.
    pub fn compile_bytes(component_bytes: &[u8]) -> Result<Self, ModuleRuntimeError> {
        if component_bytes.is_empty()
            || component_bytes.len() > MAX_ARTIFACT_BYTES
            || !component_bytes.starts_with(b"\0asm")
        {
            return Err(ModuleRuntimeError::Contract(
                "component bytes are not a bounded binary component".into(),
            ));
        }
        let mut config = Config::new();
        config
            .wasm_component_model(true)
            .consume_fuel(true)
            .wasm_memory64(false);
        let engine = Engine::new(&config).map_err(|error| {
            ModuleRuntimeError::Execution(format!("engine setup failed: {error}"))
        })?;
        let component = Component::new(&engine, component_bytes).map_err(|error| {
            ModuleRuntimeError::Execution(format!("component compilation failed: {error:#}"))
        })?;
        Ok(Self {
            engine,
            component,
            component_sha256: sha256_hex(component_bytes),
            fixture_kind: None,
        })
    }

    /// Prove exact WIT implementation and initial memory limits.
    pub fn readiness(&self) -> Result<(), ModuleRuntimeError> {
        let mut store = self.fresh_store(MAX_FUEL, MAX_LINEAR_MEMORY_BYTES)?;
        let linker = Linker::new(&self.engine);
        bindings::ModuleProduction::instantiate(&mut store, &self.component, &linker).map_err(
            |error| ModuleRuntimeError::Execution(format!("component readiness failed: {error:#}")),
        )?;
        Ok(())
    }

    /// Verify and execute one event, returning only stable bounded outcomes.
    #[must_use]
    pub fn execute(&self, request: &HostRequest, core_key: &VerifyingKey) -> HostResponse {
        let reviewed = match self
            .fixture_kind
            .and_then(|kind| reviewed_release_for(kind).ok())
        {
            Some(reviewed) if reviewed.manifest.component_sha256 == self.component_sha256 => {
                reviewed
            }
            _ => return rejected_response(request, "request_rejected"),
        };
        self.execute_release(request, core_key, &reviewed)
    }

    /// Verify and execute using exact out-of-band release trust material.
    #[must_use]
    pub fn execute_release(
        &self,
        request: &HostRequest,
        core_key: &VerifyingKey,
        reviewed: &ReviewedRelease,
    ) -> HostResponse {
        let rejected = |code: &str| HostResponse {
            format: RESPONSE_FORMAT.into(),
            event_id: request.event.event_id,
            release_id: request.event.release_id,
            admission_id: request.event.admission_id,
            admission_revision: request.event.admission_revision,
            outcome: HostResult::Rejected { code: code.into() },
        };
        if sha256_hex(&reviewed.component_bytes) != self.component_sha256 {
            return rejected("request_rejected");
        }
        let verified = match verify_host_request_with_release(request, core_key, reviewed) {
            Ok(verified) => verified,
            Err(_) => return rejected("request_rejected"),
        };
        let mut store = match self.fresh_store(
            verified.admission.budgets.fuel,
            verified.admission.budgets.memory_bytes as usize,
        ) {
            Ok(store) => store,
            Err(_) => return rejected("runtime_limit_rejected"),
        };
        let linker = Linker::new(&self.engine);
        let bindings =
            match bindings::ModuleProduction::instantiate(&mut store, &self.component, &linker) {
                Ok(bindings) => bindings,
                Err(_) => return rejected("module_instantiation_failed"),
            };
        let granted = verified
            .admission
            .granted_capabilities
            .iter()
            .fold(0_u64, |bits, capability| bits | capability.bit());
        let raw = match bindings.call_handle(
            &mut store,
            bindings::HookEvent {
                kind: 1,
                revision: request.event.causal_revision,
                granted_capabilities: granted,
            },
        ) {
            Ok(intent) => intent,
            Err(_) => return rejected("module_execution_failed"),
        };
        let outcome = match raw.kind {
            0 => HostResult::Noop,
            1 if raw.value <= 100 => HostResult::Proposed {
                intent: ModuleIntent::ModerationAddLabel {
                    expected_revision: raw.expected_revision,
                    label: raw.value,
                },
            },
            1 => return rejected("intent_outside_policy"),
            _ => return rejected("unknown_intent"),
        };
        if let HostResult::Proposed { intent } = &outcome
            && (!verified
                .admission
                .granted_capabilities
                .contains(&intent.capability())
                || intent.expected_revision() != request.event.causal_revision)
        {
            return rejected("intent_not_granted");
        }
        HostResponse {
            format: RESPONSE_FORMAT.into(),
            event_id: request.event.event_id,
            release_id: request.event.release_id,
            admission_id: request.event.admission_id,
            admission_revision: request.event.admission_revision,
            outcome,
        }
    }

    fn fresh_store(
        &self,
        fuel: u64,
        memory: usize,
    ) -> Result<Store<StoreState>, ModuleRuntimeError> {
        let limits = StoreLimitsBuilder::new()
            .memory_size(memory)
            .instances(1)
            .memories(1)
            .tables(1)
            .build();
        let mut store = Store::new(&self.engine, StoreState { limits });
        store.limiter(|state| &mut state.limits);
        store.set_fuel(fuel).map_err(|error| {
            ModuleRuntimeError::Execution(format!("fuel setup failed: {error}"))
        })?;
        Ok(store)
    }
}

/// Result of one fresh contained host invocation.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReport {
    /// Applied containment profile.
    pub containment: &'static str,
    /// Startup/readiness duration.
    pub startup_ms: u128,
    /// Request execution duration.
    pub execution_ms: u128,
    /// Measured host readiness.
    pub ready: HostReady,
    /// Bounded response.
    pub response: HostResponse,
}

/// Core-side launcher for the sibling packaged host binary.
#[derive(Clone, Debug)]
pub struct ProcessSupervisor {
    host_path: PathBuf,
}

impl ProcessSupervisor {
    /// Resolve the reviewed host only as a sibling of the running server binary.
    pub fn packaged_sibling() -> Result<Self, ModuleRuntimeError> {
        let executable = std::env::current_exe()?;
        let parent = executable.parent().ok_or_else(|| {
            ModuleRuntimeError::Containment("server executable has no parent".into())
        })?;
        Self::reviewed_path(&parent.join("omarchygs-module-host"))
    }

    /// Construct from an explicit reviewed absolute path for local conformance only.
    pub fn reviewed_path(path: &Path) -> Result<Self, ModuleRuntimeError> {
        let path = path.canonicalize()?;
        if !path.is_absolute() || !path.is_file() {
            return Err(ModuleRuntimeError::Containment(
                "reviewed module host path is invalid".into(),
            ));
        }
        Ok(Self { host_path: path })
    }

    /// Execute the production fixture under the full containment profile.
    pub fn execute(
        &self,
        request: &HostRequest,
        core_key: &VerifyingKey,
    ) -> Result<ExecutionReport, ModuleRuntimeError> {
        let reviewed = reviewed_release()?;
        self.execute_release(request, core_key, &reviewed)
    }

    /// Execute one exact verified release through a core-created private artifact.
    pub fn execute_release(
        &self,
        request: &HostRequest,
        core_key: &VerifyingKey,
        reviewed: &ReviewedRelease,
    ) -> Result<ExecutionReport, ModuleRuntimeError> {
        self.execute_release_with_failure(request, core_key, reviewed, None)
    }

    /// Execute one fixed hostile fixture for deterministic conformance.
    pub fn execute_fixture(
        &self,
        request: &HostRequest,
        core_key: &VerifyingKey,
        kind: FixtureKind,
        failure: Option<&str>,
    ) -> Result<ExecutionReport, ModuleRuntimeError> {
        let reviewed = reviewed_release_for(kind)?;
        self.execute_release_with_failure(request, core_key, &reviewed, failure)
    }

    fn execute_release_with_failure(
        &self,
        request: &HostRequest,
        core_key: &VerifyingKey,
        reviewed: &ReviewedRelease,
        failure: Option<&str>,
    ) -> Result<ExecutionReport, ModuleRuntimeError> {
        if !systemd_user_available() {
            return Err(ModuleRuntimeError::Containment(
                "systemd user scope is required for independent host limits".into(),
            ));
        }
        verify_host_request_with_release(request, core_key, reviewed)?;
        let mut artifact = tempfile::Builder::new()
            .prefix("omarchygs-module-")
            .suffix(".component.wasm")
            .tempfile()?;
        artifact
            .as_file_mut()
            .write_all(&reviewed.component_bytes)?;
        artifact.as_file_mut().sync_all()?;
        let started = Instant::now();
        let mut child = spawn_host(
            &self.host_path,
            artifact.path(),
            core_key,
            &reviewed.execution_trust(),
            failure,
        )?;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            ModuleRuntimeError::Containment("module host stdin is unavailable".into())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            ModuleRuntimeError::Containment("module host stdout is unavailable".into())
        })?;
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let ready = read_frame::<HostReady, _>(&mut reader);
            let ready_ok = ready.is_ok();
            if sender.send(HostMessage::Ready(ready)).is_err() || !ready_ok {
                return;
            }
            let response = read_frame::<HostResponse, _>(&mut reader);
            let _ = sender.send(HostMessage::Response(response));
        });
        let ready = match receiver.recv_timeout(STARTUP_DEADLINE) {
            Ok(HostMessage::Ready(Ok(ready))) => ready,
            Ok(HostMessage::Ready(Err(error))) => {
                terminate(&mut child);
                return Err(error);
            }
            Ok(HostMessage::Response(_)) => {
                terminate(&mut child);
                return Err(ModuleRuntimeError::Containment(
                    "host responded before readiness".into(),
                ));
            }
            Err(_) => {
                terminate(&mut child);
                return Err(ModuleRuntimeError::Containment(
                    "host startup deadline exceeded".into(),
                ));
            }
        };
        if let Err(error) = ready.validate() {
            terminate(&mut child);
            return Err(error);
        }
        let startup_ms = started.elapsed().as_millis();
        write_frame(&mut stdin, request)?;
        drop(stdin);
        let execution_started = Instant::now();
        let execution_deadline = Duration::from_millis(u64::from(request.event.deadline_ms));
        let response = match receiver.recv_timeout(execution_deadline) {
            Ok(HostMessage::Response(Ok(response))) => response,
            Ok(HostMessage::Response(Err(error))) => {
                terminate(&mut child);
                return Err(error);
            }
            Ok(HostMessage::Ready(_)) => {
                terminate(&mut child);
                return Err(ModuleRuntimeError::Execution(
                    "duplicate host readiness frame".into(),
                ));
            }
            Err(_) => {
                terminate(&mut child);
                return Err(ModuleRuntimeError::Execution(
                    "host execution deadline exceeded".into(),
                ));
            }
        };
        let execution_ms = execution_started.elapsed().as_millis();
        let status = wait_for_exit(&mut child, EXIT_DEADLINE)?;
        if !status.success() {
            return Err(ModuleRuntimeError::Execution(
                "host exited unsuccessfully".into(),
            ));
        }
        Ok(ExecutionReport {
            containment: "systemd-user-scope+bubblewrap+prlimit",
            startup_ms,
            execution_ms,
            ready,
            response,
        })
    }
}

enum HostMessage {
    Ready(Result<HostReady, ModuleRuntimeError>),
    Response(Result<HostResponse, ModuleRuntimeError>),
}

/// Encode a verifying key for out-of-band host provisioning.
#[must_use]
pub fn encode_verifying_key(key: &VerifyingKey) -> String {
    URL_SAFE_NO_PAD.encode(key.as_bytes())
}

/// Decode an exact canonical Ed25519 verifying key.
pub fn decode_verifying_key(value: &str) -> Result<VerifyingKey, ModuleRuntimeError> {
    let bytes = decode_bounded(value, 32, "public key")?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| ModuleRuntimeError::Integrity("invalid public key length".into()))?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|_| ModuleRuntimeError::Integrity("invalid public key".into()))
}

/// Lowercase SHA-256 fingerprint of an exact Ed25519 verifying key.
#[must_use]
pub fn verifying_key_sha256(key: &VerifyingKey) -> String {
    sha256_hex(key.as_bytes())
}

/// Canonical JSON bytes for persistence and hashing.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ModuleRuntimeError> {
    serde_json::to_vec(value).map_err(|error| {
        ModuleRuntimeError::Contract(format!("JSON serialization failed: {error}"))
    })
}

/// Lowercase SHA-256 helper.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Write one canonical bounded frame.
pub fn write_frame<T: Serialize, W: Write>(
    writer: &mut W,
    value: &T,
) -> Result<(), ModuleRuntimeError> {
    let payload = canonical_json(value)?;
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err(ModuleRuntimeError::Frame(
            "outbound frame exceeds limit".into(),
        ));
    }
    let length = u32::try_from(payload.len())
        .map_err(|_| ModuleRuntimeError::Frame("outbound frame length overflow".into()))?;
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

/// Read one strict canonical frame, bounding its declared size before allocation.
pub fn read_frame<T: DeserializeOwned + Serialize, R: Read>(
    reader: &mut R,
) -> Result<T, ModuleRuntimeError> {
    let mut header = [0_u8; 4];
    reader.read_exact(&mut header)?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(ModuleRuntimeError::Frame(
            "declared frame length rejected".into(),
        ));
    }
    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload)?;
    let value: T = serde_json::from_slice(&payload)
        .map_err(|error| ModuleRuntimeError::Frame(format!("invalid frame JSON: {error}")))?;
    if canonical_json(&value)? != payload {
        return Err(ModuleRuntimeError::Frame(
            "frame JSON is not canonical".into(),
        ));
    }
    Ok(value)
}

/// SHA-256 of the exact WIT source.
#[must_use]
pub fn wit_sha256() -> String {
    sha256_hex(include_bytes!("../wit/omarchygs-module.wit"))
}

fn default_budgets() -> ResourceBudgets {
    ResourceBudgets {
        frame_bytes: MAX_FRAME_BYTES as u32,
        memory_bytes: MAX_LINEAR_MEMORY_BYTES as u32,
        fuel: MAX_FUEL,
        execution_ms: MAX_EXECUTION_MS,
    }
}

fn exact_wit() -> WitIdentity {
    WitIdentity {
        package: WIT_PACKAGE.into(),
        world: WIT_WORLD.into(),
        major: 1,
        sha256: wit_sha256(),
    }
}

fn verify_reviewed_release(
    reviewed: &ReviewedRelease,
    kind: FixtureKind,
) -> Result<(), ModuleRuntimeError> {
    let verified = verify_release_material(
        reviewed.release.clone(),
        reviewed.provenance.clone(),
        &reviewed.execution_trust(),
        reviewed.component_bytes.clone(),
    )?;
    if verified.manifest != reviewed.manifest
        || verified.provenance_statement != reviewed.provenance_statement
        || reviewed.component_bytes != kind.component_bytes()
    {
        return Err(ModuleRuntimeError::Integrity(
            "reviewed release material mismatch".into(),
        ));
    }
    Ok(())
}

fn validate_admission(value: &ModuleAdmission) -> Result<(), ModuleRuntimeError> {
    if value.format != ADMISSION_FORMAT
        || value.server_id.is_nil()
        || value.admission_id.is_nil()
        || value.lifecycle_revision == 0
        || value.lifecycle != LifecycleStatus::Active
        || value.release_id.is_nil()
        || value.config_revision == 0
    {
        return Err(ModuleRuntimeError::Contract(
            "admission shape is invalid".into(),
        ));
    }
    validate_digest(&value.component_sha256)?;
    validate_digest(&value.release_manifest_sha256)?;
    validate_digest(&value.provenance_sha256)?;
    validate_wit(&value.wit)?;
    validate_lower_identifier("admitted module id", &value.module_id, 96, false)?;
    validate_lower_identifier("admitted state schema", &value.state_schema, 128, true)?;
    validate_sorted_unique_allow_empty("granted capabilities", &value.granted_capabilities)?;
    validate_sorted_unique("subscribed hooks", &value.subscribed_hooks)?;
    if !is_subset(
        &value.granted_capabilities,
        &[Capability::ModerationAddLabel],
    ) || value.subscribed_hooks != [HookKind::PersonaReported]
    {
        return Err(ModuleRuntimeError::Contract(
            "admission exceeds the production grant".into(),
        ));
    }
    value.budgets.validate()
}

fn validate_event(
    event: &ModuleHookEvent,
    admission: &ModuleAdmission,
) -> Result<(), ModuleRuntimeError> {
    if event.format != HOOK_FORMAT
        || event.event_id.is_nil()
        || event.attempt == 0
        || event.attempt > 8
        || event.server_id != admission.server_id
        || event.module_id != admission.module_id
        || event.release_id != admission.release_id
        || event.admission_id != admission.admission_id
        || event.admission_revision != admission.lifecycle_revision
        || event.hook != HookKind::PersonaReported
        || event.deadline_ms == 0
        || event.deadline_ms > admission.budgets.execution_ms
        || event.config_revision != admission.config_revision
        || event.state_revision != admission.state_revision
    {
        return Err(ModuleRuntimeError::Integrity(
            "event and admission context mismatch".into(),
        ));
    }
    if !admission.subscribed_hooks.contains(&event.hook) {
        return Err(ModuleRuntimeError::Integrity("hook is not admitted".into()));
    }
    let ModuleSubject::Pairwise(subject) = &event.subject;
    validate_identifier("pairwise subject", subject, 96)?;
    validate_snapshot("configuration", &event.config, 32, 4096)?;
    validate_snapshot("state", &event.state, 32, 4096)?;
    let HookPayload::PersonaReported {
        report_id,
        category,
    } = &event.payload;
    if report_id.is_nil()
        || !matches!(
            category.as_str(),
            "harassment" | "spam" | "cheating" | "other"
        )
    {
        return Err(ModuleRuntimeError::Contract(
            "report hook payload is invalid".into(),
        ));
    }
    Ok(())
}

fn validate_wit(value: &WitIdentity) -> Result<(), ModuleRuntimeError> {
    if value.package != WIT_PACKAGE
        || value.world != WIT_WORLD
        || value.major != 1
        || value.sha256 != wit_sha256()
    {
        return Err(ModuleRuntimeError::Integrity(
            "unsupported WIT identity".into(),
        ));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), ModuleRuntimeError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ModuleRuntimeError::Contract(
            "invalid lowercase SHA-256".into(),
        ));
    }
    Ok(())
}

fn validate_identifier(name: &str, value: &str, max: usize) -> Result<(), ModuleRuntimeError> {
    if value.is_empty()
        || value.len() > max
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
    {
        return Err(ModuleRuntimeError::Contract(format!("invalid {name}")));
    }
    Ok(())
}

fn validate_lower_identifier(
    name: &str,
    value: &str,
    max: usize,
    slash_allowed: bool,
) -> Result<(), ModuleRuntimeError> {
    let mut bytes = value.bytes();
    if value.len() > max
        || !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b'_')
                || (slash_allowed && byte == b'/')
        })
    {
        return Err(ModuleRuntimeError::Contract(format!("invalid {name}")));
    }
    Ok(())
}

fn validate_text(name: &str, value: &str, max: usize) -> Result<(), ModuleRuntimeError> {
    if value.is_empty()
        || value.len() > max
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(ModuleRuntimeError::Contract(format!("invalid {name}")));
    }
    Ok(())
}

fn validate_sorted_unique<T: Ord + Display>(
    name: &str,
    values: &[T],
) -> Result<(), ModuleRuntimeError> {
    if values.is_empty() || values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ModuleRuntimeError::Contract(format!(
            "{name} are empty or not sorted/unique"
        )));
    }
    Ok(())
}

fn validate_sorted_unique_allow_empty<T: Ord + Display>(
    name: &str,
    values: &[T],
) -> Result<(), ModuleRuntimeError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ModuleRuntimeError::Contract(format!(
            "{name} are not sorted/unique"
        )));
    }
    Ok(())
}

fn validate_snapshot(
    name: &str,
    values: &BTreeMap<String, String>,
    max_entries: usize,
    max_bytes: usize,
) -> Result<(), ModuleRuntimeError> {
    if values.len() > max_entries {
        return Err(ModuleRuntimeError::Contract(format!(
            "{name} has too many entries"
        )));
    }
    let mut total = 0_usize;
    for (key, value) in values {
        validate_identifier(&format!("{name} key"), key, 64)?;
        if value.len() > 512 || value.chars().any(char::is_control) {
            return Err(ModuleRuntimeError::Contract(format!(
                "invalid {name} value"
            )));
        }
        total = total.saturating_add(key.len()).saturating_add(value.len());
    }
    if total > max_bytes {
        return Err(ModuleRuntimeError::Contract(format!(
            "{name} exceeds byte quota"
        )));
    }
    Ok(())
}

fn is_subset<T: Ord>(subset: &[T], superset: &[T]) -> bool {
    subset
        .iter()
        .all(|item| superset.binary_search(item).is_ok())
}

fn signature_message(format: &str, payload: &[u8]) -> Vec<u8> {
    let mut message = b"OmarchyGS server module signed document\0".to_vec();
    message.extend_from_slice(format.as_bytes());
    message.push(0);
    message.extend_from_slice(payload);
    message
}

fn decode_bounded(value: &str, max: usize, name: &str) -> Result<Vec<u8>, ModuleRuntimeError> {
    if value.len() > max.saturating_mul(2).saturating_add(8) {
        return Err(ModuleRuntimeError::Contract(format!(
            "{name} encoding exceeds limit"
        )));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| ModuleRuntimeError::Contract(format!("invalid {name} base64url")))?;
    if bytes.len() > max || URL_SAFE_NO_PAD.encode(&bytes) != value {
        return Err(ModuleRuntimeError::Contract(format!(
            "{name} is not canonical or exceeds limit"
        )));
    }
    Ok(bytes)
}

fn spawn_host(
    host_path: &Path,
    component_path: &Path,
    core_key: &VerifyingKey,
    trust: &ExecutionTrust,
    failure: Option<&str>,
) -> Result<Child, ModuleRuntimeError> {
    let component_path = component_path.canonicalize()?;
    if !component_path.is_absolute() || !component_path.is_file() {
        return Err(ModuleRuntimeError::Containment(
            "private module artifact path is invalid".into(),
        ));
    }
    let mut sandbox_args = vec![
        "--unshare-all".to_owned(),
        "--die-with-parent".to_owned(),
        "--new-session".to_owned(),
        "--clearenv".to_owned(),
        "--cap-drop".to_owned(),
        "ALL".to_owned(),
        "--ro-bind".to_owned(),
        "/usr".to_owned(),
        "/usr".to_owned(),
        "--symlink".to_owned(),
        "usr/lib".to_owned(),
        "/lib".to_owned(),
        "--symlink".to_owned(),
        "usr/lib".to_owned(),
        "/lib64".to_owned(),
        "--proc".to_owned(),
        "/proc".to_owned(),
        "--dev".to_owned(),
        "/dev".to_owned(),
        "--tmpfs".to_owned(),
        "/tmp".to_owned(),
        "--dir".to_owned(),
        "/app".to_owned(),
        "--dir".to_owned(),
        "/module".to_owned(),
        "--ro-bind".to_owned(),
        host_path.to_string_lossy().into_owned(),
        "/app/omarchygs-module-host".to_owned(),
        "--ro-bind".to_owned(),
        component_path.to_string_lossy().into_owned(),
        "/module/component.wasm".to_owned(),
        "--setenv".to_owned(),
        "PATH".to_owned(),
        "/usr/bin".to_owned(),
        "--".to_owned(),
        "/app/omarchygs-module-host".to_owned(),
        "--component".to_owned(),
        "/module/component.wasm".to_owned(),
        "--publisher-key-id".to_owned(),
        trust.publisher_key_id.clone(),
        "--publisher-public-key".to_owned(),
        encode_verifying_key(&trust.publisher_public_key),
        "--provenance-key-id".to_owned(),
        trust.provenance_key_id.clone(),
        "--provenance-public-key".to_owned(),
        encode_verifying_key(&trust.provenance_public_key),
        "--provenance-class".to_owned(),
        trust.provenance_class.clone(),
        "--provenance-server-id".to_owned(),
        trust
            .provenance_server_id
            .map_or_else(|| "none".to_owned(), |id| id.to_string()),
        "--core-public-key".to_owned(),
        encode_verifying_key(core_key),
    ];
    if let Some(failure) = failure {
        sandbox_args.push("--conformance-failure".into());
        sandbox_args.push(failure.into());
    }
    let mut command = Command::new("/usr/bin/systemd-run");
    command.args([
        "--user",
        "--scope",
        "--quiet",
        "--property=MemoryMax=268435456",
        "--property=CPUQuota=50%",
        "--property=TasksMax=16",
        "/usr/bin/prlimit",
        "--nofile=64:64",
        "--",
        "/usr/bin/bwrap",
    ]);
    command
        .args(sandbox_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(ModuleRuntimeError::Io)
}

fn systemd_user_available() -> bool {
    Command::new("/usr/bin/systemctl")
        .args(["--user", "is-system-running"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn wait_for_exit(
    child: &mut Child,
    deadline: Duration,
) -> Result<std::process::ExitStatus, ModuleRuntimeError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if started.elapsed() >= deadline {
            terminate(child);
            return Err(ModuleRuntimeError::Execution(
                "host did not exit after response".into(),
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn server_environment_absent() -> bool {
    !std::env::vars_os().any(|(key, _)| {
        key.to_str().is_some_and(|key| {
            matches!(
                key,
                "DATABASE_URL"
                    | "OGS_MFA_ENCRYPTION_KEY"
                    | "OGS_MODULE_ADMISSION_SIGNING_SEED"
                    | "OGS_MODULE_PAIRWISE_SECRET"
            ) || key.starts_with("OGS_SECRET_")
                || key.starts_with("OMARCHYGS_SECRET_")
        })
    })
}

fn loopback_only() -> Result<bool, ModuleRuntimeError> {
    let network = std::fs::read_to_string("/proc/net/dev")?;
    Ok(network.lines().skip(2).all(|line| {
        line.split_once(':')
            .is_some_and(|(name, _)| name.trim() == "lo")
    }))
}

fn resident_kib() -> Result<u64, ModuleRuntimeError> {
    let status = std::fs::read_to_string("/proc/self/status")?;
    status
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
        .ok_or_else(|| ModuleRuntimeError::Containment("host RSS unavailable".into()))
}
