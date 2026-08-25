//! Signal Siege v1: a deterministic asynchronous human-versus-bot duel.

use omarchy_game_runtime::{
    GameCommandRejection, GameDefinition, GameInitializationError, GameManifest, GameSessionStatus,
    GameTransition,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GAME_KEY: &str = "signal_siege";
pub const GAME_VERSION: u32 = 1;
pub const DISPLAY_NAME: &str = "Signal Siege";

const STARTING_CORE: u8 = 8;
const STARTING_ENERGY: u8 = 2;
const MAX_ENERGY: u8 = 4;
const MAX_ROUNDS: u8 = 12;
const ACTION_COST: u8 = 1;
const STRIKE_DAMAGE: u8 = 2;
const GUARD_BLOCK: u8 = 2;
const CHARGE_GAIN: u8 = 2;

/// The immutable production definition for Signal Siege v1.
#[derive(Debug, Clone, Copy, Default)]
pub struct SignalSiege;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct State {
    schema_version: u8,
    rules_version: u32,
    round: u8,
    max_rounds: u8,
    phase: Phase,
    human: Combatant,
    bot: Combatant,
    last_round: Option<RoundRecord>,
    outcome: Option<Outcome>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Phase {
    AwaitingHuman,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Combatant {
    core: u8,
    energy: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RoundRecord {
    round: u8,
    human_action: Action,
    bot_action: Action,
    damage_to_human: u8,
    damage_to_bot: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Outcome {
    winner: Winner,
    reason: OutcomeReason,
    human_core: u8,
    bot_core: u8,
    human_energy: u8,
    bot_energy: u8,
    rounds_played: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Winner {
    Human,
    Bot,
    Draw,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OutcomeReason {
    CoreDestroyed,
    RoundLimit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Action {
    Strike,
    Guard,
    Charge,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    kind: CommandKind,
    action: Action,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CommandKind {
    Play,
}

impl GameDefinition for SignalSiege {
    fn manifest(&self) -> GameManifest {
        GameManifest {
            key: GAME_KEY.to_owned(),
            version: GAME_VERSION,
            display_name: DISPLAY_NAME.to_owned(),
            min_human_players: 1,
            max_human_players: 1,
        }
    }

    fn initial_state(&self, human_players: u8) -> Result<Value, GameInitializationError> {
        if human_players != 1 {
            return Err(GameInitializationError);
        }
        to_value(&State {
            schema_version: 1,
            rules_version: GAME_VERSION,
            round: 0,
            max_rounds: MAX_ROUNDS,
            phase: Phase::AwaitingHuman,
            human: Combatant {
                core: STARTING_CORE,
                energy: STARTING_ENERGY,
            },
            bot: Combatant {
                core: STARTING_CORE,
                energy: STARTING_ENERGY,
            },
            last_round: None,
            outcome: None,
        })
        .map_err(|_| GameInitializationError)
    }

    fn apply_command(
        &self,
        state: &Value,
        actor_seat: u8,
        command: &Value,
    ) -> Result<GameTransition, GameCommandRejection> {
        if actor_seat != 0 {
            return Err(GameCommandRejection);
        }
        let mut state = parse_state(state)?;
        if state.phase != Phase::AwaitingHuman || state.outcome.is_some() {
            return Err(GameCommandRejection);
        }
        let command: Command =
            serde_json::from_value(command.clone()).map_err(|_| GameCommandRejection)?;
        let CommandKind::Play = command.kind;
        ensure_affordable(state.human.energy, command.action)?;

        let bot_action = choose_bot_action(&state);
        ensure_affordable(state.bot.energy, bot_action)?;
        let human_action = command.action;
        apply_energy(&mut state.human, human_action);
        apply_energy(&mut state.bot, bot_action);

        let damage_to_bot = damage(human_action, bot_action);
        let damage_to_human = damage(bot_action, human_action);
        state.bot.core = state.bot.core.saturating_sub(damage_to_bot);
        state.human.core = state.human.core.saturating_sub(damage_to_human);
        state.round = state.round.checked_add(1).ok_or(GameCommandRejection)?;
        state.last_round = Some(RoundRecord {
            round: state.round,
            human_action,
            bot_action,
            damage_to_human,
            damage_to_bot,
        });

        let outcome_reason = if state.human.core == 0 || state.bot.core == 0 {
            Some(OutcomeReason::CoreDestroyed)
        } else if state.round >= state.max_rounds {
            Some(OutcomeReason::RoundLimit)
        } else {
            None
        };
        let status = if let Some(reason) = outcome_reason {
            state.phase = Phase::Completed;
            state.outcome = Some(build_outcome(&state, reason));
            GameSessionStatus::Completed
        } else {
            GameSessionStatus::Active
        };

        Ok(GameTransition {
            state: to_value(&state)?,
            status,
        })
    }
}

fn parse_state(value: &Value) -> Result<State, GameCommandRejection> {
    let state: State = serde_json::from_value(value.clone()).map_err(|_| GameCommandRejection)?;
    let scalar_bounds_are_valid = state.schema_version == 1
        && state.rules_version == GAME_VERSION
        && state.max_rounds == MAX_ROUNDS
        && state.round <= state.max_rounds
        && state.human.core <= STARTING_CORE
        && state.bot.core <= STARTING_CORE
        && state.human.energy <= MAX_ENERGY
        && state.bot.energy <= MAX_ENERGY;
    let lifecycle_is_valid = match state.phase {
        Phase::AwaitingHuman => {
            state.round < state.max_rounds
                && state.human.core > 0
                && state.bot.core > 0
                && state.outcome.is_none()
                && last_round_is_valid(&state)
                && (state.round != 0
                    || (state.human
                        == Combatant {
                            core: STARTING_CORE,
                            energy: STARTING_ENERGY,
                        }
                        && state.bot
                            == Combatant {
                                core: STARTING_CORE,
                                energy: STARTING_ENERGY,
                            }))
        }
        Phase::Completed => {
            let reason = if state.human.core == 0 || state.bot.core == 0 {
                Some(OutcomeReason::CoreDestroyed)
            } else if state.round == state.max_rounds {
                Some(OutcomeReason::RoundLimit)
            } else {
                None
            };
            state.round > 0
                && last_round_is_valid(&state)
                && reason.is_some_and(|reason| {
                    state
                        .outcome
                        .as_ref()
                        .is_some_and(|outcome| *outcome == build_outcome(&state, reason))
                })
        }
    };
    let valid = scalar_bounds_are_valid && lifecycle_is_valid;
    if valid {
        Ok(state)
    } else {
        Err(GameCommandRejection)
    }
}

fn last_round_is_valid(state: &State) -> bool {
    match (state.round, state.last_round.as_ref()) {
        (0, None) => true,
        (0, Some(_)) | (_, None) => false,
        (round, Some(record)) => {
            record.round == round
                && record.damage_to_bot == damage(record.human_action, record.bot_action)
                && record.damage_to_human == damage(record.bot_action, record.human_action)
        }
    }
}

fn to_value(value: &State) -> Result<Value, GameCommandRejection> {
    serde_json::to_value(value).map_err(|_| GameCommandRejection)
}

fn ensure_affordable(energy: u8, action: Action) -> Result<(), GameCommandRejection> {
    if matches!(action, Action::Strike | Action::Guard) && energy < ACTION_COST {
        Err(GameCommandRejection)
    } else {
        Ok(())
    }
}

fn choose_bot_action(state: &State) -> Action {
    if state.bot.energy == 0 {
        return Action::Charge;
    }
    if state.bot.core <= 3 && state.human.energy > 0 {
        return Action::Guard;
    }
    let policy_index =
        (u16::from(state.round) + u16::from(state.human.energy) + u16::from(state.bot.core)) % 3;
    match policy_index {
        0 => Action::Strike,
        1 => Action::Guard,
        _ => Action::Charge,
    }
}

fn apply_energy(combatant: &mut Combatant, action: Action) {
    match action {
        Action::Strike | Action::Guard => combatant.energy -= ACTION_COST,
        Action::Charge => {
            combatant.energy = combatant.energy.saturating_add(CHARGE_GAIN).min(MAX_ENERGY);
        }
    }
}

fn damage(attacker: Action, defender: Action) -> u8 {
    if attacker != Action::Strike {
        return 0;
    }
    let blocked = if defender == Action::Guard {
        GUARD_BLOCK
    } else {
        0
    };
    STRIKE_DAMAGE.saturating_sub(blocked)
}

fn build_outcome(state: &State, reason: OutcomeReason) -> Outcome {
    let winner = match state.human.core.cmp(&state.bot.core) {
        std::cmp::Ordering::Greater => Winner::Human,
        std::cmp::Ordering::Less => Winner::Bot,
        std::cmp::Ordering::Equal => match state.human.energy.cmp(&state.bot.energy) {
            std::cmp::Ordering::Greater => Winner::Human,
            std::cmp::Ordering::Less => Winner::Bot,
            std::cmp::Ordering::Equal => Winner::Draw,
        },
    };
    Outcome {
        winner,
        reason,
        human_core: state.human.core,
        bot_core: state.bot.core,
        human_energy: state.human.energy,
        bot_energy: state.bot.energy,
        rounds_played: state.round,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn play(state: &Value, action: &str) -> Result<GameTransition, GameCommandRejection> {
        SignalSiege.apply_command(state, 0, &json!({"kind": "play", "action": action}))
    }

    #[test]
    fn manifest_and_initial_state_are_exact_and_deterministic() {
        let definition = SignalSiege;
        assert_eq!(
            definition.manifest(),
            GameManifest {
                key: "signal_siege".to_owned(),
                version: 1,
                display_name: "Signal Siege".to_owned(),
                min_human_players: 1,
                max_human_players: 1,
            }
        );
        let first = definition
            .initial_state(1)
            .expect("one-human game should initialize");
        let second = definition
            .initial_state(1)
            .expect("initialization should repeat");
        assert_eq!(first, second);
        assert_eq!(first["phase"], "awaiting_human");
        assert_eq!(first["round"], 0);
        assert_eq!(first["outcome"], Value::Null);
        assert!(definition.initial_state(2).is_err());
    }

    #[test]
    fn each_action_resolves_simultaneously_and_the_bot_is_state_deterministic() {
        let initial = SignalSiege
            .initial_state(1)
            .expect("initial state should build");
        let first = play(&initial, "strike").expect("strike should be affordable");
        let repeated = play(&initial, "strike").expect("same input should repeat");
        assert_eq!(first, repeated);
        assert_eq!(first.status, GameSessionStatus::Active);
        assert_eq!(first.state["round"], 1);
        assert_eq!(first.state["human"]["energy"], 1);
        assert_eq!(first.state["last_round"]["human_action"], "strike");
        assert_eq!(first.state["last_round"]["bot_action"], "guard");
        assert_eq!(first.state["last_round"]["damage_to_bot"], 0);

        let charged = play(&initial, "charge").expect("charge should apply");
        assert_eq!(charged.state["human"]["energy"], MAX_ENERGY);
        assert_eq!(charged.state["last_round"]["bot_action"], "guard");
    }

    #[test]
    fn malformed_unaffordable_wrong_seat_and_completed_commands_reject() {
        let initial = SignalSiege
            .initial_state(1)
            .expect("initial state should build");
        for command in [
            json!({"kind": "wait", "action": "strike"}),
            json!({"kind": "play", "action": "unknown"}),
            json!({"kind": "play", "action": "strike", "extra": true}),
            json!([]),
        ] {
            assert!(SignalSiege.apply_command(&initial, 0, &command).is_err());
        }
        assert!(
            SignalSiege
                .apply_command(&initial, 1, &json!({"kind": "play", "action": "strike"}))
                .is_err()
        );

        let empty = json!({
            "schema_version": 1,
            "rules_version": 1,
            "round": 2,
            "max_rounds": 12,
            "phase": "awaiting_human",
            "human": {"core": 8, "energy": 0},
            "bot": {"core": 8, "energy": 2},
            "last_round": {
                "round": 2,
                "human_action": "strike",
                "bot_action": "strike",
                "damage_to_human": 2,
                "damage_to_bot": 2
            },
            "outcome": null
        });
        assert!(play(&empty, "strike").is_err());
        assert!(play(&empty, "guard").is_err());
        assert!(play(&empty, "charge").is_ok());

        let malformed = json!({"schema_version": 1, "rules_version": 1});
        assert!(play(&malformed, "charge").is_err());

        for malformed_state in [
            json!({
                "schema_version": 1,
                "rules_version": 1,
                "round": 12,
                "max_rounds": 12,
                "phase": "awaiting_human",
                "human": {"core": 8, "energy": 2},
                "bot": {"core": 8, "energy": 2},
                "last_round": {
                    "round": 12,
                    "human_action": "guard",
                    "bot_action": "guard",
                    "damage_to_human": 0,
                    "damage_to_bot": 0
                },
                "outcome": null
            }),
            json!({
                "schema_version": 1,
                "rules_version": 1,
                "round": 1,
                "max_rounds": 12,
                "phase": "awaiting_human",
                "human": {"core": 0, "energy": 2},
                "bot": {"core": 8, "energy": 2},
                "last_round": {
                    "round": 1,
                    "human_action": "guard",
                    "bot_action": "guard",
                    "damage_to_human": 0,
                    "damage_to_bot": 0
                },
                "outcome": null
            }),
            json!({
                "schema_version": 1,
                "rules_version": 1,
                "round": 1,
                "max_rounds": 12,
                "phase": "awaiting_human",
                "human": {"core": 8, "energy": 2},
                "bot": {"core": 8, "energy": 2},
                "last_round": null,
                "outcome": null
            }),
            json!({
                "schema_version": 1,
                "rules_version": 1,
                "round": 1,
                "max_rounds": 12,
                "phase": "completed",
                "human": {"core": 0, "energy": 2},
                "bot": {"core": 8, "energy": 2},
                "last_round": {
                    "round": 1,
                    "human_action": "strike",
                    "bot_action": "strike",
                    "damage_to_human": 2,
                    "damage_to_bot": 2
                },
                "outcome": {
                    "winner": "human",
                    "reason": "core_destroyed",
                    "human_core": 0,
                    "bot_core": 8,
                    "human_energy": 2,
                    "bot_energy": 2,
                    "rounds_played": 1
                }
            }),
        ] {
            assert!(play(&malformed_state, "charge").is_err());
        }
    }

    #[test]
    fn core_destruction_and_round_limit_produce_bounded_terminal_outcomes() {
        let core_finish = json!({
            "schema_version": 1,
            "rules_version": 1,
            "round": 4,
            "max_rounds": 12,
            "phase": "awaiting_human",
            "human": {"core": 8, "energy": 2},
            "bot": {"core": 2, "energy": 0},
            "last_round": {
                "round": 4,
                "human_action": "strike",
                "bot_action": "charge",
                "damage_to_human": 0,
                "damage_to_bot": 2
            },
            "outcome": null
        });
        let completed = play(&core_finish, "strike").expect("finishing strike should apply");
        assert_eq!(completed.status, GameSessionStatus::Completed);
        assert_eq!(completed.state["phase"], "completed");
        assert_eq!(completed.state["outcome"]["winner"], "human");
        assert_eq!(completed.state["outcome"]["reason"], "core_destroyed");
        assert!(play(&completed.state, "charge").is_err());

        let round_finish = json!({
            "schema_version": 1,
            "rules_version": 1,
            "round": 11,
            "max_rounds": 12,
            "phase": "awaiting_human",
            "human": {"core": 6, "energy": 4},
            "bot": {"core": 6, "energy": 4},
            "last_round": {
                "round": 11,
                "human_action": "guard",
                "bot_action": "guard",
                "damage_to_human": 0,
                "damage_to_bot": 0
            },
            "outcome": null
        });
        let completed = play(&round_finish, "guard").expect("last round should apply");
        assert_eq!(completed.status, GameSessionStatus::Completed);
        assert_eq!(completed.state["round"], 12);
        assert_eq!(completed.state["outcome"]["reason"], "round_limit");
        assert!(matches!(
            completed.state["outcome"]["winner"].as_str(),
            Some("human" | "bot" | "draw")
        ));
    }

    #[test]
    fn every_legal_strategy_reaches_terminal_state_within_twelve_rounds() {
        for preferred in ["strike", "guard", "charge"] {
            let mut state = SignalSiege
                .initial_state(1)
                .expect("initial state should build");
            let mut status = GameSessionStatus::Active;
            for _ in 0..MAX_ROUNDS {
                let energy = state["human"]["energy"].as_u64().unwrap_or(0);
                let action = if preferred != "charge" && energy == 0 {
                    "charge"
                } else {
                    preferred
                };
                let transition = play(&state, action).expect("legal strategy should apply");
                state = transition.state;
                status = transition.status;
                if status == GameSessionStatus::Completed {
                    break;
                }
            }
            assert_eq!(status, GameSessionStatus::Completed);
            assert!(state["round"].as_u64().is_some_and(|round| round <= 12));
            assert!(state["outcome"].is_object());
        }
    }
}
