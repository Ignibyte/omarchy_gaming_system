---
title: Public marketplace trust enrollment, rotation, and client package channel — notes
pipeline_id: d9d78401-aa06-4134-bba0-61c5683cd5c2
---

# Public marketplace trust enrollment, rotation, and client package channel — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 035 is delivered at `e84c480`; local and remote `main`
  identities matched and the worktree was clean before Ticket 036 opened. No
  active pipeline or open ticket remained, and pipeline tooling passed with
  CodeGraph 1.5.0, OpenWiki 0.3.3, and Codex-only provenance active.
- Recall: the first external two-clean-install acceptance run remains the
  earliest unchecked private-alpha roadmap item, but it explicitly requires
  external people/installations and cannot be truthfully executed by this
  workstation. Public marketplace trust enrollment is the next ordered,
  locally actionable product slice.
- Recall: the packaged launcher currently resolves one absolute regular
  marketplace-key file from explicit environment, per-user config, or system
  config. The companion parses it once, QML receives only a readiness boolean,
  and every acquisition/mount/render operation requires exact full-key or
  SHA-256-fingerprint equality.
- Recall: this single-key boundary is intentionally independent from a
  selected server, but it has no public enrollment or rotation protocol.
  Replacing the file causes all mounts under the old fingerprint to fail
  closed, and Ticket 035 historical acquisition also requires retained
  snapshot evidence whose exact older key currently must equal the one runtime
  key.
- Recall: the server similarly accepts one configured marketplace key.
  Marketplace synchronization, current acquisition, historical acquisition,
  lifecycle policy, retained evidence, and distribution runtime all assume
  exact equality with that singleton. Rotation therefore crosses both server
  and client trust consumers rather than only launcher UX.
- Recall: the Arch package is deterministic and source-digest bound, but the
  player currently obtains and verifies one `.pkg.tar.zst` plus sidecar digest
  manually. The QML/companion process is same-user and must never gain package
  manager, sudo, shell, or privileged installation authority.
- Recalled prevention rules:
  `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001`,
  `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001`,
  `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001`,
  `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001`,
  `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001`,
  `PR-omarchy-gaming-system-derive-digests-with-verifier-encoding-001`,
  `PR-omarchy-gaming-system-stabilize-package-build-paths-for-reproducibility-001`,
  `PR-omarchy-gaming-system-prove-native-linking-in-package-environment-001`,
  `PR-omarchy-gaming-system-separate-historical-provenance-from-current-use-policy-001`,
  and `PR-omarchy-gaming-system-align-producer-consumer-limits-and-uniqueness-001`.
- Decision: add a domain-separated offline-root-signed channel document that
  binds one stable channel, a monotonic validity-bounded trust version, exact
  marketplace keys with snapshot-version eligibility, and immutable native
  package artifacts. Only its public root and canonical channel location may
  enter a reviewed client package.
- Decision: use active, retired, and revoked key states. Exactly one active key
  may sign new snapshots; a bounded retired key may authenticate only its
  explicit historical snapshot-version interval; a revoked key is terminally
  denied for acquisition, mount, and render use.
- Decision: explicit enrollment/sync atomically publishes a complete private
  per-user keyring only after guarded independent-channel fetch and root
  verification. A selected server, catalog, acquisition, or QML document
  cannot contribute trust material or channel location.
- Decision: authenticate and optionally stage exact package artifacts, but
  stop at player-visible provenance and a copyable/revealable manual install
  command. No client process starts pacman, sudo, or a shell.
- Decision: preserve the existing explicit manual single-key mode and
  no-key/social-only mode. Mixed manual/channel trust configuration fails
  rather than silently choosing an authority.
- Phase 1 is PASS. Fifteen observable requirements define one end-to-end trust
  bootstrap, rotation, historical-evidence, package-provenance, compatibility,
  and recovery boundary without adding a private key or privileged installer.

## Phase 2 — Design

- CodeGraph evidence:
  - Design explores ran while the active spec was in Phase 1 and produced a
    matching `design` receipt for pipeline
    `d9d78401-aa06-4134-bba0-61c5683cd5c2` and the current gated worktree.
  - Server synchronization flows through `synchronize` →
    `synchronize_with_client` → `publish_snapshot`. The exact configured key
    verifies the snapshot, every policy, secure-store staging, PostgreSQL
    singleton, evidence retention, catalog activation, distribution, session
    presentation, and action admission. Rotation therefore requires one shared
    trust decision, not isolated key-file parsing at startup.
  - `acquire_exact` reads the current singleton key and snapshot, while
    `acquire_session_exact` reads immutable retained evidence. Both converge in
    `build_acquisition`, which currently assumes the evidence key is also the
    current policy key. Under rotation those are independent: an old snapshot
    may use a retired key while the release's newer current lifecycle policy
    is signed by the active key.
  - `marketplace_sync_state` stores the current key, and its existing database
    guard prohibits any `key_id` change. `marketplace_releases` stores mutable
    signed policy but not the key that signed it. Migration `0023` must permit
    an authenticated key transition only with a newer snapshot and bind each
    release's current policy to its exact key/snapshot version.
  - `SecureCartridgeStore` shares immutable release bytes by digest but keeps
    one mutable policy file per digest. A newly active key would replace that
    policy and make an old mount unverifiable under its retained fingerprint.
    Policy cache identity must become digest plus exact marketplace-key
    fingerprint, with a narrow legacy single-key read migration.
  - `ClientCartridgeCache` currently validates one complete server profile
    against one caller-supplied key fingerprint. Its exact mount records
    already retain snapshot version and key fingerprint, so profile format v1
    can remain readable while validation moves to a bounded trust keyring and
    each render resolves the exact retained key for its mount.
  - `CompanionState` holds `Option<Arc<CatalogPublicKey>>`, and each mount,
    acquisition, removal, and render handler clones that singleton. It becomes
    an atomically replaceable immutable trust snapshot plus a descriptor-bound
    trust/package store; every long operation rechecks the latest trust state
    before publishing effects.
  - `GuardedMarketplaceClient` already supplies the required canonical HTTPS,
    public-DNS, explicit socket resolution, custom TLS, no proxy/redirect/
    decompression, timeout, and streaming body bounds. Its reusable transport
    moves behind a shared trust-channel crate so server and client enforcement
    cannot drift; the package path adds checked streaming into a caller-owned
    private staging file.
  - Shell/QML/package formats are outside CodeGraph's reliable Rust topology.
    Direct inspection confirmed the launcher precedence
    environment → per-user key → system key, the static QML readiness boolean,
    deterministic package source digest, generated build provenance, and no
    current package-channel/bootstrap payload.

