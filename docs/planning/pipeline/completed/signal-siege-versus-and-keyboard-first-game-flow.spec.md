---
title: Signal Siege versus and keyboard-first game flow
pipeline_id: 8d6fff91-f81f-4d9f-b0d3-302d96960781
status: Phase 5 — Complete PASS
ticket: TICKET-024
ticket_doc: docs/planning/tickets/closed/TICKET-024-signal-siege-versus-and-keyboard-first-game-flow.md
aar: docs/planning/knowledge/aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md
created: 2026-08-25
---

# Signal Siege versus and keyboard-first game flow — spec

## Intent

Deliver the first complete two-player game loop from the existing QML persona
home: browse the production catalog, challenge a connected persona, accept the
challenge, play an authoritative asynchronous match, and recover its terminal
result. The slice adds a new immutable Signal Siege rules version rather than
changing delivered v1, and feeds supported state through a platform-owned
presenter built from trusted inert QML nodes without claiming cartridge
provenance.

## Scope

- In: all nine Ticket 024 requirements; Signal Siege v2; production registry;
  strict selected-persona game/challenge/session client; trusted
  platform-owned presenter; keyboard screens; deterministic and real
  two-authority QML evidence;
  documentation, OpenWiki, security review, and AAR.
- Out: v1 semantic changes; schema/API migrations; untrusted executable
  frontend code; cartridge acquisition; remote-provider presentation;
  WebSocket/polling; matchmaking/rematches/spectators; rewards; moderation;
  installer; broad polish; Git delivery.

## Acceptance criteria (EARS)

The authoritative requirements are REQ-001 through REQ-009 in
[`TICKET-024`](../../tickets/closed/TICKET-024-signal-siege-versus-and-keyboard-first-game-flow.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Signal Siege v1 remains byte-for-behavior immutable; v2 is a separately registered exact two-human definition with alternating server-authoritative turns. | Delivered durable sessions pin v1, while challenge acceptance needs a real definition admitting exactly two humans. |
| 2 | V2 uses visible alternating turns rather than hidden simultaneous choices. | The current participant-private session snapshot is shared with both players; hidden pending actions would require a new per-seat redaction/private-state contract. |
| 3 | A dedicated game controller shares the one bearer-owning `ApiClient` only through the selected-persona request gateway. | Preserve the established process-memory credential and actor-authority boundary. |
| 4 | Supported compiled state is rendered by a clearly labeled platform-owned presenter using repository-owned inert node components; it is not wrapped as `omarchygs.render-plan/v1` or assigned a cartridge origin. | `TrustedCartridgeSurface` promises an authenticated archive origin. Preserving that provenance is more important than cosmetically reusing its envelope before package acquisition exists. |
| 5 | Catalog, challenges, sessions, and other-player turns refresh explicitly from REST after entry/action; this ticket adds neither polling nor WebSocket lifetime. | REST remains durable truth, and live transport scheduling is a separate authority/concurrency slice. |
| 6 | Only exact Signal Siege v1/v2 compiled sessions are playable in this client slice. Other catalog/session authority is displayed safely as unavailable presentation, not guessed or rendered as raw protocol state. | Remote-provider and installed-cartridge presentation need their own verified client integration. |
| 7 | Completion requires two independent QML authority controllers using two real accounts against the migrated production server to challenge, accept, alternate, complete, and refetch. | Fixture-only UI cannot prove the product's first-playable trust and persistence path. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-024-signal-siege-versus-and-keyboard-first-game-flow.md`
- Architecture: `docs/architecture/system-overview.md`, `docs/product-charter.md`
- Dependencies: Tickets 013, 016, 020–023; existing v1 game/challenge/session APIs.
- Intake: none.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, EARS scope, active spec/notes, open AAR | bounded first-playable challenge/game slice |
| 2 Design | Rules/state, authority/data flow, exact schemas, presenter/cartridge boundary, file manifest, regression map, CodeGraph receipt | actionable no-new-secret/no-executable design |
| 3 Implement | Signal Siege v2, game controller/screens/presenter, fixtures, live proof, and docs | focused Rust/QML/live checks |
| 3.5 Inspect | Correctness/game-state/security/privacy/concurrency/accessibility ledger and fixes | resolved findings and fresh CodeGraph receipt |
| 4 Validate | Focused suites and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki, AAR/knowledge, ticket/archive | matching completion and delivery receipts |
| Delivery | Staged review and separately authorized commit/push | explicit delivery authorization |
