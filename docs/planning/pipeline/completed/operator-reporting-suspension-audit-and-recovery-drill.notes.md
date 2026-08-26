---
title: Operator reporting, suspension, audit, and recovery drill — notes
pipeline_id: 3515d516-b7b1-475d-bcbc-e44c383d7215
---

# Operator reporting, suspension, audit, and recovery drill — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 028 delivered the native player package to remote `main` at
  `9d97d95ef65f259b096289754fecbdd22e858f91`; no active pipeline or critical
  bulletin remained when Ticket 029 opened.
- Recall: the next unchecked private-alpha roadmap outcome is reporting,
  suspension, audit records, backups, and a restore drill. Invite-only external
  testing follows it.
- Recall: accounts already use `active`, `suspended`, and `disabled` states,
  and session authentication fails closed for inactive accounts. Existing
  tests change status directly; there is no general sysop action contract,
  player report API, immutable platform moderation audit, or platform restore
  proof.
- Recall: the provider subsystem has a separate operator registry, lifecycle,
  audit, and provider-database restore proof. Ticket 029 must not reuse provider
  authority or confuse the provider database with the platform database.
- Decision: implement a database-local sysop CLI rather than a remote admin
  endpoint. The CLI receives PostgreSQL authority from the operator environment
  and every mutation still requires bounded actor/reason input and durable
  audit.
- Decision: reports target another persona and contain a small fixed category,
  bounded plain-text detail, and idempotency UUID. Attachments, message/game
  evidence, content deletion, bans, and appeals remain separate policy slices.
- Decision: suspension revokes all current sessions atomically; reactivation
  permits fresh login but cannot restore revoked tokens. The `disabled` state
  stays outside reversible suspension.

## Phase 2 — Design

- Architecture and authority flow:
  1. `POST /v1/personas/{reporter_persona_id}/reports` authenticates through
     the existing Bearer session and owner-scopes the path persona before any
     report or subject state is disclosed. `reports.rs` validates an exact UUID
     idempotency key, a fixed category (`harassment`, `spam`, `cheating`, or
     `other`), and 1–1,000 trimmed plain-text detail characters with unsafe
     controls rejected.
  2. The report transaction locks the guaranteed reporter persona root, checks
     replay before current open-report admission, rejects a self-report,
     validates the subject persona, enforces at most 25 open reports per
     reporter, and inserts one receipt. An exact replay returns the original;
     the same key with different subject/category/detail conflicts. The player
     response contains only `id`, `idempotency_key`, `status`, and
     `created_at`; it does not reveal account ownership, operator state, other
     reporters, or report inventory.
  3. `SocialController` adds a distinct report-by-exact-handle flow rather than
     overloading connection lookup state. It resolves the existing public
     persona endpoint, then submits the report through the same bearer-owning
     onboarding gateway with a fresh UUID. A small Social-screen form owns
     exact handle, fixed category, and bounded detail. Success validates the
     exact receipt and clears handle/detail; transport, protocol, API, and
     invalid-session paths retain existing fail-closed behavior.
  4. `omarchygs-admin` is a second binary in the server package. It receives
     only `DATABASE_URL` from the environment. `reports [status] [limit]`
     produces a bounded newest-first JSON inventory for a trusted local sysop;
     `apply <document>` reads one nonempty bounded non-symlink regular JSON file
     and accepts only tagged account-status or report-status commands with an
     idempotency UUID, bounded actor, and bounded reason. No route, admin token,
     account role, or network listener is added.
  5. Account action acquires the target account row, resolves an exact action
     replay before current-state checks, permits only active ↔ suspended,
     updates `accounts.updated_at`, revokes every unrevoked session with one
     transaction timestamp when suspending, and appends the audit event before
     commit. Reactivation never clears `revoked_at`; `disabled` is denied.
  6. Report action locks the target report, resolves exact replay, permits only
     open → resolved/dismissed, sets one terminal timestamp, and appends the
     audit event in the same transaction. Target-scoped unique idempotency and
     root locking serialize simultaneous identical or conflicting operations.
  7. `operator_audit_events` is insert-only: database triggers reject update
     and delete. Each event binds its UUID, target kind/ID, operation UUID,
     actor, reason, action, previous state, resulting state, and timestamp.
     Report rows also reject deletion; player-supplied detail never enters the
     operator audit record or process log.
  8. `scripts/test-operator-recovery.sh` creates validated per-process source
     and restore database names, applies the complete forward migration set,
     seeds bounded representative platform state, drives the real admin CLI,
     writes a custom-format `pg_dump`, restores with `--exit-on-error`
     `--no-owner`, and compares every public application-table count plus
     focused report/audit/account/session/social/inbox/game assertions. It
     starts the production server on loopback against the restore and proves a
     pre-suspension raw token is rejected. Cleanup drops only the two exact
     validated databases.
