---
title: Usurper Level Fifteen Dungeon Band — notes
pipeline_id: ad75250b-4fc4-4cd3-80e0-5cfdf006e4f7
---

# Usurper Level Fifteen Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 068 supplies exact rules/state/cartridge v19 through Level 14 plus
    matching OpenWiki/gate evidence, unique live-plan readback, and the
    recursive instantiated-delegate replacement test;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-140 draw to remain visible in deterministic traces;
  - `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001`,
    `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`, and
    `PR-omarchy-gaming-system-count-instantiated-delegates-across-plan-replacement-001`
    keep the fifteenth control unique and inside the actual Qt input boundary.
- Source preflight:
  - authenticated source Git and archive copies of `EDMONST.PAS` remain
    byte-identical at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - lines 4020–4119 define Level 15 records 140–149 as Giant Scorpion, Small
    Ape, Capricorn, Nightmare, Dark Hoodlum, Lurking Grudge, Cobra, Troll
    General, Hellcat, and Leper Woman, all at base strength 20 with exact
    equipment flags;
  - authenticated Git/archive `DUNGEONC.PAS` copies match at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`;
    lines 924–955 keep events separate, spend a fight, and repeat
    `Random(level*10)` until the result exceeds `(level-1)*10`; Level 15
    therefore normally selects records 141–149 and preserves record 140 only
    as source data;
  - the unregistered guard applies only when dungeon level is greater than 89,
    so Level 15 remains on the ordinary branch;
  - authenticated Git/archive `PLVSMON.PAS` copies match at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`;
    lines 68–98 use `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initialize monster HP to strength times three.
- Existing boundary fit:
  - the provider/rules reducer is generic through the implemented maximum;
  - the dungeon screen occupies the first fourteen bounded option fields, so
    Level 15 needs one new external projection field across `GameView`, schema,
    fixtures, and signed binding without changing the platform renderer;
  - the real-input suite and recursive delegate-count assertion remain
    game-neutral and will be ratcheted to the new eighteen-button surface.
- The information-only rebuild bulletin was acknowledged and
  `docs/planning/REBUILD_HANDOFF.md` read. Pipeline tools report CodeGraph
  1.5.0 and OpenWiki 0.3.3 ready; no Docker service is active.
- Baseline `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` passed: formatting, strict Clippy, 101 Rust tests,
  rustdoc, authenticated upstream/provenance checks, six real-input QML cases,
  seventeen unique signed screens, and provider-backed local play.
- Decision: implement Level 15 as the next normal dungeon band and defer Level
  16, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, random draws, monster data, and provider projections;
  - the provider validates a revision-bound signed action, maps the fixed Level
    15 action to the existing typed `EnterDungeon` command, and asks the pure
    reducer for the next state and view;
  - the signed inert cartridge binds `option_o` to one declared button node;
    the platform authenticates the package, validates the view schema and
    declared action, lowers each accepted node once, and trusted QML dispatches
    the unconfirmed action without gaining gameplay authority;
  - state flow remains `signed button -> local revision check -> provider
    action mapping -> pure reducer -> v20 state/view -> authenticated render
    plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` validates the authenticated schema/view and declared
    action set before iterating the signed screen once; `lower_node` returns one
    `RenderedNode::Button` for each accepted button node, and `finish_plan`
    retains the bounded plan;
  - blast radius identifies renderer integration tests, the client cartridge
    runtime, and preview CLI as consumers. None requires Level 15-specific
    platform behavior because bindings are schema-driven strings;
  - QML and the separate external repository are outside this platform index,
    so the recursive trusted-node test, external Rust/JSON/scripts, and
    authenticated Pascal source were inspected directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `ad75250b-4fc4-4cd3-80e0-5cfdf006e4f7` at gated state
    `1bc66a1cf5983a1f8b5b95a275d62565e12ab709ff7fd94749cf4d5cfcf72956`.
- Exact implementation manifest, with one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_o` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 140–149,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v20, extend validation/switching/labels through Level 15, and add
    encounter, retreat, deterministic, and hostile-state evidence;
  - external `crates/usurper-provider/src/lib.rs`: map the fixed Level 15
    action and prove generic/fixed equivalence, projection, encounter, replay,
    and live profile behavior;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare one
    Level 15 action/button, and require bounded `option_o`;
  - all seventeen external `fixtures/presentation/*.json` files: supply the
    exact new required field, with a non-empty Level 15 label only on the
    dungeon view and source-valid Level 15 facts on the combat fixture;
  - external `provenance/source-trace.json`: register the reviewed Level 15
    source records and the existing selection/HP/retreat branches;
  - external `scripts/test-cartridge.sh`, `scripts/test-provider.sh`,
    `scripts/test.sh`, and `scripts/play.sh`: assert exact v20/Level 15
    identities and uniqueness, run the live profile twice across restart, and
    finish smoke play in a Level 15 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the newly implemented band, fix the
    stale Level-14-and-above exclusion, and retain all other limits;
  - platform
    `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`: ratchet
    the game-neutral large-screen regression from 17-to-18 actual delegates;
  - platform `docs/architecture/game-cartridges.md`: reconcile the durable
    external-development boundary through rules v20/Level 15 during Phase 5;
    no platform production renderer, server, migration, Cargo, or client
    source change is required.
