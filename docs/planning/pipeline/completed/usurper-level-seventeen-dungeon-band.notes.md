---
title: Usurper Level Seventeen Dungeon Band — notes
pipeline_id: 9719bac4-c792-49d8-844c-009cf24da181
---

# Usurper Level Seventeen Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 070 supplies exact rules/state/cartridge v21 through Level 16 plus
    matching OpenWiki/gate evidence, fresh workspace-8 play, and the exact
    18-to-19 instantiated-delegate replacement test;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-160 draw to remain visible in deterministic traces;
  - `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001`,
    `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`, and
    `PR-omarchy-gaming-system-count-instantiated-delegates-across-plan-replacement-001`
    keep the seventeenth choice unique and inside the actual Qt input boundary;
  - `PR-omarchy-gaming-system-lock-provider-corpus-to-tested-phase-transitions-001`
    requires any Level 17 conformance sequence to assert its post-combat phase
    before selecting the next fixed command.
- Source preflight:
  - authenticated Git/archive copies of `EDMONST.PAS` remain byte-identical at
    SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - lines 4222–4321 define Level 17 records 160–169 as Orc Scoundrel, Giant
    Ant, Rabid hound, Red Witch, Club Champion, Mad Woman, Scum of the Earth,
    Red Wizard, Soldier of Fortune, and Zuul Mercenary, all at base strength
    21 with exact equipment flags;
  - authenticated Git/archive copies of `DUNGEONC.PAS` match at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`;
    lines 924–955 keep events separate, spend a fight, and repeat
    `Random(level*10)` until the result exceeds `(level-1)*10`, so Level 17
    normally selects records 161–169 and stores record 160 only as source data;
  - the unregistered guard applies only above dungeon level 89, so Level 17
    remains on the ordinary branch;
  - authenticated Git/archive copies of `PLVSMON.PAS` match at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`;
    lines 68–98 use `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initialize monster HP to strength times three.
- Existing boundary fit:
  - the rules and provider reducers are generic through their implemented
    maximum; the current bound and data tables stop at Level 16;
  - the dungeon view occupies bounded option fields A–P, so Level 17 needs one
    new external projection field without a platform renderer change;
  - the real-input suite and recursive delegate-count assertion are
    game-neutral and can be ratcheted from nineteen to twenty controls.
- The information-only rebuild bulletin and handoff were reviewed. Pipeline
  tools report CodeGraph 1.5.0 and OpenWiki 0.3.3 ready; no Docker service is
  active.
- Baseline `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` passed: formatting, strict Clippy, 111 Rust tests,
  rustdoc, authenticated upstream/provenance checks, six real-input QML cases,
  seventeen unique signed screens, and provider-backed local play.
- Decision: implement Level 17 as the next normal dungeon band and defer Level
  18, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, RNG draws, monster data, and provider projections;
  - the provider validates a revision-bound action, maps fixed Level 17 to the
    existing typed `EnterDungeon` command, and asks the pure reducer for the
    next v22 state and view;
  - the signed inert cartridge binds `option_q` to one declared button. The
    platform authenticates its schema and action, lowers it into one
    `RenderedNode::Button`, and trusted QML dispatches only an unconfirmed
    request;
  - state flow remains `signed button -> local revision/screen check ->
    provider action mapping -> pure reducer -> v22 state/view -> authenticated
    render plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` validates the authenticated view schema and declared
    action set before lowering each signed node once; `RenderedNode::Button`
    carries one ID, label, action, and accessible label;
  - the Core profile admits up to 256 nodes, so a twenty-button dungeon screen
    requires no production platform extension;
  - the blast radius includes the client cartridge runtime, preview CLI,
    renderer tests, cartridge contracts, and platform session/distribution
    consumers; none depends on Usurper option letters or dungeon levels;
  - the separate Usurper repository has no CodeGraph index and QML is not in
    the Rust AST graph, so its Rust/JSON/shell surfaces and actual delegate
    tree are reviewed directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `9719bac4-c792-49d8-844c-009cf24da181` at gated state
    `7613126f3b142929e2778f484827bcc001b338277a8ad9182a6ff656b46408f7`.
- Exact implementation manifest, one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_q` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 160–169,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v22, extend validation/switching/labels through Level 17, and add exact
    encounter, retreat, deterministic, and hostile-state evidence;
  - external `crates/usurper-provider/src/lib.rs`: map fixed Level 17 and prove
    generic/fixed equivalence, projection, encounter, replay, and a focused
    post-combat phase sequence;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare
    one Level 17 action/button, and require bounded `option_q`;
  - all seventeen external `fixtures/presentation/*.json`: supply the required
    field, with non-empty Level 17 text only on the dungeon view and
    source-valid Level 17 facts on the combat fixture;
  - external `provenance/source-trace.json`: register reviewed Level 17 editor,
    selection, HP, and retreat branches;
  - external `scripts/test-cartridge.sh`, `scripts/test-provider.sh`,
    `scripts/test.sh`, and `scripts/play.sh`: assert exact v22/Level 17
    identities and uniqueness, lock the provider corpus to its tested
    post-combat phase, and end smoke play in a Level 17 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the implemented band and remaining
    visible limits;
  - platform
    `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`: ratchet
    the game-neutral large-screen proof from 19 to 20 actual delegates;
  - platform `docs/architecture/game-cartridges.md`: reconcile durable
    external-development facts through v22/Level 17 during Phase 5. No
    platform production renderer, server, migration, Cargo, or client source
    change is required.
