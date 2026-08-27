---
title: Historical session cartridge acquisition and multi-screen navigation — notes
pipeline_id: e6c0e63b-200a-481d-8670-8531db96661f
---

# Historical session cartridge acquisition and multi-screen navigation — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 034 is delivered at `4fecef5`; local and remote commit/tree
  identities matched and the worktree was clean before Ticket 035 opened.
- Recall: no active bulletin, pipeline, or ticket blocked new work. PostgreSQL
  is healthy, and `scripts/check-pipeline-tools.sh` passed with CodeGraph
  1.5.0, OpenWiki 0.3.3, and Codex-only provenance active.
- Recall: Ticket 032 retains only the newest signed marketplace snapshot in
  the singleton sync row. Historical `marketplace_releases` rows remain, but a
  later omission means the server lacks the old signed snapshot bytes required
  by the v1 acquisition verifier.
- Recall: Ticket 033 acquisition independently authenticates discovery,
  current catalog selection before and after transfer, a client-controlled
  marketplace key, the signed snapshot, publisher release, policy, bytes,
  SDK/host compatibility, and an exact server-scoped mount. It deliberately
  cannot install a release absent from the current catalog.
- Recall: Ticket 034 immutably pins a release/admission revision to an eligible
  session and renders only an already matching local mount. The server action
  path validates only the signed entry screen, and the client exposes an
  explicit unavailable state rather than auto-downloading an absent mount.
- Recall: the cartridge contract already verifies up to 32 signed screens and
  the production renderer already accepts an exact optional `screen_id`, but
  the action contract, companion request/response, and QML controller expose
  only the entry screen. Existing action definitions do not distinguish local
  navigation from gameplay.
- Recall: retained session state remains authoritative REST data. Each signed
  screen already names its own restricted schema, so the same provider or
  compiled view can be independently validated for each presentation.
- Recalled prevention rules:
  `PR-omarchy-gaming-system-bind-profile-mounts-to-origin-and-server-001`,
  `PR-omarchy-gaming-system-persist-action-admission-before-external-effects-001`,
  `PR-omarchy-gaming-system-render-only-from-accepted-plan-state-001`,
  `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001`,
  `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001`,
  `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001`,
  and `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001`.
- Decision: retain bounded signed snapshot evidence for each reviewed release
  and use it only as cryptographic acquisition proof; current signed lifecycle
  policy and the immutable session pin still decide whether delivery is
  allowed.
- Decision: expose a participant-authorized session acquisition route and an
  explicit QML install control. The companion validates the exact session pin
  before and after transfer under its client-controlled trust root; neither
  side selects the current catalog release.
- Decision: add a distinct signed host-navigation declaration. Only reviewed
  button emitters can select an existing screen with no payload, URL, network,
  or backend effect. Gameplay actions remain separate and gain exact screen
  binding through the server boundary.
- Decision: scope a bounded QML navigation stack to one session/release and
  compile every destination in the companion against authoritative REST view
  data. Door Legends becomes the clean historical multi-screen proof.
- CodeGraph design exploration showed that Ticket 033 deliberately keys one
  profile mount per game. Historical acquisition therefore also requires a
  bounded exact multi-release mount set; otherwise installing an old session's
  cartridge would evict the current one or vice versa.
- Decision: profile mounts are keyed by game, archive digest, and admission
  revision. Current and historical releases may coexist within the existing
  bound, and exact removal retains all other mounts and shared content.
- Phase 1 is PASS. Fifteen observable requirements define one coherent
  vertical from retained marketplace evidence through explicit historical
  install, signed screen navigation, and screen-bound gameplay dispatch.

## Phase 2 — Design

