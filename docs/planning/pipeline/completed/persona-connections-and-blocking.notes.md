---
title: Persona connections and blocking — notes
pipeline_id: fd2023e5-d943-4466-9320-28bcfdd97358
---

# Persona connections and blocking — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: work through the next five roadmap tickets. The first
  ordered slice is requests, acceptance, removal, and blocking; tickets remain
  sequential because the constitution permits only one active spec/notes pair.
- Recall: account identity remains private, personas are the public social and
  game identity, and every owned mutation derives its account principal from a
  validated device session. Public persona DTOs already structurally exclude
  `account_id` and authentication material.
- Recalled rules: `PR-omarchy-bbs-owner-scope-account-resources-001` requires
  owner-and-object predicates; `PR-omarchy-bbs-graph-coverage-is-advisory-001`
  requires direct test inspection and executed evidence; the vertical-slice
  rule requires the migration, HTTP API, and live client path together.
- Nearest pipeline: TICKET-006 established multi-persona ownership, exact safe
  profile responses, indistinguishable foreign/absent errors, and adversarial
  multi-account PostgreSQL tests. TICKET-008 left connections/inbox as the
  next roadmap area and preserved the same authenticated principal boundary.
- Preflight: no active pipeline existed, the next number was 009, BUL-001 is a
  non-blocking warning that the remote still has no confirmed `main`, and
  `scripts/check-pipeline-tools.sh` reported CodeGraph 1.5.0 and OpenWiki 0.3.3
  ready with verified Codex-only provenance.
- Ordered follow-ons derived from the authoritative roadmap are TICKET-010
  inbox conversations/messages, TICKET-011 cursor sync/WebSockets, TICKET-012
  game registry/versioned sessions, and TICKET-013 idempotent revision-checked
  game commands. Only TICKET-009 is opened now.

## Phase 2 — Design

- Architecture: add a `connections` domain beside `personas`. Axum extracts
  the Bearer token and two path UUID strings, but the domain authenticates the
  account, verifies ownership of the acting persona, owns pair-state policy,
  performs all SQL, and returns explicit social models containing the existing
  public persona shape. No account ID enters a social response.
- Command API: `PUT
  /v1/personas/{persona_id}/connection-requests/{other_persona_id}` creates or
  idempotently returns an outgoing pending request; `PUT
  /v1/personas/{persona_id}/connections/{other_persona_id}` accepts an incoming
  request or idempotently returns an existing accepted connection; `DELETE`
  on that connection path cancels pending state or removes accepted state.
  `PUT`/`DELETE /v1/personas/{persona_id}/blocks/{other_persona_id}` block and
  unblock. New idempotent `PUT` resources return 201 and existing ones return
  200; acceptance returns 200; deletes return 204.
- Query API: `GET /connection-requests` returns separately ordered `incoming`
  and `outgoing` arrays; `GET /connections` returns mutual accepted entries;
  `GET /blocks` returns only the actor's directional blocks. Each entry embeds
  the seven-field public persona response plus `created_at` or `connected_at`.
  All query and command responses are `Cache-Control: no-store`.
- Persistence: migration `0006` adds one `persona_connections` row per
  canonical UUID-ordered pair. It records requester/addressee, `pending` or
  `accepted`, creation/update time, and a non-null acceptance time only for an
  accepted row. Check constraints prove both directional IDs are exactly the
  canonical pair. `persona_blocks` has one directional `(blocker, blocked)`
  row. Foreign keys cascade with future persona deletion and indexes support
  both sides of inventories and block checks.
- Concurrency: every pair mutation begins a transaction, locks both extant
  persona rows in ascending UUID order, then verifies actor ownership, target
  existence/different-account policy, and blocks. Opposite requests therefore
  create one pending row; concurrent acceptance/removal/blocking has a linear
  order; and block insertion plus relationship deletion is atomic. Unblock
  never recreates an old row. Reads rely on committed database state.
- Errors and privacy: invalid/foreign acting personas share
  `persona_not_found`; missing, malformed, same-account, or either-direction
  blocked targets on state creation share `connection_unavailable`. An
  outgoing pending request cannot self-accept and shares
  `connection_request_not_found` with absent state. Existing accepted state is
  `connection_already_exists`; a reverse pending request is
  `connection_request_pending`. Deletes authenticate and owner-check the actor
  but deliberately return 204 for missing/malformed target state.
- CodeGraph evidence: the design exploration covered `router`, the shared
  `bearer_token` boundary, `PersonaResponse`/`persona_response`, the persona
  handlers, and the multi-account persona tests. It found the router and
  response mapper as the direct transport blast radius; graph test association
  remained advisory, so the new domain receives its own executed router and
  PostgreSQL suite. The design receipt for pipeline
  `fd2023e5-d943-4466-9320-28bcfdd97358` was written against this gated
  worktree.
