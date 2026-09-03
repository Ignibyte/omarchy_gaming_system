---
title: Usurper Level Twenty Dungeon Band — notes
pipeline_id: b14f7737-8d10-47ed-9f34-5f376c56a0f0
---

# Usurper Level Twenty Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 073 supplies rules/state/cartridge v24, levels one through nineteen,
    a 256-draw bounded trace with provider-size evidence, and real activation
    of all twenty-two current controls;
  - the legacy-branch, rejected-RNG, trace-tail, provider-corpus, delegate
    lifecycle, row-geometry, and enabled-state rules remain binding;
  - the user's duplicate/inert report makes the twenty-third control surface a
    direct release boundary, even though the fresh v24 plan is unique.
- Source preflight:
  - authenticated `EDMONST.PAS` at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`
    lines 4527–4627 defines Level 20 records 190–199 as Hypnotic Snake,
    Dragon Owl, Huge Dront, Giant Eagle, Wizard, Orc Teenager, Orc Child,
    Celtic Warrior, Viking Leader, and Green Dwarf, all at base strength 24
    with source equipment flags;
  - authenticated `DUNGEONC.PAS` at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`
    lines 869–955 keeps events separate, spends a fight, and repeats
    `Random(level*10)` until the result exceeds `(level-1)*10`, so Level 20
    normally selects records 191–199 and stores record 190 only as source data;
  - the unregistered guard applies only above dungeon level 89, so Level 20
    remains on the ordinary branch;
  - authenticated `PLVSMON.PAS` at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`
    lines 68–138 uses `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initializes monster HP to strength times three.
- Bounded-trace preflight:
  - under uniform `Random(200)`, 256 consecutive rejected results have
    probability `(191/200)^256 = 7.600866862204e-6`;
  - deterministic development RNG state 367368 reaches accepted result 196 on
    draw 256 after 255 rejected values, giving an exact at-cap progress fixture;
  - the serialized-state ceiling remains an implementation-time proof, not an
    assumption.
- Decision: implement Level 20 as the next normal dungeon band and defer Level
  21, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, RNG draws, monster data, and provider projections;
  - the provider validates a revision-bound fixed Level 20 action, maps it to
    the existing typed `EnterDungeon` command, and asks the pure reducer for
    the next v25 state and view;
  - the signed inert cartridge binds bounded `option_t` to one declared
    button. The platform authenticates its schema/action, lowers it into the
    existing `RenderedNode::Button`, and trusted QML dispatches only an
    unconfirmed request;
  - state flow remains `signed button -> local revision/screen check ->
    provider action mapping -> pure reducer -> v25 state/view -> authenticated
    render plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` authenticates the screen schema and view, validates
    declared actions, lowers each signed button once, charges profile budgets,
    and returns unconfirmed requested actions;
  - `RenderedNode` retains five renderer-local consumers plus renderer
    integration coverage; session presentation/action consumers remain
    game-neutral and transport exact versions, digests, authority, action,
    and JSON without Usurper option letters or dungeon levels;
  - the Core profile permits 256 nodes, so the twenty-three-control Level 20
    dungeon plan remains inside the existing platform budget;
  - QML is outside the Rust graph and the separate Usurper repository has no
    `.codegraph/` index, so those surfaces were reviewed directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `b14f7737-8d10-47ed-9f34-5f376c56a0f0` at gated state
    `67d8958eeb72a2681038009693fd8c36a046118b329f30f8aaa27c6a61a949a1`.
