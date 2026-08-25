---
type: "Reference"
title: "Development and validation"
openwiki_generated: true
sources:
  - id: openwiki-source-0bb8016edf4f4744d3a09cf4
    resource: repo://bin/gate.sh
  - id: openwiki-source-d392f8f0962c50f0d66e0629
    resource: repo://client/qml/Main.qml
  - id: openwiki-source-937883bc0b4873d5f0200c46
    resource: repo://CONSTITUTION.md
  - id: openwiki-source-9ba5739252220892895a7a47
    resource: repo://crates/server/src/connection_api_tests.rs
  - id: openwiki-source-a243b385d49ea9224173d77a
    resource: repo://crates/server/src/game_api_tests.rs
  - id: openwiki-source-b2c7af59f511c4ed8a004fb0
    resource: repo://crates/server/src/inbox_api_tests.rs
  - id: openwiki-source-22753602a862c32d10560204
    resource: repo://crates/server/src/persona_api_tests.rs
  - id: openwiki-source-46fb4135d6a71efad1062c0d
    resource: repo://crates/server/src/sync_api_tests.rs
  - id: openwiki-source-d35448de763d92d5820dbaad
    resource: repo://scripts/check-pipeline-tools.sh
  - id: openwiki-source-a5928e7ee39885995efdc170
    resource: repo://scripts/dev.sh
  - id: openwiki-source-ff3f60a113327d3006289ed7
    resource: repo://scripts/mcp-openwiki.sh
  - id: openwiki-source-037d6d04880b10f227f0ac17
    resource: repo://scripts/setup-pipeline-tools.sh
  - id: openwiki-source-77975b35449f204d64ad5930
    resource: repo://scripts/test-database.sh
