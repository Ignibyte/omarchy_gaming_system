---
title: TICKET-004-account-registration
status: done
ticket_number: 004
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/account-registration.spec.md
---

# TICKET-004-account-registration

## Summary

Add a versioned account-registration endpoint that validates and canonicalizes
usernames, stores passwords only as parameterized Argon2id hashes, and proves
the behavior against PostgreSQL.

## Why

Registration is the first unfinished identity outcome and the prerequisite for
revocable device sessions and persona ownership. The foundation schema alone
does not yet provide a safe account-creation capability.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a client submits a valid username and password to `POST /v1/accounts`, the system shall create one active account and return `201 Created` with only its public registration fields. | Router/PostgreSQL integration test and live API smoke |
| REQ-002 | When registration succeeds, the system shall canonicalize the username and store the password only as a uniquely salted Argon2id v19 PHC hash using the locked resource parameters. | Domain tests and PostgreSQL integration test |
| REQ-003 | When registration input violates the username or password contract, the system shall return a stable `422 Unprocessable Entity` JSON error and shall not create an account. | Router/PostgreSQL integration tests |
| REQ-004 | When a canonical username is already registered, the system shall return a stable `409 Conflict` JSON error without replacing account data or disclosing password-derived material. | Router/PostgreSQL integration test |
| REQ-005 | When the delivery gate validates this slice, the system shall exercise registration through real migrations and PostgreSQL in addition to the normal static and unit checks. | `bin/gate.sh --diff` |

## Scope

- In: `POST /v1/accounts`, input normalization and validation, Argon2id
  hashing off the async executor, persistence, stable error responses,
  forward-only account constraints, tests, live smoke, and API documentation.
- Out: login, session issuance, password reset/change, account recovery,
  persona creation, rate limiting, email identity, and delivery to Git.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/account-registration.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
