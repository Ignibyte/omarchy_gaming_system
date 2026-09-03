---
title: Usurper Level Twelve Dungeon Band — notes
pipeline_id: 4ec0cc25-d550-41c3-87a3-57b934c3c8d6
---

# Usurper Level Twelve Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 064 supplies exact rules/state/cartridge v16 through Level 11;
    Ticket 065 adds real pointer/Return, enablement, delegate-cardinality, and
    plan-replacement proof to both platform and external cartridge gates.
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together.
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-110 draw to remain visible in deterministic traces.
  - `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001` keeps the
    new twelfth control inside the real-input regression boundary.
- Source preflight:
  - authenticated source Git and archive copies of `EDMONST.PAS` remain
    byte-identical at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - lines 3717–3817 define Level 12 records 110–119 as Bugbear, Dront, Giant
    Dront, Dark Soul, Gorgol, Orc Leader, two more Bugbears, Mad Mage, and
    Jester, all at base strength 20 with exact equipment flags;
  - `DUNGEONC.PAS` lines 924–955 keep events separate, spend a fight, and
    repeat `Random(level*10)` until the result exceeds `(level-1)*10`; Level 12
    therefore normally selects records 111–119 and preserves record 110 only
    as source data;
  - the unregistered guard applies only when dungeon level is greater than 89,
    so Level 12 remains on the ordinary branch;
  - `PLVSMON.PAS` lines 68–98 use `Random(level*10)+3` for failed-retreat
    damage and lines 603–625 initialize monster HP to strength times three.
- Existing boundary fit:
  - the provider/rules reducer is generic through the implemented maximum;
  - the dungeon screen has occupied `option_a` through `option_k`, so Level 12
    needs one new bounded external `option_l` field across `GameView`, schema,
    fixtures, and signed binding without changing the platform renderer;
  - Ticket 065's real-input suite is game-neutral and will rerun unchanged.
- Baseline `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` passed: formatting, strict Clippy, 88 Rust tests, rustdoc,
  authenticated upstream/provenance checks, five real-input QML cases,
  seventeen unique signed screens, and provider-backed local play.
- Decision: implement Level 12 as the next normal dungeon band and defer Level
  13, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and data flow:
  - `usurper-data` owns the immutable Level 12 records and exact equipment
    flags; `usurper-rules` remains the only selector/combat/state authority;
  - `usurper-model::GameView` gains one bounded string, `option_l`, which the
    rules projection fills only for the twelfth dungeon level;
  - `usurper-provider` decodes generic level 12 and one exact fixed action,
    then returns the same external data-only view through the existing SDK;
  - the signed cartridge schema authenticates `option_l`, and its dungeon
    screen binds exactly one button to `enter_dungeon_level_12`;
  - OmarchyGS continues to validate the signed schema/action, lower a generic
    `RenderedNode::Button`, and submit one expected-revision command. It gains
    no Usurper-specific rule, field, route, state, or migration.
- The external repository has no `.codegraph` index, so its model, data,
  reducer, provider, JSON, and shell contracts were inspected directly.
  Platform CodeGraph traced signed button lowering through
  `compile_render_plan`, declared-action validation, and renderer consumers.
  The blast radius remains the generic renderer/runtime tests; `option_l` is
  interpreted only as authenticated external view data. Design receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `4ec0cc25-d550-41c3-87a3-57b934c3c8d6`, gated state
  `825b00a007df59a9849c4400f2f3a4fe3833ec60c344b7a427d7a4934e486879`.
- API/state compatibility:
  - advance strict external rules/state/cartridge identity from v16 to v17;
    reject old v16 serialized state rather than silently relabeling it;
  - accept generic/fixed dungeon levels 1–12 and reject zero, thirteen, and
    larger levels without state or RNG change;
  - require an active monster to match its implemented band, source name,
    selected level, and bounded scalars, excluding every normally unreachable
    band-boundary record;
  - keep Provider SDK/protocol v1, provider/game identity, player-private state,
    and the existing seventeen-screen presentation protocol unchanged.
- Exact canonical Level 12 data contract:

  | Index | Name | Base strength | Armor user | Weapon user | Normal selection |
  |---:|---|---:|---|---|---|
  | 110 | Bugbear | 20 | no | no | no — boundary record |
  | 111 | Dront | 20 | no | no | yes |
  | 112 | Giant Dront | 20 | no | no | yes |
  | 113 | Dark Soul | 20 | yes | yes | yes |
  | 114 | Gorgol | 20 | no | yes | yes |
  | 115 | Orc Leader | 20 | yes | yes | yes |
  | 116 | Bugbear | 20 | no | no | yes |
  | 117 | Bugbear | 20 | no | no | yes |
  | 118 | Mad Mage | 20 | no | yes | yes |
  | 119 | Jester | 20 | yes | yes | yes |
