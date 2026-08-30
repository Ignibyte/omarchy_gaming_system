//! Validated provider registry, lifecycle, scope, and operator input models.

use std::{collections::BTreeSet, net::IpAddr};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ProviderError, Result};

pub use omarchygs_provider_sdk::ProviderScope;

/// Maximum accepted operator command document.
pub const MAX_OPERATOR_DOCUMENT_BYTES: usize = 256 * 1024;
/// Maximum decoded TLS trust anchor.
pub const MAX_TLS_ROOT_BYTES: usize = 32 * 1024;
/// Exact Ed25519 public-key length.
pub const ED25519_PUBLIC_KEY_BYTES: usize = 32;

/// Operator-controlled lifecycle for the one enabled remote gameplay pilot.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PilotStatus {
    /// Advertise the pilot and admit new and existing gameplay.
    Active,
    /// Hide the pilot and deny new launches while preserving recovery data.
    Suspended,
    /// Permanently disable the pilot. This transition is terminal.
    Retired,
}

impl PilotStatus {
    /// Stable database representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Retired => "retired",
        }
    }
}

/// One platform-owned achievement definition accepted from an exact release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AchievementDefinitionInput {
    /// Canonical game-scoped claim key.
    pub key: String,
    /// Public display label.
    pub display_name: String,
    /// Public bounded explanation.
    pub description: String,
}

impl AchievementDefinitionInput {
    fn validate(&self) -> Result<()> {
        if !is_identifier(&self.key, 2, 48, b"-_")
            || !is_display_name(&self.display_name)
            || self.display_name.chars().count() > 96
            || self.description.is_empty()
            || self.description.chars().count() > 256
            || self.description.chars().any(char::is_control)
        {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

/// Complete operator-pinned public policy for the sole remote pilot release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActivatePilotInput {
    /// Exact previously registered release.
    pub release_id: Uuid,
    /// Public catalog label.
    pub display_name: String,
    /// Minimum supported human participants.
    pub min_human_players: u8,
    /// Maximum supported human participants.
    pub max_human_players: u8,
    /// Exact platform-owned achievement allowlist.
    pub achievements: Vec<AchievementDefinitionInput>,
}

impl ActivatePilotInput {
    /// Validate public policy before database work.
    pub fn validate(&self) -> Result<()> {
        if self.release_id.is_nil()
            || !is_display_name(&self.display_name)
            || self.display_name.chars().count() > 64
            || self.min_human_players == 0
            || self.max_human_players < self.min_human_players
            || self.max_human_players > 8
            || self.achievements.len() > 64
        {
            return Err(ProviderError::InvalidInput);
        }
        require_unique(
            self.achievements
                .iter()
                .map(|definition| definition.key.as_str()),
        )?;
        self.achievements
            .iter()
            .try_for_each(AchievementDefinitionInput::validate)
    }
}

/// Mutable lifecycle for providers, releases, scopes, and operational keys.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    /// Admits operations allowed by the remaining exact policy.
    Active,
    /// Temporarily denies or narrows operations under active-session policy.
    Suspended,
    /// Terminal immediate denial.
    Revoked,
}

impl LifecycleStatus {
    /// Stable database representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
        }
    }

    #[cfg(feature = "platform")]
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "revoked" => Ok(Self::Revoked),
            _ => Err(ProviderError::Internal),
        }
    }
}

/// Pinned handling for already-existing sessions during suspension.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActiveSessionPolicy {
    /// Deny every operation.
    Terminate,
    /// Permit authenticated reconciliation only.
    ReadOnly,
    /// Permit commands and reconciliation for a proven existing session.
    Continue,
}

impl ActiveSessionPolicy {
    /// Stable database representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Terminate => "terminate",
            Self::ReadOnly => "read_only",
            Self::Continue => "continue",
        }
    }

    #[cfg(feature = "platform")]
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "terminate" => Ok(Self::Terminate),
            "read_only" => Ok(Self::ReadOnly),
            "continue" => Ok(Self::Continue),
            _ => Err(ProviderError::Internal),
        }
    }
}

