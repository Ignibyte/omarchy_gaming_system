---
title: Persona lifecycle and privacy
pipeline_id: de76c9a6-5620-4e28-84da-dab463607c4a
status: Phase 5 — Complete PASS
ticket: TICKET-006
ticket_doc: docs/planning/tickets/closed/TICKET-006-persona-lifecycle-and-privacy.md
aar: docs/planning/knowledge/aar/AAR-006-persona-lifecycle-and-privacy.md
created: 2026-08-24
---

# Persona lifecycle and privacy — spec

## Intent

Expose the first public identity resource without weakening the private
account/session boundary: authenticated accounts own persona mutations, while
all returned representations contain an intentionally small public profile.

## Scope

- In: create/list/edit persona commands, exact public handle lookup, multiple
  personas per account, validation and canonical uniqueness, object-level
  authorization, explicit safe response mapping, PostgreSQL tests, smoke, docs.
- Out: deletion, media, presence, moderation, social graph, broad discovery,
  private profile fields, admin operations, commits, pushes, pull requests.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a valid device session submits a valid persona to `POST /v1/personas`, the system shall create it and return `201 Created` with only its explicit public profile fields. | Router/PostgreSQL integration test and live smoke |
| REQ-002 | When an authenticated account requests `GET /v1/personas`, the system shall return every persona owned by that account and no persona owned by another account, without exposing `account_id`, credentials, tokens, or token digests. | Multi-account router/PostgreSQL integration test |
| REQ-003 | When any client requests `GET /v1/personas/by-handle/{handle}`, the system shall perform canonical case-insensitive lookup and return only public profile fields, or the same `404` result for invalid and absent handles. | Public router/PostgreSQL integration test and live smoke |
| REQ-004 | When an authenticated account patches one of its persona UUIDs, the system shall update only the allowed profile fields and timestamp; absent and foreign UUIDs shall return the same `404` result and make no change. | Multi-account router/PostgreSQL integration test and live smoke |
| REQ-005 | When create or edit input violates profile rules or canonical handle uniqueness, the system shall return stable `422` or `409` JSON errors and preserve existing persona state. | Unit and router/PostgreSQL tests |
| REQ-006 | When the delivery gate validates this slice, the system shall exercise creation, owned inventory, public lookup, editing, uniqueness, and privacy through real migrations and PostgreSQL. | `bin/gate.sh --diff` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Use authenticated `POST/GET /v1/personas`, authenticated `PATCH /v1/personas/{persona_id}`, and public `GET /v1/personas/by-handle/{handle}`. | Separates account-owned mutation/inventory from the one intentionally public lookup surface. |
| 2 | Allow multiple personas per account, matching the existing non-unique `account_id` schema. | Preserves the deliberate account/persona distinction and avoids turning the first public persona into the credential identity. |
| 3 | Canonicalize handles by trimming and ASCII-lowercasing, then require 3–24 bytes matching `[a-z0-9][a-z0-9_-]*`; enforce the same format with a forward migration. | Prevents confusables and guarantees case-insensitive lookup/uniqueness across every writer. |
| 4 | Trim display names and status messages; require display names of 1–64 non-control characters, status messages of 0–160 non-control characters, and bios of at most 1,000 characters with only tab/newline controls allowed. | Supports human-facing Unicode and multiline bios while bounding storage and response size. |
| 5 | Cherry-pick only `id`, `handle`, `display_name`, `bio`, `status_message`, `created_at`, and `updated_at` into one public response model. | Prevents excessive property exposure and keeps `account_id` and authentication material structurally unavailable to serialization. |
| 6 | Derive ownership from the authenticated session on every owned list/create/edit request; edit SQL predicates on both persona and account IDs. | Enforces deny-by-default object authorization without trusting a client-supplied owner. |
| 7 | Return identical `persona_not_found` 404 bodies for absent/foreign edit UUIDs and invalid/absent public handles; handle conflicts return 409 without disclosing an account owner. | Reduces object/account disclosure while keeping exact public handle discovery useful. |
| 8 | Reject empty patches; permit only optional handle/display/bio/status fields, and set `updated_at=now()` only for a successful owner-scoped update. | Avoids mass assignment and gives clients a meaningful edit contract. |
| 9 | Apply an 8 KiB body limit and `Cache-Control: no-store` to authenticated persona responses; public exact-handle responses expose only the safe profile. | Bounds escaped Unicode/bio payloads and avoids caching account-associated inventory or mutations. |
| 10 | Extend the isolated multi-account SQLx tests and live smoke through create, owned list, public lookup, owner edit, foreign denial, and handle movement. | Makes BOLA and excessive-property regression evidence part of the canonical gate. |

## Linked artifacts

- Ticket: [TICKET-006](../../tickets/closed/TICKET-006-persona-lifecycle-and-privacy.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled authorization guidance | public/private and ownership contracts recorded |
| 2 Design | Persona domain, route/SQL ownership flow, file manifest, regression plan | actionable design and CodeGraph evidence |
| 3 Implement | Persona domain/API/migration/tests/smoke/docs | focused checks and self-review |
| 3.5 Inspect | Findings ledger and fixes | fresh post-edit CodeGraph analysis |
| 4 Validate | Unit/integration tests and canonical delivery gate | matching gate receipt |
| 5 Complete | OpenWiki reconciliation, AC audit, submitted AAR, archive | OpenWiki finish receipt and no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
