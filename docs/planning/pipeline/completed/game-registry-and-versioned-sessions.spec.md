---
title: Game registry and versioned sessions
pipeline_id: 191a6334-576a-4573-844f-629a365ed8b2
status: Phase 5 — Complete PASS
ticket: TICKET-012
ticket_doc: docs/planning/tickets/closed/TICKET-012-game-registry-and-versioned-sessions.md
aar: docs/planning/knowledge/aar/AAR-012-game-registry-and-versioned-sessions.md
created: 2026-08-24
---

# Game registry and versioned sessions — spec

## Intent

Ship the first game-runtime foundation: compiled definitions are validated and
discoverable, while every durable session pins one exact rules version and an
initial deterministic snapshot before commands and challenges build on it.

## Scope

- In: the seven Ticket 012 EARS requirements, compiled registry, empty-until-
  playable production catalog, durable session/participant persistence,
  trusted internal creation, participant-private queries, sync invalidation,
  tests, live catalog smoke, and documentation.
- Out: public session creation, challenges, commands, idempotency, revision
  updates, turns, results, expiry, game inbox messages, bots, production game
  rules, QML game UI, external plugin loading, commits, pushes, and pull
  requests.

## Acceptance criteria (EARS)

See the seven requirements in
[`TICKET-012`](../../tickets/closed/TICKET-012-game-registry-and-versioned-sessions.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Production exposes an honestly empty compiled catalog until Ticket 015 adds the first playable game; tests inject a deterministic compiled fixture. | A placeholder game would turn roadmap intent into a misleading public contract, while injection still proves the runtime boundary now. |
| 2 | A session stores an immutable canonical game key and positive rules version, plus revision zero and a JSON object snapshot initialized by that exact compiled definition. | Durable state must remain interpretable after newer rules versions are registered, and Ticket 013 needs an explicit revision base. |
| 3 | Session creation is a trusted internal domain operation in this slice; only participant-scoped list/detail and the public catalog are routed. | Challenge acceptance will own public creation policy, so an interim endpoint must not bypass social authorization or become compatibility debt. |
| 4 | Human participants are unique existing personas assigned deterministic zero-based seats; the initial global bound is one through eight humans and each game manifest may narrow it. | Keeps persistence bounded and deterministic without predesigning bots or challenge policy. |
| 5 | Session responses expose public persona profiles plus game key/version, state, revision, status, and timestamps; they never expose account ownership or registry internals. | Game identity is persona-facing and the account/persona privacy boundary remains unchanged. |
| 6 | Session creation appends a `game_session_changed` invalidation for each participant in the same transaction, carrying only the session UUID. | Clients can recover new game inventory immediately without moving state onto WebSockets or revealing another participant through the hint. |

## Linked artifacts

- Ticket: [TICKET-012](../../tickets/closed/TICKET-012-game-registry-and-versioned-sessions.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled game/privacy/sync rules | observable version-pinned foundation recorded |
| 2 Design | Registry/session API, schema, manifest, regression table, CodeGraph evidence | actionable deterministic design |
| 3 Implement | Game-runtime crate, persistence, routes, sync variant, tests, smoke, and docs | focused checks and self-review |
| 3.5 Inspect | Correctness/security/concurrency/privacy/game-state ledger and fixes | fresh CodeGraph receipt |
| 4 Validate | Focused tests and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki lifecycle, submitted AAR, closed ticket and archive | no silent drops and matching wiki receipt |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