- CodeGraph evidence:
  - Design explores ran while the active spec was in Phase 1 and produced a
    matching `design` receipt for pipeline
    `e6c0e63b-200a-481d-8670-8531db96661f` and the current gated worktree.
  - `publish_snapshot` is the exclusive advisory-lock writer for authenticated
    marketplace releases and the singleton signed snapshot. It already has the
    exact signed bytes, digest, key, payload, and upserted release IDs needed to
    retain normalized historical evidence in that same transaction.
  - `acquire_exact` has one Axum caller and deliberately joins only the current
    selected release/current snapshot. Historical delivery therefore needs a
    separate participant/session query; weakening the current catalog route
    would conflate two different authorization contracts.
  - `verify_acquisition_bytes` independently authenticates the server
    admission, marketplace key/snapshot entry, publisher release, conformance,
    attestation, and exact bytes. The historical path can reuse this public
    envelope without inventing a server-authored trust format.
  - Client `acquire` performs strict discovery, initial/final catalog reads,
    one same-origin acquisition, and `MountRecord::from_verified`; its new
    sibling must replace only the catalog reads with initial/final
    participant-authorized session reads.
  - `ClientCartridgeCache::resolve_mounted` already resolves the full
    origin/server/game/digest/admission tuple, but `install`, profile ordering,
    and QML inventory currently enforce one mount per game. CodeGraph's
    one-hop blast radius is `render.rs`, while direct QML inspection exposed
    the additional `CartridgeController.qml` assumption.
  - `compile_render_plan` already accepts an optional exact `screen_id` and
    validates that screen's own authenticated schema. The companion always
    passes `None`, and its exact two-field response carries no current/entry
    screen or signed navigation destinations.
  - `admit_session_action` has three handler-level dependents and calls
    `validate_entry_screen_action`; action admission/replay and provider or
    compiled dispatch are otherwise already the correct lifecycle and
    authority boundary.
  - `GameController.qml` strictly accepts the current render response, sends no
    screen identity, and clears its plan before every compile. QML is outside
    CodeGraph's reliable Rust topology, so it and the production-root fixtures
    were reviewed directly.

