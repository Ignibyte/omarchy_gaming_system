---
title: Usurper Level Thirteen Dungeon Band — notes
pipeline_id: 12f2dd10-2c8a-4444-8e76-248d153146ca
---

# Usurper Level Thirteen Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 066 supplies exact rules/state/cartridge v17 through Level 12 plus
    matching OpenWiki/gate evidence and the current workspace-8 preview;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-120 draw to remain visible in deterministic traces;
  - `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001`
    and `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001` keep
    the thirteenth control unique and inside the actual Qt input boundary.
- Source preflight:
  - authenticated source Git and archive copies of `EDMONST.PAS` remain
    byte-identical at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - lines 3819–3918 define Level 13 records 120–129 as Blind Fool, Rude Boy,
    Big Bad Wolf, Floating Wizard, Dragon of Fire, Red Dragon, White Dragon,
    Spider Dragon, Ghoul, and White Elf, all at base strength 20 with exact
    equipment flags;
  - `DUNGEONC.PAS` lines 924–955 keep events separate, spend a fight, and
    repeat `Random(level*10)` until the result exceeds `(level-1)*10`; Level 13
    therefore normally selects records 121–129 and preserves record 120 only
    as source data;
  - the unregistered guard applies only when dungeon level is greater than 89,
    so Level 13 remains on the ordinary branch;
  - `PLVSMON.PAS` lines 68–98 use `Random(level*10)+3` for failed-retreat
    damage and lines 603–625 initialize monster HP to strength times three.
- Existing boundary fit:
  - the provider/rules reducer is generic through the implemented maximum;
  - the dungeon screen occupies `option_a` through `option_l`, so Level 13
    needs one new bounded external `option_m` field across `GameView`, schema,
    fixtures, and signed binding without changing the platform renderer;
  - Ticket 065's real-input suite and Ticket 066's unique-control assertions
    are game-neutral and will rerun unchanged.
- Baseline `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` passed: formatting, strict Clippy, 92 Rust tests, rustdoc,
  authenticated upstream/provenance checks, five real-input QML cases,
  seventeen unique signed screens, and provider-backed local play.
- Decision: implement Level 13 as the next normal dungeon band and defer Level
  14, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, random draws, monster data, and provider projections;
  - the local provider validates a revision-bound signed action, maps the
    bounded fixed Level 13 action to the existing `EnterDungeon` command, and
    asks the pure reducer for the next state and view;
  - the signed inert cartridge binds the new `option_m` projection to one
    declared button node; the platform renderer authenticates and lowers each
    declared node once, and trusted QML dispatches the unconfirmed action back
    to the provider without acquiring gameplay authority;
  - state flow is therefore `signed button -> local-play revision check ->
    provider action mapping -> pure reducer -> v18 state/view -> authenticated
    render plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` finds one authenticated screen, lowers its
    presentation nodes in declared order, requires every button action to be
    declared, and pushes each successful lowered node exactly once;
  - its blast radius identifies the client cartridge runtime and preview
    binary as callers and the renderer integration suite as direct coverage;
  - QML is not indexed, so `TrustedCartridgeSurface.qml`, its button/grid
    delegates, the real-input Qt tests, and the live provider path were
    inspected directly;
  - the worktree-bound design receipt is recorded for pipeline
    `12f2dd10-2c8a-4444-8e76-248d153146ca`.