- Database and migration consequences: none in OmarchyGS. The external starter
  retains separate provider persistence; strict v17 uses fresh development
  sessions rather than rewriting v16 rows.
- Exact external file manifest:
  - `crates/usurper-model/src/lib.rs` — add the bounded serialized `option_l`;
  - `crates/usurper-data/src/lib.rs` — exact records 110–119, lookup, and data
    regressions;
  - `crates/usurper-rules/src/lib.rs` — v17 bounds, projection, encounter,
    retreat, state-hostility, and reducer regressions;
  - `crates/usurper-provider/src/lib.rs` — fixed action plus generic/fixed,
    replay, view, and live-profile coverage;
  - `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json` — v17 identity, authenticated
    `option_l`, and exactly one Level 12 control;
  - every `fixtures/presentation/*.json` view — exact required `option_l`, with
    the dungeon fixture populated and all other screens empty;
  - `provenance/source-trace.json` — source-to-Rust Level 12 evidence;
  - `scripts/play.sh`, `scripts/test-provider.sh`, and `scripts/test.sh` — live
    Level 12 path, exact v17 corpus, and current gate label;
  - `README.md`, `docs/COMPATIBILITY.md`, and `docs/RUST_PORT_MAP.md` — current
    scope, compatibility ledger, and milestone.
- Platform files remain limited to Ticket 066 lifecycle/completion evidence
  unless a real platform regression is discovered. No renderer, QML, server,
  SDK, route, database, migration, package, admission, or deployment change is
  designed.
- Regression table:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Exact ten-row order/name/strength/flag tests, authenticated source hashes, source-trace validation, and compatibility review. |
  | REQ-002 | Exact-v17 schema test; unsupported level, boundary/unknown record, wrong name/level, malformed JSON, and complete state/RNG immutability. |
  | REQ-003 | Draw-free level 1–12 transitions, visible labels, location/phase/monster assertions, and zero/thirteen/max rejection. |
  | REQ-004 | Forced rejected/accepted `Random(120)` trace, record-110 exclusion, records 111–119 bound, 20/10/60 combat state, fight spend, and deterministic twins. |
  | REQ-005 | Exact failed-retreat `(2, 120)` trace plus existing attack, heal, spell, class-special, reward, poison, and complete-day regressions. |
  | REQ-006 | Generic/fixed provider equality and replay, unique signed label/action, all-screen and real-input QML suites, one-click/one-revision local play, restarted live corpus, platform diff gate, scope/security review, and workspace-8 play. |
- Risk, privacy, concurrency, reconnect, and rollback review:
  - wrong duplicate Bugbear row flags/order or accidental record-110 reachability
    are covered by exact arrays and rejection traces;
  - adding `option_l` requires every exact JSON view to change together;
    `deny_unknown_fields`, `additionalProperties:false`, required schema fields,
    renderer binding tests, and full fixture inventory prevent partial drift;
  - a twelfth control remains within renderer node/text budgets, but unique
    labels, exact delegate cardinality, real pointer/Return tests, auto-repeat
    suppression, and provider revision assertions remain release gates;
  - provider revision/idempotency checks cover duplicate activation and replay;
    no shared writer or new concurrency primitive is introduced;
  - no new credential, identity, personal data, network listener, production
    configuration, database, executable code, or shared-realm surface is added;
  - rollback is the unchanged v16 release/session identity, not in-place state
    conversion.
- Alternatives rejected:
  - reusing `option_k` would conflate Level 11 and Level 12 and recreate a
    duplicate/ambiguous control contract;
  - accepting record 110 because it exists in the editor table contradicts the
    ordinary rejection loop;
  - adding Level 13 or events would cross independent source/control-flow
    boundaries; adding platform game logic would violate provider ownership.
- Phase 2 exit: the source contract, bounded schema extension, exact manifest,
  tests, risks, rollback, and matching CodeGraph receipt are actionable.

## Phase 3 — Implement

- Added exact immutable Level 12 records 110–119, lookup coverage, and a
  ten-row order/name/strength/equipment-flag regression in `usurper-data`.
