//! Public, finite provider conformance and deterministic developer-kit release.
//!
//! This crate has no provider registry, admission, discovery, platform
//! database, account identity, device credential, or publication authority.

mod callback;
mod release;
mod runner;

pub use callback::{CallbackObservation, CallbackSink, CallbackSinkConfig};
pub use release::{
    DeveloperKitIdentity, DeveloperKitReleaseSigner, export_developer_kit, verify_developer_kit,
};
pub use runner::{
    ConformanceCase, ConformanceReceipt, ConformanceTarget, REQUIRED_CASES, run_conformance,
};
