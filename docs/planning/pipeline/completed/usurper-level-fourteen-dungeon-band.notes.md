---
title: Usurper Level Fourteen Dungeon Band — notes
pipeline_id: c469bdaa-0e18-4fe2-bedc-01b8d44b3832
---

# Usurper Level Fourteen Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 067 supplies exact rules/state/cartridge v18 through Level 13 plus
    matching OpenWiki/gate evidence and the actual-delegate replacement test;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-130 draw to remain visible in deterministic traces;
  - `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001`,
    `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`, and
    `PR-omarchy-gaming-system-count-instantiated-delegates-across-plan-replacement-001`
    keep the fourteenth control unique and inside the actual Qt input boundary.
- Source preflight:
  - authenticated source Git and archive copies of `EDMONST.PAS` remain
    byte-identical at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - lines 3920–4018 define Level 14 records 130–139 as Gooie, Dark Phantom,
    Scorpion, Mutant Dwarf, Flying Vaporizer, Black Mage, Dark Knight, Thief,
    Wood Elf, and Black Scorpion, all at base strength 20 with exact equipment
    flags;
  - authenticated Git/archive `DUNGEONC.PAS` copies match at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`;
    lines 924–955 keep events separate, spend a fight, and repeat
    `Random(level*10)` until the result exceeds `(level-1)*10`; Level 14
    therefore normally selects records 131–139 and preserves record 130 only
    as source data;
  - the unregistered guard applies only when dungeon level is greater than 89,
    so Level 14 remains on the ordinary branch;
  - authenticated Git/archive `PLVSMON.PAS` copies match at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`;
    lines 68–98 use `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initialize monster HP to strength times three.
- Existing boundary fit:
  - the provider/rules reducer is generic through the implemented maximum;
  - the dungeon screen occupies `option_a` through `option_m`, so Level 14
    needs one new bounded external `option_n` field across `GameView`, schema,
    fixtures, and signed binding without changing the platform renderer;
  - Ticket 065's real-input suite and Ticket 067's recursive delegate-count
    assertions are game-neutral and will rerun unchanged.
- The information-only rebuild bulletin was acknowledged and
  `docs/planning/REBUILD_HANDOFF.md` read. Pipeline tools report CodeGraph
  1.5.0 and OpenWiki 0.3.3 ready; no Docker service is active.
- Baseline `env -u CARGO_TARGET_DIR CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` passed: formatting, strict Clippy, 96 Rust tests, rustdoc,
  authenticated upstream/provenance checks, six real-input QML cases,
  seventeen unique signed screens, and provider-backed local play.
- Decision: implement Level 14 as the next normal dungeon band and defer Level
  15, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, random draws, monster data, and provider projections;
  - the local provider validates a revision-bound signed action, maps the
    bounded fixed Level 14 action to the existing `EnterDungeon` command, and
    asks the pure reducer for the next state and view;
  - the signed inert cartridge binds the new `option_n` projection to one
    declared button node; the platform renderer authenticates and lowers each
    declared node once, and trusted QML dispatches the unconfirmed action back
    to the provider without acquiring gameplay authority;
  - state flow remains `signed button -> local-play revision check -> provider
    action mapping -> pure reducer -> v19 state/view -> authenticated render
    plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` validates the authenticated presentation and each
    declared action, while `lower_node` resolves one bounded string binding
    and returns one `RenderedNode::Button` for a successful button node;
  - the blast radius identifies the renderer integration suite, client
    cartridge runtime, preview binary, and provider-game boundary as the
    relevant platform consumers; no Level 14-specific platform rule is
    required;
  - QML and the separate external repository are outside this platform index,
    so `TrustedCartridgeSurface.qml`, the recursive delegate test, local-play
    QML, external Rust/JSON/scripts, and authenticated Pascal source were
    inspected directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `c469bdaa-0e18-4fe2-bedc-01b8d44b3832` at gated state
    `28a1180d4d5d028186f81b5bccc4f8d7cc270f13ecc1b30fdaf3b59ca5e888cd`.
- Exact implementation manifest, with one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_n` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 130–139,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v19, extend validation/switching/labels through Level 14, and add
    encounter, retreat, deterministic, and hostile-state evidence;
  - external `crates/usurper-provider/src/lib.rs`: map the fixed Level 14
    action and prove equivalence, projection, encounter, replay, and live
    profile behavior;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare one
    Level 14 action/button, and require the bounded `option_n` field;
  - all seventeen external `fixtures/presentation/*.json` files: provide the
    exact new required field, with a non-empty Level 14 label only on the
    dungeon view and Level 14 encounter facts on the combat view;
  - external `provenance/source-trace.json`: register the reviewed Level 14
    source records and existing selection/HP/retreat branches;
  - external `scripts/test-provider.sh`, `scripts/test.sh`, and
    `scripts/play.sh`: expect v19/Level 14, run the live profile twice across
    restart, and end smoke play in a Level 14 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the newly implemented normal band and
    unchanged exclusions;
  - platform
    `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`: update the
    game-neutral large-screen regression from 10-to-11 to 16-to-17 actual
    delegates, matching the new dungeon surface cardinality;
  - platform `docs/architecture/game-cartridges.md`: reconcile the durable
    external-development boundary through rules v19/Level 14 during Phase 5;
    no platform production renderer, server, migration, Cargo, or client
    source is required.
