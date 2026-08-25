---
title: Revocable device sessions — notes
pipeline_id: 04b14a8f-9de6-4c77-9b34-d71fd2ea2132
---

# Revocable device sessions — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: continue the newly enforced pipeline through the second
  roadmap identity outcome after account registration completed.
- Recall: account/persona identities remain separate; domain modules own
  authentication and authorization; raw credentials and bearer secrets must
  not enter responses, storage, or logs beyond their one necessary exchange;
  PostgreSQL behavior requires live evidence.
- Prior slice: TICKET-004 established canonical usernames, Argon2id PHC hashes,
  generic internal errors, a real-stack gate, and a reusable PostgreSQL router
  test path. Its production dependency failure adds a required plain binary
  build after runtime dependency changes.
- Upstream grounding: OWASP calls for CSPRNG session identifiers with at least
  128 random bits, server-side meaning/invalidation, idle and absolute timeouts,
  no token logging, and `no-store` on responses containing identifiers. Its
  authentication guidance requires generic login errors and comparable work for
  missing and existing accounts. RFC 6750 defines the Bearer authorization
  header used by the API.
- Known limitation: this local product slice does not add distributed login
  throttling or production TLS/proxy policy. Both remain required before a
  public deployment; the implementation still bounds credential request bodies
  and Argon2 inputs.

## Phase 2 — Design

- Architecture: extract Argon2 construction and blocking hash/verify work into
  a credential module shared by registration and login. Add a session domain
  module that owns credential lookup, token generation/digesting, database
  issuance, authenticated lookup, account scoping, listing, and revocation.
  `app.rs` remains a transport adapter for JSON, Bearer headers, status codes,
  cache headers, and path extraction; the existing pool remains shared state.
- Enumeration/timing: device-name validation is independent of identity. Every
  syntactically acceptable login performs one Argon2 operation; missing or
  invalid usernames use a dummy hash path, while wrong-password and inactive
  accounts verify the stored hash before returning the same public 401.
- Persistence/races: issuance uses `INSERT ... SELECT` from an active account
  after password verification, so an intervening suspension cannot create a
  usable session. Authentication atomically updates last use only for a token
  whose account is active and whose revocation, idle, and absolute conditions
  pass. PostgreSQL remains authoritative for token-digest uniqueness.
- CodeGraph evidence: explored the current handler → account domain → Argon2
  flow and all returned callers/tests before edits. Registration is the sole
  credential-helper caller; the router is its only external caller. This makes
  the shared-credential extraction bounded to `accounts.rs`, `app.rs`, and the
  two new domain modules. Graph coverage remains advisory and is supplemented
  by direct embedded and PostgreSQL test inspection.
- Unsupported surfaces: migrations, Bearer shell smoke, dependency manifests,
  and API docs were inspected directly because CodeGraph does not model those
  formats.

### File manifest

| Path | Purpose |
|---|---|
| `Cargo.toml`, `crates/server/Cargo.toml`, `Cargo.lock` | Add current base64url, SHA-256, and UUID dependencies; enable SQLx UUID support. |
| `crates/server/src/credentials.rs` | Centralize explicit Argon2id hashing, verification, dummy work, and blocking-task error handling. |
| `crates/server/src/accounts.rs` | Reuse shared credential hashing and expose canonical username parsing inside the crate. |
| `crates/server/src/sessions.rs` | Own device-name rules, credential authentication, 256-bit opaque tokens, digest lookup, timeouts, listing, and owner-scoped revocation. |
| `crates/server/src/app.rs` | Add session routes, Bearer parsing, no-store issuance responses, stable errors, and `WWW-Authenticate`. |
| `crates/server/src/session_api_tests.rs`, `main.rs` | Keep multi-account PostgreSQL lifecycle tests separate and register new modules. |
| `migrations/0003_device_session_metadata.sql` | Add bounded device names and require 32-byte token digests. |
| `scripts/dev.sh` | Extend smoke through login, authenticated list, self-revocation, and rejected reuse. |
| `docs/api.md`, README, architecture, roadmap, OpenWiki | Document implemented authentication semantics, TLS/throttling limits, and the next identity boundary. |
| Ticket/spec/notes/AAR/knowledge index | Preserve evidence, failures, rules, and decisions. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Router/PostgreSQL test asserts 201, exact one-time token placement, no-store, safe session metadata, and active database row. |
| REQ-002 | Unit tests decode 32 random bytes, prove unique tokens and deterministic 32-byte digests; database test proves raw-token absence and response digest absence. |
| REQ-003 | PostgreSQL test compares unknown, wrong-password, suspended, and disabled outcomes and asserts no rows; credential tests cover correct/wrong/dummy Argon paths. |
| REQ-004 | Bearer list test proves account scoping, last-use advance, account-status enforcement, seven-day idle rejection, and absolute-expiry rejection. |
| REQ-005 | Multi-account test proves owned/idempotent/self revocation, immediate bearer rejection, and indistinguishable absent/foreign 404 outcomes; live smoke covers self-revocation. |
| REQ-006 | Existing non-fast database tier runs all ignored migrated SQLx tests before the expanded live API/QML smoke. |

## Phase 3 — Implement

- Built: shared blocking Argon2id hashing/verification with dummy missing-account
  work; OS-random 256-bit base64url bearer tokens; SHA-256 digest-only storage;
  device metadata migration; active-account login; 30-day absolute and
  seven-day idle authentication; last-use updates; owner-scoped inventory and
  idempotent revocation; stable Bearer/API errors; no-store issuance; unit and
  multi-account PostgreSQL tests; expanded live smoke; and API/operator docs.
