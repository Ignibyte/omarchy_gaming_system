---
type: "Reference"
title: "Development and validation"
openwiki_generated: true
sources:
  - id: openwiki-source-0bb8016edf4f4744d3a09cf4
    resource: repo://bin/gate.sh
  - id: openwiki-source-cfb5585994628fc6aaff1dd4
    resource: repo://client/qml/cartridge/nodes/TrustedImageNode.qml
  - id: openwiki-source-d392f8f0962c50f0d66e0629
    resource: repo://client/qml/Main.qml
  - id: openwiki-source-937883bc0b4873d5f0200c46
    resource: repo://CONSTITUTION.md
  - id: openwiki-source-fdf115002c4aabad0babec70
    resource: repo://crates/game-cartridge-renderer/src/lib.rs
  - id: openwiki-source-305772806daa653bb2bc0a61
    resource: repo://crates/game-cartridge-renderer/tests/rendering.rs
  - id: openwiki-source-9eb807576928ef92a7b8b32a
    resource: repo://crates/game-cartridge/tests/conformance.rs
  - id: openwiki-source-358b091c74e2027615ce8f4c
    resource: repo://crates/game-cartridge/tests/sdk_release.rs
  - id: openwiki-source-fea3ada71e31ee06122151f5
    resource: repo://crates/game-provider/tests/conformance.rs
  - id: openwiki-source-522c1bcb889a85d7a91b25af
    resource: repo://crates/game-provider/tests/registry.rs
  - id: openwiki-source-df8490db5b51be8096630e7e
    resource: repo://crates/game-signal-siege/src/lib.rs
  - id: openwiki-source-2c054a2481343f8aacaf65ae
    resource: repo://crates/server/src/challenge_api_tests.rs
  - id: openwiki-source-9ba5739252220892895a7a47
    resource: repo://crates/server/src/connection_api_tests.rs
  - id: openwiki-source-a243b385d49ea9224173d77a
    resource: repo://crates/server/src/game_api_tests.rs
  - id: openwiki-source-b2c7af59f511c4ed8a004fb0
    resource: repo://crates/server/src/inbox_api_tests.rs
  - id: openwiki-source-22753602a862c32d10560204
    resource: repo://crates/server/src/persona_api_tests.rs
  - id: openwiki-source-76060b846b9222af2c790243
    resource: repo://crates/server/src/signal_siege_api_tests.rs
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
  - id: openwiki-source-d69dbacb0ae7fe382ee46161
    resource: repo://scripts/test-game-cartridge-renderer.sh
  - id: openwiki-source-8df9ad1a3495f8360740ff03
    resource: repo://scripts/test-game-cartridge-sdk.sh
  - id: openwiki-source-4e51428e90d3c7db3949b09b
    resource: repo://scripts/test-game-cartridge-spike.sh
  - id: openwiki-source-68106a790eb8acc94f8d3540
    resource: repo://scripts/test-game-cartridge.sh
  - id: openwiki-source-513cfb82a80f03b4b9a1484e
    resource: repo://scripts/test-provider-conformance.sh
generated: {by: "codex", at: "2026-08-25T22:05:16.359Z"}
---

# Development and validation

## Run the vertical slice

`scripts/dev.sh` is the local orchestration entrypoint. It verifies Docker,
mise, QML, curl, jq, OpenSSL, Python, and `cmp`; starts PostgreSQL; launches the
Rust server; requires that exact child to remain alive and emit its listening
log before accepting health; and opens the QML client.
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