- Architecture and trust flow:
  - A new non-SDK workspace crate owns
    `omarchygs.marketplace-trust-channel/v2`. It defines separate Ed25519
    offline root key types, canonical domain-separated signing/verification,
    exact key fingerprints, bounded trust transitions, package metadata, and
    the shared guarded HTTPS client. Keeping it outside the Game Cartridge
    crate preserves the released SDK v1 lock and existing attestations.
  - The signed payload binds `channel_id`, display name, canonical channel and
    marketplace origins, strictly increasing `bundle_version`, bounded Unix
    `not_before`/`expires_at`, one marketplace authority, up to 16 ordered key
    records, and up to 32 ordered package artifacts. Signed and decoded payload
    bytes are capped at 256 KiB and must be canonical exact-schema JSON.
  - A key record contains the complete `CatalogPublicKey`, exact SHA-256,
    `active|retired|revoked`, and a contiguous inclusive snapshot interval.
    History starts at version 1; exactly one final active record has no upper
    bound. Prior records end immediately before the next begins. Key IDs,
    bytes, and fingerprints are unique, every authority matches, and a
    terminally revoked record can never disappear or become trusted again.
  - Bundle updates require the same root/channel/origins/authority, a greater
    bundle version, retained complete prior key history, monotonic time, and
    only `active → retired|revoked` or `retired → revoked` transitions with a
    replacement active range. Missing intermediate bundle versions are safe
    because the complete history is revalidated; rollback and same-version
    byte changes fail.
  - The trust object exposes exact decisions: the active key may verify a new
    snapshot only when its version exactly matches the root-authenticated
    `current_snapshot_version`; active or retired exact bytes may verify
    current/historical evidence inside their range; revoked, unknown,
    expired, out-of-range, duplicate-label, or fingerprint-substituted keys are
    denied. Marketplace lifecycle remains a separate later decision.
  - The project package optionally installs a canonical bootstrap document
    containing only one public root, channel identity/origin, fixed manifest
    path, and release platform metadata. `build-client-package.sh` accepts an
    explicit reviewed bootstrap input, authenticates and hashes it into source
    and build provenance, and passes it to `PKGBUILD`. A normal build without
    that option installs no channel and truthfully remains manual/no-key mode.
  - The launcher rejects mixed manual-key and packaged-channel modes. In
    channel mode it passes absolute packaged bootstrap and private data-root
    paths to the companion; it never parses a downloaded key or supplies one
    to QML. Manual explicit/per-user/system key precedence remains unchanged.
  - A descriptor-anchored same-user `ClientTrustStore` keeps the last complete
    root-verified signed bundle under the existing private application data
    root. Enrollment/synchronization fetches only the package-configured
    manifest, checks public DNS and standard WebPKI TLS, verifies the root and
    wall-clock window, compares the prior transition, fsyncs a 0600 temporary,
    atomically renames, fsyncs the directory, then swaps one immutable
    `Arc<MarketplaceTrust>` under an `RwLock`.
  - Trust expiry denies cartridge acquisition/mount/render and package staging
    until a valid refresh; social/server gameplay REST remains available and
    trusted platform presenters may continue. The highest verified bundle and
    terminal key history remain persisted, so clock rollback or restart cannot
    restore a revoked/older trust state.
  - Manual mode wraps the existing one complete key as an unexpiring exact
    compatibility trust object. No-channel/no-key mode starts normally with no
    cartridge authority. Channel state never merges with manual state.
  - Companion loopback adds strict credential/Host/no-store endpoints:
    `GET /v1/trust`, `POST /v1/trust/synchronize`,
    `GET /v1/client-packages`, and `POST /v1/client-packages/stage`.
    Responses expose channel/version/fingerprints/status/artifact facts but no
    root bytes, filesystem authority beyond the host-generated staged package
    path, server credential, or private material.
  - Trust synchronization clones no selected-server state. Acquisitions clone
    one trust snapshot for network verification, then re-read current trust
    before cache installation. Mount inventory/render resolves each record's
    exact trusted key and current status. Revoked records remain visible as
    remove-only local facts; exact removal is structural and cannot be blocked
    by the authority it removes.
  - Profile documents stay format v1 and may contain multiple key
    fingerprints. Each exact mount's snapshot version selects its keyring
    record; unknown/revoked mounts cannot render. Shared content remains keyed
    by digest. Cached signed policies become
    `(archive_sha256, marketplace_key_sha256)` records, preserving old/new key
    policies simultaneously and migrating a valid legacy digest-only policy
    only under the exact manual key.
  - Server local catalog configuration becomes mutually exclusive manual-key
    or root+signed-bundle mode. Bundle mode independently verifies local files,
    requires the configured marketplace origin to equal the signed payload,
    uses only its active key for new sync, and carries the full immutable trust
    object into catalog, distribution, session pin/action, and admin paths.
  - Migration `0023` adds exact `policy_marketplace_key` and
    `policy_snapshot_version` to releases, backfills them from existing
    singleton evidence, and strengthens monotonic guards. The sync singleton
    may change key only with a higher snapshot while origin/authority and
    same-version bytes remain immutable. Release policy/key changes require a
    newer policy/snapshot; key provenance cannot change alone.
  - Synchronization verifies the new snapshot and policies with the one active
    key, stages policy under that fingerprint, and persists it on every
    release update. Current catalog activation and new session pins use that
    release's exact policy key after trust authorization rather than a label
    comparison with the singleton.
  - Current acquisition emits singleton evidence under the active key.
    Historical acquisition independently authorizes the retained evidence key
    and snapshot range, resolves the release under the separately stored
    current policy key/range, emits acquisition v2 with separate signed
    evidence and policy snapshots, and self-verifies both exact keys. The v1
    verifier remains compatible for single-key evidence; no server-selected
    channel is accepted.
  - A package artifact record binds platform `arch-linux`, architecture
    `x86_64`, package/version/filename, relative channel path, exact bytes,
    SHA-256, source revision/digest, and build provenance digest. Ordering and
    uniqueness reject ambiguous “latest” selection; the client selects only
    its packaged platform and a version newer than its installed release.
  - Package staging streams at most 256 MiB through the guarded channel into a
    descriptor-created 0600 non-executable temporary under `updates/`, hashes
    while writing, verifies exact byte count/digest, fsyncs, and atomically
    publishes one digest-named artifact. Cancellation/tamper retains no final
    file and never affects trust or cartridges.
  - Trusted platform QML adds an explicit marketplace enrollment/sync card and
    package update card to the Games surface. It shows origin/channel, bundle
    version/expiry, key status, package version/digest, stable offline/error
    states, and keyboard-accessible controls. A staged package yields only a
    platform-generated copyable `sudo pacman -U -- <fixed-safe-path>` command;
    QML/companion never spawns a process or requests elevation.

