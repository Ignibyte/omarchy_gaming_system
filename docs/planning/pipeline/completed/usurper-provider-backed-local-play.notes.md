---
title: Usurper Provider-Backed Local Play — notes
pipeline_id: 91f08583-7519-448d-9c69-7e8790d469bf
---

# Usurper Provider-Backed Local Play — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 060 completed exact Usurper levels one through eight and proved all
    seventeen signed screens through the trusted QML surface.
  - `AAR-060` records the user-visible gap: the signed fixture preview looks
    playable even though it only logs unconfirmed requests.
  - `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001` requires
    every request to remain bound to the exact current signed node and payload.
  - `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001` keeps the
    QML acceptance boundary responsible for validating every replacement plan.
  - `PR-omarchy-gaming-system-resolve-cargo-artifacts-from-metadata-001` applies
    to every platform and provider executable launched by the new script.
  - The rebuilt local PostgreSQL service is healthy, but this slice does not
    need or claim durable provider persistence.
- Decisions:
  - Build the provider-backed local-play shell before Level 9 so visible testing
    exercises genuine state mutation rather than accumulating more inert views.
  - Keep the production separation between provider actions and signed
    `navigate.*` actions.
  - Use one in-memory development session, loopback-only HTTP, unguessable
    capabilities, exact current-plan action checks, expected revisions, and
    render-before-commit semantics.
  - Extend only the development preview compiler for explicit screen selection;
    do not change the render-plan format, Provider SDK/protocol, server routes,
    registration, admission, database, deployment, or publication surfaces.
  - Disable and visibly label the old fixture preview outside its automated
    input smoke.

## Phase 2 — Design

- Architecture and data flow:
  - `scripts/play.sh` remains an external developer entry point. It resolves
    both Cargo target directories from structured metadata, builds the exact
    platform cartridge/preview tools and Usurper local-play binary, signs one
    temporary development cartridge, launches the loopback driver, then opens
    the platform-owned local-play QML shell.
  - The driver constructs the real `UsurperGame`, launches one in-memory
    `GameState`, and serializes all requests through one session lock. No QML,
    Python, platform server, or local-play adapter owns a second copy of game
    rules.
  - For every initial or candidate state, the driver asks `ProviderGame::view`
    for bounded presentation data and invokes the exact platform preview CLI
    against the signed archive, selected signed screen, public key, and trusted
    preferences. It accepts no hand-authored live render plan.
  - The new preview CLI form selects an explicit authenticated screen while
    retaining the existing `prepare` contract unchanged. The underlying
    `compile_render_plan` and render-plan v1 types do not change.
  - The driver records only actions emitted by the current compiled plan.
    A current `navigate.*` action selects its signed target and recompiles the
    unchanged provider view; any other current action is wrapped through the
    existing provider adapter. Candidate provider state and revision commit
    only after view generation and signed-plan compilation succeed.
  - The platform-owned `CartridgeLocalPlay.qml` validates the loopback endpoint,
    capability, exact response envelope, revision, screen, asset capability,
    and every replacement plan through `TrustedCartridgeSurface.acceptPlan`.
    It disables actions while one request is pending and reports confirmed or
    rejected results in a visible development-only header.
  - Renderer-emitted assets remain in private generation directories and are
    exposed only through one random path capability plus validated digest/file
    tokens. State and action routes require a separate random header
    capability; neither capability is a platform credential.
- API and compatibility contract:
  - retain `omarchygs-cartridge-preview prepare <archive> <key> <profile>
    <view> <state> <preferences> <output>` byte-for-byte and add
    `prepare-screen <archive> <key> <profile> <screen> <view> <state>
    <preferences> <output>`;
  - the local driver prints one bounded startup document containing format,
    loopback endpoint, and the state/action capability;
  - `GET /v1/session` returns the current exact local-play response;
    `POST /v1/actions` accepts only an exact object containing
    `expected_revision`, `screen_id`, `action`, and an empty object `payload`;
  - successful responses contain format, provider revision, current screen,
    immutable asset generation, random asset capability, and render plan;
  - provider transitions advance revision once; signed navigation leaves the
    provider revision unchanged but advances the immutable render generation;
  - stale revisions/screens, actions absent from the current plan, nonempty or
    non-object payloads, oversized/malformed bodies, missing/wrong capabilities,
    invalid asset generations/tokens, and renderer/provider errors fail closed;
  - the harness provides no reconnect or durable resume contract: closing the
    launcher ends and discards the one development session.
