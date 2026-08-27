//! Bounded, deterministic Omarchy Gaming System cartridge contracts.
//!
//! This crate verifies inert data packages. It intentionally has no network,
//! database, platform-credential, dynamic-library, or executable-code path.

mod acquisition;
mod archive;
mod compatibility;
mod contract;
mod error;
mod io;
mod keys;
mod lifecycle;
mod marketplace;
mod release;
mod sdk;
mod secure_store;
mod store;
mod validate;

pub use acquisition::*;
pub use archive::{pack_directory, verify_archive, verify_archive_bytes};
pub use compatibility::{
    baseline_host_profile, core_host_profile, evaluate_compatibility, rich_2d_host_profile,
};
pub use contract::*;
pub use error::{CartridgeError, Result};
pub use keys::{
    PublisherPrivateKey, PublisherPublicKey, generate_keypair, read_private_key, read_public_key,
};
pub use lifecycle::*;
pub use marketplace::*;
pub use release::*;
pub use sdk::*;
pub use secure_store::*;
pub use store::{install_cartridge, resolve_active_cartridge, revoke_cartridge};
pub use validate::{
    NAVIGATION_ACTION_PREFIX, navigation_target, validate_entry_screen_action,
    validate_screen_action,
};
