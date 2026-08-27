---
title: Session-pinned cartridge render plan and gameplay launch — notes
pipeline_id: 68a0691d-8e6d-48d0-83a1-8c43c6b68b29
---

# Session-pinned cartridge render plan and gameplay launch — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 033 is delivered at `41f2fa3`; local and remote commit/tree
  identities matched and the worktree was clean before Ticket 034 opened.
- Recall: no active bulletin or pipeline blocked the work. PostgreSQL is
  healthy, and `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0,
  OpenWiki 0.3.3, and Codex-only provenance active.
- Recall: Ticket 033's profile mount is an exact independently verified local
  presentation fact, not a game session, render plan, or launch grant. The
  mount binds stable server UUID/origin, trusted marketplace-key fingerprint,
  publisher/game/rules/cartridge identity, archive digest, lifecycle policy,
  and server admission revision while leaving cached content inert.
- Recall: the production renderer already accepts only a
  `VerifiedCartridge`, validates a bounded schema-pinned view, incrementally
  charges resource budgets, and emits inert `omarchygs.render-plan/v1` tags
  plus digest assets. `TrustedCartridgeSurface.qml` independently validates
  plan keys, origin, nodes, preferences, and aggregate profile bounds before
  instantiating repository-owned components.
- Recall: the server's compiled and registered-provider session paths already
  enforce one exact authority, participant authorization, durable
  idempotency, expected revisions, provider lifecycle, and REST recovery, but
  session rows currently pin no marketplace presentation release.
- Recall: the clean-room Door Legends v1 cartridge declares the exact
  `welcome`, `status`, and `enter_label` schema produced by the separately
  deployed provider and one empty-payload `enter` button action. This is a real
  already-proven release/provider seam, not a new fixture-only protocol.
- Recalled prevention rules:
  `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001`,
  `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001`,
  `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001`,
  `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001`,
  `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001`,
  `PR-omarchy-gaming-system-bind-presentation-cardinality-before-indexing-001`,
  and `PR-omarchy-gaming-system-inventory-callers-after-exact-contract-break-001`.
- Decision: pin a nullable exact marketplace release and admission revision
  in each eligible new platform session; absence remains a compatible honest
  state, while later catalog updates never mutate the pin.
- Decision: the client companion resolves the profile mount and cached signed
  content, compiles the production plan, and owns bounded ephemeral asset
  authority. QML receives a plan and host-created asset root, never a cache
  path, publisher program, marketplace selector, provider URL, or device
  credential.
- Decision: introduce a cartridge-action route whose exact origin/action/body
  is checked against the session pin and verified entry-screen declaration
  before adapting it into the existing compiled/provider command path.
- Decision: make Door Legends the first portable playable proof and retain the
  Signal Siege platform presenter as the compatibility fallback.
- Phase 1 is PASS. Fourteen observable requirements define one vertical trust
  boundary from server session pin through mounted rendering and declared
  action dispatch without expanding cartridge execution authority.

## Phase 2 — Design

- Architecture and data flow:
  - Migration `0021` adds an immutable one-to-zero-or-one
    `game_session_cartridge_presentations` row rather than mutable cartridge
    columns on `game_sessions`. The row references the retained marketplace
    release, snapshots its admission revision, and is insert-only. A database
    trigger verifies that the release game/rules identity equals its session
    and that the exact catalog row/revision is current at insertion; missing
    presentation remains the backward-compatible state.
  - New-session creation receives the optional configured
    `CartridgeDistributionRuntime`. While holding the existing session
    transaction, it share-locks the exact current catalog/release/snapshot,
    requires imported/compatible/current `active|deprecated` admission and—
    for a registered provider—the provider release's immutable cartridge
    digest, verifies the database marketplace key against runtime
    configuration, re-resolves the exact secure-store release for
    `LifecycleUse::NewLaunch`, then inserts the presentation pin. No runtime,
    no exact release, or a digest mismatch produces no pin rather than a false
    binding or a fallback release.
  - Compiled solo creation, compiled challenge acceptance, and provider solo
    creation call the same pin primitive before their surrounding transaction
    commits. Durable start replay returns the originally pinned or unpinned
    session before consulting current catalog state. Catalog updates share the
    same locked rows, so pinning linearizes either before or after an exact
    admission transition and never combines identities.
  - Participant session queries left-join the immutable pin and current signed
    lifecycle record. The additive exact `presentation` object is null or
    `omarchygs.session-cartridge/v1` with publisher/game/rules/cartridge,
    archive/signed-identity digest, pinned admission revision,
    `active|deprecated|suspended|revoked|retired` lifecycle, and the derived
    `continue|suspend|terminate` active-session decision. It exposes no
    marketplace/publisher key, operator reason, path, credential, provider
    endpoint, or internal release UUID.
  - A new participant route
    `POST /v1/personas/{persona_id}/game-sessions/{session_id}/cartridge-actions`
    accepts exactly `idempotency_key`, `expected_revision`,
    `archive_sha256`, `action`, and object `payload`. The domain reauthorizes
    the participant, loads the immutable pin, requires the request digest,
    rechecks the runtime/database marketplace key, current signed
    active-session lifecycle, publisher release and secure-store bytes, and
    validates the action against the signed v1 entry screen. Button actions
    require `{}`; Grid actions require exact bounded integer `column,row` in
    the signed grid dimensions. Unused, unknown, differently shaped, or
    cross-screen actions are denied.
  - After validation, the server constructs the command itself. The v1
    registered-provider shape is the exact flat `{action, ...payload}` already
    consumed by Door Legends; the compiled path uses that baseline with the
    existing Signal Siege `{kind:"play",action}` adapter. It invokes the
    existing compiled/provider command functions with the same idempotency key
    and expected revision. Those functions remain the sole state/revision
    authorities; the new route returns their bounded receipt plus the pinned
    archive digest and QML immediately refetches REST truth.
  - `SecureCartridgeStore` gains a descriptor-relative cached-policy exact
    resolver. It reads the already retained digest-named signed policy through
    the checked policy descriptor, authenticates it with the client-controlled
    marketplace key, applies `LifecycleUse::ActiveSession`, and then performs
    the existing exact archive/release/conformance verification. It does not
    add a pathname or permissive resolver.
  - Ticket 033 mounts did not need to retain the publisher public key required
    for later exact resolution. The client cache therefore adds a private
    descriptor-relative immutable `publisher-keys/<archive>.json` record on
    successful acquisition, while leaving the public v1 mount/profile schema
    unchanged. Existing pre-034 mounts continue to list safely but cannot
    render until that exact release is reinstalled; no server-supplied key is
    accepted at render time.
  - The companion adds an authenticated, body-bounded
    `POST /v1/render-plans`. Its strict request binds canonical selected origin
    and server UUID, session UUID/revision, the complete server presentation
    binding, authoritative object view, one fixed surface state, and bounded
    trusted preferences. The cache requires one exact profile mount, matching
    key fingerprint/admission/identity and private publisher key, resolves the
    immutable release under cached active-session policy, and calls the
    production renderer on the signed entry screen.
  - The render response is
    `omarchygs.session-cartridge-render/v1` and binds session UUID/revision,
    archive digest, one host-created `asset_root`, and the unchanged production
    render plan. Assets stay in companion memory behind a random 256-bit
    per-plan loopback capability. Asset GET requires the exact expected Host,
    exact random capability, digest filename, allowlisted PNG/WAV media, and
    returns only authenticated bytes with `nosniff`/`no-store`; global plan,
    asset-count, byte, age, and least-recent eviction limits prevent retained
    memory growth. The capability URL is created from the companion's bound
    loopback address, never from server or cartridge data, and expires with
    eviction/process exit.
  - `GameController.qml` keeps the selected-server bearer in the existing
    session-owned request gateway and uses a separate helper API containing
    only the companion credential. A valid bound session triggers render-plan
    preparation after the authoritative REST response. The controller accepts
    only a session/revision/digest-matching helper envelope and passes the plan
    and asset root to `TrustedCartridgeSurface`; that surface retains its
    independent exact-key/node/budget validation before instantiation.
  - Gameplay prefers an accepted bound cartridge plan. A missing helper,
    trust key, publisher-key upgrade record, matching mount, compatible plan,
    or allowed lifecycle produces an explicit platform-owned state and no
    action. An unbound or unavailable cartridge preserves the existing Signal
    Siege presenter where supported. Cartridge actions are disabled while
    loading, offline/stale, completed, not the actor's turn, or during any
    request; transport uncertainty retains the exact idempotency identity for
    explicit retry and authoritative refresh.
  - Door Legends supplies the executable proof: its clean-room cartridge's
    `welcome/status/enter_label` schema matches the authenticated provider
    view and its empty-payload `enter` button maps to the provider's existing
    command. The provider remains the sole durable gameplay owner and the
    client loads no Door Legends QML, JavaScript, native library, endpoint, or
    credential.
- Database and compatibility consequences:
  - `game_session_cartridge_presentations` is additive and absent for every
    legacy row. The referenced marketplace release identity is already
    immutable and nondeletable; the presentation row additionally forbids
    update/delete/truncate and checks positive admission revision plus exact
    identity at insert.
  - The session JSON contract additively gains mandatory `presentation` with
    null for unbound rows. The exact QML and test validators update together.
    Existing `/commands`, `/v1/games`, challenge, provider, and Signal Siege
    behavior remains supported.
  - New sessions never fail merely because no cartridge runtime/selection
    exists. If an exact selected release exists but its retained evidence is
    corrupt, pinning fails closed and the session transaction fails rather
    than recording an unverified binding.
  - Current profile mounts remain canonical v1 documents. Pre-034 mounts lack
    only the new private publisher-key side record and receive a clear local
    reinstall requirement; acquisition/update writes it atomically before
    publishing the mount.
- API contracts:
  - Session `presentation` has exact keys `format`, `publisher_id`,
    `game_key`, `rules_version`, `cartridge_version`, `archive_sha256`,
    `signed_identity_sha256`, `admission_revision`, `lifecycle_status`,
    `active_session_policy`, and optional `warning` only for deprecated
    releases.
  - Cartridge action request has exact keys `idempotency_key`,
    `expected_revision`, `archive_sha256`, `action`, `payload`. Its response
    has the existing command receipt fields plus `archive_sha256`; all are
    `Cache-Control: no-store` and existing stable game error codes are reused
    where possible, with one non-disclosing `game_cartridge_unavailable` code
    for pin/evidence/lifecycle/action denial.
  - Companion render request/response and asset routes are loopback-only
    local protocols. Render requires the existing helper bearer and exact
    Host. Asset fetch intentionally uses only its unguessable per-plan
    capability because Qt image/audio loaders cannot attach the helper bearer;
    assets are already public cartridge bytes and the URL contains no cache
    path, session credential, or server token.
- Exact file manifest:

  | Surface | Planned change |
  |---|---|
  | `migrations/0021_game_session_cartridge_presentations.sql` | Add immutable exact session-presentation pins, insertion validation, and indexes. |
  | `crates/server/src/session_cartridges.rs`, `lib.rs` | Own new-session pinning, participant projection, secure exact action validation, and authority command translation. |
  | `crates/server/src/games.rs`, `challenges.rs`, `provider_games.rs` | Invoke optional pinning in every session-creation path, load the presentation projection, and preserve command/replay authority. |
  | `crates/server/src/app.rs`, `server_discovery.rs` | Add the exact session JSON field, conditional capability, cartridge-action route/request/response, and runtime wiring. |
  | Server game/challenge/provider/cartridge API tests | Cover schema, pin/no-pin/mismatch, concurrency, lifecycle, privacy, replay, and real Door Legends dispatch. |
  | `crates/game-cartridge/src/secure_store.rs`, `contract.rs` or a narrow action module, exports/tests | Resolve exact cached signed policy and validate one signed entry-screen action/emitter payload. |
  | `crates/client-cartridge-runtime/src/cache.rs`, new `render.rs`, `service.rs`, `lib.rs`, `Cargo.toml` | Retain private publisher keys, resolve mounted releases, compile session plans, bound ephemeral assets, and expose local routes. |
  | `Cargo.toml`, `Cargo.lock` | Link the production renderer into the native companion workspace member. |
  | `client/qml/GameController.qml`, `Main.qml`, `screens/GameplayScreen.qml`, `cartridge/TrustedCartridgeSurface.qml` | Wire helper authority, strict render envelopes, trusted cartridge preference/fallback states, disabled actions, and server action dispatch. |
  | `client/qml/tests/fixture_server.py`, `tests/fixture/tst_games.qml`, live fixture if required | Exercise exact QML render/action/fallback/hostile/minimum-layout behavior. |
  | `examples/first-party-door-legends/`, provider pilot test/script | Compose the existing clean-room signed cartridge digest, server catalog, provider release, companion renderer, action, completion, restart, and recovery evidence without a platform path dependency. |
  | Package source/build/test manifests and scripts | Prove the renderer-linked companion and unchanged reviewed QML inventory ship reproducibly and clean up local authority. |
  | `README.md`, `docs/api.md`, client/operator and architecture docs, roadmap, OpenWiki | Document the first playable portable cartridge, trust/action/lifecycle behavior, configuration, limitations, and remaining work. |
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Migration trigger/check corpus plus compiled solo, challenge, and provider insert/select tests under exact and concurrent catalog transitions. |
  | REQ-002 | Provider pilot registration digest equal/absent/different cases assert exact pin or null while provider authority remains unchanged. |
  | REQ-003 | Immutable-row tests plus upgrade/rollback/omission and all five signed lifecycle statuses prove no repin and explicit active-session decisions. |
  | REQ-004 | Participant/foreign session list/detail exact-schema tests and negative secret/path/key/endpoint inventory. |
  | REQ-005 | Cache tests cover missing/changed private publisher key, marketplace fingerprint, server origin/UUID, all identity/admission fields, signed policy, symlink/race/restart, and legacy v1 mounts. |
  | REQ-006 | Real Door Legends mounted view compiles through the production renderer; malformed/oversized view, preference, profile, identity, and lifecycle cases fail before plan publication. |
  | REQ-007 | In-process companion tests fetch valid PNG/WAV then reject wrong Host/capability/token/digest/type/path, expired/evicted plans, excess counts/bytes, and concurrent replacement. |
  | REQ-008 | QML fixtures cover plan loading/acceptance, platform fallback, missing helper/key/mount, legacy mount, every fixed state, completion, focus/keyboard/accessibility/plain text, and 640×420/920×600 geometry. |
  | REQ-009 | QML captures exact empty Button and Grid payload requests, busy/turn/completion denial, uncertain same-ID retry, and authority reset on server/persona/session changes. |
  | REQ-010 | Shared action unit corpus plus PostgreSQL foreign/digest/action/payload/screen/policy tampering and successful compiled/provider dispatch. |
  | REQ-011 | Existing command regressions and new route exact replay/collision/revision/timeout/refetch tests prove no second effect or guessed state. |
  | REQ-012 | Extended clean-clone Door Legends pilot plus client-runtime/QML vertical proves signed mount → authenticated provider view → trusted plan → `enter` → provider terminal receipt → restart/readback. |
  | REQ-013 | Existing server/QML suites, unconfigured runtime and null-binding fixtures, catalog-only capability, Signal Siege solo/versus, and legacy rows remain green. |
  | REQ-014 | Workspace/cartridge/renderer/client/server/QML/provider/package focused suites followed by the complete canonical diff gate. |
- Security, privacy, concurrency, and rollback risks:
  - Trust substitution: the client uses only its configured marketplace key,
    the profile fingerprint, a publisher key privately retained from the
    verified acquisition, and the cached signed monotonic policy. Neither QML
    nor a session response supplies a verifier key.
  - Session/catalog race: presentation insertion share-locks the same exact
    catalog/release state updated by synchronization/admission and happens in
    the session transaction. The row is immutable, so later selection changes
    cannot rewrite history.
  - Stale or forged action: session participation, immutable digest, current
    signed active-session policy, verified entry-screen action, exact emitter
    payload, expected revision, and idempotency identity are all required
    before the existing authority receives a host-built command.
  - Asset capability leakage: asset bytes are public inert cartridge content;
    URLs are random per plan, loopback Host-bound, short-lived/evicted,
    non-loggable, no-store, nosniff, and reveal no filesystem or credential.
  - Memory/GPU denial: render request/response and view limits precede JSON;
    the renderer charges all plan resources; the asset cache independently
    caps plans/count/bytes/age; QML independently recounts scene budgets.
  - Lifecycle race: new work uses `NewLaunch`; render/action uses
    `ActiveSession`. Deprecated/retired may continue pinned sessions,
    suspended denies while preserving state, revoked terminates presentation
    authority, and omission never selects a substitute. Existing gameplay
    authority lifecycle remains an additional required gate.
  - Rollback: migration is additive and legacy rows are null-bound. Disabling
    distribution stops new pins and cartridge actions while retaining
    platform presenters and read-only session history. Reverting the client
    leaves bindings inert; removing a profile mount never deletes session or
    provider state.
- Alternatives rejected:
  - Letting QML open cached archives or compile presentation would move
    publisher parsing, filesystem authority, and trust-root use into the
    JavaScript runtime.
  - Returning a server-built render plan would make the server—not the
    independently verified local mount—the frontend authority and would lose
    client profile/resource compatibility.
  - Trusting an action because it appeared in a QML plan would make the client
    the only authorization boundary; the server must re-resolve the signed
    entry-screen contract before its gameplay authority sees the command.
  - Pinning only game key/rules version or looking up current selection on each
    read would silently relabel sessions after upgrade/rollback.
  - Loading publisher QML/JavaScript, provider Web content, or direct provider
    URLs remains prohibited and is unnecessary for the Door Legends proof.
  - Automatically downloading a historical release while opening a session
    requires a distinct participant-scoped distribution and multi-mount
    retention contract; this slice fails clearly and asks for reinstall
    rather than bypassing Ticket 033 admission semantics.
- CodeGraph design evidence: pipeline
  `68a0691d-8e6d-48d0-83a1-8c43c6b68b29` traced the compiled solo and
  challenge session insertion paths, provider envelope creation before
  network launch, replay exits, participant session projections, current
  catalog selection, exact secure-store distribution, client mount staging,
  cached content resolution, production plan compilation, companion routes,
  and the QML detail/action surfaces. Its blast-radius hints identified
  `games.rs`, `provider_games.rs`, `challenges.rs`, `app.rs`,
  `cartridge_catalog.rs`, `cartridge_distribution.rs`, `secure_store.rs`,
  `cache.rs`, `service.rs`, and renderer callers; QML, SQL, shell, package
  metadata, and exact fixture inventories remain direct-review surfaces.
- Phase 2 is PASS. The design maps all fourteen EARS requirements to
  executable evidence, keeps every authority singular, and introduces no
  cartridge execution, credential, cache-path, or direct network capability.

## Phase 3 — Implement

- Built:
  - Added migration `0021` with one immutable exact release/admission pin per
    session, insertion-time catalog/session identity checks, and bound-session
    authority identity protection.
  - Added server pin, projection, signed entry-screen action validation, exact
    action dispatch, conditional discovery capability, and runtime wiring for
    compiled solo, accepted challenge, and registered-provider starts.
  - Added cached exact-policy resolution, private publisher-key retention,
    mounted-release render compilation, authenticated render-plan service, and
    bounded Host/capability/digest asset delivery in the native companion.
  - Wired exact presentation/render/action contracts into the trusted QML
    controller and presenter while preserving the Signal Siege fallback.
  - Extended the clean-provider Door Legends pilot to build and mount a real
    signed cartridge, pin its exact digest, compile its production plan, send
    `enter` through the cartridge route, and retain provider restart/backup
    recovery proof.
  - Added hostile digest, payload, participant, response, body-size, Host,
    capability, asset-budget, and immutable-identity coverage.
- Focused implementation evidence:
  - `cargo check --workspace --all-targets` passed after the final negative
    cases; `cargo fmt --all -- --check` passed.
  - Client-runtime library tests passed 9/9; the QML fixture passed 47/47; the
    upgraded clean-provider authority pilot passed before its final negative
    assertions were added, so the canonical gate must rerun the complete
    vertical.
  - PostgreSQL migration/application coverage passed except for one stale
    pre-feature exact-key assertion; that assertion was corrected and its
    focused test passed. The full database suite remains a Phase 4 gate item.
- Deviations:
  - The v1 render helper request does not carry session UUID/revision because
    the companion is not a gameplay authority and cannot authenticate those
    server facts. It binds the exact server mount, game/release digest,
    admission revision, independently trusted marketplace key, signed active
    policy, schema-checked view, and returned plan origin; QML and the action
    route separately bind the authoritative session/revision.
  - V1 historical-release auto-acquisition remains out of scope. A bound
    session whose exact release is not mounted fails clearly without selecting
    a current substitute.

- Phase 3 is PASS; implementation is complete and the worktree is in the
  independent inspection phase.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security diff / origin binding | A connect-once origin could claim an existing server UUID and request a trusted render plan from that UUID's mount because the render request did not carry the current canonical origin. | Low / P3 | Fixed. QML now sends the selected canonical origin, the companion canonicalizes it, mount resolution requires exact stored-origin equality, and one UUID profile cannot mix server IDs or origins. Hostile cross-origin runtime and profile tests pass. |
| 2 | Security diff / compiled lifecycle | Signed lifecycle validation and the compiled command transaction had no durable authorization linearization point, allowing an admitted action to straddle a later suspension/revocation. | Low / P3 | Fixed. The server now writes one immutable exact cartridge-action admission while holding the shared marketplace-snapshot advisory lock and locked expected session revision; the lock ends before compiled execution. Exact replay/collision and action-first post-transition recovery tests pass. |
| 3 | Security diff / provider lifecycle | Provider dispatch had the same lifecycle gap and an exact retry after transport uncertainty could be denied before provider idempotency recovered a pre-transition operation. | Low / P3 | Fixed. Provider and compiled paths share the durable admission primitive, return the stored host-translated command on exact replay, and branch on the immutable admitted authority. Writer-first denial and provider post-suspension replay coverage pass. |
| 4 | Defense in depth / asset memory | `RenderAssetCache::get` cloned each retained `Vec<u8>` for a response. The same-user capability model and bounded plan/cache did not establish a reportable remote attack path, but concurrent reads could temporarily amplify resident memory. | Informational | Hardened. Cached assets are converted once to reference-counted Axum `Bytes`; response clones are constant-time shared views while retained count/byte/age bounds remain unchanged. |
| 5 | Canonical gate / trusted preview | `TrustedCartridgeSurface` correctly separated untrusted input from `acceptedPlan`, but the standalone cartridge preview still dereferenced the now-unset raw `renderPlan` after calling `acceptPlan` directly. | Medium regression | Fixed. The preview consumes only `acceptedPlan` for smoke assertions and metrics. The complete renderer/QML profile, state, input, accessibility, rejection, and memory harness passes. The gate also caught and fixed test-module placement and one formatting drift. |

- Codex Security reviewed every one of the 34 changed or directly supporting
  Rust, SQL, QML, and fixture files in full. The sealed pre-fix diff scan
  reported exactly the three low findings above; no credential disclosure,
  unsigned content, arbitrary bytes, executable cartridge path, cross-user
  authority, or additional reportable finding was established.
- Focused remediation evidence:
  - `cargo check --workspace --all-targets` passed after the admission and
    origin changes.
  - `cargo test -p omarchygs-client-cartridge-runtime` passed 9/9, including
    cross-origin render denial and profile server/origin isolation.
  - The focused PostgreSQL durable admission/revocation test and the
    writer-first advisory-lock ordering test passed.
  - `./scripts/test-qml-onboarding.sh` passed 47/47 after QML began supplying
    the selected server origin.
- A fresh worktree-bound CodeGraph inspection traced mount selection and the
  action admission/authority blast radius after the final gated fix. No new
  blocker or missing caller was found; direct inspection covered QML, SQL,
  tests, and untracked sources outside CodeGraph's indexed-symbol support.
- Phase 3.5 is PASS. All validated findings are fixed with regression evidence
  and the worktree is ready for the complete Phase 4 validation matrix.

## Phase 4 — Validate

- Tests run:
  - The first canonical `bin/gate.sh --diff` completed all 22 stages. Stages
    3-11 and 13-22 passed, including the complete workspace, reproducible
    package, 55 PostgreSQL API tests, live QML smoke, provider conformance,
    clean-clone Door Legends authority pilot, backup/restore, and private-alpha
    drills.
  - Focused repair validation passed `cargo clippy --workspace --all-targets
    -- -D warnings` and the complete trusted Game Cartridge renderer harness.
- Gate run: the first pass was RED with three local validation failures:
  rustfmt on the latest provider assertion, Clippy's test-module placement
  rule, and the standalone preview's stale raw-plan dereference. All three
  were corrected.
- Gate run: the second canonical `bin/gate.sh --diff` passed all 22 stages and
  wrote a receipt whose hash exactly matched the validated worktree. This
  reran formatting, Clippy, all workspace tests, rustdoc, Compose validation,
  pipeline/hook/secret/whitespace checks, every cartridge and client package
  proof, all 55 PostgreSQL integration tests, live PostgreSQL/Rust/QML smoke,
  remote-provider conformance, the clean-clone Door Legends authority pilot,
  operator backup/restore, and private-alpha admission.
- Skips or pre-existing failures: none.
- Phase 4 is PASS. The complete canonical validation matrix is green against
  the inspected implementation and Phase 5 documentation/completion work may
  proceed.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Completion evidence |
  |---|---|---|
  | REQ-001 | PASS | Migration `0021` insertion/immutability triggers plus compiled solo, challenge, provider, mismatch, and snapshot-lock PostgreSQL tests prove an atomic optional exact pin. |
  | REQ-002 | PASS | Provider launch tests prove exact provider/catalog digest equality pins one release, while absent or mismatched presentation stays null without changing provider authority. |
  | REQ-003 | PASS | Immutable-row, upgrade/rollback/omission, all-five-lifecycle, action-first, and writer-first tests prove no repin and explicit active-session policy. |
  | REQ-004 | PASS | Session list/detail exact-schema and privacy tests expose only the bounded `presentation` identity/policy and exclude keys, paths, credentials, endpoints, grants, operator reasons, and private authority. |
  | REQ-005 | PASS | Nine client-runtime tests cover exact origin/UUID profile isolation, marketplace and publisher keys, identity/digest/revision, signed policy, lifecycle, restart, and hostile cache/mount cases. |
  | REQ-006 | PASS | Production renderer and companion tests compile the real Door Legends entry screen and reject malformed view, preference, profile, identity, lifecycle, and incompatible plans. |
  | REQ-007 | PASS | Companion tests prove exact Host/capability/digest/media admission, body and retained plan/count/byte/age bounds, concurrent eviction, shutdown, and reference-counted response bytes. |
  | REQ-008 | PASS | The 47-test QML fixture plus complete renderer harness prove trusted-plan preference, platform fallback, fixed failure states, hostile-envelope rejection, keyboard/focus/accessibility/plain text, and minimum layouts. |
  | REQ-009 | PASS | QML fixtures capture the exact session revision/digest/action/payload request, busy/turn/completion denial, selected-server routing, and same-identity uncertain retry. |
  | REQ-010 | PASS | Shared action validation and PostgreSQL API tests reject foreign sessions, digest/action/payload/screen/policy tampering and prove both host-translated compiled and provider paths. |
  | REQ-011 | PASS | Existing command replay plus immutable cartridge-admission collision/revision/completion, post-suspension replay, timeout/reconcile, and QML refetch tests preserve authoritative REST truth. |
  | REQ-012 | PASS | The clean-clone Door Legends pilot builds the independent cartridge/provider, pins and mounts its exact digest, compiles the authenticated view, invokes `enter`, reaches provider terminal state, and proves restart/backup recovery without publisher code. |
  | REQ-013 | PASS | Null-binding, catalog-only, missing-helper/key/mount, legacy-row, Signal Siege solo/versus, provider, discovery, and existing QML/server suites remain green with fail-closed fallback. |
  | REQ-014 | PASS | Package source/build/smoke, workspace/cartridge/renderer/SDK/provider/database/QML/recovery/private-alpha checks and the second canonical 22-stage diff gate passed with a matching worktree receipt. |

- Docs: OpenWiki update run `b6462295-f3f7-4069-b538-70712799e347`
  completed after reconciling quickstart, cartridge, runtime, validation, and
  product-boundary pages. It preserved four Claims sidecars because of
  pre-existing unresolved evidence debt and reported that warning explicitly.
  `README.md`, API, client installation, system/cartridge architecture,
  owner-operator guidance, and roadmap now document the implemented launch,
  action, lifecycle, security, limitations, and Door Legends vertical.
- AAR: submitted as effective with four failures, three prevention rules, and
  one architecture decision appended to the knowledge register.
- Archive: Ticket 034 moves to `tickets/closed/` and the sole active spec/notes
  pair moves to `pipeline/completed/`; no active pipeline remains.
- Phase 5 is PASS. All fourteen requirements are accounted for, durable docs
  and lessons are reconciled, and the pipeline is ready for authorized
  delivery.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A different selected origin could reuse a trusted mount for the same claimed server UUID. | The render request and mount resolver bound server UUID but not the canonical current origin. | Added `server_origin`, canonicalized it, required exact origin/UUID/profile equality, and rejected mixed-origin profiles. | Bind mount authority to canonical origin and stable server identity together. |
| 2 | A compiled cartridge action could validate before a concurrent lifecycle transition and execute after it. | Lifecycle verification and command execution had no durable linearization point. | Added immutable exact action admission under the shared marketplace snapshot lock before compiled execution. | Persist exact authorization before external or deferred effects. |
| 3 | Provider action replay could be denied after suspension even when the first operation had already been admitted. | Current lifecycle was rechecked before any durable record of pre-transition intent. | Shared the immutable admission primitive across authorities and resolve exact replay before current-policy denial. | Separate authorization of new work from recovery of an already admitted idempotent operation. |
| 4 | The standalone preview dereferenced null raw plan input after direct accepted-plan setup. | The trusted surface separated raw and accepted state, but one preview consumer retained the old authority assumption. | Made preview assertions and metrics consume only `acceptedPlan`; the full renderer/QML harness passes. | Every post-validation consumer must use accepted state only. |
| 5 | The first canonical gate also rejected one formatting drift and a test module placed before later items. | Final focused edits were not yet checked by the repository-wide formatting and warning-denied layout rules. | Ran rustfmt, moved the test module to the file end, reran Clippy and the complete gate. | Preserve final workspace-wide format/lint evidence after focused repairs. |