- Exact file manifest:

  | File(s) | One purpose |
  |---|---|
  | `Cargo.toml`, `Cargo.lock`, `crates/marketplace-trust/Cargo.toml`, `src/lib.rs`, `src/transport.rs`, `src/bin/omarchygs-marketplace-channel.rs`, tests | Add the non-SDK root-signed trust/channel contract, keyring transition verifier, guarded transport, deterministic CLI, and hostile conformance corpus. |
  | `migrations/0023_marketplace_trust_key_rotation.sql` | Bind every release policy to its exact signing key/snapshot and permit only newer-snapshot singleton key transitions. |
  | `crates/game-cartridge/src/secure_store.rs` | Key cached signed policy by digest plus marketplace fingerprint with exact legacy compatibility, without changing SDK-exported public contract bytes. |
  | `crates/server/Cargo.toml`, `marketplace_egress.rs`, `marketplace_sync.rs`, `cartridge_catalog.rs`, `cartridge_distribution.rs`, `session_cartridges.rs`, `config.rs`, `app.rs`, admin binary | Consume manual or verified keyring trust for sync, policy persistence, catalog/session admission, current/historical acquisition, and actions while sharing guarded transport. |
  | `crates/server/src/marketplace_sync_tests.rs`, `cartridge_catalog_api_tests.rs`, `provider_game_api_tests.rs`, `server_discovery_api_tests.rs` | Prove rotation/revocation/database guards, policy/evidence key separation, both acquisition paths, session actions, compatibility, and clean Door Legends behavior. |
  | `crates/client-cartridge-runtime/Cargo.toml`, `src/trust.rs`, `src/package_channel.rs`, `src/main.rs`, `src/lib.rs`, `src/service.rs`, `src/remote.rs`, `src/cache.rs`, `src/render.rs` | Add descriptor-bound enrollment/package state, mutable trust snapshots/endpoints, keyring acquisition/cache/render decisions, and bounded artifact staging. |
  | `client/qml/OnboardingController.qml`, `CartridgeController.qml`, `screens/GamesScreen.qml`, `Main.qml`, QML fixtures/server | Negotiate exact trust/package responses and expose explicit accessible enrollment, sync, staging, command-copy, failure, and compatibility states. |
  | `packaging/arch/PKGBUILD`, `packaging/arch/omarchygs`, `scripts/build-client-package.sh`, package source/test scripts and fixtures | Optionally bind reviewed public bootstrap inputs and installed release metadata into reproducible package provenance while preserving manual builds. |
  | `scripts/test-marketplace-trust-channel.sh`, existing database/QML/package/provider/recovery gate scripts | Prove deterministic channel production, clean-client enrollment/rotation/package flow, and keep all nested delivery evidence gated. |
  | `README.md`, API/client/operator/system/cartridge architecture/roadmap docs, OpenWiki | Reconcile player/operator/release trust bootstrap, key states, expiry/recovery, package flow, limitations, and remaining custom/provider/module work in Phase 5. |

- API, storage, and compatibility contract:
  - Existing public server discovery, catalog, acquisition v1, session
    presentation/action, render-plan v2, and mount-record v1 contracts remain
    unchanged. Keyring choice is local server/client authority, never public
    selected-server input.
  - Companion endpoints are additive and exact-schema. Existing acquisition,
    removal, and render requests stay unchanged; mount-list projections add a
    required host-generated trust status only for the packaged matching
    QML/companion pair.
  - The native trust bundle and bootstrap are separate from Game Cartridge SDK
    v1. The current SDK lock
    `7a732939918254ca1fb399f1fa4a4ef70d252ad683c13696dec8db8e2e88a045`
    remains an explicit regression invariant.
  - Manual `OGS_CLIENT_MARKETPLACE_PUBLIC_KEY` and
    `OGS_MARKETPLACE_PUBLIC_KEY` retain exact behavior when no channel bundle
    variables/files are present. Any partial or mixed mode is invalid.
  - Existing PostgreSQL rows, profiles, release bytes, and digest-only policies
    are not rewritten destructively. Forward schema is additive; exact legacy
    policy migration occurs only when authenticated with its supplied manual
    key. Code rollback can ignore new columns/files but cannot undo schema or
    key revocation history.

