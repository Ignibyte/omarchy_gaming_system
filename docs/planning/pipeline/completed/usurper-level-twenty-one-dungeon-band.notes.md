---
title: Usurper Level Twenty-One Dungeon Band — notes
pipeline_id: 6f6945fc-f320-4e6d-931b-15a042c659eb
---

# Usurper Level Twenty-One Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 074 supplies rules/state/cartridge v25, levels one through twenty,
    a 256-draw bounded trace with provider-size evidence, private local-play
    startup handoff, and real activation of all twenty-three current controls;
  - the legacy-branch, rejected-RNG, trace-tail, provider-corpus, private-file,
    real-compositor-output, delegate lifecycle, row-geometry, and enabled-state
    rules remain binding;
  - the live v25 workspace-8 frame shows one visible delegate per current
    dungeon label, and a real Return press advanced provider revision 6 to a
    Level 1 Beggar encounter at revision 7. The user's duplicate/inert report
    remains a release-blocking twenty-four-control regression boundary.
- Source preflight:
  - authenticated `EDMONST.PAS` at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`
    lines 4629–4729 defines Level 21 records 200–209 as Barbarian, Master
    Barbarian, Renegade Elf, Dwarf Lord, Dwarf King, Bard, Thief, Gnoll,
    Villain, and Red Dwarf, all at base strength 25 and all with armor and
    weapon usage enabled;
  - authenticated `DUNGEONC.PAS` at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`
    lines 869–955 keeps events separate, spends a fight, and repeats
    `Random(level*10)` until the result exceeds `(level-1)*10`, so Level 21
    normally selects records 201–209 and stores record 200 only as source data;
  - the unregistered guard applies only above dungeon level 89, so Level 21
    remains on the ordinary branch;
  - authenticated `PLVSMON.PAS` at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`
    lines 68–138 uses `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initializes monster HP to strength times three.
- Bounded-trace preflight:
  - under uniform `Random(210)`, 256 consecutive rejected results have
    probability `(201/210)^256 = 1.34912207275037e-5`;
  - deterministic development RNG state 553101 reaches accepted result 208 on
    draw 256 after 255 rejected values, giving an exact at-cap progress fixture;
  - the serialized-state ceiling remains an implementation-time proof, not an
    assumption.
- Decision: implement Level 21 as the next normal dungeon band and defer Level
  22, dungeon events, shared realm, and unrelated combat breadth.
- Phase 1 exit: source identity, game boundary, acceptance criteria, operational
  evidence, and explicit exclusions are concrete; Phase 1 passes.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, RNG draws, monster data, and provider projections;
  - the provider validates a revision-bound fixed Level 21 action, maps it to
    the existing typed `EnterDungeon` command, and asks the pure reducer for
    the next v26 state and view;
  - the signed inert cartridge binds bounded `option_u` to one declared
    button. The platform authenticates its schema/action, lowers it into the
    existing `RenderedNode::Button`, and trusted QML dispatches only an
    unconfirmed request;
  - state flow remains `signed button -> local revision/screen check ->
    provider action mapping -> pure reducer -> v26 state/view -> authenticated
    render plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` validates the authenticated view against its signed
    schema, collects declared action identifiers, lowers every signed screen
    node exactly once through `lower_node`, charges the selected profile, and
    appends one `RenderedNode` per accepted node;
  - `RenderedNode` retains five renderer-local consumers plus renderer
    integration coverage; no platform session, server, or database consumer
    knows Usurper option letters or dungeon levels;
  - the Core profile permits 256 nodes, so the twenty-four-control Level 21
    dungeon plan remains inside the existing platform budget;
  - QML is outside the Rust graph and the separate Usurper repository has no
    `.codegraph/` index, so those surfaces were reviewed directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `6f6945fc-f320-4e6d-931b-15a042c659eb` at gated state
    `8ad814ee522412a1d5ba0059de126cf3617361c8f2cb4fd89c986a762ddcabab`.
  - After the workspace-8 audit reproduced stale post-transition AT-SPI
    geometry and duplicate press actions, the design was reopened. A fresh
    CodeGraph exploration reconfirmed that signed plan production remains in
    `compile_render_plan`, with its client-runtime/preview consumers unchanged;
    QML materialization is still outside the Rust graph and was inspected
    directly. The design receipt remains bound to this pipeline and gated
    state because pipeline notes/specification files do not alter the gated
    application state.
