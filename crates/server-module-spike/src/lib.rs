//! Isolated executable architecture proof for future OmarchyGS server modules.
//!
//! This crate is intentionally a nested workspace. Nothing here is linked into
//! the production server, and the proof does not authorize a module loader.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder};
use wit_component::{ComponentEncoder, StringEncoding, embed_component_metadata};
use wit_parser::Resolve;

/// Maximum component and manifest input accepted by the proof.
pub const MAX_ARTIFACT_BYTES: usize = 1024 * 1024;
/// Maximum canonical JSON frame accepted before allocation.
pub const MAX_FRAME_BYTES: usize = 64 * 1024;
/// Maximum guest linear-memory allocation.
pub const MAX_LINEAR_MEMORY_BYTES: usize = 4 * 1024 * 1024;
/// Deterministic Wasmtime fuel available to one invocation.
pub const MAX_FUEL: u64 = 100_000;
/// Exact WIT package supported by the proof.
pub const WIT_PACKAGE: &str = "ignibyte:omarchygs-server-module@1.0.0";
/// Exact WIT world supported by the proof.
pub const WIT_WORLD: &str = "module-proof";
/// Release manifest format.
pub const RELEASE_FORMAT: &str = "omarchygs.server-module-release/v1";
/// Provenance statement format.
pub const PROVENANCE_FORMAT: &str = "omarchygs.server-module-provenance/v1";
/// Core admission format.
pub const ADMISSION_FORMAT: &str = "omarchygs.server-module-admission/v1";
/// Hook request format.
pub const HOOK_FORMAT: &str = "omarchygs.server-module-hook/v1";
/// Host response format.
pub const RESPONSE_FORMAT: &str = "omarchygs.server-module-response/v1";
const PUBLISHER_KEY_ID: &str = "publisher-ignibyte-1";
const PROVENANCE_KEY_ID: &str = "provenance-authority-1";
const CORE_KEY_ID: &str = "server-core-1";

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "module-proof",
    });
}

/// Stable proof failures. Detailed runtime internals are deliberately not sent
/// across the untrusted module protocol.
#[derive(Debug, Error)]
pub enum ProofError {
    /// Input or state violated a bounded contract.
    #[error("contract rejected: {0}")]
    Contract(String),
    /// Signature or digest verification failed.
    #[error("integrity rejected: {0}")]
    Integrity(String),
    /// A bounded frame was malformed or non-canonical.
    #[error("frame rejected: {0}")]
    Frame(String),
    /// Component compilation, instantiation, or execution failed.
    #[error("module execution rejected: {0}")]
    Execution(String),
    /// A core-owned authorization check failed.
    #[error("core authorization rejected: {0}")]
    Authorization(String),
    /// An idempotency identity was replayed with a different body.
    #[error("replay identity conflict")]
    ReplayConflict,
    /// A namespaced state revision was stale.
    #[error("state revision conflict")]
    RevisionConflict,
    /// A bounded queue cannot accept more work.
    #[error("bounded queue is full")]
    QueueFull,
    /// Local process I/O failed.
    #[error("local I/O failed: {0}")]
    Io(#[from] std::io::Error),
}

/// Canonically signed JSON. The signature binds the format name and exact
/// canonical payload bytes; each trust claim uses a separate envelope.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedEnvelope {
    /// Envelope format/domain.
    pub document_format: String,
    /// Stable public-key identifier.
    pub key_id: String,
    /// Canonical JSON encoded as unpadded base64url.
    pub payload: String,
    /// Ed25519 signature encoded as unpadded base64url.
    pub signature: String,
}

impl SignedEnvelope {
    /// Sign a strictly serializable payload with a domain-separated format.
    pub fn sign<T: Serialize>(
        document_format: &str,
        key_id: &str,
        payload: &T,
        key: &SigningKey,
    ) -> Result<Self, ProofError> {
        validate_identifier("document_format", document_format, 96)?;
        validate_identifier("key_id", key_id, 96)?;
        let bytes = canonical_json(payload)?;
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ProofError::Contract("signed payload exceeds limit".into()));
        }
        let message = signature_message(document_format, &bytes);
        let signature = key.sign(&message);
        Ok(Self {
            document_format: document_format.to_owned(),
            key_id: key_id.to_owned(),
            payload: URL_SAFE_NO_PAD.encode(bytes),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        })
    }

    /// Verify the exact domain, signature, strict schema, and canonical bytes.
    pub fn verify<T: DeserializeOwned + Serialize>(
        &self,
        expected_format: &str,
        key: &VerifyingKey,
    ) -> Result<T, ProofError> {
        if self.document_format != expected_format {
            return Err(ProofError::Integrity("signature domain mismatch".into()));
        }
        validate_identifier("key_id", &self.key_id, 96)?;
        let payload = decode_bounded(&self.payload, MAX_ARTIFACT_BYTES, "signed payload")?;
        let signature_bytes = decode_bounded(&self.signature, 64, "signature")?;
        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|_| ProofError::Integrity("invalid signature length".into()))?;
        key.verify(&signature_message(expected_format, &payload), &signature)
            .map_err(|_| ProofError::Integrity("signature mismatch".into()))?;
        let value: T = serde_json::from_slice(&payload)
            .map_err(|error| ProofError::Contract(format!("invalid signed JSON: {error}")))?;
        if canonical_json(&value)? != payload {
            return Err(ProofError::Contract(
                "signed payload is not canonical JSON".into(),
            ));
        }
        Ok(value)
    }

    /// Hash the exact decoded canonical payload.
    pub fn payload_sha256(&self) -> Result<String, ProofError> {
        let payload = decode_bounded(&self.payload, MAX_ARTIFACT_BYTES, "signed payload")?;
        Ok(sha256_hex(&payload))
    }
}

/// One typed hook offered by the proof.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    /// Observe a persona report after the platform transaction commits.
    PersonaReported,
}

/// A least-privilege effect that core may grant to a module.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Propose a bounded moderation label; core still authorizes and commits it.
    ModerationAddLabel,
    /// Propose delivery to a core-owned integration destination.
    ExternalDelivery,
}

impl Capability {
    fn bit(self) -> u64 {
        match self {
            Self::ModerationAddLabel => 1,
            Self::ExternalDelivery => 2,
        }
    }
}

/// Exact Component Model identity admitted by core.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WitIdentity {
    /// WIT package including semantic version.
    pub package: String,
    /// WIT world name.
    pub world: String,
    /// Supported interface major.
    pub major: u16,
    /// Digest of the exact WIT source.
    pub sha256: String,
}