- Regression plan:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Deterministic root/channel CLI plus canonical schema, limits, ordering, private-field, signature-domain, and tamper tests. |
  | REQ-002 | Manual and channel package builds, extracted payload/provenance inspection, source digest, identical double-build hashes, and absent/mixed bootstrap cases. |
  | REQ-003 | Companion/QML clean enrollment with hostile selected-server discovery/catalog/acquisition root, origin, label, and byte substitutions. |
  | REQ-004 | Shared guarded transport public/private/mixed/excess DNS, TLS, proxy, redirect, decompression, media, length, stream, timeout, and path corpus. |
  | REQ-005 | Descriptor trust-store root/symlink/hardlink/mode/owner, canonical/tamper/expiry/rollback/collision, partial write, lock, atomic replace, and restart tests. |
  | REQ-006 | Complete active/retired/revoked contiguous range and update-transition matrix with duplicate IDs/bytes/fingerprints and terminal revocation. |
  | REQ-007 | Server manual/bundle configuration and PostgreSQL old-active → retired/new-active snapshots, replay, restart, policy restaging, invalid key change, and catalog activation. |
  | REQ-008 | Current and historical acquisition across active/retired/revoked evidence and separate current-policy keys plus suspension/revocation lifecycle cases. |
  | REQ-009 | Same-server mixed-fingerprint mounts, trusted/retired/revoked listing, render resolution, exact removal, origin isolation, legacy policy/profile, and restart. |
  | REQ-010 | Trust sync versus acquire/install/list/render/stage concurrency, failed/crashed replacement, terminal restart, and server snapshot writer/acquisition lock ordering. |
  | REQ-011 | Platform/version selection, exact metadata response, streamed artifact size/hash/path/mode, ambiguity, capacity, cancel/tamper cleanup, coexistence, and restart. |
  | REQ-012 | Production QML keyboard/focus/accessibility/plain-text provenance and copy-command flow with explicit assertions that no process/installer request occurs. |
  | REQ-013 | QML/runtime unenrolled/enrolling/current/rotated/revoked/expired/offline/downloading/ready/retry matrix plus clean-client live smoke. |
  | REQ-014 | Existing server/client/cartridge/QML/package/provider suites plus no-channel, no-key, manual precedence, mixed-mode rejection, legacy profile/policy, and platform presenter cases. |
  | REQ-015 | Channel/package reproducibility, unchanged SDK identity, native package, database, QML, provider, recovery, docs/OpenWiki, and canonical diff gate. |

- Security, concurrency, recovery, and rollout risks:
  - The offline root is a high-value authority. Only public material is
    packageable; producer commands require explicit absolute private-key paths,
    never log key bytes, and secret scanning covers generated fixtures.
  - A valid but expired bundle cannot reveal a new revocation. Expiry therefore
    denies cartridge/package trust rather than becoming an availability-based
    bypass. The broader social client remains useful and reports the state.
  - Root-signed rollback with an artificially greater bundle version is still
    possible after root compromise; offline custody and future root-rotation
    policy remain operational requirements outside this ticket.
  - Rotated key policy and old evidence are distinct. Every database/runtime
    query carries both exact keys/snapshot versions where necessary; no helper
    silently substitutes the active key for historical proof.
  - Trust synchronization must not hold an async lock across network I/O. It
    verifies into an immutable candidate, serializes only final compare/store/
    swap, and operations recheck before effects.
  - Package bytes are executable after a human privileged install. Root-signed
    digest/size/source/build provenance and private non-executable staging are
    required, but this ticket never claims malware review beyond the signed
    channel metadata.
  - Same-user state is not a privilege boundary against that user. Descriptor
    containment prevents accidental/symlink redirection and cross-process
    races; pacman retains the actual privileged filesystem boundary.

- Material alternatives rejected:
  - Fetching a marketplace key from the selected server was rejected because
    it makes the provenance claim self-authenticating.
  - Replacing one key file in place was rejected because it cannot distinguish
    valid historical evidence, terminal revocation, overlap, or rollback.
  - Trusting all prior keys forever was rejected because key compromise would
    remain permanently useful; retired intervals and revoked denial are
    explicit.
  - Re-signing or rewriting old snapshots/mounts under the new key was rejected
    because it fabricates provenance and destroys exact historical identity.
  - Putting the trust contract into the Game Cartridge SDK crate was rejected
    because it would change the already released SDK v1 identity for a host
    distribution concern.
  - Running pacman/sudo or downloading an arbitrary repository database from
    QML was rejected because the same-user client has no privileged installer
    authority and the signed bounded artifact manifest is sufficient.
  - Shipping a development default root was rejected because public-key
    presence would falsely imply a maintained production private-key/channel
    lifecycle.

- Phase 2 is PASS. All fifteen requirements map to concrete contract,
  transport, database, server, companion, cache, QML, package, clean-client,
  compatibility, and recovery evidence; the CodeGraph design receipt matches
  the unchanged gated application worktree.

## Phase 3 — Implement