- Exact implementation manifest, one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_t` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 190–199,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v25, extend validation/switching/labels through Level 20, and add exact
    encounter, at-cap rejection, retreat, deterministic, and hostile-state
    evidence;
  - external `crates/usurper-provider/src/lib.rs`: map fixed Level 20, prove
    generic/fixed equivalence and projection, lock a focused combat lifecycle,
    and retain maximum-state evidence with Level 20 trace values;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare
    one Level 20 action/button, and require bounded `option_t`;
  - all seventeen external `fixtures/presentation/*.json`: supply the required
    field, with non-empty Level 20 text only on the dungeon view and
    source-valid Level 20 facts on the combat fixture;
  - external `provenance/source-trace.json`: register reviewed Level 20 editor,
    selection, HP, and retreat branches;
  - external `scripts/test-cartridge.sh`, `scripts/test-provider.sh`,
    `scripts/test.sh`, and `scripts/play.sh`: assert exact v25/Level 20
    identities and uniqueness, lock the provider corpus to its focused
    post-combat phase, and end smoke play in a Level 20 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the implemented band, the 256-draw Level
    20 tail evidence, and remaining visible limits;
  - platform `client/qml/cartridge/TrustedCartridgeSurface.qml`,
    `client/qml/tests/CartridgeLocalPlay.qml`, and
    `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`:
    ratchet the game-neutral largest-plan proof from 22 to 23 actual delegates
    while retaining positive-height, non-overlap, enablement, stale-removal,
    and one real pointer plus Return activation per current control; expose
    bounded loaded-node/action diagnostics and use them to drive the exact
    provider-backed entry, race, class, street, Level 1, Level 20, Look, and
    combat-action lifecycle;
  - platform `docs/architecture/game-cartridges.md`: reconcile durable
    external-development facts through v25/Level 20 during Phase 5. No
    platform production QML, gameplay, renderer protocol/compiler, server,
    migration, Cargo, or provider-protocol change is required.
- Database and migration consequences: none. Provider-owned state stays in the
  external adapter; this slice adds no platform persistence, table, column,
  migration, or PostgreSQL write path.
- API and compatibility contract:
  - strict state JSON requires exact `schema_version: 25`; v24 and malformed
    v25 state fail before RNG construction or mutation;
  - the view schema adds required string `option_t` with the existing
    64-character bound. All screens provide it; only the dungeon screen binds
    it to `enter_dungeon_level_20`;
  - signed `rules_version` and `cartridge_version` advance together to 25;
    SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_20` accepts an empty payload and maps to the existing
    typed command. Levels 0, 21, and `u16::MAX` remain rejected without
    revision or RNG advance;
  - `MAX_TRACE_DRAWS` remains 256. Under uniform `Random(200)` values the
    theoretical probability of 256 consecutive rejected results is
    `7.600866862204e-6`; deterministic state 367368 supplies a valid 256-draw
    run (255 rejections, then result 196), and the maximum-state regression
    must still prove the 32 KiB starter ceiling.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_TWENTY_MONSTERS` arrays, source-trace validation, authenticated-source hashes/readback, compatibility and port-map review |
  | REQ-002 | v25 identity checks; old/missing/unknown JSON fields; Level 21, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | sequential levels 1–20 switching with unchanged RNG/empty traces, complete visible labels, ascent/descent/remain behavior, and rejected 0/21/max inputs |
  | REQ-004 | forced rejected `Random(200)` draw followed by 191–199, exact 24 strength/12 defence/72 HP, fight decrement, deterministic twin equality, 256-draw seed-367368 progression, quantified tail, and maximum serialized-state proof |
  | REQ-005 | exact failed-retreat `(2, 1), (200, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_t`, focused phase-aware live profile and restart corpus, signed-screen/action uniqueness, 22-to-23 Qt delegate replacement, seven-action provider-backed local-play confirmation, and post-unlock workspace-8 runtime audit |
- Risks and controls:
  - strict schemas, identifier checks, empty payloads, authenticated cartridge
    content, loopback capability, and revision/screen binding reject undeclared
    or stale actions without accepting platform identity or reusable secrets;
  - provider serialization and pre-RNG validation prevent rejected or stale
    commands from partially advancing state;
  - the source rejection loop is theoretically unbounded while the provider
    trace is capped. The explicit tail calculation, exact 256-draw valid case,
    and maximum serialized-state test expose the remaining development limit;
  - the focused provider corpus runs twice across process restart and asserts
    its actual post-combat phase before the next fixed command;
  - the twenty-three-button dungeon plan is the new largest action surface.
    Recursive 22-to-23 delegate counts, positive row geometry, stale removal,
    enabled-state propagation, surface hit-testing, and Return exactly-once
    activation cover duplicate or inert controls;
  - the local launcher remains same-developer, loopback/private, ephemeral and
    must gain hard render-generation eviction before lower-authority, shared,
    remote, multi-user, or long-lived reuse;
  - v25 artifacts can be removed before delivery without migration; no
    publication or delivery action is authorized.
- Decisions and rejected alternatives:
  - preserve record 190 as canonical data but never select it normally; direct
    selection would contradict the reviewed rejection loop;
  - add bounded `option_t`; overloading prior fields or changing the renderer
    would widen semantics unnecessarily;
  - retain 256 draws because Level 20 reaches a valid result exactly at the cap
    and the maximum-state proof exercises its envelope. Widening the trace
    would consume the fixed state budget without solving compact history;
  - reuse the generic reducer; provider or QML copies would create a second
    rules authority;
  - keep events and registration paths excluded because neither is required
    for the ordinary Level 20 band.
- Phase 2 exit: the source, ownership, compatibility break, exact file
  manifest, regression evidence, and operational risks are fully specified.

## Phase 3 — Implement

- Built the Phase 2 manifest without widening platform production ownership or
  persistence:
  - external model/view now includes bounded `option_t`;
  - external data preserves exact editor records 190–199 and source equipment
    flags, including the normally unreachable Hypnotic Snake boundary row;
  - external rules use exact v25 state, allow levels 1–20, preserve rejected
    record-190 draws, initialize strength-24 monsters at 72 HP, and retain the
    Level 20 retreat bound;
  - provider maps the fixed Level 20 action through the generic reducer,
    projects the twentieth label, and proves its deterministic combat sequence;
  - signed manifest/presentation/schema, seventeen screen fixtures,
    provenance, scripts, and compatibility docs now describe v25/Level 20;
  - the platform QML regression now replaces twenty-two old controls with
    twenty-three current controls while retaining recursive cardinality,
    positive non-overlapping row geometry, dynamic enablement, stale-delegate
    removal, and one real pointer plus Return activation per current control;
  - after the user reported that the live controls appeared duplicated and
    inert, the generic trusted surface gained bounded loaded-node and exact
    action-count diagnostics, and the provider-backed smoke was expanded from
    one confirmation to seven consecutive actions across six replaced plans,
    ending with a confirmed Level 20 combat action.
- Focused implementation checks:
  - exact Level 20 data, three Level 20 reducer tests, fixed/generic provider
    equivalence, focused live profile, strict v25 hostile state, draw-free
    level switching, and maximum 256-draw provider-state ceiling: PASS;
  - deterministic RNG state 367368 produced 255 rejected `Random(200)` draws
    and accepted result 196 on draw 256 exactly;
  - signed cartridge and trusted-QML suite: PASS, six cases and seventeen
    signed screens;
  - JSON parsing, shell syntax, formatting, and whitespace checks: PASS.
- Full external `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh`: PASS, including strict Clippy/rustdoc, authenticated
  sources/provenance, 136 Rust tests (22 data, 39 provider, 3 local-play, 71
  rules, and 1 integration), six QML cases, seventeen signed screens, and
  provider-backed local play.
- The post-report focused QML control suite passed all six cases, and the
  expanded real-provider local-play smoke passed the seven-action lifecycle.
  An offscreen capture of the same trusted combat plan showed one visual row
  per control with no duplicate rendering.
- The full fixed fifteen-case TLS/replay/fault/callback corpus passed twice
  across provider restart against an isolated PostgreSQL 18 fixture on port
  55432. The first disposable container exited before testing because its
  tmpfs used the pre-18 data path; the corrected `/var/lib/postgresql` fixture
  passed. Both the failed ephemeral container and successful volatile
  database, listener, and mode-0600 credential file were removed. The system
  PostgreSQL service on 5432 was not changed.
- Live workspace evidence: the old v24 process was stopped and its private run
  root removed. The first fresh v25 QML process was launched on workspace 8
  while Hyprlock was active; Qt reported that Wayland exposed no outputs and
  created a placeholder screen. That session was retired because its painting
  and pointer behavior were not valid runtime evidence. A clean v25 launch is
  queued for workspace 8 after unlock while workspace 1 remains active.
- Deviations: user runtime evidence widened the planned test-only platform
  manifest to the trusted-surface diagnostics and multi-transition local-play
  smoke described above; it did not change rendering or gameplay behavior.
  The corrected PostgreSQL 18 tmpfs mount was a fixture-only operational
  adjustment.
- Phase 3 exit: implementation and focused evidence satisfy the designed
  manifest; the worktree is ready for correctness, architecture, security,
  test-quality, and scope inspection.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness | Level 20 preserves exact records 190–199, admits only 191–199 through the reviewed rejection condition, performs all transition work on a clone, and fails closed at the 256-draw ceiling. | None | Accepted; focused and full reducer/provider evidence covers the boundary. |
| 2 | Architecture | External model/data/rules/provider/cartridge ownership remains intact; platform Rust renderer, Provider SDK/protocol, server, database, and migrations do not acquire Usurper rules. | None | Accepted; CodeGraph inspection receipt recorded for the current worktree. |
| 3 | Security | `scripts/play.sh` placed the local session bearer in curl and long-lived QML argv; a distinct UID recovered it through procfs and used it for one authenticated read and mutation. | Low | Fixed: curl now reads a mode-0600 private config and QML reads the validated mode-0600 private startup file. A repeated cross-UID probe found no bearer or endpoint in argv and could not read the startup file. |
| 4 | Test quality | The original provider-backed QML smoke confirmed only its first action, so it did not prove successive plan replacement and later controls. | Medium | Fixed: smoke now confirms seven actions through entry, race, class, street, Level 1, Level 20, Look, and Attack while checking one loaded node and one matching action at every step. |
| 5 | Runtime evidence | A preview launched while Hyprlock hid Wayland outputs used Qt's placeholder screen, invalidating visual and pointer observations. | Medium | Retired; the replacement launch waits for unlock and must complete on workspace 8 during Phase 4. |
| 6 | Scope | No Level 21/event/shared-realm behavior, production gameplay QML, Rust renderer/protocol, persistence, migration, packaging, admission, deployment, or publication change entered the slice. | None | Accepted. |

- CodeGraph inspection:
  - `compile_render_plan` remains the single signed-schema/action lowering path
    to `RenderedNode::Button`; five renderer-local consumers and their existing
    test blast radius were identified;
  - QML stays outside the Rust graph and was inspected directly;
  - receipt `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt` binds
    pipeline `b14f7737-8d10-47ed-9f34-5f376c56a0f0` to state
    `4cb3599767cab845eec9f6fb80a395739b9d36ddee9299a4792dd244c01ad7fc`.
- Codex Security diff scan
  `68b84967-e53e-49f4-a53d-74f0610b6787` completed all seventeen external
  inventory items with complete coverage. It reported one high-confidence,
  low-severity finding, `FD-LOCAL-ARGV-001`, and no rules, provider-state,
  cartridge, provenance, injection, path, deserialization, arithmetic, or
  bounded-resource finding.
- Security reproduction and remediation evidence:
  - before the fix, UID 65534 read the UID 1000 QML argv, recovered the bearer
    and loopback endpoint, authenticated `GET /v1/session`, and submitted the
    fresh revision-zero `continue` action; the disposable session was stopped
    immediately;
  - after the fix, the same cross-UID context reported no bearer or endpoint
    in QML argv and could not read the private startup file;
  - shell syntax, QML lint, whitespace, six QML controls, and the seven-action
    provider-backed smoke all pass after remediation.
- Phase 3.5 exit: all findings are either accepted or fixed with direct
  evidence. Post-unlock workspace-8 observation and full gates remain Phase 4
  obligations.

## Phase 4 — Validate

- Tests run:
  - full external `env -u CARGO_TARGET_DIR TMPDIR=/tmp
    CARGO_NET_OFFLINE=true
    OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    ./scripts/test.sh`: PASS, including strict Clippy/rustdoc, 22 data tests,
    39 provider tests, three local-play tests, 71 rules tests, one integration
    test, authenticated source/provenance validation, six QML cases,
    seventeen signed screens, and the seven-action provider-backed lifecycle;
  - complete fixed fifteen-case provider TLS, replay, fault, callback, and
    reconciliation profile twice across restart: PASS against the isolated
    PostgreSQL 18 fixture;
  - focused post-fix private-handoff probe: PASS; cross-UID inspection found
    neither bearer nor endpoint in QML argv and could not read the mode-0600
    startup document;
  - post-unlock live workspace-8 observation: PASS; the v25 QML client mapped
    on a real Wayland output, showed one separated row for each race action,
    and reached provider revision 1 on `create-race` after the visible
    `Continue` control was activated. The previously active workspace was
    restored after capture, and the clean game window remains on workspace 8.
- Gate run:
  - `bin/gate.sh --diff`, executed through the canonical isolated receipt
    wrapper with disposable compose project `ogs_ticket074_gate` and host port
    55432: `GATE GREEN [diff]`, all twenty-four stages;
  - the pre-completion receipt recorded gated state
    `4cb3599767cab845eec9f6fb80a395739b9d36ddee9299a4792dd244c01ad7fc`;
  - the disposable relay runtime, PostgreSQL container, volume, and listener
    were absent after cleanup. The system PostgreSQL listener on 5432 remained
    available and was not changed.
- Skips or pre-existing failures:
  - no pre-existing failure was accepted;
  - the gate's inner production-runtime-containment path was intentionally
    deferred by the isolated harness and then executed by its canonical outer
    wrapper; stage 24 passed, so no required containment evidence was skipped.
- Phase 4 exit: every planned focused, full, restart, security-remediation,
  compositor, and delivery-gate check passes. The pipeline is ready for its
  acceptance audit, durable documentation, AAR, and archive.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS: authenticated-source hashes/readback, exact Level 20 data
    tests, source-trace validation, and compatibility docs bind editor records
    190–199 plus selection, HP, and retreat branches;
  - REQ-002 PASS: exact v25 identity, hostile JSON/state, out-of-band level and
    monster, oversized scalar, and complete state/RNG immutability tests pass;
  - REQ-003 PASS: sequential levels 1–20 switching, all boundary rejections,
    location/phase assertions, empty monster state, and unchanged RNG/traces
    pass;
  - REQ-004 PASS: exact 191–199 reachable roster, preserved rejected record
    190, 72 HP, fight spend, deterministic equality, seed-367368 at-cap
    progression, quantified `7.600866862204e-6` tail risk, and maximum-state
    ceiling pass;
  - REQ-005 PASS: attack, retreat, potion, spell, class-special, reward,
    poison, replay/restart, and exact Level 20 retreat trace regressions pass;
  - REQ-006 PASS: fixed/generic provider equivalence, signed v25 screens,
    twenty-two-to-twenty-three non-overlapping delegate replacement, one real
    pointer and Return activation per current control, seven provider-confirmed
    QML actions, restart corpus, security remediation, and the clean live
    workspace-8 revision-1 race screen all pass. Level 21, events, shared
    realm, platform gameplay ownership, packaging, admission, deployment, and
    publication remain absent.
- Docs:
  - hand-maintained `docs/architecture/game-cartridges.md` now records v25,
    Level 20, the twenty-three-control and seven-action evidence, the private
    startup/config handoff, and the real-compositor-output rule;
  - the external `README.md`, `docs/COMPATIBILITY.md`, and `docs/RUST_PORT_MAP.md`
    describe the implemented band, source links, trace limit, local play, and
    remaining boundaries;
  - OpenWiki update run `877b8062-0476-4c60-8e3c-4c8851226243` completed after
    readback of `openwiki/game-cartridges.md` and `openwiki/quickstart.md`.
    Finalization retained its existing warnings for unresolved repository-wide
    Claims evidence debt on both pages; it did not block lifecycle completion
    and no claim sidecar was hand-edited.
- Final gate:
  - the first completion rerun omitted the compose project/file environment
    inside the namespace, so database stages attempted the host-occupied 5432
    mapping and the gate ended red. Its exact disposable default-project
    container, volume, and network were removed; no source or host database was
    changed;
  - the corrected canonical isolated wrapper propagated the project and
    override, after which all twenty-four stages passed with
    `GATE GREEN [diff]`;
  - gate and OpenWiki completion receipts both match state
    `8ad814ee522412a1d5ba0059de126cf3617361c8f2cb4fd89c986a762ddcabab`;
  - both disposable compose projects, their volumes and relays are absent,
    port 55432 is free, and the system PostgreSQL listeners on 5432 remain.
- AAR: submitted `AAR-074`; registered the capability-argv and placeholder-
  output evidence failures plus the real-compositor-output prevention rule.
- Archive: closed Ticket 074 and moved this spec/notes pair to `completed/`.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A workspace-8 preview appeared to render duplicate controls and did not accept input. | The QML process was launched while Hyprlock held the compositor; Qt received no Wayland outputs and created a placeholder screen, so the window was not valid visual/input evidence. | Retired the session, queued relaunch until after unlock, captured the trusted plan offscreen, and expanded local play to seven provider-confirmed actions with exact loaded-node/action checks. | Never accept a preview launched against Qt's placeholder output; require post-unlock Wayland launch plus provider-confirmed multi-plan interaction evidence. |
| 2 | The security scan reproduced cross-user theft of the ephemeral local-play bearer. | The launcher copied the bearer from a private startup document into curl and QML command-line arguments, while this host exposes process argv across UIDs. | Moved curl authorization into a mode-0600 config, moved QML setup to the validated private startup file, and dynamically verified the old attack path is closed. | Treat argv and environment as public metadata; regress both private-file properties and absence of capability-shaped QML arguments. |
| 3 | The first post-OpenWiki gate rerun reported four database-stage failures. | The namespace invocation omitted the compose project/file environment, so Docker attempted the default host-5432 mapping instead of the isolated 55432 fixture. | Removed only the disposable default-project resources, propagated the compose override, and reran the complete gate to green. | Apply the existing fixture-port/identity preflight to both the outer fixture and every compose invocation inside the namespace. |
