//! Trusted same-user acquisition, verification, and profile-mount runtime for
//! the OmarchyGS flagship client.

mod cache;
mod package_channel;
mod remote;
mod render;
mod service;
mod trust;

pub use cache::{
    ClientCartridgeCache, MountRecord, OperatorCustomMountProvenance, OperatorCustomMountRecord,
    OperatorCustomTrust,
};
pub use package_channel::{ClientPackageChannel, ClientPackageStatus, StagedPackage};
pub use remote::{
    AcquireRequest, OperatorCustomDiscovery, SessionAcquireRequest, acquire,
    acquire_operator_custom, acquire_operator_custom_session, acquire_session,
    discover_operator_custom,
};
pub use render::{RenderRequest, compile_mounted_render_plan};
pub use service::{CompanionState, router};
pub use trust::{ClientMarketplaceTrust, ClientTrustSnapshot, ClientTrustStore, TrustStatus};

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
    #[error("the server operator custom-cartridge key is not explicitly trusted")]
    OperatorCustomUntrusted,
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
            Self::OperatorCustomUntrusted => "companion_operator_custom_untrusted",
            Self::AdmissionChanged => "companion_admission_changed",
            Self::MountMissing => "companion_mount_missing",
            Self::Cache => "companion_cache_failure",
            Self::Render => "companion_render_failure",
        }
    }
}

pub type Result<T> = std::result::Result<T, CompanionError>;
