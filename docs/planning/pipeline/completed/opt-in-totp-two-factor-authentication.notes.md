---
title: Opt-in TOTP two-factor authentication — notes
pipeline_id: b5b83a39-3fca-4351-a192-509b5b9ffa20
---

# Opt-in TOTP two-factor authentication — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: continue getting the product started, add opt-in 2FA, and use
  “OmarchyGS” as the human shorthand.
- Current state: foundation, account registration, revocable sessions, personas,
  and the game-first product rebrand are complete; no active pipeline existed.
  Connections/inbox is next on the roadmap, but the explicitly requested
  account-protection gap belongs before those higher-impact account surfaces.
- Recall: preserve account/persona separation; derive private ownership from
  a validated session; keep handlers thin; bound all memory-hard password work;
  keep primary login failures generic; prove migrations/API/QML together; and
  inspect untracked files in all delivery evidence.
- RFC 6238 fixes the interoperable TOTP profile and recommends a 30-second step,
  at most one step of transmission delay, secure verifier-side key storage, and
  rejection of a second use in the same validity window.
- NIST SP 800-63B-4 requires verifier-side OTP secrets to be strongly protected,
  OTP collection over a protected channel, one-time acceptance, defined TOTP
  lifetime, and account-level failed-attempt rate limiting. It also notes that
  manually entered OTP is not phishing-resistant; WebAuthn remains a later,
  stronger option rather than being mislabeled as part of this TOTP slice.
- RFC 4226 requires throttling to span login sessions so multiple challenges
  cannot reset guessing limits.
- Preflight: CodeGraph 1.5.0 and OpenWiki 0.3.3 are ready. PostgreSQL is healthy,
  though its running Compose container retains the historical project label.
  The warning bulletin still records that remote `main` is unconfirmed.

## Phase 2 — Design

- Architecture: `mfa.rs` owns TOTP/recovery cryptography, encrypted-at-rest
  authenticator state, enrollment/status/disablement, short-lived login
  challenges, account-wide throttling, and factor consumption. `sessions.rs`
  continues to own primary credential exchange and opaque device-session
  issuance, but returns either a created session or an MFA challenge. Challenge
  completion locks the challenge, account, and authenticator, consumes the
  factor and challenge, and inserts the device session in one PostgreSQL
  transaction. `app.rs` only translates these domain outcomes to versioned
  JSON/status/header contracts.
- Enrollment flow: authenticate the Bearer session, verify the password through
  the existing shared/bounded Argon2 path, generate a random 160-bit secret and
  96-bit AES-GCM nonce, bind the ciphertext to the account UUID as associated
  data, and upsert only an unconfirmed enrollment. Confirmation locks the row,
  rejects enrollment older than ten minutes, verifies and consumes the TOTP
  step, inserts ten recovery-code digests, and marks the row enabled.
- Login flow: validate device name and primary credentials exactly as today,
  then lock the active account. If no enabled authenticator exists, insert the
  existing 30-day device session. If MFA is enabled, delete only expired or
  consumed challenges, count the remaining live set under the account lock,
  and either insert an independent five-minute `ogm1_` challenge or return 429
  at the ten-challenge cap. Completion validates and consumes only the selected
  challenge and factor under row locks before inserting one session.
- Factor rules: decode exactly six ASCII TOTP digits and compare RFC 6238
  HMAC-SHA-1 values at current, previous, and next 30-second steps. Reject a
  matched step less than or equal to `last_used_step`. Recovery codes contain
  120 random bits, use a grouped `OGS-...` presentation, and persist only a
  SHA-256 digest. Successful use resets failures; invalid use increments the
  authenticator-wide counter, invalidates a challenge at five failures, and
  applies a five-minute account lock at five cross-challenge failures.
- Configuration/operations: server startup requires a base64url-encoded 32-byte
  `OGS_MFA_ENCRYPTION_KEY`. Normal development creates one durable ignored key
  under `.dev/` with mode 0600 when the variable is absent. Operators must back
  up and consistently supply the key; loss makes enrolled TOTP secrets
  unrecoverable. TLS and distributed edge throttling remain required before
  public exposure.
