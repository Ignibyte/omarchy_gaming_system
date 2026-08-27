//! Trusted same-user acquisition, verification, and profile-mount runtime for
//! the OmarchyGS flagship client.

mod cache;
mod remote;
mod service;

pub use cache::{ClientCartridgeCache, MountRecord};
pub use remote::{AcquireRequest, acquire};
pub use service::{CompanionState, router};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompanionError {
    #[error("invalid companion request")]
    InvalidInput,
    #[error("companion authorization failed")]
    Unauthorized,
    #[error("selected server is unavailable")]
    Unavailable,
    #[error("selected server response was rejected")]
    Rejected,
    #[error("no independently trusted marketplace key is configured")]
    MarketplaceUntrusted,
    #[error("cartridge is no longer admitted")]
    AdmissionChanged,
    #[error("local cartridge cache operation failed")]
    Cache,
}

impl CompanionError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput => "companion_invalid_input",
            Self::Unauthorized => "companion_unauthorized",
            Self::Unavailable => "companion_server_unavailable",
            Self::Rejected => "companion_server_rejected",
            Self::MarketplaceUntrusted => "companion_marketplace_untrusted",
            Self::AdmissionChanged => "companion_admission_changed",
            Self::Cache => "companion_cache_failure",
        }
    }
}

pub type Result<T> = std::result::Result<T, CompanionError>;