- Database and migration consequences: none. Provider-owned state stays in the
  external adapter; this slice adds no platform persistence, table, column,
  migration, or PostgreSQL write path.
- API and compatibility contract:
  - strict state JSON requires exact `schema_version: 22`; v21 and malformed
    v22 state fail before RNG construction or mutation;
  - the view schema adds required string `option_q` with the existing
    64-character bound. All screens provide it; only the dungeon screen binds
    it to `enter_dungeon_level_17`;
  - the signed manifest advances `rules_version` and `cartridge_version`
    together to 22; SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_17` accepts an empty payload and maps to the existing
    typed command. Levels 0, 18, and `u16::MAX` remain rejected without
    revision or RNG advance.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_SEVENTEEN_MONSTERS` arrays, source-trace validation, authenticated-source hashes/readback, compatibility and port-map review |
  | REQ-002 | v22 identity checks; old/missing/unknown JSON fields; Level 18, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | sequential levels 1–17 switching with unchanged RNG/empty traces, complete visible labels, ascent/descent/remain behavior, and rejected 0/18/max inputs |
  | REQ-004 | forced rejected `Random(170)` draw followed by 161–169, exact 21 strength/10 defence/63 HP, fight decrement, and deterministic twin equality |
  | REQ-005 | exact failed-retreat `(2, 1), (170, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_q`, focused phase-aware live profile and restart corpus, signed-screen/action uniqueness, 19-to-20 Qt delegate replacement, local-play confirmation, and workspace-8 runtime audit |
- Risks and controls:
  - security/input: strict schemas, identifier checks, empty payloads,
    authenticated cartridge content, loopback capability, and revision/screen
    binding reject undeclared or stale actions;
  - privacy/secrets: no platform identity or reusable credential enters game
    state; local capabilities and signing keys remain temporary and unlogged;
  - state/concurrency: provider serialization and pre-RNG validation prevent
    rejected or stale commands from partially advancing state;
  - reconnect/restart: the fixed provider corpus runs twice across process
    restart, and a focused Level 17 test must assert the actual phase/state
    before the next command is encoded;
  - rendering: the twenty-button dungeon plan is the new largest action
    surface. Recursive 19-to-20 instantiated-delegate counts, signed-plan
    uniqueness, and real pointer/Return input cover duplicate or inert
    controls;
  - rollback: v22 artifacts can be removed before delivery without migration;
    no publication or delivery action is authorized.
