---
title: Persona connections and blocking
pipeline_id: fd2023e5-d943-4466-9320-28bcfdd97358
status: Phase 5 — Complete PASS
ticket: TICKET-009
ticket_doc: docs/planning/tickets/closed/TICKET-009-persona-connections-and-blocking.md
aar: docs/planning/knowledge/aar/AAR-009-persona-connections-and-blocking.md
created: 2026-08-24
---

# Persona connections and blocking — spec

## Intent

Ship the private-alpha social graph foundation on public persona identity so
future inboxes and challenges can authorize participant interactions without
collapsing personas into private accounts.

## Scope

- In: authenticated persona-scoped request creation and inventories, accepted
  connection inventories, addressee acceptance, participant removal and
  cancellation, directional block inventory, block/unblock, deterministic
  ordering, transaction/race handling, stable JSON contracts, tests, live
  smoke, and documentation.
- Out: inbox conversations or messages, unread state, challenges, presence,
  discovery beyond exact public handle lookup, connection counts on public
  profiles, recommendations, persona deletion, administrative moderation,
  WebSockets, cursor synchronization, commits, pushes, and pull requests.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an authenticated account uses an owned persona to request a valid foreign persona, the system shall create at most one pending relationship and return only public persona data; repeating the same request shall be idempotent. | Multi-account router/PostgreSQL tests and live smoke |
| REQ-002 | When an authenticated account reads connection requests for an owned persona, the system shall return its incoming and outgoing pending requests in stable order and no requests for another persona or account. | Multi-account router/PostgreSQL tests |
| REQ-003 | When the addressee accepts a pending request, the system shall atomically create one mutual connection visible to both personas; the requester, a foreign persona, or an absent request shall not be able to accept it. | Authorization, transaction, and concurrent PostgreSQL tests |
| REQ-004 | When either connected participant removes the other persona, the system shall remove the mutual connection; removal shall also cancel a pending request and remain idempotent without exposing prior relationship state. | Multi-account router/PostgreSQL tests and live smoke |
| REQ-005 | When an owned persona blocks another persona, the system shall privately record the directional block, atomically remove any pending or accepted relationship, reject requests in either direction with one non-disclosing error, and restore only the ability to request after idempotent unblock. | Multi-account block/race PostgreSQL tests and live smoke |
| REQ-006 | When a caller uses an absent, malformed, or foreign-owned acting persona, or a state-creating command names an invalid or same-account target, the system shall fail with stable non-disclosing errors and shall never reveal account ownership or mutate another persona's social state; idempotent delete commands may return `204` for absent target state after authenticating the actor. | Adversarial multi-account router/PostgreSQL tests and response-field review |
| REQ-007 | When the delivery gate validates this slice, the system shall exercise request, acceptance, removal, block, unblock, privacy, and concurrency behavior through real migrations and PostgreSQL. | `bin/gate.sh --diff` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Social resources are persona-to-persona; every acting persona is owner-scoped to the Bearer-authenticated account. | Preserves the account/persona privacy boundary while letting one account manage multiple public identities. |
| 2 | A canonical unordered persona pair has at most one pending or accepted relationship; the pending row records requester and addressee. | Makes mutual connection state unique and prevents duplicate or contradictory request rows. |
| 3 | Use idempotent path-addressed commands: `PUT` a request, `PUT` an accepted connection from an incoming request, `DELETE` the pair, and `PUT`/`DELETE` a directional block. | Gives reconnecting clients safe retry semantics before the general command-idempotency runtime exists. |
| 4 | A pair operation locks both persona rows in UUID order before reading or changing relationship/block state. | Serializes request-versus-block and opposite-request races without deadlocks or cross-table invariant windows. |
| 5 | Blocking is directional and private, removes pending or accepted state in the same transaction, and unblocking never restores old state. | Gives the blocker reliable safety and avoids revealing block state or resurrecting unwanted relationships. |
| 6 | Same-account personas cannot connect to or block each other. | Prevents self-graph manipulation and keeps account ownership from becoming an accidental social feature. |
| 7 | Inventories serialize only the existing seven-field public persona model plus relationship timestamps/direction; authenticated lists use `Cache-Control: no-store`. | Prevents account identifiers, block direction, and persistence internals from leaking through social DTOs or caches. |
| 8 | Missing/foreign acting personas share `persona_not_found`; blocked or otherwise unavailable targets share `connection_unavailable`; removal and unblock return 204 whether state existed or not after actor authorization. | Avoids object and block-state disclosure while keeping command outcomes stable. |
| 9 | Inbox, notification, and synchronization behavior remains a later ticket; this slice emits no fake events. | Keeps one shippable social-graph primitive and avoids pre-committing the durable event contract. |

## Linked artifacts

- Ticket: [TICKET-009](../../tickets/closed/TICKET-009-persona-connections-and-blocking.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled privacy and authorization rules | observable social contract recorded |
| 2 Design | Schema, domain/transport flow, exact file manifest, regression table, CodeGraph evidence | actionable concurrency-safe design |
| 3 Implement | Connection domain, migration, routes, tests, smoke, and hand-maintained docs | focused checks and self-review |
| 3.5 Inspect | Correctness/security/concurrency/privacy ledger and fixes | fresh post-edit CodeGraph receipt |
| 4 Validate | Focused tests and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki lifecycle, submitted AAR, closed ticket and archive | no silent drops and matching wiki receipt |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