- Unsupported surfaces inspected directly: all migrations, `scripts/dev.sh`,
  `scripts/test-database.sh`, API/product/architecture docs, module wiring, and
  the complete relevant session/persona SQL and tests.

### File manifest

| Path | Purpose |
|---|---|
| `migrations/0006_persona_connections_and_blocks.sql` | Add canonical one-row relationship pairs, directional blocks, checks, foreign keys, and query indexes. |
| `crates/server/src/connections.rs` | Own authentication, owner/target policy, ordered pair locking, request/accept/remove/block transactions, inventories, stable errors, and safe social models. |
| `crates/server/src/app.rs` | Add thin social routes, explicit response DTOs, status/error mapping, and no-store handling while reusing the public persona mapper. |
| `crates/server/src/connection_api_tests.rs`, `main.rs` | Register and execute multi-account, privacy, idempotency, and race tests against migrated PostgreSQL. |
| `scripts/dev.sh` | Extend the real live slice with a second account/persona and request, accept, remove, block, failed request, unblock, and re-request flow before session revocation/QML. |
| `docs/api.md`, `README.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md` | Document the public contract, social invariants, implemented roadmap outcome, and remaining inbox/sync work. |
| Ticket/spec/notes/AAR/knowledge/OpenWiki | Preserve the evidence ledger, decisions, generated documentation, and completion record. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Multi-account router/PostgreSQL test proves 201 then idempotent 200, one canonical pending row, safe target profile, no-store, and rejection of duplicates/opposite request ambiguity. |
| REQ-002 | Inventory test creates multiple personas/accounts and proves incoming/outgoing direction, stable timestamp/UUID order, actor ownership, exact fields, and absence of foreign rows. |
| REQ-003 | Acceptance and concurrent opposite-request tests prove only the addressee can transition one row, both sides see one mutual connection, retry is safe, and requester/foreign/absent attempts do not mutate. |
| REQ-004 | Lifecycle test proves either participant can remove accepted state, either can cancel/decline pending state, retries stay 204, and no unrelated pair changes; live smoke covers accepted removal. |
| REQ-005 | Block lifecycle plus concurrent request/block test proves private directional inventory, atomic relationship removal, generic two-way request rejection, idempotent block/unblock, no restoration, and the final invariant of block-without-relationship. |
| REQ-006 | Adversarial test covers invalid/foreign actor, invalid/absent/same-account target, authentication precedence, response-key allowlists, non-disclosing bodies, and unchanged database counts. |
| REQ-007 | All new ignored SQLx tests run in the PostgreSQL tier, and the expanded server/QML live path runs inside a final `bin/gate.sh --diff`. |

## Phase 3 — Implement

- Built: forward migration `0006` with canonical pair and directional block
  constraints/indexes; a connection domain owning ordered row locks,
  owner/target checks, idempotent request/accept/remove/block/unblock commands,
  private ordered inventories, explicit public-profile models, and stable
  errors; six thin Axum route groups; four isolated multi-account and race
  tests; the live two-account social lifecycle; and API, README, roadmap, and
  system-overview documentation.
- Focused checks: `cargo check --workspace --all-targets`, warning-denied
  Clippy, Bash syntax, and `cargo test --workspace` passed with 23 fast tests
  and 16 database tests intentionally ignored in that tier. Then
  `scripts/test-database.sh` applied every migration and passed all 16 tests,
  including all four new connection cases. `scripts/dev.sh --smoke-test`
  passed request, incoming inventory, acceptance, mutual inventory, removal,
  block, private blocked rejection, block inventory, unblock, re-request,
  cancellation, existing account/session/persona/MFA behavior, and headless
  QML. The QML run emitted only the known non-fatal EGL `dri2` warnings.
- Deviations: the same EARS privacy intent was clarified during design so
  authenticated idempotent `DELETE` commands deliberately return 204 for
  malformed or absent target state, while every state-creating command retains
  a non-disclosing failure. No implementation file departed from the approved
  manifest.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | EARS/correctness | Request, inventory, acceptance, removal, block, and unblock behavior matches REQ-001 through REQ-006, including the documented idempotent-delete exception. | None | Pass; direct router/PostgreSQL assertions cover every state transition. |