- Data model:
  - `persona_reports`: UUID ID, reporter/subject persona FKs, idempotency UUID,
    fixed category, bounded detail, open/resolved/dismissed status,
    created/updated/closed timestamps, reporter/key uniqueness, self-report
    check, status/timestamp consistency, lookup indexes, and deletion denial.
  - `operator_audit_events`: UUID ID, operation UUID, target discriminator and
    nullable account/report FK with an exact-one-target check, fixed action,
    bounded actor/reason, prior/result state, created timestamp, target-scoped
    idempotency indexes, and update/delete denial.
  - No existing table or authority discriminator is repurposed. Migration
    `0016` is forward-only; rollback is a later forward migration, not a down
    script.
- API/error contract:
  - Created report: HTTP 201; exact replay: 200.
  - Invalid body/category/detail/self-report: 422 `invalid_report`.
  - Invalid/foreign reporter or absent subject: 404 `persona_not_found` after
    authentication precedence.
  - Idempotency collision: 409 `report_idempotency_conflict`.
  - Open-report cap: 429 `report_limit_reached`.
  - Missing/invalid/inactive session: existing 401 `invalid_session`.
  - Report responses use `Cache-Control: no-store`; no sync event or subject
    notification is introduced in this slice.
- CLI contract:
  - `omarchygs-admin reports [open|resolved|dismissed|all] [1..100]` emits one
    exact JSON object and does not mutate or audit a read.
  - `omarchygs-admin apply PATH` accepts a deny-unknown-fields tagged document:
    `set_account_status` targets `active` or `suspended`; `set_report_status`
    targets `resolved` or `dismissed`. Both require UUID `idempotency_key`,
    actor, reason, and target UUID and return an exact audit receipt.
  - Stdout is machine-readable JSON; stderr exposes only stable error codes.
    Database messages, URLs, credentials, report detail, and password/session
    material are not logged.
- Database/migration consequences: two retained platform tables, indexes, and
  immutability triggers. Suspending an account mutates its existing row and
  current sessions transactionally; it does not delete personas, inboxes,
  reports, game history, MFA configuration, or provider state.
- Compatibility consequences: one additive player REST route and additive
  CLI. Existing auth, persona, social, inbox, game, sync, provider, and QML
  routes retain their shapes. Provider lifecycle/audit tables and Door Legends
  backup remain independent.
- Exact implementation manifest:
  - `migrations/0016_operator_reporting_and_audit.sql` — report/audit schema,
    indexes, and immutability triggers.
  - `crates/server/src/reports.rs`, `app.rs`, `main.rs` — report domain, route,
    wire responses/errors, and module registration.
  - `crates/server/src/operator_admin.rs` plus
    `crates/server/src/bin/omarchygs-admin.rs` — shared local CLI actions and
    bounded command adapter; the main server never instantiates admin
    authority.
  - `crates/server/src/report_api_tests.rs` and operator-focused tests — API,
    transaction, transition, concurrency, privacy, and audit proof.
  - `client/qml/SocialController.qml`, `screens/SocialScreen.qml`, fixture
    server, `tst_social.qml`, accessibility fixture, and live social scenario —
    exact-handle player report UX and real vertical slice.
  - `scripts/test-operator-recovery.sh`, `scripts/test-database.sh`,
    `scripts/dev.sh`, and `bin/gate.sh` — focused recovery, migrated/live, and
    canonical delivery evidence.
  - `docs/api.md`, `docs/operators/operator-safety-and-recovery.md`, README,
    product/architecture/roadmap, OpenWiki, and pipeline knowledge — public and
    operator contracts.
