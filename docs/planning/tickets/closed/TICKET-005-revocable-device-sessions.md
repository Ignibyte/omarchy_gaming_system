---
title: TICKET-005-revocable-device-sessions
status: done
ticket_number: 005
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/revocable-device-sessions.spec.md
---

# TICKET-005-revocable-device-sessions

## Summary

Add password-authenticated device sessions whose opaque bearer tokens are
stored only as hashes, expire server-side, can list only their account's
devices, and can be revoked immediately.

## Why

Registration creates credential identities but no authenticated capability.
Revocable sessions are the next roadmap dependency and must exist before
persona ownership can be authorized without exposing raw account credentials.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an active account submits correct credentials and a valid device name to `POST /v1/sessions`, the system shall create a device session and return `201 Created` with its opaque token exactly once under `Cache-Control: no-store`. | Router/PostgreSQL integration test and live API smoke |
| REQ-002 | When a session is created or presented, the system shall use a CSPRNG token with 256 random bits, persist only its SHA-256 digest, accept it only through `Authorization: Bearer`, and never log or return the stored digest. | Domain tests, database inspection, and response audit |
| REQ-003 | When login receives an unknown username, wrong password, or inactive account, the system shall perform password-cost work, return the same generic `401 Unauthorized` envelope, and create no session. | Router/PostgreSQL integration test |
| REQ-004 | When an unrevoked session remains inside its seven-day idle and 30-day absolute limits, the system shall authenticate it, update last use, and list only sessions owned by the same account. | Router/PostgreSQL integration test |
| REQ-005 | When an authenticated account revokes one of its session IDs, the system shall return `204 No Content`, immediately reject that bearer token, and return the same `404` result for absent and foreign session IDs. | Multi-account router/PostgreSQL integration test and live smoke |
| REQ-006 | When the delivery gate validates this slice, the system shall exercise session creation, authentication, expiry, ownership, and revocation through real migrations and PostgreSQL. | `bin/gate.sh --diff` |

## Scope

- In: `POST /v1/sessions`, authenticated `GET /v1/sessions`, authenticated
  `DELETE /v1/sessions/{session_id}`, device labels, hashed opaque tokens,
  generic login failures, absolute/idle expiry, last-use updates, account-status
  enforcement, ownership-safe revocation, tests, smoke, and documentation.
- Out: refresh tokens, token rotation, cookies, OAuth/OIDC, MFA, password reset,
  proxy/TLS deployment, distributed login throttling, persona endpoints, and
  delivery to Git.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/revocable-device-sessions.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