In smoke mode the script first requires public `GET /v1/games` to return
exactly Signal Siege v1. It then creates a uniquely named account
through `POST /v1/accounts`, verifies the success response omits password-
derived data, and repeats the request to require `username_taken` with HTTP 409.
It then creates a device session and verifies the authenticated inventory.
Before revocation it creates a persona, proves the exact public field set and
private-field absence, checks owned inventory and public lookup, edits the
persona, and proves the old handle disappears while the new handle resolves the
updated profile. Before creating the social peer, it starts Signal Siege for
the owned persona, proves exact idempotent launch and payload-minimal sync,
plays bounded integer-valued revisions to a terminal outcome, replays the final
command exactly, recovers completed detail and inventory, verifies minimal
command invalidations, and rejects a new post-completion command. It creates a
second account and persona, then exercises
request, incoming inventory, acceptance, and mutual inventory. Acceptance must
also expose one private conversation with a typed system message. While the pair
is connected, the smoke submits an unregistered game challenge and requires
`game_unavailable` with no partial cursor event. The peer then sends a user
message, the first persona reads ascending
history and clears its unread state. Removal must preserve that history while
rejecting another send.
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
PostgreSQL → Rust game-catalog/health/account/session/persona/social/inbox/
challenge/sync/MFA API → QML smoke, provider security conformance, and the
Door Legends authority pilot, then writes a receipt for the exact gated
worktree at
`.git/omarchy-gaming-system-gate-receipt`.

The gate currently covers:

1. production-workspace Rust formatting, warning-denied Clippy, tests, and
   warning-denied rustdoc;
2. Compose, shell, pipeline, secret, hook, and whitespace checks;
3. the production Game Cartridge's twenty-test hostile conformance corpus,
   deterministic CLI pack/conform/install/revoke lifecycle, and isolation
   assertions;
4. the trusted production renderer's two unit and nine integration tests,
   signed Core/Rich-2D preparation, private output, QML state/input/accessibility
   smoke, aggregate-plan rejection, raster admission, and frame/RSS profile
   enforcement;
5. deterministic SDK export, two clean-clone first-party builds, byte-identical
   signed release verification, signed five-state catalog policy, secure local
   import, and permission/rollback/concurrency regressions;
6. the isolated Game Cartridge workspace format, Clippy, tests, binaries,
   rustdoc, signed package, broker/provider/probe exchange, privacy assertions,
   trusted-QML smoke, and frame/memory/package measurements;
7. forty-four ignored router tests against SQLx-managed PostgreSQL databases
   in diff/full modes; and
8. the live Signal Siege catalog, idempotent launch, bounded completed match,
   final replay/history/sync, health, registration, duplicate-conflict,
   session creation/list, persona creation/list/public lookup/edit/handle
   movement, connection, fail-closed unavailable-game challenge rejection,
   private inbox, synchronization recovery, and block lifecycle, TOTP
   enrollment, challenged login, recovery replay rejection, MFA disablement,
   session revocation, rejected-token, and QML smoke.
9. the production provider boundary's operator registry, lifecycle,
   grants, fixed signed messages, public-only pinned HTTPS egress, replay and
   callback deduplication, quotas, concurrency leases, audit, and fail-closed
   behavior against migrated PostgreSQL and a separate TLS provider process.
10. the first-party Door Legends authority pilot built from a clean clone,
    running through the real player-server bridge against an independent
    provider database, with replay, revision races, callbacks, projection,
    outage/restart/reconciliation, lifecycle, privacy, and backup/restore proof.

### Production Game Cartridge conformance

`scripts/test-game-cartridge.sh` is the focused entrypoint for the production
data-only v1 contract. It runs twenty tests spanning canonical identity,
signature and content tampering, archive/path/resource attacks, strict schema
and media handling, node-to-capability binding, compatibility fallbacks,
bounded regular-file input, content-addressed installation, and fail-closed
revocation.

The script then packs the same fixture after changing source mtimes and modes
and requires byte-identical output. It conforms under unusable network,
database, and credential environment values, installs only the exact read-only
archive, and denies resolution after revocation. This is gate 11. It validates
inert packaging and local storage, not production rendering or provider access.

### Trusted Game Cartridge renderer

`scripts/test-game-cartridge-renderer.sh` is the focused production renderer
entrypoint and gate 12. It runs the renderer unit/integration corpus, builds the
production package and preview CLIs, creates ephemeral signing keys, and packs
real base, Core, and Rich-2D cartridges. Preview runs with deliberately unusable
database, device-token, and proxy environment values and must report that it
contacted no provider, needed no database, and read no platform credential.