- Database and migration consequences: none. The harness deliberately uses an
  in-memory `GameState`; the production starter's PostgreSQL persistence,
  replay receipts, callbacks, and TLS corpus remain covered by the existing
  provider test rather than being weakened or duplicated.
- Exact platform file manifest:
  - `crates/game-cartridge-renderer/src/bin/omarchygs-cartridge-preview.rs` —
    parse and execute the backward-compatible explicit-screen command;
  - `crates/game-cartridge-renderer/tests/rendering.rs` — prove old/default and
    explicit-screen CLI behavior plus invalid-screen rejection;
  - `client/qml/cartridge/CartridgePreview.qml` — label fixture mode and disable
    ordinary visible controls while preserving automated renderer input smoke;
  - `client/qml/tests/CartridgeLocalPlay.qml` — new non-packaged,
    capability-bound live loop and trusted replacement-plan validation;
  - `client/qml/cartridge/TrustedCartridgeSurface.qml` — keep dynamically
    created controls bound to surface action authority across loading/busy
    transitions;
  - `docs/architecture/game-cartridges.md` and affected `openwiki/` pages —
    reconcile the development preview/local-play distinction during Phase 5.
- Exact external Usurper file manifest:
  - `Cargo.toml`, `crates/usurper-provider/Cargo.toml`, `Cargo.lock` — declare
    the already-locked Axum HTTP dependency required by the local binary;
  - `crates/usurper-provider/src/bin/usurper-local-play.rs` — loopback service,
    in-memory provider session, render transaction, action/revision checks,
    private assets, and focused unit tests;
  - `scripts/play.sh` — metadata-resolved signed visible launcher;
  - `scripts/test-local-play.sh` — live HTTP and offscreen trusted-QML smoke;
  - `scripts/test.sh` — include local-play proof in the complete external suite;
  - `scripts/show.sh` — identify the retained fixture viewer as non-interactive;
  - `README.md` — distinguish fixture rendering from provider-backed play.