/// Immutable operational public-key family.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationalKeyKind {
    /// Ed25519 provider HTTP-message verification key.
    MessageEd25519,
    /// DER X.509 certificate trusted as a TLS root for the exact endpoint.
    TlsRootDer,
}

impl OperationalKeyKind {
    /// Stable database representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MessageEd25519 => "message_ed25519",
            Self::TlsRootDer => "tls_root_der",
        }
    }
}

/// Bounded, exact quotas pinned to one release configuration revision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderQuotas {
    /// Grant issuances admitted per UTC database minute.
    pub grants_per_minute: u32,
    /// Outbound requests admitted per UTC database minute.
    pub requests_per_minute: u32,
    /// Inbound callbacks admitted per UTC database minute.
    pub callbacks_per_minute: u32,
    /// Cross-process in-flight request ceiling.
    pub max_concurrent_requests: u16,
    /// Maximum serialized outbound body.
    pub request_body_bytes: u32,
    /// Maximum streamed response or callback body.
    pub response_body_bytes: u32,
    /// TCP/TLS connection deadline.
    pub connect_timeout_ms: u32,
    /// Complete request/response deadline.
    pub total_timeout_ms: u32,
}

impl ProviderQuotas {
    /// Validate every registered quota against the v1 production envelope.
    pub fn validate(&self) -> Result<()> {
        if !(1..=10_000).contains(&self.grants_per_minute)
            || !(1..=10_000).contains(&self.requests_per_minute)
            || !(1..=10_000).contains(&self.callbacks_per_minute)
            || !(1..=64).contains(&self.max_concurrent_requests)
            || !(1024..=65_536).contains(&self.request_body_bytes)
            || !(1024..=524_288).contains(&self.response_body_bytes)
            || !(100..=5_000).contains(&self.connect_timeout_ms)
            || !(250..=15_000).contains(&self.total_timeout_ms)
            || self.total_timeout_ms < self.connect_timeout_ms
        {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

/// Canonical registered HTTPS destination.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderEndpoint {
    /// Lowercase DNS host. IP literals are prohibited.
    pub host: String,
    /// Explicit TLS port.
    #[serde(default = "default_https_port")]
    pub port: u16,
    /// Absolute canonical path prefix ending in `/`.
    pub base_path: String,
}

const fn default_https_port() -> u16 {
    443
}

impl ProviderEndpoint {
    /// Validate the endpoint without performing DNS or network I/O.
    pub fn validate(&self) -> Result<()> {
        if !is_dns_name(&self.host)
            || self.host.parse::<IpAddr>().is_ok()
            || self.host.ends_with(".local")
            || self.host.ends_with(".localhost")
            || self.base_path.is_empty()
            || self.base_path.len() > 256
            || !self.base_path.starts_with('/')
            || !self.base_path.ends_with('/')
            || self.base_path.contains("//")
            || self
                .base_path
                .split('/')
                .any(|part| part == "." || part == "..")
            || !self.base_path.is_ascii()
            || self.base_path.bytes().any(|byte| {
                !(byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'/' | b'-' | b'_'))
            })
        {
            Err(ProviderError::InvalidInput)
        } else {
            Ok(())
        }
    }

    /// Canonical RFC 9421 authority value.
    #[must_use]
    pub fn authority(&self) -> String {
        if self.port == 443 {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    /// Build an operation URL from one allowlisted single path segment.
    pub fn operation_url(&self, operation: &str) -> Result<url::Url> {
        self.validate()?;
        if !is_identifier(operation, 2, 32, b"-_") {
            return Err(ProviderError::InvalidInput);
        }
        url::Url::parse(&format!(
            "https://{}:{}{}{}",
            self.host, self.port, self.base_path, operation
        ))
        .map_err(|_| ProviderError::InvalidInput)
    }
}

/// One immutable public operational key supplied by an operator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperationalKeyInput {
    /// Canonical key identity.
    pub key_id: String,
    /// Standard base64-encoded public material.
    pub public_material_base64: String,
    /// Inclusive validity start as Unix seconds.
    pub valid_from: i64,
    /// Exclusive optional validity end as Unix seconds.
    pub valid_until: Option<i64>,
}

impl OperationalKeyInput {
    /// Validate and decode public material for the declared kind.
    pub fn decode(&self, kind: OperationalKeyKind) -> Result<Vec<u8>> {
        if !is_identifier(&self.key_id, 3, 64, b"._-")
            || self
                .valid_until
                .is_some_and(|valid_until| valid_until <= self.valid_from)
            || self.public_material_base64.len() > MAX_TLS_ROOT_BYTES * 2
        {
            return Err(ProviderError::InvalidInput);
        }
        let decoded = STANDARD
            .decode(&self.public_material_base64)
            .map_err(|_| ProviderError::InvalidInput)?;
        let length_valid = match kind {
            OperationalKeyKind::MessageEd25519 => decoded.len() == ED25519_PUBLIC_KEY_BYTES,
            OperationalKeyKind::TlsRootDer => (64..=MAX_TLS_ROOT_BYTES).contains(&decoded.len()),
        };
        if length_valid {
            Ok(decoded)
        } else {
            Err(ProviderError::InvalidInput)
        }
    }
}

/// Complete exact provider release registration input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RegisterReleaseInput {
    /// Canonical operator-controlled provider identity.
    pub provider_id: String,
    /// Human-readable operator label.
    pub display_name: String,
    /// Stable release UUID.
    pub release_id: Uuid,
    /// Exact canonical game key.
    pub game_key: String,
    /// Exact positive rules version.
    pub rules_version: u32,
    /// Lowercase SHA-256 cartridge digest.
    pub cartridge_digest: String,
    /// Registered HTTPS destination.
    pub endpoint: ProviderEndpoint,
    /// Pinned handling for existing sessions during suspension.
    pub active_session_policy: ActiveSessionPolicy,
    /// Exact allowed capabilities.
    pub scopes: Vec<ProviderScope>,
    /// Initial provider message keys.
    pub message_keys: Vec<OperationalKeyInput>,
    /// Initial TLS roots trusted for this endpoint.
    pub tls_roots: Vec<OperationalKeyInput>,
    /// Exact release quotas.
    pub quotas: ProviderQuotas,
}

impl RegisterReleaseInput {
    /// Validate a complete registration before any database work.
    pub fn validate(&self) -> Result<()> {
        if !is_identifier(&self.provider_id, 3, 64, b"-_")
            || !is_display_name(&self.display_name)
            || self.release_id.is_nil()
            || !is_identifier(&self.game_key, 3, 32, b"-_")
            || self.rules_version == 0
            || !is_sha256_hex(&self.cartridge_digest)
            || self.scopes.is_empty()
            || self.message_keys.is_empty()
            || self.tls_roots.is_empty()
        {
            return Err(ProviderError::InvalidInput);
        }
        self.endpoint.validate()?;
        self.quotas.validate()?;
        require_unique(self.scopes.iter().copied())?;
        require_unique(self.message_keys.iter().map(|key| key.key_id.as_str()))?;
        require_unique(self.tls_roots.iter().map(|key| key.key_id.as_str()))?;
        for key in &self.message_keys {
            key.decode(OperationalKeyKind::MessageEd25519)?;
        }
        for key in &self.tls_roots {
            key.decode(OperationalKeyKind::TlsRootDer)?;
        }
        Ok(())
    }
}

/// One bounded operator control-plane command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum OperatorCommand {
    /// Register one provider and exact release atomically.
    RegisterRelease {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Complete exact registration.
        registration: RegisterReleaseInput,
    },
    /// Append a new immutable overlapping operational key.
    RotateKey {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Exact release.
        release_id: Uuid,
        /// Public-key family.
        key_kind: OperationalKeyKind,
        /// New immutable key.
        key: OperationalKeyInput,
    },
    /// Change provider lifecycle.
    SetProviderStatus {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Exact provider.
        provider_id: String,
        /// Requested lifecycle.
        status: LifecycleStatus,
    },
    /// Change one release lifecycle.
    SetReleaseStatus {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Exact release.
        release_id: Uuid,
        /// Requested lifecycle.
        status: LifecycleStatus,
    },
    /// Change one registered scope lifecycle.
    SetScopeStatus {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Exact release.
        release_id: Uuid,
        /// Exact scope.
        scope: ProviderScope,
        /// Requested lifecycle.
        status: LifecycleStatus,
    },
    /// Change one operational key lifecycle.
    SetKeyStatus {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Exact release.
        release_id: Uuid,
        /// Public-key family.
        key_kind: OperationalKeyKind,
        /// Exact key ID.
        key_id: String,
        /// Requested lifecycle.
        status: LifecycleStatus,
    },
    /// Replace bounded quotas and advance configuration revision.
    UpdateQuotas {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Exact release.
        release_id: Uuid,
        /// Complete replacement quotas.
        quotas: ProviderQuotas,
    },
    /// Enable one exact registered release as the sole remote gameplay pilot.
    ActivatePilot {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Complete public pilot policy.
        pilot: ActivatePilotInput,
    },
    /// Suspend, restore, or permanently retire an activated pilot.
    SetPilotStatus {
        /// Audited operator identity.
        actor: String,
        /// Audited reason.
        reason: String,
        /// Exact pilot release.
        release_id: Uuid,
        /// Requested pilot lifecycle.
        status: PilotStatus,
    },
}