/// Resource ceilings requested by the release and granted by admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBudgets {
    /// Maximum framed request/response bytes.
    pub frame_bytes: u32,
    /// Maximum linear-memory bytes.
    pub memory_bytes: u32,
    /// Maximum deterministic fuel per call.
    pub fuel: u64,
    /// Outer execution deadline after readiness.
    pub execution_ms: u32,
}

impl ResourceBudgets {
    fn validate(&self) -> Result<(), ProofError> {
        if self.frame_bytes == 0 || self.frame_bytes as usize > MAX_FRAME_BYTES {
            return Err(ProofError::Contract("invalid frame budget".into()));
        }
        if self.memory_bytes == 0 || self.memory_bytes as usize > MAX_LINEAR_MEMORY_BYTES {
            return Err(ProofError::Contract("invalid memory budget".into()));
        }
        if self.fuel == 0 || self.fuel > MAX_FUEL {
            return Err(ProofError::Contract("invalid fuel budget".into()));
        }
        if self.execution_ms == 0 || self.execution_ms > 500 {
            return Err(ProofError::Contract("invalid execution deadline".into()));
        }
        Ok(())
    }
}

/// Immutable publisher release statement.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleReleaseManifest {
    /// Exact format identifier.
    pub format: String,
    /// Stable module identity.
    pub module_id: String,
    /// Stable publisher identity.
    pub publisher_id: String,
    /// Immutable release identity.
    pub release_id: Uuid,
    /// Semantic release version.
    pub version: String,
    /// Digest of the exact component bytes.
    pub component_sha256: String,
    /// Exact typed interface identity.
    pub wit: WitIdentity,
    /// Sorted, unique requested capabilities.
    pub requested_capabilities: Vec<Capability>,
    /// Sorted, unique requested hooks.
    pub subscribed_hooks: Vec<HookKind>,
    /// Requested resource ceilings.
    pub budgets: ResourceBudgets,
    /// Immutable configuration schema identity.
    pub config_schema: String,
    /// Immutable state schema identity.
    pub state_schema: String,
    /// Exact component export.
    pub entrypoint: String,
}

impl ModuleReleaseManifest {
    /// Validate strict release shape independently of trust or admission.
    pub fn validate(&self) -> Result<(), ProofError> {
        if self.format != RELEASE_FORMAT {
            return Err(ProofError::Contract("release format mismatch".into()));
        }
        validate_identifier("module_id", &self.module_id, 64)?;
        validate_identifier("publisher_id", &self.publisher_id, 64)?;
        validate_semver(&self.version)?;
        validate_digest(&self.component_sha256)?;
        validate_wit(&self.wit)?;
        validate_sorted_unique("requested capabilities", &self.requested_capabilities)?;
        validate_sorted_unique("subscribed hooks", &self.subscribed_hooks)?;
        if self.requested_capabilities.is_empty() || self.subscribed_hooks.is_empty() {
            return Err(ProofError::Contract(
                "release must request at least one hook and capability".into(),
            ));
        }
        self.budgets.validate()?;
        validate_identifier("config_schema", &self.config_schema, 96)?;
        validate_identifier("state_schema", &self.state_schema, 96)?;
        if self.entrypoint != "handle" {
            return Err(ProofError::Contract("unsupported entrypoint".into()));
        }
        Ok(())
    }
}

/// Independent review/trust provenance. It is deliberately not a capability.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "class", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProvenanceClass {
    /// A marketplace reviewer attested this exact release manifest.
    MarketplaceVetted { review_id: Uuid },
    /// The server operator explicitly trusted custom executable code.
    OperatorCustom { server_id: Uuid },
}

/// Signed provenance statement binding one release manifest digest.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleProvenance {
    /// Exact statement format.
    pub format: String,
    /// Digest of the signed release manifest payload.
    pub release_manifest_sha256: String,
    /// Review/operator trust class.
    pub provenance: ProvenanceClass,
}

/// Core-owned lifecycle state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    /// Bytes and contracts are verified but not runnable.
    Staged,
    /// Installed but no delivery is allowed.
    Disabled,
    /// Readiness is in progress.
    Enabling,
    /// Exact release may receive admitted hooks.
    Active,
    /// Circuit breaker has paused fresh work.
    Degraded,
    /// Operator/security policy has stopped work.
    Suspended,
    /// Terminal retained tombstone.
    Retired,
}

/// Separate core admission. This converts requested power into an exact grant.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleAdmission {
    /// Exact admission format.
    pub format: String,
    /// Stable owner-operated server identity.
    pub server_id: Uuid,
    /// Immutable admission identity.
    pub admission_id: Uuid,
    /// Monotonic lifecycle revision.
    pub lifecycle_revision: u64,
    /// Exact admitted lifecycle.
    pub lifecycle: LifecycleStatus,
    /// Bound module identity.
    pub module_id: String,
    /// Bound release identity.
    pub release_id: Uuid,
    /// Bound component digest.
    pub component_sha256: String,
    /// Bound release manifest payload digest.
    pub release_manifest_sha256: String,
    /// Bound provenance payload digest.
    pub provenance_sha256: String,
    /// Bound interface identity.
    pub wit: WitIdentity,
    /// Sorted, unique granted capability subset.
    pub granted_capabilities: Vec<Capability>,
    /// Sorted, unique subscribed hook subset.
    pub subscribed_hooks: Vec<HookKind>,
    /// Core-enforced resource ceilings.
    pub budgets: ResourceBudgets,
    /// Immutable configuration revision.
    pub config_revision: u64,
    /// Bound state schema identity.
    pub state_schema: String,
    /// Immutable state revision supplied to this event.
    pub state_revision: u64,
}

/// Pairwise/public subject only; account ownership never crosses the boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "scope",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ModuleSubject {
    /// Opaque pairwise subject generated for this module.
    Pairwise(String),
    /// Explicitly public domain identifier.
    Public(String),
}

/// Allowlisted payload for the proof hook.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HookPayload {
    /// Metadata-only post-commit report observation.
    PersonaReported {
        /// Opaque public report identity.
        report_id: Uuid,
        /// Bounded category code, not free-form private content.
        category: String,
    },
}