- Database and migration consequences: none. Provider-owned state remains
  serialized inside the external adapter, and this slice adds no platform
  persistence, table, column, migration, or PostgreSQL write path.
- API and compatibility contract:
  - state JSON remains strict and deny-unknown-fields, but exact
    `schema_version: 20` replaces v19; v19 and malformed v20 state fail before
    RNG construction or mutation;
  - the view schema adds required string `option_o` with the existing
    64-character bound; all screens provide it, and only the dungeon screen
    binds it to `enter_dungeon_level_15`;
  - the signed manifest advances `rules_version` and `cartridge_version`
    together to 20; SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_15` accepts an empty payload only and maps to the
    existing typed command. Levels 0, 16, and `u16::MAX` remain rejected
    without revision or RNG advance.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_FIFTEEN_MONSTERS` arrays, source-trace validation, authenticated-source hash/readback, compatibility and port-map review |
  | REQ-002 | v20 identity checks; old/missing/unknown JSON fields; Level 16, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | sequential levels 1–15 switch test with unchanged RNG/empty traces, visible labels, ascent/descent/remain behavior, and rejected 0/16/max inputs |
  | REQ-004 | forced rejected `Random(150)` draw followed by 141–149, exact 20 strength/10 defence/60 HP, fight decrement, and deterministic twin equality |
  | REQ-005 | exact failed-retreat `(2, 1), (150, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_o`, live Level 15 profile twice across restart, signed-screen/action uniqueness, 17-to-18 Qt delegate replacement, local-play confirmation, and workspace-8 visual/readback audit |
- Risks and controls:
  - security/input: strict schemas, identifier checks, empty payloads,
    authenticated cartridge content, loopback capability, and revision/screen
    binding continue to reject undeclared or stale actions;
  - privacy/secrets: no account/persona identity or reusable credential enters
    game state; generated local capabilities/private keys remain temporary and
    must not be logged or committed;
  - state/concurrency: the provider serializes revision-aware actions and the
    reducer validates before constructing RNG, so rejected or stale commands
    cannot partially advance state;
  - reconnect/restart: the deterministic provider corpus runs twice around a
    fresh process and compares exact output; provider-owned sessions never
    fail back to platform rules;
  - rendering: the eighteen-button dungeon plan becomes the largest trusted
    action surface. Recursive instantiated-delegate counts across a realistic
    17-to-18 replacement, signed-plan uniqueness, and real pointer/Return
    input cover the reported duplicate/inert-control symptom;
  - rollback: v20 artifacts can be removed before delivery without data
    migration. Published rollback remains out of scope, and no delivery action
    is authorized.
- Decisions and rejected alternatives:
  - preserve record 140 as canonical source data but not a normal encounter;
    selecting it directly would contradict the reviewed rejection loop;
  - add bounded `option_o`; overloading primary/secondary labels would blur
    phase semantics, while a grid or platform renderer change would expand
    architecture without benefit;
  - reuse the generic dungeon/combat reducer; duplicating Level 15 logic in
    provider or QML would introduce a second rules authority;
  - keep events and registration behavior excluded; neither is needed to
    prove the ordinary Level 15 band.

## Phase 3 — Implement

- Implemented external rules/state/cartridge v20 with exact Level 15 records
  140–149, `option_o`, level bounds, one fixed provider action, Level 15 signed
  presentation, fixtures, source trace, live paths, and compatibility docs.
- Added deterministic evidence for source-order Level 15 data, forced rejected
  `Random(150)` work, normal records 141–149, boundary-record exclusion,
  20-strength/10-defence/60-HP construction, failed-retreat bound/damage,
  generic/fixed provider equivalence, replay, live-profile recovery, and strict
  v20 hostile state.
- Ratcheted the platform's game-neutral actual-delegate replacement regression
  from 17-to-18 controls. This is test-only; no platform production renderer,
  rules, server, database, or migration source changed.
- Extended `scripts/test-cartridge.sh` to assert exact v20 identities, bounded
  required `option_o`, one exact signed Level 15 node, and—explicitly learning
  from Ticket 068—one exact empty-payload action-registry declaration. All
  seventeen fixtures carry the field, and only the dungeon view populates it.