- Database and migration consequences: none. Provider-owned state remains
  serialized inside the external adapter, and this slice adds no platform
  persistence, tables, columns, data migration, or PostgreSQL write path.
- API and compatibility contract:
  - state JSON remains strict and deny-unknown-fields, but exact
    `schema_version: 19` replaces v18; v18 and malformed v19 state fail before
    RNG construction or mutation;
  - the view schema adds required string `option_n` with the existing
    64-character bound; all screens supply it, and only the dungeon screen
    binds it to `enter_dungeon_level_14`;
  - the signed manifest advances `rules_version` and `cartridge_version`
    together to 19; SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_14` accepts an empty payload only and maps to the
    existing typed command. Levels 0, 15, and `u16::MAX` remain rejected
    without a revision or RNG advance.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_FOURTEEN_MONSTERS` arrays, source-trace validation, authenticated-source hash/readback, compatibility and port-map review |
  | REQ-002 | v19 identity checks; old/missing/unknown JSON fields; Level 15, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | one sequential levels 1–14 switch test with unchanged RNG/empty traces, visible labels, ascent/descent/remain behavior, and rejected 0/15/max inputs |
  | REQ-004 | forced rejected `Random(140)` draw followed by 131–139, exact 20 strength/10 defence/60 HP, fight decrement, and deterministic twin equality |
  | REQ-005 | exact failed-retreat `(2, 1), (140, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_n`, live Level 14 profile twice across restart, signed-screen/action uniqueness, 16-to-17 Qt delegate replacement, local-play action confirmation, and workspace-8 visual/readback audit |
- Risks and controls:
  - security/input: strict schemas, identifier checks, empty payloads,
    authenticated cartridge content, loopback capability, and revision/screen
    binding continue to reject undeclared or stale actions;
  - privacy/secrets: no account/persona identifiers or reusable credentials
    cross the local provider boundary; generated capabilities/private keys
    remain temporary and must not be logged or committed;
  - state/concurrency: the provider serializes revision-aware actions and the
    pure reducer validates before constructing RNG, so rejected or stale
    actions cannot partially advance state;
  - reconnect/restart: the deterministic provider corpus runs twice around a
    fresh process and compares exact output; provider-owned sessions never
    fall back to platform rules;
  - rendering: the new 17-button dungeon plan is the largest current trusted
    action surface. Recursive instantiated-delegate counts before and after a
    16-to-17 replacement plus real Return input guard against the previously
    reported duplicate/inert-control symptom;
  - rollback: v19 artifacts can be removed before delivery without data
    migration. Published rollback remains out of scope and no delivery action
    is authorized.
- Decisions and rejected alternatives:
  - preserve record 130 as source data but not a normal encounter; directly
    choosing 130 would contradict the reviewed rejection loop;
  - extend `GameView` with `option_n`; reusing a primary/secondary field would
    overload its phase meaning, while a grid or platform renderer change would
    expand the architecture unnecessarily;
  - reuse the generic dungeon/combat reducer; duplicating Level 14 logic in the
    provider or QML would create a second rules authority;
  - keep dungeon events and registration behavior excluded; neither is needed
    to prove the ordinary Level 14 band.

## Phase 3 — Implement

- Implemented external rules/state/cartridge v19 with exact Level 14 records
  130–139, `option_n`, level bounds, fixed provider action, Level 14 signed
  presentation, fixtures, source trace, live paths, and compatibility docs.
- Added deterministic Level 14 evidence for source-order data, forced rejected
  `Random(140)` work, normal records 131–139, boundary-record exclusion,
  20-strength/10-defence/60-HP construction, failed-retreat bound/damage,
  generic/fixed provider equivalence, replay, live restart profile, and strict
  v19 hostile state.
- Updated the platform's game-neutral actual-delegate replacement regression
  from 10-to-11 controls to the current 16-to-17 dungeon-size transition. This
  is test-only; no platform production renderer or gameplay source changed.
