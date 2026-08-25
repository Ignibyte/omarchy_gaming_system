---
title: TICKET-006-persona-lifecycle-and-privacy
status: done
ticket_number: 006
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/persona-lifecycle-and-privacy.spec.md
---

# TICKET-006-persona-lifecycle-and-privacy

## Summary

Add authenticated persona creation, owned inventory and editing, plus public
case-insensitive handle lookup, while keeping account ownership and session
internals out of every persona response.

## Why

Accounts and sessions now establish private credential identity. Personas are
the final roadmap identity boundary and the public identity required before
connections, inboxes, and games can safely refer to users.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a valid device session submits a valid persona to `POST /v1/personas`, the system shall create it and return `201 Created` with only its explicit public profile fields. | Router/PostgreSQL integration test and live smoke |
| REQ-002 | When an authenticated account requests `GET /v1/personas`, the system shall return every persona owned by that account and no persona owned by another account, without exposing `account_id`, credentials, tokens, or token digests. | Multi-account router/PostgreSQL integration test |
| REQ-003 | When any client requests `GET /v1/personas/by-handle/{handle}`, the system shall perform canonical case-insensitive lookup and return only public profile fields, or the same `404` result for invalid and absent handles. | Public router/PostgreSQL integration test and live smoke |
| REQ-004 | When an authenticated account patches one of its persona UUIDs, the system shall update only the allowed profile fields and timestamp; absent and foreign UUIDs shall return the same `404` result and make no change. | Multi-account router/PostgreSQL integration test and live smoke |
| REQ-005 | When create or edit input violates profile rules or canonical handle uniqueness, the system shall return stable `422` or `409` JSON errors and preserve existing persona state. | Unit and router/PostgreSQL tests |
| REQ-006 | When the delivery gate validates this slice, the system shall exercise creation, owned inventory, public lookup, editing, uniqueness, and privacy through real migrations and PostgreSQL. | `bin/gate.sh --diff` |

## Scope

- In: authenticated `POST/GET /v1/personas`, authenticated
  `PATCH /v1/personas/{persona_id}`, public
  `GET /v1/personas/by-handle/{handle}`, multiple personas per account,
  canonical handles, bounded profile fields, explicit public DTOs,
  ownership-safe mutation, tests, smoke, and documentation.
- Out: persona deletion, avatars/media, presence, moderation, blocking,
  connections, search/discovery beyond exact handle lookup, private profile
  fields, administrator APIs, and delivery to Git.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/persona-lifecycle-and-privacy.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