- CodeGraph evidence:
  - `compile_render_plan` has eight callers across the preview CLI and mounted
    client runtime, with renderer integration coverage; the design adds a CLI
    selector without changing its shared typed contract.
  - `write_prepared_preview` owns private-empty-directory, read-only plan/asset,
    sync, and asset-token checks, so the driver uses one new output directory
    per generation rather than overwriting render output.
  - production `translate_command` binds registered-provider actions by merging
    the action with the exact payload, while `GameController` separates
    `navigate.*` from provider mutation. The local shell preserves that split
    and further restricts current Usurper payloads to their signed empty shape.
  - QML is not indexed, so `CartridgePreview.qml`, `TrustedCartridgeSurface.qml`,
    its trusted nodes, and `GameController.qml` were inspected directly.
  - worktree-bound design receipt:
    `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
    `91f08583-7519-448d-9c69-7e8790d469bf`, state hash
    `f6c27243492f58e2fcd7712e9af59398b67d2be08636d48a110874b2cb7a5fcd`.
- Regression plan:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Signed launcher smoke; startup envelope assertions; QML confirms Entry `continue`, then the signed navigation path exposes the provider's race-selection view; visible workspace-8 play. |
  | REQ-002 | Unit tests prove current declared command, next provider state, one revision advance, updated signed screen, and no commit when renderer execution fails. |
  | REQ-003 | Unit/integration tests prove navigation without provider revision, stale revision/screen rejection, undeclared action rejection, empty-payload enforcement, and unchanged state after every rejection. |
  | REQ-004 | Bind-address assertion; missing/wrong capability HTTP cases; body limit; exact response validation; invalid generation/token asset cases; private-directory and path review. |
  | REQ-005 | Existing offscreen fixture input smoke remains enabled only under `--smoke-test`; ordinary QML title/header and disabled-control assertions plus visible review. |
  | REQ-006 | Renderer tests, QML style/tests, provider crate/full suite, live provider corpus, platform fast/diff gate, scope/security review, and no database/protocol diff. |
- Risks and rollback:
  - Security: a web page can reach loopback, so state/action routes require a
    high-entropy header capability, JSON content type, exact bounded schemas,
    and current-plan/revision binding. Asset requests cannot attach the header,
    so they use a distinct random URL capability and strict renderer token.
  - Concurrency: QML permits one request at a time; the driver serializes all
    state access and verifies expected revision plus current screen. Duplicate
    or delayed requests cannot apply twice.
  - Failure atomicity: provider transitions are cloned candidates and commit
    only after successful view and render; navigation selection follows the
    same compile-before-publish rule.
  - Privacy: no account/persona ID, platform bearer, database URL, TLS key, or
    production provider credential enters the harness. Capabilities are
    ephemeral and emitted only to the private launcher log/child arguments.
  - Reconnect: there is intentionally no reconnect or persistence. A crashed
    or closed launcher discards state and a new run creates new capabilities.
  - Rollback is removal of the new QML/binary/scripts and explicit-screen CLI
    branch; the old preview command, renderer library API, provider adapter,
    rules/state v13, and production client/server remain unchanged.
- Alternatives rejected:
  - registering/admitting Usurper into the complete platform stack would turn a
    developer UX gap into a production trust/deployment decision outside the
    authorized scope;
  - polling mutable render-plan files would create stale/partial-read races and
    would not carry revision, confirmation, navigation, or asset-generation
    semantics;
  - reproducing the reducer or presentation mapping in QML/Python would violate
    provider and trusted-renderer ownership;
  - repacking the cartridge with a different entry screen after every action
    would change the signed archive identity inside one local session.

## Phase 3 — Implement

- Built:
  - Added backward-compatible `prepare-screen` handling to the platform preview
    CLI. It passes an explicit authenticated screen into the existing renderer
    while the original eight-argument `prepare` path remains unchanged.
  - Added renderer integration coverage for default, explicit valid, and
    explicit unknown screen behavior.
  - Changed the fixture preview title to `Signed Fixture Preview —
    Non-interactive`, disabled ordinary input outside smoke mode, and changed
    the external launcher message to match.
  - Added `CartridgeLocalPlay.qml`, which admits only a strict loopback endpoint
    and 64-hex capability, sends exact revision/screen/action/empty-payload
    requests, validates the exact local response envelope, runs every returned
    plan through the trusted surface, disables requests in flight, and shows
    explicit provider/navigation confirmation in development-only chrome.
  - Added `usurper-local-play`, an Axum loopback binary that launches the real
    `UsurperGame`, keeps one in-memory session, requires a state/action header
    capability, serves assets behind a distinct URL capability, admits only
    current rendered actions, checks expected revision and screen, distinguishes
    signed navigation from provider commands, and publishes cloned candidate
    state only after signed rendering succeeds.
  - The driver retains the selected signed screen across provider mutation,
    matching the production client: provider action confirmation updates the
    view on that screen, while a separate current `navigate.*` action changes
    presentation without advancing provider revision.
  - Added immutable private render generations, bounded same-inode plan/asset
    reads, strict digest asset tokens, exact config paths, random capabilities,
    graceful shutdown, and generic external error bodies.
  - Added unit coverage for identifier/asset/action admission, exact request
    decoding, and failed-render atomicity; added `play.sh` HTTP checks for auth,
    bounds, current action, stale revision/screen, provider revision, and signed
    navigation followed by a fresh offscreen trusted-QML click/confirmation.
  - Added the local smoke to the full Usurper test suite and documented the
    fixture-versus-live distinction in the external README.
- Focused evidence:
  - `cargo test -p usurper-provider --bin usurper-local-play`: 3 passed,
    including failed-render rollback;
  - `cargo clippy -p usurper-provider --all-targets --all-features -- -D
    warnings`: passed;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    scripts/test-local-play.sh`: loopback HTTP and trusted-QML smoke passed;
  - platform renderer warning-denying Clippy and all 11 renderer tests (2 unit,
    9 integration): passed;
  - `python3 scripts/check-qml-style.py`: 34 QML files passed;
  - `scripts/test-qml-onboarding.sh`: 55 passed;
  - `shellcheck`, `bash -n`, and `git diff --check` on the affected external
    scripts/worktree: passed.
- Deviations:
  - The exact platform manifest expanded to modify
    `client/qml/cartridge/TrustedCartridgeSurface.qml`. The first local QML
    smoke loaded its plan but timed out because node controls copied the
    surface's disabled loading state once at construction and never followed
    later enablement. Replacing that copy with a `Qt.binding` is required for
    local play and also restores the intended existing production busy/loading
    behavior. The rerun and complete QML suite passed.
  - The first full platform gate rejected the new local-play QML file because
    it lived under the exact production cartridge inventory without being in
    the signed package manifest. The file is development-only, so it moved to
    `client/qml/tests/`; the corrected gate proved the production inventory
    remains exactly forty files while the separate local-play smoke still
    instantiates the harness and performs a real click.
  - The external manifest includes direct Axum dependency metadata in
    `Cargo.lock`; Axum was already locked transitively through the public
    provider starter, so no new third-party package entered the graph.
  - No server route, database, migration, Provider SDK/protocol, Usurper
    rules/state/cartridge identity, admission, registration, deployment, or
    publication surface changed.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Provider authority and atomicity | The local service invokes the exact `UsurperGame`, serializes one session, admits only current plan actions, and commits a cloned provider candidate only after view generation and trusted rendering succeed. The generation counter may skip on a failed render but provider state/revision/current assets do not advance. | none | PASS — direct trace and failed-render unit proof agree. |
