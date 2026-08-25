---
title: First-party remote-provider migration pilot
pipeline_id: f1e50ed7-4fdc-4df7-9aa9-5a208b7405a5
status: Phase 5 — Complete PASS
ticket: TICKET-019
ticket_doc: docs/planning/tickets/closed/TICKET-019-first-party-remote-provider-migration-pilot.md
aar: docs/planning/knowledge/aar/AAR-019-first-party-remote-provider-migration-pilot.md
created: 2026-08-25
---

# First-party remote-provider migration pilot — spec

## Intent

Make the accepted remote-provider architecture real for one operator-owned
first-party game. Door Legends v1 will move from a cartridge-only clean-room
proof to a separately built TLS provider that owns its rules, private state,
revision, time, and outcome, while OmarchyGS owns the player identity,
platform session envelope, launch policy, authenticated receipts, public
result/achievement projections, recovery state, and client-facing APIs. The
slice must prove one durable owner, not create a platform shadow engine.

## Scope

- In: all seven Ticket 019 requirements; the exact Constitution §10 amendment
  and ADR consequence; one active first-party Door Legends v1 release; an
  independently built and deployed provider process with its own durable
  database; all-or-none platform broker key configuration; operator-only pilot
  activation/suspension/retirement; provider-backed catalog/start/command/get/
  list behavior; signed callbacks; atomic result and achievement projection;
  participant-private recovery and sync; explicit provisioning, ready,
  reconciling, unavailable, suspended, completed, and retired presentation;
  restart, backup/restore, and permanent-retirement drill; tests, gate,
  operations docs, OpenWiki, and AAR.
- Out: external or self-service providers; marketplace/federation; more than
  one active pilot release; provider-hosted or executable frontend code;
  direct client-provider traffic; raw endpoint/key/grant/subject exposure;
  remote migration or rewriting of any existing Signal Siege session;
  compiled fallback for a provider-owned session; main-QML browse/play UI;
  ranking, economy, generalized cross-game rewards, and Git delivery.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | Before any provider-backed player route is enabled, Constitution §10 and ADR-0002 shall authorize only operator-registered, brokered, exact-release gameplay authority while preserving OmarchyGS identity, catalog, envelope, projection, audit, and recovery authority. | Constitutional/ADR review, structure checks, and full gate |
| REQ-002 | When Door Legends is launched, OmarchyGS shall create an idempotent participant-private platform envelope pinned to one provider release, and the separate provider shall be the only durable owner of game state and revision; OmarchyGS shall retain no writable gameplay snapshot or compiled fallback. | Migration constraints, server/provider database assertions, start/replay/race tests |
| REQ-003 | When a provider launch or command succeeds, conflicts, times out after commit, replays, or races, OmarchyGS shall converge by stable idempotency key, expected revision, authenticated receipt, and explicit reconciliation without timestamp winner selection or duplicate effects. | Separate-process fault, replay, restart, and concurrency tests |
| REQ-004 | When a signed result or achievement event arrives, OmarchyGS shall atomically authenticate and deduplicate it, bind it to the pinned provider/game/release/session/subject/revision and participant, validate exact platform-owned definitions, record only public projections, and append minimal participant sync invalidations. | Callback tamper/privacy, duplicate/race, result/achievement policy, rollback, and sync tests |
| REQ-005 | When the provider or pilot is unavailable, suspended, recovering, restored, or retired, new and existing sessions shall follow explicit fail-closed states and the operator runbook shall prove backup/restore, reconciliation, suspension, and permanent retirement without copying provider gameplay state into OmarchyGS. | Lifecycle matrix, outage/restart/restore drill, audit assertions, operator readback |
| REQ-006 | When public APIs expose the pilot, they shall return only allowlisted catalog, authority, availability, validated view, result, achievement, participant, and timestamp fields while keeping account identity, provider endpoint, pairwise subject, grants, keys, signed bodies, credentials, and database details private. | Exact multi-account API response and negative privacy corpus |
| REQ-007 | When the authority-pilot gate runs, it shall build Door Legends from a clean Git clone with the public provider protocol package only, launch its TLS server against an independent durable database, drive the real OmarchyGS broker/API/callback flow through restart and restore, and fail the canonical gate on any regression. | `scripts/test-provider-authority-pilot.sh` and `bin/gate.sh --diff` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Door Legends v1 is the sole remote-only pilot; compiled Signal Siege v1 and all existing sessions remain platform-owned and byte-for-byte compatible. | Door Legends already has separate-repository cartridge evidence, and a new remote-only release proves authority without attempting an unsafe live snapshot conversion. |
| 2 | A provider-backed `game_sessions` row is a platform envelope with nullable local state, a pinned immutable release, participants, provider-reported revision, lifecycle/availability, and authenticated projection references; commands can never run through `GameRegistry`. | A second writable state object or compiled fallback would create dual authority. |
| 3 | The provider view retained by OmarchyGS is a bounded authenticated presentation cache, never command input or recovery authority. | The trusted cartridge needs recoverable display data, but only the provider may advance rules state and revision. |
| 4 | Provider network calls remain server-to-server through `ProviderBroker`; clients receive no endpoint or reusable provider credential. | This preserves the accepted privacy, SSRF, TLS, quota, replay, and audit boundary. |
| 5 | Result and achievement definitions are operator-pinned platform policy. A provider proposes exact game-scoped claims; one platform transaction authenticates/deduplicates and records projections plus sync. | Providers own game outcomes, not global identity, storage, achievement definitions, or notification policy. |
| 6 | A remote session never fails back to compiled rules. Operational rollback disables new launches and marks existing envelopes unavailable/read-only until provider restore, reconciliation, completion, or permanent retirement. | Failback would fork state and make the authority boundary unverifiable. |
| 7 | Production broker secrets are an all-or-none startup configuration; absent configuration leaves provider routes/catalog disabled, while partial or malformed configuration prevents startup. | Silent partial activation is unsafe, but local compiled operation must remain available when the optional pilot is deliberately disabled. |
| 8 | The Door Legends provider is built from a clean clone and uses an independent database and only the public protocol surface; platform tests may use the compile-time exact-loopback conformance transport, which remains absent from the production server binary. | This proves the user's separate-game/server model without creating a general private-network escape hatch. |
| 9 | REST/cursor recovery remains durable truth. WebSockets may wake clients but are neither provider transport nor projection authority. | Reconnect, result delivery, and outage recovery cannot depend on a live socket. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-019-first-party-remote-provider-migration-pilot.md`
- Architecture: `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md`, `docs/architecture/game-cartridges.md`
- Dependency: `docs/planning/pipeline/completed/production-remote-provider-security-foundation.spec.md`
- First-party release proof: `examples/first-party-door-legends/`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Refined ticket, spec, notes, open AAR | user-authorized autonomous continuation and bounded locked decisions |
| 2 Design | Authority/data flow, migration/API/protocol manifests, regression plan, CodeGraph receipt | actionable single-owner design |
| 3 Implement | Platform integration, separate provider, tests, runbooks, amendment | focused build and conformance |
| 3.5 Inspect | Correctness/security/data/concurrency/simplification findings and fixes | resolved ledger and fresh CodeGraph receipt |
| 4 Validate | Focused tests, independent provider drill, canonical gate | matching gate receipt |
| 5 Complete | EARS audit, OpenWiki, AAR/knowledge, ticket/archive | matching completion and gate receipts |
| Delivery | Staged review and explicit commit/push authorization | matching receipt and user authorization |