generated: {by: "codex", at: "2026-08-25T01:37:12.518Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-25T01:37:12.518Z
---

# Development and validation

## Run the vertical slice

`scripts/dev.sh` is the local orchestration entrypoint. It verifies Docker,
mise, QML, curl, jq, OpenSSL, and Python; starts PostgreSQL; launches the Rust
server; waits for a successful health response; and opens the QML client.
Closing the client stops the child server while leaving PostgreSQL running for
subsequent work.

If `OGS_MFA_ENCRYPTION_KEY` is absent, the script creates and reuses a
mode-0600 key at `.dev/mfa-encryption-key`, which is ignored by Git. Supplying
the variable explicitly overrides that development key. Production replicas
and restores need deliberate shared key management; the local file is only a
development convenience.

The executable path uses the `omarchy-gaming-system-server` Cargo package,
`OGS_BIND_ADDRESS`, the `omarchy_gaming_system` development database, and the
gaming-system log target. Smoke mode requires `/health.service` to equal
`omarchy-gaming-system` and requires newly issued bearer tokens to start with
`ogs1_` before exercising authenticated operations.

In smoke mode the script first requires public `GET /v1/games` to return the
honest empty production catalog. It then creates a uniquely named account
through `POST /v1/accounts`, verifies the success response omits password-
derived data, and repeats the request to require `username_taken` with HTTP 409.
It then creates a device session and verifies the authenticated inventory.
Before revocation it creates a persona, proves the exact public field set and
private-field absence, checks owned inventory and public lookup, edits the
persona, and proves the old handle disappears while the new handle resolves the
updated profile. It creates a second account and persona, then exercises
request, incoming inventory, acceptance, and mutual inventory. Acceptance must
also expose one private conversation with a typed system message; the peer then
sends a user message, the first persona reads ascending history and clears its
unread state. Removal must preserve that history while rejecting another send.
The flow continues through block, private blocked-request rejection, private
block inventory, unblock, re-request, and pending cancellation. It then enrolls
and confirms TOTP, proves primary login creates no premature session, completes
the MFA challenge with a recovery code, rejects replay, disables MFA with both
required proofs, and proves password-only login returns. Finally it revokes the
current device and requires the reused token to return `invalid_session` with
HTTP 401. The smoke also captures the persona synchronization baseline and
requires ordered REST invalidations for the exercised social and inbox writes;
it deliberately leaves the richer WebSocket matrix to the real-TCP integration
suite.

The QML client polls `http://127.0.0.1:8080/health`, distinguishes connected,
offline, and invalid-JSON states, and offers a reconnect action. Smoke mode uses
the offscreen Qt backend and exits after the request completes or after a
five-second watchdog.

Useful commands:

```bash
./scripts/dev.sh
./scripts/dev.sh --smoke-test
docker compose down
```

## Canonical gate

`bin/gate.sh --fast` runs the static development loop without writing a receipt.
`bin/gate.sh --diff` adds isolated migrated PostgreSQL tests plus the live
PostgreSQL → Rust game-catalog/health/account/session/persona/social/inbox/sync/
MFA API → QML smoke and writes a receipt for the exact gated worktree at
`.git/omarchy-gaming-system-gate-receipt`.

The gate currently covers:

1. Rust formatting and warning-denied Clippy;
2. workspace tests and warning-denied rustdoc;
3. Compose, shell, pipeline, secret, hook, and whitespace checks;
4. thirty-three ignored router tests against SQLx-managed PostgreSQL databases in
   diff/full modes;
5. the live empty game catalog, health, registration, duplicate-conflict,
   session creation/list, persona creation/list/public lookup/edit/handle
   movement, connection, private inbox, synchronization recovery, and block
   lifecycle, TOTP enrollment, challenged login, recovery replay rejection,
   MFA disablement, session revocation, rejected-token, and QML smoke.

Five game integration tests prove atomic exact-version initialization and
commands, ordered participants, semantic replay, isolated idempotency and
revision conflicts, rollback silence, minimal participant sync events, bounded
participant-private reads, response allowlists, version preservation after
registry changes, indistinguishable foreign and absent sessions, monotonic
timestamps, and one winner when two commands race at one revision. Two local
router tests separately prove stable catalog order, the empty production
contract, and the pre-database command body cap.

The three persona integration tests use multiple accounts to prove response
allowlists, owner-only inventory and mutation, indistinguishable foreign and
absent objects, handle uniqueness, input validation, and preservation after
rejected writes.

Five connection integration tests prove directional idempotent requests and
private response shapes, hard incoming/outgoing pending limits under boundary
races, participant-only acceptance and removal, private directional blocks that
atomically win request races, and serialization of opposite requests and
concurrent acceptance. Five inbox tests prove transition-only conversation
creation, typed body-only messages, private monotonic unread state,
conversation-local order, bounded durable history, lifecycle send denial,
no-store failures, and concurrent send/read behavior. Four MFA integration
tests prove encrypted pending enrollment, confirmation and
status privacy, TOTP/recovery/challenge replay resistance, account inactivity,
independent bounded challenge issuance, cross-challenge attempt locks, and
dual-proof disablement with cleanup. Together with two registration and three
session tests plus three persona tests, five game tests, and six synchronization
tests, these make thirty-three PostgreSQL-backed tests. The synchronization cases
exercise durable baseline/incremental/reset behavior, mutation-coupled event
delivery, owner privacy, and real-TCP WebSocket authentication, hinting, frame
bounds, quotas, permit release, lag recovery, and no-touch session lifecycle
checks. Unit tests cover cursor continuity and quota accounting; smoke owns the
minimum REST recovery path.

A later gated edit makes the delivery receipt stale. Run the diff gate after
the last code, migration, client, script, Codex configuration, skill, or
generated wiki edit. Cargo commands run sequentially; do not terminate another
Cargo process to make the gate proceed.

## Pipeline-tool provenance

`scripts/setup-pipeline-tools.sh` installs CodeGraph and a Codex-only OpenWiki
build under ignored `.dev/pipeline-tools` state. It downloads the exact
CodeGraph wrapper and current Linux architecture tarballs, verifies
repository-reviewed SHA-512 values before installation, disables lifecycle
scripts, requires exactly the reviewed package pair and executable link, checks
a hardcoded relative package-tree digest, and records installation provenance.
OpenWiki is separately pinned to a reviewed commit. Setup verifies the upstream
`packageManager` integrity, downloads that exact pnpm tarball, checks its
SHA-512 digest, installs the OpenWiki dependency graph from `pnpm-lock.yaml`
with `--frozen-lockfile` and install scripts disabled, applies the reviewed
Codex-only source patch, builds the project, and records hashes for the pnpm
tree, lock, patch, and distribution tree.

`scripts/check-pipeline-tools.sh` compares both live installations with their
reviewed identities and receipts. It rejects an unexpected CodeGraph package
set, tree digest, executable link, or provenance field and rejects unexpected
OpenWiki versions, npm lock state, source changes, or build drift.
`scripts/mcp-openwiki.sh` runs the same readiness check before starting the MCP
server, so absent or stale provenance fails closed.

## Failure routing

- Startup failure: inspect `.dev/server.log` and PostgreSQL health.
- HTTP 503: verify the database and the `SELECT 1` path.
- Registration 422: compare the request with `docs/api.md`; registration 409
  means the canonical username is already stored.
- Login 401: credentials or account status failed generically. Authenticated
  API 401: the Bearer token is malformed, expired, idle, revoked, or belongs to
  an inactive account. Session revocation 404 also covers foreign IDs.
- Persona 422: compare profile bounds or ensure an edit contains an allowlisted
  field. Persona 409 means the canonical handle is already used. Persona 404
  covers invalid, absent, and foreign IDs; public lookup also uses it for an
  invalid or absent exact handle.
- Connection 404 distinguishes a missing owned actor or incoming request without
  revealing foreign ownership. Connection 409 means the target is unavailable,
  either pending direction has reached 100 entries, the reverse request is
  already pending, or the pair is already accepted.
  Block inventories are private; use `connection_api_tests.rs` before changing
  error equivalence, idempotent deletes, pair locking, or response fields.
- Inbox 404 covers invalid, absent, and foreign conversations or messages
  without exposing participant state. Inbox 409 denies sends after disconnect
  or either-direction block. Inbox 422 covers body and pagination bounds. Use
  `inbox_api_tests.rs` before changing tagged message shapes, local sequences,
  unread cursors, social lock order, history retention, or no-store handling.
- Game-session 404 covers absent, malformed, and non-participant sessions;
  persona ownership failures use `persona_not_found`, while invalid limits use
  422. Command rejection and malformed command input use 422; revision,
  idempotency, and unavailable-version conflicts use 409 without returning the
  current revision. Use the runtime unit tests and `game_api_tests.rs` before
  changing manifests, exact-version initialization or transitions, transaction
  ownership, participant locks, replay identity, response fields, or sync event
  privacy.
- Sync 404 covers an absent or foreign acting persona; 422 covers malformed
  cursors or bounds, while `reset_required` means retained continuity cannot be
  proven. WebSocket 429 means a persona, account, or process socket quota is
  full; `resync_required` means return to REST. Use `sync_api_tests.rs` before
  changing event privacy, cursor semantics, hint delivery, or session checks.
- MFA 409: enrollment state conflicts with the requested operation. MFA 401:
  the password, factor, or challenge failed. MFA 429: factor attempts are
  temporarily locked or ten live challenges already exist. Use
  `mfa_api_tests.rs` before changing those distinctions.
- QML protocol error: inspect the health JSON contract.
- Pipeline structure failure: repair the ticket/spec/AAR/skill or Codex wiring
  named by `scripts/check-pipeline.sh`.
- Pipeline-tool readiness failure: rerun `scripts/setup-pipeline-tools.sh` only
  after reviewing any changed upstream pin, integrity, lock, or patch.
- Commit denial: finish the active pipeline and rerun the diff gate after the
  last gated edit.
