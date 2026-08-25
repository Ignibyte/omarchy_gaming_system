---
title: Persona lifecycle and privacy — notes
pipeline_id: de76c9a6-5620-4e28-84da-dab463607c4a
---

# Persona lifecycle and privacy — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: complete the third roadmap identity outcome through the
  enforced pipeline after account registration and sessions were archived.
- Recall: accounts are private credential identities; sessions authenticate
  account authority; personas are the public social/game identity. Transport
  must not serialize persistence rows or trust client owner fields.
- Prior slices: TICKET-004 supplied canonical identity validation and explicit
  DTO/error patterns. TICKET-005 supplied server-derived account principals,
  generic Bearer errors, multi-account authorization tests, no-store responses,
  and a shared bound for password work.
- Upstream grounding: OWASP requires deny-by-default permission checks on every
  request using an object ID and warns that UUID unpredictability is not a
  substitute for object authorization. Its property-level guidance recommends
  explicit allowlisted input/output fields instead of generic object binding or
  serialization.

## Phase 2 — Design

- Architecture: add a persona domain module that owns validation,
  canonicalization, safe public profile models, authenticated persistence, and
  owner predicates. `app.rs` remains the transport adapter for JSON, path and
  Bearer extraction, explicit response mapping, cache headers, and stable HTTP
  errors. The session module exposes only a crate-private authenticated
  principal so persona operations never accept a client-supplied account ID.
- Authorization and privacy: create/list/edit authenticate before database
  work; list filters on the derived account ID and edit predicates on both
  persona and account IDs. Public lookup accepts only an exact canonicalizable
  handle. A dedicated public persona model contains only `id`, `handle`,
  `display_name`, `bio`, `status_message`, `created_at`, and `updated_at`, so
  `account_id` and session material cannot reach serialization accidentally.
- Persistence and conflicts: PostgreSQL remains authoritative for
  case-insensitive uniqueness. Writers normalize handles before insertion or
  update, a forward constraint prevents non-canonical direct writes, and the
  named unique index is mapped to a stable conflict. Updates use allowlisted
  optional fields and reject an empty patch before SQL; successful owner
  updates alone advance `updated_at`.
- CodeGraph evidence: explored the router, transport DTO/error boundary,
  Bearer parsing, session authentication, authenticated principal, session
  callers, module wiring, and router tests before edits. `authenticate` has two
  existing domain callers and is currently private; the router has five
  callers and direct router/PostgreSQL coverage. The bounded blast radius is
  `sessions.rs`, `app.rs`, `main.rs`, and the new persona domain/test modules.
  Graph coverage is advisory, so every new object-authorization path receives
  direct multi-account tests.
- Unsupported surfaces: migrations, dependency manifests, live shell smoke,
  planning artifacts, and API/architecture docs were inspected directly
  because CodeGraph does not model those formats completely.

### File manifest

| Path | Purpose |
|---|---|
| `crates/server/src/personas.rs` | Own canonical handles, bounded profile validation, public models, authenticated create/list/lookup/edit SQL, conflict mapping, and owner-scoped authorization. |
| `crates/server/src/sessions.rs` | Expose the validated account principal only within the crate for reuse by owned persona operations. |
| `crates/server/src/app.rs` | Add persona request/response DTOs, routes, 8 KiB body limits, no-store authenticated responses, exact-handle lookup, and stable persona errors. |
| `crates/server/src/persona_api_tests.rs`, `main.rs` | Add isolated multi-account PostgreSQL lifecycle/privacy tests and register the new modules. |
| `migrations/0004_canonical_persona_handles.sql` | Require every stored handle to match the application canonical format. |
| `scripts/dev.sh` | Extend the live smoke through persona create, owned list, public lookup, edit, handle movement, privacy checks, and existing revocation. |
| `docs/api.md`, `README.md`, architecture, roadmap, OpenWiki | Document persona contracts, ownership/privacy boundaries, implemented roadmap state, and operator smoke behavior. |
| Ticket/spec/notes/AAR/knowledge index | Preserve requirements, evidence, findings, lessons, and architecture decisions. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Router/PostgreSQL test asserts authenticated 201, canonical stored handle, trimmed profile values, no-store, exact public field set, and no private fields. |
| REQ-002 | Multi-account router/PostgreSQL test creates multiple owned personas plus a foreign persona and proves the owned list contains all and only the caller's profiles. |
| REQ-003 | Public router/PostgreSQL test proves mixed-case/trimmed canonical lookup, exact safe fields, and identical invalid/absent 404 bodies. |
| REQ-004 | Multi-account test proves owner edit, foreign denial, indistinguishable absent/foreign/malformed 404 outcomes, unchanged foreign state, and old/new handle movement. |
| REQ-005 | Unit and PostgreSQL tests cover every length/control/format boundary, empty patches, duplicate create/edit conflict, and preservation of the prior row after rejection. |
| REQ-006 | The non-fast database tier runs all migrated ignored SQLx tests before the expanded live server/QML smoke; the canonical diff gate records the final receipt. |

## Phase 3 — Implement