- Decisions and rejected alternatives:
  - preserve record 160 as canonical data but never select it normally;
    direct selection would contradict the reviewed rejection loop;
  - add bounded `option_q`; overloading primary/secondary fields would blur
    phase semantics, while a grid or renderer change would unnecessarily
    expand platform behavior;
  - reuse the generic reducer; copying Level 17 rules into provider or QML
    would create a second authority;
  - keep events and registration paths excluded because neither is required
    for the ordinary Level 17 band.
- Phase 2 exit: the source, ownership, compatibility break, exact file
  manifest, regression evidence, and operational risks are fully specified.

## Phase 3 — Implement

- Implemented the Phase 2 manifest without widening platform production
  ownership or persistence:
  - external model/view now includes bounded `option_q`;
  - external data preserves exact editor records 160–169 and lookup routing;
  - external rules use exact v22 state, allow levels 1–17, preserve rejected
    record-160 draws, initialize strength-21 monsters at 63 HP, and retain the
    Level 17 retreat bound;
  - provider maps the fixed Level 17 action through the generic reducer and
    projects the seventeenth label;
  - signed manifest/presentation/schema, seventeen screen fixtures,
    provenance, scripts, and compatibility docs now describe v22/Level 17;
  - platform QML replacement coverage expects nineteen old and twenty new
    recursively instantiated button delegates.
- During focused provider testing, the deterministic Gnoll/Cleric Level 17
  sequence reached `dead` after spell then retreat, not `dungeon`. This was a
  useful phase-contract failure rather than a reducer defect: Level 17's
  source-defined damage bound is larger than Level 16's. The Level 17 test and
  restart corpus now assert death and issue `reenter`; a narrowly scoped
  regression confirms Level 16 still survives and returns with
  `main_street`.