- Exact implementation manifest, with one purpose per surface:
  - `crates/usurper-model/src/lib.rs`: add bounded serialized `option_m` to
    `GameView`;
  - `crates/usurper-data/src/lib.rs`: add exact records 120–129, lookup routing,
    and source-order/strength/equipment tests;
  - `crates/usurper-rules/src/lib.rs`: advance strict identity to v18, extend
    validation/switching/labels through Level 13, and add encounter, retreat,
    deterministic, and hostile-state evidence;
  - `crates/usurper-provider/src/lib.rs`: map the fixed Level 13 action and
    prove equivalence, projection, encounter, replay, and restart behavior;
  - `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare one
    Level 13 action/button, and require the bounded `option_m` field;
  - all seventeen `fixtures/presentation/*.json` files: provide the exact new
    required field, with a non-empty Level 13 label only on the dungeon view
    and Level 13 encounter facts on the combat view;
  - `provenance/source-trace.json`: register the reviewed Level 13 source
    records and existing selection/HP/retreat branches;
  - `scripts/test-cartridge.sh`, `scripts/test-provider.sh`, `scripts/test.sh`,
    and `scripts/play.sh`: expect v18/Level 13, preserve unique-control and
    real-input proof, run the new live profile twice, and end smoke play in a
    Level 13 encounter;
  - `README.md`, `docs/COMPATIBILITY.md`, and `docs/RUST_PORT_MAP.md`: document
    the newly implemented normal band and the unchanged exclusions;
  - platform `docs/architecture/game-cartridges.md`: reconcile the durable
    external-development boundary through rules v18/Level 13 during Phase 5;
  - `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`: strengthen
    the existing platform regression harness with actual delegate-tree counts
    across the ten-button race to eleven-button class replacement reported by
    the user;
  - the platform repository otherwise changes only lifecycle records for this
    ticket; no platform renderer, server, migration, Cargo, or client
    production code is required.
- Database and migration consequences: none. Provider-owned state is serialized
  inside the external adapter, and this slice adds no platform persistence,
  tables, columns, data migration, or PostgreSQL write path.
- API and compatibility contract:
  - state JSON remains strict and deny-unknown-fields, but exact
    `schema_version: 18` replaces v17; v17 and malformed v18 state fail before
    RNG construction or mutation;
  - the view schema adds required string `option_m` with the existing
    64-character bound; all screens supply it, and only the dungeon screen
    binds it to `enter_dungeon_level_13`;
  - the signed manifest advances `rules_version` and `cartridge_version`
    together to 18; SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_13` accepts an empty payload only and maps to the
    existing typed command. Levels 0, 14, and `u16::MAX` remain rejected
    without a revision or RNG advance.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_THIRTEEN_MONSTERS` arrays, source-trace validation, authenticated-source hash/readback, compatibility and port-map review |
  | REQ-002 | v18 identity checks; old/missing/unknown JSON fields; Level 14, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | one sequential levels 1–13 switch test with unchanged RNG/empty traces, visible labels, ascent/descent/remain behavior, and rejected 0/14/max inputs |
  | REQ-004 | forced rejected `Random(130)` draw followed by 121–129, exact 20 strength/10 defence/60 HP, fight decrement, and deterministic twin equality |
  | REQ-005 | exact failed-retreat `(2, 1), (130, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_m`, live Level 13 profile twice across restart, signed-screen/action uniqueness, Qt pointer/keyboard/screen-replacement suite, local-play action confirmation, and workspace-8 visual/readback audit |
- Risks and controls:
  - security/input: strict schemas, identifier checks, empty payloads,
    authenticated cartridge content, loopback capability, and revision/screen
    binding continue to reject undeclared or stale actions;
  - privacy/secrets: no account/persona identifiers or reusable credentials
    cross the local provider boundary; generated capabilities/private keys
    remain temporary and are not logged or committed;
  - state/concurrency: the provider serializes revision-aware actions and the
    pure reducer validates before constructing RNG, so rejected or concurrent
    stale actions cannot partially advance state;
  - reconnect/restart: the deterministic provider corpus runs twice around a
    fresh process and compares exact output; provider-owned sessions never
    fall back to platform rules;
  - rendering: a user reported duplicate/inert buttons during the v17 preview.
    A fresh workspace-8 run advanced once per real Return input through entry,
    race, class, and Main Street, and every signed screen inspected so far had
    equal total/unique IDs and actions. Phase 4 will still rerun pointer,
    keyboard, delegate-replacement, signed uniqueness, one-revision, and live
    workspace evidence; any repeat will be fixed at its proven producer;
  - rollback: v18 artifacts can be removed before delivery without data
    migration. Published rollback is out of scope and no delivery action is
    authorized.
- Decisions and rejected alternatives:
  - preserve record 120 as source data but not a normal encounter; directly
    choosing 120 would contradict the reviewed rejection loop;
  - extend `GameView` with `option_m`; reusing a primary/secondary field would
    overload its phase meaning, while a grid or platform renderer change would
    expand the architecture unnecessarily;
  - reuse the generic dungeon/combat reducer; duplicating Level 13 logic in the
    provider or QML would create a second rules authority;
  - keep dungeon events and registration behavior excluded; neither is needed
    to prove the ordinary Level 13 band.

## Phase 3 — Implement

- Implemented external rules/state/cartridge v18 with exact Level 13 records
  120–129, `option_m`, level bounds, fixed provider action, Level 13 signed
  presentation, fixtures, source trace, live paths, and compatibility docs.
- Added deterministic Level 13 evidence for source-order data, forced rejected
  `Random(130)` work, normal records 121–129, boundary-record exclusion,
  20-strength/10-defence/60-HP construction, failed-retreat bound/damage,
  generic/fixed provider equivalence, replay, restart profile, and strict v18
  hostile state.
- Expanded signed-cartridge validation from duplicate labels to duplicate node
  IDs, labels, actions, and triples on all seventeen screens. Expanded the
  local HTTP smoke to require unique button IDs/actions on race, class, Main
  Street, dungeon, and combat screens.
- In response to the user's live report, strengthened the platform Qt
  regression test to count actual trusted delegates across a 10-button to
  11-button screen replacement. This is a test-only deviation from the
  external-only manifest; no platform production renderer/gameplay code
  changed.
- Focused implementation checks passed:
  - `cargo test -p usurper-data -p usurper-rules -p usurper-provider
    --all-features`: 15 data, 24 provider, 3 local-play binary, 53 rules, and 1
    integration test passed;
  - Qt offscreen trusted-control suite: 6 total cases passed, including pointer,
    single Return activation, delegate removal, and large-screen replacement.
- First `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` run failed because the expanded sequential switch test
  exceeded the strict 100-line Clippy limit. Replaced the repetitive blocks
  with a table-driven levels 2–13 loop plus an exact thirteen-label array; no
  lint suppression was added.
- The fresh full external command then passed formatting, strict Clippy, 96
  Rust tests, rustdoc, authenticated source/provenance checks, all Qt control
  cases, seventeen signed unique-screen checks, and provider-backed local HTTP
  plus trusted-QML smoke.

## Phase 3.5 — Inspect

- Correctness/EARS: exact source data, bounds, schema identity, provider mapping,
  cartridge binding, fixtures, and all planned Level 13 transitions are
  present. The record-120 data/selection distinction and `Random(130)` trace
  match the reviewed source branch.
- State/concurrency: level switching stays inside the generic reducer and is
  draw-free; Look spends one fight before selection; failed commands validate
  before RNG construction; provider actions remain screen/revision bound.
- Security/privacy: no new network authority, platform identity, credential,
  database, executable cartridge code, or reusable capability was introduced.
  The fixed action requires an empty payload, signed content, and strict view
  schema; the full external secret/identity scan passed.
- Simplicity/reuse: no Level 13 reducer fork or QML gameplay rule was added.
  One data row set, one bounded view field, one provider mapping, and one
  signed button reuse the existing Level 2–12 path.
- QML/usability: current workspace-8 entry, race, class, and Main Street plans
  had equal total/unique button IDs and actions and advanced exactly one
  revision per real Return input. The strengthened delegate-tree test proves
  10-to-11 control replacement contains no old or duplicate trusted nodes.
  The user's duplicate/inert report is therefore not reproducible in the
  current tree; it remains guarded by the expanded Phase 4 corpus and fresh
  v18 visual readback.
- Finding ledger:

  | Finding | Disposition |
  |---|---|
  | Expanded switch test exceeded strict Clippy line limit | Confirmed and fixed with a table-driven sequence; fresh full external command passed. |
  | Possible duplicate/inert QML controls reported against the live preview | Investigated; no duplicate node/action/delegate and real Return advanced once. Added signed-plan and actual-delegate regressions; no production change justified without a current reproducer. |
  | Level 13 could drift into platform gameplay ownership | Rejected by inspection: only the external reducer/provider owns the new behavior; platform change is test-only. |
- Fresh post-implementation CodeGraph confirmed the unchanged flow
  `compile_render_plan -> lower_node -> rendered.push`, identified the client
  runtime/preview callers and renderer tests, and issued the matching inspect
  receipt for this pipeline. Unsupported QML and external-game files were
  inspected directly.

## Phase 4 — Validate

- Focused external Rust validation passed:
  `cargo test -p usurper-data -p usurper-rules -p usurper-provider
  --all-features` completed 96 tests across data, rules, provider,
  local-play-binary, and integration targets.
- Fresh full external validation passed:
  `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` completed formatting, strict Clippy, all 96 Rust tests,
  rustdoc, authenticated source/provenance checks, six Qt cases, all seventeen
  signed screens, and provider-backed local HTTP plus trusted-QML smoke.
- The full external provider security/conformance command passed its fixed
  fifteen-case TLS, replay, fault, callback, reconciliation, and gameplay
  corpus twice across a process restart with exact rules/state/cartridge v18
  and a Level 13 terminal state.
- `bin/gate.sh --diff` passed all 24 platform stages against an isolated
  PostgreSQL 18 instance, including the strengthened trusted-renderer QML
  suite, cartridge/runtime checks, SDK proofs, database/API smoke, remote
  provider security, backup/restore, admission, and server-module
  conformance. Matching receipt:
  `28a1180d4d5d028186f81b5bccc4f8d7cc270f13ecc1b30fdaf3b59ca5e888cd`.
- Live provider-backed validation replaced the older preview and left
  cartridge v18 running on workspace 8. Real Tab/Return input advanced Main
  Street revision 5 to dungeon revision 6, selected Level 13 at revision 7,
  and started the Level 13 Big Bad Wolf encounter at revision 8. The visible
  plans and actual screenshots showed exactly 12/12/12 Main Street, 16/16/16
  dungeon, and 5/5/5 combat button/unique-ID/unique-action counts. The Level
  13 story, fight decrement, and 60/60 monster HP were visible; no duplicate
  controls were present and each real activation advanced exactly once.
- Scope readback found no Level 14 action, dungeon event, shared-realm,
  packaging, admission, deployment, database, migration, or platform gameplay
  implementation. No commit, push, or publication was performed.
- Phase 5 architecture reconciliation updates only the platform's durable
  description of the external v18/Level 13 proof and strengthened QML
  regression; it does not change runtime authority or production behavior.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — authenticated v0.20e hash/readback, exact records 120–129,
    source trace, compatibility docs, and fixed-data tests agree;
  - REQ-002 PASS — exact-v18 state, malformed JSON, old schema, unsupported
    level, boundary/unknown record, wrong name/level, and state/RNG
    immutability tests pass;
  - REQ-003 PASS — levels 1–13 switch without draws and zero, fourteen, and
    larger levels reject unchanged;
  - REQ-004 PASS — forced `Random(130)` traces retain rejected record 120,
    accept records 121–129, spend one fight, and prove 20/10/60 combat state;
  - REQ-005 PASS — Level 13 retreat damage plus existing attack, heal, spell,
    class-special, reward, poison, replay, and complete-day regressions pass;
  - REQ-006 PASS — generic/fixed provider equality, exactly one signed Level
    13 control over `option_m`, signed-plan and instantiated-delegate
    uniqueness, real-input one-revision proof, restarted live corpus, trusted
    QML combat, inspection, the 24-stage platform gate, and workspace-8 play
    pass.
- Documentation: the external README, compatibility ledger, Rust port map,
  provenance, fixtures, and scripts plus the platform Game Cartridges
  architecture and generated OpenWiki quickstart/Game Cartridges pages now
  describe rules/state/cartridge v18 and the normal Level 13 band while
  retaining the provider-owned, non-production boundary. OpenWiki run
  `5a397945-61ee-4596-af94-58d0a81fff52` completed; its warnings concern the
  pre-existing unresolved quickstart/Game Cartridges evidence debt rather than
  an incomplete Ticket 067 update.
- The final post-documentation `bin/gate.sh --diff` passed all 24 stages and
  printed `GATE GREEN [diff]`, with matching receipt
  `28a1180d4d5d028186f81b5bccc4f8d7cc270f13ecc1b30fdaf3b59ca5e888cd`.
  Before that clean run, isolation-harness attempts failed because a
  nonstandard `TMPDIR`, missing Compose overrides, and the shell's ambient
  `CARGO_TARGET_DIR=/mnt/fast/target` violated test-script assumptions; the
  last one redirected a deliberately clean-clone provider binary away from
  the asserted path. No product source changed in response. Focused rerun and
  the complete gate passed after normalizing those environment variables.
- AAR 067 is submitted and effective. The new durable rule requires recursive
  instantiated-delegate cardinality checks across realistic plan replacement,
  closing the evidence gap highlighted by the user's duplicate/inert-control
  report without a speculative production renderer change or a new ADR.
- Archive: Ticket 067 is closed and this single spec/notes pair is moved to
  `pipeline/completed` with no remaining active pair.
- Phase 5 exit: pipeline complete. Delivery remains unauthorized, so no
  commit, push, pull request, registration, admission, deployment, or
  publication was performed.