impl OperatorCommand {
    /// Validate all operator-controlled values without database access.
    pub fn validate(&self) -> Result<()> {
        let (actor, reason) = self.actor_reason();
        if !is_actor(actor) || !is_reason(reason) {
            return Err(ProviderError::InvalidInput);
        }
        match self {
            Self::RegisterRelease { registration, .. } => registration.validate(),
            Self::RotateKey {
                release_id,
                key_kind,
                key,
                ..
            } => {
                if release_id.is_nil() {
                    return Err(ProviderError::InvalidInput);
                }
                key.decode(*key_kind).map(|_| ())
            }
            Self::SetProviderStatus { provider_id, .. } => {
                if is_identifier(provider_id, 3, 64, b"-_") {
                    Ok(())
                } else {
                    Err(ProviderError::InvalidInput)
                }
            }
            Self::SetReleaseStatus { release_id, .. }
            | Self::SetScopeStatus { release_id, .. }
            | Self::UpdateQuotas { release_id, .. }
            | Self::SetPilotStatus { release_id, .. } => {
                if release_id.is_nil() {
                    Err(ProviderError::InvalidInput)
                } else if let Self::UpdateQuotas { quotas, .. } = self {
                    quotas.validate()
                } else {
                    Ok(())
                }
            }
            Self::SetKeyStatus {
                release_id, key_id, ..
            } => {
                if release_id.is_nil() || !is_identifier(key_id, 3, 64, b"._-") {
                    Err(ProviderError::InvalidInput)
                } else {
                    Ok(())
                }
            }
            Self::ActivatePilot { pilot, .. } => pilot.validate(),
        }
    }

