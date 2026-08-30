use omarchygs_provider_sdk::{ProviderError, Result, protocol::{
    ProviderEventKind, ProviderSessionStatus,
}};
use omarchygs_provider_starter::{
    GameEvent, GameIdentity, GameState, GameTransition, ProviderGame,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Clean-room deterministic Relay Forge rules. Transport, storage, signing,
/// callback delivery, and process lifecycle are owned by the starter crate.
#[derive(Debug, Clone)]
pub struct RelayForge {
    identity: GameIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RelayState {
    ore: u8,
    energy: u8,
    round: u8,
    forged: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LaunchPayload {
    player_count: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandEnvelope {
    command: RelayCommand,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RelayCommand {
    action: String,
}

impl RelayForge {
    /// Bind Relay Forge to one exact reviewed cartridge digest.
    #[must_use]
    pub fn new(cartridge_digest: String) -> Self {
        Self {
            identity: GameIdentity {
                provider_id: "relay-labs".to_owned(),
                game_key: "relay-forge".to_owned(),
                rules_version: 1,
                cartridge_digest,
            },
        }
    }

    fn decode(state: &GameState) -> Result<RelayState> {
        serde_json::from_value(state.state.clone()).map_err(|_| ProviderError::Internal)
    }

    fn encode(state: &RelayState) -> Result<Value> {
        serde_json::to_value(state).map_err(|_| ProviderError::Internal)
    }
}

impl ProviderGame for RelayForge {
    fn identity(&self) -> &GameIdentity {
        &self.identity
    }

    fn launch(&self, payload: &Value) -> Result<GameState> {
        let payload: LaunchPayload =
            serde_json::from_value(payload.clone()).map_err(|_| ProviderError::InvalidInput)?;
        if payload.player_count != 1 {
            return Err(ProviderError::InvalidInput);
        }
        Ok(GameState {
            status: ProviderSessionStatus::Active,
            state: Self::encode(&RelayState {
                ore: 0,
                energy: 0,
                round: 0,
                forged: false,
            })?,
        })
    }

    fn command(&self, current: &GameState, payload: &Value) -> Result<GameTransition> {
        if current.status == ProviderSessionStatus::Completed {
            return Err(ProviderError::Conflict);
        }
        let envelope: CommandEnvelope =
            serde_json::from_value(payload.clone()).map_err(|_| ProviderError::InvalidInput)?;
        let mut state = Self::decode(current)?;
        if state.forged || state.round >= 12 {
            return Err(ProviderError::Conflict);
        }
        match envelope.command.action.as_str() {
            "mine" if state.ore < 4 => state.ore += 1,
            "charge" if state.energy < 3 => state.energy += 1,
            "forge" if state.ore >= 2 && state.energy >= 1 => {
                state.ore -= 2;
                state.energy -= 1;
                state.forged = true;
            }
            _ => return Err(ProviderError::InvalidInput),
        }
        state.round = state
            .round
            .checked_add(1)
            .ok_or(ProviderError::Internal)?;
        Ok(GameTransition {
            status: if state.forged {
                ProviderSessionStatus::Completed
            } else {
                ProviderSessionStatus::Active
            },
            state: Self::encode(&state)?,
        })
    }

    fn view(&self, current: &GameState) -> Result<Value> {
        let state = Self::decode(current)?;
        Ok(json!({
            "energy": state.energy,
            "forge_ready": state.ore >= 2 && state.energy >= 1 && !state.forged,
            "forged": state.forged,
            "ore": state.ore,
            "round": state.round,
            "status": if state.forged {
                "The relay core is forged."
            } else {
                "Gather two ore and one energy, then forge the relay core."
            }
        }))
    }

    fn event(&self, current: &GameState) -> Result<Option<GameEvent>> {
        let state = Self::decode(current)?;
        if !state.forged || current.status != ProviderSessionStatus::Completed {
            return Ok(None);
        }
        Ok(Some(GameEvent {
            kind: ProviderEventKind::ResultAvailable,
            payload: json!({
                "achievements": ["first_relay"],
                "outcome": "relay_forged",
                "public_summary": {"rounds": state.round},
                "view": self.view(current)?
            }),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launched(game: &RelayForge) -> GameState {
        game.launch(&json!({"player_count": 1})).expect("launch")
    }

    #[test]
    fn rules_are_deterministic_distinct_and_terminal() {
        let game = RelayForge::new("a".repeat(64));
        let initial = launched(&game);
        assert_eq!(initial, launched(&game));
        let first = game
            .command(&initial, &json!({"command": {"action": "mine"}}))
            .expect("mine");
        let first_state = GameState {
            status: first.status,
            state: first.state,
        };
        let second = game
            .command(&first_state, &json!({"command": {"action": "mine"}}))
            .expect("mine twice");
        let second_state = GameState {
            status: second.status,
            state: second.state,
        };
        let charged = game
            .command(&second_state, &json!({"command": {"action": "charge"}}))
            .expect("charge");
        let charged_state = GameState {
            status: charged.status,
            state: charged.state,
        };
        let forged = game
            .command(&charged_state, &json!({"command": {"action": "forge"}}))
            .expect("forge");
        let terminal = GameState {
            status: forged.status,
            state: forged.state,
        };
        assert_eq!(terminal.status, ProviderSessionStatus::Completed);
        assert_eq!(game.view(&terminal).expect("view")["forged"], true);
        assert!(game.event(&terminal).expect("event").is_some());
        assert!(game
            .command(&terminal, &json!({"command": {"action": "mine"}}))
            .is_err());
    }

    #[test]
    fn invalid_launch_and_commands_preserve_input_state() {
        let game = RelayForge::new("b".repeat(64));
        assert!(game.launch(&json!({"player_count": 2})).is_err());
        let initial = launched(&game);
        assert!(game
            .command(&initial, &json!({"command": {"action": "forge"}}))
            .is_err());
        assert!(game
            .command(&initial, &json!({"command": {"action": "enter"}}))
            .is_err());
        assert_eq!(initial, launched(&game));
    }
}