- Built:
  - Added the non-SDK `omarchygs-marketplace-trust` workspace crate with
    separate offline Ed25519 root keys, canonical domain-separated signed
    channel/bootstrap contracts, active/retired/revoked snapshot ranges,
    exact package metadata, deterministic CLI tooling, guarded shared HTTPS
    transport, and private-key file/path enforcement.
  - Added migration `0023` and server trust decisions that permit only a
    higher-snapshot authenticated marketplace key rotation, persist each
    release policy's exact key/snapshot, retain older snapshot evidence, and
    authorize current sync, historical/current acquisition, activation,
    session preparation, and actions under the appropriate key and range.
  - Keyed server and client signed-policy caches by archive digest plus exact
    marketplace fingerprint with a narrow authenticated legacy fallback.
    Client profile v2 retains distinct evidence/policy fingerprints and
    snapshot versions while v1 profiles remain readable.
  - Added acquisition envelope v2 so retired historical snapshot evidence and
    current lifecycle policy can carry different exact marketplace keys. V2
    carries the complete signed current policy-bearing marketplace snapshot;
    clients derive its version and exact policy bytes from that signature and
    require the policy key to remain active. The v1 producer/verifier remains
    available for single-key compatibility and the released Game Cartridge
    SDK export identity remains unchanged.
  - Added descriptor-bound per-user channel enrollment, atomic monotonic trust
    publication, restart-stable terminal revocation, multi-key mount/render
    decisions, and additive authenticated companion endpoints for trust
    status/sync plus package inventory/staging.
  - Added exact package selection and guarded streaming into bounded 0600
    non-executable staging. Cross-process publication locks, an eight-object/
    512 MiB staging ceiling, current-bundle recheck, digest/size verification,
    and an exact text-only pacman command prevent automatic or unbounded
    installation behavior.
  - Added `MarketplaceController.qml` and Games UI states for explicit
    enrollment/sync, exact channel/key/package/build provenance, verify-and-
    stage, copy-only installation text, multi-key local mount status, and
    remove-only inventory when trust is absent or expired.
  - Extended the optional Arch package build with a verified public bootstrap
    payload and provenance digest while preserving byte-reproducible manual
    builds and rejecting mixed/manual-channel configuration.
  - Added deterministic channel CLI checks, trust/keyring/package hostile
    tests, key-rotation PostgreSQL integration, acquisition-v2 coverage,
    staging bounds/restart validation, guarded package-download checks,
    QML hostile-envelope/offline/copy-only cases, and manual/channel package
    extraction tests.
- Focused evidence run during implementation:
  - `cargo test --locked -p omarchygs-marketplace-trust` passed 5 tests.
  - `cargo test --locked -p omarchygs-game-cartridge` passed 36 tests.
  - `cargo test --locked -p omarchygs-client-cartridge-runtime --lib` passed
    13 tests.
  - `scripts/test-database.sh` passed 2 TLS marketplace integrations, 56
    server tests, 5 admin tests, and 3 operator CLI tests.
  - `scripts/test-qml-onboarding.sh` passed 52 QML tests after the final QML
    implementation fix.
  - `scripts/test-client-package.sh` passed reproducible manual builds,
    channel bootstrap/provenance extraction, tamper rejection, and extracted
    client smoke.
  - `scripts/test-marketplace-trust-channel.sh` and the focused guarded TLS
    package-download test passed.
- Deviations:
  - Design expected the existing acquisition v1 envelope to carry rotated
    evidence unchanged. Code inspection proved v1 has one marketplace key for
    both historical evidence and current policy, so implementation preserves
    v1 compatibility but emits a versioned v2 envelope with two independently
    verified keys. This is the smallest honest representation of locked
    decision 8.
  - Legacy database releases whose old migration state has no policy-key
    binding may accept only monotonic metadata policy updates. They remain
    undistributable until an authenticated synchronization binds exact policy
    evidence; no key is inferred.
  - Stale crash-created package temporary files are counted against the hard
    staging ceiling instead of being deleted during startup, because deleting
    another live process's in-flight file cannot be made safe without a lease.
    Normal failure/cancellation removes its own temporary file.
- Phase 3 is PASS. The approved slice is implemented and focused checks are
  green; the complete diff is ready for independent inspection.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security: lifecycle currentness | Security diff scan `785f21de-c3c4-4d5e-b13f-f5fd02865ec4` found that acquisition v2 accepted an unsigned `policy_snapshot_version` beside an authentic historical policy, allowing a selected server to replay retired-key lifecycle state. | Low | Confirmed and fixed. V2 now transports a separately signed policy-bearing marketplace snapshot, derives the version/policy from verified bytes, requires the exact release/publisher key, and admits the policy snapshot only under the current active key. The server self-verifies the signed current snapshot and exact database policy. Acquisition and server PostgreSQL tests pass. |