- Focused checks: the plain production build passed after dependency changes;
  warning-denied Clippy passed; ten fast tests and five ignored isolated
  PostgreSQL tests passed; `scripts/dev.sh --smoke-test` completed registration,
  login, inventory, self-revocation, rejected reuse, and QML; Bash syntax,
  whitespace, and pipeline structure passed.
- Deviations: none. The current crate resolver selected UUID 1.25.0 within the
  declared compatible 1.24.1 range; no API or design change was needed.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Bearer secrecy | The raw 256-bit token exists only in the non-`Debug` creation model/response, while persistence and lookup use a validated 32-byte SHA-256 digest. | critical | Confirmed through graph flow, source audit, unit decoding/digest tests, database inspection, response field checks, and the absence of token/header logging. |
| 2 | Enumeration | Missing usernames could skip stored-hash verification and inactive status could produce a distinguishable outcome. | high | Dummy hashing ensures one Argon2 operation for missing accounts; existing accounts verify before status rejection; all tested failures share one 401 body/header and create no row. |
| 3 | Resource exhaustion | `spawn_blocking` protected Tokio workers but allowed too many concurrent 19 MiB Argon2 jobs. | high | Added one four-permit semaphore shared by registration hashing and login verification; the final graph pass confirms both callers acquire it before blocking work. |
| 4 | Response caching | The creation response was no-store, but authenticated device inventory remained cacheable by default. | medium | Added and tested `Cache-Control: no-store` on list responses. |
| 5 | IDOR/privacy | Session UUIDs must not reveal or mutate another account's device rows. | critical | Authentication derives account ownership server-side; list filters by that ID, revoke predicates on both IDs, and foreign/absent tests return identical 404 bodies. No account IDs or digests enter JSON. |
| 6 | Expiry/revocation | Client-side expiry or stale account status could leave a stolen bearer usable. | critical | A single authenticated UPDATE requires active account, unrevoked row, absolute validity, and idle validity before advancing last use; tests cover each invalidation and immediate self-revocation. |
| 7 | Deployment boundary | Bearer transport needs TLS and login needs distributed attempt throttling before public exposure. | high | Explicitly out of this local slice and documented in the API and plan; do not treat the in-process Argon semaphore as rate limiting. |
| 8 | Coverage interpretation | CodeGraph again reported session symbols uncovered while returning the dedicated lifecycle tests. | low | Applied the standing advisory rule; direct unit, isolated PostgreSQL, and live smoke evidence remain authoritative. |

- Final CodeGraph pass: traced issuance, list, authentication, digesting,
  credential permits, and revocation after the two inspection fixes. The
  inspection receipt matches the current implementation worktree.

## Phase 4 — Validate

- Tests run: ten fast tests passed for accounts, shared credentials, token
  entropy/digesting, device names, request bounds, health, and configuration.
  Five ignored router tests passed against isolated migrated PostgreSQL
  databases, including the two registration regressions and three multi-account
  session lifecycles.
- Gate run: `bin/gate.sh --diff` passed all 12 checks: rustfmt, Clippy, fast
  tests, rustdoc, Compose, shell syntax, pipeline structure, changed secret
  scan, Codex hook self-tests, whitespace, PostgreSQL integration tests, and the
  live registration/session/QML smoke. It printed `GATE GREEN [diff]` and wrote
  a matching worktree receipt.
- Skips or pre-existing failures: no validation skips. The explicitly ignored
  SQLx tests ran in the mandatory database tier. Headless QML emitted the
  existing non-fatal EGL warnings and passed.

## Phase 5 — Complete

- Acceptance-criteria audit:

| Requirement | Verdict | Evidence |
|---|---|---|
| REQ-001 | satisfied | Valid active-account login returned 201, one opaque token and safe device metadata under `Cache-Control: no-store`; database and live smoke both exercised it. |
| REQ-002 | satisfied | Unit tests decoded 32 random bytes and verified deterministic 32-byte SHA-256 digests; migration enforces digest length; PostgreSQL/JSON audits proved raw-token and account/digest separation; Bearer parsing is header-only. |
| REQ-003 | satisfied | Unknown, wrong-password, suspended, and disabled logins performed the shared Argon path, returned identical 401 bodies/headers, and inserted no additional sessions. |
| REQ-004 | satisfied | Multi-account PostgreSQL tests proved account-scoped inventory, current-device marking, last-use advance, seven-day idle failure, 30-day absolute failure, and inactive-account failure. |
| REQ-005 | satisfied | Foreign and absent UUIDs returned identical 404 bodies; owned revocation was idempotent; revoked and self-revoked tokens failed immediately; the live smoke repeated self-revocation/reuse. |
| REQ-006 | satisfied | The final `bin/gate.sh --diff` passed all 12 checks, including five migrated PostgreSQL router tests and the live registration/session/QML path. |

- Docs: updated `docs/api.md`, README, system overview, roadmap, and four
  OpenWiki pages. OpenWiki run `3a46ace1-e44c-4c55-83a8-a8239d70f94b`
  completed after atomically rejecting two mistyped claim-ID attempts; the
  successful pass repaired all nine stale claims and produced a matching
  completion receipt.
- AAR: AAR-005 submitted. Registered the unbounded Argon2 concurrency failure,
  shared memory-hard work rule, and opaque revocable-session architecture
  decision.
- Archive: TICKET-005 closed; its active spec/notes pair moved to completed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
