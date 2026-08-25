---
title: Revocable device sessions
pipeline_id: 04b14a8f-9de6-4c77-9b34-d71fd2ea2132
status: Phase 5 — Complete PASS
ticket: TICKET-005
ticket_doc: docs/planning/tickets/closed/TICKET-005-revocable-device-sessions.md
aar: docs/planning/knowledge/aar/AAR-005-revocable-device-sessions.md
created: 2026-08-24
---

# Revocable device sessions — spec

## Intent

Turn registered account credentials into an opaque, server-revocable device
capability that can authorize the next persona slice without storing bearer
secrets or exposing account ownership.

## Scope

- In: credential login, opaque bearer issuance, digest-only persistence,
  device-session listing and revocation, idle/absolute expiry, account-status
  checks, stable authorization failures, live PostgreSQL tests, smoke, and docs.
- Out: refresh/rotation protocols, cookies, external identity providers, MFA,
  recovery, proxy/TLS setup, distributed login throttling, persona behavior,
  commits, pushes, and pull requests.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an active account submits correct credentials and a valid device name to `POST /v1/sessions`, the system shall create a device session and return `201 Created` with its opaque token exactly once under `Cache-Control: no-store`. | Router/PostgreSQL integration test and live API smoke |
| REQ-002 | When a session is created or presented, the system shall use a CSPRNG token with 256 random bits, persist only its SHA-256 digest, accept it only through `Authorization: Bearer`, and never log or return the stored digest. | Domain tests, database inspection, and response audit |
| REQ-003 | When login receives an unknown username, wrong password, or inactive account, the system shall perform password-cost work, return the same generic `401 Unauthorized` envelope, and create no session. | Router/PostgreSQL integration test |
| REQ-004 | When an unrevoked session remains inside its seven-day idle and 30-day absolute limits, the system shall authenticate it, update last use, and list only sessions owned by the same account. | Router/PostgreSQL integration test |
| REQ-005 | When an authenticated account revokes one of its session IDs, the system shall return `204 No Content`, immediately reject that bearer token, and return the same `404` result for absent and foreign session IDs. | Multi-account router/PostgreSQL integration test and live smoke |
| REQ-006 | When the delivery gate validates this slice, the system shall exercise session creation, authentication, expiry, ownership, and revocation through real migrations and PostgreSQL. | `bin/gate.sh --diff` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Use `POST /v1/sessions`, `GET /v1/sessions`, and `DELETE /v1/sessions/{session_id}`. | Separates credential exchange, device inventory, and revocation into durable resource semantics. |
| 2 | Issue `bbs1_` plus 32 OS-random bytes encoded base64url without padding; store only `SHA-256(token)` in the existing unique `BYTEA` column. | Supplies 256 unpredictable bits, keeps tokens opaque, and allows indexed lookup/revocation without retaining bearer secrets. |
| 3 | Return the raw token only from successful login with `Cache-Control: no-store`; require the RFC 6750 `Authorization: Bearer` header everywhere else. | Narrows token exposure and keeps it out of URLs, bodies, persistence, and ordinary logs. |
| 4 | Enforce a seven-day idle limit and a 30-day absolute expiry on every authenticated request; update `last_used_at` only after a valid lookup. | Server-side timeout enforcement limits stolen-token lifetime while supporting persistent devices. |
| 5 | Always perform one Argon2 operation and return the same `invalid_credentials` 401 for missing, wrong-password, suspended, or disabled accounts. | Reduces response and timing differences that enable account enumeration. |
| 6 | Trim device names, require 1–64 non-control characters, list every owned session with timestamps/revocation state, and never expose `account_id` or `token_hash`. | Gives clients enough device-management context without crossing the account/persona privacy boundary. |
| 7 | Revocation is owner-scoped and idempotent for an owned row; absent and foreign IDs both return `session_not_found` 404. A device may revoke its current session. | Prevents IDOR disclosure and makes retry behavior predictable while invalidating access immediately. |
| 8 | Keep SQLx PostgreSQL tests ignored in the fast loop and mandatory in non-fast gates; extend the live smoke through login, authenticated list, self-revocation, and rejected reuse. | Preserves quick local feedback while making the real session lifecycle delivery evidence. |
| 9 | Share a four-permit semaphore across registration and login Argon2 jobs, while keeping public/distributed attempt throttling out of this local slice. | Bounds the memory-heavy credential work without pretending an in-process limit is a deployment-wide abuse control. |

## Linked artifacts

- Ticket: [TICKET-005](../../tickets/closed/TICKET-005-revocable-device-sessions.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled security guidance | API, token, timeout, and privacy contracts recorded |
| 2 Design | Credential/session boundaries, migration, file manifest, regression plan | actionable design and CodeGraph evidence |
| 3 Implement | Session domain, API, migration, tests, smoke, docs | focused checks and self-review |
| 3.5 Inspect | Findings ledger and fixes | fresh post-edit CodeGraph analysis |
| 4 Validate | Unit/integration tests and canonical delivery gate | matching gate receipt |
| 5 Complete | OpenWiki reconciliation, AC audit, submitted AAR, archive | OpenWiki finish receipt and no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