- Regression plan:

| Requirement | Evidence |
|---|---|
| REQ-001 | PostgreSQL API tests cover exact response, replay/collision, self/foreign/absent cases, validation, open cap, and simultaneous first delivery. |
| REQ-002 | QML fixture covers keyboard form, exact requests, clearing on success, retry-safe failure, hostile document/size/session outcomes, plain text, accessible names, focus, and 640×420 layout; live social QML proves the real API row. |
| REQ-003 | CLI test checks status filters, limit bounds, stable ordering, exact keys, public personas, subject account target, and absence of secret/password/session fields. |
| REQ-004 | Operator transaction tests prove one suspend event, all live sessions revoked at the same timestamp, old Bearers denied, exact replay, and one serialized winner. |
| REQ-005 | Transition matrix proves suspended → active, old tokens stay revoked, fresh password/MFA login is eligible, and disabled transitions fail. |
| REQ-006 | Report action tests prove exact replay, collision, terminal-transition rejection, competing resolve/dismiss winner, and immutable linked audit. |
| REQ-007 | Router inventory and source review show no admin route/listener/token; CLI file/env/error bounds and secret scan pass. |
| REQ-008 | Isolated full-schema custom dump/restore compares every application table and proves representative security/history state plus old-token rejection against the restored production server. |
| REQ-009 | Operator guide and canonical DIFF/FULL gate cover action, recovery, key custody, rollback, and limitations. |

- Risks and mitigations:
  - Reports can be abused for write amplification: fixed body limit, strict
    validation, idempotency, and a transactionally enforced open-report cap.
  - Subject existence can be probed: the route requires an authenticated owned
    reporter and exposes only the same public persona identity available by
    exact handle; it never exposes the subject account.
  - Sysop CLI database authority is powerful: it is local-only, has no network
    listener/token, accepts a narrow command enum, writes mandatory audit, and
    documents that database credentials remain operator secrets.
  - Suspension can race login/MFA/session use: the account row is the canonical
    lock; all issuance already locks/rechecks active account state, session
    authentication predicates on active status, and suspension revokes current
    sessions inside its transaction.
  - Audit can leak player text or secrets: audit stores only bounded operator
    actor/reason and state identifiers; report detail stays in the report row.
  - Restore proof can damage a live database: generated names are validated,
    the admin URL targets PostgreSQL's administrative database, and cleanup
    operates only on the two exact names.
  - A database backup without the MFA key cannot service enrolled accounts:
    docs require separate protected custody and restoration of the exact
    `OGS_MFA_ENCRYPTION_KEY`; the key is never placed in a dump.
- Alternatives rejected:
  - A remote admin API/role model would add a second high-authority
    authentication and exposure system before private-alpha need justifies it.
  - Direct SQL instructions provide no input contract, action receipt,
    transactionally coupled session containment, or consistent audit.
  - Permanent ban/content deletion in the first reporting slice would require
    retention, appeal, evidence, and legal policy that is not yet designed.
  - Reusing provider operator tables would collapse different targets,
    databases, keys, and authority domains.
  - Backing up the developer's fixed database would be stateful and destructive;
    the drill owns isolated databases and leaves the normal service untouched.