- Exact implementation manifest, one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_u` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 200–209,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v26, extend validation/switching/labels through Level 21, and add exact
    encounter, at-cap rejection, retreat, deterministic, and hostile-state
    evidence;
  - external `crates/usurper-provider/src/lib.rs`: map fixed Level 21, prove
    generic/fixed equivalence and projection, lock a focused combat lifecycle,
    and retain maximum-state evidence with Level 21 trace values;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare
    one Level 21 action/button, and require bounded `option_u`;
  - all seventeen external `fixtures/presentation/*.json`: supply the required
    field, with non-empty Level 21 text only on the dungeon view and
    source-valid Level 21 facts on the combat fixture;
  - external `provenance/source-trace.json`: register reviewed Level 21 editor,
    selection, HP, and retreat branches;
  - external `scripts/test-cartridge.sh`, `scripts/test-provider.sh`,
    `scripts/test.sh`, and `scripts/play.sh`: assert exact v26/Level 21
    identities and uniqueness, lock the provider corpus to its focused
    post-combat phase, and end smoke play in a Level 21 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the implemented band, the 256-draw Level
    21 tail evidence, and remaining visible limits;
  - platform `client/qml/tests/CartridgeLocalPlay.qml` and
    `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`: ratchet
    the game-neutral largest-plan proof from 23 to 24 actual delegates while
    retaining positive-height, non-overlap, enablement, stale-removal, and one
    real pointer plus Return activation per current control; keep the exact
    provider-backed entry, race, class, street, Level 1, Level 21, Look, and
    combat-action lifecycle;
  - platform `client/qml/cartridge/TrustedCartridgeSurface.qml`: invalidate the
    prior delegate model immediately and defer current-plan materialization by
    one guarded event-loop turn so Qt accessibility unregisters replaced
    objects before the next set is exposed;
  - platform `client/qml/cartridge/nodes/TrustedButtonNode.qml`: retain the
    existing visual contract on a native Qt Quick Controls `Button`, remove
    the redundant manual accessibility press action and overlay mouse area,
    and keep pointer/Return/Enter/Space dispatch exactly once;
  - platform `docs/architecture/game-cartridges.md`: reconcile durable
    external-development facts through v26/Level 21 during Phase 5. No
    platform production QML, gameplay, renderer protocol/compiler, server,
    migration, Cargo, or provider-protocol change is required.
- Database and migration consequences: none. Provider-owned state stays in the
  external adapter; this slice adds no platform persistence, table, column,
  migration, or PostgreSQL write path.
- API and compatibility contract:
  - strict state JSON requires exact `schema_version: 26`; v25 and malformed
    v26 state fail before RNG construction or mutation;
  - the view schema adds required string `option_u` with the existing
    64-character bound. All screens provide it; only the dungeon screen binds
    it to `enter_dungeon_level_21`;
  - signed `rules_version` and `cartridge_version` advance together to 26;
    SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_21` accepts an empty payload and maps to the existing
    typed command. Levels 0, 22, and `u16::MAX` remain rejected without
    revision or RNG advance;
  - `MAX_TRACE_DRAWS` remains 256. Under uniform `Random(210)` values the
    theoretical probability of 256 consecutive rejected results is
    `1.34912207275037e-5`; deterministic state 553101 supplies a valid
    256-draw run (255 rejections, then result 208), and the maximum-state
    regression must still prove the 32 KiB starter ceiling.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_TWENTY_ONE_MONSTERS` arrays, source-trace validation, authenticated-source hashes/readback, compatibility and port-map review |
  | REQ-002 | v26 identity checks; old/missing/unknown JSON fields; Level 22, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | sequential levels 1–21 switching with unchanged RNG/empty traces, complete visible labels, ascent/descent/remain behavior, and rejected 0/22/max inputs |
  | REQ-004 | forced rejected `Random(210)` draw followed by 201–209, exact 25 strength/12 defence/75 HP, fight decrement, deterministic twin equality, 256-draw seed-553101 progression, quantified tail, and maximum serialized-state proof |
  | REQ-005 | exact failed-retreat `(2, 1), (210, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_u`, focused phase-aware live profile and restart corpus, signed-screen/action uniqueness, 23-to-24 Qt delegate replacement, seven-action provider-backed local-play confirmation, and workspace-8 runtime audit |
- Risks and controls:
  - strict schemas, identifier checks, empty payloads, authenticated cartridge
    content, loopback capability, and revision/screen binding reject undeclared
    or stale actions without accepting platform identity or reusable secrets;
  - the private mode-0600 startup/config handoff keeps local authority out of
    process metadata, and its existing cross-UID regression remains binding;
  - provider serialization and pre-RNG validation prevent rejected or stale
    commands from partially advancing state;
  - the source rejection loop is theoretically unbounded while the provider
    trace is capped. The explicit tail calculation, exact 256-draw valid case,
    and maximum serialized-state test expose the remaining development limit;
  - the focused provider corpus runs twice across process restart and asserts
    its actual post-combat phase before the next fixed command;
  - the twenty-four-button dungeon plan is the new largest action surface.
    Recursive 23-to-24 delegate counts, an observable guarded retirement turn,
    positive row geometry, stale removal, enabled-state propagation, native
    button hit-testing/accessibility, and Return exactly-once activation cover
    duplicate or inert controls;
  - visual/pointer evidence is accepted only from a post-unlock process on a
    real compositor output; placeholder-output sessions are retired;
  - v26 artifacts can be removed before delivery without migration; no
    publication or delivery action is authorized.
- Decisions and rejected alternatives:
  - preserve record 200 as canonical data but never select it normally; direct
    selection would contradict the reviewed rejection loop;
  - add bounded `option_u`; overloading prior fields or changing the renderer
    would widen semantics unnecessarily;
  - retain 256 draws because Level 21 reaches a valid result exactly at the cap
    and the maximum-state proof exercises its envelope. Widening the trace
    would consume the fixed state budget without solving compact history;
  - reuse the generic reducer; provider or QML copies would create a second
    rules authority;
  - preserve the trusted surface's one-loader ownership while adding a guarded
    empty-model turn and native button semantics; merely accepting clean
    screenshots would leave the reproduced post-transition AT-SPI bounds and
    duplicate press action unresolved;
  - keep events and registration paths excluded because neither is required
    for the ordinary Level 21 band.
- Phase 2 exit: the source, ownership, compatibility break, exact file
  manifest, regression evidence, and operational risks are fully specified;
  Phase 2 passes and implementation may begin.

## Phase 3 — Implement

- Built the Phase 2 manifest without widening platform production ownership or
  persistence:
  - external model/view now includes bounded `option_u`;
  - external data preserves exact editor records 200–209 and their all-true
    equipment flags, including the normally unreachable Barbarian boundary
    row;
  - external rules use exact v26 state, allow levels 1–21, preserve rejected
    record-200 draws, initialize strength-25 monsters at 75 HP, and retain the
    Level 21 retreat bound;
  - provider maps the fixed Level 21 action through the generic reducer,
    projects the twenty-first label, and proves its deterministic combat
    sequence;
  - signed manifest/presentation/schema, seventeen screen fixtures,
    provenance, scripts, and compatibility docs now describe v26/Level 21;
  - the platform QML regression now replaces twenty-three old controls with
    twenty-four current controls while retaining recursive cardinality,
    positive non-overlapping row geometry, dynamic enablement, stale-delegate
    removal, and one real pointer plus Return activation per current control;
  - the provider-backed smoke retains seven consecutive actions across six
    replaced plans and now selects the unique Level 21 control.
- Focused implementation checks:
  - exact Level 21 data, three Level 21 reducer tests, strict v26 hostile state,
    draw-free level switching, fixed/generic provider equivalence, focused live
    profile, and maximum 256-draw provider-state ceiling: PASS;
  - deterministic RNG state 553101 produced 255 rejected `Random(210)` draws
    and accepted result 208 on draw 256 exactly;
  - signed cartridge and trusted-QML suite: PASS, six cases and seventeen
    signed screens; the 23-to-24 plan replacement emitted one pointer and one
    Return action per current control with no missing, duplicate, zero-height,
    or overlapping delegate;
  - provider-backed local HTTP/QML seven-action smoke: PASS;
  - JSON parsing, shell syntax, formatting, and whitespace checks: PASS.
- Focused-test correction:
  - the first new Level 21 provider lifecycle assumed the Level 20
    death/re-entry outcome, but deterministic Level 21 draw consumption makes
    that retreat succeed and return to the dungeon;
  - the test and restart corpus now assert the observed positive-HP dungeon
    state and issue `main_street`; the retained Level 17/20 profiles still
    prove death and re-entry. No reducer behavior changed.
- Deviations: none. The trusted renderer itself already passes clean v25 live
  input and cardinality evidence, so this slice ratchets its tests without
  changing production QML behavior.
- Post-validation production-QML correction prompted by the user's live report:
  - `TrustedButtonNode` now uses a native Qt Quick Controls `Button`; its
    custom visual contract remains, while the overlay `MouseArea` and manual
    `Accessible.onPressAction` are gone. The live object exposes one `Press`
    action plus `SetFocus`, rather than two `Press` actions;
  - `TrustedCartridgeSurface` invalidates the prior model immediately, guards
    replacement with a monotonically increasing generation, materializes the
    accepted nodes on the next event-loop turn, and publishes button
    accessibility after a 50 ms layout/polish interval;
  - the 23-to-24 regression now observes the empty retirement state before
    the current plan materializes and waits for all current buttons to become
    accessibility-ready before performing the existing cardinality, geometry,
    pointer, and Return checks;
  - focused `qmltestrunner` completed six cases green, and the complete
    renderer script passed Rust renderer tests, QML controls, every 256-node
    and rich-profile performance fixture, all non-ready states, and trusted
    accessibility rendering;
  - real-output evidence separated provisional hidden-workspace bounds from
    painted bounds: after workspace 8 receives a compositor frame, ten race
    controls expose ten unique non-overlapping rectangles. A registered
    one-click uinput pointer advanced exactly one provider revision; the live
    app was then restarted and parked untouched at revision 5 on the Level 21
    dungeon screen with 24 unique button IDs/actions.
- Deviation from the original manifest: production trusted-QML behavior is
  now intentionally changed in the two designed renderer files because the
  live audit reproduced duplicate native press semantics and provisional
  post-transition accessibility bounds. No protocol, gameplay, persistence,
  packaging, or provider boundary changed.
- Phase 3 exit: implementation and focused evidence satisfy the designed
  manifest; Phase 3 passes and the worktree is ready for skeptical inspection.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness | Level 21 preserves records 200–209, admits only 201–209 through the reviewed rejection condition, performs transition work on a clone, and fails closed at the 256-draw ceiling. | None | Accepted; exact encounter, boundary, at-cap, retreat, hostile-state, provider, and state-size evidence passes. |
| 2 | Architecture | External model/data/rules/provider/cartridge ownership remains intact. Platform lowering still emits one `RenderedNode` for each accepted signed node; no platform gameplay, server, database, migration, or protocol owner acquired Usurper rules. | None | Accepted; fresh CodeGraph plus direct QML inspection covered the one-hop blast radius. |
| 3 | Security | The complete seventeen-surface external diff review found no plausible capability, authorization, injection, path, deserialization, state-integrity, arithmetic, or resource-bound vulnerability. Retained render generations remain a documented prerequisite before any remote, lower-authority, or long-lived reuse. | None | Accepted; Codex Security scan `9ab6f0c9-ef3c-40df-86c4-0bc3b6d3d140` completed with zero findings and complete coverage. |
| 4 | QML / test quality | The signed dungeon screen contains twenty-four buttons, twenty-four unique button IDs, and twenty-four unique actions. The corrected trusted surface retires the old model for one guarded event-loop turn, delays accessibility publication until layout/polish, binds enabled state dynamically, and blocks ambiguous duplicate-action triggering. Native `Button` semantics remove the second manual accessibility press path. | None | Accepted; the 23-to-24 replacement test observes zero old delegates, then exactly twenty-four accessibility-ready current delegates and activates every control once by pointer and once by Return. |
| 5 | Runtime evidence | Fresh v26 screenshots show one visible row per control. Hidden-workspace AT-SPI bounds are provisional until the compositor paints the workspace; after a real workspace-8 frame, ten race controls exposed ten unique, vertically non-overlapping rectangles. Each native control exposes one `Press` plus `SetFocus`, and a compositor-routed physical pointer advanced exactly one provider revision. | None | Accepted; the failing layer was duplicate manual/native press semantics plus pre-paint provisional accessibility geometry, not duplicate signed nodes or visual delegates. The fresh app is parked at Level 21 revision 5 with 24 unique IDs/actions for user testing. |
| 6 | Scope | No Level 22+, dungeon-event/shared-realm behavior, renderer protocol/compiler semantics, persistence, packaging, admission, deployment, commit, push, or publication entered the slice. Two production trusted-QML files changed within the reopened design to correct the observed control lifecycle. | None | Accepted; the QML change remains game-neutral and does not acquire Usurper rules or provider authority. |

- Post-implementation structural inspection:
  - `compile_render_plan` validates the authenticated view, iterates each
    signed screen node once through `lower_node`, and appends one accepted
    `RenderedNode`; the Core budget remains 256 nodes;
  - direct QML inspection confirmed one `Repeater` over `acceptedPlan.nodes`,
    one `Loader` delegate per model row, `height: item ? item.height : 0`, and
    exactly-once pointer/Return dispatch through `TrustedButtonNode`;
  - the separate Usurper repository has no CodeGraph index, so its exact
    model/data/rules/provider/cartridge surfaces were reviewed directly;
  - the matching worktree-bound inspect receipt is recorded at
    `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`.
  - after the trusted-QML correction, fresh CodeGraph inspection reconfirmed
    that `compile_render_plan` still lowers one authenticated typed node per
    signed presentation node, requires declared actions, and remains bounded
    to its renderer/client-runtime consumers. Direct inspection covered QML,
    which remains unsupported by the Rust graph. The replacement inspect
    receipt is bound to gated state
    `4d92078454f7a7af30f10860f87aaad63d81d5b8067c67ab26c1c440299e6d2b`.
- Codex Security diff scan
  `9ab6f0c9-ef3c-40df-86c4-0bc3b6d3d140` completed all seventeen
  authoritative external review items with complete coverage and zero
  reportable findings. Its readable report is
  `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260903T132058Z_a74hpb59/report.md`.
- The reopened platform diff received a complete parent review because the
  session policy disabled delegated workers. Codex Security scan
  `ab7aa3ff-e9cd-499e-98fa-0ab5e10ef811` closed all three native inventory
  rows plus direct QML/supporting-test inspection with zero findings. Its
  report is
  `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchy_gaming_system/b7428c813bd72c1a8759333d20beef7b67696db4_20260903T143652Z_v9sbjj04/report.md`;
  measured usage was 4,518,005 total tokens, 4,504,623 input tokens, and
  4,443,264 cached input tokens with complete measurement coverage.
- Phase 3.5 exit: all inspection lenses are closed. Fresh full gates, isolated
  provider restart validation, and v26 workspace-8 runtime evidence remain
  Phase 4 obligations.

## Phase 4 — Validate

- External full validation: PASS. `scripts/test.sh` completed data (23),
  provider (41), local-play (3), rules (74), and integration (1) Rust tests,
  clippy, rustdoc, provenance/source hashes, six trusted-QML cases, seventeen
  signed screens, and the seven-action provider-backed local-play lifecycle.
- External provider restart validation: PASS. The fixed fifteen-case
  TLS/replay/fault/callback corpus completed twice across a real PostgreSQL
  provider restart on the disposable port-55432 stack; the stack and private
  credential file were removed afterward.
- First platform `bin/gate.sh --diff`: RED for test-environment containment,
  not accepted as product evidence. Stages 1--16, 18--20, 22, and 23 passed;
  stages 17, 21, and 24 failed because the isolated namespace launcher omitted
  the user-session environment needed by `systemd-run --user`. That made the
  packaged module host unavailable and left the recovery drill's module
  lifecycle disabled. The isolated database/network resources were removed,
  the host PostgreSQL listener remained untouched, and a namespace probe with
  the corrected `XDG_RUNTIME_DIR`/session-bus environment reported the systemd
  user manager running. A clean full rerun remains required.
- Corrected-environment diagnosis: PASS. The standalone production
  server-module conformance script completed its real systemd-user-scope plus
  Bubblewrap host flow, and a fresh disposable namespace/database rerun passed
  all 86 ignored database tests (including both prior CLI failures) plus the
  complete platform backup/restore drill. The targeted Compose project,
  volume, namespace, veth, proxy, and port-55432 listener were removed; the
  pre-existing host PostgreSQL listener on loopback port 5432 remained
  untouched.
- Fresh workspace-8 runtime setup: the exact old v25 process tree was stopped
  and cleaned, then a single current v26 local-play window was launched on
  workspace 8 while workspace 1 remained selected. Its live entry response is
  revision 0 with three nodes, one button, and zero duplicate IDs, actions, or
  labels. A second accessibility-enabled v26 launch exposed the real
  workspace-8 entry frame as five accessibility objects and exactly one
  3380-by-46 button rectangle, with no duplicate button name or bounds. The
  compositor is locked, so no pointer/keyboard input or visual result is
  claimed yet.
- Real-compositor signed dungeon cardinality: PASS. A temporary exact v26
  signed dungeon preview on workspace 9 exposed 28 accessibility objects and
  exactly 24 button objects. All 24 accessible names and screen rectangles
  were unique, every rectangle was 3380 by 46 pixels, and their y positions
  advanced by 54 pixels without overlap. The temporary preview was stopped
  and its generated run directory removed. This disproves duplicate delegate
  instantiation and overlapping control geometry on the current signed
  dungeon fixture, but does not replace the pending interactive v26 audit.
- Corrected full platform `bin/gate.sh --diff`: GREEN. All stages 1--24
  passed, including reproducible native packaging, the complete PostgreSQL
  integration corpus, API/QML smoke, provider conformance and recovery drills,
  platform backup/restore, and the real contained production module host. The
  gate receipt and current gated state both equal
  `68cf93ac1bf6e1ebcdfb9d92a4072a82a0510d0512f981925e4280798f6c3544`.
  The second isolated Compose project, volume, namespace, veth, proxy, and
  port-55432 listener were removed; only the pre-existing host loopback
  PostgreSQL listener remains.
- Post-report root-cause and live audit: PASS. The signed plan and visual
  delegate tree were unique; the defect was the custom button's overlapping
  native/manual accessibility press semantics together with accessibility
  publication before a replaced hidden-workspace plan had received its first
  compositor layout. The game-neutral trusted-QML correction now uses one
  native button, retires the old delegate model for a guarded event-loop turn,
  and publishes current accessibility only after layout/polish. On a painted
  workspace-8 frame the controls had unique non-overlapping rectangles, and a
  compositor-routed pointer click advanced exactly one provider revision. The
  fresh app is parked untouched on the Level 21 dungeon screen for user play.
- Post-correction external full validation: PASS. With the repository's
  explicit platform-root environment, `scripts/test.sh` completed all 142
  Rust tests (23 data, 41 provider, 3 local-play, 74 rules, and 1 integration),
  clippy, rustdoc, authenticated provenance, six trusted-QML cases, seventeen
  signed screens, and provider-backed local play.
- First post-correction platform `bin/gate.sh --diff`: RED only at stages 13a,
  13b, and 20 because the disposable namespace lacked an outbound route and
  DNS for clean-room crates.io package resolution. Every non-egress stage,
  including the complete PostgreSQL integration/API/QML, provider, recovery,
  private-alpha, and contained-module suites, passed. The namespace, veth,
  Compose project/volume, relay, and port-55432 listener were removed.
- Final post-correction platform `bin/gate.sh --diff`: GREEN. A fresh
  disposable namespace added temporary exact DNS/NAT egress while retaining a
  private PostgreSQL instance and the real user-session environment. All
  stages 1--24 passed, including the six-case trusted control suite, byte-for-
  byte SDK/provider/client packaging, 86 PostgreSQL-backed tests, 55 QML UI
  tests, live API smoke, provider restart/recovery, backup/restore, and the
  contained production module host. The gate receipt and current gated state
  both equal
  `4d92078454f7a7af30f10860f87aaad63d81d5b8067c67ab26c1c440299e6d2b`.
  Cleanup left no namespace, veth, NAT rule, Compose container/volume, relay,
  temporary DNS/Compose file, or port-55432 listener; the pre-existing host
  PostgreSQL listener on loopback port 5432 remained healthy.
- Phase 4 exit: focused, full, security, restart, real-input, visual, cleanup,
  and matching-receipt evidence is complete. Phase 4 passes.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS: authenticated source hashes/readback, exact records 200–209,
    source-trace validation, fixed-data tests, compatibility notes, and the
    Rust port map agree on the Level 21 band;
  - REQ-002 PASS: exact v26 identity, hostile/out-of-band state, malformed JSON,
    complete-state equality, and no-RNG-advance rejection tests pass;
  - REQ-003 PASS: levels 1–21 switch draw-free from valid phases, clear active
    encounters, and reject 0, 22, and `u16::MAX` unchanged;
  - REQ-004 PASS: the `Random(210)` rejection trace retains record 200,
    selects 201–209, initializes 25/12/75 combat, reaches accepted result 208
    on draw 256 for seed 553101, and remains within the 32 KiB state ceiling;
  - REQ-005 PASS: attack, retreat, potion, spell, class-special, reward, poison,
    deterministic replay, provider restart, and exact Level 21 retreat
    regressions pass without new combat behavior;
  - REQ-006 PASS: fixed/generic provider equivalence, bounded `option_u`,
    signed v26 screens, a guarded 23-to-24 delegate replacement, native
    accessibility/current bounds, exactly-once pointer and Return activation,
    seven provider-confirmed QML actions, painted workspace-8 evidence,
    security review, and the complete platform gate pass. Level 22, events,
    shared realm, platform gameplay ownership, packaging/admission/deployment,
    and publication remain absent.
- Documentation:
  - the external README, compatibility guide, Rust port map, and provenance
    trace describe the exact Level 21 source boundary and remaining limits;
  - `docs/architecture/game-cartridges.md` records rules v26, Level 21,
    twenty-four controls, guarded delegate retirement/materialization, and
    native button accessibility semantics;
  - OpenWiki update run `9023d073-b49a-45a0-85d6-d81a87842b07` completed
    after claim inspection/resolution and readback of
    `openwiki/quickstart.md` and `openwiki/game-cartridges.md`.
    Finalization retained pre-existing unresolved Claims evidence-debt warnings
    for those two pages but completed successfully; no Claims sidecar was
    hand-edited.
- Structural evidence:
  - final CodeGraph inspection reconfirmed authenticated view compilation,
    declared button/grid actions, one typed node per accepted signed node, the
    Core node budget, and the bounded renderer/client-runtime blast radius;
  - the matching inspect and OpenWiki completion receipts both bind pipeline
    `6f6945fc-f320-4e6d-931b-15a042c659eb` to gated state
    `06d97cc21082c889ba9006824c7dd51590069216d4ff7bf080e5c7ab879cd6f7`.
- Final gate:
  - a fresh post-documentation `bin/gate.sh --diff` passed all twenty-four
    stages, including the six trusted-control cases, reproducible client and
    provider packaging, 86 PostgreSQL-backed tests, 55 QML UI tests, live API
    smoke, provider restart/recovery, backup/restore, and the contained
    production module host;
  - the gate receipt and current gated state both equal
    `06d97cc21082c889ba9006824c7dd51590069216d4ff7bf080e5c7ab879cd6f7`;
  - cleanup left no Ticket 075 namespace, veth, NAT rule, Compose
    container/volume, relay, temporary DNS/Compose file, or port-55432
    listener. The pre-existing loopback PostgreSQL listeners on 5432 remain
    healthy.
- Runtime closeout: at the user's request, the Usurper QML client, Rust server,
  launcher, and temporary play directory were stopped/removed; workspace 8 has
  zero clients.
- Knowledge and archive: submitted `AAR-075`, registered three failures and
  three prevention rules, closed Ticket 075, and moved this spec/notes pair to
  `completed/`.
- Phase 5 exit: all six requirements are satisfied, durable documentation and
  knowledge are reconciled, receipts match the final gated worktree, cleanup
  is verified, and no delivery action was taken. Phase 5 passes.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first Level 21 provider profile expected the prior Level 20 death/re-entry outcome, but the deterministic Level 21 retreat succeeded and returned to the dungeon. | The test copied a prior band's terminal phase instead of deriving the outcome from the new band's actual draw tape. | Assert the observed positive-HP dungeon phase and continue with `main_street`; restore the accidentally touched Level 17 death/re-entry profile. | Lock each band's focused lifecycle to its own exact draw results and rerun the neighboring retained profile after any broad textual edit. |
| 2 | The user observed duplicate and inert controls in the loaded v25 view, and later clarified that most buttons appeared twice. | The custom trusted button combined native accessibility behavior with a manual press action, while replacement controls could be exposed before a hidden workspace received a compositor layout; the signed plan and painted delegate rows themselves were unique. | Use one native Qt Quick Controls button, retire the previous model for a guarded event-loop turn, delay accessibility publication until layout/polish, and prove a painted workspace-8 pointer advances one revision exactly once. | Keep 23-to-24 delegate retirement, native action cardinality/current bounds, real pointer/keyboard exactly-once activation, and provider-backed multi-screen replacement as release-blocking evidence for every larger plan. |
| 3 | The first post-correction full gate failed three clean-room package stages while every non-egress stage passed. | The disposable namespace intentionally isolated the database but did not yet have an outbound route or namespace DNS, so crates.io resolution could not occur. | Add temporary exact namespace DNS plus narrowly tagged forwarding/NAT rules, rerun the entire gate, and delete all network/database artifacts afterward. | Probe required clean-room egress before starting the full gate, retain exact tagged cleanup, and accept only a matching green receipt from the complete run. |
