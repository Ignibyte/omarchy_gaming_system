//! Deterministic, database-free contracts for compiled Omarchy game definitions.

use std::{collections::BTreeMap, sync::Arc};

use serde_json::Value;

const MIN_GAME_KEY_BYTES: usize = 3;
const MAX_GAME_KEY_BYTES: usize = 32;
const MAX_DISPLAY_NAME_CHARACTERS: usize = 64;
const MAX_HUMAN_PLAYERS: u8 = 8;
const MAX_GAME_STATE_BYTES: usize = 64 * 1024;
const MAX_GAME_COMMAND_BYTES: usize = 16 * 1024;

/// Public metadata that identifies one immutable compiled rules version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameManifest {
    pub key: String,
    pub version: u32,
    pub display_name: String,
    pub min_human_players: u8,
    pub max_human_players: u8,
}

/// A compiled game definition. Implementations must be deterministic and must
/// not perform database, network, clock, or ambient-randomness work.
pub trait GameDefinition: Send + Sync {
    fn manifest(&self) -> GameManifest;

    fn initial_state(&self, human_players: u8) -> Result<Value, GameInitializationError>;

    fn apply_command(
        &self,
        state: &Value,
        actor_seat: u8,
        command: &Value,
    ) -> Result<Value, GameCommandRejection>;
}

/// Deliberately non-descriptive failure returned by trusted compiled game code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameInitializationError;