/// Bounded typed event delivered at least once.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleHookEvent {
    /// Exact hook format.
    pub format: String,
    /// Idempotency identity owned by core.
    pub event_id: Uuid,
    /// One-based delivery attempt.
    pub attempt: u16,
    /// Stable server identity.
    pub server_id: Uuid,
    /// Exact module identity.
    pub module_id: String,
    /// Exact release identity.
    pub release_id: Uuid,
    /// Exact admission identity.
    pub admission_id: Uuid,
    /// Typed hook.
    pub hook: HookKind,
    /// Causal platform revision.
    pub causal_revision: u64,
    /// Budget supplied to the dispatcher, not a guest clock.
    pub deadline_ms: u32,
    /// Opaque pairwise or public subject.
    pub subject: ModuleSubject,
    /// Bounded immutable configuration snapshot.
    pub config: BTreeMap<String, String>,
    /// Configuration revision.
    pub config_revision: u64,
    /// Bounded module-only state snapshot.
    pub state: BTreeMap<String, String>,
    /// State revision.
    pub state_revision: u64,
    /// Allowlisted typed payload.
    pub payload: HookPayload,
}

/// Complete host request with independently verifiable trust claims.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostRequest {
    /// Publisher-signed release statement.
    pub release: SignedEnvelope,
    /// Publisher Ed25519 public key.
    pub publisher_public_key: String,
    /// Separately signed marketplace/operator provenance.
    pub provenance: SignedEnvelope,
    /// Provenance authority Ed25519 public key.
    pub provenance_public_key: String,
    /// Core-signed admission.
    pub admission: SignedEnvelope,
    /// Core Ed25519 public key provisioned to this host.
    pub core_public_key: String,
    /// Exact typed event.
    pub event: ModuleHookEvent,
}

/// One typed effect proposal returned by the isolated component.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModuleIntent {
    /// Add one allowlisted numeric moderation label.
    ModerationAddLabel {
        /// Target revision that core must still revalidate.
        expected_revision: u64,
        /// Bounded label code.
        label: u64,
    },
    /// Deliver through a separately configured, core-owned integration.
    ExternalDelivery {
        /// Target revision that core must still revalidate.
        expected_revision: u64,
        /// Core-owned destination slot, never a URL.
        destination_slot: u64,
    },
}

impl ModuleIntent {
    fn capability(&self) -> Capability {
        match self {
            Self::ModerationAddLabel { .. } => Capability::ModerationAddLabel,
            Self::ExternalDelivery { .. } => Capability::ExternalDelivery,
        }
    }

    fn expected_revision(&self) -> u64 {
        match self {
            Self::ModerationAddLabel {
                expected_revision, ..
            }
            | Self::ExternalDelivery {
                expected_revision, ..
            } => *expected_revision,
        }
    }
}

/// Stable host result. Runtime errors never become unbounded error text.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
pub enum HostResult {
    /// Component deliberately proposed no effect.
    Noop,
    /// One typed effect is proposed for core authorization.
    Proposed { intent: ModuleIntent },
    /// Host rejected component behavior with a stable code.
    Rejected { code: String },
}

/// Authenticated response context echoed by the host.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostResponse {
    /// Exact response format.
    pub format: String,
    /// Bound event identity.
    pub event_id: Uuid,
    /// Bound release identity.
    pub release_id: Uuid,
    /// Bound admission identity.
    pub admission_id: Uuid,
    /// Stable result.
    pub outcome: HostResult,
}

/// Readiness proof emitted before a request is accepted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostReady {
    /// Protocol marker.
    pub format: String,
    /// Component compiled and instantiated against the exact WIT world.
    pub component_ready: bool,
    /// The sandbox did not mount a home tree.
    pub home_absent: bool,
    /// The sandbox did not mount the host password database.
    pub passwd_absent: bool,
    /// Server credentials/configuration are absent from the environment.
    pub server_environment_absent: bool,
    /// Network namespace exposes no non-loopback interface.
    pub loopback_only: bool,
    /// Resident memory observed by the host itself after readiness.
    pub resident_kib: u64,
}

/// Verified request facts used by the host and then rechecked by core.
#[derive(Clone, Debug)]
pub struct VerifiedRequest {
    /// Publisher release statement.
    pub release: ModuleReleaseManifest,
    /// Provenance statement.
    pub provenance: ModuleProvenance,
    /// Exact core admission.
    pub admission: ModuleAdmission,
}

/// Verify every separate trust claim and all cross-document bindings.
pub fn verify_host_request(
    request: &HostRequest,
    component_bytes: &[u8],
) -> Result<VerifiedRequest, ProofError> {
    if component_bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(ProofError::Contract("component exceeds limit".into()));
    }
    let trusted_publisher = SigningKey::from_bytes(&[7_u8; 32]).verifying_key();
    let trusted_provenance = SigningKey::from_bytes(&[9_u8; 32]).verifying_key();
    let trusted_core = SigningKey::from_bytes(&[11_u8; 32]).verifying_key();
    if request.publisher_public_key != encode_verifying_key(&trusted_publisher)
        || request.provenance_public_key != encode_verifying_key(&trusted_provenance)
        || request.core_public_key != encode_verifying_key(&trusted_core)
        || request.release.key_id != PUBLISHER_KEY_ID
        || request.provenance.key_id != PROVENANCE_KEY_ID
        || request.admission.key_id != CORE_KEY_ID
    {
        return Err(ProofError::Integrity(
            "request authority is not provisioned by the proof host".into(),
        ));
    }
    let publisher_key = decode_verifying_key(&request.publisher_public_key)?;
    let provenance_key = decode_verifying_key(&request.provenance_public_key)?;
    let core_key = decode_verifying_key(&request.core_public_key)?;
    let release: ModuleReleaseManifest = request.release.verify(RELEASE_FORMAT, &publisher_key)?;
    release.validate()?;
    let provenance: ModuleProvenance = request
        .provenance
        .verify(PROVENANCE_FORMAT, &provenance_key)?;
    let admission: ModuleAdmission = request.admission.verify(ADMISSION_FORMAT, &core_key)?;

    if provenance.format != PROVENANCE_FORMAT {
        return Err(ProofError::Contract("provenance format mismatch".into()));
    }
    validate_digest(&provenance.release_manifest_sha256)?;
    match &provenance.provenance {
        ProvenanceClass::MarketplaceVetted { review_id } if review_id.is_nil() => {
            return Err(ProofError::Contract(
                "marketplace provenance identity is nil".into(),
            ));
        }
        ProvenanceClass::OperatorCustom { server_id } if server_id.is_nil() => {
            return Err(ProofError::Contract(
                "operator provenance server identity is nil".into(),
            ));
        }
        ProvenanceClass::OperatorCustom { server_id } if *server_id != admission.server_id => {
            return Err(ProofError::Integrity(
                "operator provenance server binding mismatch".into(),
            ));
        }
        ProvenanceClass::MarketplaceVetted { .. } | ProvenanceClass::OperatorCustom { .. } => {}
    }
    if admission.format != ADMISSION_FORMAT {
        return Err(ProofError::Contract("admission format mismatch".into()));
    }
    if release.release_id.is_nil()
        || admission.server_id.is_nil()
        || admission.admission_id.is_nil()
    {
        return Err(ProofError::Contract(
            "release/admission identity is nil".into(),
        ));
    }
    if admission.lifecycle != LifecycleStatus::Active || admission.lifecycle_revision == 0 {
        return Err(ProofError::Authorization("admission is not active".into()));
    }
    validate_sorted_unique("granted capabilities", &admission.granted_capabilities)?;
    validate_sorted_unique("admitted hooks", &admission.subscribed_hooks)?;
    admission.budgets.validate()?;
    validate_wit(&admission.wit)?;

    let release_sha = request.release.payload_sha256()?;
    let provenance_sha = request.provenance.payload_sha256()?;
    let component_sha = sha256_hex(component_bytes);
    if release.component_sha256 != component_sha
        || admission.component_sha256 != component_sha
        || provenance.release_manifest_sha256 != release_sha
        || admission.release_manifest_sha256 != release_sha
        || admission.provenance_sha256 != provenance_sha
        || admission.module_id != release.module_id
        || admission.release_id != release.release_id
        || admission.wit != release.wit
        || admission.state_schema != release.state_schema
    {
        return Err(ProofError::Integrity(
            "release/provenance/admission binding mismatch".into(),
        ));
    }
    if !is_subset(
        &admission.granted_capabilities,
        &release.requested_capabilities,
    ) || !is_subset(&admission.subscribed_hooks, &release.subscribed_hooks)
    {
        return Err(ProofError::Authorization(
            "admission exceeds release request".into(),
        ));
    }
    if admission.budgets.frame_bytes > release.budgets.frame_bytes
        || admission.budgets.memory_bytes > release.budgets.memory_bytes
        || admission.budgets.fuel > release.budgets.fuel
        || admission.budgets.execution_ms > release.budgets.execution_ms
    {
        return Err(ProofError::Authorization(
            "admission exceeds requested budgets".into(),
        ));
    }
    validate_event(&request.event, &admission)?;
    Ok(VerifiedRequest {
        release,
        provenance,
        admission,
    })
}