    pub(crate) fn actor_reason(&self) -> (&str, &str) {
        match self {
            Self::RegisterRelease { actor, reason, .. }
            | Self::RotateKey { actor, reason, .. }
            | Self::SetProviderStatus { actor, reason, .. }
            | Self::SetReleaseStatus { actor, reason, .. }
            | Self::SetScopeStatus { actor, reason, .. }
            | Self::SetKeyStatus { actor, reason, .. }
            | Self::UpdateQuotas { actor, reason, .. }
            | Self::ActivatePilot { actor, reason, .. }
            | Self::SetPilotStatus { actor, reason, .. } => (actor, reason),
        }
    }
}

/// Session evidence supplied by the future platform envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAdmission {
    /// No provider session exists yet.
    New,
    /// The platform envelope proves an existing provider session.
    Existing,
}

/// Exact release policy loaded from PostgreSQL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleasePolicy {
    /// Provider identity.
    pub provider_id: String,
    /// Provider lifecycle.
    pub provider_status: LifecycleStatus,
    /// Exact release UUID.
    pub release_id: Uuid,
    /// Exact game identity.
    pub game_key: String,
    /// Exact rules version.
    pub rules_version: u32,
    /// Exact cartridge digest.
    pub cartridge_digest: String,
    /// Registered HTTPS endpoint.
    pub endpoint: ProviderEndpoint,
    /// Release lifecycle.
    pub release_status: LifecycleStatus,
    /// Suspension policy.
    pub active_session_policy: ActiveSessionPolicy,
    /// Current configuration revision.
    pub config_revision: u64,
    /// Current quotas.
    pub quotas: ProviderQuotas,
}