- Architecture and data flow:
  - Migration `0022` adds two normalized append-only evidence tables. One row
    stores each retained signed marketplace snapshot once by SHA-256 with its
    bounded bytes, version, and complete marketplace public key; a second row
    binds an exact `marketplace_release_id` to its first retained authentic
    snapshot. Immutable/no-truncate triggers prevent replacement. A guarded
    backfill links currently present legacy releases to the existing Ticket
    020 singleton evidence when it exists.
  - `publish_snapshot` retains evidence only when a payload contains a release
    that lacks a link, so ordinary policy-only snapshots do not grow storage
    indefinitely and one large snapshot is not duplicated per release. Replay
    retention performs the same repair after upgrade. Exact stored fields are
    read back on conflict; a different byte/key/version claim fails rather
    than rewriting history.
  - New session presentation pins require a retained evidence link in both the
    Rust selection query and the database insertion trigger. Existing pins
    remain readable; a legacy pin with no reconstructable evidence reports an
    explicit unavailable acquisition instead of synthesizing review proof.
  - A distribution-runtime-only route
    `GET /v1/personas/{persona_id}/game-sessions/{session_id}/cartridge-acquisition`
    authenticates the device and owned persona, then takes the shared
    marketplace advisory lock and loads only a participant-visible immutable
    session pin, its current release lifecycle, retained evidence, and stable
    server UUID. It never joins `server_cartridge_catalogs` as a selector.
  - The server requires the evidence key to equal the independently configured
    distribution key, re-resolves exact secure-store content using the current
    signed policy with `LifecycleUse::ActiveSession`, builds the existing v1
    acquisition envelope from the retained snapshot, and independently
    verifies it before returning bounded no-store JSON. Suspension/revocation
    fails even if the older retained snapshot described an allowed release.
    An acquisition that wins the shared lock may finish from that exact
    allowed policy; a writer that wins first causes denial.
  - Discovery adds `games.session-cartridge-acquisition.v1` only with the
    complete distribution runtime. The existing
    `games.cartridge-acquisition.v1` and current catalog route remain unchanged.
  - The companion adds authenticated `POST /v1/session-acquisitions`. Its body
    carries canonical origin/server UUID, device bearer, persona UUID, and
    session UUID—no release selector. It authenticates discovery, reads the
    participant session before and after the bounded same-origin acquisition,
    derives the expected admission from the immutable presentation, requires
    `continue`, independently verifies the v1 envelope with the client-owned
    marketplace key, and rejects any changed identity or denial before cache
    installation. The bearer remains zeroized and never enters cache state.
  - `MountRecord::from_session_verified` derives display/provenance/policy from
    authenticated release/snapshot evidence and server admission only from the
    exact session projection. Profile identity becomes the ordered tuple
    `(game_key, archive_sha256, admission_revision)`. Install replaces only an
    identical tuple, exact removal names all three values, and content remains
    digest-shared. Legacy profiles with one game-level mount remain valid.
  - Multi-screen navigation is an additive capability that does not rewrite
    the immutable SDK v1 export or schemas. The existing action grammar already
    admits `navigate.<screen_id>` and empty Button payload definitions. The
    verifier treats that reserved prefix as host navigation only when
    `presentation.navigation.v1` is required, the suffix names an existing
    signed screen, the complete action fits the existing identifier bound, its
    payload is empty, and every emitter is a Button. Grids, missing targets,
    payloads, and ambiguous gameplay interpretation are rejected. Cycles are
    allowed because host history is bounded.
  - `rich_2d_host_profile` and Core advertise the navigation capability. The
    old SDK lock SHA-256
    `7a732939918254ca1fb399f1fa4a4ef70d252ad683c13696dec8db8e2e88a045`
    remains a regression invariant, so already released v1 attestations stay
    verifiable. Older hosts lack the capability and fail compatibility before
    rendering a navigable release.
  - The renderer continues emitting unchanged
    `omarchygs.render-plan/v1`. `PreparedPreview` additionally exposes the
    authenticated current screen, manifest entry screen, and only reserved
    navigation actions actually emitted by Buttons on the current screen. The
    companion accepts an optional requested screen and returns exact
    `omarchygs.session-cartridge-render/v2` with those fields, the v1 plan, and
    the host-created asset root. Missing `screen_id` still means entry.
  - New QML accepts strict v2 and the old exact two-field response for a
    mixed-version entry-only fallback. It independently validates identifiers,
    entry/current screen, unique navigation mappings, targets, and that each
    mapping corresponds to a rendered Button. Only an accepted mapping can
    start local navigation. Host Back/Entry controls request signed screens
    through the companion, history is capped at 16 and scoped by
    session/digest, and focus returns deterministically to the first trusted
    control. Navigation sends no server gameplay request.
  - Cache resolution distinguishes a genuinely absent exact tuple from corrupt
    or denied content. On `companion_mount_missing`, the gameplay screen keeps
    authoritative REST state and offers `INSTALL PINNED CARTRIDGE` only when
    helper, independent trust, device authority, and the new server capability
    are present. Success validates the exact mount receipt and compiles the
    entry screen; denial/offline/protocol states remain explicit.
  - Gameplay action JSON adds optional `screen_id`. New v2 QML always sends its
    accepted current screen; old clients omit it and therefore mean the signed
    entry screen. Server validation becomes `validate_screen_action`, rejects
    reserved navigation IDs, checks only emitters on the requested signed
    screen, and persists the effective screen on every new immutable action
    admission before existing compiled/provider dispatch.
  - Migration `0022` adds nullable `screen_id` to historical admission rows but
    requires it in every new trigger-validated insertion. A legacy exact replay
    may recover its already committed same action/payload despite missing old
    screen metadata; no new effect is admitted. New rows bind and compare the
    exact screen for collision detection.
  - On authoritative refresh, QML recompiles its current screen only when the
    session/digest context is unchanged. Unknown or schema-invalid secondary
    state makes one explicit entry-screen recovery attempt; session/release
    change clears history immediately. Asset-plan eviction remains bounded by
    the existing companion cache.
  - Door Legends keeps provider rules/state separate, bumps its immutable
    cartridge release, requires `presentation.navigation.v1`, and adds a signed
    cyclic Lobby ↔ Chronicle Button path using the same authoritative view.
    The clean vertical pins that release, advances the current catalog to a
    different release, starts from an empty client cache, installs the old pin,
    navigates both ways without provider traffic, then executes and recovers
    the real provider action.

