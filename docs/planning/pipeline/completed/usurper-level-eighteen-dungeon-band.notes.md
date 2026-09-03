---
title: Usurper Level Eighteen Dungeon Band — notes
pipeline_id: f688f06e-041d-43c6-956f-5f56de3c88e3
---

# Usurper Level Eighteen Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 071 supplies rules/state/cartridge v22, levels one through
    seventeen, a 256-draw bounded trace with provider-size evidence, live
    unique-control readback, and recursive 19-to-20 delegate proof;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-170 draw to remain visible in deterministic traces;
  - `PR-omarchy-gaming-system-size-rejection-traces-against-valid-tail-risk-001`
    requires Level 18's bounded-trace risk, long valid progression, and maximum
    serialized provider state to be proved rather than assumed;
  - `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001`,
    `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`, and
    `PR-omarchy-gaming-system-count-instantiated-delegates-across-plan-replacement-001`
    keep the eighteenth choice unique and inside the actual Qt input boundary;
  - `PR-omarchy-gaming-system-lock-provider-corpus-to-tested-phase-transitions-001`
    requires any Level 18 conformance sequence to assert its post-combat phase
    before selecting the next fixed command.
- Source preflight:
  - authenticated `EDMONST.PAS` at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`
    lines 4323–4423 defines Level 18 records 170–179 as Ghoul, Giant Eagle,
    Gnome Warrior, Baby Dront, Flying Bear, Uruk-Hai, Dunedain, Dream Lord,
    Fallguy, and Renegade Troll, all at base strength 22 with source equipment
    flags;
  - authenticated `DUNGEONC.PAS` at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`
    lines 868–955 keeps events separate, spends a fight, and repeats
    `Random(level*10)` until the result exceeds `(level-1)*10`, so Level 18
    normally selects records 171–179 and stores record 170 only as source data;
  - the unregistered guard applies only above dungeon level 89, so Level 18
    remains on the ordinary branch;
  - authenticated `PLVSMON.PAS` at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`
    lines 68–138 uses `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initializes monster HP to strength times three.
- The informational rebuild bulletin and handoff were reviewed. Pipeline tools
  report CodeGraph 1.5.0 and OpenWiki 0.3.3 ready; no Docker service is active.
- Baseline `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` passed: formatting, strict Clippy, 118 Rust tests,
  rustdoc, authenticated upstream/provenance checks, six real-input QML cases,
  seventeen unique signed screens, and provider-backed local play.
- Decision: implement Level 18 as the next normal dungeon band and defer Level
  19, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, RNG draws, monster data, and provider projections;
  - the provider validates a revision-bound action, maps fixed Level 18 to the
    existing typed `EnterDungeon` command, and asks the pure reducer for the
    next v23 state and view;
  - the signed inert cartridge binds bounded `option_r` to one declared
    button. The platform authenticates its schema and action, lowers it into
    one `RenderedNode::Button`, and trusted QML dispatches only an unconfirmed
    request;
  - state flow remains `signed button -> local revision/screen check ->
    provider action mapping -> pure reducer -> v23 state/view -> authenticated
    render plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` is used by the client cartridge runtime, preview CLI,
    and renderer tests; `RenderedNode` is lowered inside the renderer and its
    direct coverage remains `crates/game-cartridge-renderer/tests/rendering.rs`;
  - Core profile capacity is 256 nodes, so the Level 18 dungeon plan remains
    far inside the platform budget and does not require a renderer change;
  - server session-cartridge presentation/action consumers use game-neutral
    version, digest, action, and JSON contracts and do not depend on Usurper
    option letters or dungeon levels;
  - QML is outside the Rust AST graph and the separate Usurper repository has
    no `.codegraph/` index, so their actual files/tests were reviewed directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `f688f06e-041d-43c6-956f-5f56de3c88e3` at gated state
    `61a39516d74c6ff20c871d76f6e3931c82b515d98bf8c196b65cffe6651d89a5`.