fn validate_event(event: &ModuleHookEvent, admission: &ModuleAdmission) -> Result<(), ProofError> {
    if event.event_id.is_nil() {
        return Err(ProofError::Contract("event identity is nil".into()));
    }
    if event.format != HOOK_FORMAT
        || event.server_id != admission.server_id
        || event.module_id != admission.module_id
        || event.release_id != admission.release_id
        || event.admission_id != admission.admission_id
        || event.config_revision != admission.config_revision
        || event.state_revision != admission.state_revision
    {
        return Err(ProofError::Integrity("event context mismatch".into()));
    }
    if event.attempt == 0 || event.attempt > 16 {
        return Err(ProofError::Contract("invalid delivery attempt".into()));
    }
    if event.deadline_ms == 0 || event.deadline_ms > admission.budgets.execution_ms {
        return Err(ProofError::Contract("invalid event deadline".into()));
    }
    if !admission.subscribed_hooks.contains(&event.hook) {
        return Err(ProofError::Authorization("hook not admitted".into()));
    }
    validate_snapshot("configuration", &event.config, 32, 4096)?;
    validate_snapshot("state", &event.state, 32, 4096)?;
    match &event.subject {
        ModuleSubject::Pairwise(subject) | ModuleSubject::Public(subject) => {
            validate_identifier("subject", subject, 96)?;
        }
    }
    match (&event.hook, &event.payload) {
        (
            HookKind::PersonaReported,
            HookPayload::PersonaReported {
                report_id,
                category,
            },
        ) => {
            if report_id.is_nil() {
                return Err(ProofError::Contract("report identity is nil".into()));
            }
            validate_identifier("report category", category, 32)?;
        }
    }
    Ok(())
}

/// Wasmtime component runtime with no imports or WASI and a fresh Store per
/// readiness check and invocation.
pub struct ModuleRuntime {
    engine: Engine,
    component: Component,
    component_bytes: Vec<u8>,
}

struct StoreState {
    limits: StoreLimits,
}

impl ModuleRuntime {
    /// Compile a bounded component under the pinned runtime configuration.
    pub fn compile(component_bytes: &[u8]) -> Result<Self, ProofError> {
        if component_bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ProofError::Contract("component exceeds limit".into()));
        }
        if !component_bytes.starts_with(b"\0asm") {
            return Err(ProofError::Contract(
                "runtime accepts only completed binary components".into(),
            ));
        }
        let mut config = Config::new();
        config
            .wasm_component_model(true)
            .consume_fuel(true)
            .wasm_memory64(false);
        let engine = Engine::new(&config)
            .map_err(|error| ProofError::Execution(format!("engine setup failed: {error}")))?;
        let component = Component::new(&engine, component_bytes).map_err(|error| {
            ProofError::Execution(format!("component compilation failed: {error:#}"))
        })?;
        Ok(Self {
            engine,
            component,
            component_bytes: component_bytes.to_vec(),
        })
    }

    /// Prove the component implements the exact WIT world and fits limits.
    pub fn readiness(&self) -> Result<(), ProofError> {
        let mut store = self.fresh_store(MAX_FUEL, MAX_LINEAR_MEMORY_BYTES)?;
        let linker = Linker::new(&self.engine);
        bindings::ModuleProof::instantiate(&mut store, &self.component, &linker).map_err(
            |error| ProofError::Execution(format!("component readiness failed: {error:#}")),
        )?;
        Ok(())
    }

    /// Verify the request, execute one fresh instance, and convert the result
    /// to an allowlisted typed proposal or stable rejection.
    pub fn execute(&self, request: &HostRequest) -> HostResponse {
        let rejected = |code: &str| HostResponse {
            format: RESPONSE_FORMAT.to_owned(),
            event_id: request.event.event_id,
            release_id: request.event.release_id,
            admission_id: request.event.admission_id,
            outcome: HostResult::Rejected {
                code: code.to_owned(),
            },
        };
        let verified = match verify_host_request(request, &self.component_bytes) {
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
            match bindings::ModuleProof::instantiate(&mut store, &self.component, &linker) {
                Ok(bindings) => bindings,
                Err(_) => return rejected("module_instantiation_failed"),
            };
        let granted_capabilities = verified
            .admission
            .granted_capabilities
            .iter()
            .fold(0_u64, |bits, capability| bits | capability.bit());
        let event = bindings::HookEvent {
            kind: match request.event.hook {
                HookKind::PersonaReported => 1,
            },
            revision: request.event.causal_revision,
            granted_capabilities,
        };
        let raw_intent = match bindings.call_handle(&mut store, event) {
            Ok(intent) => intent,
            Err(_) => return rejected("module_execution_failed"),
        };
        let outcome = match raw_intent.kind {
            0 => HostResult::Noop,
            1 => HostResult::Proposed {
                intent: ModuleIntent::ModerationAddLabel {
                    expected_revision: raw_intent.expected_revision,
                    label: raw_intent.value,
                },
            },
            2 => HostResult::Proposed {
                intent: ModuleIntent::ExternalDelivery {
                    expected_revision: raw_intent.expected_revision,
                    destination_slot: raw_intent.value,
                },
            },
            _ => HostResult::Rejected {
                code: "unknown_intent".to_owned(),
            },
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
            format: RESPONSE_FORMAT.to_owned(),
            event_id: request.event.event_id,
            release_id: request.event.release_id,
            admission_id: request.event.admission_id,
            outcome,
        }
    }

    fn fresh_store(&self, fuel: u64, memory: usize) -> Result<Store<StoreState>, ProofError> {
        let limits = StoreLimitsBuilder::new()
            .memory_size(memory)
            .instances(1)
            .memories(1)
            .tables(1)
            .build();
        let mut store = Store::new(&self.engine, StoreState { limits });
        store.limiter(|state| &mut state.limits);
        store
            .set_fuel(fuel)
            .map_err(|error| ProofError::Execution(format!("fuel setup failed: {error}")))?;
        Ok(store)
    }
}