| 2 | Security: cross-process revocation | The same scan found that an already-open companion process retained its instance-local trust `Arc` after another process persisted a newer rotation or revocation. | Low | Confirmed and fixed. Every security-sensitive snapshot/status/inventory read now reconciles descriptor-bound persisted trust under the in-process mutex and cross-process file lock, rejects rollback/removal, and atomically refreshes the local view. A two-store test proves live rotation and terminal revocation without reopen. |
| 3 | Security: package bootstrap TOCTOU | The same scan found that the package builder verified an external bootstrap path but passed that mutable path to later hashing and `makepkg`. | Low | Confirmed and fixed. The builder copies the candidate once into a mode-0600 private build-owned snapshot, then verifies, hashes, and packages only that snapshot. The package test mutates the original after the stable build lock is acquired and proves the verified snapshot and provenance are unchanged. |
| 4 | Package lifecycle | The already-staged fast path rehashed exact bytes but did not recheck that the exact artifact remained present in current root-authenticated metadata. | Robustness | Confirmed and fixed. Both existing and newly downloaded artifacts require exact membership in a freshly reconciled trust snapshot before a staged receipt is returned. |
| 5 | Network boundedness | Public DNS resolution happened before the configured HTTP/connect timeout budget, so a resolver stall was not time-bounded by the channel client. | Robustness | Confirmed and fixed. Production lookup is now wrapped in the same 15-second total deadline and maps expiry to a stable unavailable result. Marketplace transport tests pass. |
| 6 | Staging concurrency | Concurrent stage requests in one companion could each download a full temporary artifact before aggregate capacity admission. | Robustness | Confirmed and fixed. An asynchronous per-channel admission mutex serializes the bounded download/publication path; the cross-process publication lock and final capacity/current-trust checks remain authoritative. |
| 7 | Quality | The first post-inspection `bin/gate.sh --fast` run found two warnings denied by Clippy: an eight-argument server resolver and a `while let` iterator loop. | Build | Confirmed and fixed by introducing `CurrentPolicyEvidence` and using the directory iterator directly. The subsequent workspace/all-target Clippy run and fast-gate rerun passed. |
| 8 | Security: same-key snapshot freshness | The second sealed security scan `704cf74c-0a00-405e-b560-20d22fcf0975` found that active-key authorization still accepted any version in the key's open-ended range, allowing a selected server to replay a genuine older lifecycle snapshot before a newer policy had been cached. | Low | Confirmed and fixed. Root-signed trust-channel v2 authenticates one exact `current_snapshot_version`; every channel-mode server sync and client acquisition requires that exact version under the active key. Transitions prohibit rollback and prohibit assigning a previously authenticated current version to a replacement key. Exact-current, stale-current, package-only transition, and rotation-reassignment regressions pass. |
| 9 | Security: retired-key render authority | The second sealed scan found that mounted render resolved its current lifecycle policy through historical key lookup, so a retired key could remain a current-use authority after rotation. | Low | Confirmed and fixed. Render resolves historical release evidence separately, then requires the policy key and version to satisfy active exact-current authorization before secure-store lifecycle evaluation. A rotation regression denies the old mount; the QML client converts this denial to an explicit install/refresh flow, and active sessions may reacquire a current policy whose release lifecycle is `retired`. |
| 10 | Staging concurrency | The second scan considered cross-process temporary download capacity before publication. | Robustness | Suppressed as outside the stated security boundary: only same-UID callers already authorized to write the player's private data root can create another companion process. Each stream remains individually bounded, finalized inventory remains cross-process serialized and aggregate-bounded, and the condition remains documented as robustness rather than a trust-boundary vulnerability. |
| 11 | Fix regression: retired-session refresh | The required fresh bypass/regression reviewer confirmed rows 8–9 close their original paths but found that session reacquisition still delegated to secure-store new-launch staging, which denies `retired` even though the active-session lifecycle decision is `continue`. | Low | Confirmed and fixed. Generic acquisition remains `NewLaunch`; the session endpoint now explicitly stages under `ActiveSession`. A focused end-to-end client regression acquires and installs a retired active-session cartridge, while the SDK store regression proves ordinary new-launch staging still denies the same retired policy. Client 13/13, SDK 9/9, and targeted warnings-as-errors Clippy pass. |
| 12 | Security: server trust rollback | The fresh final-scan architecture pass found that the server verified only the configured bundle at startup and did not persist the highest bundle/key-status history, so restarting with an older still-valid bundle could revive evidence under a key revoked by a later trust-only bundle. | Low | Confirmed and fixed. PostgreSQL now persists the authenticated root fingerprint and complete trust payload, enforces monotonic/equal-version invariants, updates trust on exact marketplace-snapshot replay, rejects channel-to-manual downgrade, and validates configured continuity before constructing the server distribution runtime. The rotation test proves a trust-only revocation persists, the prior bundle is denied on restart, and the highest bundle succeeds. |
| 13 | Security: equal-version policy mutation | The same architecture pass found that a higher snapshot version could replace signed policy bytes/key without advancing `policy_version`, weakening the prior equal-version immutability invariant during key rotation. | Low | Confirmed and fixed. Both the application upsert and database trigger now require identical signed bytes and policy key when `policy_version` is equal. The root-authenticated rotation test first attempts a new-key snapshot with unchanged policy version and receives a conflict, then succeeds only after policy version advances. |
| 14 | Security: stale live server trust | A subsequent fresh architecture pass found that a long-running server distribution runtime could continue authorizing its startup trust payload after another administrator process persisted a newer trust-only revocation. | Low | Confirmed and fixed. Acquisition, session pinning, fresh action admission, and admin catalog apply now read root fingerprint and trust payload from the same database snapshot as release evidence and reject a stale or manual runtime. The rotation database test constructs the stale runtime before trust-only revocation, proves it denies afterward, and proves a runtime on the persisted bundle succeeds. |
| 15 | Security: fresh enrollment replay | The next exact-snapshot discovery pass found that a cache-cleared or first-run client knew the root and channel but no authenticated minimum bundle/snapshot version, so a selected compromised channel could replay an older, still-valid root-signed bundle and revive a subsequently revoked marketplace key. | Medium | Confirmed and fixed. The immutable package bootstrap now authenticates minimum bundle and current-snapshot floors; publish rejects older enrollment, and a persisted bundle below a newly installed package floor is treated as unenrolled so synchronization can advance it. The client regression rejects bundle 1 under a packaged bundle-3/snapshot-6 floor and enrolls the exact revocation bundle. |
| 16 | Security: historical upgrade availability | The same discovery pass found that migration 0023 backfilled every retained release with the singleton's current snapshot version even when a release had last appeared in an older snapshot, violating the new provenance constraint and preventing an existing database from upgrading. | Medium | Confirmed and fixed. Migration backfill now binds the persisted marketplace key to each release's own `last_seen_snapshot_version`. A scratch-schema regression builds the schema through migration 0022, seeds a retained historical release, applies 0023, and proves the `1:1` evidence version is preserved. |
| 17 | Security: floor-advance continuity | The next fresh client discovery pass found that a persisted bundle below a newly packaged floor was correctly unavailable for authorization but was also hidden from publication's transition check, allowing a newer individually valid root-signed bundle to overwrite authenticated terminal key history without continuity validation. | Low | Confirmed and fixed as defense in depth. Eligibility still treats below-floor trust as unenrolled, but publication reads the raw root-authenticated persisted bundle and always verifies the transition before replacement. The regression proves an individually valid higher-floor bundle cannot change a revoked key back to retired, while a continuity-preserving bundle at the same floor enrolls successfully. |
| 18 | Validation: provider lifecycle fixture | The first full diff gate found that the clean-clone provider authority pilot directly replaced a signed lifecycle policy without advancing its policy snapshot provenance, so migration 0023 correctly rejected the stale test write. | Build | Confirmed and fixed in the test boundary. The provider pilot now calls the shared lifecycle-publication fixture, which advances `policy_snapshot_version` and `last_seen_snapshot_version` under the snapshot advisory lock. The focused authority pilot then passed its full suspend/replay/restart flow. |