- Corrected the external compatibility ledger's stale statement that Level 14
  and above were unimplemented; the v20 ledger now consistently leaves Level
  16 and above out of scope.
- Focused implementation checks passed:
  - `cargo test -p usurper-data -p usurper-rules -p usurper-provider
    --all-features`: 17 data, 28 provider, 3 local-session binary, 57 rules,
    and 1 integration test passed (106 total);
  - strict focused Clippy passed with warnings denied;
  - Qt offscreen trusted-control suite passed all six cases, including pointer,
    single Return activation, stale-delegate removal, and 17-to-18 replacement;
  - signed seventeen-screen cartridge/conformance passed;
  - provider-backed local HTTP plus trusted-QML smoke passed;
  - `git diff --check` passed in both repositories.
- Phase 3 exit: the external v20 Level 15 slice and focused evidence are ready
  for skeptical inspection.

## Phase 3.5 — Inspect

- Completed a terminal Codex Security working-tree review of all seventeen
  source-like files in the cumulative external Usurper change. The sealed
  report is
  `/mnt/fast/tmp/codex-security-scans/omarchygs_usurper/bb31caa_worktree_20260903T050200Z/report.md`;
  coverage is complete, all seventeen full-file receipts are closed, no work
  is deferred, and no reportable finding survived discovery.
- Security inspection covered strict state/command decoding, deterministic
  rejection sampling and arithmetic, fixed provider actions, production
  starter authentication/replay/store boundaries, loopback capabilities,
  asset-token/path constraints, render-before-commit behavior, declarative
  cartridge/schema injection and action ambiguity, generated signing
  material, test credentials, and temporary/process boundaries.
- TAC access could not be verified because its connector was not signed in.
  This affects protected report rendering only; the complete local artifact
  bundle was still generated and sealed.
- Skeptical correctness inspection confirmed:
  - exact source order, strength, defence, equipment flags, normal selection
    range 141–149, stored boundary record 140, three-times-strength HP, and
    Level 15 retreat draw/damage agree with the authenticated v0.20e source;
  - rules validate current v20 state before RNG construction and validate the
    result before release; rejected Level 0/16/max, wrong-band, unknown-record,
    and stale commands cannot advance committed state;
  - fixed `enter_dungeon_level_15` and generic typed commands converge on the
    same reducer, while the signed cartridge declares exactly one matching
    empty-payload action and bounded `option_o` binding;
  - all seventeen fixtures and the strict schema stay in parity, and only the
    dungeon fixture exposes the new choice;
  - the only platform change remains a game-neutral QML test ratchet from
    seventeen to eighteen instantiated controls; no production renderer,
    client, server, database, migration, protocol, or game-rules source
    changed.
- The live v19 provider plan was read without exposing its capability and
  contained five combat buttons with five unique IDs and five unique actions.
  The user's visible duplicate/inert-button report therefore does not
  originate in duplicated provider data. It remains a required live v20 QML
  presentation and real-input check in Phase 4.
- Inspection found and corrected one non-security provenance drift:
  `source-trace.json` named the obsolete v18 schema/poison proof. It now names
  the existing v20 test; the complete JSON file was re-read, parsed, hashed,
  and retained its no-candidate security disposition.
- Post-implementation CodeGraph inspection re-traced authenticated
  `compile_render_plan`/`lower_node` lowering, renderer/runtime consumers, and
  the QML one-delegate invariant. QML remains outside the AST relationship
  index and was therefore directly inspected with its tests. CodeGraph issued
  the worktree-bound inspection receipt at gated state
  `7fd3492f55b26ee2c661348e82e47e632f1c642840ffe34aa634377ce09b219e`.
- Phase 3.5 exit: no unresolved correctness, architecture, security, privacy,
  state/concurrency, reconnect, rollback, or performance finding blocks full
  validation.

## Phase 4 — Validate

- Full external validation passed with the ambient target directory removed
  and offline dependency resolution:
  `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh`. Evidence included formatting, warnings-denied Clippy,
  rustdoc, source/provenance verification, seventeen data tests, twenty-eight
  provider tests, three local-session tests, fifty-seven rules tests, one
  integration test, six trusted-control QML cases, seventeen signed screens,
  and provider-backed local-play smoke.
- The production provider passed its fixed fifteen-case TLS, replay, fault,
  callback, and persistence corpus twice across a real restart against a
  ticket-scoped PostgreSQL 18 instance. The host PostgreSQL service was neither
  stopped nor reused.
- The first platform gate attempt identified validation-environment conflicts,
  not product defects: the machine-wide temporary directory broke a self-test
  cleanup guard that intentionally accepts only `/tmp`, the host already owned
  port 5432, four gate scripts hard-code that port for child database URLs,
  and the ambient `CARGO_TARGET_DIR` defeated a clean-clone location assertion.
  A ticket-scoped Compose project used an unexposed PostgreSQL socket and a
  process-scoped temporary `connect(2)` redirect for only
  `127.0.0.1:5432`; `TMPDIR=/tmp` and an unset `CARGO_TARGET_DIR` restored the
  tests' documented assumptions without changing repository code or touching
  the host database.