| 2 | Authentication/authorization | Every social entry point parses a Bearer token in transport code, authenticates again in the domain, derives the account principal from the session, and owner-checks the acting persona before any pair mutation or private inventory. | None | Pass; foreign, absent, malformed, and unauthenticated actor cases are indistinguishable where required. |
| 3 | Privacy/API contract | Social DTOs reuse the seven-field public persona model; block inventories are directional and private; state-creating target failures do not disclose target existence, same-account ownership, or block direction. | None | Pass; exact-key and forbidden-field assertions plus `no-store` checks passed. |
| 4 | Concurrency/data integrity | Pair mutations lock both persona rows in canonical UUID order before relationship or block reads/writes; block insertion and relationship deletion share one transaction; schema checks bind direction IDs to the canonical pair. | None | Pass; opposite-request, concurrent-acceptance, and request-versus-block database tests passed. |
| 5 | Failure/operability | Database failures are logged without returning internals, mutations are parameterized, inventory ordering is deterministic, and the live development flow exercises the complete two-account lifecycle. | None | Pass; Clippy, 16 migrated PostgreSQL tests, and the headless live smoke were green. |
| 6 | Security diff scan | Frozen 56-item review found one CodeGraph provenance candidate; validation rejected it because exact-version npm delivery, registry integrity metadata, and threat-model constraints did not establish a realistic lower-privileged substitution path. | None | Sealed scan `3f34054d-26ed-4ba2-b06b-4e37da55bc2c` completed with zero reportable findings and complete coverage. |
| 7 | Structural inspection | CodeGraph traced the Axum-to-domain request path, session boundary, canonical pair locking, and direct handler blast radius. It did not associate the standalone SQLx test module, so graph coverage remains advisory. | None | Pass; the complete test module was inspected directly and all four cases executed in PostgreSQL. |

- Inspection conclusion: no correctness, security, privacy, concurrency, or
  simplification defect remains. A fresh post-implementation CodeGraph receipt
  is required after this ledger/status update so it binds the final inspected
  worktree.

## Phase 4 — Validate

- Tests run: the canonical diff gate passed 23 fast Rust tests and then all 16
  ignored SQLx cases against isolated migrated PostgreSQL databases. The four
  connection cases covered request direction/idempotency/privacy, participant
  acceptance/removal, private atomic blocks and request races, and serialized
  opposite requests/concurrent acceptance.
- Gate run: `bin/gate.sh --diff` passed all 12 stages: rustfmt, warning-denied
  Clippy, fast tests, rustdoc, Compose validation, shell syntax, pipeline
  structure, changed-file secret scan, Codex hook self-tests, whitespace,
  PostgreSQL integration tests, and the PostgreSQL/Rust/QML live smoke. It
  wrote a receipt matching state hash
  `11093958840af8bc02f07c139e348911a850d70c873dbcbcf9a875c00811e68d`.
- Skips or pre-existing failures: none. The fast tier intentionally reported
  the 16 database tests as ignored before the non-fast tier executed all 16.
  QML emitted the known non-fatal EGL `dri2` warnings during the successful
  offscreen smoke run.

## Phase 5 — Complete

- Acceptance-criteria audit: REQ-001 and REQ-002 passed through the directional,
  idempotent, owner-scoped request/inventory test and live request/incoming
  flow; REQ-003 passed addressee-only, mutual-inventory, opposite-request, and
  concurrent-acceptance evidence; REQ-004 passed participant removal, pending
  cancellation, and idempotent-delete evidence; REQ-005 passed private block
  inventory, atomic removal, two-way non-disclosing rejection, unblock, and
  request-versus-block evidence; REQ-006 passed adversarial actor/target and
  exact-field assertions; REQ-007 passed all 16 migrated PostgreSQL tests and
  the live social path inside `bin/gate.sh --diff`. No requirement was dropped.
- Docs: the hand-maintained API, README, roadmap, and system overview were
  reconciled during implementation. OpenWiki update run
  `02433a12-e6fb-4a78-9dd9-efe3306d8a8c` updated the quickstart, runtime,
  product-boundary, and development/validation pages and returned
  `status: complete`. Its receipt matches gated state
  `926f3408135d7cbaceee484d279a4b02a0ddf8489f719dca3df45bce81884caf`.
- AAR: `AAR-009-persona-connections-and-blocking` submitted at effectiveness
  5/5. It added
  `PR-omarchy-gaming-system-lock-social-pairs-before-state-001` and
  `AD-omarchy-gaming-system-persona-social-pair-model-001`; both IDs were
  appended to the knowledge register.
- Archive: ticket, spec, and notes moved to their closed/completed stores with
  Phase 5 PASS. Delivery remains intentionally separate and unauthorized: no
  commit, push, or pull request was performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | An early `cargo fmt --all -- --check` could not resolve the newly declared `connection_api_tests` module. | `main.rs` was wired before the new test file had been added to the worktree. | Added the test module, reran formatting, and completed every focused check successfully. | Add a new module file before declaring it, or apply both changes in the same patch before invoking Cargo. |