/// Deliberately non-descriptive rejection returned by trusted compiled rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameCommandRejection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameRegistryError {
    InvalidManifest,
    DuplicateDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitializeGameError {
    GameUnavailable,
    InvalidPlayerCount,
    InitializationFailed,
    InvalidInitialState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyGameCommandError {
    GameUnavailable,
    InvalidState,
    InvalidActorSeat,
    InvalidCommand,
    CommandRejected,
    InvalidTransition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitializedGame {
    pub manifest: GameManifest,
    pub state: Value,
}

#[derive(Clone, Default)]
pub struct GameRegistry {
    entries: Arc<BTreeMap<GameId, RegisteredGame>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GameId {
    key: String,
    version: u32,
}

#[derive(Clone)]
struct RegisteredGame {
    manifest: GameManifest,
    definition: Arc<dyn GameDefinition>,
}

impl GameRegistry {
    /// Construct a validated immutable registry from compiled definitions.
    pub fn new(
        definitions: impl IntoIterator<Item = Arc<dyn GameDefinition>>,
    ) -> Result<Self, GameRegistryError> {
        let mut entries = BTreeMap::new();
        for definition in definitions {
            let manifest = definition.manifest();
            validate_manifest(&manifest)?;
            let id = GameId {
                key: manifest.key.clone(),
                version: manifest.version,
            };
            if entries
                .insert(
                    id,
                    RegisteredGame {
                        manifest,
                        definition,
                    },
                )
                .is_some()
            {
                return Err(GameRegistryError::DuplicateDefinition);
            }
        }
        Ok(Self {
            entries: Arc::new(entries),
        })
    }

    /// Construct the valid empty production registry used until a game ships.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Return public manifests in canonical `(key, version)` order.
    pub fn catalog(&self) -> Vec<GameManifest> {
        self.entries
            .values()
            .map(|registered| registered.manifest.clone())
            .collect()
    }

    /// Initialize state from exactly the requested rules version.
    pub fn initialize(
        &self,
        key: &str,
        version: u32,
        human_players: u8,
    ) -> Result<InitializedGame, InitializeGameError> {
        let registered = self
            .entries
            .get(&GameId {
                key: key.to_owned(),
                version,
            })
            .ok_or(InitializeGameError::GameUnavailable)?;
        if human_players < registered.manifest.min_human_players
            || human_players > registered.manifest.max_human_players
        {
            return Err(InitializeGameError::InvalidPlayerCount);
        }
        let state = registered
            .definition
            .initial_state(human_players)
            .map_err(|_| InitializeGameError::InitializationFailed)?;
        if !is_bounded_object(&state, MAX_GAME_STATE_BYTES) {
            return Err(InitializeGameError::InvalidInitialState);
        }
        Ok(InitializedGame {
            manifest: registered.manifest.clone(),
            state,
        })
    }

    /// Apply a command using exactly the requested immutable rules version.
    pub fn apply_command(
        &self,
        key: &str,
        version: u32,
        state: &Value,
        actor_seat: u8,
        command: &Value,
    ) -> Result<Value, ApplyGameCommandError> {
        let registered = self
            .entries
            .get(&GameId {
                key: key.to_owned(),
                version,
            })
            .ok_or(ApplyGameCommandError::GameUnavailable)?;
        if !is_bounded_object(state, MAX_GAME_STATE_BYTES) {
            return Err(ApplyGameCommandError::InvalidState);
        }
        if actor_seat >= registered.manifest.max_human_players {
            return Err(ApplyGameCommandError::InvalidActorSeat);
        }
        if !is_bounded_object(command, MAX_GAME_COMMAND_BYTES) {
            return Err(ApplyGameCommandError::InvalidCommand);
        }
        let next_state = registered
            .definition
            .apply_command(state, actor_seat, command)
            .map_err(|_| ApplyGameCommandError::CommandRejected)?;
        if !is_bounded_object(&next_state, MAX_GAME_STATE_BYTES) {
            return Err(ApplyGameCommandError::InvalidTransition);
        }
        Ok(next_state)
    }
}

fn is_bounded_object(value: &Value, max_bytes: usize) -> bool {
    value.is_object()
        && serde_json::to_vec(value).is_ok_and(|serialized| serialized.len() <= max_bytes)
}

fn validate_manifest(manifest: &GameManifest) -> Result<(), GameRegistryError> {
    let key = manifest.key.as_bytes();
    let key_is_canonical = (MIN_GAME_KEY_BYTES..=MAX_GAME_KEY_BYTES).contains(&key.len())
        && key.first().is_some_and(u8::is_ascii_alphanumeric)
        && key.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        });
    let display_name_length = manifest.display_name.chars().count();
    let display_name_is_valid = (1..=MAX_DISPLAY_NAME_CHARACTERS).contains(&display_name_length)
        && manifest
            .display_name
            .chars()
            .all(|character| !character.is_control());
    let player_bounds_are_valid = manifest.version > 0
        && manifest.min_human_players > 0
        && manifest.min_human_players <= manifest.max_human_players
        && manifest.max_human_players <= MAX_HUMAN_PLAYERS;

    if key_is_canonical && display_name_is_valid && player_bounds_are_valid {
        Ok(())
    } else {
        Err(GameRegistryError::InvalidManifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct FixtureGame {
        manifest: GameManifest,
        state: Value,
        fail: bool,
    }

    impl GameDefinition for FixtureGame {
        fn manifest(&self) -> GameManifest {
            self.manifest.clone()
        }

        fn initial_state(&self, human_players: u8) -> Result<Value, GameInitializationError> {
            if self.fail {
                Err(GameInitializationError)
            } else {
                let mut state = self.state.clone();
                if let Some(object) = state.as_object_mut() {
                    object.insert("human_players".to_owned(), json!(human_players));
                }
                Ok(state)
            }
        }

        fn apply_command(
            &self,
            state: &Value,
            actor_seat: u8,
            command: &Value,
        ) -> Result<Value, GameCommandRejection> {
            if command.get("kind") == Some(&json!("reject")) {
                return Err(GameCommandRejection);
            }
            if command.get("kind") == Some(&json!("invalid_output")) {
                return Ok(json!([]));
            }
            if command.get("kind") == Some(&json!("oversized_output")) {
                return Ok(json!({"payload": "x".repeat(MAX_GAME_STATE_BYTES)}));
            }
            let mut state = state.clone();
            let state = state.as_object_mut().ok_or(GameCommandRejection)?;
            let turn = state
                .get("turn")
                .and_then(Value::as_u64)
                .ok_or(GameCommandRejection)?;
            state.insert("turn".to_owned(), json!(turn + 1));
            state.insert("last_actor_seat".to_owned(), json!(actor_seat));
            Ok(Value::Object(state.clone()))
        }
    }

    fn fixture(key: &str, version: u32) -> Arc<dyn GameDefinition> {
        Arc::new(FixtureGame {
            manifest: GameManifest {
                key: key.to_owned(),
                version,
                display_name: format!("Fixture {version}"),
                min_human_players: 1,
                max_human_players: 2,
            },
            state: json!({"turn": 0}),
            fail: false,
        })
    }

    #[test]
    fn registry_validates_manifests_and_rejects_duplicate_versions() {
        let invalid = Arc::new(FixtureGame {
            manifest: GameManifest {
                key: "Not Canonical".to_owned(),
                version: 0,
                display_name: String::new(),
                min_human_players: 0,
                max_human_players: 9,
            },
            state: json!({}),
            fail: false,
        });
        assert!(matches!(
            GameRegistry::new([invalid as Arc<dyn GameDefinition>]),
            Err(GameRegistryError::InvalidManifest)
        ));
        assert!(matches!(
            GameRegistry::new([fixture("fixture", 1), fixture("fixture", 1)]),
            Err(GameRegistryError::DuplicateDefinition)
        ));
    }

    #[test]
    fn registry_orders_catalog_and_resolves_exact_versions() {
        let registry =
            GameRegistry::new([fixture("zeta", 2), fixture("alpha", 2), fixture("alpha", 1)])
                .expect("fixture manifests should form a registry");
        let identities = registry
            .catalog()
            .into_iter()
            .map(|manifest| (manifest.key, manifest.version))
            .collect::<Vec<_>>();
        assert_eq!(
            identities,
            vec![
                ("alpha".to_owned(), 1),
                ("alpha".to_owned(), 2),
                ("zeta".to_owned(), 2)
            ]
        );
        assert_eq!(
            registry
                .initialize("alpha", 1, 2)
                .expect("exact version should initialize")
                .state,
            json!({"turn": 0, "human_players": 2})
        );
        assert_eq!(
            registry.initialize("alpha", 3, 2),
            Err(InitializeGameError::GameUnavailable)
        );
    }

    #[test]
    fn initialization_is_bounded_object_shaped_and_deterministic() {
        let registry =
            GameRegistry::new([fixture("fixture", 1)]).expect("fixture registry should build");
        let first = registry
            .initialize("fixture", 1, 1)
            .expect("fixture should initialize");
        let second = registry
            .initialize("fixture", 1, 1)
            .expect("fixture should initialize repeatedly");
        assert_eq!(first, second);
        assert_eq!(
            registry.initialize("fixture", 1, 3),
            Err(InitializeGameError::InvalidPlayerCount)
        );

        let array_game = Arc::new(FixtureGame {
            manifest: fixture("array", 1).manifest(),
            state: json!([]),
            fail: false,
        });
        let array_registry = GameRegistry::new([array_game as Arc<dyn GameDefinition>])
            .expect("array fixture manifest should be valid");
        assert_eq!(
            array_registry.initialize("fixture", 1, 1),
            Err(InitializeGameError::GameUnavailable)
        );
        assert_eq!(
            array_registry.initialize("array", 1, 1),
            Err(InitializeGameError::InvalidInitialState)
        );

        let failing_game = Arc::new(FixtureGame {
            manifest: fixture("failure", 1).manifest(),
            state: json!({}),
            fail: true,
        });
        let failing_registry = GameRegistry::new([failing_game as Arc<dyn GameDefinition>])
            .expect("failing fixture manifest should be valid");
        assert_eq!(
            failing_registry.initialize("failure", 1, 1),
            Err(InitializeGameError::InitializationFailed)
        );
    }

    #[test]
    fn commands_resolve_exact_versions_and_are_deterministic() {
        let registry = GameRegistry::new([fixture("fixture", 1), fixture("fixture", 2)])
            .expect("fixture registry should build");
        let state = json!({"turn": 0});
        let command = json!({"kind": "advance"});

        let first = registry
            .apply_command("fixture", 1, &state, 1, &command)
            .expect("registered version should apply a command");
        let second = registry
            .apply_command("fixture", 1, &state, 1, &command)
            .expect("the same inputs should apply repeatedly");
        assert_eq!(first, second);
        assert_eq!(first, json!({"turn": 1, "last_actor_seat": 1}));
        assert_eq!(
            registry.apply_command("fixture", 3, &state, 0, &command),
            Err(ApplyGameCommandError::GameUnavailable)
        );
        assert_eq!(
            registry.apply_command("fixture", 1, &state, 2, &command),
            Err(ApplyGameCommandError::InvalidActorSeat)
        );
    }

    #[test]
    fn commands_and_transitions_are_bounded_objects_with_stable_rejection() {
        let registry =
            GameRegistry::new([fixture("fixture", 1)]).expect("fixture registry should build");
        let state = json!({"turn": 0});

        assert_eq!(
            registry.apply_command("fixture", 1, &json!([]), 0, &json!({})),
            Err(ApplyGameCommandError::InvalidState)
        );
        assert_eq!(
            registry.apply_command(
                "fixture",
                1,
                &json!({"payload": "x".repeat(MAX_GAME_STATE_BYTES)}),
                0,
                &json!({})
            ),
            Err(ApplyGameCommandError::InvalidState)
        );
        assert_eq!(
            registry.apply_command("fixture", 1, &state, 0, &json!([])),
            Err(ApplyGameCommandError::InvalidCommand)
        );
        assert_eq!(
            registry.apply_command(
                "fixture",
                1,
                &state,
                0,
                &json!({"payload": "x".repeat(MAX_GAME_COMMAND_BYTES)})
            ),
            Err(ApplyGameCommandError::InvalidCommand)
        );
        assert_eq!(
            registry.apply_command("fixture", 1, &state, 0, &json!({"kind": "reject"})),
            Err(ApplyGameCommandError::CommandRejected)
        );
        assert_eq!(
            registry.apply_command("fixture", 1, &state, 0, &json!({"kind": "invalid_output"})),
            Err(ApplyGameCommandError::InvalidTransition)
        );
        assert_eq!(
            registry.apply_command(
                "fixture",
                1,
                &state,
                0,
                &json!({"kind": "oversized_output"})
            ),
            Err(ApplyGameCommandError::InvalidTransition)
        );
    }
}