- Operational/rollback behavior: suspend first for reversible containment;
  reactivation requires fresh player login. Report dispositions are terminal
  but preserve the original report. Backup artifacts are private temporary
  files deleted by the test. Production operators must separately choose
  encrypted storage, scheduling, retention, off-host copies, monitoring, and
  provider-database recovery.
- CodeGraph evidence: three worktree-bound explorations traced
  `sessions::authenticate`, current session issuance/revocation, the Axum
  router and handler boundary, `AppState`, persona ownership callers, server
  configuration, and the broad router test blast radius. Authentication already
  predicates every use on `accounts.status = 'active'`, login locks/rechecks the
  account before issuance, and the router has no admin route. CodeGraph did not
  establish the QML, migration, or shell recovery graph and returned ambiguous
  provider CLI symbols for the broad query, so direct review of
  `SocialController.qml`, `SocialScreen.qml`, migration order, the provider
  admin adapter, QML fixtures, and `test-provider-authority-pilot.sh` is
  authoritative for those formats. The matching design receipt is bound to
  pipeline `3515d516-b7b1-475d-bcbc-e44c383d7215` at gated state
  `40d5cf55ceb852eeecc18bc361dc38d1bc0eb45bbd3951c15ecf6baa5337c4b5`.

## Phase 3 — Implement

- Added migration `0016` with retained persona reports, target-scoped operator
  audit, report deletion denial, and audit update/deletion denial.
- Added authenticated owner-scoped report creation with strict body/category/
  detail bounds, 25-open admission, exact idempotency, privacy-minimal receipts,
  no-store responses, and explicit API errors. Exact replay remains the
  original open creation receipt even after operator disposition.
- Added the keyboard-first Social report form/controller path. It resolves the
  exact public handle, retains the operation UUID across uncertain retries,
  validates exact receipts, clears text only after success, and inherits the
  existing hostile-size/protocol/session teardown boundary.
- Added `omarchygs-admin` as a second server-package binary. Its bounded,
  non-symlink regular JSON adapter reads only `DATABASE_URL`, exposes no
  listener or admin token, lists newest-first reports, and transactionally
  applies account/report state commands with stable non-disclosing errors.
- Account actions lock the target root, revoke every live device session on
  suspension, permit only active ↔ suspended, preserve revoked tokens on
  reactivation, deny disabled accounts, and append one audit event. Report
  actions lock the report and permit one open → resolved/dismissed transition.
  Both action families return the original audit receipt on exact retry and
  serialize competing operations.
- Added `scripts/test-operator-recovery.sh` and gate stage 21. The drill owns
  two validated generated databases, migrates through the production server,
  seeds representative platform state, drives the real CLI, performs a custom
  dump/restore, compares every application-table count, checks focused state
  and restored immutability, and proves the restored production server rejects
  a pre-suspension token.
- Added the operator guide, player API contract, architecture/product/roadmap
  updates, live QML report proof, and native development smoke integration.
- Focused evidence completed during implementation:
  - `scripts/test-qml-onboarding.sh`: 41 passed, zero failed;
  - report API PostgreSQL tests: 2 passed, zero failed;
  - sysop domain PostgreSQL tests: 3 passed, zero failed;
  - real sysop CLI PostgreSQL test: 1 passed, zero failed;
  - `scripts/test-operator-recovery.sh`: passed;
  - `scripts/dev.sh --smoke-test`: hostile fixture plus four real QML scenarios
    passed, including the stored player report.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Authorization and privacy | The report path authenticates first, binds the reporter persona to the session account under a row lock, exposes only the creation receipt, and emits neither subject notification nor sync data. | none | Pass; API tests cover invalid session precedence, foreign reporter denial, absent target, exact response keys, and no-store success/errors. |
