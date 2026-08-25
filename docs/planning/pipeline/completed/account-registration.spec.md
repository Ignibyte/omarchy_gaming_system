---
title: Account registration
pipeline_id: 1dfcb0d0-9a29-4774-86aa-b93e82fd9d11
status: Phase 5 — Complete PASS
ticket: TICKET-004
ticket_doc: docs/planning/tickets/closed/TICKET-004-account-registration.md
aar: docs/planning/knowledge/aar/AAR-004-account-registration.md
created: 2026-08-24
---

# Account registration — spec

## Intent

Ship the first authoritative identity command: safe account creation through a
versioned JSON API with database-backed proof of canonical uniqueness and
Argon2id-only password storage.

## Scope

- In: `POST /v1/accounts`, account-domain validation and persistence,
  Argon2id hashing, stable client errors, database constraints, focused tests,
  live smoke coverage, and operator/API documentation.
- Out: authentication, device sessions, persona operations, password
  lifecycle features, rate limiting, email, commits, pushes, and pull requests.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a client submits a valid username and password to `POST /v1/accounts`, the system shall create one active account and return `201 Created` with only its public registration fields. | Router/PostgreSQL integration test and live API smoke |
| REQ-002 | When registration succeeds, the system shall canonicalize the username and store the password only as a uniquely salted Argon2id v19 PHC hash using the locked resource parameters. | Domain tests and PostgreSQL integration test |
| REQ-003 | When registration input violates the username or password contract, the system shall return a stable `422 Unprocessable Entity` JSON error and shall not create an account. | Router/PostgreSQL integration tests |
| REQ-004 | When a canonical username is already registered, the system shall return a stable `409 Conflict` JSON error without replacing account data or disclosing password-derived material. | Router/PostgreSQL integration test |
| REQ-005 | When the delivery gate validates this slice, the system shall exercise registration through real migrations and PostgreSQL in addition to the normal static and unit checks. | `bin/gate.sh --diff` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Expose registration as `POST /v1/accounts`; return only `id` and canonical `username`. | Keeps the durable API versioned and excludes internal status, timestamps, and password-derived data. |
| 2 | Trim and ASCII-lowercase usernames, then require 3–32 bytes matching `[a-z0-9][a-z0-9_-]*`. | Produces one predictable canonical identity while rejecting Unicode confusables in the account namespace. |
| 3 | Require passwords of 12–128 bytes without composition rules and never trim them. | Establishes a meaningful floor while allowing passphrases and bounding hashing work. |
| 4 | Hash in `spawn_blocking` with random salts and Argon2id v19 parameters `m=19456 KiB,t=2,p=1`, stored as PHC strings. | Meets the OWASP Argon2id baseline without blocking Tokio worker threads and retains parameters for future verification/upgrades. |
| 5 | Map validation to `422`, canonical uniqueness to `409`, and unexpected failures to a generic `500` envelope. | Gives clients stable outcomes without exposing database or cryptographic internals. |
| 6 | Use isolated `#[sqlx::test]` databases with repository migrations for persistence behavior; keep the full gate's existing live stack smoke. | Proves real PostgreSQL semantics without shared-test cleanup or treating schema inspection as executable evidence. |

## Linked artifacts

- Ticket: [TICKET-004](../../tickets/closed/TICKET-004-account-registration.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled constraints | scope and security contract recorded |
| 2 Design | Account boundary, file manifest, migration and regression plan | actionable design and CodeGraph evidence |
| 3 Implement | Domain, API, migration, tests, smoke, docs | focused checks and self-review |
| 3.5 Inspect | Findings ledger and fixes | fresh post-edit CodeGraph analysis |
| 4 Validate | Unit/integration tests and canonical delivery gate | matching gate receipt |
| 5 Complete | OpenWiki reconciliation, AC audit, submitted AAR, archive | OpenWiki finish receipt and no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
