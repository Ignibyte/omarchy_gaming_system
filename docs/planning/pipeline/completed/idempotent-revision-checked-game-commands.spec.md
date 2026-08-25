---
title: Idempotent revision-checked game commands
pipeline_id: 0857c2e2-6272-46f1-88d0-972c3d6d8f97
status: Phase 5 — Complete PASS
ticket: TICKET-013
ticket_doc: docs/planning/tickets/closed/TICKET-013-idempotent-revision-checked-game-commands.md
aar: docs/planning/knowledge/aar/AAR-013-idempotent-revision-checked-game-commands.md
created: 2026-08-24
---

# Idempotent revision-checked game commands — spec

## Intent

Ship the first safe mutable game boundary: one authorized participant command
executes through the session's pinned compiled rules, advances one durable
revision, remains replay-safe, and invalidates participant clients atomically.

## Scope

- In: the seven Ticket 013 EARS requirements, runtime transition contract,
  durable receipt schema, command route/domain transaction, exact-version and
  revision enforcement, participant sync, tests, smoke preservation, and docs.
- Out: session creation/challenges, visible command history, turns/messages,
  timers/results/rewards, bots, production rules, QML game UI, external plugin
  loading, commits, pushes, and pull requests.

## Acceptance criteria (EARS)

See the seven requirements in
[`TICKET-013`](../../tickets/closed/TICKET-013-idempotent-revision-checked-game-commands.md#ears-requirements).

## Locked decisions

### Runtime contract

- `GameDefinition::apply_command` receives only the current JSON object state,
  the actor's zero-based seat, and a JSON object command. It returns a new JSON
  object state or `GameCommandRejection`; no database handle, account/session
  identity, wall clock, network client, or random source crosses the crate
  boundary.
- `GameRegistry::apply_command(key, version, state, actor_seat, command)` resolves
  the exact immutable key/version. The runtime rejects non-object or oversized
  state, non-object or oversized command, fixture rejection, and non-object or
  oversized output with stable typed errors. State is bounded at 64 KiB and a
  command at 16 KiB after JSON serialization.
- Production continues to construct `GameRegistry::empty()`. The command API is
  real, but a stored session is executable only while its exact compiled rules
  version is present. PostgreSQL tests inject a deterministic fixture game.

### HTTP contract

- `POST /v1/personas/{persona_id}/game-sessions/{game_session_id}/commands`
  accepts a body capped at 32 KiB with `deny_unknown_fields`:

  ```json
  {
    "idempotency_key": "8f5d8f1d-48df-4f5a-b6e7-ad26eb30ae88",
    "expected_revision": 0,
    "command": { "kind": "advance" }
  }
  ```

- A committed or matching replay returns `200`, `Cache-Control: no-store`, and
  exactly `game_session_id`, `revision`, and `state`.
- Authentication failures remain `401`. An unowned actor persona is the same
  `404 persona_not_found` as other persona routes. A malformed, absent, or
  non-participating session is `404 game_session_not_found`. Malformed command
  inputs and stable game rejections are `422`; revision, idempotency, and
  unavailable-version failures are `409`. Conflict bodies do not disclose the
  current revision; clients refetch the participant-authorized session.

### Durable transaction and replay contract

- Migration `0011_idempotent_revision_checked_game_commands.sql` adds
  `game_session_commands`, keyed by `(game_session_id, idempotency_key)`, with
  the actor, expected/applied revisions, semantic `JSONB` command, committed
  `JSONB` state, and creation time. Composite membership and revision
  constraints preserve participant and one-receipt-per-revision invariants.
- After authenticating and owner-scoping the actor, the server begins a
  transaction and selects the active session joined to that participant using
  `FOR UPDATE OF session`. This row lock serializes all mutations for the
  session.
- While holding the lock, the server checks the session-wide idempotency UUID
  before checking the current revision. If an existing receipt has the same
  actor, expected revision, and PostgreSQL `JSONB`-equal command, its stored
  response is returned without runtime execution or another event. Any mismatch
  is an idempotency conflict.
- A new UUID must carry the current nonnegative revision. The server resolves
  the stored exact game version, executes the deterministic transition, updates
  state/revision/timestamp, inserts the receipt, and appends one minimal sync
  event per canonically ordered participant in the same transaction.
- Only accepted, committed commands receive durable replay receipts. Validation
  failures, game rejection, revision conflict, unavailable rules, and rollback
  leave no receipt, state update, or sync event.

### Regression plan

| Contract | Evidence |
|---|---|
| exact version and deterministic bounded runtime inputs/output | game-runtime unit tests |
| successful revision, response allowlist, receipt, and one event per participant | PostgreSQL/router test |
| semantic replay and actor/revision/command key collision | PostgreSQL/router test |
| stale/future revision and unavailable exact version rollback | PostgreSQL/router test |
| absent/malformed/non-participant privacy and owner scoping | multi-account router test |
| two distinct commands racing at one revision produce one winner | concurrent PostgreSQL/router test |
| game rejection and invalid output leave all durable state unchanged | runtime plus PostgreSQL tests |
| catalog, private reads, sync shape, and QML connector remain unchanged | existing suite and canonical diff gate |

### File manifest

- `crates/game-runtime/src/lib.rs` — deterministic command trait and registry.
- `migrations/0011_idempotent_revision_checked_game_commands.sql` — receipts.
- `crates/server/src/games.rs` — authorization and atomic command transaction.
- `crates/server/src/app.rs` — request/response route and stable errors.
- `crates/server/src/game_api_tests.rs` — fixture transitions and PostgreSQL API
  regressions.
- `docs/api.md`, `docs/architecture/system-overview.md`, `README.md` — public
  contract and architecture.
- Ticket/spec/notes/AAR, roadmap, knowledge, and OpenWiki artifacts — lifecycle
  evidence.

### Alternatives rejected

- WebSocket commands were rejected because sockets are wakeup hints, not a
  durable mutation or retry boundary.
- A global idempotency table or actor-scoped key was rejected because the
  contract is session-wide and the session row already provides the correct
  serialization boundary.
- Rechecking revision before replay lookup was rejected because a legitimate
  retry necessarily carries the now-stale original expected revision.
- Storing raw request text was rejected because semantically identical JSON
  object order must replay; PostgreSQL `JSONB` supplies semantic equality.
- A visible command/event history was deferred; the receipt is private
  operational state and stores only what is required to reproduce the response.

## Linked artifacts

- Ticket: [TICKET-013](../../tickets/closed/TICKET-013-idempotent-revision-checked-game-commands.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled revision/privacy rules | observable replay-safe command outcome recorded |
| 2 Design | Runtime transition, receipt schema, route/transaction, error map, regression table, CodeGraph evidence | actionable deterministic design |
| 3 Implement | Runtime/server/migration/route/tests/docs | focused checks and self-review |
| 3.5 Inspect | Correctness/security/concurrency/privacy/game-state ledger and fixes | fresh CodeGraph receipt |
| 4 Validate | Focused tests and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki lifecycle, submitted AAR, closed ticket and archive | no silent drops and matching wiki receipt |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