- Concurrency and privacy: account, challenge, and authenticator rows are
  locked during security-state transitions; successful factor use and session
  creation share a transaction. Responses never include account IDs,
  ciphertext, nonces, digests, failure counters, or lock timestamps. Pending
  enrollment never affects login. Disabling requires the validated Bearer,
  password, and an unused factor and atomically removes the authenticator,
  recovery codes, and outstanding challenges.
- Compatibility: password-only accounts retain the exact `POST /v1/sessions`
  201 response. Enabled accounts receive the new 202 challenge branch. Existing
  Bearer tokens and legacy `bbs1_` compatibility remain unchanged. “OmarchyGS”
  is display shorthand only; `ogs` runtime namespaces do not change.
- CodeGraph evidence: the pre-edit graph traced `app::create_session` to the
  sole `sessions::create_session` credential/token path, found all router/test
  callers, and showed that `sessions::authenticate` is the stable principal
  boundary consumed by session inventory/revocation and personas. The router
  constructor has production plus both PostgreSQL test callers, so key injection
  must update each. Direct review covered migrations, shell smoke, manifests,
  environment examples, and API docs because those formats are outside the
  structural graph.

### File manifest

| Path | Purpose |
|---|---|
| `Cargo.toml`, `crates/server/Cargo.toml`, `Cargo.lock` | Add AES-GCM, HMAC-SHA-1, base32, percent-encoding, and zeroization dependencies. |
| `crates/server/src/mfa.rs` | Own key parsing/encryption, RFC 6238, recovery codes, enrollment, status, disablement, challenge creation/verification, throttling, and transactional factor consumption. |
| `crates/server/src/sessions.rs` | Split primary verification from session insertion, return created/challenge outcomes, and issue sessions inside MFA completion transactions. |
| `crates/server/src/app.rs` | Add MFA enrollment/status/confirm/disable and challenge-completion routes plus stable no-store response/error contracts. |
| `crates/server/src/config.rs`, `main.rs` | Require, validate, and inject the MFA encryption key; register MFA modules/tests. |
| `crates/server/src/mfa_api_tests.rs` | Exercise enrollment, encrypted persistence, confirmation/recovery, gated login, replay/expiry/throttling, disablement, and multi-account isolation against PostgreSQL. |
| `migrations/0005_totp_two_factor_authentication.sql` | Add encrypted authenticator state, recovery-code digests, and digest-only login challenges with constraints/indexes. |
| `scripts/dev.sh`, `.env.example` | Persist a private local development key, extend smoke through MFA, and document production key supply. |
| `docs/api.md`, `README.md`, `docs/product-charter.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md` | Document OmarchyGS shorthand, MFA contracts/limits, operations, and completed identity protection. |
| OpenWiki and TICKET-008 pipeline/AAR/knowledge records | Reconcile durable engineering knowledge and completion evidence. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Unit tests prove 160-bit randomness, encryption/decryption/AAD failure, and key parsing; PostgreSQL/router test proves password/session requirements, pending state, encrypted-only storage, URI contract, and no-store. |
| REQ-002 | PostgreSQL/router test proves invalid/replayed confirmation cannot enable, valid confirmation returns ten unique high-entropy codes once, stores only 32-byte digests, and enables status. |
| REQ-003 | Multi-account test compares the unchanged 201 password-only result with the enabled account's 202/no-store challenge and proves no pre-MFA session row; existing generic primary failures remain byte-identical. |
| REQ-004 | PostgreSQL transaction tests prove valid TOTP and recovery completion, one-time challenge/factor use, expiry, attempt exhaustion, inactive-account rejection, and exactly one inserted session under concurrent replay. |
| REQ-005 | RFC 6238 SHA-1 test vectors and deterministic step-window tests prove formatting, drift, and replay decisions; PostgreSQL tests prove failures span newly issued challenges and lock then recover after expiry. |
| REQ-006 | Router tests prove status secrecy/scoping and require Bearer + correct password + unused factor for disablement; successful disable removes MFA rows/challenges and restores ordinary login. |
| REQ-007 | Non-fast database tier runs every ignored MFA test and live smoke performs enrollment, recovery-code MFA login, replay rejection, disablement, ordinary relogin, and QML health before the diff receipt. |