fn require_unique<T>(values: impl IntoIterator<Item = T>) -> Result<()>
where
    T: Ord,
{
    let mut unique = BTreeSet::new();
    if values.into_iter().all(|value| unique.insert(value)) {
        Ok(())
    } else {
        Err(ProviderError::InvalidInput)
    }
}

pub(crate) fn is_identifier(value: &str, min: usize, max: usize, extras: &[u8]) -> bool {
    let bytes = value.as_bytes();
    (min..=max).contains(&bytes.len())
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || extras.contains(byte))
}

pub(crate) fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_dns_name(value: &str) -> bool {
    value.len() <= 253
        && value.contains('.')
        && value == value.to_ascii_lowercase()
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        })
}

fn is_display_name(value: &str) -> bool {
    let characters = value.chars().count();
    (1..=96).contains(&characters) && !value.chars().any(char::is_control)
}

fn is_actor(value: &str) -> bool {
    let characters = value.chars().count();
    (1..=96).contains(&characters) && !value.chars().any(char::is_control)
}

fn is_reason(value: &str) -> bool {
    let characters = value.chars().count();
    (3..=512).contains(&characters) && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_is_dns_only_and_operation_is_one_segment() {
        let endpoint = ProviderEndpoint {
            host: "games.example.test".to_owned(),
            port: 443,
            base_path: "/omarchygs/provider/v1/".to_owned(),
        };
        assert!(endpoint.validate().is_ok());
        assert_eq!(endpoint.authority(), "games.example.test");
        assert_eq!(
            endpoint
                .operation_url("commands")
                .expect("canonical operation should parse")
                .as_str(),
            "https://games.example.test/omarchygs/provider/v1/commands"
        );
        for host in ["127.0.0.1", "localhost", "games.local", "UP.example.test"] {
            let mut invalid = endpoint.clone();
            invalid.host = host.to_owned();
            assert!(invalid.validate().is_err(), "{host} should reject");
        }
        assert!(endpoint.operation_url("../admin").is_err());
    }

    #[test]
    fn quotas_enforce_the_complete_v1_envelope() {
        let valid = ProviderQuotas {
            grants_per_minute: 10,
            requests_per_minute: 20,
            callbacks_per_minute: 20,
            max_concurrent_requests: 4,
            request_body_bytes: 8192,
            response_body_bytes: 65_536,
            connect_timeout_ms: 500,
            total_timeout_ms: 2_000,
        };
        assert!(valid.validate().is_ok());
        let mut invalid = valid.clone();
        invalid.max_concurrent_requests = 0;
        assert!(invalid.validate().is_err());
        let mut invalid = valid;
        invalid.total_timeout_ms = 250;
        assert!(invalid.validate().is_err());
    }
}