- Exact file manifest:

  | File(s) | One purpose |
  |---|---|
  | `migrations/0022_historical_session_cartridge_acquisition.sql` | Add normalized immutable snapshot/release evidence, evidence-required future presentation pins, and screen-bound future action admissions with legacy compatibility. |
  | `crates/game-cartridge/src/compatibility.rs`, `validate.rs`, `lib.rs` | Advertise and enforce the reserved signed navigation capability plus arbitrary-screen gameplay validation without changing SDK v1 bytes. |
  | `crates/game-cartridge/tests/conformance.rs` | Prove valid cycles and reject hostile namespace, target, node-family, payload, capability, and limit cases; pin the old SDK identity. |
  | `crates/game-cartridge-renderer/src/lib.rs`, `tests/rendering.rs` | Expose authenticated current/entry screen and current-screen navigation destinations while preserving render-plan v1. |
  | `crates/server/src/cartridge_catalog.rs`, `marketplace_sync_tests.rs` | Retain normalized first authentic snapshot evidence under the existing sync lock and prove replay/omission/non-rewrite behavior. |
  | `crates/server/src/cartridge_distribution.rs`, `session_cartridges.rs`, `app.rs`, `server_discovery.rs` | Serve participant-authorized exact historical acquisition, bind gameplay admissions to signed screens, register routes/errors, and advertise the additive capability. |
  | `crates/server/src/cartridge_catalog_api_tests.rs`, `provider_game_api_tests.rs`, `server_discovery_api_tests.rs` | Exercise trust/privacy/concurrency, catalog-independent acquisition, screen action denial/success/replay, capability subsets, and the Door Legends vertical. |
  | `crates/client-cartridge-runtime/src/lib.rs`, `remote.rs`, `cache.rs`, `render.rs`, `service.rs` | Add session-derived remote acquisition, exact multi-release mounts/removal, missing-mount classification, requested-screen compile, and strict v2 local response. |
  | `client/qml/OnboardingController.qml`, `GameController.qml`, `CartridgeController.qml`, `screens/GameplayScreen.qml` | Negotiate historical capability, install exact session pins, retain exact mount inventory, validate/navigate signed screens, and submit screen-bound gameplay actions. |
  | `client/qml/tests/fixture/tst_games.qml`, `fixture_server.py` | Prove keyboard/focus/history, explicit install/failure states, no-network navigation, action shape, fallback, and live capability behavior. |
  | `examples/first-party-door-legends/cartridge/manifest.json`, `presentation.json`, `README.md` | Publish the real inert multi-screen navigation example without changing provider authority. |
  | `scripts/test-provider-authority-pilot.sh` and existing cartridge/renderer/SDK/package gate scripts as assertions require | Extend clean-clone historical/navigation proof while keeping package and SDK provenance reproducible. |
  | `README.md`, `docs/api.md`, `docs/architecture/game-cartridges.md`, `docs/architecture/system-overview.md`, `docs/client-installation.md`, `docs/operators/owner-operated-servers.md`, `docs/planning/ROADMAP.md`, OpenWiki | Reconcile player/operator/API/trust behavior, compatibility, limits, and remaining work in Phase 5. |

- Database and migration consequences:
  - All new tables are forward-only, foreign-keyed, bounded, append-only, and
    non-truncatable. Signed bytes are stored once per useful snapshot rather
    than once per release or session.
  - The current singleton snapshot remains the source for current catalog
    visibility; retained evidence has no catalog-selection authority.
  - Existing presentation pins and action admissions are not rewritten.
    Current legacy releases are backfilled only from authentic evidence already
    present in the database; missing evidence stays honestly unavailable.
  - Future presentation insertion is denied without one evidence link. Future
    action insertion requires a canonical non-null screen ID, while legacy null
    rows remain immutable and replay-only.
  - The marketplace advisory lock remains the canonical order: sync writer
    exclusive; session pin/action/acquisition shared. No provider network I/O
    occurs while a database transaction is open.

- API and compatibility contract:
  - Additive public server capability and participant GET route; current
    `/v1/cartridges` and selected-release acquisition are unchanged.
  - Additive loopback companion POST route and render-response v2. Old render
    requests still default to entry and new QML strictly supports the prior
    two-field response only as an entry-only fallback.
  - Cartridge action `screen_id` is optional solely for existing entry-screen
    clients; new QML sends it. Response shape and command receipt semantics do
    not change.
  - The signed package JSON and SDK v1 lock do not change. Navigation is gated
    by a new required capability and an already schema-valid reserved action
    convention, so old hosts reject navigable releases while old releases stay
    bit-for-bit verifiable.
  - Profile and mount document formats stay v1. The previously valid sorted
    one-mount-per-game subset remains readable; new ordering extends the key to
    digest and revision. Local removal accepts the new required revision from
    packaged QML and preserves a narrow missing-field legacy behavior only for
    an old helper client.