- The sealed pre-fix security scan covered 32 authoritative changed source
  files and supporting QML/packaging boundaries. It reported exactly three
  low-severity findings (rows 1–3); its snapshot digest was
  `codex-security-snapshot/v1:sha256:67b3273c1f7a36e158dbb489d020f77e9f1d880a25c4579b79c3f777712fc028`.
- The second sealed post-fix scan covered all 32 authoritative changed source
  items plus the supporting QML and packaging boundaries. It reported the two
  additional low-severity findings in rows 8–9 and suppressed row 10 under the
  explicit same-UID boundary. Scan ID
  `704cf74c-0a00-405e-b560-20d22fcf0975`; snapshot digest
  `codex-security-snapshot/v1:sha256:4616acf2e05f9a6bed12644c3f9ce0ae057ce6db20e9a836e179b7728b2ee979`.
- Post-fix focused evidence:
  - acquisition v2: 2/2 passed;
  - client runtime library: 13/13 passed, including live peer refresh;
  - marketplace trust: 5/5 passed, including exact-current ownership across
    package-only updates and key rotation;
  - `scripts/test-database.sh`: 2 TLS marketplace integrations, 56 server
    tests, 5 admin tests, and 3 operator CLI tests passed;
  - QML onboarding/interaction: 52/52 passed, including stale-policy refresh;
  - native client package build/extraction, including deterministic bootstrap
    swap after snapshot: passed;
  - workspace/all-target Clippy with warnings denied: passed;
  - `bin/gate.sh --fast`: green after the final source fixes.
- Final sealed security scan `56922d30-0cad-4d75-a677-12e1219e3292`
  reviewed 35 authoritative files at snapshot
  `codex-security-snapshot/v1:sha256:dfeb1edf0c42017b814bb8e947c47a264a06fba99232b4c01a41eeec056bf91b`
  and completed clean with no reportable findings. The TAC advisory connector
  was unavailable because its account was not logged in; the repository scan
  and its three independent exact-snapshot reviews still completed.
- A fresh post-validation CodeGraph inspection confirmed that the provider
  pilot now shares the same monotonic policy-publication helper as the
  cartridge action regression. `cargo fmt --all --check` identified only
  import ordering, `cargo fmt --all` corrected it, and
  `scripts/test-provider-authority-pilot.sh` then passed.

## Phase 4 — Validate

- Tests run:
  - Marketplace trust contract/transport: 5/5 Rust tests plus the deterministic
    channel script passed.
  - Client cartridge runtime: 13/13 library tests passed, including live peer
    refresh, floor advancement, terminal revocation, rotated mounts, and
    bounded package staging.
  - PostgreSQL: migration 0023 historical upgrade, 2 guarded TLS integrations,
    56 server tests, 5 admin tests, and 3 operator CLI tests passed.
  - QML: 52/52 onboarding/interaction cases passed, including trust states,
    hostile envelopes, stale-policy recovery, package provenance, copy-only
    installation text, and production-root behavior.
  - Native packaging: manual and channel builds, extracted bootstrap and
    provenance, tamper rejection, executable linkage, clean-client smoke, and
    byte reproducibility passed; the channel package digest was
    `1d025af674c86952a919190bcb97f33bec7c16370c68fda4985b91effbc939f7`.
  - The focused first-party provider authority pilot passed after its lifecycle
    fixture was corrected to advance policy snapshot provenance.
- Gate run:
  - The first captured `bin/gate.sh --diff` run passed every stage except the
    first-party provider authority pilot. Its preserved output proved migration
    0023 correctly rejected the pilot's stale direct policy write.
  - After reusing the shared monotonic lifecycle-publication fixture, the fresh
    canonical `bin/gate.sh --diff` run passed all 23 labeled stages and wrote
    worktree receipt
    `c788b1e6db9529538f399b10768c099e9c1c2f2f9c5de46b54fd3c1ed6aa0c3a`.