## Phase 3 — Implement

- Built: AES-256-GCM account-bound TOTP secret encryption; required base64url
  key configuration and durable mode-0600 local development key; pending
  password-reverified enrollment; RFC 6238 confirmation; ten digest-only
  120-bit recovery codes; MFA status; five-minute digest-only login challenges;
  unchanged password-only session creation; transactional TOTP/recovery,
  challenge, and session consumption; current/past/future step validation with
  replay state; per-challenge and authenticator-wide attempt limits; secure
  disablement; forward-only schema constraints/indexes; unit and PostgreSQL
  tests; live MFA/QML smoke; API/operator/architecture/product docs; and the
  OmarchyGS human shorthand.
- Focused checks: `cargo check --workspace --all-targets` passed after the first
  compile repair; `cargo test --workspace` passed 22 fast tests with 11 database
  tests intentionally ignored; `scripts/test-database.sh` passed all 11 migrated
  PostgreSQL tests after one test-fixture correction; `cargo build --workspace
  --bins` passed the production target; and `./scripts/dev.sh --smoke-test`
  completed registration, sessions, personas, TOTP enrollment, recovery-code
  login, recovery replay rejection, MFA disablement, restored ordinary login,
  revocation, and headless QML.
- Deviations: the first database run correctly rejected a test-only timestamp
  mutation that placed challenge expiry before creation. The fixture now ages
  both timestamps while preserving the migration invariant; product code and
  schema were unchanged. The crates.io resolver selected the documented current
  AES-GCM/HMAC/SHA-1 dependency line under Rust 1.98. No scope or API contract
  changed.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Account identity privacy | Anonymous registration returns `409 username_taken` for an existing private account username and `201` for an available one. A live HTTP reproduction and the PostgreSQL router test confirmed the one-bit oracle. | Low security | Risk accepted for this slice by the user; a verifiable private registration channel remains future work. |
| 2 | MFA availability and retry state | Every correct-password login consumed all live MFA challenges before inserting a replacement, but issuance did not consume the authenticator failure budget. | Low security | Fixed and verified: ten independent live challenges are bounded without invalidation; the eleventh returns 429 and completing one replenishes the budget. |
| 3 | Pipeline dependency provenance | OpenWiki declared `pnpm@10.33.2` and tracked `pnpm-lock.yaml`, while setup used `npm install`, created an ignored 710-package npm graph, and executed its compiler/runtime without binding the install to the reviewed lock. | Low security | Fixed and verified: SHA-512-verified pnpm bootstrap, frozen lock, scripts disabled, provenance-bound build, and fail-closed MCP startup. |
| 4 | Codex gated-path classification | `normalize_path` preserved internal `..` components, so an accepted path such as `docs/../crates/...` could be classified as documentation while the filesystem mutated a gated source path. | Workflow defect | Hardened and verified against relative traversal, absolute aliases, internal symlinks, unresolved/outside paths, and a legitimate docs edit. |
| 5 | Codex commit-gate parsing | An unrelated `--help`, `-h`, or `--dry-run` token anywhere in a compound Bash command exempted a real `git commit` subcommand from the active-pipeline and receipt checks. | Workflow defect | Hardened and verified: exact standalone non-mutating forms pass; semicolon and newline compound forms remain blocked. |
| 6 | Shell mutation coverage | The phase hook is not attached to arbitrary Bash mutations. No untrusted product input reaches Codex shell tools, hooks are cooperative guardrails, and the Codex editing contract requires `apply_patch`. | Not applicable | Closed as not applicable under the repository threat model; no hostile-filesystem boundary is claimed. |
| 7 | MFA confidentiality, replay, authorization, and transaction boundaries | The scan found no surviving plaintext-secret storage, challenge/session token persistence, TOTP/recovery replay, cross-account authorization, SQL injection, or factor/session transaction finding. | None | Closed by source review, PostgreSQL tests, and live smoke evidence. |

