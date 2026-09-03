---
title: Usurper Level Nineteen Dungeon Band — notes
pipeline_id: bea038ec-1d39-483d-b23f-7f1b8b2a8625
---

# Usurper Level Nineteen Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 072 supplies rules/state/cartridge v23, levels one through
    eighteen, a 256-draw bounded trace with provider-size evidence, explicit
    asynchronous Loader-row geometry, and real activation of all twenty-one
    current controls;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-180 draw to remain visible in deterministic traces;
  - `PR-omarchy-gaming-system-size-rejection-traces-against-valid-tail-risk-001`
    requires Level 19's bounded-trace risk, long valid progression, and maximum
    serialized provider state to be proved rather than assumed;
  - `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001`,
    `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`,
    `PR-omarchy-gaming-system-count-instantiated-delegates-across-plan-replacement-001`,
    and `PR-omarchy-gaming-system-bind-loader-row-to-loaded-item-geometry-001`
    keep the nineteenth choice unique, non-overlapping, enabled, and inside the
    actual Qt input boundary;
  - `PR-omarchy-gaming-system-lock-provider-corpus-to-tested-phase-transitions-001`
    requires any Level 19 conformance sequence to assert its post-combat phase
    before selecting the next fixed command.
- Source preflight:
  - authenticated `EDMONST.PAS` at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`
    lines 4425–4525 defines Level 19 records 180–189 as Small Dront, Wonder of
    Evil, two distinct Renegade Orc records, Hobgoblin, Uruk-Hai, Sheriff,
    Ghostrider, Mountain Tiger, and Mamba, all at base strength 23 with source
    equipment flags;
  - authenticated `DUNGEONC.PAS` at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`
    lines 868–955 keeps events separate, spends a fight, and repeats
    `Random(level*10)` until the result exceeds `(level-1)*10`, so Level 19
    normally selects records 181–189 and stores record 180 only as source data;
  - the unregistered guard applies only above dungeon level 89, so Level 19
    remains on the ordinary branch;
  - authenticated `PLVSMON.PAS` at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`
    lines 68–138 uses `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initializes monster HP to strength times three.
- The informational rebuild bulletin and handoff were reviewed. Pipeline tools
  report CodeGraph 1.5.0 and OpenWiki 0.3.3 ready; the sole active spec/notes
  pair belongs to Ticket 073.
- Decision: implement Level 19 as the next normal dungeon band and defer Level
  20, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, RNG draws, monster data, and provider projections;
  - the provider validates a revision-bound fixed Level 19 action, maps it to
    the existing typed `EnterDungeon` command, and asks the pure reducer for
    the next v24 state and view;
  - the signed inert cartridge binds bounded `option_s` to one declared
    button. The platform authenticates its schema/action, lowers it into the
    existing `RenderedNode::Button`, and trusted QML dispatches only an
    unconfirmed request;
  - state flow remains `signed button -> local revision/screen check ->
    provider action mapping -> pure reducer -> v24 state/view -> authenticated
    render plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` has eight callers in the client cartridge runtime and
    preview CLI, and direct renderer integration coverage; `RenderedNode` has
    five renderer-local consumers with `rendering.rs` coverage;
  - server session-cartridge presentation/action consumers remain
    game-neutral: they transport exact versions, digests, authority, action,
    and JSON without depending on Usurper option letters or dungeon levels;
  - the Core profile capacity remains 256 nodes, so the twenty-two-control
    Level 19 dungeon plan is within the existing platform budget and requires
    no protocol or compiler change;
  - QML is outside the Rust graph and the separate Usurper repository has no
    `.codegraph/` index, so their current files/tests were reviewed directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `bea038ec-1d39-483d-b23f-7f1b8b2a8625` at gated state
    `84d45886eafd6815d64614fd84edf31a34ca93c171743585a4b2c39028e6aca5`.