- Regression plan:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Migration tests plus sync first-seen/replay/new-policy/omission/tamper and normalized-storage assertions. |
  | REQ-002 | Participant session-acquisition API matrix across exact old pin, foreign/null/mismatch, upgrade/rollback/omission, and lifecycle. |
  | REQ-003 | Existing acquisition verifier plus exact response/privacy and key/snapshot/release/store corruption cases. |
  | REQ-004 | Writer-first/acquisition-first shared-lock tests and secure-store newer-denial cases. |
  | REQ-005 | Companion initial/acquisition/final session fixture with origin, redirect, proxy, bearer, key, pin, lifecycle, and response races. |
  | REQ-006 | Same-game multi-digest/revision profile install, resolution, exact removal, capacity, concurrency, and restart corpus. |
  | REQ-007 | Production QML missing-mount install and every loading/offline/denied/incompatible/retry state plus live smoke. |
  | REQ-008 | Verifier and unchanged-SDK tests for valid cyclic Buttons and rejected grid/payload/prefix/target/capability/limit inputs. |
  | REQ-009 | Renderer/runtime entry/secondary schema and strict response tests with unknown/cross-release/lifecycle/tamper failures. |
  | REQ-010 | QML accepted-map navigation, Back/Entry, 16-entry cap, focus/accessibility, context reset, and zero gameplay-request counts. |
  | REQ-011 | Unit/PostgreSQL entry/secondary actions, wrong/cross screen, navigation injection, exact replay/collision, revision, and compiled/provider paths. |
  | REQ-012 | QML refresh/retry/completion/context-change preservation and one-shot entry recovery fixtures. |
  | REQ-013 | Clean-clone Door Legends old-pin install, cyclic navigation, real `enter`, terminal state, provider counts, and restart recovery. |
  | REQ-014 | Existing catalog/cache/server/QML suites plus legacy null pin/admission, no distribution, and Signal Siege fallbacks. |
  | REQ-015 | Deterministic SDK/package checks, authored/generated documentation review, focused tests, and final `bin/gate.sh --diff`. |

- Security, privacy, concurrency, reconnect, and rollback risks:
  - Old snapshot evidence can authenticate provenance but cannot authorize
    current use. Every server delivery uses current signed policy under the
    shared lifecycle lock; client rendering also requires the current session
    projection and server actions recheck policy.
  - Evidence storage can amplify database size. Normalization plus first-link
    retention bounds growth to snapshots that introduce at least one release,
    with the existing 1 MiB snapshot and release-count ceilings.
  - A server must not choose the client trust root. Both acquisition paths
    compare complete evidence to the key provisioned before network access.
  - Multiple same-game mounts can create ambiguous UI lookups. Every runtime
    operation uses digest+revision; catalog UI explicitly selects the current
    release tuple and never indexes by game alone for authority.
  - Reserved action parsing must be identical across verifier, renderer, QML
    envelope checks, and server denial. One shared Rust helper owns parsing;
    QML receives lowered exact destinations and does not derive targets from
    arbitrary raw action text.
  - Navigation cycles are intentional but history/memory growth is not. QML
    caps history at 16, the companion keeps existing plan/asset caps, and a new
    accepted plan replaces rather than accumulates scene nodes.
  - A stale secondary screen may no longer validate after a game action. REST
    truth wins; one entry fallback is explicit and bounded, with no client-side
    state synthesis.
  - The new screen field must not weaken idempotency replay after revocation.
    New admissions persist it before effects; legacy null rows may recover only
    an already admitted identical action/payload and cannot authorize new work.
  - Rollback is code/config rollback only. Forward schema/evidence stays
    additive; older binaries ignore evidence tables and nullable screen data,
    while no existing release/session row is rewritten.

- Material alternatives rejected:
  - Serving the current catalog selection for an old session was rejected
    because it silently changes reviewed presentation identity.
  - Storing one full signed snapshot per release or per session was rejected as
    quadratic/duplicative; normalized first authentic evidence is sufficient.
  - Trusting database columns without signed snapshot bytes was rejected
    because it turns marketplace review into a server-authored claim.
  - Mutating exported SDK v1 schemas/README or simply declaring SDK v2 was
    rejected in this slice: the current verifier supports one exact SDK lock,
    so either choice would make existing v1 release attestations unverifiable.
    The reserved capability convention is already valid v1 data and gives old
    hosts an explicit compatibility denial.
  - Making navigation a provider command was rejected because local screen
    selection has no game-state effect and would add latency, outage coupling,
    and an unnecessary authority path.
  - Letting QML parse cartridge presentation or cache files was rejected; only
    Rust consumes authenticated package data and QML receives a strict lowered
    plan/navigation envelope.

- Phase 2 is PASS. The design maps all fifteen EARS requirements to concrete
  database, Rust, QML, clean-clone, compatibility, and gate evidence; its
  CodeGraph receipt matches the unchanged gated worktree.

## Phase 3 — Implement

