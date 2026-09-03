---
title: Usurper Level Nine Dungeon Band — notes
pipeline_id: 08d838f4-e3ad-4918-b5da-d4da451ee10f
---

# Usurper Level Nine Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - `BUL-002-pre-rebuild-delivery-handoff` remains informational: the ignored
    upstream corpus, provider kit, preview keys, database state, and workflow
    receipts were checked as local evidence and remain outside source control.
  - Ticket 060 completed exact levels one through eight as rules/state/cartridge
    v13; Ticket 061 added real provider-backed trusted-QML play and kept the
    fixture viewer explicitly inert.
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires the generic dungeon calculation and enclosing event/registration
    branches to be read before translating the Level 9 outcome set.
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    record-80 rejection draws to remain observable in the deterministic trace.
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` requires
    the live profile to follow the actual post-Level-9 phase/HP trace rather
    than assuming the prior Level 8 command sequence still fits.
  - `PR-omarchy-gaming-system-resolve-cargo-artifacts-from-metadata-001` and the
    Ticket 061 no-override launcher lesson remain applicable to visible/live
    validation.
- Source preflight:
  - authenticated upstream Git tree is clean at
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`;
  - `EDMONST.PAS` lines 3412–3512 define records 80–89 as Sabre Wulf, Dwarf
    King, Great Paladin, Orc Chieftain, Silver Elf, Insane Gnoll, Unknown
    Monster, Orc Noble, Severe Madman, and Uruk-Hai, all with base strength 19;
  - `DUNGEONC.PAS` derives `xx=(level-1)*10`, `x=level*10`, then repeats
    `Random(x)` until the result exceeds `xx`, so Level 9 retains record 80 but
    normally selects 81–89 through `Random(90)`;
  - `PLVSMON.PAS` initializes a loaded monster to strength-times-three HP and
    uses `Random(global_dungeonlevel*10)+3` for failed-retreat damage.
- Decision: implement Level 9 as the next normal-dungeon band; defer Level 10,
  dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and data flow:
  - `usurper-data` owns the exact immutable Level 9 editor fixtures;
  - `usurper-rules` remains the sole game authority for level switching,
    rejection-loop selection, combat, validation, and deterministic RNG;
  - `usurper-provider` decodes only the generic `enter_dungeon` and fixed
    `enter_dungeon_level_9` forms, then returns the ordinary bounded view;
  - the signed cartridge binds the existing `option_i` view field to one inert
    `enter_dungeon_level_9` button; the trusted renderer already admits this
    node shape and the dungeon screen remains below its node budget;
  - OmarchyGS continues authenticating and translating the signed zero-payload
    action, brokering it to the exact provider release, and rendering the
    returned typed plan without Usurper-specific rules or state.
- CodeGraph cannot inspect the separate Usurper repository because it has no
  `.codegraph/` index, so its Rust sources and non-Rust contracts were inspected
  directly. The worktree-bound platform exploration traced
  `ValidatedSessionCartridgeAction`, cartridge presentation validation,
  provider release identity, provider-game execution, and bounded render-plan
  consumption. It found no Level-9-specific platform dependency or necessary
  platform source change. Design receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `08d838f4-e3ad-4918-b5da-d4da451ee10f`, state hash
  `b5d7e662163688ad86aed83ab36d5d6038a6568126b41904d111a6b22a8da041`.
- API/state and compatibility contract:
  - advance external state, rules, and cartridge identity from v13 to v14 and
    accept exact v14 only; no v13 state is silently migrated;
  - accept generic `enter_dungeon` levels 1–9 and fixed level actions 1–9;
    reject 0, 10, and larger unchanged and without RNG work;
  - require active monsters to belong to the selected implemented band, match
    the exact source-linked name, retain bounded scalars, and exclude every
    normally unreachable boundary record;
  - retain record 80 in immutable data while normal Level 9 selection accepts
    only records 81–89 after all source-order `Random(90)` rejections;
  - preserve Provider SDK/protocol v1, game key, provider ID, player-private
    state shape, and seventeen-screen presentation protocol.
- Exact canonical Level 9 data contract:

  | Index | Name | Base strength | Armor user | Weapon user | Normal Level 9 selection |
  |---:|---|---:|---|---|---|
  | 80 | Sabre Wulf | 19 | no | no | no — boundary record |
  | 81 | Dwarf King | 19 | yes | yes | yes |
  | 82 | Great Paladin | 19 | yes | yes | yes |
  | 83 | Orc Chieftain | 19 | yes | yes | yes |
  | 84 | Silver Elf | 19 | yes | yes | yes |
  | 85 | Insane Gnoll | 19 | yes | yes | yes |
  | 86 | Unknown Monster | 19 | no | yes | yes |
  | 87 | Orc Noble | 19 | yes | yes | yes |
  | 88 | Severe Madman | 19 | yes | yes | yes |
  | 89 | Uruk-Hai | 19 | yes | yes | yes |