/// Core-owned idempotent commit receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoreReceipt {
    /// Source event.
    pub event_id: Uuid,
    /// Whether protected state changed.
    pub committed: bool,
    /// Revision after the decision.
    pub resulting_revision: u64,
    /// Stable result code.
    pub code: String,
}

/// Minimal core authorizer proving that host output has no direct effect.
#[derive(Default)]
pub struct ProofCore {
    target_revision: u64,
    labels: Vec<u64>,
    receipts: HashMap<(Uuid, Uuid), (String, CoreReceipt)>,
}

impl ProofCore {
    /// Create core state at a known target revision.
    pub fn at_revision(revision: u64) -> Self {
        Self {
            target_revision: revision,
            ..Self::default()
        }
    }

    /// Re-authorize and idempotently commit one host response.
    pub fn apply(
        &mut self,
        request: &HostRequest,
        response: &HostResponse,
        component_bytes: &[u8],
    ) -> Result<CoreReceipt, ProofError> {
        let request_hash = sha256_hex(&canonical_json(request)?);
        let receipt_key = (request.event.release_id, request.event.event_id);
        if let Some((stored_hash, receipt)) = self.receipts.get(&receipt_key) {
            if stored_hash == &request_hash {
                return Ok(receipt.clone());
            }
            return Err(ProofError::ReplayConflict);
        }
        let verified = verify_host_request(request, component_bytes)?;
        if response.format != RESPONSE_FORMAT
            || response.event_id != request.event.event_id
            || response.release_id != request.event.release_id
            || response.admission_id != request.event.admission_id
        {
            return Err(ProofError::Integrity("response context mismatch".into()));
        }
        let receipt = match &response.outcome {
            HostResult::Noop => CoreReceipt {
                event_id: request.event.event_id,
                committed: false,
                resulting_revision: self.target_revision,
                code: "noop".into(),
            },
            HostResult::Rejected { .. } => CoreReceipt {
                event_id: request.event.event_id,
                committed: false,
                resulting_revision: self.target_revision,
                code: "module_rejected".into(),
            },
            HostResult::Proposed { intent } => {
                if !verified
                    .admission
                    .granted_capabilities
                    .contains(&intent.capability())
                {
                    return Err(ProofError::Authorization(
                        "intent capability is not granted".into(),
                    ));
                }
                if intent.expected_revision() != self.target_revision {
                    return Err(ProofError::RevisionConflict);
                }
                match intent {
                    ModuleIntent::ModerationAddLabel { label, .. } if *label <= 100 => {
                        self.labels.push(*label);
                        self.target_revision = self.target_revision.saturating_add(1);
                        CoreReceipt {
                            event_id: request.event.event_id,
                            committed: true,
                            resulting_revision: self.target_revision,
                            code: "moderation_label_added".into(),
                        }
                    }
                    ModuleIntent::ModerationAddLabel { .. } => {
                        return Err(ProofError::Authorization(
                            "moderation label is outside policy".into(),
                        ));
                    }
                    ModuleIntent::ExternalDelivery { .. } => {
                        return Err(ProofError::Authorization(
                            "external delivery is not implemented by proof core".into(),
                        ));
                    }
                }
            }
        };
        self.receipts
            .insert(receipt_key, (request_hash, receipt.clone()));
        Ok(receipt)
    }

    /// Current core-owned revision.
    pub fn revision(&self) -> u64 {
        self.target_revision
    }

    /// Core-owned labels; the component never gets this mutable collection.
    pub fn labels(&self) -> &[u64] {
        &self.labels
    }
}

/// Bounded per-partition outbox demonstrating ordering and backpressure.
pub struct DispatchQueue {
    capacity: usize,
    len: usize,
    partitions: BTreeMap<String, VecDeque<ModuleHookEvent>>,
}

impl DispatchQueue {
    /// Create a bounded queue.
    pub fn new(capacity: usize) -> Result<Self, ProofError> {
        if capacity == 0 || capacity > 1024 {
            return Err(ProofError::Contract("invalid queue capacity".into()));
        }
        Ok(Self {
            capacity,
            len: 0,
            partitions: BTreeMap::new(),
        })
    }

    /// Append to one stable partition, preserving its sequence.
    pub fn enqueue(&mut self, partition: &str, event: ModuleHookEvent) -> Result<(), ProofError> {
        validate_identifier("partition", partition, 128)?;
        if self.len == self.capacity {
            return Err(ProofError::QueueFull);
        }
        self.partitions
            .entry(partition.to_owned())
            .or_default()
            .push_back(event);
        self.len += 1;
        Ok(())
    }

    /// Pop only the oldest event in one partition.
    pub fn pop(&mut self, partition: &str) -> Option<ModuleHookEvent> {
        let (event, empty) = {
            let queue = self.partitions.get_mut(partition)?;
            let event = queue.pop_front();
            (event, queue.is_empty())
        };
        if event.is_some() {
            self.len -= 1;
        }
        if empty {
            self.partitions.remove(partition);
        }
        event
    }

    /// Active partitions retaining at least one queued event.
    pub fn active_partition_count(&self) -> usize {
        self.partitions.len()
    }
}

/// One bounded namespaced state operation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum StateOperation {
    /// Insert or replace one key.
    Set { key: String, value: String },
    /// Remove one key.
    Remove { key: String },
}

