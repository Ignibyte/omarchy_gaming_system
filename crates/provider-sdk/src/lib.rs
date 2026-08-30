//! Public, provider-facing OmarchyGS protocol and release contract.
//!
//! This crate intentionally contains no platform registry, broker, egress,
//! database, administrator, or provider-admission implementation. Publishing
//! or consuming it grants no authority to register or activate a provider.

pub mod model;
pub mod protocol;
pub mod release;

pub use model::ProviderScope;

use thiserror::Error;

/// A bounded provider-boundary failure. Variants intentionally carry no remote
/// body, URL, credential, database string, or cryptographic material.
#[derive(Debug, Error)]
pub enum ProviderError {
    /// The caller supplied malformed or out-of-policy input.
    #[error("invalid provider input")]
    InvalidInput,
    /// The exact registered resource does not exist.
    #[error("provider resource not found")]
    NotFound,
    /// An immutable or idempotent identity conflicts with existing state.
    #[error("provider state conflict")]
    Conflict,
    /// Current lifecycle, key, scope, or session policy denies the operation.
    #[error("provider operation denied")]
    Denied,
    /// A registered quota or concurrency ceiling was reached.
    #[error("provider quota exceeded")]
    QuotaExceeded,
    /// An authenticated protocol message was malformed, invalid, or mismatched.
    #[error("provider protocol rejected")]
    ProtocolRejected,
    /// The guarded remote transport did not produce an authenticated result.
    #[error("provider unavailable")]
    Unavailable,
    /// Durable storage or another internal boundary failed closed.
    #[error("provider internal failure")]
    Internal,
}

impl ProviderError {
    /// Stable non-disclosing error code suitable for logs or API mapping.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput => "provider_invalid_input",
            Self::NotFound => "provider_not_found",
            Self::Conflict => "provider_conflict",
            Self::Denied => "provider_denied",
            Self::QuotaExceeded => "provider_quota_exceeded",
            Self::ProtocolRejected => "provider_protocol_rejected",
            Self::Unavailable => "provider_unavailable",
            Self::Internal => "provider_internal",
        }
    }
}

/// Result type used throughout the public provider boundary.
pub type Result<T> = std::result::Result<T, ProviderError>;