The QML matrix uses Qt's offscreen software backend at 920×600 and one CPU when
affinity is available. It warms 60 frames, samples 120, enforces a 33.3 ms
average ceiling and the profile hard RSS cap, exercises Grid/Button focus and
actions, covers 2× scale/high contrast/reduced motion/mute, and instantiates
zero cartridge nodes for loading, offline, stale, empty, protocol-error,
unsupported-capability, and revoked states. A substituted per-node-valid Core
plan containing a particle must fail the QML aggregate-profile recount. A real
2,048-pixel Rich-2D raster must render inside the performance envelope, while a
4,096-pixel raster must fail before any prepared plan is published.

### Game Cartridge SDK and first-party release

`scripts/test-game-cartridge-sdk.sh` is the Ticket 017 production portability
entrypoint and gate 13. It runs eight release/lifecycle/secure-store tests,
exports the SDK twice and requires byte-identical files, then copies only the
public cartridge and preview binaries plus SDK and publisher key into two clean
Git clones of the first-party example repository. Both clones must produce
byte-identical read-only archives, conformance reports, and signed release
attestations bound to the source revision, builder binary, SDK lock, and exact
artifact digests.

The same gate verifies a signed five-state catalog policy, imports the release
through the descriptor-relative Linux store, compares the installed blob with
the release artifact, and prepares it through the production previewer. The
test environment supplies unusable database, provider-proxy, device-token, and
MFA-key values and rejects any platform source-tree path in the release proof.
Focused regressions cover unsafe directory permissions and symlinks, policy
rollback, denial persistence across restart, and concurrent policy versions.

### Game Cartridge architecture proof

`scripts/test-game-cartridge-spike.sh` is the focused entrypoint for the
non-production Ticket 014 proof and gate 14. It runs the nested Cargo workspace
under the shared repository target directory, creates ephemeral mode-0600 signing keys
and a temporary signed fixture, launches the provider and broker on separate
loopback-only ports, and drives the complete launch/command/replay flow with a
Rust probe. The response must explicitly prove that raw persona identity, the
device token, and database access were not disclosed.

The script then runs the trusted proof QML offscreen, rejects known runtime
contract errors, captures a 120-frame timing sample and peak resident memory,
reports expanded package size, and removes the exact child processes and
temporary material on exit. It proves architecture semantics and the
measurement harness; it is not a production provider service, a published SDK,
or a minimum-hardware Rich-2D benchmark. See [Game Cartridges](game-cartridges.md)
for the boundary and remaining production work.

### Remote-provider security foundation

`scripts/test-provider-conformance.sh` is the Ticket 018 production security
entrypoint and gate 17 in diff/full modes. It runs provider unit and public
protocol tests, then serializes the ignored operator CLI, PostgreSQL registry,
separate-process TLS egress, and end-to-end broker conformance cases against the
migrated database. The corpus covers immutable release registration and key
rotation; lifecycle denial; 60-second one-scope pairwise grants; exact request,
response, and callback authentication; public-only resolution and socket
pinning; strict body/time limits; idempotent replay and concurrent callback
deduplication; quota and lease races; retry-after-unknown behavior; and safe
audit records.

Gate 17 proves the reusable provider security/control-plane boundary. The
player server instantiates that crate only when its all-or-none provider
configuration is present; gate 18 owns the separately reviewed player-route
and authority proof.

### First-party remote-provider authority pilot

`scripts/test-provider-authority-pilot.sh` is the Ticket 019 entrypoint and gate
18 in diff/full modes. It packages the public provider protocol, copies the
Door Legends example into a fresh Git repository, clones it, and builds its TLS
provider with default platform features disabled. The script rejects a
platform-only dependency or source-tree path in the resulting binary.

The proof creates an independent provider database, runs the real server router
with an empty compiled registry plus the operator-enabled release, and verifies
authority-tagged catalog, start, command, read, result, achievement, and sync
responses. Its single PostgreSQL integration case covers exact replay,
expected-revision command races, callback tamper/deduplication/policy,
participant privacy, timeout-after-commit reconciliation, outage and process
restart, suspension, restoration, and terminal retirement. Finally it dumps
the provider database, restores it into a second database, and checks the
authoritative sessions, operation receipts, and delivered event outbox.