/// Explicit forward migration plan.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationPlan {
    /// Required current schema.
    pub from_schema: String,
    /// New schema after success.
    pub to_schema: String,
    /// Bounded deterministic operations.
    pub operations: Vec<StateOperation>,
}

/// Core-owned module namespace with compare-and-set revisions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NamespaceState {
    /// Stable module identity.
    pub module_id: String,
    /// Exact state schema.
    pub schema: String,
    /// Monotonic namespace revision.
    pub revision: u64,
    /// Bounded values owned only by this namespace.
    pub values: BTreeMap<String, String>,
}

impl NamespaceState {
    /// Construct an empty namespace.
    pub fn empty(module_id: &str, schema: &str) -> Result<Self, ProofError> {
        validate_identifier("module_id", module_id, 64)?;
        validate_identifier("state schema", schema, 96)?;
        Ok(Self {
            module_id: module_id.to_owned(),
            schema: schema.to_owned(),
            revision: 0,
            values: BTreeMap::new(),
        })
    }

    /// Apply operations only at the exact expected revision.
    pub fn compare_and_set(
        &mut self,
        expected_revision: u64,
        operations: &[StateOperation],
    ) -> Result<u64, ProofError> {
        if self.revision != expected_revision {
            return Err(ProofError::RevisionConflict);
        }
        let next = apply_state_operations(&self.values, operations)?;
        self.values = next;
        self.revision = self.revision.saturating_add(1);
        Ok(self.revision)
    }

    /// Stage and atomically apply one explicit forward migration. Failure leaves
    /// the original namespace byte-for-byte unchanged.
    pub fn migrate(
        &mut self,
        expected_revision: u64,
        plan: &MigrationPlan,
    ) -> Result<NamespaceState, ProofError> {
        if self.revision != expected_revision {
            return Err(ProofError::RevisionConflict);
        }
        if plan.from_schema != self.schema || plan.to_schema == self.schema {
            return Err(ProofError::Contract("invalid migration schemas".into()));
        }
        validate_identifier("migration target schema", &plan.to_schema, 96)?;
        let rollback = self.clone();
        let next = apply_state_operations(&self.values, &plan.operations)?;
        self.values = next;
        self.schema.clone_from(&plan.to_schema);
        self.revision = self.revision.saturating_add(1);
        Ok(rollback)
    }

    /// Serialize a deterministic backup snapshot.
    pub fn backup(&self) -> Result<Vec<u8>, ProofError> {
        canonical_json(self)
    }

    /// Restore and strictly validate a deterministic snapshot.
    pub fn restore(bytes: &[u8], expected_module: &str) -> Result<Self, ProofError> {
        if bytes.len() > MAX_FRAME_BYTES {
            return Err(ProofError::Contract("state backup exceeds limit".into()));
        }
        let state: Self = serde_json::from_slice(bytes)
            .map_err(|error| ProofError::Contract(format!("invalid state backup: {error}")))?;
        if canonical_json(&state)? != bytes || state.module_id != expected_module {
            return Err(ProofError::Integrity(
                "state backup binding mismatch".into(),
            ));
        }
        validate_snapshot("restored state", &state.values, 32, 4096)?;
        Ok(state)
    }
}

/// Immutable lifecycle audit entry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleAudit {
    /// Idempotent admin operation.
    pub operation_id: Uuid,
    /// Expected prior revision.
    pub prior_revision: u64,
    /// Resulting revision.
    pub resulting_revision: u64,
    /// Prior state.
    pub from: LifecycleStatus,
    /// New state.
    pub to: LifecycleStatus,
    /// Bounded operator reason.
    pub reason: String,
}

/// Exact-release lifecycle model with idempotent expected-state operations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleLifecycle {
    /// Exact release.
    pub release_id: Uuid,
    /// Monotonic lifecycle revision.
    pub revision: u64,
    /// Current lifecycle status.
    pub status: LifecycleStatus,
    /// Immutable audit history.
    pub audit: Vec<LifecycleAudit>,
}

impl ModuleLifecycle {
    /// Create a staged exact release.
    pub fn staged(release_id: Uuid) -> Self {
        Self {
            release_id,
            revision: 0,
            status: LifecycleStatus::Staged,
            audit: Vec::new(),
        }
    }

    /// Apply or replay one legal transition.
    pub fn transition(
        &mut self,
        operation_id: Uuid,
        expected_revision: u64,
        to: LifecycleStatus,
        reason: &str,
    ) -> Result<LifecycleAudit, ProofError> {
        if let Some(existing) = self
            .audit
            .iter()
            .find(|entry| entry.operation_id == operation_id)
        {
            if existing.prior_revision == expected_revision
                && existing.to == to
                && existing.reason == reason
            {
                return Ok(existing.clone());
            }
            return Err(ProofError::ReplayConflict);
        }
        if self.revision != expected_revision {
            return Err(ProofError::RevisionConflict);
        }
        validate_text("lifecycle reason", reason, 1, 256)?;
        if !legal_transition(self.status, to) {
            return Err(ProofError::Authorization(
                "illegal lifecycle transition".into(),
            ));
        }
        let audit = LifecycleAudit {
            operation_id,
            prior_revision: self.revision,
            resulting_revision: self.revision.saturating_add(1),
            from: self.status,
            to,
            reason: reason.to_owned(),
        };
        self.status = to;
        self.revision = audit.resulting_revision;
        self.audit.push(audit.clone());
        Ok(audit)
    }
}

/// Write one canonical bounded frame.
pub fn write_frame<T: Serialize, W: Write>(writer: &mut W, value: &T) -> Result<(), ProofError> {
    let payload = canonical_json(value)?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(ProofError::Frame("outbound frame exceeds limit".into()));
    }
    let length = u32::try_from(payload.len())
        .map_err(|_| ProofError::Frame("outbound frame length overflow".into()))?;
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

/// Read one bounded strict canonical JSON frame, rejecting its declared length
/// before allocating the payload.
pub fn read_frame<T: DeserializeOwned + Serialize, R: Read>(
    reader: &mut R,
) -> Result<T, ProofError> {
    let mut header = [0_u8; 4];
    reader.read_exact(&mut header)?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(ProofError::Frame("declared frame length rejected".into()));
    }
    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload)?;
    let value: T = serde_json::from_slice(&payload)
        .map_err(|error| ProofError::Frame(format!("invalid frame JSON: {error}")))?;
    if canonical_json(&value)? != payload {
        return Err(ProofError::Frame("frame JSON is not canonical".into()));
    }
    Ok(value)
}