- Exact implementation manifest, one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_r` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 170–179,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v23, extend validation/switching/labels through Level 18, and add exact
    encounter, long-rejection, retreat, deterministic, and hostile-state
    evidence;
  - external `crates/usurper-provider/src/lib.rs`: map fixed Level 18, prove
    generic/fixed equivalence and projection, lock a focused combat lifecycle,
    and retain maximum-state evidence with Level 18 trace values;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare
    one Level 18 action/button, and require bounded `option_r`;
  - all seventeen external `fixtures/presentation/*.json`: supply the required
    field, with non-empty Level 18 text only on the dungeon view and
    source-valid Level 18 facts on the combat fixture;
  - external `provenance/source-trace.json`: register reviewed Level 18 editor,
    selection, HP, and retreat branches;
  - external `scripts/test-cartridge.sh`, `scripts/test-provider.sh`,
    `scripts/test.sh`, and `scripts/play.sh`: assert exact v23/Level 18
    identities and uniqueness, lock the provider corpus to its focused
    post-combat phase, and end smoke play in a Level 18 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the implemented band, 256-draw Level 18
    tail evidence, and remaining visible limits;
  - platform
    `client/qml/cartridge/TrustedCartridgeSurface.qml` and
    `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`: ratchet
    the game-neutral large-screen proof from 20 to 21 actual delegates and,
    after live defect evidence, bind every loader row to its loaded item's
    height;
  - platform `docs/architecture/game-cartridges.md`: reconcile durable
    external-development facts through v23/Level 18 during Phase 5. No
    platform gameplay, renderer protocol/compiler, server, migration, Cargo,
    or provider-protocol change is required.
- Database and migration consequences: none. Provider-owned state stays in the
  external adapter; this slice adds no platform persistence, table, column,
  migration, or PostgreSQL write path.
- API and compatibility contract:
  - strict state JSON requires exact `schema_version: 23`; v22 and malformed
    v23 state fail before RNG construction or mutation;
  - the view schema adds required string `option_r` with the existing
    64-character bound. All screens provide it; only the dungeon screen binds
    it to `enter_dungeon_level_18`;
  - the signed manifest advances `rules_version` and `cartridge_version`
    together to 23; SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_18` accepts an empty payload and maps to the existing
    typed command. Levels 0, 19, and `u16::MAX` remain rejected without
    revision or RNG advance;
  - `MAX_TRACE_DRAWS` remains 256. Under uniform `Random(180)` values the
    theoretical probability of 256 consecutive rejected results is
    `1.982635846e-6`; deterministic state 124639 supplies a 232-draw valid run
    (231 rejections, then result 173), and the maximum-state regression must
    still prove the 32 KiB starter ceiling.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_EIGHTEEN_MONSTERS` arrays, source-trace validation, authenticated-source hashes/readback, compatibility and port-map review |
  | REQ-002 | v23 identity checks; old/missing/unknown JSON fields; Level 19, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | sequential levels 1–18 switching with unchanged RNG/empty traces, complete visible labels, ascent/descent/remain behavior, and rejected 0/19/max inputs |
  | REQ-004 | forced rejected `Random(180)` draw followed by 171–179, exact 22 strength/11 defence/66 HP, fight decrement, deterministic twin equality, 232-draw seed-124639 progression, quantified `1.982635846e-6` tail, and maximum serialized-state proof |
  | REQ-005 | exact failed-retreat `(2, 1), (180, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_r`, focused phase-aware live profile and restart corpus, signed-screen/action uniqueness, 20-to-21 Qt delegate replacement, local-play confirmation, and workspace-8 runtime audit |
- Risks and controls:
  - security/input: strict schemas, identifier checks, empty payloads,
    authenticated cartridge content, loopback capability, and revision/screen
    binding reject undeclared or stale actions;
  - privacy/secrets: no platform identity or reusable credential enters game
    state; local capabilities and signing keys remain temporary and unlogged;
  - state/concurrency: provider serialization and pre-RNG validation prevent
    rejected or stale commands from partially advancing state;
  - RNG/resource: the source rejection loop is theoretically unbounded while
    the provider trace is capped. The explicit tail calculation, 232-draw
    deterministic case, and maximum serialized-state test make this remaining
    development limitation visible; deeper levels still require compaction or
    redesign rather than indefinite vector growth;
  - reconnect/restart: the fixed provider corpus runs twice across process
    restart, and a focused Level 18 test must assert the actual phase/state
    before the next command is encoded;
  - rendering: the twenty-one-button dungeon plan is the new largest action
    surface. Recursive 20-to-21 instantiated-delegate counts, signed-plan
    uniqueness, and real pointer/Return input cover duplicate or inert
    controls;
  - rollback: v23 artifacts can be removed before delivery without migration;
    no publication or delivery action is authorized.
- Decisions and rejected alternatives:
  - preserve record 170 as canonical data but never select it normally;
    direct selection would contradict the reviewed rejection loop;
  - add bounded `option_r`; overloading primary/secondary fields would blur
    phase semantics, while a grid or renderer change would unnecessarily
    expand platform behavior;
  - retain the 256-draw limit for Level 18 because it keeps the state contract,
    yields a quantified roughly two-in-a-million tail, and supports a
    deterministic 232-draw valid case. Widening without a compact encoding
    would consume the fixed state budget and merely postpone the structural
    issue;
  - reuse the generic reducer; copying Level 18 rules into provider or QML
    would create a second authority;
  - keep events and registration paths excluded because neither is required
    for the ordinary Level 18 band.
- Phase 2 exit: the source, ownership, compatibility break, exact file
  manifest, regression evidence, and operational risks are fully specified.

## Phase 3 — Implement

- Implemented the Phase 2 manifest without widening platform production
  ownership or persistence:
  - external model/view now includes bounded `option_r`;
  - external data preserves exact editor records 170–179 and lookup routing;
  - external rules use exact v23 state, allow levels 1–18, preserve rejected
    record-170 draws, initialize strength-22 monsters at 66 HP, and retain the
    Level 18 retreat bound;
  - provider maps the fixed Level 18 action through the generic reducer,
    projects the eighteenth label, and proves its deterministic death/re-entry
    combat sequence;
  - signed manifest/presentation/schema, seventeen screen fixtures,
    provenance, scripts, and compatibility docs now describe v23/Level 18;
  - platform QML replacement coverage expects twenty old and twenty-one new
    recursively instantiated button delegates.
- The deterministic Level 18 Gnoll/Cleric sequence confirmed the expected
  `dead` phase after spell then retreat; the focused test and restart corpus
  both use `reenter`, preserving the phase-locked behavior established by
  Ticket 070.
- Focused implementation checks:
  - exact Level 18 data test: PASS;
  - three Level 18 reducer tests, including the seed-124639 232-draw sequence:
    PASS;
  - fixed/generic provider equivalence and focused combat lifecycle: PASS;
  - strict v23 hostile-state regression and maximum 256-draw provider-state
    ceiling: PASS;
  - signed cartridge and trusted-QML suite: PASS, six cases;
  - provider-backed local HTTP/QML smoke: PASS;
  - JSON parsing, shell syntax, formatting, and whitespace checks: PASS.
- Full external `env -u CARGO_TARGET_DIR TMPDIR=/tmp
  CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh`: PASS, including strict Clippy/rustdoc, authenticated
  sources/provenance, 124 Rust tests (20 data, 35 provider, 3 local-play, 65
  rules, and 1 integration), six QML cases, seventeen signed screens, and
  provider-backed local play.
- Completeness checks found `option_r` in all seventeen fixtures and populated
  only on `dungeon.json`; the signed presentation contains exactly one Level
  18 action/button and no Level 19 action. No Cargo, platform gameplay,
  renderer protocol/compiler, provider protocol, migration, packaging,
  admission, or publication surface entered this slice.
- Live-control defect follow-up:
  - the user reported that the previously open workspace-8 window had inert
    controls, then confirmed that most controls appeared twice even after the
    current build was opened;
  - provider readback from that v22 process proved the active combat plan had
    five buttons, five unique IDs, and five unique actions, excluding duplicate
    provider projection as the source;
  - frame-stable offscreen renders of the same signed plans showed one visual
    control per action. The stale v22 QML process had been instantiated before
    the current dynamic `actionsEnabled` binding and retained the old disabled
    delegate state, explaining the inert controls but not the separate live
    duplicate-render report;
  - the duplicate-render report exposed a transient layout blind spot: each
    heterogeneous `Repeater` delegate was an asynchronous `Loader` whose row
    height was implicit until its item settled. The production trusted surface
    now binds the loader height explicitly to the loaded item, while retaining
    the existing width and action-authority boundaries;
  - the stale window/provider were stopped and replaced with a fresh v23
    process after the row-height change. It is the only Usurper window on
    workspace 8; the active user workspace remained 1;
  - real Hyprland `Return`/`Tab` input delivered directly to that window drove
    entry -> Human -> Alchemist -> Main Street -> dungeon level 18 -> Look.
    Readback proved revision 6 combat against a Level 18 Fallguy at 66 HP;
  - after the row-height change, the refreshed dungeon readback contained
    exactly twenty-one buttons, twenty-one unique IDs, twenty-one unique
    actions, one Level 18 action, and no Level 19 action. The window remains
    open on workspace 8 at Level 18 for user visual confirmation;
  - the platform QML regression now waits for settled layout, proves all
    twenty-one controls do not overlap, verifies each disabled-to-enabled
    binding, and delivers exactly one Return action per control. All six QML
    cases and the complete trusted-renderer gate pass.
- Phase 3 exit: implementation and focused evidence satisfy the designed
  manifest; the worktree is ready for independent correctness, architecture,
  security, test-quality, and scope inspection.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness | Exact Level 18 source data, rejection-loop selection, 66-HP derivation, v23 state/view identity, fixed/generic provider mapping, and restart corpus agree. | none | PASS; no correction required. |
| 2 | Architecture | The external provider remains the only game-state/rules authority. The platform change supplies only game-neutral delegate geometry and does not interpret Usurper state or actions. | none | PASS; spec scope amended to include the required layout hardening. |
| 3 | Security | Codex Security scan `5670926d-67b1-41ec-9260-baf28c327095` closed all 17 frozen review items with zero reportable findings. Forty valid local actions reproduced forty retained render generations, but attack-path policy rejected the candidate as same-developer, loopback-only, ephemeral self-impact. | informational lifecycle debt | Preserve the rejected coverage row and require eviction/count/byte limits before any shared, lower-authority, remote, multi-user, or long-lived reuse. Report: `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260903T085815Z_vwa05xf5/report.md`. |
| 4 | Test quality | Settled object cardinality did not constrain the asynchronous loader row during plan replacement. | medium regression gap | Fixed with explicit loader height, positive-height/non-overlap assertions, and one real Return event per each of 21 controls; full renderer gate passes. |
| 5 | Scope | The live duplicate-control report required one platform production-QML geometry change beyond the original implementation manifest. | low controlled deviation | Accepted and documented because it directly satisfies REQ-006; no platform gameplay, protocol, compiler, server, database, packaging, deployment, or publication surface changed. |

- Phase 3.5 exit: PASS. Correctness, ownership, security, test-quality, and
  scope findings are dispositioned; no reportable security finding or
  unresolved code-inspection issue remains.

## Phase 4 — Validate

- Tests run:
  - external full `./scripts/test.sh`: PASS after the Level 18 implementation,
    with formatting, strict Clippy, rustdoc, 124 Rust tests, authenticated
    source/provenance checks, six QML cases, seventeen signed screens, and
    provider-backed local play;
  - platform `./scripts/test-game-cartridge-renderer.sh`: PASS after the
    explicit loader-height change, including 11 Rust renderer tests, six QML
    cases, hostile contracts, and the complete performance/state matrix;
  - fresh workspace-8 v23 play: PASS for real Return/Tab transitions through
    entry, creation, Main Street, and Level 18 selection; the current live
    session is revision 7 on the Level 18 dungeon screen with 21 unique
    button IDs and actions.
- Gate run:
  - the first `bin/gate.sh --diff` was RED only in checks 17, 18, 19, 19a, 20,
    21, and 22 because the machine's enabled system `postgresql.service`
    already owns `127.0.0.1:5432`; the compose test database could not bind,
    and alternating stages reached the unrelated system cluster where the
    expected `omarchy_gaming_system` role does not exist;
  - the unchanged gate was rerun inside a temporary network namespace whose
    loopback port 5432 forwarded to a disposable compose PostgreSQL published
    only on an isolated host-veth address and port 55432. User-session D-Bus,
    runtime, and Wayland variables were restored inside the namespace for the
    module-host CLI tests;
  - all 24 stages passed, including every database/API/provider/backup/restore
    stage plus formatting, Clippy, Rust tests/docs, compose validation,
    shell/pipeline/secret/hook/whitespace checks, cartridge/renderer/SDK/provider
    clean-room/package proofs, marketplace drills, and server-module proofs;
  - the successful pre-completion receipt matched
    `f97491a669eeb48cdd6c4433975a89c8a4152b7ff08da04ce688f1f1ee64322e`.
    The temporary container, network, volume, namespace, veth pair, forwarder,
    and override were removed afterward; the unrelated system PostgreSQL and
    live Usurper process were not stopped or changed.
  - the first post-archive wrapper attempt omitted `CARGO_NET_OFFLINE` and the
    isolated compose file/project environment. Provider packaging therefore
    attempted unreachable registry access and later scripts selected the
    default compose project. The non-receipt run was stopped; its newly created
    default disposable container, network, and volume were removed. No product
    source or host PostgreSQL state changed;
  - the corrected post-archive gate used the same isolated database with exact
    offline, compose-project, user-runtime, D-Bus, and Wayland bindings. All 24
    stages passed (`GATE GREEN [diff]`), and the final receipt/current gated
    state match at
    `84d45886eafd6815d64614fd84edf31a34ca93c171743585a4b2c39028e6aca5`.
    The isolated database volume, container, network, namespace, veth pair,
    forwarder, and override were then removed; their disposable test data is
    not recoverable.
- Skips or pre-existing failures:
  - none in the isolated green rerun. The earlier seven failures remain
    recorded as an environmental port/role collision, not a Level 18 or
    trusted-QML assertion failure;
  - user visual confirmation of the refreshed workspace-8 row-height build is
    still pending, while structural and real-input validation are green.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS: authenticated source readback, exact records 170–179,
    provenance, fixed-data tests, and compatibility docs agree;
  - REQ-002 PASS: strict v23 state/schema tests reject old, malformed,
    out-of-band, and internally inconsistent state without mutation or RNG
    advance;
  - REQ-003 PASS: levels 1–18 switch draw-free with no retained monster, while
    0, 19, and maximum inputs reject unchanged;
  - REQ-004 PASS: exact `Random(180)` rejection traces, record-170 exclusion,
    records 171–179, 22 strength/11 defence/66 HP, 232-draw valid progression,
    quantified tail risk, and maximum serialized-state proof pass;
  - REQ-005 PASS: Level 18 retreat plus attack, potion, spell, class-special,
    poison, death, reward, replay, restart, and complete-day regressions pass;
  - REQ-006 PASS: fixed provider mapping, signed `option_r`, restart corpus,
    exactly one Level 18 action, no Level 19, non-overlapping 20-to-21 delegate
    replacement, one real Return activation per control, and workspace-8 live
    revisions pass. No excluded platform rules, migrations, shared realm,
    packaging, admission, deployment, or publication entered the slice.
- Docs: reconciled the hand architecture plus generated OpenWiki quickstart and
  Game Cartridges pages through Ticket 072, rules v23, Level 18, `option_r`,
  and explicit loaded-item row geometry. OpenWiki run
  `5fb50ba7-45f9-410d-8253-c774253f1f9c` completed the prose update; after the
  architecture/ticket archive, reconciliation run
  `94ef6479-b0b0-4126-bede-885feab13d58` completed against the durable paths.
  Both retained the existing unrelated unresolved-claims-debt warnings rather
  than rewriting either page's Claims sidecar.
- AAR: submitted `AAR-072` as effective and registered the implicit Loader-row
  and test-database-port failures with reusable geometry and fixture-preflight
  prevention rules.
- Archive: closed Ticket 072 and archived the sole active spec/notes pair after
  the final OpenWiki reconciliation. Delivery remains unauthorized: no commit,
  push, pull request, production package, registration, admission, deployment,
  or publication was performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The stale workspace-8 v22 window's buttons were inert. | The old QML engine retained delegates created with the pre-fix one-time disabled-state assignment. | Replaced the stale process with the current v23 build and retained the dynamic `Qt.binding`. | Require a fresh process and real input after any QML lifecycle fix; do not infer live code replacement from the worktree. |
| 2 | Most controls appeared twice in the live window even though provider IDs/actions and settled offscreen frames were unique. | Asynchronous loader delegates had no explicit row height until their loaded item settled, leaving a real-window plan-replacement geometry gap outside the object-count assertion. | Bind every loader row height to its loaded item and require positive, non-overlapping geometry for the 21-control replacement. | For every new largest action surface, prove provider uniqueness, explicit row geometry, enabled-state propagation, exactly one real key action per control, a fresh-process render, and live compositor input. |
| 3 | The first diff gate's seven database stages failed while all other checks passed. | The host's enabled system PostgreSQL owns port 5432, preventing the compose fixture from binding; stages that reached the system cluster found no test role. | Reran the unchanged gate inside a temporary network namespace that forwarded its isolated loopback 5432 to a disposable compose database on host-veth port 55432; all 24 stages passed, then every temporary resource was removed. | Preflight the required fixture port and database identity before starting the receipt gate; never treat a reachable unrelated PostgreSQL cluster as the test fixture. |