- Advanced external rules/state/cartridge identity to v17 and widened the
  generic dungeon reducer only to levels 1–12. The rules projection now
  carries bounded `option_l`; Level 12 uses the established rejection loop,
  three-times-strength HP, level-derived defence, and retreat damage without
  adding a second combat implementation.
- Added the exact fixed `enter_dungeon_level_12` provider action alongside the
  generic command, replay/view equality, back-to-Level-11 switching, and a
  deterministic authenticated-combat/re-entry profile.
- Added one signed Level 12 dungeon button, one declared empty-payload action,
  the required schema field, and `option_l` to all seventeen exact view
  fixtures. The dungeon and combat fixtures now exercise Level 12; no Level 13
  or navigation twin was introduced.
- Updated the local-play HTTP contract and live provider corpus to traverse
  Level 12 under v17, plus provenance, compatibility, port-map, and release
  summary documentation.
- Focused implementation checks:
  - `cargo fmt --all -- --check` passed;
  - `cargo test -p usurper-data -p usurper-rules -p usurper-provider
    --all-features` passed 92 unit/integration tests after one stale shared
    boundary assertion was corrected from rejecting newly supported Level 12
    to rejecting Level 13;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    ./scripts/test-cartridge.sh` passed all five real-input QML lifecycle cases
    and the seventeen-screen signed-cartridge smoke;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    ./scripts/test-local-play.sh` passed the provider-backed local HTTP and
    trusted-QML smoke;
  - full `./scripts/test.sh` passed formatting, strict Clippy, 92 Rust tests,
    rustdoc, immutable-source/provenance checks, the real-input QML cases,
    all seventeen signed screens, and provider-backed local play;
  - the fixed live 15-case TLS/replay/fault/callback provider corpus passed
    twice across restart against a temporary isolated PostgreSQL 18 container.
    The container and mode-0600 test credential file were removed; the host
    PostgreSQL instance was not used or changed.
- Implementation stayed within the approved external manifest. Platform
  changes remain lifecycle evidence only; there is no platform game rule,
  renderer vocabulary, QML, API, database, migration, package, admission,
  deployment, commit, push, or publication change.
- Phase 3 exit: the external v17 Level 12 slice and focused evidence are ready
  for skeptical inspection.

## Phase 3.5 — Inspect

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Canonical source and rules | Records 110–119, equipment flags, the rejected record-110 draw, accepted 111–119 range, 20/10/60 combat state, and `(2, 120)` failed-retreat trace agree with the authenticated source contract. | None | No change required. |
| 2 | Boundary regression | The widened shared transition test initially still listed Level 12 among rejected inputs. | Low test correctness | Corrected the stale assertion to reject Level 13; the focused and full suites then passed. |
| 3 | State/provider/replay | Generic and fixed Level 12 actions converge on one reducer command; strict v17 state validation rejects old schema, unsupported levels, boundary/unknown monsters, wrong names, and level mismatch. Restart replay and expected revisions retain one state transition per accepted action. | None | No change required. |
| 4 | Signed presentation and QML | The signed dungeon plan has 117 globally unique node IDs, exactly one Level 12 node/action, one required bounded `option_l`, and all seventeen fixtures carry the exact field. Existing pointer, Return, enablement, auto-repeat, and plan-replacement cases cover the generic control lifecycle. | None | No change required. |
| 5 | Authentication, secrets, privacy, and abuse | No identity, credential, personal-data, public listener, executable-cartridge, or authorization surface was added. The live corpus used its secure credential-file contract and an isolated test database that was removed. | None | No change required. |
| 6 | Scope, persistence, and complexity | The change reuses the deterministic reducer/provider and generic renderer. No platform gameplay rule, database/migration, concurrent writer, protocol, navigation twin, Level 13, event, package, admission, deployment, or publication path was introduced. | None | No change required. |

- Direct inspection covered the unindexed external Rust, JSON, fixtures, shell,
  documentation, and authenticated Pascal source. `git diff --check`, exact
  JSON parsing, fixture cardinality, node/action uniqueness, and schema-field
  probes passed.
- Fresh platform CodeGraph inspection traced authenticated schema validation,
  declared-action checking, `RenderedNode::Button` lowering, generic renderer
  consumers, and the existing control test boundary. It found no Level
  12-specific platform dependency. Inspect receipt:
  `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`, pipeline
  `4ec0cc25-d550-41c3-87a3-57b934c3c8d6`, state hash
  `825b00a007df59a9849c4400f2f3a4fe3833ec60c344b7a427d7a4934e486879`.
