---
title: Private inbox conversations and messages
pipeline_id: 6ee75fc9-36ae-4660-997e-cf22d0adc11a
status: Phase 5 — Complete PASS
ticket: TICKET-010
ticket_doc: docs/planning/tickets/closed/TICKET-010-private-inbox-conversations-and-messages.md
aar: docs/planning/knowledge/aar/AAR-010-private-inbox-conversations-and-messages.md
created: 2026-08-24
---

# Private inbox conversations and messages — spec

## Intent

Ship the durable private inbox foundation on accepted persona connections so
later challenge, turn, and result events have a typed server-authored home while
REST remains authoritative and reconnect notifications stay a separate slice.

## Scope

- In: accepted-pair conversation creation, typed connection system messages,
  bounded user messages and history, owner/participant authorization, public
  response DTOs, durable unread positions/counts, read acknowledgements,
  disconnect/block send policy, deterministic concurrency, tests, live smoke,
  and documentation.
- Out: group inboxes, client-authored system messages, game payload schemas,
  attachments, message mutation, search, WebSockets, durable notification
  cursors, commits, pushes, and pull requests.

## Acceptance criteria (EARS)

See the seven requirements in
[`TICKET-010`](../../tickets/closed/TICKET-010-private-inbox-conversations-and-messages.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Each canonical accepted persona pair owns one durable one-to-one conversation, created on first acceptance and reused across disconnect/reconnect cycles. | Prevents duplicate threads while keeping historical messages independent from live connection state. |
| 2 | The acceptance transaction appends a typed `connection_accepted` system message only when pending state transitions to accepted; an accepted retry does not append. | Proves the server-only typed-message path without allowing clients to forge future game events. |
| 3 | Conversation rows store both canonical participant IDs, each participant's monotonic last-read sequence, and the latest message sequence; messages use a conversation-local sequence plus a public UUID. | Keeps fixed-pair membership and unread invariants in one row, gives deterministic bounded history, and prevents one private thread from revealing unrelated message volume. |
| 4 | User messages contain trimmed 1–4,000 character control-safe text. The public POST accepts only `body`; type, sender, sequence, timestamps, and system fields are server-owned. | Bounds storage/rendering and structurally prevents client-authored system events or impersonation. |
| 5 | History defaults to 50 messages, caps at 100, orders each page by ascending message sequence, and returns `next_before` for older pages. Conversation inventory is capped at the 100 most recently active rows. | Prevents unbounded reads while keeping a simple stable REST recovery contract. |
| 6 | Sending locks both persona roots in canonical order through the connection domain, verifies an accepted unblocked pair, and only then locks/appends to the conversation. Acceptance follows the same persona-before-conversation order. | Serializes sends with removal/blocking and avoids a persona/conversation lock-order deadlock. |
| 7 | Removal or blocking never deletes conversations or messages. Both participants retain history access, but sends require a currently accepted, unblocked connection; unblock alone is insufficient. | Separates durable history from live social authorization and gives blocking a reliable no-contact boundary. |
| 8 | A user message is read for its sender at insertion and unread for the other participant. The acceptance system message is read for the accepting actor and unread for the requester. Read acknowledgement uses `GREATEST` and a message belonging to that conversation. | Makes unread state durable, monotonic, race-safe, and participant-private. |
| 9 | Message JSON is an explicit tagged union: user messages expose public sender plus body; system messages expose a typed `connection_accepted` object with its public actor. | Keeps clients exhaustive and safe while allowing later migrations to add reviewed game message variants. |
| 10 | A persona may have at most 100 pending incoming and 100 pending outgoing connection requests; creation enforces both limits while the requester and addressee roots are locked. | Bounds owner-scoped inventories and makes concurrent attempts converge on the same hard limit without adding pagination to this slice. |
| 11 | Directional block rows and inventories remain owner-private, but the product does not promise to conceal every indirect inference from a denied interaction with a known persona. | A fabricated successful request would be observably inconsistent unless the product added suppressed request state; the API instead documents the residual interaction-level disclosure honestly. |

## Linked artifacts

- Ticket: [TICKET-010](../../tickets/closed/TICKET-010-private-inbox-conversations-and-messages.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled social/privacy/concurrency rules | observable inbox contract recorded |
| 2 Design | Schema, message union, transaction flow, exact manifest, regression table, CodeGraph evidence | actionable privacy-safe design |
| 3 Implement | Inbox domain, migration, routes, acceptance hook, tests, smoke, and hand-maintained docs | focused checks and self-review |
| 3.5 Inspect | Correctness/security/concurrency/privacy ledger and fixes | fresh post-edit CodeGraph receipt |
| 4 Validate | Focused tests and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki lifecycle, submitted AAR, closed ticket and archive | no silent drops and matching wiki receipt |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