- Each formerly red suite then passed independently: provider sidecar
  transport/operations, first-party provider authority, operator backup and
  restore, and invite-only private-alpha admission.
- The complete platform command used the isolated Compose project, temporary
  local-port redirect, `/tmp`, and no ambient Cargo target directory to run
  `bin/gate.sh --diff`. It passed all twenty-four stages and emitted
  `GATE GREEN [diff]`. The matching receipt is
  `7fd3492f55b26ee2c661348e82e47e632f1c642840ffe34aa634377ce09b219e`.
- A fresh v20 provider-backed local-play window replaced the older v19
  process and was moved silently to Hyprland workspace 8 while workspace 1
  remained active. Real compositor-directed Return/Tab input advanced exactly
  once through entry, Human, Alchemist, Main Street, Dungeon, Level 15, and
  Look. Live authenticated readback at revision 5 showed twenty-one nodes,
  eighteen buttons, and eighteen unique button IDs/actions; revision 6 showed
  a Level 15 Dark Hoodlum at 60/60 HP with five combat buttons, five unique
  IDs, and five unique actions.
- The user's duplicate/inert-button report was treated as a release blocker,
  not dismissed. Provider plans were unique in both old v19 and fresh v20,
  real keyboard activation worked for every live transition, and the full
  QML suite passed pointer activation, enabled-state transition, one Return
  emission, stale-delegate removal, and exact seventeen-to-eighteen
  instantiated-delegate replacement. A screenshot/mouse click against the
  live window was not attempted through the compositor password overlay; no
  password UI was manipulated. The v20 Level 15 combat remains open on
  workspace 8 for direct user observation after unlock.
- Phase 4 exit: REQ-001 through REQ-006 have implementation, isolation,
  security, full-regression, and live-runtime evidence. No unresolved product
  failure blocks completion.

## Phase 5 — Complete

- Acceptance audit:
  - REQ-001 satisfied by authenticated source hash/readback, exact Level 15
    records 140–149, source-trace validation, and compatibility/port-map
    documentation;
  - REQ-002 satisfied by exact v20 state identity, levels 1–15 consistency,
    and hostile old/missing/unknown/wrong-band/wrong-record/oversized cases
    that preserve state and RNG;
  - REQ-003 satisfied by the sequential draw-free levels 1–15 switch test and
    unchanged rejection of 0, 16, and `u16::MAX`;
  - REQ-004 satisfied by the forced rejected-140 trace, accepted records
    141–149, exact 20 strength/10 defence/60 HP, fight decrement, and
    deterministic twins;
  - REQ-005 satisfied by exact failed-retreat `(2, 1), (150, 10)` evidence and
    the complete attack/potion/spell/special/poison/death/reward/day regression
    suite;
  - REQ-006 satisfied by fixed/generic provider equivalence, one declared
    Level 15 action, unique signed controls, 17-to-18 actual-delegate proof,
    provider restart conformance, real workspace-8 Tab/Return revisions, and
    the explicit absence of Level 16, shared realm, platform gameplay,
    packaging, admission, deployment, and publication work.
- Hand-maintained `docs/architecture/game-cartridges.md` now records rules v20,
  levels 1–15, boundary record 140, normal records 141–149, `option_o`, and the
  17-to-18 QML replacement while preserving the development-only authority
  boundary.
- The required OpenWiki update lifecycle completed. `openwiki/quickstart.md`
  and `openwiki/game-cartridges.md` now carry the same v20/Level 15 and QML
  evidence. OpenWiki warned that those large pages retain pre-existing
  unresolved Claims evidence debt; it still returned `status: complete`, and
  no Claims sidecar was edited manually.
- `AAR-069` was submitted as effective. No new `BF-*`, `PR-*`, or `AD-*` was
  created because the validation-environment conflicts and reported live
  symptom are already covered by the existing Cargo, isolated-test, unique
  command, real-input, and instantiated-delegate rules.
- The final post-documentation `bin/gate.sh --diff` passed all twenty-four
  stages and printed `GATE GREEN [diff]`. Its receipt and the completed
  OpenWiki receipt both match gated state
  `7fbc73560498d635a3939b3088e3bb3682ca3949d216d4f9c421adf607ae7f92`.
- Archive: Ticket 069 is closed and this single spec/notes pair is moved to
  `pipeline/completed` with no remaining active pair.
- Phase 5 exit: pipeline complete. Delivery remains unauthorized, so no
  commit, push, pull request, registration, admission, deployment, or
  publication was performed.
