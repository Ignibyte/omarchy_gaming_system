---
title: TICKET-012-game-registry-and-versioned-sessions
status: closed
ticket_number: 012
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/game-registry-and-versioned-sessions.spec.md
---

# TICKET-012-game-registry-and-versioned-sessions

## Summary

Add the compiled-game registry and durable version-pinned session foundation so
later commands, challenges, and the first original game share one deterministic
server-authoritative boundary.

## Why

Identity, social state, inboxes, and reconnect synchronization now exist. The
next slice must establish how compiled game versions are discovered and how a
session pins its rules, participants, initial state, and revision before any
public command or challenge workflow is added.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the process builds a compiled game registry, the system shall accept only canonical bounded manifests, reject duplicate `(game_key, version)` definitions, provide deterministic exact-version lookup, and return a stable catalog order. | Game-runtime unit tests |
| REQ-002 | When any client requests `GET /v1/games`, the system shall return only public metadata for compiled production games in stable order; until the first original game ships, the valid production catalog shall be empty rather than advertising a non-playable game. | Router test and live smoke |
| REQ-003 | When trusted server orchestration creates a game session for an exact registered version and a valid unique participant set, the system shall atomically persist that immutable game key/version, deterministic initial JSON state, revision zero, active status, and ordered persona seats; invalid inputs or initialization failure shall persist nothing. | Registry/domain unit tests and PostgreSQL transaction tests |
| REQ-004 | When an authenticated account lists or reads game sessions through an owned participating persona, the system shall return only that persona's bounded stable session inventory or requested session, public participant profiles, pinned game identity/version, state, revision, and status; absent and foreign objects shall remain indistinguishable. | Multi-account router/PostgreSQL tests |
| REQ-005 | When a stored session is read after the process registry changes, the system shall preserve its pinned game key/version and durable snapshot without silently substituting a newer rules version. | Registry-version and PostgreSQL read tests |
| REQ-006 | When session creation changes a participant's visible game inventory, the system shall append one minimal `game_session_changed { game_session_id }` event for every participant inside the creation transaction and expose no state, account ownership, or participant details in the sync event. | Transaction/event-shape PostgreSQL tests |
| REQ-007 | When the delivery gate validates this slice, the system shall exercise the registry, migration, atomic session creation, owner/participant privacy, version pinning, sync invalidation, public catalog, and unchanged QML health connector against real PostgreSQL. | `bin/gate.sh --diff` |

## Scope

- In: a compiled Rust game-runtime crate, validated versioned manifests,
  production/test registry injection, public catalog, forward-only game session
  and participant persistence, trusted internal session creation, participant-
  scoped REST inventory/detail, initial revision/snapshot, minimal persona sync
  invalidation, tests, smoke, and documentation.
- Out: public session creation, challenges/invites, player commands,
  idempotency keys, revision mutation, turns, timers, results, inbox game
  messages, bots, a production game implementation, QML game UI, third-party
  plugins, and Git delivery.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/game-registry-and-versioned-sessions.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Completion: all seven EARS requirements passed the canonical diff gate.
