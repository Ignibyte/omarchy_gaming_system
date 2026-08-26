//! Signal Siege: immutable deterministic solo and two-human rules versions.

use omarchy_game_runtime::{
    GameCommandRejection, GameDefinition, GameInitializationError, GameManifest, GameSessionStatus,
    GameTransition,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GAME_KEY: &str = "signal_siege";
pub const GAME_VERSION: u32 = 1;
pub const DISPLAY_NAME: &str = "Signal Siege";
pub const VERSUS_GAME_VERSION: u32 = 2;
pub const VERSUS_DISPLAY_NAME: &str = "Signal Siege Versus";

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

/// The immutable production definition for Signal Siege v2 two-human play.
#[derive(Debug, Clone, Copy, Default)]
pub struct SignalSiegeVersus;

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

const MAX_VERSUS_TURNS: u8 = MAX_ROUNDS * 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct VersusState {
    schema_version: u8,
    rules_version: u32,
    turn: u8,
    max_turns: u8,
    phase: VersusPhase,
    active_seat: Option<u8>,
    players: [VersusCombatant; 2],
    last_turn: Option<TurnRecord>,
    outcome: Option<VersusOutcome>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum VersusPhase {
    AwaitingAction,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct VersusCombatant {
    seat: u8,
    core: u8,
    energy: u8,
    guard: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TurnRecord {
    turn: u8,
    actor_seat: u8,
    action: Action,
    damage_to_opponent: u8,
    blocked_damage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct VersusOutcome {
    winner: VersusWinner,
    reason: OutcomeReason,
    seat_0_core: u8,
    seat_1_core: u8,
    seat_0_energy: u8,
    seat_1_energy: u8,
    turns_played: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum VersusWinner {
    #[serde(rename = "seat_0")]
    Seat0,
    #[serde(rename = "seat_1")]
    Seat1,
    #[serde(rename = "draw")]
    Draw,
}

impl GameDefinition for SignalSiegeVersus {
    fn manifest(&self) -> GameManifest {
        GameManifest {
            key: GAME_KEY.to_owned(),
            version: VERSUS_GAME_VERSION,
            display_name: VERSUS_DISPLAY_NAME.to_owned(),
            min_human_players: 2,
            max_human_players: 2,
        }
    }

    fn initial_state(&self, human_players: u8) -> Result<Value, GameInitializationError> {
        if human_players != 2 {
            return Err(GameInitializationError);
        }
        versus_to_value(&VersusState {
            schema_version: 1,
            rules_version: VERSUS_GAME_VERSION,
            turn: 0,
            max_turns: MAX_VERSUS_TURNS,
            phase: VersusPhase::AwaitingAction,
            active_seat: Some(0),
            players: [
                VersusCombatant {
                    seat: 0,
                    core: STARTING_CORE,
                    energy: STARTING_ENERGY,
                    guard: 0,
                },
                VersusCombatant {
                    seat: 1,
                    core: STARTING_CORE,
                    energy: STARTING_ENERGY,
                    guard: 0,
                },
            ],
            last_turn: None,
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
        let mut state = parse_versus_state(state)?;
        if state.phase != VersusPhase::AwaitingAction
            || state.active_seat != Some(actor_seat)
            || actor_seat > 1
        {
            return Err(GameCommandRejection);
        }
        let command: Command =
            serde_json::from_value(command.clone()).map_err(|_| GameCommandRejection)?;
        let CommandKind::Play = command.kind;
        let actor_index = usize::from(actor_seat);
        let opponent_index = usize::from(1 - actor_seat);
        ensure_affordable(state.players[actor_index].energy, command.action)?;

        state.players[actor_index].guard = 0;
        let (damage_to_opponent, blocked_damage) = match command.action {
            Action::Strike => {
                state.players[actor_index].energy -= ACTION_COST;
                let blocked = state.players[opponent_index].guard.min(GUARD_BLOCK);
                state.players[opponent_index].guard = 0;
                let applied = STRIKE_DAMAGE.saturating_sub(blocked);
                state.players[opponent_index].core =
                    state.players[opponent_index].core.saturating_sub(applied);
                (applied, blocked)
            }
            Action::Guard => {
                state.players[actor_index].energy -= ACTION_COST;
                state.players[actor_index].guard = GUARD_BLOCK;
                (0, 0)
            }
            Action::Charge => {
                state.players[actor_index].energy = state.players[actor_index]
                    .energy
                    .saturating_add(CHARGE_GAIN)
                    .min(MAX_ENERGY);
                (0, 0)
            }
        };

        state.turn = state.turn.checked_add(1).ok_or(GameCommandRejection)?;
        state.last_turn = Some(TurnRecord {
            turn: state.turn,
            actor_seat,
            action: command.action,
            damage_to_opponent,
            blocked_damage,
        });
        let outcome_reason = if state.players.iter().any(|player| player.core == 0) {
            Some(OutcomeReason::CoreDestroyed)
        } else if state.turn >= state.max_turns {
            Some(OutcomeReason::RoundLimit)
        } else {
            None
        };
        let status = if let Some(reason) = outcome_reason {
            state.phase = VersusPhase::Completed;
            state.active_seat = None;
            state.outcome = Some(build_versus_outcome(&state, reason));
            GameSessionStatus::Completed
        } else {
            state.active_seat = Some(1 - actor_seat);
            GameSessionStatus::Active
        };
        Ok(GameTransition {
            state: versus_to_value(&state)?,
            status,
        })
    }
}

fn parse_versus_state(value: &Value) -> Result<VersusState, GameCommandRejection> {
    let state: VersusState =
        serde_json::from_value(value.clone()).map_err(|_| GameCommandRejection)?;
    let scalar_bounds_are_valid = state.schema_version == 1
        && state.rules_version == VERSUS_GAME_VERSION
        && state.max_turns == MAX_VERSUS_TURNS
        && state.turn <= state.max_turns
        && state.players[0].seat == 0
        && state.players[1].seat == 1
        && state.players.iter().all(|player| {
            player.core <= STARTING_CORE
                && player.energy <= MAX_ENERGY
                && matches!(player.guard, 0 | GUARD_BLOCK)
        });
    let lifecycle_is_valid = match state.phase {
        VersusPhase::AwaitingAction => {
            state.turn < state.max_turns
                && state.players.iter().all(|player| player.core > 0)
                && state.active_seat == Some(state.turn % 2)
                && state.outcome.is_none()
                && last_turn_is_valid(&state)
                && (state.turn != 0
                    || (state.players
                        == [
                            VersusCombatant {
                                seat: 0,
                                core: STARTING_CORE,
                                energy: STARTING_ENERGY,
                                guard: 0,
                            },
                            VersusCombatant {
                                seat: 1,
                                core: STARTING_CORE,
                                energy: STARTING_ENERGY,
                                guard: 0,
                            },
                        ]
                        && state.last_turn.is_none()))
        }
        VersusPhase::Completed => {
            let reason = if state.players.iter().any(|player| player.core == 0) {
                Some(OutcomeReason::CoreDestroyed)
            } else if state.turn == state.max_turns {
                Some(OutcomeReason::RoundLimit)
            } else {
                None
            };
            state.turn > 0
                && state.active_seat.is_none()
                && last_turn_is_valid(&state)
                && reason.is_some_and(|reason| {
                    state
                        .outcome
                        .as_ref()
                        .is_some_and(|outcome| *outcome == build_versus_outcome(&state, reason))
                })
        }
    };
    if scalar_bounds_are_valid && lifecycle_is_valid {
        Ok(state)
    } else {
        Err(GameCommandRejection)
    }
}

fn last_turn_is_valid(state: &VersusState) -> bool {
    match (state.turn, state.last_turn.as_ref()) {
        (0, None) => true,
        (0, Some(_)) | (_, None) => false,
        (turn, Some(record)) => {
            if record.actor_seat > 1 {
                return false;
            }
            let action_evidence_is_valid = match record.action {
                Action::Strike => {
                    record.damage_to_opponent.checked_add(record.blocked_damage)
                        == Some(STRIKE_DAMAGE)
                        && matches!(record.blocked_damage, 0 | GUARD_BLOCK)
                        && state.players[usize::from(1 - record.actor_seat)].guard == 0
                }
                Action::Guard => {
                    record.damage_to_opponent == 0
                        && record.blocked_damage == 0
                        && state.players[usize::from(record.actor_seat)].guard == GUARD_BLOCK
                }
                Action::Charge => {
                    record.damage_to_opponent == 0
                        && record.blocked_damage == 0
                        && state.players[usize::from(record.actor_seat)].guard == 0
                }
            };
            record.turn == turn && record.actor_seat == (turn - 1) % 2 && action_evidence_is_valid
        }
    }
}

fn versus_to_value(value: &VersusState) -> Result<Value, GameCommandRejection> {
    serde_json::to_value(value).map_err(|_| GameCommandRejection)
}

fn build_versus_outcome(state: &VersusState, reason: OutcomeReason) -> VersusOutcome {
    let winner = match state.players[0].core.cmp(&state.players[1].core) {
        std::cmp::Ordering::Greater => VersusWinner::Seat0,
        std::cmp::Ordering::Less => VersusWinner::Seat1,
        std::cmp::Ordering::Equal => match state.players[0].energy.cmp(&state.players[1].energy) {
            std::cmp::Ordering::Greater => VersusWinner::Seat0,
            std::cmp::Ordering::Less => VersusWinner::Seat1,
            std::cmp::Ordering::Equal => VersusWinner::Draw,
        },
    };
    VersusOutcome {
        winner,
        reason,
        seat_0_core: state.players[0].core,
        seat_1_core: state.players[1].core,
        seat_0_energy: state.players[0].energy,
        seat_1_energy: state.players[1].energy,
        turns_played: state.turn,
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

    fn versus_play(
        state: &Value,
        actor_seat: u8,
        action: &str,
    ) -> Result<GameTransition, GameCommandRejection> {
        SignalSiegeVersus.apply_command(
            state,
            actor_seat,
            &json!({"kind": "play", "action": action}),
        )
    }

    #[test]
    fn versus_manifest_and_initial_state_are_exact_and_deterministic() {
        let definition = SignalSiegeVersus;
        assert_eq!(
            definition.manifest(),
            GameManifest {
                key: "signal_siege".to_owned(),
                version: 2,
                display_name: "Signal Siege Versus".to_owned(),
                min_human_players: 2,
                max_human_players: 2,
            }
        );
        let first = definition
            .initial_state(2)
            .expect("two-human game should initialize");
        let second = definition
            .initial_state(2)
            .expect("initialization should repeat");
        assert_eq!(first, second);
        assert_eq!(first["phase"], "awaiting_action");
        assert_eq!(first["active_seat"], 0);
        assert_eq!(first["players"].as_array().map(Vec::len), Some(2));
        assert!(definition.initial_state(1).is_err());
        assert!(definition.initial_state(3).is_err());
    }

    #[test]
    fn versus_turns_apply_guard_strike_charge_and_determinism() {
        let initial = SignalSiegeVersus
            .initial_state(2)
            .expect("initial state should build");
        let guarded = versus_play(&initial, 0, "guard").expect("seat zero should guard");
        let repeated = versus_play(&initial, 0, "guard").expect("same input should repeat");
        assert_eq!(guarded, repeated);
        assert_eq!(guarded.state["turn"], 1);
        assert_eq!(guarded.state["active_seat"], 1);
        assert_eq!(guarded.state["players"][0]["energy"], 1);
        assert_eq!(guarded.state["players"][0]["guard"], GUARD_BLOCK);

        let struck =
            versus_play(&guarded.state, 1, "strike").expect("seat one should strike the guard");
        assert_eq!(struck.state["players"][0]["core"], STARTING_CORE);
        assert_eq!(struck.state["players"][0]["guard"], 0);
        assert_eq!(struck.state["last_turn"]["blocked_damage"], GUARD_BLOCK);
        assert_eq!(struck.state["last_turn"]["damage_to_opponent"], 0);

        let charged = versus_play(&struck.state, 0, "charge").expect("seat zero should charge");
        assert_eq!(charged.state["players"][0]["energy"], 3);
        assert_eq!(charged.state["active_seat"], 1);
    }

    #[test]
    fn versus_rejects_wrong_turn_unaffordable_malformed_and_inconsistent_state() {
        let initial = SignalSiegeVersus
            .initial_state(2)
            .expect("initial state should build");
        assert!(versus_play(&initial, 1, "charge").is_err());
        for command in [
            json!({"kind": "wait", "action": "strike"}),
            json!({"kind": "play", "action": "unknown"}),
            json!({"kind": "play", "action": "strike", "extra": true}),
            json!([]),
        ] {
            assert!(
                SignalSiegeVersus
                    .apply_command(&initial, 0, &command)
                    .is_err()
            );
        }

        let seat_zero_struck = versus_play(&initial, 0, "strike").expect("strike should apply");
        let seat_one_charged =
            versus_play(&seat_zero_struck.state, 1, "charge").expect("charge should apply");
        let seat_zero_empty =
            versus_play(&seat_one_charged.state, 0, "strike").expect("strike should apply");
        let seat_zero_turn =
            versus_play(&seat_zero_empty.state, 1, "charge").expect("charge should apply");
        assert!(versus_play(&seat_zero_turn.state, 0, "strike").is_err());
        assert!(versus_play(&seat_zero_turn.state, 0, "guard").is_err());
        assert!(versus_play(&seat_zero_turn.state, 0, "charge").is_ok());

        for malformed in [
            json!({"schema_version": 1, "rules_version": 2}),
            {
                let mut value = initial.clone();
                value["active_seat"] = json!(1);
                value
            },
            {
                let mut value = initial.clone();
                value["players"][1]["seat"] = json!(0);
                value
            },
            {
                let mut value = initial.clone();
                value["players"][0]["guard"] = json!(1);
                value
            },
            {
                let mut value = initial.clone();
                value["phase"] = json!("completed");
                value["active_seat"] = Value::Null;
                value
            },
        ] {
            assert!(versus_play(&malformed, 0, "charge").is_err());
        }
    }

    #[test]
    fn versus_core_and_turn_limit_completion_are_explicit() {
        let mut core_finish = SignalSiegeVersus
            .initial_state(2)
            .expect("initial state should build");
        for (actor, action) in [
            (0, "strike"),
            (1, "charge"),
            (0, "strike"),
            (1, "charge"),
            (0, "charge"),
            (1, "charge"),
            (0, "strike"),
            (1, "charge"),
        ] {
            core_finish = versus_play(&core_finish, actor, action)
                .expect("setup action should apply")
                .state;
        }
        assert_eq!(core_finish["players"][1]["core"], 2);
        let completed =
            versus_play(&core_finish, 0, "strike").expect("finishing strike should apply");
        assert_eq!(completed.status, GameSessionStatus::Completed);
        assert_eq!(completed.state["phase"], "completed");
        assert!(completed.state["active_seat"].is_null());
        assert_eq!(completed.state["outcome"]["winner"], "seat_0");
        assert_eq!(completed.state["outcome"]["reason"], "core_destroyed");
        assert!(versus_play(&completed.state, 1, "charge").is_err());

        let mut state = SignalSiegeVersus
            .initial_state(2)
            .expect("initial state should build");
        let mut status = GameSessionStatus::Active;
        for turn in 0..MAX_VERSUS_TURNS {
            let actor = turn % 2;
            let transition = versus_play(&state, actor, "charge")
                .expect("charge-only strategy should remain legal");
            state = transition.state;
            status = transition.status;
        }
        assert_eq!(status, GameSessionStatus::Completed);
        assert_eq!(state["turn"], MAX_VERSUS_TURNS);
        assert_eq!(state["outcome"]["winner"], "draw");
        assert_eq!(state["outcome"]["reason"], "round_limit");
    }

    #[test]
    fn versus_legal_aggressive_strategy_terminates_within_twenty_four_turns() {
        let mut state = SignalSiegeVersus
            .initial_state(2)
            .expect("initial state should build");
        let mut status = GameSessionStatus::Active;
        for _ in 0..MAX_VERSUS_TURNS {
            let actor = state["active_seat"]
                .as_u64()
                .and_then(|seat| u8::try_from(seat).ok())
                .expect("active state should name a seat");
            let energy = state["players"][usize::from(actor)]["energy"]
                .as_u64()
                .unwrap_or(0);
            let action = if energy == 0 { "charge" } else { "strike" };
            let transition = versus_play(&state, actor, action)
                .expect("aggressive strategy should remain legal");
            state = transition.state;
            status = transition.status;
            if status == GameSessionStatus::Completed {
                break;
            }
        }
        assert_eq!(status, GameSessionStatus::Completed);
        assert!(
            state["turn"]
                .as_u64()
                .is_some_and(|turn| turn <= u64::from(MAX_VERSUS_TURNS))
        );
        assert!(state["outcome"].is_object());
    }
}
