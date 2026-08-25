---
title: TICKET-013-idempotent-revision-checked-game-commands
status: closed
ticket_number: 013
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/idempotent-revision-checked-game-commands.spec.md
---

# TICKET-013-idempotent-revision-checked-game-commands

## Summary

Add the first participant command boundary for durable game sessions so every
accepted transition executes through the pinned compiled rules version,
advances one optimistic revision, survives client retries, and wakes every
participant without moving game state onto WebSockets.

## Why

Ticket 012 established immutable game versions, deterministic initial state,
ordered persona participants, and participant-private reads. The next slice
must make state mutable without allowing lost updates, duplicate client
commands, registry version substitution, or non-participant access.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a compiled game definition evaluates a participant command, the runtime shall resolve only the session's exact key/version, receive the current bounded object state, actor seat, and bounded object command, and return a deterministic bounded object state or a stable non-descriptive rejection without database, network, clock, account, session, or ambient-randomness access. | Game-runtime unit tests |
| REQ-002 | When an authenticated account submits a command through an owned participating persona for an active game session, the system shall authorize both persona ownership and durable session membership before disclosing or mutating the session; absent, malformed, and non-participant sessions shall remain indistinguishable. | Multi-account PostgreSQL/router tests |
| REQ-003 | When a first-use command carries the session's current nonnegative expected revision and a registered exact rules version accepts it, the system shall atomically replace the snapshot, increment the revision by exactly one, update the timestamp, and return only the session ID, committed revision, and committed state; a stale or future expected revision shall change nothing. | Revision and rollback PostgreSQL tests |
| REQ-004 | When a client retries the same session-wide idempotency UUID with the same actor, expected revision, and semantic JSON command, the system shall return the original committed response without executing again, advancing the revision, or appending another invalidation; reusing that UUID for a different actor, revision, or command shall return a stable conflict and change nothing. | Idempotency replay/collision PostgreSQL tests |
| REQ-005 | When multiple authorized commands race from the same expected revision, the system shall serialize on the durable game session so at most one distinct first-use command advances that revision and every loser receives a revision conflict or the matching idempotent replay without a lost update. | Concurrent PostgreSQL test |
| REQ-006 | When a command commits a new revision, the system shall append one minimal `game_session_changed { game_session_id }` event for every participant inside the same transaction; rejected commands, revision conflicts, idempotent replays, and rollbacks shall append no new event or expose snapshot, command, account, or participant data through sync/WebSocket payloads. | Transaction/event-shape PostgreSQL tests |
| REQ-007 | When the delivery gate validates this slice, the system shall exercise runtime command bounds, exact-version execution, owner/participant privacy, revision conflicts, idempotent replay/collision, concurrent commands, atomic snapshot plus sync persistence, the empty production catalog, and the unchanged QML health connector against real PostgreSQL. | `bin/gate.sh --diff` |

## Scope

- In: deterministic compiled command transition contract, bounded command and
  next-state JSON, durable idempotency receipts, participant-authorized command
  POST route, exact version resolution, optimistic revisions, atomic snapshot
  update and sync invalidation, concurrency/privacy tests, smoke preservation,
  and documentation.
- Out: public session creation, challenges/invites, user-visible command
  history, turn inbox messages, deadlines/expiration, results, rewards, bots,
  a production game definition, QML game UI, server-provided clock/randomness,
  third-party plugins, and Git delivery.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/idempotent-revision-checked-game-commands.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