- Codex Security scan `5a1216ff-fa08-4c8b-9cd2-fd0b265bd5fa` reviewed all
  53 compact-diff source items against frozen snapshot
  `codex-security-snapshot/v1:sha256:2d27f78d750a2af31dc13043c427e38dd49e1e2533852bbcffeea154b755b2d2`.
  The sealed report contains three high-confidence, low-severity findings and
  no deferred coverage.
- The external advisory connector was unavailable, so the scan did not claim
  current third-party advisory coverage. This limitation does not change the
  locally proven OpenWiki lockfile mismatch.
- Phase 3.5 remains FAIL. Per the Codex Security remediation gate, no finding
  was changed after the frozen scan without an explicit user decision.
- User approved the recommended disposition: fix MFA challenge churn and
  OpenWiki dependency provenance; harden the two confirmed Codex hook defects;
  and temporarily risk-accept private username enumeration until registration
  has a verifiable out-of-band identifier channel.
- Remediation returned to Phase 2 Design before code edits. The locked design
  uses a ten-entry independent live-challenge budget under the existing account
  row lock, a SHA-512-verified pnpm bootstrap plus frozen upstream lock and
  ignored build provenance, lexical/real path canonicalization for hook inputs,
  and exact standalone matching for non-mutating commit exceptions.
- Remediation implementation: challenge issuance now deletes only expired or
  consumed rows, counts the independent live set, and returns the existing 429
  `mfa_rate_limited` contract at ten; `SessionError` carries that expected
  branch without converting it to an internal error. The regression creates
  ten simultaneous challenges, rejects the eleventh, completes the first,
  replenishes the budget, then completes the last original challenge.
- Pipeline-tool implementation: setup downloaded exact `pnpm@10.33.2`, matched
  its tarball to the SHA-512 integrity embedded at the pinned OpenWiki commit,
  installed 650 packages from `pnpm-lock.yaml` with `--frozen-lockfile` and
  scripts disabled, built OpenWiki, and recorded lock/patch/pnpm-tree/dist
  digests. The checker and MCP wrapper both failed when the provenance receipt
  was temporarily removed and passed after it was restored.
- Hook implementation: `realpath -m` canonicalization and worktree containment
  cover internal `..`, absolute aliases, internal symlinks, and unresolved or
  outside paths. Commit exceptions now match only exact standalone `git commit
  --help`, `git commit -h`, or `git commit --dry-run` commands; three compound
  variants remain blocked. The adversarial self-test and ordinary docs-edit
  control passed.
- Remediation checks: `cargo check --workspace --all-targets`, the 22-test fast
  suite, all 12 migrated PostgreSQL tests, shell syntax/ShellCheck, hook
  self-tests, OpenWiki setup/readiness/provenance, and `bin/gate.sh --fast`
  passed. The first fast gate exposed only inconsistent underscore grouping in
  pre-existing RFC vector literals; formatting those six decimal values made
  the complete fast gate green without changing their numeric values.
- Post-edit CodeGraph inspected `MAX_ACTIVE_CHALLENGES`,
  `create_challenge_if_enabled`, `SessionError::RateLimited`, the Axum error
  mapping, and `complete_login_challenge`. It found one issuance caller and one
  completion caller; direct SQL review confirmed only the selected challenge
  is consumed during completion and only explicit MFA disablement removes the
  account's remaining challenges. The matching inspection receipt is bound to
  the final gated worktree hash.

## Phase 4 — Validate

- Tests run: the canonical gate repeated Rust formatting, Clippy with warnings
  denied, 22 fast tests, Rustdoc, Compose validation, shell/pipeline/secret/hook
  checks, all 12 isolated migrated PostgreSQL tests, and the live PostgreSQL +
  Rust API + headless QML smoke.
- Gate run: `bin/gate.sh --diff` completed `GATE GREEN [diff]` after the final
  gated edit and wrote the matching worktree receipt.