| 2 | Action/navigation binding | Requests bind exact expected provider revision, selected signed screen, action present in the current compiled plan, and an empty object payload. `navigate.*` recompiles unchanged provider state and does not advance revision; other actions pass through the strict provider decoder. | none | PASS — hostile HTTP cases and provider/navigation sequence passed. |
| 3 | Loopback and capability boundary | The listener binds only `127.0.0.1:0`; state/action requests require a distinct 64-hex random header capability, Axum rejects oversized bodies, and no CORS permission is installed. The capability is given only to the child QML process in this single-user development workflow. | none | PASS — missing/wrong capability and 40 KiB request cases passed; same-user compromise remains an explicit non-isolation assumption. |
| 4 | Asset and filesystem boundary | Asset lookup requires a separate random URL capability, a known immutable generation, and an exact lowercase digest plus `.png`/`.wav` token. Reads reject symlink/non-regular/changed-inode/oversized files; render inputs are absolute regular paths and renderer execution uses direct arguments rather than a shell. | none | PASS — invalid generation/token paths and static source/control/sink trace close traversal and command-injection candidates. |
| 5 | Trusted rendering and QML | Every live response is compiled from the signed archive by the platform preview binary and revalidated by `TrustedCartridgeSurface`. Node `actionsEnabled` now follows the parent dynamically across loading/busy transitions. The fixture viewer is explicitly inert. | none | PASS — renderer, local-play, QML policy, and 55-test client suite passed. |
| 6 | Compatibility and blast radius | The old preview CLI form remains unchanged; `prepare-screen` only supplies the existing optional screen selector to `compile_render_plan`. CodeGraph reports the shared renderer's eight callers and existing integration tests; no server, runtime, protocol, database, or admission consumer changed. | none | PASS — focused old/new/unknown CLI tests and worktree-bound graph inspection passed. |
| 7 | Complete external security diff | The immutable working-tree snapshot covered all 15 changed source-like files, including prior Level 7/8 rules/data/presentation and all local-play code/scripts. | none | PASS — sealed scan `0dcf8d84-a7bf-48b2-8d43-c9de6a0be69f`, complete coverage, zero findings. TAC status was unavailable because its connector was not signed in; delegated review was unavailable by session policy, so the parent reviewed every item sequentially. |
| 8 | Platform CodeGraph inspection | Fresh worktree inspection traced renderer preparation/output, `ProviderGame`, production cartridge action translation, and caller/test blast radius. QML remains unsupported by the graph and was inspected directly. | informational | PASS — inspect receipt matches pipeline `91f08583-7519-448d-9c69-7e8790d469bf` and gated state hash `057467ca94500f4601d531e548bf96e7af5804abd4c28218246e71dc919397d5`. |

- Security report:
  `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260902T201439Z_4ud3ov6m/report.md`.
- Security scan measured 3,604,126 total tokens across one parent thread, with
  complete recorded coverage and no deferred candidates.
- Phase 3.5 exit: every changed security-relevant source path and platform
  consumer boundary is dispositioned; no reportable or deferred issue remains.

## Phase 4 — Validate

- Tests run:
  - `cargo test -p usurper-provider --bin usurper-local-play`: 3 passed,
    including render-failure atomicity;
  - strict provider Clippy plus the complete external `scripts/test.sh`: passed
    all 74 Rust tests, rustdoc, twelve immutable source hashes,
    provenance/privacy checks, seventeen signed screens, and the local HTTP/QML
    smoke;
  - the fixed fifteen-case live TLS, authentication, replay, fault, callback,
    and reconciliation corpus passed twice across provider restart;
  - platform renderer Clippy and 11 tests, exact QML package-source inventory,
    55 QML onboarding tests, shell checks, and local trusted-QML click smoke all
    passed. The smoke emitted `OGS_LOCAL_PLAY_ACTION confirmed=true revision=1
    screen=entry` after clicking the first real provider action.
  - The final visible launch caught a stale no-override platform sibling name in
    `scripts/play.sh`; after correcting it to `omarchy_gaming_system`, `bash -n`,
    ShellCheck, and the real-click local-play smoke passed again without an
    override.
  - sealed security diff scan
    `0dcf8d84-a7bf-48b2-8d43-c9de6a0be69f` covered the complete external
    source-like diff and reported zero findings.
