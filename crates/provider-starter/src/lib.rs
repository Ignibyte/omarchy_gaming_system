//! Public game-agnostic backend starter for OmarchyGS providers.
//!
//! This crate owns provider-side protocol admission, durable operation
//! receipts, generic JSON game state, and callback delivery. It deliberately
//! owns no OmarchyGS registry, broker, platform database, operator command, or
//! provider-admission authority.

mod callback;
mod rules;
mod runtime;
mod store;

pub use callback::CallbackConfig;
pub use rules::{GameEvent, GameIdentity, GameState, GameTransition, ProviderGame};
pub use runtime::{ProviderStarter, ProviderStarterConfig, StarterLimits};

/// Embedded forward-only schema for a provider-owned PostgreSQL database.
pub const PROVIDER_STARTER_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
