---
title: Signal Siege compiled game and solo bot matches
pipeline_id: b5b42330-4027-4e01-bad9-eaf21d858869
status: Phase 5 — Complete PASS
ticket: TICKET-021
ticket_doc: docs/planning/tickets/closed/TICKET-021-signal-siege-compiled-game-and-solo-bot-matches.md
aar: docs/planning/knowledge/aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md
created: 2026-08-25
---

# Signal Siege compiled game and solo bot matches — spec

## Intent

Turn the durable game foundation into an honest production-playable loop by
shipping one original, deterministic, asynchronous human-versus-bot game and
the owner-scoped launch/completion plumbing needed to start, finish, retain, and
recover a match through public REST and persona sync.

## Scope

- In: all seven Ticket 021 EARS requirements; Signal Siege v1 as a dedicated
  compiled Rust game; immutable production registry entry; one-owned-persona
  session start with durable idempotency and active inventory bound; typed
  active/completed transitions and explicit outcome; exact final replay;
  participant-private API and minimal sync; migration, tests, smoke, docs,
  OpenWiki, security inspection, and AAR.
- Out: fake bot identity rows, multiplayer Signal Siege, challenge integration,
  result-derived achievements or rewards, background scheduling, QML gameplay
  flows, Game Cartridge presentation/launch, provider authority, and delivery.

## Acceptance criteria (EARS)

The authoritative requirements are REQ-001 through REQ-007 in
[`TICKET-021`](../../tickets/closed/TICKET-021-signal-siege-compiled-game-and-solo-bot-matches.md#ears-requirements).

## Locked product decisions

| # | Decision | Reason |
|---|---|---|
| 1 | Signal Siege v1 is a one-human command-paced tactical duel against a deterministic server-owned bot. | It provides the roadmap's immediately playable opponent without manufacturing social identity or network authority. |
| 2 | Each accepted human action resolves one complete round, including the bot choice and simultaneous effects, and a fixed round ceiling guarantees termination. | A command-paced round is asynchronous/reconnect-safe and avoids a worker or hidden bot turn queue. |
| 3 | The human chooses from a small explicit action vocabulary with server-owned energy, defense, core, round, and outcome rules; the bot chooses from pre-command durable state only. | This is keyboard/accessibility friendly, deterministic, and prevents the bot from unfairly reading the submitted choice. |
| 4 | A solo launch is an owner-scoped public command distinct from the two-person challenge workflow and accepts only an exact definition that requires one human. | Challenges remain social invitations; a private bot match needs no second persona or connection. |
| 5 | Completion is durable session lifecycle, not a convention inferred only by the client from arbitrary JSON. | History, retry, and later result/achievement work need an authoritative terminal boundary. |

## Phase 2 decisions

- Signal Siege state starts at round 0 with eight core and two energy for each
  side, caps energy at four, and completes after core destruction or 12
  resolved rounds. The public state contains schema/rules version, round,
  phase, both bounded combatants, the last resolved round, and an optional
  explicit outcome.
- A command is exactly `{"kind":"play","action":"strike|guard|charge"}`.
  Strike and guard each cost one energy; strike deals two damage and guard
  blocks two for that round; charge gains two energy up to the cap. Human and
  bot effects resolve simultaneously.
- The bot chooses only from the pre-command durable state through a fixed v1
  policy, with charge as the no-energy fallback. It never inspects the current
  human action and has no clock, random source, database, network, persona, or
  hidden persisted state.
- Round-limit outcomes compare remaining core, then remaining energy, then
  draw. Core-destruction outcomes compare remaining core and may also draw
  after simultaneous damage. The completed outcome records winner, reason,
  cores, energies, and rounds played.
- The runtime returns a typed `active` or `completed` transition separately
  from JSON. PostgreSQL persists that lifecycle and completion timestamp, and
  each command receipt persists the applied lifecycle so a final retry returns
  the exact terminal result after the session no longer admits new commands.
- `POST /v1/personas/{persona_id}/game-sessions` starts only exact definitions
  whose manifest requires exactly one human. The owner-scoped UUID receipt is
  checked before current registry/cap admission, and at most 25 active solo
  starts per persona may exist concurrently.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, EARS scope, active spec/notes, open AAR | playable slice and exclusions fixed |
| 2 Design | Rules/state contract, registry/API/schema transaction design, file manifest, regression map | CodeGraph receipt and actionable design |
| 3 Implement | Game crate, lifecycle/start plumbing, migration, tests, smoke, hand docs | focused loop green |
| 3.5 Inspect | Rules, replay, auth/privacy, lifecycle, abuse, concurrency, simplification ledger | findings resolved plus fresh CodeGraph receipt |
| 4 Validate | Complete regression matrix and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki, AAR, ticket archive | matching OpenWiki and delivery receipts |
| Delivery | Authorized commit/push only | explicit user authorization |