- Gate run:
  - The first `bin/gate.sh --diff` passed all substantive behavior but correctly
    failed package stages 15 and 16 because the development-only local-play QML
    had been placed in the production inventory.
  - After moving that harness to `client/qml/tests/`, the complete corrected
    `bin/gate.sh --diff` passed stages 1–24 and printed `GATE GREEN [diff]`.
    Receipt `.git/omarchy-gaming-system-gate-receipt` matches gated state
    `1aad50e803b3b80328e8b5db57dc0db31bd3a7820285e313412e6c7f9cbca29d`.
- Skips or pre-existing failures:
  - No required validation was skipped. The server-module suite's one ordinary
    ignored containment case ran and passed through its dedicated stage-24
    systemd/Bubblewrap harness. TAC inspection status was unavailable because
    its connector was not signed in; this did not reduce scan coverage or any
    validation command.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Evidence |
  |---|---|---|
  | REQ-001 | SATISFIED | The launcher starts one real in-memory `UsurperGame`, compiles the signed Entry screen with the platform renderer, and instantiates the development-labeled trusted QML shell; automated and visible workspace-8 runs passed. |
  | REQ-002 | SATISFIED | Current provider actions run through `ProviderGame`; the failed-render test proves candidate state does not commit, while the trusted-QML click confirmed revision 1. |
  | REQ-003 | SATISFIED | Signed navigation changes screen without advancing provider revision; stale revision/screen, undeclared action, malformed and nonempty payload cases reject with unchanged state. |
  | REQ-004 | SATISFIED | Loopback-only binding, separate high-entropy request and asset capabilities, exact bounded schemas, immutable generations, strict tokens, and stable-descriptor reads passed hostile tests and security trace. |
  | REQ-005 | SATISFIED | The fixture title explicitly says non-interactive and its controls are disabled outside the isolated automated input smoke. |
  | REQ-006 | SATISFIED | All 74 external Rust tests, live provider corpus twice, renderer/QML suites, exact production package inventory, zero-finding security scan, and corrected full platform diff gate passed without database/protocol/admission/deployment/publication changes. |
- Docs:
  - Updated `docs/architecture/game-cartridges.md` with the explicit-screen CLI,
    inert fixture, non-packaged live harness, dynamic action binding, capability
    boundary, render-before-commit semantics, and non-admission boundary.
  - OpenWiki reconciled `openwiki/game-cartridges.md` and
    `openwiki/quickstart.md`; final update run
    `2f8d4a22-7289-497f-9351-1632bf4021b0` completed after Phase 4 and issued a
    matching pipeline completion receipt. Its warnings were pre-existing
    unresolved-claim debt on large pages, not an incomplete lifecycle.
- AAR:
  - Submitted `AAR-061` as effective with two new failure IDs, two prevention
    rules, one architecture decision, and all five IDs registered in the local
    knowledge index.
- Archive:
  - Closed Ticket 061, removed it from the open queue, set the exact completed
    status, and archived this spec/notes pair under `pipeline/completed/`.
  - OpenWiki's gated state is
    `b5d7e662163688ad86aed83ab36d5d6038a6568126b41904d111a6b22a8da041`;
    the full gate was rerun after completion because the lifecycle updated gated
    OpenWiki metadata, and the matching receipt again proves `GATE GREEN
    [diff]`.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first local QML smoke loaded but exited on its ten-second timeout without emitting a request. | Trusted node controls copied `actionsEnabled=false` while the plan loaded and never followed the parent surface after loading completed. | Bind each node's property dynamically to `TrustedCartridgeSurface.actionsEnabled`; rerun local and complete QML suites. | Test every interactive trusted surface across a disabled-loading-to-enabled transition, not only with controls enabled at construction. |
| 2 | The first full gate rejected native packaging despite all behavior checks passing. | The development-only local-play QML was placed under the exact production cartridge source inventory. | Move the harness to `client/qml/tests/` and retain imports of the production trusted surface. | Keep development QML outside production inventory and prove both exact package contents and the separate live harness. |
| 3 | The final no-override visible launcher initially failed before opening QML. | The launcher's default platform sibling still used the retired `omarchy_bbs` name; automation had supplied the current root explicitly. | Default to the current `omarchy_gaming_system` sibling and rerun syntax, ShellCheck, real-click smoke, and visible launch. | Include the ordinary no-override developer command in final visible validation. |