- Built:
  - Added forward migration `0022` with normalized immutable signed-snapshot
    evidence, first authentic release links, evidence-required future session
    pins, and screen-bound future action admissions while preserving nullable
    historical rows.
  - Added participant-only historical session acquisition on the server and
    companion, including discovery negotiation, pre/post session pin checks,
    independent marketplace-key verification, exact mount construction, and a
    dedicated explicit install endpoint.
  - Made cache profiles exact-tuple multi-mount inventories and exact removal
    receipts while retaining digest-addressed shared cartridge content.
  - Added the reserved `presentation.navigation.v1` contract, cyclic reviewed
    Button navigation, arbitrary signed-screen rendering, strict render
    envelope v2, and screen-specific server gameplay validation/admission.
  - Added QML missing-mount recovery, accepted-map-only navigation, bounded
    16-entry release/session history, Back/Entry controls, one-shot entry
    recovery, current-screen refresh, screen-bound gameplay, and strict legacy
    entry-only render-envelope fallback.
  - Published Door Legends cartridge v2 with Lobby/Chronicle navigation over
    the same provider-authenticated view and extended clean-clone provider
    coverage through historical acquisition and a real secondary-screen
    `enter` action.
  - Focused evidence is green: game-cartridge and companion Rust tests, all 47
    QML fixture tests, 56 PostgreSQL server tests plus operator suites, the
    deterministic SDK release (unchanged SDK v1 lock
    `7a732939918254ca1fb399f1fa4a4ef70d252ad683c13696dec8db8e2e88a045`),
    and the clean-clone remote-provider authority pilot.
- Deviations:
  - CodeGraph exposed the existing one-mount-per-game profile assumption, so
    REQ-006 and the implementation were expanded to exact same-game
    digest+revision coexistence before coding.
  - Historical evidence intentionally retains the first authentic snapshot
    that introduced a release. Its lifecycle claim may differ from the
    session's current lifecycle projection; the server authorizes current use,
    while the client separately verifies the retained marketplace provenance.
  - The old exact two-field companion response remains renderable as an
    entry-only compatibility surface and continues to use the legacy omitted
    screen action shape; every v2 response/action uses an explicit screen ID.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security / protocol namespace | The parser recognized only well-formed `navigate.<screen>` values, so the otherwise-valid reserved action `navigate.` could fall through as ordinary gameplay instead of failing closed. A malicious game gained no authority beyond its own provider command space, so this was a contract-integrity defect rather than a reportable vulnerability. | Medium | Fixed by rejecting every action beginning with the reserved prefix in both presentation validation and gameplay validation; the malformed-prefix regression passes. |
| 2 | Lifecycle / provenance | The first implementation compared the historical evidence policy lifecycle with the session's current lifecycle projection. A valid continuing retired/deprecated session can have current policy different from the first authentic retained snapshot. | Medium | Fixed by keeping current lifecycle as server/QML use authority while validating the retained acquisition policy independently as active/deprecated historical provenance. Companion, QML, PostgreSQL, and provider-pilot evidence passes. |
| 3 | Navigation ambiguity | Duplicate Button emitters for the same navigation action were accepted by the cartridge validator but collapsed by the renderer and rejected by QML's exact mapping contract. | Medium | Fixed by rejecting duplicate navigation emitters and adding a verifier regression. |
| 4 | Client budget alignment | QML initially capped navigation mappings below the rich renderer's 512-node budget, creating an avoidable valid-plan rejection. | Low | Fixed by aligning the strict QML navigation bound with the rich render-plan node ceiling. |
| 5 | QML runtime | The first fixture pass exposed a JavaScript scope mistake in the new helper response path and a transient null-plan warning caused by clearing the accepted plan before its acceptance flag. | Low | Fixed the helper scope and changed the state-clear order; all 47 production-root QML tests pass without the warning. |
| 6 | Clean-clone drift | The provider pilot still expected Door Legends cartridge v1 after the immutable multi-screen release became v2. | Low | Updated the clean-clone expectation and extended the pilot through catalog advancement, historical acquisition, navigation-action denial, secondary-screen `enter`, and recovery. |
| 7 | Security diff scan | Parent-only review accounted for all 19 generator-recognized source files plus 13 changed QML/Python/provider/signed-cartridge runtime inputs. Authorization, immutable evidence, independent client trust, exact mounts, navigation, gameplay admission, and lifecycle surfaces produced no reportable finding. TAC output was unavailable because its connector is not configured. | Informational | PASS. Sealed report: `/tmp/codex-security-scans/omarchy_bbs/4fecef5_working_tree_20260827T051925Z/report.md`; snapshot `codex-security-snapshot/v1:sha256:a15deadf309e4f0826cd702faff4ba0f9bfa2f6d00d671d924336fe4794d5198`. |
| 8 | Structural blast radius | Fresh CodeGraph inspection traced historical acquisition through `acquire_session_exact` → `acquire_session` → `from_session_verified`, exact mount/render resolution, and gameplay through `apply_cartridge_action` → `admit_session_action` → `validate_screen_action`. Direct QML/migration inspection covered unsupported formats. | Informational | PASS. The inspect receipt matches pipeline `e6c0e63b-200a-481d-8670-8531db96661f` and the final post-fix worktree. |

