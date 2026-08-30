---
type: "Reference"
title: "Development and validation"
openwiki_generated: true
sources:
  - id: openwiki-source-0bb8016edf4f4744d3a09cf4
    resource: repo://bin/gate.sh
  - id: openwiki-source-cfb5585994628fc6aaff1dd4
    resource: repo://client/qml/cartridge/nodes/TrustedImageNode.qml
  - id: openwiki-source-fc035ef77d2451c6e8138211
    resource: repo://client/qml/tests/fixture/tst_accessibility.qml
  - id: openwiki-source-77962cc0ed2673a227f6eaee
    resource: repo://client/qml/tests/fixture/tst_transport.qml
  - id: openwiki-source-3156e0b1532bb1d02a0118e1
    resource: repo://client/qml/tests/live/tst_live_onboarding.qml
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
  - id: openwiki-source-ba452807898e03f1e2e27204
    resource: repo://crates/marketplace-publisher/tests/publication.rs
  - id: openwiki-source-24c51fe062f01ef4523fa0b7
    resource: repo://crates/server-module-runtime/tests/conformance.rs
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
  - id: openwiki-source-1b621f94587f7516bb90c07a
    resource: repo://crates/server/src/server_discovery_api_tests.rs
  - id: openwiki-source-286b9fd128ca0b68cd7c1f30
    resource: repo://crates/server/src/server_module_custom_tests.rs
  - id: openwiki-source-76060b846b9222af2c790243
    resource: repo://crates/server/src/signal_siege_api_tests.rs
  - id: openwiki-source-46fb4135d6a71efad1062c0d
    resource: repo://crates/server/src/sync_api_tests.rs
  - id: openwiki-source-617c314455b6ad7778b62ccf
    resource: repo://crates/server/tests/operator_cli.rs
  - id: openwiki-source-6ef5cb9ff978eb09c62cd313
    resource: repo://scripts/build-client-package.sh
  - id: openwiki-source-1951c64828cbf175c78556c4
    resource: repo://scripts/check-client-package-source.sh
  - id: openwiki-source-b4c3a622bb0ce5add91c5513
    resource: repo://scripts/check-local-only-automation.sh
  - id: openwiki-source-d35448de763d92d5820dbaad
    resource: repo://scripts/check-pipeline-tools.sh
  - id: openwiki-source-f30a02c87f1e4ddc4bad65fa
    resource: repo://scripts/check-qml-style.py
  - id: openwiki-source-a5928e7ee39885995efdc170
    resource: repo://scripts/dev.sh
  - id: openwiki-source-ff3f60a113327d3006289ed7
    resource: repo://scripts/mcp-openwiki.sh
  - id: openwiki-source-037d6d04880b10f227f0ac17
    resource: repo://scripts/setup-pipeline-tools.sh
  - id: openwiki-source-b88b0812532ef24df7a88f1e
    resource: repo://scripts/test-client-package.sh
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
  - id: openwiki-source-e8e6f7d2dadb4ddb710ef9c6
    resource: repo://scripts/test-marketplace-publication.sh
  - id: openwiki-source-e08dc6155c081d7928029e27
    resource: repo://scripts/test-operator-recovery.sh
  - id: openwiki-source-a0a026a4d434d1b48884aa8e
    resource: repo://scripts/test-private-alpha.sh
  - id: openwiki-source-513cfb82a80f03b4b9a1484e
    resource: repo://scripts/test-provider-conformance.sh
  - id: openwiki-source-121d7623408fcbcd07e6d9fc
    resource: repo://scripts/test-qml-onboarding.sh
  - id: openwiki-source-8128bd5b86e858053bc20c68
    resource: repo://scripts/test-server-module-spike.sh
  - id: openwiki-source-5f564ae64057cbe621fc587a
    resource: repo://scripts/test-server-modules.sh
