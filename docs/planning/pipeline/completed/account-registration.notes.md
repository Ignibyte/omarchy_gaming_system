---
title: Account registration — notes
pipeline_id: 1dfcb0d0-9a29-4774-86aa-b93e82fd9d11
---

# Account registration — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: run the first three unfinished roadmap identity outcomes
  through the newly enforced pipeline, one active ticket at a time.
- Recall: server-side modules own identity invariants; handlers stay thin;
  accounts remain separate from public personas; migrations are forward-only;
  database claims need live PostgreSQL evidence.
- Knowledge: retained worktree-bound gates, untracked-file coverage, and the
  rule that CodeGraph coverage hints supplement direct test inspection.
- Bulletin: the repository still has no remote `main`; all work and validation
  remain local, and no delivery action is authorized.
- Upstream grounding: OWASP's current baseline is Argon2id with 19 MiB memory,
  two iterations, and parallelism one. RustCrypto emits parameterized PHC
  strings, and SQLx 0.9 can create isolated PostgreSQL test databases and apply
  migrations automatically.

## Phase 2 — Design

- Architecture: keep Axum transport translation in `app.rs` and add an
  `accounts` domain module that owns canonicalization, validation, hashing, and
  insertion. The existing `PgPool` remains the shared state, startup remains
  the sole migration runner, and the new route returns transport DTOs rather
  than persistence rows.
- Security/privacy: registration never logs or returns the password/hash;
  password hashing uses a random salt and explicit Argon2id v19 parameters in
  `spawn_blocking`; database/cryptography failures collapse to a generic server
  error. Account identifiers and usernames are registration-only data here and
  do not create a public persona surface.
- CodeGraph evidence: explored `router`, `AppState`, `health`, `main`,
  `MIGRATOR`, and `connect_database` before edits. The graph found the full
  three-file startup/route blast radius and no pre-existing account callers. It
  flagged `AppState`, `MIGRATOR`, and database connection paths as lacking
  covering tests, so the regression plan adds router-level PostgreSQL tests
  and retains the independent live stack smoke.
- Unsupported surfaces: CodeGraph does not model the SQL and shell changes;
  migration constraints, gate wiring, and curl assertions were inspected
  directly and remain subject to executed checks.

### File manifest

| Path | Purpose |
|---|---|
| `Cargo.toml`, `crates/server/Cargo.toml`, `Cargo.lock` | Add RustCrypto Argon2 plus router-test utilities. |
| `crates/server/src/accounts.rs` | Own username/password validation, Argon2id hashing, conflict classification, and account insertion. |
| `crates/server/src/app.rs` | Add the thin `POST /v1/accounts` handler, stable JSON DTOs/errors, and router/PostgreSQL integration tests. |
| `crates/server/src/main.rs` | Register the account module. |
| `migrations/0002_canonical_account_usernames.sql` | Enforce the canonical ASCII account namespace for every database writer. |
| `scripts/test-database.sh`, `bin/gate.sh` | Run ignored isolated SQLx tests against the Compose PostgreSQL service in non-fast gates. |
| `scripts/dev.sh` | Exercise successful and duplicate registration through the live API smoke path. |
| `docs/api.md`, `README.md` | Document the endpoint, request rules, responses, and local example. |
| Ticket/spec/notes/AAR and OpenWiki pages | Preserve pipeline and durable knowledge evidence. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Router test asserts `201`, exact public JSON fields, active row, and canonical username; live curl smoke repeats the success path. |
| REQ-002 | Unit tests inspect/verify PHC hashes and their parameters; PostgreSQL test proves plaintext is absent and the stored hash verifies. |
| REQ-003 | Unit validation matrix plus router test asserts `422` codes and zero inserted rows. |
| REQ-004 | Router test submits a canonical duplicate, asserts `409`, one row, and unchanged stored hash. |
| REQ-005 | Non-fast gate starts Compose PostgreSQL, runs isolated migrated SQLx tests sequentially, then runs the existing Rust/QML smoke with registration assertions. |

## Phase 3 — Implement

- Built: the account domain boundary, canonical username/password validation,
  OS-random salted Argon2id hashing on `spawn_blocking`, conflict-aware
  PostgreSQL insertion, a thin versioned Axum handler, stable JSON errors, a
  forward canonical-username constraint, unit tests, ignored isolated SQLx
  tests, non-fast database gate wiring, live success/duplicate smoke checks,
  and API/operator documentation.
- Focused checks: `cargo build --workspace`, `cargo clippy --workspace
  --all-targets -- -D warnings`, six unit tests, two ignored tests against
  isolated migrated PostgreSQL databases, `scripts/dev.sh --smoke-test`, Bash
  syntax, whitespace, and pipeline structure all passed. The QML smoke retained
  its existing non-fatal EGL warnings.