- Extended `scripts/test-cartridge.sh` beyond the Phase 2 manifest to assert
  exact v19 manifest identities, the required bounded `option_n` schema field,
  one exact Level 14 signed action/button, and all seventeen fixture fields.
  The additional test evidence is within the approved acceptance scope.
- Focused implementation checks passed:
  - `cargo test -p usurper-data -p usurper-rules -p usurper-provider
    --all-features`: 16 data, 26 provider, 3 local-play binary, 55 rules, and 1
    integration test passed (101 total);
  - strict focused Clippy passed with warnings denied;
  - Qt offscreen trusted-control suite passed all six cases, including pointer,
    single Return activation, delegate removal, and 16-to-17 replacement;
  - signed seventeen-screen cartridge/conformance and provider-backed local
    HTTP plus trusted-QML smoke passed.
- The first signed-cartridge run failed with
  `invalid_cartridge_presentation`: the Level 14 button existed but its exact
  zero-payload action was absent from the presentation action registry. Added
  the missing action declaration; the focused verifier and complete cartridge
  suite then passed. No verifier or gate was weakened.
- An initial direct QML runner command used an unreliable executable lookup
  expression and exited before running tests; the canonical qmake-derived
  executable then ran and passed. This was a harness invocation error, not a
  product or test failure.
- Phase 3 exit: the external v19 Level 14 slice and focused evidence are ready
  for skeptical inspection.

## Phase 3.5 — Inspect

- CodeGraph post-implementation inspection re-read the production
  `compile_render_plan`/`lower_node` path and issued the worktree-bound inspect
  receipt for pipeline `c469bdaa-0e18-4fe2-bedc-01b8d44b3832` at gated state
  `241333014f7adc3444da69d6af29cc422ef314e29b5b281140f65695597a938f`.
- Correctness/EARS review found the exact Level 14 data, v19 identities,
  level bounds, forced selection trace, HP/defence derivation, retreat trace,
  provider action/projection, fixture/schema parity, and source-trace entry
  aligned. Rejected levels validate before RNG construction; draw-free level
  switching, fight expenditure, and revision checks preserve state ownership.
- Security/privacy review found only a declared fixed empty-payload action
  crosses the signed cartridge boundary. No identity, reusable credential,
  account data, network protocol, database, migration, or platform-gameplay
  authority was added. The local capability remains ephemeral and was not
  printed or persisted in the repository.
- Rendering/simplicity review found one generic reducer path, one presentation
  node for Level 14, and one trusted QML loader per accepted node. Node IDs and
  action IDs are globally unique, every button action is declared with an
  exact empty payload, all seventeen fixtures match the strict view schema,
  only the dungeon fixture populates `option_n`, and no Level 15 action exists.
- Findings ledger:
  - confirmed/fixed: the initial Level 14 button omitted its presentation
    action-registry declaration, causing the signed cartridge to be rejected;
    the exact zero-payload declaration was added and the full cartridge suite
    passed;
  - rejected as an architecture drift: no platform Level 14 rule or renderer
    branch is needed; the indexed production compiler already lowers one
    authenticated button node to one rendered node;
  - no duplicate or stale delegate was found by the six-case Qt suite,
    including recursive removal/count checks over a 16-to-17 replacement.
- During inspection the user reported that most buttons in the existing v18
  workspace-8 preview appeared twice. Readback found exactly one mapped QML
  window, a live plan with nineteen unique node IDs and sixteen unique button
  IDs/labels/actions, and no duplicate plan data. An isolated full-height QML
  render of that exact live plan instantiated nineteen nodes and visibly
  rendered each control once. The desktop capture itself was obscured by the
  password overlay, so Phase 4 must replace the stale v18 process with a fresh
  v19 process and repeat both programmatic cardinality and visible capture;
  the report is not dismissed merely because the isolated reproduction is
  clean.
- No new durable prevention rule is proposed. The existing one-phase-valid
  command rule directly exposed the missing action declaration, while the
  existing real-input and recursive-delegate rules already express the live
  symptom's required safeguards.
- Phase 3.5 exit: all confirmed implementation findings are fixed. Fresh v19
  provider, gate, and workspace-8 validation remain mandatory.

## Phase 4 — Validate

- Full external workspace validation passed with
  `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh`: strict formatting/Clippy/rustdoc, 101 Rust tests, exact
  upstream/provenance checks, six trusted-control QML cases, seventeen signed
  screens, and provider-backed local HTTP/QML smoke all passed.
- The isolated PostgreSQL conformance run passed the fixed fifteen-case
  TLS/replay/fault/callback corpus twice across a provider process restart.
  It used a ticket-scoped Compose project and Unix-socket proxy so the
  unrelated host PostgreSQL service was neither stopped nor reused.