/// Read one regular component artifact through a single handle while enforcing
/// the byte ceiling during the read, before compilation or signature work.
pub fn read_bounded_artifact(path: &Path) -> Result<Vec<u8>, ProofError> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_ARTIFACT_BYTES as u64 {
        return Err(ProofError::Contract("component file rejected".into()));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    let mut limited = file.take(MAX_ARTIFACT_BYTES as u64 + 1);
    limited.read_to_end(&mut bytes)?;
    if bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(ProofError::Contract("component file rejected".into()));
    }
    Ok(bytes)
}

/// Deterministic valid fixture, with separate publisher, provenance, and core
/// signatures. The provenance class can vary without changing the runtime
/// contract or grant.
pub fn fixture_request(
    component_bytes: &[u8],
    provenance_class: ProvenanceClass,
) -> Result<HostRequest, ProofError> {
    let publisher_key = SigningKey::from_bytes(&[7_u8; 32]);
    let provenance_key = SigningKey::from_bytes(&[9_u8; 32]);
    let core_key = SigningKey::from_bytes(&[11_u8; 32]);
    let wit = WitIdentity {
        package: WIT_PACKAGE.to_owned(),
        world: WIT_WORLD.to_owned(),
        major: 1,
        sha256: wit_sha256(),
    };
    let budgets = ResourceBudgets {
        frame_bytes: MAX_FRAME_BYTES as u32,
        memory_bytes: MAX_LINEAR_MEMORY_BYTES as u32,
        fuel: MAX_FUEL,
        execution_ms: 500,
    };
    let release_manifest = ModuleReleaseManifest {
        format: RELEASE_FORMAT.to_owned(),
        module_id: "ignibyte.sentinel".into(),
        publisher_id: "ignibyte".into(),
        release_id: uuid("10000000-0000-4000-8000-000000000001")?,
        version: "1.0.0".into(),
        component_sha256: sha256_hex(component_bytes),
        wit: wit.clone(),
        requested_capabilities: vec![Capability::ModerationAddLabel],
        subscribed_hooks: vec![HookKind::PersonaReported],
        budgets: budgets.clone(),
        config_schema: "ignibyte.sentinel.config/v1".into(),
        state_schema: "ignibyte.sentinel.state/v1".into(),
        entrypoint: "handle".into(),
    };
    let release = SignedEnvelope::sign(
        RELEASE_FORMAT,
        PUBLISHER_KEY_ID,
        &release_manifest,
        &publisher_key,
    )?;
    let provenance_statement = ModuleProvenance {
        format: PROVENANCE_FORMAT.to_owned(),
        release_manifest_sha256: release.payload_sha256()?,
        provenance: provenance_class,
    };
    let provenance = SignedEnvelope::sign(
        PROVENANCE_FORMAT,
        PROVENANCE_KEY_ID,
        &provenance_statement,
        &provenance_key,
    )?;
    let server_id = uuid("20000000-0000-4000-8000-000000000002")?;
    let admission = ModuleAdmission {
        format: ADMISSION_FORMAT.to_owned(),
        server_id,
        admission_id: uuid("30000000-0000-4000-8000-000000000003")?,
        lifecycle_revision: 3,
        lifecycle: LifecycleStatus::Active,
        module_id: release_manifest.module_id.clone(),
        release_id: release_manifest.release_id,
        component_sha256: release_manifest.component_sha256.clone(),
        release_manifest_sha256: release.payload_sha256()?,
        provenance_sha256: provenance.payload_sha256()?,
        wit,
        granted_capabilities: vec![Capability::ModerationAddLabel],
        subscribed_hooks: vec![HookKind::PersonaReported],
        budgets,
        config_revision: 4,
        state_schema: release_manifest.state_schema.clone(),
        state_revision: 8,
    };
    let admission_envelope =
        SignedEnvelope::sign(ADMISSION_FORMAT, CORE_KEY_ID, &admission, &core_key)?;
    let event = ModuleHookEvent {
        format: HOOK_FORMAT.to_owned(),
        event_id: uuid("40000000-0000-4000-8000-000000000004")?,
        attempt: 1,
        server_id,
        module_id: release_manifest.module_id,
        release_id: release_manifest.release_id,
        admission_id: admission.admission_id,
        hook: HookKind::PersonaReported,
        causal_revision: 42,
        deadline_ms: 500,
        subject: ModuleSubject::Pairwise("pairwise-persona-7".into()),
        config: BTreeMap::from([("policy".into(), "strict".into())]),
        config_revision: admission.config_revision,
        state: BTreeMap::from([("observations".into(), "2".into())]),
        state_revision: admission.state_revision,
        payload: HookPayload::PersonaReported {
            report_id: uuid("50000000-0000-4000-8000-000000000005")?,
            category: "abuse".into(),
        },
    };
    Ok(HostRequest {
        release,
        publisher_public_key: encode_verifying_key(&publisher_key.verifying_key()),
        provenance,
        provenance_public_key: encode_verifying_key(&provenance_key.verifying_key()),
        admission: admission_envelope,
        core_public_key: encode_verifying_key(&core_key.verifying_key()),
        event,
    })
}

/// Digest the exact checked-in WIT source.
pub fn wit_sha256() -> String {
    sha256_hex(include_bytes!("../wit/omarchygs-module.wit"))
}

/// Deterministically componentize a checked-in proof fixture against the exact
/// WIT world. This builder is test tooling; [`ModuleRuntime`] itself accepts
/// only completed Component Model artifacts.
pub fn build_fixture_component(source: &[u8]) -> Result<Vec<u8>, ProofError> {
    if source.len() > MAX_ARTIFACT_BYTES {
        return Err(ProofError::Contract("fixture source exceeds limit".into()));
    }
    let source_text = std::str::from_utf8(source)
        .map_err(|_| ProofError::Contract("fixture source is not UTF-8".into()))?;
    if source_text.trim_start().starts_with("(component") {
        return wat::parse_bytes(source)
            .map(|bytes| bytes.into_owned())
            .map_err(|error| {
                ProofError::Execution(format!("component fixture parsing failed: {error:#}"))
            });
    }

    let mut module = wat::parse_bytes(source)
        .map(|bytes| bytes.into_owned())
        .map_err(|error| {
            ProofError::Execution(format!("core fixture parsing failed: {error:#}"))
        })?;
    let mut resolve = Resolve::default();
    let package = resolve
        .push_str(
            "omarchygs-module.wit",
            include_str!("../wit/omarchygs-module.wit"),
        )
        .map_err(|error| ProofError::Execution(format!("WIT parsing failed: {error:#}")))?;
    let world = resolve
        .select_world(&[package], Some(WIT_WORLD))
        .map_err(|error| ProofError::Execution(format!("WIT world selection failed: {error:#}")))?;
    embed_component_metadata(&mut module, &resolve, world, StringEncoding::UTF8)
        .map_err(|error| ProofError::Execution(format!("WIT embedding failed: {error:#}")))?;
    ComponentEncoder::default()
        .module(&module)
        .and_then(|encoder| encoder.validate(true).encode())
        .map_err(|error| {
            ProofError::Execution(format!("fixture componentization failed: {error:#}"))
        })
}