- Implementation checks:
  - `cargo fmt --all -- --check`: PASS;
  - focused Level 16 provider survivor test: PASS;
  - focused Level 17 provider equivalence/live-profile tests: 2 PASS;
  - `jq empty` over manifest, presentation, schema, fixtures, and provenance:
    PASS;
  - `bash -n scripts/*.sh` and `git diff --check`: PASS;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    ./scripts/test-cartridge.sh`: PASS, including six actual QML input/delegate
    cases and signed seventeen-screen cartridge checks;
  - `cargo test --workspace --all-targets --all-features`: PASS, 116 tests;
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
    PASS.
- Completeness searches found only the intentionally rejected v21 state
  fixture; Level 18 remains absent from declared actions and production
  bounds. All seventeen presentation-screen fixtures contain `option_q`, with
  a value only on the dungeon fixture.
- Phase 3 exit: implementation and focused evidence satisfy the designed
  manifest; the worktree is ready for independent correctness, architecture,
  security, test-quality, and scope inspection.

## Phase 3.5 — Inspect

- Security preflight was ready against external-repository base
  `bb31caa122de669d72a265860b19969fcd28505f`. The TAC connector was not
  available, so the inspection used local evidence and explicitly records no
  TAC-derived context.
- Codex Security scan `40b068d7-d7aa-459c-b161-d2c46dfd5eb8` froze workbench
  digest
  `codex-security-snapshot/v1:sha256:4649bc5a6f775502c5d277e40383b4af463e5f215c0d455d852c109caa2a7c1a`,
  reviewed all 17 source-like changed files across three discovery shards plus
  parent reconciliation, verified 29 threat-model citations, and completed
  with zero reportable findings and no deferred coverage. Its sealed report is
  `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260903T073303Z_hehts9sr/report.md`.
- Candidate dispositions:
  - a same-host process can read the ephemeral local-play capability from QML
    argv, validated with a disposable non-secret marker. This is a real local
    observation but not reportable under the repository policy because the
    loopback session is developer-owned, ephemeral, and carries no durable or
    sensitive authority;
  - a bearer can retain render generations until local-play exits, reproduced
    through the authenticated HTTP surface as 44 directories/107,887 bytes
    after 43 phase-valid actions. This is also policy-excluded developer-local
    resource use and exit cleanup remains in place;
  - the former 64-draw trace bound could exhaust on a valid deterministic Level
    17 rejection run. State 6 reproduced 64 rejected draws and identical retry
    failure. It has no attacker-controlled cross-boundary path and was
    suppressed as a security finding, but was promoted to a correctness
    release blocker and fixed before validation.
- The correctness fix raises the still-bounded command trace from 64 to 256
  draws. A new regression proves the state-6 sequence reaches monster record
  168 on draw 82 after 81 rejected values, and another constructs the largest
  relevant 256-draw/provider-view state and proves it remains inside the
  Provider Starter's 32 KiB state limit. The probability of a Level 17 trace
  exhausting the bound falls from approximately 3.08% to `8.96e-7`; future
  much deeper levels must revisit trace compaction instead of widening the
  bound indefinitely.
- The sealed security snapshot predates that correction. Targeted post-scan
  review found its only production delta is the constant `64 -> 256`; it
  widens one bounded vector but changes no network, authentication, secret,
  action, or ownership boundary, and the maximum-state regression covers the
  serialization/resource consequence. The remaining post-scan changes are the
  two regressions and compatibility wording.
- Final CodeGraph inspection reconfirmed that `compile_render_plan` validates
  the signed declared action before lowering each node exactly once through
  `lower_node`; Core admits the twenty nodes, and runtime, preview, renderer,
  and session consumers require no platform production change. QML remains
  outside the Rust AST graph and is covered directly by its recursive
  delegate-count and actual-input tests.
- Findings ledger:
  - correctness: the trace-exhaustion blocker was fixed and focused tests pass;
  - architecture/scope: clean; rules/state remain external and the platform
    production renderer is unchanged;
  - security/privacy: no reportable finding, no durable secret or identity
    added, and both local-only observations are documented above;
  - state/concurrency: strict pre-RNG validation and revision binding remain
    intact;
  - test quality/rendering: signed action uniqueness and recursive nineteen to
    twenty delegate replacement pass; workspace-8 live input remains a Phase 4
    release gate;
  - reconnect/performance: restart corpus and full provider validation remain
    Phase 4 gates; local-only render retention is accepted for this
    developer-runner scope.
- Post-inspection `git diff --check`, `cargo fmt --all -- --check`, ShellCheck,
  exact numeric v22 manifest checks, exact one-action Level 17 cardinality,
  and no-Level-18 searches pass.
- Phase 3.5 exit: zero reportable security findings and no deferred inspection
  coverage; the only correctness blocker was repaired and directly regressed.

## Phase 4 — Validate

- Full external validation after the trace-bound correction passed with
  `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh`:
  - format, strict Clippy, rustdoc, authenticated upstream archives, provenance,
    JSON/shell contracts, and provider-backed local HTTP smoke passed;
  - all 118 Rust tests passed: 19 data, 33 provider, 3 local-play, 62 rules,
    and 1 complete-day integration test;
  - the signed seventeen-screen cartridge passed, including exactly one Level
    17 action and no Level 18;
  - all six trusted-QML cases passed, including actual Return/pointer input and
    recursive replacement from nineteen old to twenty new delegates with one
    delegate per control.
- Against the ticket-scoped PostgreSQL service on loopback port 55432, the
  production Usurper provider passed the fixed 15-case
  TLS/replay/fault/callback corpus twice across process restart. The private
  credential-file and port-redirect path kept credential content out of argv
  and logs.
- The complete platform `bin/gate.sh --diff` passed every stage through native
  packaging, PostgreSQL/API/QML integration, remote-provider security,
  sidecar/restart, authority pilot, backup/restore, private-alpha admission,
  and server-module isolation/conformance. Receipt and current gated-state hash
  matched at `b63706d47bc53e11270a6eb10ea77361fdb3163c303685cf94c862d47c008cfc`.
- Live provider-backed v22 proof used compositor-delivered keyboard input on
  window `0x55f0134dc0c0`, moved silently to workspace 8 while workspace 1
  remained active:
  - Return advanced entry to Human, then Alchemist; Tab plus Return entered the
    dungeon; seventeen compositor Tab events plus Return selected Level 17;
    Return then started combat;
  - authenticated readback advanced exactly through revisions 0–6 and showed
    20 buttons/20 unique IDs/20 unique actions on both dungeon revisions, with
    one `dungeon_level_seventeen` / `enter_dungeon_level_17` control labeled
    `Dungeon level 17`;
  - combat showed five buttons/five unique IDs/five unique actions and exact
    source-valid text `A level 17 Rabid hound blocks your way.` with monster HP
    `63/63`;
  - the preserved v21 window was closed only after this proof. The verified v22
    combat window remains open on workspace 8 for user inspection.
- The isolated `ogs-ticket071` PostgreSQL container, network, and test volume
  were removed after validation; no shared PostgreSQL service was reused or
  stopped.
- Phase 4 exit: all automated, restart, renderer-cardinality, real-input, live
  visibility, and matching-receipt gates pass with no unresolved blocker.

## Phase 5 — Complete

- Acceptance audit:
  - REQ-001 PASS: authenticated source hashes/readback, exact records 160–169,
    provenance, data tests, and compatibility docs agree;
  - REQ-002 PASS: exact v22 state/schema tests reject old, malformed,
    out-of-band, and inconsistent state before mutation or RNG advance;
  - REQ-003 PASS: levels 1–17 switch draw-free with no monster, while 0, 18,
    and maximum inputs reject unchanged;
  - REQ-004 PASS: exact `Random(170)` rejection traces, boundary record 160
    exclusion, records 161–169, 21 strength/10 defence/63 HP, and the 82-draw
    long-run regression pass;
  - REQ-005 PASS: Level 17 retreat plus attack, potion, spell, class-special,
    poison, death, reward, replay, and complete-day regressions pass;
  - REQ-006 PASS: the fixed provider action, signed `option_q` binding, full
    restart corpus, exactly one Level 17 action, no Level 18, recursive 19-to-20
    delegate proof, actual QML input, and workspace-8 live revisions all pass;
    no excluded platform rule, migration, shared realm, packaging, admission,
    deployment, or publication work entered the slice.
- Updated the platform architecture record and generated OpenWiki quickstart
  and Game Cartridges pages through rules/cartridge v22, Level 17, record 160,
  `option_q`, and the 19-to-20 actual-delegate proof. OpenWiki run
  `96cf3cd8-51e6-4412-9210-42623847da19` returned `status: complete`; it
  retained pre-existing unresolved claims-debt warnings for both pages rather
  than rewriting claims sidecars. The earlier run
  `4bb04cff-60a3-40c1-b79c-a27ab053bff1` is explicitly recorded as
  interrupted after exposing the architecture-ordering issue.
- Submitted `AAR-071` as effective. Added
  `BF-omarchy-gaming-system-usurper-rng-trace-bound-exhaustion-001` and
  `PR-omarchy-gaming-system-size-rejection-traces-against-valid-tail-risk-001`
  to the AAR and knowledge register.
- Closed Ticket 071 and archived the sole active spec/notes pair. Delivery is
  still unauthorized: no commit, push, pull request, production package,
  registration, admission, deployment, or publication was performed.
- Re-ran the complete platform `bin/gate.sh --diff` after the OpenWiki update
  and pipeline archive. Every stage passed again (`GATE GREEN [diff]`), and the
  final receipt/current gated-state hash match at
  `61a39516d74c6ff20c871d76f6e3931c82b515d98bf8c196b65cffe6651d89a5`.
