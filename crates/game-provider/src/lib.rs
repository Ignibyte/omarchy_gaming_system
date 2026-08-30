//! Production security and control-plane foundation for registered game providers.
//!
//! This crate is intentionally dormant with respect to player-facing gameplay.
//! It supplies operator registration, exact release policy, authenticated
//! protocol messages, guarded egress, durable replay/quota/audit state, and a
//! conformance boundary for a later authority-migration pipeline.

#[cfg(feature = "platform")]
pub mod broker;
#[cfg(feature = "platform")]
pub mod egress;
pub mod model;
pub mod protocol {
    //! Source-compatible re-export of the public SDK protocol.
    pub use omarchygs_provider_sdk::protocol::*;
}
#[cfg(feature = "platform")]
pub mod registry;

pub use omarchygs_provider_sdk::{ProviderError, Result};