- Exact implementation manifest, one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_s` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 180–189,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v24, extend validation/switching/labels through Level 19, and add exact
    encounter, long-rejection, retreat, deterministic, and hostile-state
    evidence;
  - external `crates/usurper-provider/src/lib.rs`: map fixed Level 19, prove
    generic/fixed equivalence and projection, lock a focused combat lifecycle,
    and retain maximum-state evidence with Level 19 trace values;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare
    one Level 19 action/button, and require bounded `option_s`;
  - all seventeen external `fixtures/presentation/*.json`: supply the required
    field, with non-empty Level 19 text only on the dungeon view and
    source-valid Level 19 facts on the combat fixture;
  - external `provenance/source-trace.json`: register reviewed Level 19 editor,
    selection, HP, and retreat branches;
  - external `scripts/test-cartridge.sh`, `scripts/test-provider.sh`,
    `scripts/test.sh`, and `scripts/play.sh`: assert exact v24/Level 19
    identities and uniqueness, lock the provider corpus to its focused
    post-combat phase, and end smoke play in a Level 19 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the implemented band, 256-draw Level 19
    tail evidence, and remaining visible limits;
  - platform `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`:
    ratchet the game-neutral largest-plan proof from 21 to 22 actual delegates
    while retaining positive-height, non-overlap, enablement, stale-removal,
    and one real Return activation per current control;
  - platform `docs/architecture/game-cartridges.md`: reconcile durable
    external-development facts through v24/Level 19 during Phase 5. No
    platform production QML, gameplay, renderer protocol/compiler, server,
    migration, Cargo, or provider-protocol change is required.
- Database and migration consequences: none. Provider-owned state stays in the
  external adapter; this slice adds no platform persistence, table, column,
  migration, or PostgreSQL write path.
- API and compatibility contract:
  - strict state JSON requires exact `schema_version: 24`; v23 and malformed
    v24 state fail before RNG construction or mutation;
  - the view schema adds required string `option_s` with the existing
    64-character bound. All screens provide it; only the dungeon screen binds
    it to `enter_dungeon_level_19`;
  - the signed manifest advances `rules_version` and `cartridge_version`
    together to 24; SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_19` accepts an empty payload and maps to the existing
    typed command. Levels 0, 20, and `u16::MAX` remain rejected without
    revision or RNG advance;
  - `MAX_TRACE_DRAWS` remains 256. Under uniform `Random(190)` values the
    theoretical probability of 256 consecutive rejected results is
    `4.025254300e-6`; deterministic state 420373 supplies a 254-draw valid run
    (253 rejections, then result 181), and the maximum-state regression must
    still prove the 32 KiB starter ceiling.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_NINETEEN_MONSTERS` arrays, source-trace validation, authenticated-source hashes/readback, compatibility and port-map review |
  | REQ-002 | v24 identity checks; old/missing/unknown JSON fields; Level 20, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | sequential levels 1–19 switching with unchanged RNG/empty traces, complete visible labels, ascent/descent/remain behavior, and rejected 0/20/max inputs |
  | REQ-004 | forced rejected `Random(190)` draw followed by 181–189, exact 23 strength/11 defence/69 HP, fight decrement, deterministic twin equality, 254-draw seed-420373 progression, quantified `4.025254300e-6` tail, and maximum serialized-state proof |
  | REQ-005 | exact failed-retreat `(2, 1), (190, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_s`, focused phase-aware live profile and restart corpus, signed-screen/action uniqueness, 21-to-22 Qt delegate replacement, local-play confirmation, and workspace-8 runtime audit |
- Risks and controls:
  - security/input: strict schemas, identifier checks, empty payloads,
    authenticated cartridge content, loopback capability, and revision/screen
    binding reject undeclared or stale actions;
  - privacy/secrets: no platform identity or reusable credential enters game
    state; local capabilities and signing keys remain temporary and unlogged;
  - state/concurrency: provider serialization and pre-RNG validation prevent
    rejected or stale commands from partially advancing state;
  - RNG/resource: the source rejection loop is theoretically unbounded while
    the provider trace is capped. The explicit tail calculation, 254-draw
    deterministic case, and maximum serialized-state test expose the remaining
    development limitation; deeper levels still require compact trace work as
    the valid tail narrows;
  - reconnect/restart: the fixed provider corpus runs twice across process
    restart, and a focused Level 19 test asserts actual phase/state before the
    next command;
  - rendering: the twenty-two-button dungeon plan is the new largest action
    surface. Recursive 21-to-22 instantiated-delegate counts, positive row
    geometry, signed-plan uniqueness, enabled-state propagation, and real
    Return input cover duplicate or inert controls;
  - lifecycle: the development launcher retains bounded render generations
    until process exit. It remains a same-developer, loopback/private,
    ephemeral boundary and must gain hard generation/count/byte eviction before
    lower-authority, shared, remote, multi-user, or long-lived reuse;
  - rollback: v24 artifacts can be removed before delivery without migration;
    no publication or delivery action is authorized.
- Decisions and rejected alternatives:
  - preserve record 180 as canonical data but never select it normally; direct
    selection would contradict the reviewed rejection loop;
  - add bounded `option_s`; overloading prior fields would blur phase semantics,
    while a grid or renderer change would unnecessarily expand platform
    behavior;
  - retain the 256-draw limit for Level 19 because the 254-draw valid fixture
    and maximum-state proof exercise the contract edge while the quantified
    failure probability remains roughly four per million. Widening without a
    compact encoding would consume the fixed state budget and only postpone
    the structural issue;
  - reuse the generic reducer; copying Level 19 rules into provider or QML
    would create a second authority;
  - keep events and registration paths excluded because neither is required
    for the ordinary Level 19 band.
- Phase 2 exit: the source, ownership, compatibility break, exact file
  manifest, regression evidence, and operational risks are fully specified.

## Phase 3 — Implement

- Implemented the Phase 2 manifest without widening platform production
  ownership or persistence:
  - external model/view now includes bounded `option_s`;
  - external data preserves exact editor records 180–189, including the two
    distinct source rows named Renegade Orc, and extends lookup routing;
  - external rules use exact v24 state, allow levels 1–19, preserve rejected
    record-180 draws, initialize strength-23 monsters at 69 HP, and retain the
    Level 19 retreat bound;
  - provider maps the fixed Level 19 action through the generic reducer,
    projects the nineteenth label, and proves its deterministic death/re-entry
    combat sequence;
  - signed manifest/presentation/schema, seventeen screen fixtures,
    provenance, scripts, and compatibility docs now describe v24/Level 19;
  - the platform QML regression now replaces twenty-one old controls with
    twenty-two current controls while retaining recursive cardinality,
    positive non-overlapping row geometry, dynamic enablement, stale-delegate
    removal, and one real Return activation per current control.
- Focused implementation checks:
  - exact Level 19 data test: PASS;
  - three Level 19 reducer tests, including seed 420373's 254-draw sequence:
    PASS;
  - fixed/generic provider equivalence and focused combat lifecycle: PASS;
  - strict v24 hostile-state regression and maximum 256-draw provider-state
    ceiling: PASS in the full external suite;
  - signed cartridge and trusted-QML suite: PASS, six cases;
  - provider-backed local HTTP/QML smoke: PASS;
  - complete TLS/replay/fault/callback provider corpus: PASS twice across
    restart against an isolated disposable PostgreSQL fixture;
  - JSON parsing, shell syntax, formatting, and whitespace checks: PASS.
- Full external `env -u CARGO_TARGET_DIR TMPDIR=/tmp
  CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh`: PASS, including strict Clippy/rustdoc, authenticated
  sources/provenance, 130 Rust tests (21 data, 37 provider, 3 local-play, 68
  rules, and 1 integration), six QML cases, seventeen signed screens, and
  provider-backed local play.
- Completeness checks found `option_s` in all seventeen fixtures and populated
  only on `dungeon.json`; the signed presentation contains exactly one Level
  19 action/button and no Level 20 action. No Cargo, platform production QML,
  gameplay, renderer protocol/compiler, provider protocol, migration,
  packaging, admission, or publication surface entered this slice.
- The first provider-conformance attempt selected the default compose fixture
  and stopped before game assertions because the host system PostgreSQL owns
  `127.0.0.1:5432`. Its created-but-never-started container, network, and empty
  volume were removed. The rerun used an explicit port-55432 PostgreSQL 18
  tmpfs container and private mode-0600 credential file; it passed and then
  the container, volatile database, listener, and credential file were
  removed. The unrelated system service was not changed.
- Deviations: none from the approved file manifest. The isolated database
  fixture reused Ticket 072's recorded collision-preflight rule.
- Phase 3 exit: implementation and focused evidence satisfy the designed
  manifest; the worktree is ready for correctness, architecture, security,
  test-quality, and scope inspection.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security | Codex Security scan `be59accb-8e32-4acd-b174-84bc48b24709` closed all seventeen frozen changed-source review items with zero plausible candidates and zero reportable findings. The supplied same-developer render-generation retention and archive/digest caller obligations remain rejected lifecycle debt, not new Level 19 impact. TAC advisory status could not be verified because its connector was unavailable. | informational | Accepted. Preserve the lower-authority/shared/remote/multi-user/long-lived reuse gate. Sealed report: `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260903T102810Z_4_65d494/report.md`; complete measured scan usage: 8,783,541 tokens. |
| 2 | Architecture and scope | Post-implementation CodeGraph inspection found no new platform gameplay owner or renderer-protocol/compiler dependency in the test-only QML ratchet; the external provider remains the sole owner of Level 19 state/rules. | informational | Accepted; retain the direct QML inspection because QML is not structurally covered by the Rust graph. |
| 3 | Live rendering | The user reported that most controls in the previously running v23 preview appeared twice and that its controls were inert. Its provider plan contained twenty-one unique button IDs, labels, and actions, ruling out duplicate game declarations. The report was not reproducible in the settled trusted surface: a fresh 21-to-22 replacement contained exactly twenty-two recursive delegates with positive, non-overlapping geometry, and each current control accepted one surface-level pointer click and one real Return event with exactly one requested action. | release-blocking | Resolved for the build. The old preview/provider tree was stopped and its private run root removed; one fresh v24 process with a unique authenticated plan is now the only Usurper window on workspace 8 while workspace 1 remains active. The desktop lock prevented an honest compositor screenshot, so user visual confirmation remains welcome and is not misreported as completed. |

- Phase 3.5 exit: PASS. Exact data/rules/provider behavior, architecture,
  security, test quality, scope, and the live rendering report are
  dispositioned. The fresh CodeGraph receipt matches gated state
  `e6a384dbdde2a902b31110a10eba16383f1d3401271aa3e2b33168be855eb710`;
  no unresolved inspection finding remains.

## Phase 4 — Validate

- Tests run:
  - focused exact-data, reducer, provider, signed-cartridge, local HTTP/QML,
    and trusted-QML checks: PASS;
  - full external `./scripts/test.sh`: PASS after the final surface-level
    pointer ratchet, including strict formatting/Clippy/rustdoc,
    authenticated sources/provenance, 130 Rust tests, six QML cases,
    seventeen signed screens, and provider-backed local play;
  - platform `bin/gate.sh --fast`: PASS after the final QML regression,
    including the twenty-two surface-level pointer and Return activations;
  - complete TLS/replay/fault/callback provider corpus: PASS twice across a
    restart against an isolated disposable PostgreSQL 18 fixture.
- Gate run:
  - the first isolated wrapper attempt passed stages 1–15b, then stage 16
    correctly refused `makepkg` because `--map-root-user` presented the
    namespace process as UID 0. The run was stopped after the harness-only
    cause was established; its exact disposable compose container, network,
    and volume were removed;
  - the corrected namespace mapped its setup identity separately, brought up
    loopback and the database relay with namespace capability, then ran the
    gate as mapped UID 1000. This preserved package safety without exposing
    the host PostgreSQL listener;
  - all twenty-four stages passed (`GATE GREEN [diff]`), including formatting,
    strict Clippy/rustdoc/tests, secret/hook/whitespace checks, QML and
    renderer matrices, deterministic package releases, every PostgreSQL/API/
    provider/recovery/admission stage, and both server-module proofs;
  - the matching pre-completion gate receipt is
    `e6a384dbdde2a902b31110a10eba16383f1d3401271aa3e2b33168be855eb710`.
    The disposable PostgreSQL 18 container, network, volume, socket relays,
    and port-55432 listener were removed; their test data is not recoverable.
    The unrelated system PostgreSQL remained active and unchanged.
  - after OpenWiki updated its gated lifecycle metadata, the same isolated
    gate passed all twenty-four stages again. The final receipt and current
    gated state match at
    `67d8958eeb72a2681038009693fd8c36a046118b329f30f8aaa27c6a61a949a1`.
- Skips or pre-existing failures:
  - none in completed tests;
  - the host system PostgreSQL owns loopback port 5432, so the green receipt
    gate used a disposable compose database through a temporary network
    namespace without stopping or modifying the unrelated service;
  - desktop lock state prevented a compositor screenshot of the refreshed
    workspace-8 window. Authenticated plan readback and real Qt input evidence
    pass; visual confirmation is not claimed.
- Phase 4 exit: PASS. The complete isolated diff gate and its receipt match
  the current gated worktree; there are no failed product stages or skips in
  the successful run.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS: authenticated source readback, exact records 180–189,
    provenance, fixed-data tests, compatibility notes, and the Rust port map
    agree;
  - REQ-002 PASS: exact v24 state/view/schema tests reject old, missing,
    malformed, out-of-band, and internally inconsistent state without
    mutation or RNG advance;
  - REQ-003 PASS: levels 1–19 switch draw-free with no retained monster, while
    0, 20, and maximum inputs reject unchanged;
  - REQ-004 PASS: exact `Random(190)` rejection traces, record-180 exclusion,
    records 181–189, 23 strength/11 defence/69 HP, seed 420373's 254-draw
    valid progression, quantified `4.025254300e-6` exhaustion risk, and the
    maximum serialized-state proof pass;
  - REQ-005 PASS: Level 19 retreat plus attack, potion, spell, class-special,
    poison, death, reward, replay, restart, and complete-day regressions pass;
  - REQ-006 PASS: fixed provider mapping, signed `option_s`, restart corpus,
    exactly one Level 19 action, no Level 20, non-overlapping 21-to-22 delegate
    replacement, and one surface pointer plus Return activation per current
    control pass. The old live preview was replaced by one fresh v24 window
    on workspace 8. A compositor screenshot was unavailable while the desktop
    was locked, so user visual confirmation is intentionally not claimed.
- Docs:
  - external README, compatibility guide, Rust port map, provenance, and test
    scripts describe v24/Level 19 and the remaining Level 20+ boundary;
  - hand-maintained cartridge architecture and generated OpenWiki quickstart/
    cartridge pages record Level 19 ownership, source facts, `option_s`, and
    the twenty-two-control regression;
  - OpenWiki update `ae49e4df-459c-44a6-a4c4-cbf0abd71820` completed, and its
    completion receipt matches the final gated state. Existing broad-page
    Claims/evidence warnings remain pre-existing documentation debt; no
    Ticket 073 material fact was omitted.
- AAR: submitted and dated 2026-09-03. No new `BF-*`, `PR-*`, or `AD-*` was
  minted because the live duplicate appearance did not reproduce to a proven
  root cause; its evidence and prevention practice remain explicit here and
  in the AAR.
- Archive: Ticket 073 is closed, removed from the open queue, and the sole
  spec/notes pair is moved to `docs/planning/pipeline/completed/`. No commit,
  push, deployment, admission, packaging publication, or other delivery
  action was authorized or performed.
- Phase 5 exit: PASS. All six EARS requirements have evidence, final validation
  and OpenWiki receipts match, the AAR is submitted, and no active pipeline
  pair remains.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The previously running live preview appeared to render most controls twice and did not accept input. | Duplicate provider declarations were disproved, and the settled trusted surface could not reproduce the report. The only isolated discrepancy was the prior preview/process lifecycle; a more specific root cause is not claimed without reproduction. | Replaced the entire preview/provider process tree with v24 and expanded the regression to hit-test every current control through the surface plus a real Return event, each with an exactly-once assertion. | Do not retain an older preview across cartridge builds; keep recursive delegate cardinality, stale-removal, geometry, enabled binding, surface pointer hit-testing, and keyboard exactly-once checks at the largest supported plan. |