- Database and migration consequences: none in OmarchyGS. The external starter
  continues owning its independent PostgreSQL state and operation receipts;
  strict v14 identity uses fresh development sessions instead of mutating v13
  rows.
- Planned external-provider file manifest:
  - `crates/usurper-data/src/lib.rs` — exact rows 80–89, lookup, and data tests;
  - `crates/usurper-rules/src/lib.rs` — v14 validation, level selection/view,
    encounter/retreat behavior, and reducer regressions;
  - `crates/usurper-provider/src/lib.rs` — fixed action plus generic/fixed,
    replay, view, and live-profile coverage;
  - `cartridge/manifest.json`, `cartridge/presentation.json` — exact v14
    identity and inert Level 9 control;
  - `fixtures/presentation/dungeon.json`,
    `fixtures/presentation/combat.json` — signed Level 9 render facts;
  - `provenance/source-trace.json` — source-to-Rust Level 9 evidence;
  - `scripts/test.sh`, `scripts/test-provider.sh` — human-readable gate label,
    exact v14 live profile, and composite restart/replay assertions;
  - `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` — current
    scope, compatibility ledger, and port milestone.
- Platform files remain limited to Ticket 062 planning, OpenWiki
  reconciliation, and completion evidence. No server, SDK, QML, route,
  database, migration, or renderer-vocabulary source change is designed.
- Regression table:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Exact ten-row data/order/flag test, source hashes, source-trace validator, and compatibility review. |
  | REQ-002 | v14 exact-schema test; unsupported level, boundary/unknown record, wrong name, malformed JSON, and state/RNG immutability checks. |
  | REQ-003 | Draw-free level 1–9 transitions, visible labels, phase/location/monster checks, and 0/10/max rejection. |
  | REQ-004 | Forced rejected/accepted `Random(90)` trace, record 80 exclusion, records 81–89 bound, 19/9/57 combat state, fight spend, and deterministic twin. |
  | REQ-005 | Exact `(2, 90)` retreat trace plus existing attack, quick-heal, spell, class-special, reward, poison, and complete-day suite. |
  | REQ-006 | Generic/fixed provider equality and replay, signed cartridge conformance, all-screen QML smoke, live corpus twice across restart, platform diff gate, scope/security review, and provider-backed workspace-8 play. |
- Risk and rollback review:
  - wrong roster order/flags or exposing record 80 is covered by exact arrays,
    lookup boundaries, and forced rejection traces;
  - added encounter RNG can shift later retreat/death profile outcomes, so the
    complete live driver will be replayed and reconciled to actual phases;
  - a ninth dungeon control can regress layout or action declaration, so the
    signed all-screen QML smoke and visible provider-backed play must exercise
    it;
  - strict v14 prevents mixed-schema interpretation; rollback is the
    unmodified v13 release/session identity, not in-place state conversion;
  - no new identity, credential, network, database, executable-content, shared
    realm, or platform-authority surface is introduced.
- Alternatives rejected:
  - inferring Level 9 from the existing arithmetic without adding exact source
    rows would lose editor order, names, equipment flags, and record-80 proof;
  - adding Level 10 or dungeon events would combine distinct source branches
    and make the smallest source-complete slice materially harder to verify;
  - implementing a platform-side level rule or bespoke renderer control would
    violate the provider/cartridge authority boundary.