generated: {by: "codex", at: "2026-08-30T00:13:04.632Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-30T00:13:04.632Z
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
`omarchy-gaming-system`, then requires the exact public discovery service,
UUID, protocol 1, development server name, and implemented capability set. It
also requires newly issued bearer tokens to start with `ogs1_` before
exercising authenticated operations.

In smoke mode the script first requires public `GET /v1/games` to return
exactly Signal Siege v1 and v2. It uses the real local operator executable to
issue a one-use invitation, creates a uniquely named account through
`POST /v1/accounts`, verifies the success response omits password- and
invitation-derived data, and repeats the exact request to recover the same
receipt with HTTP 200.
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
message, and the first persona reads ascending history and clears its unread
state. The first persona next signs in through the production QML controllers,
loads the accepted peer and conversation, sends a private reply, and the shell
verifies both the committed message and one payload-minimal conversation
invalidation. Removal must preserve that history while rejecting another send.
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

The QML client starts from a direct or saved server origin and requires an exact
OmarchyGS discovery document, protocol-1 onboarding capabilities, and any
remembered UUID pin before exposing registration or sign-in. It then supports
password or MFA authentication, owned-persona loading, persona creation or
selection, and an authenticated home. The authenticated shell also exposes
keyboard-first social, inbox, games, challenges, and gameplay routes for exact-
handle connection requests, request/connection/private-block actions,
persona reporting, conversation/history paging, plain-text message send, unread
acknowledgement, compiled catalog/session history, challenge lifecycle, and
authoritative Signal Siege actions. The standalone `Main.qml` smoke forces the
offscreen software backend, exits after reaching the access screen, and fails
after a fifteen-second watchdog.

`scripts/test-qml-onboarding.sh` is the focused client entrypoint. It owns a
mode-0700 test configuration directory, forces deterministic headless Qt, and
runs the real screens and controller against two compatible servers plus
changed-identity, incompatible, malformed, wrong-service, slow, and oversized
fixture responders. Before the main fixture, separate QML writer and reader
processes prove two public profiles survive process restart in the isolated
configuration location. Before Qt starts, it runs
`scripts/check-qml-style.py`: the production visual policy centralizes six-digit
colors in `OgsTheme`, requires every visual `Text` block to select
`Text.PlainText`, rejects automatic/rich text modes, and verifies the shared
theme contract. The full Qt corpus covers contrast, semantic headings and
status, deterministic initial focus and reversible traversal, settled deferred
focus before input, Escape authority, persistent keyboard and pointer exit,
session preservation on window close, keyboard behavior, field bounds,
endpoint admission, exact discovery and capability negotiation, identity
replacement, hostile profile state, authority isolation, conflicts, timeouts,
response limits, request supersession, MFA terminal and local expiry, social inventories
and actions, retry-safe report submission and hostile report receipts, private
message history, pagination, send/read, plain-text rendering, game discovery
and challenge lifecycle, authoritative solo/versus commands, exact retry
identity, revision refetch, hostile game-envelope
rejection, invalid-session cleanup, masked invitation entry and clearing,
marketplace enrollment/synchronization, key rotation and revocation, package
inventory/staging, copy-only install text, and fixture-observed request
contracts.
Social and game tests run the production root at the
640×420 minimum and reject extra private fields, oversized responses, and
body-bearing requests to bodyless mutation endpoints.
Temporary configuration containing credentials is mode-0600, not passed on the
command line, and removed after each run.

The full development smoke additionally runs the QML controllers against the
real migrated Rust API four times: registration/password login/persona
creation; selected-persona social inventory, player reporting, and private
history/send; an MFA
recovery-code challenge with owned-persona selection; and two independent game
authorities that challenge, accept, alternate Signal Siege v2 turns, complete,
and recover the exact terminal revision and state through a fresh controller.
Each scenario proves local logout and authority cleanup. Live values cross into
QML through NUL-delimited standard input and a locked mode-0600 short-lived JSON
file, never command-line arguments.

Useful commands:

```bash
./scripts/dev.sh
./scripts/dev.sh --smoke-test
docker compose down
```

## Native client package

`scripts/build-client-package.sh` builds the player-device client as
`omarchy-gaming-system-client-0.1.0-1-x86_64.pkg.tar.zst` plus a SHA-256 sidecar.
It installs nothing and does not change the system package database. Before
`makepkg`, `scripts/check-client-package-source.sh` requires a safe, sorted,
unique, newline-terminated manifest that exactly matches the non-test
production QML tree. It also requires the native companion and shared cartridge
and marketplace-trust Rust sources, rejects symlink and non-regular inputs,
binds both manual-key and packaged-channel launcher plumbing, and checks version
drift, launcher Bash, and the desktop entry.

The builder computes source-revision, dirty-state, and aggregate-digest
provenance and serializes `makepkg` through a private, owner-checked stable
workspace so identical source on one Omarchy build host produces identical
package bytes. An optional absolute non-symlink public-channel bootstrap is
copied once into private build-owned storage, verified there, and included in
both source and installed build provenance so later caller-path mutation cannot
change the authenticated input. The artifact installs the exact 40-file QML
inventory, the native
`omarchygs-client-cartridge-runtime` companion, `/usr/bin/omarchygs`, one
application-menu entry, and the provenance record. It excludes the Rust game
server, PostgreSQL, migrations, test fixtures, provider code, credentials, and
private marketplace keys.

`scripts/test-client-package.sh` is the focused artifact entrypoint. It rejects
missing, extra, duplicate, traversal, unsorted, unterminated, and symlink
source fixtures; builds twice without changing Git status; compares the
packages byte-for-byte; and checks exact Arch metadata, payload, types, modes,
provenance, checksum, and desktop fields. It rejects a symlinked independent
trust key before runtime state is created. It also builds an optional channel
package from a verified bootstrap snapshot, proves later source-path mutation
cannot alter the packaged bytes, and rejects noncanonical bootstrap tampering.
It then extracts the artifact without
`pacman -U` and launches packaged `Main.qml` plus the real native companion
through the relocatable launcher against the bounded loopback discovery fixture
under deterministic offscreen Qt. The smoke proves private cache creation and
that companion runtime state is removed on exit.

Useful commands:

```bash
./scripts/check-client-package-source.sh
./scripts/build-client-package.sh
./scripts/test-client-package.sh
```

See `docs/client-installation.md` for inspection, `pacman -U` installation and
upgrade, launch, and `pacman -Rns` removal. A normal artifact is unsigned
private-alpha output; a neighboring checksum alone is integrity evidence, not
publisher authentication. A reviewed channel build can later authenticate and
stage an exact root-signed package record, but installation remains a separate
human `pacman -U` operation.

## Canonical gate

`bin/gate.sh --fast` runs the static development loop without writing a receipt.
It includes native client package source admission and the root-signed
marketplace trust-channel proof at stage 15a plus the deterministic static
publication and offline-root drill at stage 15b. Stage 23 runs the isolated
server-module architecture proof and stage 24 runs production server-module
conformance in both fast and diff modes.
`bin/gate.sh --diff` adds the full native artifact and cartridge-acquisition conformance, isolated
migrated PostgreSQL tests, and
the live PostgreSQL → Rust game-catalog/health/account/session/persona/
social/report/inbox/challenge/sync/MFA API → QML smoke, provider security
conformance, the Door Legends authority pilot, and the isolated platform
operator recovery and private-alpha admission drills, then writes a receipt for the exact gated worktree at
`.git/omarchy-gaming-system-gate-receipt`.

Quality and delivery automation is local-only. Pipeline validation rejects
hosted CI/CD definitions, including GitHub Actions workflows. The canonical
local diff gate and its matching worktree receipt are the delivery proof.

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
7. the native client source contract in every mode, deterministic
   root/channel/bootstrap generation and transition tests, plus reproducible
   manual and channel Arch builds, exact QML/companion package inspection,
   hostile trust denial, private-cache creation, cleanup, and extracted
   production-QML smoke in diff/full modes;
8. the historical migration-0023 upgrade regression followed by the complete
   ignored server test inventory against SQLx-managed PostgreSQL databases in
   diff/full modes, including exact cartridge distribution, key rotation,
   no-fallback, lifecycle, and concurrency cases;
9. the live Signal Siege catalog, idempotent launch, bounded completed match,
   final replay/history/sync, health, registration, duplicate-conflict,
   session creation/list, persona creation/list/public lookup/edit/handle
   movement, connection, fail-closed unavailable-game challenge rejection,
   private inbox, player reporting, synchronization recovery, and block
   lifecycle, TOTP enrollment, challenged login, recovery replay rejection, MFA disablement,
   session revocation, rejected-token, the two-process profile proof and full
   hostile/accessibility QML fixture corpus, including catalog-only
   compatibility, cartridge trust/mount behavior, channel enrollment, and
   package staging,
   real QML registration/persona, social/report/inbox, MFA/persona, and two-authority
   Signal Siege challenge/versus/recovery flows, and the standalone QML shell
   smoke;
10. the production provider boundary's operator registry, lifecycle,
   grants, fixed signed messages, public-only pinned HTTPS egress, replay and
   callback deduplication, quotas, concurrency leases, audit, and fail-closed
   behavior against migrated PostgreSQL and a separate TLS provider process;
11. the first-party Door Legends authority pilot built from a clean clone,
    running through the real player-server bridge against an independent
    provider database, with replay, revision races, callbacks, projection,
    outage/restart/reconciliation, lifecycle, privacy, and backup/restore proof.
12. the database-local operator boundary's report inventory and action tests,
    real CLI adapter test, and isolated full-schema platform dump/restore drill,
    including immutable audit/report checks and restored old-token denial.
13. the invite-only private-alpha boundary's issue, first-use registration,
    exact replay, changed-intent denial, sign-in, revocation, secret-free
    inventory, digest-only persistence, and log-secret hygiene.
14. the static marketplace publisher's canonical plan and handoff contracts,
    deterministic double builds, network-unshared offline signing, exact
    immutable tree and atomic activation, guarded identical TLS mirrors,
    rotation/revocation, and stale-publication rollback denial.
15. the isolated server-module nested workspace's format, lint, 21-test corpus,
    deterministic exact-WIT component fixtures, 13 contained process scenarios,
    typed-intent/state/lifecycle checks, and local-only automation enforcement;
16. the production server-module crate and packaged host's shared reviewed/
    custom exact release/WIT/framing contract, real OS containment, fixed
    sibling loader, private database-custodied custom artifacts, local-only
    import/lifecycle boundary, absence of public routes and host network/
    database clients, plus migrated observation, gap, receipt, readiness-race,
    upgrade/rollback/removal, state/lifecycle, disclosure, and restore evidence.

### Platform operator recovery

`scripts/test-operator-recovery.sh` is gate 21. It creates only two validated,
generated databases, applies the complete embedded migration set through the
production server, seeds representative identity, social, inbox, game, sync,
report, and session state, and drives the real `omarchygs-admin` executable to
suspend an account and resolve a report. It writes a private custom-format
PostgreSQL dump, restores into the isolated target with `--exit-on-error` and
`--no-owner`, and compares every public application-table count.

The restored database must retain the source server UUID, account suspension,
session revocation, report disposition, linked immutable audit, and
representative platform history. Before any restored startup, the drill runs a
mode-0600 `module-restore` command through the real administrator, requires the
copied active module to become disabled and restore-review-blocked, then starts
the production server on loopback with module configuration still present. The
core must become healthy without reactivating the module, preserve its source
UUID, and reject the pre-suspension raw token with `invalid_session`. Cleanup
drops only the two exact validated database names;
the ordinary development database is untouched. See
`docs/operators/operator-safety-and-recovery.md` for production key custody,
backup protection, restore review, and current limitations.

### Private-alpha admission

`scripts/test-private-alpha.sh` is gate 22. It creates a fresh isolated
database, issues two invitations through the real local `omarchygs-admin`
executable, and starts the production server after applying the complete
embedded migration set.
Its bounded startup wait allows up to 30 seconds for a cold migration path and
fails immediately if the server process exits.

The drill consumes the first invitation, requires an exact canonical replay to
recover the original receipt, denies changed username or password intent, and
proves ordinary device sign-in. It revokes the second invitation before use and
requires the same generic denial. The final evidence checks exact used and
revoked inventory without raw codes or credential fields, 32-byte digest-only
persistence, linked operator audit rows, and server logs free of both invitation
codes and the submitted password. This is software-readiness evidence; the
operator still must run the human event in `docs/operators/private-alpha.md`.

### Production server-module conformance

`scripts/test-server-modules.sh` is gate 24. It runs formatting, warning-denied
lint, unit/integration conformance, and warning-denied rustdoc for the normal
workspace module runtime, builds the packaged sibling host, and executes its
ignored real-process case under the production systemd-user, Bubblewrap,
prlimit, Wasmtime memory/fuel, and parent-deadline boundary. The script also
asserts that production uses only the packaged sibling, accepts no
environment-selected component/release/WIT/URL/host path, gives the host no
network/database client dependency, retains custom components through private
database custody, exposes custom import and lifecycle only through the local
administrator, adds no public module route, and preserves local-only quality
automation.

`scripts/test-database.sh` owns the durable adapter evidence. Five reviewed-base
server-module cases cover atomic private report emission, core reauthorization,
receipt replay and retained request evidence after pruning, bounded failures and
circuit degradation, fail-open gap accounting, readiness races,
lifecycle/state CAS and rollback, restore, and honest legacy receipt semantics.
Five custom-module cases add publisher/provenance and immutable custody,
idempotent import, the eight-identity ceiling, exact expected-revision
lifecycle, atomic upgrade and one-step rollback, terminal removal, restore
review, concurrent CAS, shared dispatch, private receipts, and stale-admission
denial. The real administrator CLI suite exercises safe inventory, private
lifecycle/restore commands, and actual custom import/replay/contained enable.
Discovery API and the 55-case QML fixture add aggregate privacy, hostile shape,
server-identity binding, profile persistence, persistent accessibility warning,
and compact-layout evidence.

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
entrypoint and gate 19 in diff/full modes. It runs provider unit and public
protocol tests, then serializes the ignored operator CLI, PostgreSQL registry,
separate-process TLS egress, and end-to-end broker conformance cases against the
migrated database. The corpus covers immutable release registration and key
rotation; lifecycle denial; 60-second one-scope pairwise grants; exact request,
response, and callback authentication; public-only resolution and socket
pinning; strict body/time limits; idempotent replay and concurrent callback
deduplication; quota and lease races; retry-after-unknown behavior; and safe
audit records.

Gate 19 proves the reusable provider security/control-plane boundary. The
player server instantiates that crate only when its all-or-none provider
configuration is present; gate 20 owns the separately reviewed player-route
and authority proof.

### First-party remote-provider authority pilot

`scripts/test-provider-authority-pilot.sh` is the Ticket 019 entrypoint and gate
20 in diff/full modes. It packages the public provider protocol, copies the
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
body cap. Signal Siege adds ten deterministic v1/v2 rule tests, one local exact
production-catalog/solo-body-limit test, and four PostgreSQL cases covering
owner scope, exact replay, registry drift, final-slot cap concurrency,
completion, final replay, no bot identity, privacy-minimal recovery, and
rollback. The challenge suite owns the production two-human alternation and
terminal-result database case.

Seven challenge integration tests prove participant-private creation and reads,
exact idempotent replay and collision handling, connected/block policy,
incoming/outgoing caps, typed lifecycle messages, payload-minimal sync events,
terminal history and lazy expiry, exact-version session creation and seat
order, initializer rollback, production Signal Siege v2 alternation and
completion, and one winner under competing transitions. One local router test
separately proves the pre-database challenge body cap.

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
dual-proof disablement with cleanup. Together with two account-registration
router cases, three invitation-registration cases, three session tests, three
persona tests, five game tests, seven challenge tests, six synchronization
tests, four Signal Siege cases, one Door Legends authority case, two report
cases, two server-discovery cases, five cartridge-catalog cases, and five
reviewed-base production server-module cases, these make sixty-three
PostgreSQL-backed server binary tests. Eight additional library integration
tests cover marketplace synchronization/key rotation, operator-custom
cartridge admission, and five operator-custom module cases. Five
operator-domain database tests cover the local queues, account containment,
invitation lifecycle, report disposition, concurrency, replay, and append-only
audit; six integration tests execute the real CLI adapters, including reviewed
and custom module commands. The complete database stage therefore runs 82
PostgreSQL-backed cases sequentially.
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

The native package smoke copies only declared source trees and excludes nested
generated `target/` directories before extracting fixtures. This keeps a nested
workspace build from recursively entering later package fixtures while still
preserving the exact source-contract checks.

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
- Server-module proof failure: run `scripts/test-server-module-spike.sh`, then
  separate contract/runtime/state failures from supervisor process outcomes.
  A missing Bubblewrap or unsupported user-scope containment is an environment
  failure; a trap, resource abuse, crash, or timeout must still produce its
  bounded stable outcome and permit a clean restart.
- Production module failure: run `scripts/test-server-modules.sh`, then separate
  signed-contract/runtime failures from packaged-host containment, private
  custody, local-command, and fixed-loader assertions. For custom import,
  lifecycle, disclosure, durable emission, gaps, receipts, readiness races,
  state, or restore failures, run the ignored `server_module_tests` and
  `server_module_custom_tests` serially through `scripts/test-database.sh` plus
  the QML fixture when discovery changed; do not bypass gates 17, 18, 21, or 24.
- Cartridge preview rejection: read the machine-readable preview error code;
  verify signature/compatibility, pinned view schema, exact action shape,
  private empty output directory, and selected Core/Rich-2D budget. Run
  `scripts/test-game-cartridge-renderer.sh` for the full trusted handoff.
- HTTP 503: verify the database and the `SELECT 1` path.
- Registration 422: compare the request with `docs/api.md`; registration 403
  means the invitation is malformed, absent, expired, revoked, already used by
  different credentials, or lost a concurrent consumption race. Registration
  409 means a valid unused invitation reached a canonical username that is
  already stored; the invitation remains usable. Diagnose the full lifecycle
  with `scripts/test-private-alpha.sh` without logging the raw code.
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
- QML client failure: run `scripts/test-qml-onboarding.sh`; inspect the
  selected endpoint, exact discovery identity and capabilities, saved UUID
  expectation, profile parser, response-size/timeout/redirect outcome, exact
  success or error shape, selected-persona gateway, game
  participant/cardinality and state invariants, and whether Bearer, MFA, social,
  or game authority was cleared on the terminal path. Use
  `scripts/dev.sh --smoke-test` when the fixture passes but the real migrated
  API flow fails.
- Native client package failure: run `scripts/check-client-package-source.sh`
  first, then `scripts/test-client-package.sh`. Repair the named source
  manifest, version, launcher, desktop, reproducibility, archive, provenance,
  or extracted-QML contract; do not bypass gates 15 or 16 and do not install an
  uninspected artifact.
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
  gate 19.
- Provider-authority failure: run `scripts/test-provider-authority-pilot.sh`
  and fix the named clean-clone dependency, authority shape, player route,
  replay/revision race, callback projection, lifecycle, independent database,
  reconciliation, restart, or restore failure; do not bypass gate 20 or widen
  the pilot to another provider.
- Pipeline structure failure: repair the ticket/spec/AAR/skill or Codex wiring
  named by `scripts/check-pipeline.sh`.
- Pipeline-tool readiness failure: rerun `scripts/setup-pipeline-tools.sh` only
  after reviewing any changed upstream pin, integrity, lock, or patch.
- Commit denial: finish the active pipeline and rerun the diff gate after the
  last gated edit.