| 2 | Idempotency and concurrency | Initial replay reconstruction projected mutable report status after an operator disposition instead of the immutable creation status. | medium correctness | Fixed before inspection closure; replay now uses immutable creation fields and a database-backed API test proves exact replay after resolution. Reporter locking also serializes the open cap and simultaneous first delivery. |
| 3 | Operator authority, file input, and secret output | The CLI has no listener or reusable administrator credential; it accepts exact argv/JSON shapes, reads one bounded non-symlink regular file, compares the opened descriptor device/inode with pre-open metadata, and suppresses database errors/URLs. | none | Pass; full source inspection and real CLI tests prove exact result keys, stable stderr, symlink rejection, and secret-free report output. |
| 4 | State transitions, audit, and session containment | Account/report root locks serialize decisions; suspension and live-session revocation share the audit transaction; reactivation cannot clear revoked timestamps or change `disabled`; report disposition is terminal. | none | Pass; focused domain concurrency/replay tests, current session authentication predicates, and restored-server denial establish the boundary. |
| 5 | Migration and destructive recovery safety | Report/audit constraints and triggers preserve application integrity; the recovery proof owns only two generated regex-validated database names and restores into isolation before comparing all application tables. | none | Pass; full migration/script review and the successful production-binary dump/restore drill close the destructive and history-loss hypotheses. |
| 6 | QML protocol, retry, and accessibility | The production controller preserves the UUID only for exact uncertain retry, validates exact public lookup/receipt schemas and bounds, tears down invalid sessions, and renders fixed guidance as plain text. | none | Pass; full production-QML review plus 41 fixture tests and live report smoke cover hostile size/schema/session behavior, keyboard access, and 640×420 layout. |
| 7 | Security diff scan | Codex Security reviewed all 11 generated source-like changed files plus the unsupported production QML files against a source-backed threat model. IDOR, quota race, retry confusion, session resurrection, command substitution, conflicting decisions, SQL injection, secret output, and destructive restore hypotheses did not retain a broken-control/impact tuple. | none | Complete with zero reportable findings; sealed terminal report: `/tmp/codex-security-scans-omarchy_bbs-iX9vU3/report.md`. TAC display status remained unknown because the access check returned `USER_NOT_LOGGED_IN`; review coverage itself was complete. |
| 8 | CodeGraph blast radius | Post-implementation exploration traced the report handler/domain/session callers and operator paths; unsupported QML, SQL, shell, and test formats were inspected directly. | none | Matching inspection receipt is bound to pipeline `3515d516-b7b1-475d-bcbc-e44c383d7215` and gated state `63dcf4dcfd56b05f35a5a9331286aaf0cb8ad67ae5a4720ba8339a6c86077d22`. |

## Phase 4 — Validate

- Tests run: the canonical gate ran rustfmt, clippy with warnings denied,
  workspace unit/doc tests, the full 47-test migrated PostgreSQL server suite,
  all three operator-domain database tests, the real operator CLI test, the
  41-test QML fixture, four production-backed live QML scenarios, cartridge
  contract/renderer/SDK/spike proofs, native client packaging, remote provider
  conformance, the clean-clone first-party provider pilot, and the isolated
  platform backup/restore drill. Every executed test passed.
- Gate run: `bin/gate.sh --diff` completed `GATE GREEN [diff]` across all 21
  stages. Its receipt, the recomputed gated state, and the Phase 3.5 CodeGraph
  receipt all equal
  `63dcf4dcfd56b05f35a5a9331286aaf0cb8ad67ae5a4720ba8339a6c86077d22`.