- Rebuilt-machine baseline:
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system scripts/test.sh`
    passed on v13 before implementation: formatting, warning-denying Clippy,
    74 Rust tests, rustdoc, immutable upstream hashes, provenance/privacy,
    signed seventeen-screen QML state smoke, and provider-backed local-play
    smoke.
- Phase 2 exit: source contract, architecture, compatibility boundary, exact
  file manifest, regression mapping, risks, baseline, and worktree-bound
  CodeGraph evidence are actionable.

## Phase 3 — Implement

- Built:
  - added exact Level 9 rows 80–89 to `usurper-data`, with canonical ordering,
    source spelling, base strength 19, equipment flags, lookup coverage, and
    explicit record-80/record-89/record-90 assertions;
  - advanced rules/state identity to v14, accepted levels one through nine,
    exposed `option_i`, and retained rejected `Random(90)` draws until exact
    records 81–89 are selected with strength 19, defence 9, and 57 HP;
  - added draw-free Level 9 switching plus selection, deterministic twin,
    boundary, retreat-damage, hostile-state, generic/fixed provider, view, and
    replay regressions while preserving the complete lower-level suite;
  - added the fixed `enter_dungeon_level_9` provider action and signed inert
    dungeon button, rules/cartridge v14 identity, Level 9 fixtures, provenance,
    and current-scope compatibility documentation;
  - reconciled the full provider live driver to actual deterministic behavior:
    unlike Level 8, the first two Level 9 retreat attempts succeed, so the
    profile returns to Main Street after the second source-composed encounter
    instead of inventing a death/re-entry transition.
- Focused and complete implementation proof:
  - `cargo test -p usurper-data`: 11 passed;
  - `cargo test -p usurper-rules`: 45 unit plus 1 integration passed;
  - `cargo test -p usurper-provider`: 19 library plus 3 local-play passed;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    scripts/test-cartridge.sh`: all seventeen signed screens and trusted-QML
    state smoke passed;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system scripts/test.sh`:
    formatting, warning-denying Clippy, all 79 Rust tests, rustdoc, immutable
    upstream/provenance and privacy assertions, signed-screen conformance, and
    provider-backed local HTTP/QML smoke passed;
  - JSON parsing, shell syntax, and `git diff --check` passed.
- Implementation stayed within the Phase 2 manifest. No platform server, SDK,
  protocol, QML, route, renderer vocabulary, database, migration, packaging,
  admission, deployment, delivery, or publication surface changed.
- Phase 3 exit: the v14 external-provider slice and focused evidence are ready
  for independent inspection.

## Phase 3.5 — Inspect ledger

- Scope inspection found no drift from the Phase 2 manifest: gameplay changes
  remain in the separate Usurper data/rules/provider/cartridge repository, and
  this platform worktree contains only lifecycle evidence for Ticket 062.
- The worktree-bound CodeGraph inspection traced the signed presentation,
  action-contract validation, provider session handoff, and trusted rendering
  boundary. It confirmed that the generic platform path accepts the new
  provider-owned `enter_dungeon_level_9` action and v14 release identity
  while the later UI repair remains generic trusted-renderer behavior rather
  than platform gameplay. The receipt was refreshed after the final renderer
  and documentation changes. Inspection receipt:
  `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`, pipeline
  `08d838f4-e3ad-4918-b5da-d4da451ee10f`, state hash
  `c4dfa62450266e03c07800d347f1ca33ebe42a308893a3c38a38f8ab0f3ab5d5`.
- Final Codex Security working-tree scans covered both repositories:
  - external scan `db03c3af-91bb-476b-b0c1-e2f642f0ab3a` reviewed the Level 9
    data/rules/provider/cartridge and repaired local-play path. Its provider
    next-screen and stored-state candidates were suppressed with high
    confidence after exact source-to-sink tracing; coverage is complete with
    zero reportable findings. Sealed report:
    `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260902T221314Z_1o2_7w_m/report.md`;
  - platform scan `79953bd0-f839-4d13-b793-5ce65612eaf5` reviewed the final
    delta after the metrics assertion update. Its explicit screen-selector
    candidate was suppressed with high confidence because selection remains
    an exact lookup inside the verified signed presentation and unknown IDs
    fail without output; coverage is complete with zero reportable findings.
    Sealed report:
    `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchy_gaming_system/b7428c813bd72c1a8759333d20beef7b67696db4_20260902T222734Z_op8fi6ub/report.md`.
- Phase 3.5 exit: implementation, architecture boundaries, scope, and security
  posture are accepted for full validation.

## Phase 4 — Validate

- Visible validation initially found two user-facing defects after the first
  provider mutation:
  - the local-play adapter retained the old signed screen after the provider
    moved phases, leaving newly relabeled controls bound to phase-invalid old
    actions;
  - the Usurper presentation rendered provider-command and `navigate.*`
    controls with the same label, while a held Return could auto-repeat across
    asynchronous plan replacement and invoke the newly focused control.
- Repair:
  - provider mutations now derive the next screen from the authenticated
    candidate view and still publish state/revision only after signed rendering
    succeeds;
  - the Usurper cartridge exposes only one phase-valid provider command for
    each visible choice and no longer requests the unused navigation
    capability; the generic adapter retains revision-neutral navigation for
    other development cartridges;
  - trusted buttons and grids accept but ignore auto-repeat activation events,
    and their QML smoke exercises that guard;
  - cartridge conformance now rejects duplicate visible button labels on every
    one of the seventeen screens.
- Focused proof after repair:
  - all seventeen signed screens and the trusted-QML state smoke passed;
  - provider-backed local HTTP/QML smoke passed through Entry, Human,
    Alchemist, Main Street, Level 1, Level 9, and combat;
  - the complete external suite again passed formatting, strict Clippy, all 79
    Rust tests, rustdoc, provenance/privacy, cartridge, and local-play checks;
  - the fixed fifteen-case TLS/replay/fault/callback provider corpus passed
    twice across its restart boundary on the isolated PostgreSQL service;
  - workspace-8 keyboard play advanced exactly one revision per activation,
    rendered unique Main Street controls, and reached a Level 9 Severe Madman
    encounter with 57/57 HP.
- One attempted provider-corpus invocation omitted the Compose override, so
  Docker failed before tests while trying to bind the occupied system port
  5432. The corrected isolated-port invocation passed and changed neither
  repository.
- The first full platform gate correctly failed its renderer stage because the
  runtime's new `repeats_blocked=2` proof made the older exact log assertion
  stale. After updating that assertion, the focused renderer suite passed and
  `bin/gate.sh --diff` passed all 24 stages with receipt/state hash
  `797bd8ed751f93a3e624a3692a2f43fc45ee373ea86381739e00d6dbb303cb2f`.
- Phase 4 exit: focused behavior, the complete external suite, restart corpus,
  visible workspace-8 play, final security scans, and the complete platform
  gate all pass.

## Phase 5 — Complete

- Acceptance audit:

  | Requirement | Final evidence | Result |
  |---|---|---|
  | REQ-001 | Authenticated v0.20e source readback, exact records 80–89 tests, provenance validation, and compatibility documentation. | PASS |
  | REQ-002 | Strict v14 schema plus hostile state/JSON, cross-field, level/record/name, and RNG-immutability tests. | PASS |
  | REQ-003 | Draw-free generic/fixed level 1–9 transitions and unchanged rejection for 0, 10, and larger values. | PASS |
  | REQ-004 | Forced `Random(90)` rejection trace, boundary-record exclusion, records 81–89 bounds, deterministic twins, and exact 19/9/57 combat state. | PASS |
  | REQ-005 | Full lower-level attack, retreat, potion, spell, class-special, reward, poison, and complete-day regressions plus exact Level 9 retreat trace. | PASS |
  | REQ-006 | Seventeen signed screens, unique visible choices, multi-phase provider-backed HTTP/QML smoke, provider corpus twice across restart, workspace-8 play, security scans, and full platform gate. | PASS |

- OpenWiki lifecycle `0c5977b6-62ac-4b39-b2dd-aade28751f6c`
  completed and reconciled `openwiki/quickstart.md` plus
  `openwiki/game-cartridges.md`. It reported only pre-existing unresolved Claims
  evidence debt for those pages; their sidecars were left unchanged.
- `docs/architecture/game-cartridges.md` now records authenticated provider
  screen following, render-before-commit, one phase-valid visible Usurper
  command, and generic trusted button/grid auto-repeat suppression.
- AAR 062 is submitted and effective. Three new BF IDs and three prevention
  rules were appended to the knowledge register; the existing provider-backed
  local-play architecture decision was amended rather than duplicated.
- Ticket 062 is closed, its index entry and artifact links are reconciled, and
  the spec/notes are ready for the completed pipeline archive.
- Scope audit found no silent drops: Level 10+, dungeon events, quests, finale,
  shared realm, new combat systems, provider protocol/schema, platform game
  rules, database migrations, registration, admission, deployment, commit,
  push, and publication remain out of scope.
- Phase 5 exit: all six acceptance criteria pass; durable documentation,
  knowledge, AAR, ticket, and OpenWiki evidence are reconciled.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first provider action succeeded but the shell kept the old screen, so later controls were phase-invalid. | The local adapter forced the currently selected screen when preparing a provider candidate instead of trusting the candidate view's authenticated screen. | Derive mutation screens from `UsurperGame::view(candidate)` and extend the HTTP smoke across the full multi-phase Level 9 path. | A local-play smoke must cross multiple provider phase/screen transitions, not stop after the first confirmed action. |
| 2 | Most Main Street choices rendered twice and one Return could cascade through newly rendered controls. | The cartridge paired provider commands with same-label navigation controls, and trusted nodes accepted auto-repeat activation after asynchronous focus replacement. | Remove the redundant Usurper navigation nodes/actions and ignore auto-repeat activation in trusted button/grid nodes. | Require unique visible button labels per signed screen and smoke the auto-repeat guard; visibly test several consecutive keyboard transitions. |
| 3 | One provider validation attempt tried to recreate the test database on occupied port 5432. | The Compose override was not exported to the script that invokes Compose internally. | Rerun with the exact project plus isolated-port Compose files in `COMPOSE_FILE`. | Treat the Compose override as part of the provider-test invocation contract, not merely container setup. |
