//! Trusted same-user acquisition, verification, and profile-mount runtime for
//! the OmarchyGS flagship client.

mod cache;
mod remote;
mod render;
mod service;

pub use cache::{ClientCartridgeCache, MountRecord};
pub use remote::{AcquireRequest, SessionAcquireRequest, acquire, acquire_session};
pub use render::{RenderRequest, compile_mounted_render_plan};
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
    #[error("the exact server-profile cartridge mount is absent")]
    MountMissing,
    #[error("local cartridge cache operation failed")]
    Cache,
    #[error("trusted cartridge render-plan compilation failed")]
    Render,
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
            Self::MountMissing => "companion_mount_missing",
            Self::Cache => "companion_cache_failure",
            Self::Render => "companion_render_failure",
        }
    }
}

pub type Result<T> = std::result::Result<T, CompanionError>;
