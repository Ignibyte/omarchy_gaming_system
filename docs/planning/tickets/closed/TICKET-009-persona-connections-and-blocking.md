---
title: TICKET-009-persona-connections-and-blocking
status: closed
ticket_number: 009
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/persona-connections-and-blocking.spec.md
---

# TICKET-009-persona-connections-and-blocking

## Summary

Add persona-to-persona connection requests, private request and connection
inventories, acceptance and removal, plus private blocking that atomically
removes any relationship and prevents a new one until unblocked.

## Why

Personas now provide the public identity boundary. Connections are the first
social primitive required before private inboxes and game challenges can
authorize interactions between people without exposing account ownership.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an authenticated account uses an owned persona to request a valid foreign persona, the system shall create at most one pending relationship and return only public persona data; repeating the same request shall be idempotent. | Multi-account router/PostgreSQL tests and live smoke |
| REQ-002 | When an authenticated account reads connection requests for an owned persona, the system shall return its incoming and outgoing pending requests in stable order and no requests for another persona or account. | Multi-account router/PostgreSQL tests |
| REQ-003 | When the addressee accepts a pending request, the system shall atomically create one mutual connection visible to both personas; the requester, a foreign persona, or an absent request shall not be able to accept it. | Authorization, transaction, and concurrent PostgreSQL tests |
| REQ-004 | When either connected participant removes the other persona, the system shall remove the mutual connection; removal shall also cancel a pending request and remain idempotent without exposing prior relationship state. | Multi-account router/PostgreSQL tests and live smoke |
| REQ-005 | When an owned persona blocks another persona, the system shall privately record the directional block, atomically remove any pending or accepted relationship, reject requests in either direction with one non-disclosing error, and restore only the ability to request after idempotent unblock. | Multi-account block/race PostgreSQL tests and live smoke |
| REQ-006 | When a caller uses an absent, malformed, or foreign-owned acting persona, or a state-creating command names an invalid or same-account target, the system shall fail with stable non-disclosing errors and shall never reveal account ownership or mutate another persona's social state; idempotent delete commands may return `204` for absent target state after authenticating the actor. | Adversarial multi-account router/PostgreSQL tests and response-field review |
| REQ-007 | When the delivery gate validates this slice, the system shall exercise request, acceptance, removal, block, unblock, privacy, and concurrency behavior through real migrations and PostgreSQL. | `bin/gate.sh --diff` |

## Scope

- In: authenticated persona-scoped request creation and inventories, accepted
  connection inventories, addressee acceptance, participant removal and
  cancellation, directional block inventory, block/unblock, deterministic
  ordering, transaction/race handling, stable JSON contracts, tests, live
  smoke, and documentation.
- Out: inbox conversations or messages, unread state, challenges, presence,
  discovery beyond exact public handle lookup, connection counts on public
  profiles, recommendations, persona deletion, administrative moderation,
  WebSockets, cursor synchronization, and Git delivery.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/persona-connections-and-blocking.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
