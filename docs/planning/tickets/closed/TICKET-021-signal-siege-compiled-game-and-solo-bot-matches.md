---
title: TICKET-021-signal-siege-compiled-game-and-solo-bot-matches
status: closed
ticket_number: 021
type: feature
created: 2026-08-25
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/signal-siege-compiled-game-and-solo-bot-matches.spec.md
---

# TICKET-021-signal-siege-compiled-game-and-solo-bot-matches

## Summary

Ship Signal Siege v1 as OmarchyGS's first production compiled game: a
deterministic asynchronous duel between one human persona and a server-owned
bot. Add the owner-scoped launch and completed-result plumbing needed to start,
play, reconnect to, and finish the match through the public API.

## Why

The platform has an exact-version registry, durable sessions, idempotent
commands, challenges, inbox notifications, and reconnect recovery, but
production still advertises no playable rules and sessions cannot reach a
completed result. This slice turns those foundations into the first honest
server-authoritative game loop without inventing a bot account or weakening the
compiled-game boundary.

## Outcome

All seven requirements passed. Production now advertises exactly Signal Siege
v1; an authenticated account can idempotently launch it for an owned persona,
play deterministic simultaneous human/bot rounds to a bounded durable outcome,
replay the exact final command, and recover completed history through
participant-private REST plus payload-minimal cursor invalidations. The bot is
rules and state rather than an account or persona. The final 16-stage gate
passed all 43 PostgreSQL tests and the live PostgreSQL/API/Signal-Siege/QML
smoke.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When production starts, the public catalog shall advertise exactly Signal Siege v1 as a one-human compiled game whose initialization and command transitions are deterministic, bounded, database-free, clock-free, and ambient-randomness-free. | Production-registry integration test plus exhaustive game-crate unit tests |
| REQ-002 | When an authenticated account starts Signal Siege for an owned persona with a new idempotency key, the system shall create one version-pinned active session with that persona in seat 0, enforce a bounded active-solo inventory, append one persona-local invalidation, and commit the launch receipt atomically. | PostgreSQL start, ownership, cap, rollback, and sync tests |
| REQ-003 | When the same persona retries an exact solo-start request, the system shall return the same durable session without another session or event even after the current registry changes; reusing the key for another game identity shall conflict. | Durable replay and collision integration tests |
| REQ-004 | When the human submits a valid Signal Siege action at the current revision, the system shall resolve exactly one human/bot round under the pinned v1 rules, persist the snapshot/revision/receipt, and notify the participant once; malformed, unaffordable, wrong-seat, stale, or colliding commands shall not mutate state. | Rules matrix and PostgreSQL command/rejection/idempotency tests |
| REQ-005 | When Signal Siege reaches a core-destruction or fixed-round terminal condition, the system shall persist one completed session and explicit bounded outcome, reject new commands, retain list/detail history, and replay the exact final command without another transition or notification. | Completion, final-replay, post-completion, and reconnect tests |
| REQ-006 | When solo-game data crosses HTTP, sync, or persistence boundaries, the system shall expose only the game/session public contract and owned public persona projection; it shall not create or expose a bot account/persona, credentials, ownership rows, idempotency keys, future bot choices, or payload-bearing WebSocket truth. | Response allowlist, database absence, privacy-equivalence, sync, and WebSocket review/tests |
| REQ-007 | When the live development smoke runs against the production registry, it shall create a persona, launch Signal Siege, play through a terminal result, refetch the same history, and observe reconnect-safe invalidations before the visible QML health connector succeeds. | `scripts/dev.sh --smoke-test` through the canonical diff gate |

## Scope

- In: original Signal Siege v1 rules and deterministic bot; compiled production
  registration; one-human idempotent session start; active-solo cap; durable
  active/completed lifecycle and outcome; final-command replay; participant
  REST/sync privacy; forward-only migration; unit, PostgreSQL, live smoke,
  architecture, API, OpenWiki, security, and AAR evidence.
- Out: a bot account or public bot persona; human-versus-human Signal Siege;
  matchmaking or challenges to the bot; achievements, rankings, rewards,
  rematches, resignations, timeout workers, QML account/persona/gameplay screens,
  cartridge/provider authority, remote execution, and Git delivery.

## Links

- Depends on: `TICKET-012`, `TICKET-013`, `TICKET-020`
- Pipeline: [completed spec](../../pipeline/completed/signal-siege-compiled-game-and-solo-bot-matches.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