- Skips or pre-existing failures: none. The unavailable TAC advisory connector
  did not gate the completed repository security scan. OpenWiki retained its
  explicit pre-existing evidence-debt warnings rather than silently claiming
  the five affected Claims sidecars were reconciled.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Delivery evidence |
  |---|---|---|
  | REQ-001 | PASS | Canonical offline-root trust/channel v2 contract, deterministic signer/verifier CLI, bounds, schema, tamper, private-material, and reproducibility tests. |
  | REQ-002 | PASS | Optional package bootstrap binds one exact public root, origin, and freshness floors; extracted manual/channel packages prove honest absence and mixed-mode rejection. |
  | REQ-003 | PASS | Explicit companion enrollment uses only package bootstrap authority; selected-server substitution fixtures cannot supply root, origin, channel, or keys. |
  | REQ-004 | PASS | Shared guarded HTTPS enforces public destinations, no proxy/redirect/credentials/decompression, exact media/size/time bounds, and hostile DNS/TLS cases. |
  | REQ-005 | PASS | Descriptor-bound trust store verifies canonical root signatures, validity, monotonicity, floors, transitions, atomic writes, concurrency, symlinks, restart, and failure preservation. |
  | REQ-006 | PASS | Active, retired, and terminally revoked keys have unique exact identities and non-overlapping bounded snapshot eligibility; rotation/revocation matrices pass. |
  | REQ-007 | PASS | Server manual/channel modes, persisted root continuity, exact-current synchronization, replay, authenticated key rotation, restart, and downgrade denial pass PostgreSQL tests. |
  | REQ-008 | PASS | Acquisition v2 independently authenticates historical evidence and signed current policy under their exact keys/versions; active, retired, revoked, stale, and lifecycle cases pass. |
  | REQ-009 | PASS | Profile/cache/render paths retain exact per-mount fingerprints, isolate server profiles, coexist across rotation, deny revocation, and remove only the requested tuple. |
  | REQ-010 | PASS | Process/file locks, database advisory locks, persisted-state reconciliation, immutable snapshots, and race/restart regressions linearize trust and cartridge effects. |
  | REQ-011 | PASS | Root-authenticated exact-platform package metadata can stream only into bounded private mode-0600 non-executable staging with exact size/digest and aggregate-capacity checks. |
  | REQ-012 | PASS | Production QML shows exact channel/version/digest/build provenance and copy-only install text; process-spawn checks prove it invokes no shell, sudo, pacman, or installer. |
  | REQ-013 | PASS | Enrollment, synchronization, rotation, revocation, expiry, offline, download, ready, and retry states are explicit while safe social/game access remains available. |
  | REQ-014 | PASS | No-key and explicit manual-key modes remain compatible; existing server/client/QML/package/provider suites pass and mixed trust fails closed. |
  | REQ-015 | PASS | SDK identity remained locked, native artifacts/docs/OpenWiki were reconciled, focused checks passed, and the canonical diff gate produced a matching receipt. |
- Docs: OpenWiki lifecycle
  `c6c9b71f-32e0-4eb1-a901-2c511ba2e626` completed for quickstart,
  game-cartridges, runtime-foundation, development-and-validation, and
  product-boundaries. It explicitly retained unresolved Claims evidence debt
  on those five pages. README, API, cartridge/system architecture,
  client-installation, owner-operator, and roadmap docs now describe the same
  root-channel, rotation, acquisition-v2, and non-privileged staging contract.
- AAR: `AAR-036` is submitted and effective. It records seven durable failure
  IDs, six prevention rules, one architecture decision, the final clean sealed
  security scan, and the completed OpenWiki lifecycle; every new ID is present
  in the knowledge register.
- Archive: Ticket 036, this spec, and these notes move to their closed/completed
  stores before the final delivery gate. No active spec/notes pair or open
  ticket remains afterward.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Acquisition and render initially allowed current lifecycle authority to be inferred from unsigned or retired-key evidence. | Historical provenance and current-use policy were represented as one trust claim. | Acquisition v2 carries a separately signed current snapshot; server/client require its exact active key and current version. | `PR-omarchy-gaming-system-bind-current-policy-to-signed-current-snapshot-001` |
| 2 | Live client/server processes could retain older trust after another authorized process persisted rotation or revocation. | Startup/process-local immutable state was not reconciled at each effect boundary. | Client operations reconcile the descriptor-bound store; server operations read persisted trust in the same database snapshot. | `PR-omarchy-gaming-system-reconcile-persisted-trust-before-effects-001` |
| 3 | Package bootstrap verification and later package use originally reread a mutable caller path. | Verification and use did not share one owned byte snapshot. | The builder copies once into private storage and verifies, hashes, and packages only that copy. | `PR-omarchy-gaming-system-snapshot-mutable-build-inputs-before-verification-001` |
| 4 | A first-run client could accept an obsolete still-valid root-signed bundle, and a raised package floor initially hid terminal history from transition checks. | Eligibility freshness and authenticated transition history were conflated. | Package bootstrap authenticates minimum floors; below-floor trust is unusable but still constrains every replacement. | `PR-omarchy-gaming-system-bind-fresh-enrollment-to-package-floors-001`; `PR-omarchy-gaming-system-preserve-ineligible-trust-as-transition-evidence-001` |
| 5 | Migration 0023 initially backfilled historical policy provenance from the current singleton. | Global current state was substituted for each retained row's older identity. | Backfill uses each release's `last_seen_snapshot_version`; a schema-through-0022 upgrade regression proves it. | `PR-omarchy-gaming-system-backfill-history-from-row-local-provenance-001` |
| 6 | Database and runtime inspection found policy/key mutation and trust rollback paths across rotation. | New rotation fields were additive without complete equal-version and persisted-continuity invariants. | Application predicates, triggers, root continuity, exact signed snapshots, and stale-runtime checks now fail closed. | Exact-key/version immutability and persisted-state regressions in migration/server suites. |
| 7 | Bounded staging and network review found DNS deadline, already-staged membership, and same-process aggregate-download gaps. | Individual byte checks existed without complete operation-level admission/currentness. | DNS shares the total deadline, staged artifacts recheck current metadata, and same-process downloads serialize before bounded publication. | Guarded transport and package-stage hostile/concurrency corpus. |
| 8 | The first full gate rejected the provider pilot after migration 0023 strengthened provenance checks. | The pilot duplicated a direct lifecycle SQL write and omitted snapshot-version advancement. | It now reuses the shared lifecycle-publication fixture under the advisory lock; the focused pilot and complete gate pass. | Keep lifecycle fixtures on the same monotonic publication helper exercised by cartridge action tests. |