- `bin/gate.sh --diff` completed all twenty-four platform stages and printed
  `GATE GREEN [diff]`. The gate receipt matches gated worktree state
  `241333014f7adc3444da69d6af29cc422ef314e29b5b281140f65695597a938f`,
  including the game cartridge contract, trusted renderer/QML input suite,
  reproducible SDK/client packaging, PostgreSQL/API/QML smoke, remote-provider
  security/restart checks, and server-module isolation.
- Fresh workspace-8 play replaced the old v18 process without changing the
  user's active workspace 1. Readback found exactly one mapped development
  QML window running cartridge v19. Targeted real Tab/Return input advanced
  exactly one revision per selection through entry, Human, Alchemist, Main
  Street, dungeon, Level 14, and Look:
  - the Level 14 dungeon state at revision 5 contained twenty unique nodes,
    seventeen unique button IDs/labels, exactly one Level 14 control, and the
    narrative `You descend to dungeon level 14.`;
  - the resulting revision-6 combat state contained nine unique nodes, five
    unique buttons, nine fights remaining, and a source-correct Level 14 Wood
    Elf at 60/60 HP;
  - the fresh v19 process remains open on workspace 8 at that encounter.
- The desktop's password overlay continued to obscure compositor screenshots,
  so no claim is made that the actual on-screen pixels were captured after the
  user's duplicate-button report. However, the exact v18 live plan rendered
  once per control in an isolated full-height QML capture, the fresh v19 live
  plan contains unique controls, real live input emits one revision per key,
  the recursive 16-to-17 delegate regression passes, and only one QML window
  is mapped. The fresh v19 window is left available for direct user visual
  confirmation after unlock.
- `git diff --check` passed in both the platform and external Usurper
  repositories. No commit, push, admission, deployment, or publication was
  performed.
- Phase 4 exit: all automated, restart, gate, and live-input requirements pass
  with a matching receipt; completion documentation and archive remain.

## Phase 5 — Complete

- Acceptance audit:
  - REQ-001 satisfied by authenticated source hash/readback, exact data/source
    trace tests, and compatibility/port-map documentation;
  - REQ-002 satisfied by exact v19 state identity, Level 14 consistency, and
    hostile old/missing/unknown/wrong-level/wrong-record/oversized cases that
    preserve state and RNG;
  - REQ-003 satisfied by the sequential draw-free levels 1–14 switch test and
    unchanged rejection of 0, 15, and `u16::MAX`;
  - REQ-004 satisfied by the forced rejected-130 trace, accepted records
    131–139, exact 20 strength/10 defence/60 HP, fight decrement, and
    deterministic twins;
  - REQ-005 satisfied by exact failed-retreat `(2, 1), (140, 10)` evidence and
    the complete attack/potion/spell/special/poison/death/reward/day regression
    suite;
  - REQ-006 satisfied by fixed/generic provider equivalence, one declared
    Level 14 action, unique signed controls, 16-to-17 actual-delegate proof,
    provider restart conformance, real workspace-8 Tab/Return revisions, and
    the explicit absence of Level 15, shared realm, platform gameplay,
    packaging, admission, deployment, and publication work.
- Hand-maintained `docs/architecture/game-cartridges.md` now records rules v19,
  levels 1–14, boundary record 130, normal records 131–139, `option_n`, and the
  16-to-17 QML replacement while preserving the development-only authority
  boundary.
- The required OpenWiki update lifecycle completed. `openwiki/quickstart.md`
  and `openwiki/game-cartridges.md` now carry the same v19/Level 14 and QML
  evidence, and the completion receipt matches gated state
  `1bc66a1cf5983a1f8b5b95a275d62565e12ab709ff7fd94749cf4d5cfcf72956`.
  OpenWiki warned that those large pages retain pre-existing unresolved Claims
  evidence debt; it still returned `status: complete`, and no Claims sidecar was
  edited manually.
- `AAR-068` was submitted as effective. No new `BF-*`, `PR-*`, or `AD-*` was
  created because the missing declaration and reported live symptom are already
  covered by recalled validation and QML lifecycle rules.
- The final post-documentation `bin/gate.sh --diff` passed all twenty-four
  stages and printed `GATE GREEN [diff]`. Its receipt matches completed
  OpenWiki/gated state
  `1bc66a1cf5983a1f8b5b95a275d62565e12ab709ff7fd94749cf4d5cfcf72956`.
- Archive: Ticket 068 is closed and this single spec/notes pair is moved to
  `pipeline/completed` with no remaining active pair.
- Phase 5 exit: pipeline complete. Delivery remains unauthorized, so no
  commit, push, pull request, registration, admission, deployment, or
  publication was performed.
