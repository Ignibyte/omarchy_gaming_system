---
title: Player cartridge acquisition, cache, and mount lifecycle — notes
pipeline_id: f7e13a3c-e3e9-4d8a-b4b0-a48cf0ef02d4
---

# Player cartridge acquisition, cache, and mount lifecycle — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 032 is delivered at
  `fc18113882ca433a4aa21ea2849faa122be5da6a`; local and remote commit/tree
  identities matched and the worktree was clean before Ticket 033 opened.
- Recall: the external two-clean-installation private-alpha acceptance event
  remains a human/machine dependency. The next independently executable
  roadmap outcome is player cartridge acquisition/cache/mount lifecycle.
- Recall: `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0,
  OpenWiki 0.3.3, and Codex-only provenance active.
- Recall: Ticket 032 deliberately returned metadata only and retained exact
  archive, publisher attestation, conformance, publisher public key, signed
  lifecycle policy, reviewed inventory, and server admission. It did not
  retain the current raw signed marketplace snapshot needed for a client to
  verify review metadata independently.
- Recall: ADR-0003 and `game-cartridges.md` require a bounded
  server-approved distribution path, separate publisher/marketplace/server
  claims, exact digest verification, a content-addressed read-only cache, and
  server-profile-scoped admission even when bytes are deduplicated.
- Recalled rules:
  `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001`,
  `PR-omarchy-gaming-system-validate-retained-directory-authority-001`,
  `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001`,
  and the Ticket 017 secure-store rule that retained descriptors—not path
  strings—must remain the filesystem authority.
- Direct client inspection found the production package is pure repository
  QML launched with `qml6`; the device bearer is process-local in
  `ApiClient.qml`, QML has no acceptable descriptor-relative cache authority,
  and the current game screen consumes only the platform `/v1/games` catalog.
- Direct server/database inspection found `marketplace_releases` already
  retains publisher keys and signed policies, while the secure store retains
  exact archive/attestation/conformance bytes. `marketplace_sync_state` retains
  only snapshot identity/digest and must add the signed current snapshot and
  marketplace public key for independent player verification.
- Decision: use a native same-user Rust companion with a random authenticated
  loopback endpoint. The trusted shell can ask it to acquire an exact release,
  but inert cartridge presentation never receives the helper credential,
  device bearer, network API, or cache paths.
- Decision: make install/update/remove player-visible in this slice but defer
  session-to-render-plan launch binding. A mount is an exact verified
  profile-scoped presentation binding ready for that later contract, not a
  false claim that arbitrary catalog games are already playable.
- Phase 1 is PASS. The thirteen EARS requirements bind one shippable trust
  boundary from the selected server's immutable store to explicit
  player-managed profile mounts without adding cartridge execution authority.

## Phase 2 — Design

- Architecture and ownership:
  - `omarchygs-game-cartridge` remains the only parser/verifier for signed
    snapshots, publisher releases, conformance, lifecycle policy, archive
    structure, compatibility, and immutable content staging. It adds a narrow
    exact-artifact export from the retained-descriptor secure store so neither
    server nor client reimplements path-based reads.
  - PostgreSQL remains the server catalog authority. Migration `0020` adds the
    exact current signed marketplace snapshot bytes and public key to the
    singleton sync record. They are nullable only for safe upgrade from a
    Ticket 032 database; an exact replay backfills them, while acquisition
    remains unavailable until evidence exists.
  - The normal server optionally owns a `CartridgeDistributionRuntime` opened
    at startup from the same checked secure-store root and marketplace public
    key used by the administrator sync path. Partial configuration is fatal;
    absent configuration is a supported metadata-only deployment and omits
    `games.cartridge-acquisition.v1`.
  - The distribution service queries one currently effective selected row,
    verifies the configured marketplace key, signed snapshot and exact review
    entry, publisher release, lifecycle policy, compatibility, and immutable
    stored bytes again, and builds a bounded `omarchygs.cartridge-acquisition/v1`
    JSON document. Base64url fields carry exact snapshot, archive,
    conformance, and release-attestation bytes; typed public keys/policy and a
    bounded server-admission record remain separate fields.
  - Axum exposes that document only at the selected server's fixed exact path
    `/v1/cartridges/{game_key}/{archive_sha256}/acquisition`, after bearer
    authentication. The path never accepts or returns a remote destination,
    redirect, local filesystem path, private key, provider endpoint, render
    document, or executable content. Existing `GET /v1/cartridges` is the
    lightweight before/after admission truth used to close the download race.
  - A new workspace crate builds `omarchygs-cartridge-companion`. Its library
    owns selected-origin transport, bundle verification, a private
    descriptor-relative cache, profile mounts, and a small loopback JSON API;
    its binary owns startup/shutdown and the protected endpoint announcement.
    The crate has no database/provider/gameplay mutation authority.
  - The launcher creates a mode-0700 runtime directory, starts the companion
    on `127.0.0.1:0`, reads a mode-0600 startup document containing the exact
    endpoint and random 256-bit credential, passes those values only to the
    repository-owned QML shell, and traps normal/error/signal exit to stop the
    child and remove the runtime directory. The companion rejects a wrong
    bearer, Host, path, method, body shape, or body size.
  - `CartridgeController.qml` owns bounded server-catalog and companion
    requests. `OnboardingController.qml` supplies one transient acquisition
    authority object only while the current authenticated selected server and
    persona are valid. The cartridge controller resets on authority/profile
    changes and never persists or displays the device bearer or companion
    credential.
  - The companion validates the canonical selected origin and stable discovery
    UUID, fetches the authenticated metadata catalog, requires the requested
    digest and revision, downloads the exact bundle without proxy or redirect,
    requires the envelope's complete marketplace key to equal one loaded from
    client-controlled local configuration, verifies every signed artifact
    through the production cartridge crate, stages immutable content, and
    fetches catalog truth again. Only an unchanged exact admission is
    atomically mounted.
  - Content lives once under a private `content/` secure store and is read-only
    with no executable bits. Mounts live under
    `profiles/<server-uuid>/mounts/<game-key>.json`, contain bounded exact
    provenance/admission identity, and are atomically replaced through retained
    directory descriptors. A process/filesystem lock serializes mount changes
    across multiple client shells. Removing one mount never mutates the server
    and initially retains unreferenced immutable bytes for safe later cache
    garbage collection; it never removes monotonic lifecycle denial evidence.
  - The Games screen keeps the platform `/v1/games` session catalog distinct
    from the selected server's signed presentation cartridges. It exposes
    explicit INSTALL, UPDATE, and REMOVE controls plus mounted, deprecated,
    unavailable, busy, and error states. Mounting is truthful preparation for
    a later session/render-plan pinning slice, not an executable or playable
    claim.
- Protocol and failure flow:
  1. Administrator sync verifies the signed marketplace document and stores
     its exact bytes/key atomically with the derived current snapshot rows.
     An equal exact replay safely fills missing upgrade evidence; conflicting
     evidence fails.
  2. On login/server selection, QML fetches effective cartridge metadata and
     asks the local companion for that server UUID's mount inventory. No
     server credential is needed to inspect or remove local mounts.
  3. On explicit install/update, QML sends the companion the selected origin,
     stable server UUID, transient device bearer, and exact catalog tuple.
  4. The companion verifies discovery and first catalog truth, then requests
     the server's exact authenticated bundle with fixed ceilings, TLS for
     remote origins, no proxy, no redirect, no content encoding, and no URL
     derived from bundle content.
  5. The companion verifies snapshot signature/current entry, marketplace
     policy, publisher release, reconstructed conformance, digests, SDK, and
     Rich-2D host compatibility, then stages read-only content without writing
     a mount.
  6. A final authenticated catalog read must contain the same exact digest and
     admission revision. Only then does one atomic descriptor-relative mount
     replace the prior record. Failure at any earlier point leaves the prior
     mount unchanged.
  7. Remove unlinks only the exact profile mount after validating the requested
     mounted digest. Shared immutable bytes and monotonic policy evidence stay
     cached; account/game/save/provider/server state is outside this process.
- File manifest:

  | Surface | Planned change |
  |---|---|
  | `migrations/0020_player_cartridge_distribution.sql` | Add bounded current signed snapshot bytes and exact public key to sync state with monotonic/evidence guards. |
  | `crates/game-cartridge/src/secure_store.rs`, exports/tests | Return exact verified retained artifacts without exposing paths or weakening descriptor containment. |
  | `crates/server/src/cartridge_catalog.rs`, `marketplace_sync.rs`, new `cartridge_distribution.rs` | Persist/replay snapshot evidence; query/reverify exact effective release; build bounded acquisition documents. |
  | `crates/server/src/config.rs`, `main.rs`, `app.rs`, `server_discovery.rs` | Optional all-or-nothing normal-runtime configuration, conditional capability/route, authenticated exact handler. |
  | Server catalog/sync/config/API/PostgreSQL tests | Snapshot backfill, exact bundle, auth/lifecycle/race/config/capability evidence. |
  | New `crates/client-cartridge-runtime/` | Remote client, strict bundle verifier, secure content/profile cache, loopback companion API/binary, and unit/integration tests. |
  | Workspace `Cargo.toml`/`Cargo.lock` and server/client crate manifests | Register production dependencies and the native companion binary. |
  | `client/qml/CartridgeController.qml`, `Main.qml`, `OnboardingController.qml`, `screens/GamesScreen.qml`, runtime manifest | Trusted explicit install/update/remove and isolated status/provenance UI. |
  | QML fixture server/tests and acquisition fixture | Hostile exact-schema, authority reset, interaction, accessibility, and live companion behavior. |
  | `packaging/arch/omarchygs`, `PKGBUILD`, source/build/test scripts | Build/install/start/stop the native companion and prove deterministic reviewed package contents. |
  | `.env.example`, `README.md`, `docs/api.md`, `docs/client-installation.md`, architecture/operator docs | Configure distribution, document trust/lifecycle/cache/mount behavior and remaining launch boundary. |
  | Active Ticket 033 artifacts, AAR, roadmap, OpenWiki | Evidence, durable lessons, completion, and knowledge reconciliation. |
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Config/startup/discovery tests cover absent, partial, invalid, symlinked, mismatched-key, and valid secure-store distribution configuration plus conditional routing/capability. |
  | REQ-002 | Migration/sync tests cover new snapshot evidence, exact replay backfill, changed-equal conflict, upgrade-null behavior, bounds, no private key, and backup/restore preservation. |
  | REQ-003 | API tests cover missing/bad/revoked session, exact body/schema/headers, effective row, immutable source, bounded response, and absence of paths/URLs/secrets/code. |
  | REQ-004 | Lifecycle/selection/omission/incompatibility transitions and a deterministic post-download admission change deny mounting without substitution. |
  | REQ-005 | Companion tests cover random endpoint/credential, Host/auth/method/body rejection, cleanup, duplicate shell serialization, and launcher/package lifecycle. |
  | REQ-006 | Remote-client corpus rejects noncanonical origins, remote HTTP, server UUID change, redirect, proxy use, wrong path/digest/revision, timeout, status, encoding, and oversized/truncated bodies. |
  | REQ-007 | Hostile bundle corpus tampers every snapshot/review/policy/publisher/conformance/archive/SDK/host/identity relation and proves only the exact production-verified release mounts. |
  | REQ-008 | Linux cache tests cover ownership/modes, symlink and rename races, descriptor containment, atomic failure, immutable conflicts, no executable bits, restart, and one-copy digest reuse. |
  | REQ-009 | Two server UUIDs sharing one digest retain independent mounts/provenance; origin/UUID replacement and cross-profile remove/update are denied. |
  | REQ-010 | QML fixtures cover empty/loading/ready/mounted/update/deprecated/unavailable/error states, keyboard focus/actions, authority reset, retry, and prior-mount preservation. |
  | REQ-011 | Local removal/reference tests assert one mount only is removed and no server mutation, credential persistence, or authoritative domain path exists. |
  | REQ-012 | Source contract and two package builds inspect exact binary/QML payload, modes, dynamic dependencies, provenance, credential absence, launcher cleanup, and extracted-package acquisition smoke. |
  | REQ-013 | Focused workspace, database, QML, cartridge, companion, and package suites followed by the complete worktree-bound diff gate. |
- Risks and mitigations:
  - Credential exfiltration: the device bearer is accepted only in one bounded
    authenticated loopback request, wrapped for zeroization, excluded from all
    logs/errors/receipts and never persisted. The per-process companion secret
    and startup file are random and private.
  - DNS/redirect abuse: the destination is the player-selected canonical
    server origin, not untrusted content; remote HTTP, ambient proxy,
    redirects, content encoding, and response-selected URLs are denied. Stable
    server UUID and exact response URL are checked over the same origin.
  - Trust conflation: the verifier checks the publisher release, current
    marketplace snapshot/review and policy, and server admission as distinct
    records and reports explicit provenance rather than one `verified` flag.
    The marketplace verification key must come from client-controlled local
    configuration; the selected server's envelope may carry only an exactly
    equal public copy.
  - Download race: exact digest/revision is checked before download and again
    after full cryptographic verification; the mount write is last and atomic.
  - Local filesystem races: private owned roots, retained descriptors,
    no-follow opens, regular-file/mode checks, per-root locking, immutable
    writes, and atomic rename keep path strings from becoming authority.
  - Cross-server confusion: mount namespaces use stable UUID, records bind the
    canonical origin and exact admission facts, and controller authority resets
    before any selected-server change.
  - Denial of service: fixed request/file/body/count/time ceilings precede
    base64, JSON, archive, conformance, and media parsing; only one mutation is
    processed at a time per companion/root.
  - False playability: UI labels distinguish platform playable games from
    installed presentation cartridges; session pinning/render-plan integration
    remains explicitly deferred.
- Alternatives rejected:
  - QML-owned filesystem installation cannot retain descriptor authority,
    securely stage immutable artifacts, or isolate untrusted presentation from
    credentials/cache paths.
  - Direct client marketplace downloads would let a server advertise one
    trust decision while sending the player to another authority/destination
    and would bypass the server's staged exact mirror.
  - Serving only `.ogsc` bytes cannot prove marketplace review/lifecycle or
    publisher release/conformance independently at the player boundary.
  - Persisting only derived review strings repeats Ticket 032's trusted-server
    metadata view but does not meet the client-verifiable marketplace claim.
  - A privileged machine-wide daemon is unnecessary for the current
    single-user Omarchy client and would add installation, IPC, upgrade, and
    local privilege boundaries beyond this slice.
  - Loading cartridge QML/JavaScript or embedding a browser remains prohibited
    because it would turn distribution into executable frontend privilege.
- Rollback: omit the two distribution environment variables and restart to
  return the normal server to metadata-only behavior; the additive nullable
  evidence columns and immutable files remain harmless. Revert the client
  package to the prior QML-only release; server-side acquisition remains an
  authenticated additive route. Unmount local records without modifying
  server catalog or authoritative player state.
- CodeGraph design evidence: the Ticket 033 receipt binds pipeline
  `f7e13a3c-e3e9-4d8a-b4b0-a48cf0ef02d4`. Exploration traced the normal
  router/AppState/discovery fan-out, authenticated `list_cartridges`, the sole
  `publish_snapshot` caller, synchronization preflight/replay, and the shared
  `SecureCartridgeStore` staging/resolution boundary. CodeGraph found the
  critical Rust blast radius in `app.rs`, `cartridge_catalog.rs`,
  `marketplace_sync.rs`, and `secure_store.rs`; QML, shell, SQL, package
  metadata, and several registered in-module tests were explicitly direct
  inspection surfaces because the index did not return them reliably.
- Phase 2 is PASS. The design adds no cartridge execution authority, keeps the
  selected server as the single network broker, makes every trust claim
  independently verifiable, and maps all thirteen requirements to executable
  evidence.

## Phase 3 — Implement

- Built:
  - Added the strict canonical `omarchygs.cartridge-acquisition/v1` shared
    contract and hostile exact-claim/tamper corpus. The verifier independently
    authenticates the retained marketplace snapshot, exact review entry,
    publisher release, conformance, lifecycle policy, SDK/host compatibility,
    archive, and selected-server admission.
  - Migration `0020` retains the bounded exact signed current snapshot and
    public key with paired-null, monotonic-update, and no-delete/truncate
    guards. Synchronization publishes the evidence atomically and an exact
    replay safely backfills a Ticket 032 row.
  - The optional normal-server distribution runtime is all-or-nothing, reopens
    the reviewed secure store, confirms the database key and effective exact
    selection, resolves immutable retained components, emits and self-verifies
    the acquisition, registers only the authenticated fixed route, and adds a
    truthful sorted discovery capability.
  - Added `omarchygs-client-cartridge-runtime`: a guarded remote client with no
    proxy/redirect/decompression, strict canonical origin/UUID/bearer/response
    validation, initial/final catalog TOCTOU checks, a private
    descriptor-anchored shared cache, server-UUID profile mounts, and a random
    authenticated loopback companion API/binary.
  - Closed the inspection trust-anchor finding at the shared verifier: every
    caller must supply an expected complete marketplace public key, the client
    loads it only from an independent local path, same-label key substitution
    is rejected, and mount records remain bound to the trusted-key fingerprint
    across restart. Without a configured key, the social client starts but
    marketplace-vetted acquisition and mount inventory fail closed.
  - Added QML acquisition authority/controller and a distinct signed-cartridge
    Games surface with explicit install/update/remove, bounded provenance and
    lifecycle status, authority resets, prior-mount preservation, and no
    cartridge execution claim.
  - The native x86_64 Arch package now builds and launches the Rust companion,
    removes its private startup document immediately after reading it, passes
    the random endpoint/credential to trusted QML, uses a private per-user data
    root, and always stops/cleans the child runtime with the shell.
  - Updated server/operator/player/API/architecture/roadmap documentation to
    distinguish implemented acquisition/mounting from the deferred live
    session/render-plan binding.
- Focused evidence:
  - `cargo test -p omarchygs-game-cartridge`: 33 tests passed across unit and
    integration targets, including the new acquisition hostile corpus.
  - `cargo test -p omarchygs-client-cartridge-runtime`: 7 tests passed,
    including a real loopback discovery/catalog/acquisition/final-catalog path,
    successful cache mount, changed-admission denial, origin rejection,
    cross-process lock release, profile isolation, and companion auth.
  - `scripts/test-database.sh`: the marketplace test, all 53 server database
    tests, 5 operator-admin tests, and 3 operator CLI tests passed, including
    migration 0020 and the production exact acquisition route.
  - QML fixture suite passed 46 tests after adding exact catalog, install,
    mount, hostile-destination/profile rejection, local removal, and
    catalog-only server coverage.
  - `scripts/test-client-package.sh` passed after two clean byte-identical
    x86_64 package builds, exact payload/mode/provenance inspection, extracted
    launcher smoke, private-cache creation, and runtime cleanup.
- Deviations:
  - Mounts use one bounded canonical mode-0400
    `profiles/<server-uuid>.json` snapshot rather than a nested file per game.
    This gives one atomic profile transition while retaining exact
    server-UUID isolation and sorted unique game keys.
  - Unreferenced immutable content is always retained in this slice; no cache
    garbage collector was added. This is the conservative behavior already
    allowed by REQ-011 and preserves authenticated denial evidence.
  - Package validation composes the real extracted companion lifecycle with
    the Rust end-to-end signed-acquisition test and the QML exact-contract
    interaction suite; the unauthenticated extracted QML smoke itself does not
    manufacture a test device credential or marketplace release.
- Phase 3 is PASS. All planned production surfaces are implemented, focused
  evidence is green, and the remaining work is independent inspection,
  canonical validation, documentation reconciliation, and delivery.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Concurrency and filesystem | The initial cross-process mount helper retained `flock` after returning, so another client handle could block indefinitely. | medium | Confirmed and fixed before the frozen security scan; centralized explicit unlock and added a second-handle nonblocking regression. |
| 2 | Native package integration | Makepkg's global GCC LTO flags made `ring` native objects unlinkable by Rust `lld`. | correctness | Confirmed and fixed with package-local `!lto`; real two-build reproducibility and smoke passed. |
| 3 | Authentication and provenance | The acquisition verifier authenticated a marketplace snapshot with the key supplied by the same selected server, allowing self-consistent false `marketplace_vetted` provenance. Codex Security scan `777bdabd-7634-488c-8585-e66b3674fad9` validated it as high-confidence P3/low for the current inert mount. | low | Confirmed and fixed: full client-controlled key equality is enforced by the shared verifier, mismatched same-label keys fail in unit and remote-path tests, and cached mounts bind the trusted-key SHA-256 fingerprint. Fresh bypass/regression review and final gates remain. |
| 4 | Capability compatibility | The initial trust-anchor fix made QML require the optional acquisition capability before requesting the base cartridge catalog, hiding metadata on supported catalog-only servers. | high | Confirmed and fixed: the authority now requires `games.cartridge-catalog.v1`, carries acquisition support as a separate flag, renders catalog metadata, and disables only install. A dedicated catalog-only discovery fixture proves the authenticated GET and rendered delegate. |
| 5 | Final structural and blast-radius inspection | Fresh CodeGraph traced server selection and immutable-store resolution into the self-verified bundle, client discovery/catalog/bundle/final-catalog checks, independent key authentication, cache key fingerprinting, serialized profile replacement, and the companion API callers. It reported incomplete test linkage for several new symbols; direct inspection covered their in-module tests plus unsupported QML, shell, SQL, packaging, and documentation surfaces. | none | PASS. No alternate acquisition or mount path bypasses the independent marketplace key, exact server admission, or profile trust-key binding. After the clippy-only correction, the refreshed worktree-bound receipt matches gated state `1b50fdc79f2ef457c3beb826695ebfe8857c653d823f1f8a6b6b4ce0b8b6e391`. |

- Phase 3.5 is PASS. Both confirmed implementation findings are fixed, the
  focused regression suites are green, and the fresh CodeGraph receipt matches
  the post-edit gated worktree.

## Phase 4 — Validate

- Tests run:
  - Focused post-inspection validation passed: 7 client-runtime tests,
    workspace clippy with `-D warnings`, formatting, and the 46-test QML
    onboarding/game fixture suite.
  - `bin/gate.sh --fast` passed all 15 checks after the one-line clippy fix.
  - `bin/gate.sh --diff` passed all 22 checks, including the complete Rust
    workspace, production cartridge and renderer proofs, SDK release,
    architecture spike, two byte-identical native client packages, all 53
    PostgreSQL server tests, 5 operator-admin tests, 3 operator CLI tests,
    real API/QML smoke, remote-provider conformance, the clean-clone Door
    Legends authority pilot, backup/restore, and private-alpha admission.
- Gate run: `GATE GREEN [diff]`; the canonical receipt and current gated state
  both equal `1b50fdc79f2ef457c3beb826695ebfe8857c653d823f1f8a6b6b4ce0b8b6e391`.
- Skips or pre-existing failures: none in the canonical diff gate. The QML
  corpus emits three existing shutdown-time `ApiClient.qml` warnings while
  its challenge test passes; no request-contract violation or failed test was
  observed.
- Phase 4 is PASS. The validation receipt and inspection receipt both match
  the final gated implementation state.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Completion evidence |
  |---|---|---|
  | REQ-001 | PASS | All-or-nothing distribution configuration, startup, route, and truthful discovery capability tests passed; catalog-only operation remains supported. |
  | REQ-002 | PASS | Migration `0020` and sync/replay PostgreSQL tests prove paired bounded signed-snapshot/key retention, monotonic publication, and upgrade backfill. |
  | REQ-003 | PASS | Authenticated Axum distribution tests prove no-store bounded exact responses assembled only from the effective admission and retained immutable artifacts. |
  | REQ-004 | PASS | Selection, lifecycle, omission, compatibility, no-fallback, concurrent change, and client final-catalog TOCTOU cases deny stale mounts. |
  | REQ-005 | PASS | Companion auth tests and extracted-package smoke prove random loopback authority, cache mutation outside QML, child cleanup, and removal of runtime state. |
  | REQ-006 | PASS | Remote-client tests reject noncanonical/remote-HTTP origins, redirects, proxy use, wrong UUID/path/digest/revision, encoding, timeout, and oversized responses. |
  | REQ-007 | PASS | The 33-test shared cartridge corpus plus remote-path regressions reject every tampered claim and a same-label substituted marketplace key. |
  | REQ-008 | PASS | Cache tests prove private descriptor-relative roots, immutable non-executable content, no-follow race resistance, atomic publication, and digest reuse. |
  | REQ-009 | PASS | Restart/multi-profile tests prove server-UUID isolation, trusted-key fingerprint binding, exact revision identity, and cross-profile denial. |
  | REQ-010 | PASS | The 46-test QML corpus covers catalog-only browsing, install/update/remove, mounted/deprecated/unavailable/error states, authority reset, and prior-mount preservation. |
  | REQ-011 | PASS | Exact removal tests unlink one profile mount only, retain shared content, make no server mutation, and have no authoritative domain deletion path. |
  | REQ-012 | PASS | Source admission plus two byte-identical x86_64 package builds prove exact 39-file QML/native-companion payload, modes, provenance, trust-key denial, cache creation, and cleanup. |
  | REQ-013 | PASS | `bin/gate.sh --diff` passed all 22 stages with the acquisition, database, QML, package, provider, recovery, and private-alpha paths. |

- Docs: OpenWiki update run `2ba59d79-bfad-42ee-bc44-705316a4175e`
  completed after reconciling quickstart, cartridge, runtime, validation, and
  product-boundary pages. It preserved their Claims sidecars because of
  pre-existing unresolved evidence debt and reported that warning explicitly.
- AAR: submitted as effective with six failures, six prevention rules, and
  one architecture decision added to the knowledge register.
- Archive: Ticket 033 moves to `tickets/closed/` and the sole active spec/notes
  pair moves to `pipeline/completed/`; no active pipeline remains.
- Phase 5 is PASS. All thirteen requirements are accounted for, durable docs
  and lessons are reconciled, and the pipeline is ready for authorized delivery.
- Delivery validation note: the first post-archive diff gate passed 21 of 22
  stages but rejected the completed spec's descriptive status suffix. The
  checker treats `Phase 5 — Complete PASS` as an exact enum; the spec was
  corrected to that value before the final gate rerun.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A second companion/cache handle could block indefinitely after the first mount operation. | The initial implementation acquired the cross-process `flock` but did not release it after the profile operation. | Centralized mount operations under a helper that always unlocks and added a second-handle nonblocking regression test. | Every retained synchronization lock needs explicit release evidence, not only mutual-exclusion tests within one handle. |
| 2 | The first real Arch package release link failed with unresolved `ring` native symbols. | Makepkg appended GCC `-flto=auto` to `ring` C objects, which the Rust `lld` link could not consume. | Marked the package `!lto`; two clean builds then linked and were byte-identical. | Exercise native dependency linking inside the real package environment and explicitly disposition incompatible global toolchain flags. |
| 3 | A fully self-consistent acquisition from a malicious selected server could create false marketplace-vetted provenance. | The shared verifier authenticated the snapshot with `marketplace_key` from the same server-controlled envelope and no independent expected key. | Required exact equality with a client-controlled startup trust key and bound every cached mount to its fingerprint. | A claim that is meant to remain independent from an authority must authenticate its trust root outside that authority and preserve the binding at rest. |
| 4 | Metadata-only servers disappeared from the cartridge screen while hardening acquisition trust. | The UI used an optional mutation/distribution capability as the admission check for the underlying read-only catalog. | Required the base catalog capability for browsing and modeled acquisition support independently for install controls. | Negotiate base read surfaces and optional mutation capabilities independently, with a fixture for every supported capability subset. |
| 5 | The first canonical fast gate failed clippy on a needless borrow in final mount construction. | Focused tests and formatting do not diagnose every workspace-wide `-D warnings` lint. | Passed the already borrowed selected release directly to `MountRecord::from_verified`. | Treat clippy as distinct required evidence even when the same path compiles and all focused runtime tests pass. |
| 6 | The first post-archive delivery gate rejected the completed spec although every code and runtime stage passed. | The spec added descriptive text after the pipeline checker's exact `Phase 5 — Complete PASS` status enum. | Restored the exact accepted status and ran the narrow pipeline check before repeating the full gate. | Treat lifecycle frontmatter as a closed schema and validate it with the owning checker before an expensive delivery run. |