- Phase 3.5 exit: the one stale test expectation is resolved and no correctness,
  security, privacy, state-integrity, QML, or scope finding remains.

## Phase 4 — Validate

- The focused 92-test Rust data/rules/provider suite, five-case real-input QML
  lifecycle suite, seventeen-screen signed-cartridge suite, provider-backed
  local-play smoke, and complete external `scripts/test.sh` all passed.
- The fixed fifteen-case live TLS/replay/fault/callback provider corpus passed
  twice across a provider restart against a disposable PostgreSQL 18 database.
  Its container, network, volume, credential file, and Unix-socket bridge were
  removed without touching the unrelated host PostgreSQL instance.
- Exact artifact probes passed: the signed presentation has 117 unique node
  IDs, exactly one Level 12 action, one required bounded `option_l`, valid JSON,
  and all seventeen fixtures carry the field while only the dungeon view
  populates it. `git diff --check` passed in both repositories.
- `bin/gate.sh --diff` passed every platform stage through packaging in an
  isolated Bubblewrap network namespace. An initial namespace configuration
  lacked the network/root facilities needed by the gate, and one later run saw
  the pre-existing provider restart callback test report zero callbacks; no
  source changed in response, and the clean rerun passed that stage and the
  entire gate. The matching receipt records state hash
  `825b00a007df59a9849c4400f2f3a4fe3833ec60c344b7a427d7a4934e486879`.
- Repacked cartridge v17 (`rules_version` and `cartridge_version` 17) is now
  visible in exactly one mapped, non-hidden local-play QML window on Hyprland
  workspace 8. Its live entry plan exposes exactly one `continue` action, and
  its signed presentation exposes exactly one `enter_dungeon_level_12` action.
- Phase 4 exit: all six requirements have passing implementation, boundary,
  live-provider, signed-QML, platform-gate, and visible-play evidence. The
  pipeline is ready for completion documentation and archival.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — authenticated v0.20e hash/readback, exact records 110–119,
    source trace, compatibility docs, and fixed-data tests agree;
  - REQ-002 PASS — exact-v17 state, malformed JSON, old schema, unsupported
    level, boundary/unknown record, wrong name/level, and state/RNG immutability
    tests pass;
  - REQ-003 PASS — levels 1–12 switch without draws and zero, thirteen, and
    larger levels reject unchanged;
  - REQ-004 PASS — forced `Random(120)` traces retain rejected record 110,
    accept records 111–119, spend one fight, and prove 20/10/60 combat state;
  - REQ-005 PASS — Level 12 retreat damage plus existing attack, heal, spell,
    class-special, reward, poison, replay, and complete-day regressions pass;
  - REQ-006 PASS — generic/fixed provider equality, exactly one signed Level
    12 control over `option_l`, duplicate-label/delegate and real-input proof,
    restarted live corpus, signed trusted-QML combat path, inspection,
    24-stage platform gate, and workspace-8 launch pass.
- Documentation: the external README, compatibility ledger, Rust port map,
  provenance, fixtures, scripts, platform Game Cartridges architecture, and
  generated OpenWiki quickstart/Game Cartridges pages describe v17/Level 12 and
  retain the non-production boundary. OpenWiki run
  `ba8f350f-62ec-4db8-84d8-50e64694caea` authored the generated update; final
  reconciliation run `ba6a3196-db86-40f7-bae5-0cf50444956b` completed cleanly
  after the hand-maintained architecture update.
- The post-documentation `bin/gate.sh --diff` passed all 24 stages and printed
  `GATE GREEN [diff]`. The gate and OpenWiki completion receipts both bind
  pipeline `4ec0cc25-d550-41c3-87a3-57b934c3c8d6` to gated state
  `11143f81e11935483982631dda60018b3cee46b62def1d736a4a3b69a3c72f4a`.
  Its disposable container, volume, network, socket, and relay were removed;
  the unrelated host PostgreSQL remained unchanged.
- AAR 066 is submitted and effective. Existing source-branch, discarded-RNG,
  one-command, and real-input prevention rules were effective; the stale test
  expectation and isolated harness attempts do not warrant duplicate durable
  knowledge IDs or a new ADR.
- Archive: Ticket 066 is closed and this single spec/notes pair is moved to
  `pipeline/completed` with no remaining active pair.
- Phase 5 exit: pipeline complete. Delivery remains unauthorized, so no commit,
  push, pull request, registration, admission, deployment, or publication was
  performed.