- Focused post-inspection regression evidence:
  - `cargo test -p omarchygs-game-cartridge`: 35 tests passed across unit,
    acquisition, conformance, marketplace, and SDK suites; malformed and
    duplicate navigation cases are covered.
  - `cargo test -p omarchygs-client-cartridge-runtime`: 10 tests passed,
    including historical session pin verification, exact mount behavior,
    independent marketplace trust, and loopback helper authorization.
- Phase 3.5 is PASS. All confirmed findings are fixed, no security candidate
  remains reportable or deferred, and fresh CodeGraph inspection evidence is
  recorded for the final source shape.

## Phase 4 — Validate

- Tests run:
  - The first canonical `bin/gate.sh --diff` attempt completed all 22 checks.
    Functional Rust, PostgreSQL, QML, package reproducibility, SDK, provider,
    backup/restore, and private-alpha admission checks passed.
  - The fresh canonical rerun again completed all 22 checks, including 56
    PostgreSQL server cases, 47 production-root QML cases, exact SDK lock
    verification, deterministic native-client packaging, remote-provider
    security conformance, the clean-clone Door Legends authority pilot, and
    operator recovery/admission drills.
- Gate run:
  - Attempt 1: RED because the initial formatting check requested two
    mechanical test-expression layouts and Clippy requested one collapsible
    guarded match arm in `crates/game-cartridge/src/validate.rs`. Both findings
    were corrected without behavior changes.
  - Attempt 2: `GATE GREEN [diff]`; the worktree-bound receipt at
    `.git/omarchy-gaming-system-gate-receipt` records
    `16044b9105dd5b104816fa8d7f245aee64fe06e2633756de79b660bd35daa683`,
    exactly matching the gated worktree after validation.
- Skips or pre-existing failures:
  - None. The package drill emitted the existing transient fakeroot diagnostic
    `payload not recognized` but produced identical package hashes and passed
    its reproducibility contract.
- Phase 4 is PASS. The complete delivery loop is green and receipt-bound;
  Phase 5 documentation/OpenWiki reconciliation may now proceed.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Completion evidence |
  |---|---|---|
  | REQ-001 | PASS | Migration `0022` append-only constraints plus first-seen, replay, later-policy, omission, tamper, and normalized-storage PostgreSQL cases prove bounded immutable release evidence. |
  | REQ-002 | PASS | Participant API tests authorize the exact persona/session pin across catalog advancement, rollback, and omission while rejecting foreign, null, mismatched, suspended, and revoked cases. |
  | REQ-003 | PASS | Exact-schema/privacy, key/snapshot/release/store tamper, secure-store, existing acquisition-verifier, and response self-verification tests prove one inert v1 historical envelope with no private authority leakage. |
  | REQ-004 | PASS | Shared-lock acquisition-first and writer-first cases plus signed-policy transition tests linearize delivery and prevent older retained evidence from bypassing current suspension/revocation. |
  | REQ-005 | PASS | Companion pre/acquire/post-session integration covers same-origin routing, redirects, proxy isolation, bearer handling, client key, exact pin, response tamper, races, atomic install, and restart. |
  | REQ-006 | PASS | Cache tests prove same-game digest/revision coexistence, exact tuple resolution/removal, legacy-profile compatibility, shared content retention, the 128-mount bound, concurrency, and restart. |
  | REQ-007 | PASS | Production-root QML fixtures and live helper smoke preserve REST session truth while exposing explicit keyboard install, loading, offline, denial, incompatibility, success, and retry states without automatic acquisition. |
  | REQ-008 | PASS | Verifier/conformance tests accept bounded cyclic Button navigation and reject missing targets, malformed/reserved prefixes, duplicate emitters, grids, payloads, capability omissions, and limit violations while preserving the SDK v1 lock. |
  | REQ-009 | PASS | Renderer/runtime tests compile entry and secondary screens for the same exact mount and reject unknown screens, invalid schemas, cross-release identity, lifecycle denial, and strict v2 envelope tampering. |
  | REQ-010 | PASS | QML fixtures prove accepted-map-only local navigation, Back/Entry, deterministic focus/accessibility, session/release-scoped 16-entry history, reset behavior, malicious-plan denial, and zero gameplay requests for navigation. |
  | REQ-011 | PASS | Unit and PostgreSQL tests prove valid secondary-screen compiled/provider actions, exact screen/action/payload admission, navigation denial, wrong/unknown/cross-screen denial, replay/collision, revision conflict, and lifecycle authorization. |
  | REQ-012 | PASS | QML/runtime refresh fixtures retain a valid current screen after REST truth changes and use one explicit entry recovery for invalid secondary state, while uncertain mutation recovery preserves exact identity. |
  | REQ-013 | PASS | The clean-clone Door Legends pilot advances the catalog, starts with no old mount, explicitly installs the historical pin, navigates Lobby/Chronicle locally, dispatches real `enter`, and recovers provider terminal state after restart. |
  | REQ-014 | PASS | Existing catalog acquisition/removal, legacy entry response/action, null presentation, metadata-only discovery, Signal Siege, provider, cache, server, and QML suites remain green. |
  | REQ-015 | PASS | Authored/generated docs, unchanged SDK v1 SHA-256, deterministic package checks, complete focused suites, and the canonical 22-stage worktree-bound diff gate agree on the shipped contract. |