/// Hex SHA-256 helper used by the signed contracts.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ProofError> {
    serde_json::to_vec(value)
        .map_err(|error| ProofError::Contract(format!("JSON serialization failed: {error}")))
}

fn signature_message(format: &str, payload: &[u8]) -> Vec<u8> {
    let mut message = b"OmarchyGS server module signed document\0".to_vec();
    message.extend_from_slice(format.as_bytes());
    message.push(0);
    message.extend_from_slice(payload);
    message
}

fn decode_bounded(encoded: &str, max: usize, name: &str) -> Result<Vec<u8>, ProofError> {
    if encoded.len() > max.saturating_mul(2).saturating_add(8) {
        return Err(ProofError::Contract(format!(
            "{name} encoding exceeds limit"
        )));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ProofError::Contract(format!("{name} is not canonical base64url")))?;
    if bytes.len() > max || URL_SAFE_NO_PAD.encode(&bytes) != encoded {
        return Err(ProofError::Contract(format!(
            "{name} exceeds or is not canonical"
        )));
    }
    Ok(bytes)
}

fn encode_verifying_key(key: &VerifyingKey) -> String {
    URL_SAFE_NO_PAD.encode(key.as_bytes())
}

fn decode_verifying_key(encoded: &str) -> Result<VerifyingKey, ProofError> {
    let bytes = decode_bounded(encoded, 32, "public key")?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| ProofError::Integrity("invalid public-key length".into()))?;
    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| ProofError::Integrity("invalid public key".into()))
}

fn validate_wit(wit: &WitIdentity) -> Result<(), ProofError> {
    if wit.package != WIT_PACKAGE || wit.world != WIT_WORLD || wit.major != 1 {
        return Err(ProofError::Contract("unsupported WIT identity".into()));
    }
    validate_digest(&wit.sha256)?;
    if wit.sha256 != wit_sha256() {
        return Err(ProofError::Integrity("WIT digest mismatch".into()));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), ProofError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ProofError::Contract("invalid lowercase SHA-256".into()));
    }
    Ok(())
}

fn validate_identifier(name: &str, value: &str, max: usize) -> Result<(), ProofError> {
    if value.is_empty()
        || value.len() > max
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
    {
        return Err(ProofError::Contract(format!("invalid {name}")));
    }
    Ok(())
}

fn validate_text(name: &str, value: &str, min: usize, max: usize) -> Result<(), ProofError> {
    if value.len() < min
        || value.len() > max
        || value.chars().any(char::is_control)
        || value.trim() != value
    {
        return Err(ProofError::Contract(format!("invalid {name}")));
    }
    Ok(())
}

fn validate_semver(value: &str) -> Result<(), ProofError> {
    let parts: Vec<&str> = value.split('.').collect();
    if value.len() > 32
        || parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty()
                || !part.bytes().all(|byte| byte.is_ascii_digit())
                || (part.len() > 1 && part.starts_with('0'))
        })
    {
        return Err(ProofError::Contract(
            "invalid proof semantic version".into(),
        ));
    }
    Ok(())
}

fn validate_sorted_unique<T: Ord + Display>(name: &str, values: &[T]) -> Result<(), ProofError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ProofError::Contract(format!(
            "{name} are not sorted/unique"
        )));
    }
    Ok(())
}

impl Display for Capability {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Display for HookKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

fn is_subset<T: Ord>(subset: &[T], superset: &[T]) -> bool {
    subset
        .iter()
        .all(|item| superset.binary_search(item).is_ok())
}

fn validate_snapshot(
    name: &str,
    values: &BTreeMap<String, String>,
    max_entries: usize,
    max_bytes: usize,
) -> Result<(), ProofError> {
    if values.len() > max_entries {
        return Err(ProofError::Contract(format!("{name} has too many entries")));
    }
    let mut total = 0_usize;
    for (key, value) in values {
        validate_identifier(&format!("{name} key"), key, 64)?;
        validate_text(&format!("{name} value"), value, 0, 512)?;
        total = total.saturating_add(key.len()).saturating_add(value.len());
    }
    if total > max_bytes {
        return Err(ProofError::Contract(format!("{name} exceeds quota")));
    }
    Ok(())
}

fn apply_state_operations(
    current: &BTreeMap<String, String>,
    operations: &[StateOperation],
) -> Result<BTreeMap<String, String>, ProofError> {
    if operations.is_empty() || operations.len() > 32 {
        return Err(ProofError::Contract("invalid state operation count".into()));
    }
    let mut next = current.clone();
    for operation in operations {
        match operation {
            StateOperation::Set { key, value } => {
                validate_identifier("state key", key, 64)?;
                validate_text("state value", value, 0, 512)?;
                next.insert(key.clone(), value.clone());
            }
            StateOperation::Remove { key } => {
                validate_identifier("state key", key, 64)?;
                next.remove(key);
            }
        }
    }
    validate_snapshot("state", &next, 32, 4096)?;
    Ok(next)
}

fn legal_transition(from: LifecycleStatus, to: LifecycleStatus) -> bool {
    matches!(
        (from, to),
        (LifecycleStatus::Staged, LifecycleStatus::Disabled)
            | (LifecycleStatus::Disabled, LifecycleStatus::Enabling)
            | (LifecycleStatus::Enabling, LifecycleStatus::Active)
            | (LifecycleStatus::Enabling, LifecycleStatus::Disabled)
            | (LifecycleStatus::Active, LifecycleStatus::Degraded)
            | (LifecycleStatus::Active, LifecycleStatus::Suspended)
            | (LifecycleStatus::Active, LifecycleStatus::Disabled)
            | (LifecycleStatus::Degraded, LifecycleStatus::Disabled)
            | (LifecycleStatus::Degraded, LifecycleStatus::Suspended)
            | (LifecycleStatus::Suspended, LifecycleStatus::Disabled)
            | (LifecycleStatus::Disabled, LifecycleStatus::Retired)
    )
}

fn uuid(value: &str) -> Result<Uuid, ProofError> {
    Uuid::parse_str(value).map_err(|_| ProofError::Contract("invalid fixture UUID".into()))
}