- Skips or pre-existing failures: none in the canonical diff gate. The ordinary
  workspace unit-test stage intentionally reports database/provider cases as
  ignored, and the later dedicated PostgreSQL/provider stages executed those
  cases successfully. The renderer emitted expected headless `libEGL`
  warnings without test failure.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — two migrated report API cases plus the local body-cap test
    prove authenticated owner scope, fixed category/detail bounds, exact
    response fields, self/absent/foreign denial, 25-open admission, exact
    replay/collision, immutable creation replay after disposition, and
    simultaneous-first-write serialization.
  - REQ-002 PASS — the 41-case QML fixture proves keyboard submission, exact
    lookup/create requests, retained UUID on uncertain retry, success-only text
    clearing, accessible controls, 640×420 containment, hostile schema/size,
    and invalid-session cleanup; the live social scenario commits the real row.
  - REQ-003 PASS — the operator-domain inventory test and real CLI test prove
    bounded newest-first filters, exact public-persona/report fields, subject
    account targeting, stable JSON, and absence of password/session secrets.
  - REQ-004 PASS — the account transaction test proves target locking, one
    serialized suspension winner, same-transaction revocation of all live
    sessions, exact replay, immutable linked audit, and subsequent Bearer
    denial; the restore drill repeats old-token denial through the real server.
  - REQ-005 PASS — the transition matrix proves suspended → active with a new
    audit event, persistent old-token revocation, fresh login eligibility, and
    denial of `disabled` transitions.
  - REQ-006 PASS — the report transaction test proves one open → resolved or
    dismissed transition, exact replay, changed-intent conflict, competing
    decision serialization, terminal denial, and immutable linked audit.
  - REQ-007 PASS — router inventory and source/security review prove no admin
    listener, route, account role, or token; the executable accepts only
    `DATABASE_URL`, bounded argv/JSON/file input, and stable non-secret errors.
  - REQ-008 PASS — `scripts/test-operator-recovery.sh` creates a custom dump,
    restores into its isolated generated database, compares every public
    application table, verifies report/audit/suspension/session and
    identity/social/inbox/game state, and rejects the old token.
  - REQ-009 PASS — the operator guide, API contract, README, product charter,
    architecture, roadmap, generated wiki, and 21-stage canonical gate cover
    action, recovery, external MFA-key custody, rollback, and limitations.
- Docs: OpenWiki update run `0d74dfce-d12e-451f-94d9-9657c843ec79`
  reconciled the quickstart, runtime, product, validation, and Codex-workflow
  claims and returned `status: complete` with zero unresolved claims. Its
  receipt is bound to pipeline `3515d516-b7b1-475d-bcbc-e44c383d7215` and
  gated state
  `506ff619057c2a3cb488cc83d43975eab9c59d9e7f4f2d44602344b23770a0ac`.
- AAR: submitted at effectiveness 5 with four captured failures, four standing
  prevention rules, and the database-local operator safety architecture
  decision appended to the knowledge register.
- Archive: ticket moved to closed and spec/notes moved to completed. No active
  spec/notes pair remains. The final post-archive `bin/gate.sh --diff` passed
  all 21 stages and wrote the matching gated-state receipt
  `506ff619057c2a3cb488cc83d43975eab9c59d9e7f4f2d44602344b23770a0ac`;
  the OpenWiki completion receipt remains matching at the same state.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first QML report API test found error responses lacked `Cache-Control: no-store`. | The success handler applied no-store, but router-level report error responses bypassed it. | Applied the response policy to the bounded report route and retained the success wrapper. | Assert no-store on both success and every error class for private mutation routes. |
| 2 | The first recovery drill seed failed because `game_sessions.authority` is required after migration 0015. | The direct representative seed was designed from the original game table without reconciling the later provider-authority alteration. | Seeded the explicit `platform_compiled` authority and reran the full drill. | Review the final cumulative schema, not only the table's creating migration, for restore fixtures. |
| 3 | Adding the sysop binary made `cargo run -p omarchy-gaming-system-server` ambiguous and broke the development smoke. | The package had relied on Cargo's single-binary inference. | Set the package `default-run` to the production server. | Every package that gains a second binary must pin its default runtime or update all launch consumers atomically. |
| 4 | Inspection found report-create replay could return a changed terminal status after operator disposition. | Replay projected the report's mutable current status instead of its immutable creation receipt. | Replay now returns the original `open` creation receipt; a PostgreSQL API test resolves the row and proves byte-equivalent replay. | Idempotent creation receipts must be reconstructed only from immutable creation fields. |