- Deviations: RustCrypto's OS RNG needed an explicit `rand_core/getrandom`
  feature. The first correction placed that crate under dev-dependencies, which
  made tests pass while the production binary failed; the live smoke exposed
  the mistake, and the dependency now correctly belongs to runtime.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Secret exposure | Password-bearing transport and domain inputs derived `Debug`, making a future broad debug log capable of exposing raw credentials. | high | Removed both `Debug` implementations; the fresh graph pass confirms only safe response/error types remain format-capable. |
| 2 | Resource bounds | Axum's default JSON body limit was much larger than this 160-byte logical input contract. | medium | Applied a route-local 1 KiB body cap and added a fast test proving oversized requests return `413` before database work. |
| 3 | Async safety | Argon2id is deliberately expensive and would starve request workers if run inline. | high | Confirmed the only handler-to-hasher call path crosses `spawn_blocking`; the configured work is bounded by the 128-byte password limit. |
| 4 | Persistence/race behavior | Canonical prechecks alone cannot safely enforce username uniqueness under concurrent requests. | high | Keep the case-insensitive unique database index as authority and map only its named `23505` conflict to `409`; isolated PostgreSQL tests cover canonical duplicates and unchanged hashes. |
| 5 | Coverage interpretation | CodeGraph found both Rust test modules in its source result but again reported the registration symbols as uncovered. | low | Applied `PR-omarchy-bbs-graph-coverage-is-advisory-001`; direct unit, router/PostgreSQL, and live smoke results remain the coverage authority. |

- Final CodeGraph pass: re-explored the complete handler → domain → blocking
  hash → insert path after the two fixes. The inspection receipt matches the
  current implementation worktree.

## Phase 4 — Validate

- Tests run: seven fast tests passed (validation, parameterized/salted hash
  verification, health/config regression, and the 1 KiB transport limit); two
  explicitly ignored router tests passed against isolated PostgreSQL databases
  with repository migrations.
- Gate run: `bin/gate.sh --diff` passed all 12 checks: rustfmt, Clippy, fast
  tests, rustdoc, Compose validation, shell syntax, pipeline structure, changed
  secret scan, Codex hook self-tests, whitespace, PostgreSQL integration tests,
  and the live PostgreSQL/Rust registration/QML smoke. It printed `GATE GREEN
  [diff]` and wrote a matching worktree receipt.
- Skips or pre-existing failures: no validation skips. The headless QML run
  emitted the existing non-fatal EGL warnings and passed.

## Phase 5 — Complete

- Acceptance-criteria audit:

| Requirement | Verdict | Evidence |
|---|---|---|
| REQ-001 | satisfied | The router/PostgreSQL test and live smoke both received `201`, an ID and canonical username only, and one active account row. |
| REQ-002 | satisfied | Unit and PostgreSQL tests verified unique salts, Argon2id v19 PHC encoding with `m=19456,t=2,p=1`, password verification, and absence of plaintext storage. |
| REQ-003 | satisfied | The router test returned stable `invalid_username` and `invalid_password` 422 errors with zero inserted rows; the fast suite also covers boundary values and the 1 KiB body cap. |
| REQ-004 | satisfied | A canonical duplicate returned `username_taken` with HTTP 409 while the original row count and password hash remained unchanged; the live smoke repeated the conflict path. |
| REQ-005 | satisfied | The final `bin/gate.sh --diff` passed all 12 checks, including isolated migrated PostgreSQL tests and the live PostgreSQL/Rust registration/QML smoke. |

- Docs: added `docs/api.md`; updated README, system overview, roadmap, and four
  OpenWiki pages. OpenWiki update runs
  `a587b08b-d8a9-40ed-adce-2397f0f6a800` and
  `25ce287d-2351-49f9-bd81-a1fe8b2b27a5` completed; the second reconciled the
  final roadmap evidence and produced the matching completion receipt.
- AAR: AAR-004 submitted. Registered the masked runtime-dependency failure, a
  non-test build prevention rule, and the account-registration boundary
  decision in the knowledge index.
- Archive: TICKET-004 closed; its active spec/notes pair moved to completed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first test compile could not import `OsRng`. | `rand_core` gates its OS RNG behind `getrandom`. | Enable the feature explicitly. | Treat cryptographic entropy features as runtime requirements, not implicit transitive defaults. |
| 2 | Tests compiled while the production server did not. | The RNG dependency was first placed in `[dev-dependencies]`, and `cargo test` made it visible to the binary test target. | Move `rand_core` to `[dependencies]` and run a plain production binary build. | Keep a non-test `cargo build` in focused checks whenever runtime dependency boundaries change. |