Five general game integration tests prove atomic exact-version initialization and
commands, ordered participants, semantic replay, isolated idempotency and
revision conflicts, rollback silence, minimal participant sync events, bounded
participant-private reads, response allowlists, version preservation after
registry changes, indistinguishable foreign and absent sessions, monotonic
timestamps, and one winner when two commands race at one revision. Two local
router tests separately prove stable catalog order and the pre-database command
body cap. Signal Siege adds five deterministic rule tests, one local exact
production-catalog/solo-body-limit test, and four PostgreSQL cases covering
owner scope, exact replay, registry drift, final-slot cap concurrency,
completion, final replay, no bot identity, privacy-minimal recovery, and
rollback.

Six challenge integration tests prove participant-private creation and reads,
exact idempotent replay and collision handling, connected/block policy,
incoming/outgoing caps, typed lifecycle messages, payload-minimal sync events,
terminal history and lazy expiry, exact-version session creation and seat
order, initializer rollback, and one winner under competing transitions. One
local router test separately proves the pre-database challenge body cap.

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
session tests plus three persona tests, five game tests, six challenge tests,
and six synchronization tests, plus four Signal Siege cases and one Door
Legends authority case, these make forty-four PostgreSQL-backed tests.
The synchronization cases
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
- Cartridge preview rejection: read the machine-readable preview error code;
  verify signature/compatibility, pinned view schema, exact action shape,
  private empty output directory, and selected Core/Rich-2D budget. Run
  `scripts/test-game-cartridge-renderer.sh` for the full trusted handoff.
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
  422. Invalid solo identity or participants use 422; unavailable versions,
  idempotency collisions, and new commands after completion use 409; the
  active-solo cap uses 429. Command rejection and malformed command input use
  422; revision conflicts use 409 without returning the current revision. Use
  the runtime and Signal Siege unit tests plus `game_api_tests.rs` and
  `signal_siege_api_tests.rs` before changing manifests, solo admission,
  exact-version initialization or lifecycle, transaction ownership,
  participant locks, replay identity, response fields, or sync privacy.
- Game-challenge 404 covers absent, malformed, and non-participant challenges;
  409 covers unavailable relationships or games, pending limits, expired
  acceptance, idempotency collisions, and invalid terminal direction/state.
  Use `challenge_api_tests.rs` before changing exact replay, participant
  privacy, pair locks, expiry, lifecycle messages, acceptance transaction
  ownership, or challenge/session/conversation invalidations.
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
- Production cartridge failure: run `scripts/test-game-cartridge.sh` and fix the
  named canonical archive, signature, schema/media, capability, bounded-input,
  store, or revocation contract; do not bypass gate 11.
- Trusted-renderer failure: run `scripts/test-game-cartridge-renderer.sh` and fix
  the named schema/action/profile/raster, QML boundary, accessibility, timing, or
  RSS failure; do not bypass gate 12.
- SDK/release/import failure: run `scripts/test-game-cartridge-sdk.sh` and fix the
  named export, release provenance, lifecycle, permission, rollback, concurrency,
  or clean-room isolation failure; do not bypass gate 13.
- Cartridge proof failure: inspect the temporary provider, broker, and QML logs
  printed by `scripts/test-game-cartridge-spike.sh`. Treat signature, identity,
  capability, privacy, replay, resource, or trusted-renderer failures as
  architecture failures; do not bypass gate 14. The binaries are loopback-only
  proof artifacts and must not be deployed.
- Provider-security failure: run `scripts/test-provider-conformance.sh` and fix
  the named registration, lifecycle, signature, egress, replay, callback,
  quota/lease, audit, TLS-process, or PostgreSQL race failure; do not bypass
  gate 17.
- Provider-authority failure: run `scripts/test-provider-authority-pilot.sh`
  and fix the named clean-clone dependency, authority shape, player route,
  replay/revision race, callback projection, lifecycle, independent database,
  reconciliation, restart, or restore failure; do not bypass gate 18 or widen
  the pilot to another provider.
- Pipeline structure failure: repair the ticket/spec/AAR/skill or Codex wiring
  named by `scripts/check-pipeline.sh`.
- Pipeline-tool readiness failure: rerun `scripts/setup-pipeline-tools.sh` only
  after reviewing any changed upstream pin, integrity, lock, or patch.
- Commit denial: finish the active pipeline and rerun the diff gate after the
  last gated edit.
