use omarchygs_provider_sdk::{
    ProviderError, Result,
    protocol::{ProviderEventKind, ProviderSessionStatus},
};
use serde_json::Value;

/// Immutable identity of one provider-owned game release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameIdentity {
    pub provider_id: String,
    pub game_key: String,
    pub rules_version: u32,
    pub cartridge_digest: String,
}

impl GameIdentity {
    pub(crate) fn validate(&self) -> Result<()> {
        if !identifier(&self.provider_id, 3, 64)
            || !identifier(&self.game_key, 3, 32)
            || self.rules_version == 0
            || self.cartridge_digest.len() != 64
            || !self
                .cartridge_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ProviderError::InvalidInput);
        }
        Ok(())
    }
}

fn identifier(value: &str, minimum: usize, maximum: usize) -> bool {
    (minimum..=maximum).contains(&value.len())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

/// Game-owned durable state and lifecycle, with revision owned by the starter.
#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub status: ProviderSessionStatus,
    pub state: Value,
}

/// One deterministic command transition.
#[derive(Debug, Clone, PartialEq)]
pub struct GameTransition {
    pub status: ProviderSessionStatus,
    pub state: Value,
}

/// Optional callback facts emitted atomically with an applied transition.
#[derive(Debug, Clone, PartialEq)]
pub struct GameEvent {
    pub kind: ProviderEventKind,
    pub payload: Value,
}

/// Narrow deterministic rule boundary implemented by a provider game.
///
/// Implementations never receive transport headers, signing material,
/// database handles, callback targets, account/persona identifiers, or
/// platform credentials.
pub trait ProviderGame: Clone + Send + Sync + 'static {
    /// Exact provider/game identity pinned to the configured database.
    fn identity(&self) -> &GameIdentity;

    /// Create revision-zero state from one authenticated launch payload.
    fn launch(&self, payload: &Value) -> Result<GameState>;

    /// Apply one command to authenticated current state.
    fn command(&self, current: &GameState, payload: &Value) -> Result<GameTransition>;

    /// Produce bounded presentation facts from current durable state.
    fn view(&self, current: &GameState) -> Result<Value>;

    /// Produce an optional callback from the newly committed state.
    fn event(&self, current: &GameState) -> Result<Option<GameEvent>>;
}
