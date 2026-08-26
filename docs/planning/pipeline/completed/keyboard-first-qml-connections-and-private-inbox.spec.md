---
title: Keyboard-first QML connections and private inbox
pipeline_id: 1dc98b6c-4c08-4ded-8c99-e1d58e9ac1a8
status: Phase 5 — Complete PASS
ticket: TICKET-023
ticket_doc: docs/planning/tickets/closed/TICKET-023-keyboard-first-qml-connections-and-private-inbox.md
aar: docs/planning/knowledge/aar/AAR-023-keyboard-first-qml-connections-and-private-inbox.md
created: 2026-08-25
---

# Keyboard-first QML connections and private inbox — spec

## Intent

Turn the authenticated persona home into the first usable two-person social
client. A player can find another persona by exact handle, manage connection
and private-block lifecycle, browse private conversations, exchange messages,
recover older history, and clear unread state with keyboard-only interaction.
The slice reuses the committed REST authority boundary and deliberately stops
before challenge/gameplay and live WebSocket hints.

## Scope

- In: all eight Ticket 023 requirements; exact social/inbox response
  validation; selected-persona actor binding; safe navigation; connection,
  block, conversation, message, unread, pagination, refresh, failure and
  invalid-session states; deterministic and live QML evidence; docs, OpenWiki,
  security review, and AAR.
- Out: challenge/game/session/cartridge UI; production two-person game
  selection; WebSocket subscription or polling; new API/schema behavior;
  persistent credentials; reporting/moderation; installer; broad visual polish;
  Git delivery.

## Acceptance criteria (EARS)

The authoritative requirements are REQ-001 through REQ-008 in
[`TICKET-023`](../../tickets/closed/TICKET-023-keyboard-first-qml-connections-and-private-inbox.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | A dedicated social controller shares Ticket 022's single `ApiClient`; it receives no raw bearer and every actor path comes from the currently selected owned persona. | Preserve one in-memory authority owner and prevent screen code from constructing credentials or choosing an arbitrary actor. |
| 2 | The client finds a new peer through the exact public-handle endpoint before issuing a UUID-based connection command. It does not add search, enumeration, or account discovery. | Exact lookup is the committed public contract and avoids inventing a broader discovery/privacy surface. |
| 3 | Connections, private blocks, conversations, and messages ship together; challenges and gameplay remain the next slice. | These resources share the same persona pair and conversation authority, while game presentation introduces separate registry/session/state contracts and a current production two-player-game gap. |
| 4 | Ticket 023 provides explicit screen-entry and manual durable refresh, not background polling or a WebSocket client. | REST is recovery truth; live hint lifetime, quotas, reconnect, and concurrent request scheduling deserve a separate reviewed client transport boundary. |
| 5 | The client renders exact allowlisted message variants as explicit plain text. Unknown system variants or malformed public profiles fail the response instead of becoming generic trusted content. | Inbox messages contain peer-controlled text and durable references; presentation must not create markup or protocol-confusion paths. |
| 6 | Removing or blocking a connection does not erase conversation history; sends are disabled only from current generic policy failure or refreshed relationship state. | This matches the durable server contract without teaching the client to infer private block direction. |
| 7 | Fixture evidence covers hostile states, but completion also requires two real accounts/personas and the migrated API through production QML controllers. | The vertical slice must prove owner scope, pair transitions, durable history, and read state across the actual server boundary. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-023-keyboard-first-qml-connections-and-private-inbox.md`
- Architecture: `docs/architecture/system-overview.md`, `docs/product-charter.md`
- Dependencies: Tickets 009–011 and 022; existing v1 social/inbox APIs.
- Intake: none.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, EARS scope, active spec/notes, open AAR | autonomous continuation and bounded social slice |
| 2 Design | Shared authority/data flow, exact schemas, file manifest, regression plan, CodeGraph receipt | actionable no-new-authority design |
| 3 Implement | Social controller, screens, fixtures, live smoke, and docs | focused deterministic and live checks |
| 3.5 Inspect | Correctness/security/privacy/concurrency/accessibility ledger and fixes | resolved findings and fresh CodeGraph receipt |
| 4 Validate | Focused tests and canonical gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki, AAR/knowledge, ticket/archive | matching completion and delivery receipts |
| Delivery | Staged review and separately authorized commit/push | explicit delivery authorization |