- Built: canonical multi-persona creation; authenticated account-only
  inventory; public exact-handle lookup; owner-scoped allowlisted editing;
  explicit seven-field public models; stable validation, conflict,
  authentication, and not-found errors; canonical-handle migration; 8 KiB
  write limits; authenticated no-store responses; unit and multi-account
  PostgreSQL coverage; live lifecycle/privacy smoke; and API, README,
  architecture, and roadmap updates.
- Focused checks: the production/test targets compiled; 13 fast tests passed
  with eight database tests correctly ignored; all eight isolated migrated
  PostgreSQL tests passed; warning-denied Clippy, Rustfmt, Bash syntax,
  whitespace, and pipeline structure passed; `scripts/dev.sh --smoke-test`
  completed registration, login, persona create/list/public lookup/edit/handle
  movement, session revocation/rejected reuse, and QML.
- Deviations: none. Optional `bio` and `status_message` creation fields use the
  schema's empty-string defaults at the transport boundary; this narrows
  required client input without changing the locked validation or public
  response contract.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Data flow and authorization | CodeGraph traced every owned handler through the persona domain to the shared session authenticator. List filters on the derived account ID; update predicates on both persona and account IDs. | None | Confirmed by direct multi-account PostgreSQL tests because graph test association is advisory. |
| 2 | Privacy and input boundaries | Persistence rows are mapped into a seven-field domain model and then a seven-field transport DTO; create/patch requests deny unknown fields. No owner or session field is available to serialization. | None | Exact-key response assertions and rejected owner-injection test retained. |
| 3 | Requirement evidence | Owner edit tests verified values and handle movement but did not directly prove timestamp advancement or authentication precedence for a malformed object ID. | Low | Added database timestamp advancement and invalid-session/malformed-ID assertions, then reran inspection and validation. |

- Post-fix CodeGraph evidence: re-explored the router, request/response DTOs,
  persona handlers, shared authentication principal, domain error mapping, and
  test call path after the final gated edit. The matching inspection receipt
  covers the current worktree; no unresolved finding remains.

## Phase 4 — Validate

- Tests run: 13 fast tests passed with eight database tests intentionally
  ignored in the fast tier; all eight migrated PostgreSQL tests passed in the
  isolated database tier. Warning-denied Clippy, Rustfmt, Rustdoc, Compose
  validation, Bash syntax, pipeline structure, hook self-tests, changed-file
  secret scanning, and whitespace checks passed. The live server/QML smoke
  passed the full account/session/persona lifecycle.
- Gate run: `bin/gate.sh --diff` completed all 12 checks and printed
  `GATE GREEN [diff]`; its receipt matched the gated worktree at the Phase 4
  exit.
- Skips or pre-existing failures: none. The eight `#[ignore]` tests shown in
  the fast tier all ran and passed in the canonical PostgreSQL tier.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Final evidence | Result |
  |---|---|---|
  | REQ-001 | Authenticated create PostgreSQL test, exact seven-field assertion, canonical stored row, no-store response, and live create smoke. | PASS |
  | REQ-002 | Multi-account inventory test proves every owned persona and no foreign persona, with exact public fields and no owner/session data. | PASS |
  | REQ-003 | Mixed-case public lookup test and smoke prove exact canonical resolution; invalid/absent lookups return identical 404 bodies. | PASS |
  | REQ-004 | Owner/foreign/absent/malformed edit test proves dual-ID SQL scoping, authentication precedence, timestamp advance, state preservation, and handle movement; smoke covers the owner path. | PASS |
  | REQ-005 | Unit and PostgreSQL tests cover bounds, controls, empty edits, unknown owner injection, duplicate create/edit, and preserved storage with stable domain errors. | PASS |
  | REQ-006 | The 12-check diff gate ran all eight migrated PostgreSQL tests and the live account/session/persona/QML lifecycle. | PASS |

- Docs: reconciled `docs/api.md`, README, roadmap, system overview, and the
  OpenWiki quickstart/runtime/boundary/validation pages. OpenWiki update run
  `3b7a22e2-8ae0-4865-8664-e34177789025` finished `complete` after claim
  inspection reported zero remaining issues.
- AAR: submitted AAR-006 at 5/5 and registered
  `PR-omarchy-bbs-owner-scope-account-resources-001` plus
  `AD-omarchy-bbs-public-persona-boundary-001` in the knowledge index.
- Archive: closed TICKET-006 and moved the sole active spec/notes pair into
  `docs/planning/pipeline/completed/`; no active pipeline remains.
- Final receipt audit: after OpenWiki authoring and archive, a fresh
  `bin/gate.sh --diff` again passed all 12 checks. The gate receipt and OpenWiki
  completion receipt both match the final gated worktree.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first no-run compile failed in two test-only expressions. | One assertion borrowed an array from a temporary decoded JSON value, and the test tried to serialize `Uuid` without enabling its optional Serde feature. | Bound the decoded document to a local variable and serialized the nil UUID as a string. | Compile all targets immediately after adding integration helpers and avoid dependency feature expansion for test literals. |
