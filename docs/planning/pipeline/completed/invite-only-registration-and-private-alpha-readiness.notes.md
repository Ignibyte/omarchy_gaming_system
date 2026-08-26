---
title: Invite-only registration and private-alpha readiness — notes
pipeline_id: 9453a1ce-c7c6-405b-bfa5-25972f28a0be
---

# Invite-only registration and private-alpha readiness — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: remote `main` and the clean local worktree began at Ticket 029 commit
  `296be36f5ca8d2cd2fbc8d92a59905895c8d10f6`; no active pipeline or bulletin
  blocked new work.
- Recall: `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0,
  OpenWiki 0.3.3, and Codex-only provenance ready. PostgreSQL is healthy in the
  local Compose project.
- Recall: the private-alpha roadmap has one unfinished outcome,
  “Invite-only external testing.” The application and packaged client already
  provide registration, password/MFA access, personas, connections, inbox,
  challenges, gameplay, reporting, suspension/audit, and recovery, but account
  registration itself is open to any network caller.
- Recall: Ticket 029 intentionally established a database-local operator
  authority with bounded JSON, mandatory audit, stable non-disclosing errors,
  and no network administrator token or role. Invitation management must reuse
  that boundary rather than create a second control plane.
- Recall: `AD-omarchy-gaming-system-registration-enumeration-risk-001` accepts
  explicit username conflicts at the current small-community boundary. Invite
  lifecycle remains more private: absent, malformed, expired, revoked, and
  changed-intent use must collapse to one response.
- Recall: QML credentials and factors live only in process memory and are
  synchronously cleared from fields. The invitation is also a bearer secret,
  so it follows the same masked field, admitted-origin request, no-persistence,
  and allowlisted-error boundary.
- Decision: Ticket 030 owns software readiness for the external alpha, not the
  external human event. The roadmap will split those facts rather than mark an
  unperformed two-installation test complete.
- Decision: the existing account route becomes invitation-required in place;
  no compatibility bypass or optional server flag will leave open registration
  active during private alpha.
- Decision: codes are cryptographically random, one-account, expiring secrets
  stored only as digests. An issuance replay can return durable metadata but
  cannot recover the raw code; the operator must revoke an invitation whose
  first output was lost and issue another.
- Decision: account creation and invitation consumption are atomic. A used code
  can recover the original public registration receipt only when canonical
  username and password prove the exact account, allowing safe manual retry
  after an uncertain response without making the code reusable.

## Phase 2 — Design

- Architecture and authority flow:
  1. `omarchygs-admin apply` accepts two new deny-unknown-fields command
     variants. `issue_registration_invite` requires a unique operation UUID,
     1–64 character label, 1–720 hour lifetime, bounded actor, and bounded
     reason. `revoke_registration_invite` requires the invite UUID plus the
     existing operation/actor/reason fields. The command remains a local
     process with PostgreSQL authority from `DATABASE_URL`; the Axum router,
     session model, and account roles gain no administrator route or token.
  2. Issuance takes one database-wide transaction advisory lock dedicated to
     invitation admission, resolves operation replay, enforces at most 500
     currently issued/unexpired invitations, generates 32 bytes from the OS
     CSPRNG, formats `ogsi_` plus canonical unpadded base64url, stores only its
     SHA-256 digest, appends an immutable operator event, and commits before
     returning the raw code. An exact operation replay validates all intent and
     returns the same metadata with `invite_code: null` and
     `first_delivery: false`; the secret is intentionally unrecoverable.
  3. `omarchygs-admin invites [issued|used|expired|revoked|all] [1..100]`
     returns newest-first invitation metadata and derived state. It includes
     the public account username only after use but excludes the raw code,
     digest, password/session material, and internal account UUID.
  4. Revocation locks the invitation row, resolves exact audit replay before
     current-state admission, permits only currently issued/unexpired →
     revoked, stores bounded revocation metadata, appends one audit event, and
     commits atomically. Used, expired, already-revoked, and absent rows remain
     unusable; competing revoke/use operations serialize on the invitation.
  5. `POST /v1/accounts` adds exact `invite_code`, retains the 1 KiB body cap,
     and applies `Cache-Control: no-store` to success and errors. The account
     domain validates username/password/code shape, hashes the canonical code,
     and cheaply reads its lifecycle before scheduling Argon2id work. Random
     absent/malformed/revoked/expired codes fail without password hashing.
  6. A currently usable invitation causes password hashing before a short
     transaction. The transaction locks the invitation, rechecks expiry/use/
     revocation, inserts the canonical account, links and timestamps one
     consumption, and commits. Username conflict rolls back every change and
     leaves the invitation usable. Two consumers cannot create two accounts.
  7. A code already linked to an account is not generally reusable. If the
     canonical submitted username equals that linked account and Argon2id
     verifies the submitted password, the domain returns the immutable public
     account receipt as an exact replay (`200 OK`); otherwise it returns the
     same `403 invalid_invitation` used for every unavailable lifecycle. A
     race discovered after the invitation lock exits the transaction before
     credential verification so expensive work never holds the row lock.
  8. `AccessScreen` shows a masked 48-character invitation field only in
     registration mode. Registration copies the code/password into the single
     admitted-origin JSON request and synchronously clears both fields;
     Escape, mode change, server change, and completion cannot retain them.
     `OnboardingController` accepts only exact `200`/`201` account receipts and
     maps `invalid_invitation` to fixed local plain text.
  9. `scripts/test-private-alpha.sh` owns a generated isolated database,
     applies the complete production migrations through server startup, drives
     the real local CLI, starts the real loopback API, and proves issue,
     first registration, exact replay, changed-intent denial, one-use denial,
     ordinary login, second-code revocation, metadata-only inventory, audit,
     and cleanup. Existing `scripts/dev.sh --smoke-test` issues distinct codes
     for the live QML and curl registration paths so the complete migrated
     QML flow exercises the production contract.
- Data model and migration:
  - `registration_invites` stores UUID ID; unique 32-byte code digest; bounded
    label; original validity hours; unique issue operation UUID; issue actor/
    reason; created/expiry timestamps; nullable, exact-pair consumption time
    and unique account FK; and nullable exact-set revocation time, actor,
    reason, and unique operation UUID. Checks deny simultaneous use/revocation,
    inconsistent nullable fields, invalid timestamp order, invalid lifetime,
    and overlong metadata. State indexes support live-cap and inventory paths.
  - Migration `0017` adds nullable `target_registration_invite_id` to
    `operator_audit_events`, replaces its exact-target constraint with an
    exact-one-of account/report/invitation constraint, admits only the matching
    target kind, and adds a target/operation uniqueness index. The existing
    insert-only audit trigger remains authoritative.
  - Existing accounts remain valid with no invitation backfill. Only new
    registration links one invitation to one account; identity, MFA, persona,
    social, inbox, game, provider, and report rows do not change shape.
- Shared code contract:
  - New `registration_invites.rs` owns the exact code grammar, CSPRNG
    generation, digest calculation, and unit tests. The production server and
    local CLI compile the same source module so code shape cannot drift.
  - `RegistrationInput` gains `invite_code`; registration returns a created or
    replay outcome. `RegistrationError::InvalidInvitation` is the only public
    unavailable-code error. Direct Rust test fixtures obtain a fresh real
    digest-backed invite through a test-only helper rather than retaining an
    uninvited production bypass.
- API and CLI compatibility:
  - Registration request changes from `{username,password}` to
    `{invite_code,username,password}`. This is an intentional synchronized
    pre-alpha break: server docs, QML, fixtures, live smoke, and package source
    move together, and no optional/open fallback is preserved.
  - Created registration remains `201`; credential-proven exact replay is
    `200`; both expose only `{id,username}`. Invalid username/password remain
    422, canonical conflicts remain 409 while the invite is unused, every
    unavailable invitation is 403 `invalid_invitation`, and internal failures
    remain non-disclosing 500.
  - Existing `reports` and account/report `apply` output shapes remain exact.
    Applying invitation issuance returns a distinct exact metadata receipt;
    invitation revocation returns the existing audit-receipt shape. The new
    `invites` inventory action is additive and bounded.
- Exact implementation manifest:
  - `migrations/0017_invite_only_registration.sql` — invitation persistence,
    lifecycle checks/indexes, and invitation-target operator audit extension.
  - `crates/server/src/registration_invites.rs` — shared code generation,
    parsing/digest, grammar, and tests.
  - `crates/server/src/accounts.rs`, `app.rs`, `main.rs` — invitation-required
    transactional registration, replay/error outcomes, transport contract,
    module registration, and route-level no-store.
  - `crates/server/src/registration_api_tests.rs` plus affected direct test
    fixtures — lifecycle, transaction, replay, concurrency, privacy, and
    synchronized `RegistrationInput` coverage.
  - `crates/server/src/operator_admin.rs`,
    `src/bin/omarchygs-admin.rs`, and `tests/operator_cli.rs` — issue/list/
    revoke commands, exact output, audit, bounds, real process proof, and no
    network authority.
  - `client/qml/OnboardingController.qml`, `screens/AccessScreen.qml`, fixture
    server, onboarding/transport/accessibility fixtures, and live scenario —
    masked invitation UX, exact body, safe state/error handling, and real API.
  - `scripts/test-private-alpha.sh`, `scripts/dev.sh`, and `bin/gate.sh` —
    isolated operator/API drill, migrated QML integration, and canonical stage.
  - `docs/api.md`, `docs/operators/private-alpha.md`, existing operator/
    architecture/product/roadmap/README surfaces, OpenWiki, AAR, and knowledge
    register — public contract, operational checklist, limitations, and memory.
- Regression plan:

| Requirement | Evidence |
|---|---|
| REQ-001 | Operator PostgreSQL tests and real CLI test assert 256-bit code grammar, uniqueness, bounded live cap, exact first/replay receipts, digest-only database/audit state, and immutable issue audit. |
| REQ-002 | Inventory matrix covers issued/used/expired/revoked/all, stable newest-first limit, used public username, exact keys, and absence of code/digest/account UUID/credential/session fields. |
| REQ-003 | Domain and CLI tests cover exact revocation replay, changed-intent collision, used/expired/revoked/absent denial, simultaneous use/revoke, and audit immutability. |
| REQ-004 | Registration PostgreSQL/API tests cover successful link, Argon2id account, username-conflict rollback, simultaneous same-code attempts, and one account/one consumption. |
| REQ-005 | API tests cover `201` first result, `200` same credential replay, canonical username replay, wrong username/password and second-account denial, and immutable two-field receipt. |
| REQ-006 | Malformed/absent/expired/revoked/used matrix asserts one 403 code/message/headers, zero account effects, request body cap, no sensitive logs, and no lifecycle/operator fields. |
| REQ-007 | QML keyboard fixture checks conditional visibility, masked value, focus/traversal, clearing on submit/Escape/mode/server changes, exact body, allowlisted 403, malformed success, plain text, accessible names, and 640×420 containment; live QML uses a real issued code. |
| REQ-008 | `scripts/test-private-alpha.sh` proves the real binary/CLI/PostgreSQL/HTTP path and gate stage after the complete migration set. |
| REQ-009 | `docs/operators/private-alpha.md` and linked owner/safety guides receive a checklist review during Phase 5 and are covered by final source/gate/OpenWiki evidence. |

- Risks and mitigations:
  - Internet callers can submit random codes: exact grammar plus indexed digest
    lookup rejects them before Argon2id; 256-bit entropy makes guessing
    infeasible, while a public edge still needs distributed request limits.
  - A code can leak in storage or logs: only a digest enters PostgreSQL;
    command documents never contain the code; stdout first delivery is the
    sole raw output; lists/audit/errors exclude it; QML sends it only in JSON
    to the already admitted origin and never persists it.
  - Issue output can be lost after commit: operation replay prevents duplicate
    invitations but cannot recover a deliberately unpersisted secret. The
    inventory identifies the row so the operator can revoke it and issue a new
    code. The runbook makes this mandatory rather than suggesting database
    recovery.
  - Consumption can race revocation or another registration: every transition
    locks the same invitation row and rechecks current/expiry state inside the
    transaction. Account insertion and use linkage commit together.
  - Exact replay could become a credential oracle: it requires possession of
    the high-entropy used code plus exact canonical username and password,
    returns only the already-public creation receipt, and otherwise collapses
    to `invalid_invitation`; ordinary login remains the supported auth path.
  - Invitation inventory could expose identity ownership: only the private
    account username linked to the used invite is returned to the database-
    local operator; no account UUID, persona mapping, hash, token, or password
    field crosses the output.
  - Downgrading after migration would reopen registration: rollback is an
    operational stop-and-forward-fix, not an old-server downgrade. Operators
    must close external ingress if the invite path cannot safely serve.
- Alternatives rejected:
  - An environment-wide shared registration password cannot be individually
    expired, revoked, attributed, or safely delivered and would make every
    tester share one reusable secret.
  - Optional invite enforcement would leave configuration drift capable of
    reopening private-alpha registration; this release intentionally changes
    the one API contract in lockstep.
  - Storing recoverable raw codes would make database backups and operator
    inventory a credential store. A separate encryption key adds recovery and
    rotation complexity without private-alpha need.
  - Email delivery, a web admin panel, or remote invite API would add new
    outbound data handling or high-authority authentication before the
    database-local workflow is proven.
  - Multi-use or bulk codes complicate attribution, quotas, leakage response,
    and concurrency. One code admits exactly one account.
- CodeGraph evidence: design exploration traced the full Axum registration
  handler through `RegistrationInput`, canonicalization, bounded Argon2id
  hashing, unique PostgreSQL insertion, exact error mapping, router callers,
  and direct Rust test fixtures. A second exploration traced the local
  `omarchygs-admin` bounded-file adapter, command validation/application,
  account/report locks, audit receipt shape, and CLI tests; ambiguous
  `apply_command` symbols from the game crates were rejected as unrelated.
  CodeGraph does not model QML, Python, Bash, SQL, or documentation completely,
  so `AccessScreen.qml`, `OnboardingController.qml`, both QML test suites,
  `fixture_server.py`, `scripts/dev.sh`, migration `0016`, `bin/gate.sh`, and
  current API/operator docs were reviewed directly. The design receipt binds
  pipeline `9453a1ce-c7c6-405b-bfa5-25972f28a0be` to gated-state hash
  `20ad03838b4cebb08b5c1a27cce7fe026a74d05cdb8797cfe1e8f04c901708e2`.

## Phase 3 — Implement

- Added forward-only migration `0017_invite_only_registration.sql` with
  digest-only invitation persistence, exact lifecycle constraints and indexes,
  one-account consumption, and invitation-target operator audit linkage. The
  complete migration set applied through both SQLx tests and the isolated
  private-alpha drill.
- Added the shared `registration_invites` module with OS-CSPRNG 256-bit code
  generation, canonical `ogsi_` base64url parsing, SHA-256 digesting, and
  unit tests. Neither the server nor the operator inventory can recover a raw
  code from persisted state.
- Replaced open registration with invitation-required transactional account
  creation. The implementation prechecks invitation lifecycle before Argon2id,
  locks and consumes the invitation in the account transaction, preserves an
  invitation after username conflict, and returns the original two-field
  public receipt only after exact used-code username/password proof.
- Fixed a concurrency defect found by the PostgreSQL test: after a transaction
  blocks on another consumer's row lock, its earlier joined account projection
  can remain snapshot-stale even though the locked invitation now shows used.
  The losing path now rolls back and performs a fresh invitation/account read
  before exact-replay verification. The focused concurrent-consumption test
  then passed with one account and one consumption.
- Extended the database-local operator CLI with exact issue, bounded inventory,
  and revoke operations. Issue uses a dedicated advisory transaction lock for
  the live-code cap, commits before first secret delivery, omits the code on
  operation replay, and writes the matching immutable audit row atomically.
  Real-process and domain tests passed for once-only delivery, digest-only
  inventory, exact replay, cap denial, lifecycle denial, and audit linkage.
- Added the masked QML invitation field, exact registration body, `200`/`201`
  receipt handling, allowlisted invalid-invitation message, and synchronous
  clearing across submission, Escape, mode change, and server change. The first
  fixture run found that the styled text-field primitive did not expose an
  editable accessibility role; assigning that role in `OgsTextField` made all
  41 QML fixture tests pass.
- Added `scripts/test-private-alpha.sh` and canonical gate stage 22. Its
  isolated generated database, real local CLI, real loopback server, HTTP
  registration/replay/denial/sign-in, revocation, inventory/audit checks,
  secret scan, and cleanup completed successfully.
- Updated `scripts/dev.sh --smoke-test` to issue separate real invitations for
  QML, primary curl, conflict rollback, and peer registration. A first full
  smoke run exposed one remaining legacy peer request without `invite_code`;
  after synchronizing it and consuming the conflict probe's still-usable code,
  the complete 41-test fixture plus four live QML/API/game/social/MFA paths
  passed and exited zero.
- Updated the versioned API contract, architecture/product/roadmap/README, and
  operator guidance. The roadmap now distinguishes software readiness from the
  still-unperformed external two-clean-installation human acceptance event.
- Focused evidence completed during implementation:
  - `cargo check -p omarchy-gaming-system-server --all-targets` passed;
  - the concurrent registration PostgreSQL test passed after the snapshot fix;
  - two focused operator invitation PostgreSQL tests passed;
  - the real operator CLI invitation test passed;
  - QML fixture totals: 41 passed, 0 failed;
  - `scripts/test-private-alpha.sh` passed;
  - `scripts/dev.sh --smoke-test` passed end to end.

## Phase 3.5 — Inspect

- Correctness and blast radius: a fresh CodeGraph exploration traced both
  registration paths from `register_account` through `exact_replay` and
  `credentials::verify_password`, including the rollback-and-refresh race
  branch, shared invitation digest module, Axum request shape, direct test
  callers, and operator surfaces. It confirmed that the final implementation
  always reaches password verification before combining a used invitation's
  username predicate. The worktree-bound inspection receipt records pipeline
  `9453a1ce-c7c6-405b-bfa5-25972f28a0be`, gated-state hash
  `6861cab279e03ef21c1a53393b45e0c9e9718902f7bbef08784ed3852bd575cd`,
  and `mcp__codegraph__codegraph_explore`; `scripts/check-pipeline.sh` passed
  against it.
- Authentication and privacy: the Codex Security diff scan froze the staged
  pre-fix patch at base
  `296be36f5ca8d2cd2fbc8d92a59905895c8d10f6`, snapshot digest
  `7245d181a5aa7aac1b25383a4f33d35c64b35fbdd7d963b606d867c58972027c`,
  and completed all inventoried executable plus supplemental QML/Python/docs
  surfaces. Its sealed report at
  `/tmp/codex-security-scans/omarchy_bbs/296be36f5ca8_20260826T190620Z/report.md`
  found one high-confidence, low-severity CWE-208 issue: a caller holding a
  leaked already-used code could distinguish its linked canonical username
  because a mismatch returned before Argon2id. The real isolated reproduction
  measured 0.003187 seconds for wrong username versus 0.318077 seconds for
  correct username/wrong password despite identical 403 bodies.
- Finding disposition: `exact_replay` now computes username equality, always
  performs the stored-hash password verification, and only then combines the
  two results into the common invalid-invitation response. Read-only fix
  verification against a rebuilt real Axum/PostgreSQL path measured 0.500563
  versus 0.494806 seconds (1.012×), confirmed identical denial bodies, and
  retained HTTP 200 for legitimate exact replay. `CAND-030-001` is fixed.
- Transaction and concurrency review found no remaining split-commit path:
  invitation admission is rechecked under the row lock, account insertion and
  use linkage share one transaction, conflicts roll back, and the blocked
  contender refreshes its joined account data only after ending the stale
  transaction. Issuance admission uses its dedicated advisory lock, while
  revoke/use serialize on the invitation row.
- Secret and authority review found no remaining raw-code persistence or
  remote administrator surface. The operator command commits before a single
  stdout delivery; database, inventory, replay, audit, application errors,
  and logs carry metadata or digests only. QML keeps the invite masked and
  clears it on every credential-lifetime boundary. The security scanner could
  not verify TAC-protected output because that optional connector was not
  configured; this limitation does not affect local source/runtime coverage.
- Contract and UX review confirmed the intentional pre-alpha API break is
  synchronized across every real Rust, curl, fixture, and live-QML caller;
  deny-unknown-fields, body bounds, exact success documents, no-store headers,
  allowlisted client text, keyboard flow, accessibility role, and minimum-size
  containment all have focused coverage.
- Inspection checks:
  - an initial direct ignored PostgreSQL test invocation failed before running
    because `DATABASE_URL` was absent; after starting the healthy Compose
    database and supplying the documented URL, the exact-replay integration
    test passed;
  - `cargo build -p omarchy-gaming-system-server --bins` passed before the
    runtime fix verification;
  - the security-fix runtime verification passed its under-2× latency bound,
    response-equivalence check, and legitimate-replay check;
  - `scripts/check-pipeline.sh` passed with the fresh inspection receipt.

## Phase 4 — Validate

- The first `bin/gate.sh --diff` run passed stages 1–20, including rustfmt,
  Clippy, all non-database Rust tests, rustdoc, Compose/schema checks, shell
  syntax, pipeline structure, changed-file secret scan, Codex hook self-tests,
  cartridge/renderer/SDK/architecture proofs, reproducible native client
  packaging, all 50 PostgreSQL API tests, all five operator-domain tests, both
  real operator CLI tests, 41 QML fixtures, four real migrated QML scenarios,
  remote-provider security conformance, and the clean-clone first-party
  provider pilot.
- That first run ended red at stages 21 and 22 because both fresh-database
  drills used a fixed ten-second server-start deadline. The recovery server
  remained alive with no error log but missed readiness after the preceding
  compile/provider load; the following alpha drill hit the same condition.
  Isolated reruns and a shell execution trace proved the variable cold
  migration startup path (about 6.5 seconds on one passing trace), while both
  full drill assertions passed independently.
- The recovery and private-alpha scripts now retain their process-death checks
  but use a bounded 30-second readiness window for a cold 17-migration
  database. `bash -n`, the complete backup/restore drill, and the complete
  private-alpha admission drill passed after the adjustment. Direct shell
  inspection found no secret-lifetime or cleanup change, and the refreshed
  CodeGraph inspection receipt binds the final gated state to
  `df9e15e99844edd5d945e2e2f3364ec117a923df54aa0a6b04258f5945b4881d`.
- The required second `bin/gate.sh --diff` run passed every stage, including
  the two repaired late drills, and ended `GATE GREEN [diff]`. The canonical
  delivery receipt at `.git/omarchy-gaming-system-gate-receipt` contains the
  same final gated-state hash
  `df9e15e99844edd5d945e2e2f3364ec117a923df54aa0a6b04258f5945b4881d`.
- Validation outcome: Ticket 030 is software-ready for an owner to execute the
  private-alpha runbook. The separate roadmap item for an actual external
  two-clean-installation human event remains honestly incomplete.

## Phase 5 — Complete

- EARS audit:

| Requirement | Completion evidence | Result |
|---|---|---|
| REQ-001 | `registration_invites::generate`, migration `0017`, operator issue transaction, operator domain test, and real CLI test prove 256-bit canonical codes, digest-only persistence, once-only raw delivery, bounds, replay, and immutable audit. | PASS |
| REQ-002 | `list_invitations`, domain/CLI exact-JSON assertions, and the private-alpha drill prove bounded newest-first state inventory with no raw code, digest, account UUID, credential, or session material. | PASS |
| REQ-003 | Row-locked revoke logic and PostgreSQL tests prove exact replay, changed-intent conflict, lifecycle denial, use/revoke serialization, and append-only audit. | PASS |
| REQ-004 | Registration transaction plus the simultaneous-consumer PostgreSQL case prove atomic account/invitation linkage, username-conflict rollback, and at most one account. | PASS |
| REQ-005 | Registration API tests and the real drill prove `201` first use, `200` exact canonical credential replay, stable two-field receipts, and uniform denial for changed intent. | PASS |
| REQ-006 | Lifecycle matrix, body/error/no-store assertions, runtime log inspection, and security review prove one `403 invalid_invitation` shape with no lifecycle/operator disclosure or account side effect. | PASS |
| REQ-007 | All 41 QML fixtures plus live migrated onboarding prove masked conditional entry, editable accessibility semantics, keyboard flow, exact body, admitted origin, allowlisted text, and synchronous secret clearing. | PASS |
| REQ-008 | `scripts/test-private-alpha.sh` runs at gate stage 22 and proves issue, first use, exact replay, denial, sign-in, revocation, inventory, persistence, audit, and cleanup through real boundaries. | PASS |
| REQ-009 | `docs/operators/private-alpha.md` covers TLS/deployment and secret preconditions, issue/delivery/revocation, clean-client onboarding, MFA, gameplay, feedback/reporting, recovery, stop conditions, and unsupported responsibilities. | PASS |

- OpenWiki update run `12e04753-89a9-401d-9730-6ba6b3f94d52` completed in
  update mode. `quickstart.md`, `runtime-foundation.md`,
  `development-and-validation.md`, and `product-boundaries.md` now describe the
  invite-only runtime, operator authority, QML secret lifecycle, gate stage 22,
  and software-readiness/human-event distinction. Finalization warned that the
  broad pages' existing Claims sidecars retain unrelated unresolved evidence
  debt and therefore left those sidecars unchanged; it did not reject the run
  or the Ticket 030 claims.
- AAR-030 was submitted at 5/5 with five failures, five prevention rules, and
  the invite-only account-admission architecture decision. Every new ID was
  appended to `docs/planning/knowledge/INDEX.md`.
- Completion disposition: all nine requirements pass, no finding remains open,
  and Ticket 030 is ready to archive. The actual external human alpha event is
  still intentionally not claimed.