- Docs: OpenWiki update run `3a7df2a9-9fda-4133-b752-ffc627dd7951`
  completed after reconciling quickstart, game-cartridges, and
  runtime-foundation. Its three warnings preserve pre-existing unresolved
  evidence debt on those pages; no Ticket 035 claim remained unresolved.
  `README.md`, API, client installation, system/cartridge architecture,
  owner-operator guidance, roadmap, and the Door Legends example now describe
  historical acquisition, exact mounts, host navigation, screen-bound action
  admission, compatibility, limits, and trust boundaries.
- AAR: submitted as effective with four failures, four prevention rules, and
  one architecture decision appended to the knowledge register.
- Archive: Ticket 035 moves to `tickets/closed/` and the sole active spec/notes
  pair moves to `pipeline/completed/`; no active pipeline remains.
- Phase 5 is PASS. All fifteen requirements are accounted for, durable docs
  and lessons are reconciled, and the pipeline is ready for delivery.

## Delivery evidence

- The post-archive canonical `bin/gate.sh --diff` completed all 22 stages and
  reported `GATE GREEN [diff]` with no skips or pre-existing failures.
- `.git/omarchy-gaming-system-gate-receipt` records
  `819f1b49d231c9d77ab2b7d7f4b55233ce4331d405193915babd7a616494a8ec`,
  matching the complete gated delivery worktree.
- The final pipeline structure check passes with only `.gitkeep` under the
  active pipeline and open-ticket directories.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Historical acquisition was rejected when a continuing session's current lifecycle differed from its first retained authentic snapshot. | Historical provenance and current use authority were treated as the same claim. | Validate retained evidence on its own authentic lifecycle and current session lifecycle on the server/QML authority path. | Keep provenance, selection, and current-use policy as separate typed/evidence boundaries. |
| 2 | A malformed action in the reserved navigation prefix could fall through to gameplay semantics. | `navigation_target` represented both `not navigation` and `invalid reserved navigation` as `None`. | Reject the complete reserved prefix before parsing a target and cover `navigate.` explicitly. | Reserved namespaces must fail closed before subtype parsing. |
| 3 | QML and renderer accepted different navigation cardinalities and duplicate semantics. | The envelope consumer budget and uniqueness rules were not derived from the producer's signed-plan limits. | Align the QML bound to 512 and reject duplicate emitters in the cartridge verifier. | Recount limits and uniqueness at every producer/consumer handoff. |
| 4 | Fixture/provider checks failed after behavior was correct. | Test scaffolding retained old variable scope and cartridge-version assumptions. | Corrected fixture state handling and updated the immutable Door Legends release expectation. | Treat clean-clone fixtures as protocol clients and update their exact schemas/versions in the same patch. |