- Skips or pre-existing failures: none. The 12 database tests are intentionally
  ignored only in the fast unit-test invocation and all ran in the database
  tier. The headless QML run emitted the known non-fatal EGL `dri2` warning and
  still passed.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 satisfied: unit and PostgreSQL evidence proved exact key parsing,
    account-bound AES-256-GCM storage, password/session-gated pending
    enrollment, Base32/URI output, ten-minute expiry, and no-store delivery.
  - REQ-002 satisfied: invalid confirmation left MFA disabled; valid RFC 6238
    confirmation returned ten unique recovery codes once, stored only their
    32-byte digests, and rejected repeated confirmation.
  - REQ-003 satisfied: password-only accounts retained `201`; enabled accounts
    returned `202` with no session; ten overlapping challenges remained usable,
    the eleventh returned 429, and consuming one replenished the budget without
    invalidating another live challenge.
  - REQ-004 satisfied: transaction and concurrent-replay tests proved only one
    valid challenge/factor completion creates a session, while malformed,
    expired, consumed, reused, locked, and inactive-account paths create none.
  - REQ-005 satisfied: RFC 6238 vectors, deterministic window tests, and
    PostgreSQL attempt tests proved six-digit HMAC-SHA-1, 30-second steps,
    one-step drift, step replay rejection, and five-attempt account-wide locks.
  - REQ-006 satisfied: status exposed only enabled state and recovery count;
    disablement required Bearer + password + unused factor, cleared all MFA
    rows/challenges, retained existing sessions, and restored password login.
  - REQ-007 satisfied: the final post-wiki `bin/gate.sh --diff` returned `GATE
    GREEN [diff]` with 22 fast tests, all 12 PostgreSQL tests, and live
    registration/session/persona/MFA/QML smoke at state
    `6ef93e06b0e0dc9f6501add66dbea4536396a26846a8a4a31ec7e0b93adc41c9`.
- Docs: OpenWiki update run `e619c235-6318-4f39-9fff-3f3d3ee6a35f`
  resolved all 14 stale/unresolved claims, added grounded MFA and enforcement
  claims, refreshed quickstart/runtime/development/workflow/product pages, and
  returned `status: complete`. README, API, product charter, roadmap, and
  system overview were reconciled before the generated lifecycle.
- AAR: submitted `AAR-008` at 5/5. Registered four confirmed failure IDs, four
  prevention rules, the opt-in TOTP architecture decision, and the explicitly
  accepted registration-enumeration decision in both the AAR and knowledge
  register.
- Archive: closed TICKET-008, removed it from the open queue, and archived this
  Phase 5 PASS spec/notes pair. No commit, push, pull request, or other delivery
  action was performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first PostgreSQL run had one failed expiry test setup. | The fixture moved `expires_at` before `created_at`, violating the intentional schema constraint before endpoint behavior ran. | Age both challenge timestamps while keeping expiry after creation and before database `now()`. | Treat schema constraints as part of the fixture contract; test expiry with a valid historical interval. |
| 2 | Repeated correct-password login invalidated another device's live MFA challenge. | Issuance treated all prior live challenges as replaceable account state even though password proof had not completed the second factor. | Keep up to ten independent live challenges and reject excess issuance with 429. | `PR-omarchy-gaming-system-preserve-independent-mfa-challenges-001`. |
| 3 | The pinned OpenWiki checkout executed dependencies from npm rather than its tracked pnpm lock. | Repository revision pinning was mistaken for transitive executable provenance. | Verify the exact pnpm tarball, use the frozen lock with scripts disabled, record install/build hashes, and fail MCP startup closed on drift. | `PR-omarchy-gaming-system-bind-generated-tools-to-lock-provenance-001`. |
| 4 | Lexical edit-path classification could miss a gated target named through traversal, absolute, or symlink aliases. | The hook classified the supplied string rather than the canonical filesystem target. | Canonicalize inside the worktree with `realpath -m` and fail outside/unresolved inputs closed. | `PR-omarchy-gaming-system-canonicalize-hook-paths-001`. |
| 5 | A help/dry-run token elsewhere in a compound command could exempt a real commit. | The exception regex searched the whole command for a harmless token without requiring the command itself to be harmless. | Match only three exact standalone non-mutating commit forms and adversarially test compound commands. | `PR-omarchy-gaming-system-exact-command-exemptions-001`. |
