//! Provider-facing capability model.

use serde::{Deserialize, Serialize};

/// One provider capability. Grants always carry exactly one non-event scope.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProviderScope {
    /// Create or resume a provider-owned gameplay session.
    #[serde(rename = "game.launch")]
    Launch,
    /// Apply one revision-aware idempotent command.
    #[serde(rename = "game.command")]
    Command,
    /// Query authoritative provider state and receipts.
    #[serde(rename = "game.reconcile")]
    Reconcile,
    /// Deliver an authenticated provider event to the platform boundary.
    #[serde(rename = "game.event")]
    Event,
}

impl ProviderScope {
    /// Stable protocol/database scope string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Launch => "game.launch",
            Self::Command => "game.command",
            Self::Reconcile => "game.reconcile",
            Self::Event => "game.event",
        }
    }
}

pub(crate) fn is_identifier(value: &str, min: usize, max: usize, extra: &[u8]) -> bool {
    let bytes = value.as_bytes();
    (min..=max).contains(&bytes.len())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_uppercase()
                || byte.is_ascii_digit()
                || extra.contains(byte)
        })
}

pub(crate) fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
