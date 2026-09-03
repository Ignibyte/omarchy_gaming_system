---
title: Usurper Level-Seven Dungeon Band — notes
pipeline_id: b2381524-f89d-45f6-822e-85c0cda31800
---

# Usurper Level-Seven Dungeon Band — running notes

## Phase 1 — Recall and plan

- The user asked to continue building the established Usurper port. No active
  pipeline, open ticket, or blocking bulletin existed, so Ticket 059 takes the
  next source-complete dungeon band without broadening authority or release
  scope.
- `BUL-002-pre-rebuild-delivery-handoff` was acknowledged. The rebuilt platform
  and sibling Usurper repositories are clean on `main`; the ignored v0.20e
  upstream corpus, local provider kit, and build output are present. Pipeline
  tools are ready and PostgreSQL is healthy.
- Recalled knowledge:
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires preserving the stored boundary row independently from normal
    reachability;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps the slice in solo non-classic normal dungeon combat;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001`
    makes every rejected `Random(70)` result observable deterministic behavior;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`
    requires replaying the complete provider profile after the new earlier RNG
    work changes later outcomes;
  - `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001`
    remains applicable to the provider conformance credential boundary;
  - Ticket 058 supplies the rules-v11 levels-one-through-six implementation,
    seventeen-screen cartridge, clean security scan, and visible baseline.
- Canonical v0.20e readback establishes ten Level 7 editor records at indices
  60–69 and the following exact base fixture values:

  | Index | Name | Base strength | Armor user | Weapon user | Normal Level 7 selection |
  |---:|---|---:|---|---|---|
  | 60 | Large Snake | 17 | no | no | unreachable boundary |
  | 61 | Orc | 17 | yes | yes | accepted |
  | 62 | Sword Champion | 17 | yes | yes | accepted |
  | 63 | Orc Lieutenant | 17 | yes | yes | accepted |
  | 64 | Stone Elf | 17 | yes | yes | accepted |
  | 65 | Uruk-Hai | 17 | yes | yes | accepted |
  | 66 | Gnoll | 17 | no | yes | accepted |
  | 67 | Monk | 17 | no | yes | accepted |
  | 68 | Wizard | 17 | no | yes | accepted |
  | 69 | Lion | 17 | no | no | accepted |

- Source anchors:
  - `SOURCE/EDITOR/EDMONST.PAS:3209-3309` declares the Level 7 rows;
  - `SOURCE/EDITOR/ADDMONST.PAS:43-72` distinguishes editor base strength from
    initialized-world randomization, matching the existing development fixture
    policy;
  - `SOURCE/USURPER/DUNGEONC.PAS:868-955` spends a fight and repeats
    `Random(level*10)` until the candidate exceeds `(level-1)*10`;
  - `SOURCE/USURPER/PLVSMON.PAS:603-625` sets HP to strength times three;
  - `SOURCE/USURPER/PLVSMON.PAS:68-138` makes failed retreat damage use
    `Random(global_dungeonlevel*10)+3`.
- Phase 1 exit: scope, six EARS requirements, six locked decisions, Ticket 059,
  pipeline UUID `b2381524-f89d-45f6-822e-85c0cda31800`, and open AAR are settled.

## Phase 2 — Design

- Architecture and data flow:
  - `usurper-data` owns the exact immutable Level 7 editor fixtures;
  - `usurper-rules` remains the sole game authority for level switching,
    rejection-loop selection, combat, validation, and deterministic RNG;
  - `usurper-provider` decodes only the generic `enter_dungeon` and fixed
    `enter_dungeon_level_7` forms, then returns the ordinary bounded view;
  - the signed cartridge binds the existing `option_g` view field to one inert
    `enter_dungeon_level_7` button; the trusted renderer already admits this
    node shape and the resulting dungeon screen remains far below its node
    budget;
  - OmarchyGS continues authenticating and translating the signed zero-payload
    action, brokering it to the exact registered provider release, and rendering
    the returned typed plan without Usurper-specific rules or state.
- CodeGraph traced `ValidatedSessionCartridgeAction`, registered-provider
  command translation, `ProviderGame::command`/`view`, and `RenderPlan`. It
  confirms that the action schema remains cartridge-owned, the provider game
  receives neither platform identity nor credentials, and the generic renderer
  consumes only bounded nodes. Design receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `b2381524-f89d-45f6-822e-85c0cda31800`, state hash
  `6338bf4bde9e1d901ebfd644f203f8327f2ade170b188a879160e92052687936`.
- API/state and compatibility contract:
  - advance external state, rules, and cartridge identity from v11 to v12 and
    accept exact v12 only; no v11 state is silently migrated;
  - accept generic `enter_dungeon` levels 1–7 and fixed level actions 1–7;
    reject 0, 8, and larger unchanged and without RNG work;
  - require active monsters to belong to the selected implemented band, match
    the exact source-linked name, retain bounded scalars, and exclude every
    normally unreachable boundary record; encounter initialization still uses
    the exact reviewed base-strength fixture;
  - retain record 60 in immutable data while normal Level 7 selection accepts
    only 61–69 after all source-order `Random(70)` rejections;
  - preserve Provider SDK/protocol v1, the existing game key, provider ID,
    player-private state shape, and seventeen-screen presentation protocol.
- Database and migration consequences: none in OmarchyGS. The external starter
  continues owning its independent PostgreSQL state and operation receipts;
  strict v12 identity means validation uses fresh development sessions rather
  than mutating v11 rows.
- Planned implementation files in the external provider:
  - `crates/usurper-data/src/lib.rs` — exact records 60–69, lookup, and data
    tests;
  - `crates/usurper-rules/src/lib.rs` — v12 validation, level selection/view,
    encounter/retreat behavior, and reducer regressions;
  - `crates/usurper-provider/src/lib.rs` — fixed action plus generic/fixed and
    replay coverage;
  - `cartridge/manifest.json`, `cartridge/presentation.json` — exact v12
    identity and inert Level 7 control;
  - `fixtures/presentation/dungeon.json`,
    `fixtures/presentation/combat.json` — signed Level 7 render facts;
  - `provenance/source-trace.json` — source-to-Rust Level 7 evidence;
  - `scripts/test.sh`, `scripts/test-provider.sh` — human-readable gate label,
    exact v12 live profile, and composite replay assertions;
  - `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` — current
    scope, compatibility ledger, and port milestone.
- Platform changes remain limited to Ticket 059 planning, architecture/wiki
  reconciliation, and completion evidence. No server, SDK, QML, route,
  database, migration, or renderer-vocabulary source change is designed.
- Regression table:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Exact ten-row data/order/flag test, source hashes, source-trace validator, and compatibility review. |
  | REQ-002 | v12 exact-schema test; unsupported level, boundary/unknown record, wrong name, oversized scalar, malformed JSON, and state/RNG immutability checks. |
  | REQ-003 | Draw-free level 1–7 transitions, visible labels, phase/location/monster checks, and 0/8/max rejection. |
  | REQ-004 | Forced rejected/accepted `Random(70)` trace, record 60 exclusion, records 61–69 bound, 17/8/51 combat state, fight spend, and deterministic twin. |
  | REQ-005 | Exact `(2, 70)` retreat trace plus existing attack, quick-heal, spell, class-special, reward, poison, and complete-day suite. |
  | REQ-006 | Generic/fixed provider equality and replay, signed cartridge conformance, all-screen QML smoke, fixed live corpus twice across restart, platform diff gate, scope/security review, and visible Level 7 preview. |

- Risk and rollback review:
  - wrong roster order/flags or exposing record 60 is covered by exact arrays,
    lookup boundaries, and forced rejection traces;
  - added encounter RNG can shift later retreat/death profile outcomes, so the
    complete live command driver must be replayed and adjusted only to match the
    deterministic source-composed phases;
  - a seventh dungeon control could regress layout or action declaration, so
    the signed all-screen QML smoke and visible preview must exercise it;
  - strict v12 prevents mixed-schema interpretation; rollback is the unmodified
    v11 release/session identity, not in-place state conversion;
  - no new identity, credential, network, database, executable-content, shared
    realm, or platform-authority surface is introduced.
- Rebuilt-machine baseline evidence:
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system scripts/test.sh`
    passed before implementation: formatting, warning-denying Clippy, 61 Rust
    tests, rustdoc, immutable upstream hashes, provenance/privacy assertions,
    signed cartridge conformance, and all seventeen trusted-QML screens;
  - the first live provider baseline did not start the test corpus because
    Compose attempted its documented host port 5432 while a rebuilt-machine
    PostgreSQL service already owned that port. The failure recreated the
    project DB container in `Created` state and changed no repository files.
    Phase 3 may use a private temporary Compose port override and descriptor-
    validated admin URL at 55432, then rerun the exact test unchanged.
  - that local-only override was validated and the unchanged
    `scripts/test-provider.sh` then passed the fixed fifteen-case TLS,
    authentication, replay, fault, callback, and reconciliation corpus twice
    across an independent provider restart.
- Phase 2 exit: architecture, compatibility/state contract, exact file
  manifest, regression mapping, risks, and CodeGraph evidence are actionable.

## Phase 3 — Implement

- Added exact Level 7 rows 60–69 to `usurper-data`, including source spelling,
  reviewed base strength 17, equipment flags, lookup coverage, and explicit
  record-60/record-69 table assertions.
- Advanced rules identity to v12. State and commands now accept levels 1–7,
  the dungeon view exposes `option_g`, and encounter selection preserves every
  rejected `Random(70)` result until records 61–69 are selected. Level 7
  monsters initialize at strength 17, defence 8, and 51 HP.
- Added Level 7 rejection-trace, deterministic-twin, boundary, draw-free
  switching, retreat-bound, hostile-state, generic/fixed provider, view, and
  replay regressions while retaining the complete lower-level suite.
- Added the fixed `enter_dungeon_level_7` provider decoder, rules/cartridge v12
  identity, signed inert dungeon button/action, Level 7 fixtures, provenance,
  live-profile selection, and compatibility/port-map descriptions.
- Focused proof after formatting:
  - `cargo test -p usurper-data`: 9 passed;
  - `cargo test -p usurper-rules`: 41 unit plus 1 integration passed;
  - `cargo test -p usurper-provider`: 15 passed, including the Level 7 live
    death/re-entry profile;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    scripts/test-cartridge.sh`: all seventeen signed screens and trusted-QML
    state smoke passed;
  - `git diff --check`: passed in both repositories.
- Implementation stayed within the Phase 2 manifest. The historical rebuild
  handoff remains an accurate record of the delivered Ticket 058 checkpoint
  and was not rewritten as current development documentation.
- Phase 3 exit: the approved implementation and focused evidence are ready for
  independent inspection.

## Phase 3.5 — Inspect

- Canonical source and the implemented diff were reread after focused tests.
  The Level 7 table remains records 60–69, normal encounter selection remains
  repeated `Random(70)` until the result is greater than 60, HP remains three
  times reviewed base strength, and retreat damage remains
  `Random(level*10)+3`.
- Complete-file security review covered the reducer, provider adapter, data
  tables, manifest, presentation, provenance, and test scripts; the changed
  documentation and fixtures were also reconciled. One candidate asked whether
  a forged stored monster could retain a canonical name/index while changing
  bounded combat scalars. Targeted validation traced the pinned starter and
  proved that authenticated command payloads never supply current state: all
  stored rows originate only from `game.launch` or the prior validated
  `game.command` transition. The required direct database/provider-host write
  is an excluded privileged compromise, so the candidate was suppressed with
  no code change.
- The final sealed Codex Security working-tree report, refreshed after the
  test-harness portability fix, completed with zero reportable findings and
  complete coverage:
  `/mnt/fast/tmp/codex-security-scans/omarchygs_usurper/bb31caa_worktree_20260902T182753Z/report.md`.
  TAC access could not be verified because its connector was not connected;
  the local source-backed review and canonical artifacts completed normally.
- Independent architecture review agreed on the signed-TLS → starter-owned
  identity/replay/store → strict adapter → deterministic reducer boundary. It
  also identified the platform-owned presentation/navigation boundary and the
  operator configuration/database/TLS/callback assets. The old v11 language in
  `docs/REBUILD_HANDOFF.md` remains intentional historical Ticket 058 evidence;
  current compatibility documentation correctly lists boundary records 10, 20,
  30, 40, 50, and 60 exactly once.
- Fresh post-implementation CodeGraph inspection traced the signed zero-payload
  action through session-cartridge translation, registered-provider transport,
  provider view decoding, render-plan construction, and trusted QML dispatch.
  No platform source, SDK, route, migration, persistence, QML, or renderer-
  vocabulary change is required. Receipt:
  `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`, pipeline
  `b2381524-f89d-45f6-822e-85c0cda31800`, state hash
  `6338bf4bde9e1d901ebfd644f203f8327f2ade170b188a879160e92052687936`.
- Phase 3.5 exit: every inspection hypothesis is disposed, the external diff
  remains within the approved manifest, and no finding requires a code change.

## Phase 4 — Validate

- External `scripts/test.sh` passed the final v12 patch: formatting,
  warning-denying Clippy, 9 data tests, 41 rules unit tests plus 1 integration
  test, 15 provider tests, rustdoc, immutable upstream/source-trace checks,
  privacy assertions, signed cartridge conformance, and all seventeen trusted-
  QML screens.
- The first post-change live provider attempt returned 401 because this rebuilt
  machine exports `CARGO_TARGET_DIR=/mnt/fast/target`: Cargo compiled v12 there,
  but the script launched the stale v11 binary from `<repo>/target`. The script
  now resolves the platform and Usurper target directories through exact-
  manifest `cargo metadata`, requires absolute nonempty paths, and quotes both
  executable operands. `bash -n` passed, and the unchanged fixed fifteen-case
  TLS/authentication/replay/fault/callback/reconciliation corpus then passed
  twice across an independent provider restart.
- The first full platform gate exposed three rebuilt-machine assumptions rather
  than product defects: project PostgreSQL could not own the documented host
  port 5432, four existing drills hard-code that port, hook cleanup expects a
  `/tmp`-rooted temporary directory, and the Door Legends clean-clone drill
  assumes Cargo's default local target directory. Validation used a temporary
  Compose mapping to 55432, a process-local loopback-only 5432-to-55432 connect
  adapter, `TMPDIR=/tmp`, and an environment with `CARGO_TARGET_DIR` unset. It
  did not stop or alter the system PostgreSQL service and made no platform code
  change.
- With those explicit test-environment conditions, `bin/gate.sh --diff` passed
  every current stage and printed `GATE GREEN [diff]`. Receipt
  `.git/omarchy-gaming-system-gate-receipt` matches state hash
  `6338bf4bde9e1d901ebfd644f203f8327f2ade170b188a879160e92052687936`.
- Final security snapshot
  `codex-security-snapshot/v1:sha256:903225e5650fa73d7f408c93c06752c78802c88cbd4270ec3c344db83c32b504`
  has complete diff coverage and zero reportable findings. TAC remained
  unavailable because its configured connector was not connected; the sealed
  local canonical artifacts and source-backed validation completed normally.
- Visible acceptance passed with `scripts/show.sh combat`: the production
  trusted QML preview visibly shows signed cartridge v12, `A level 7 Orc
  blocks your way.`, monster HP 51, and the bounded attack, retreat, quick-heal,
  spell, and class-special controls. The process remains open in ignored run
  `.preview/run.mug8Lw` on requested Hyprland workspace 8.
- Requirement audit: REQ-001 through REQ-006 are satisfied by canonical source
  readback, exact fixtures, hostile/deterministic reducer and provider tests,
  replay across restart, signed-cartridge/QML evidence, complete security
  inspection, and the green platform gate. No requirement was narrowed or
  dropped.
- Scope audit: no level eight, composite dungeon event, shared realm, platform
  gameplay rule, protocol, migration, packaging, admission, deployment,
  commit, push, or publication was introduced.
- Phase 4 exits PASS with delivery-gate state hash
  `6338bf4bde9e1d901ebfd644f203f8327f2ade170b188a879160e92052687936`.

## Phase 5 — Complete

- Acceptance criteria: REQ-001 through REQ-006 remain satisfied by exact
  source/data review, hostile and deterministic reducer/provider coverage, the
  full external suite, fixed live provider corpus twice across restart, zero-
  finding final security snapshot, complete platform gate, signed QML smoke,
  and visible Level 7 combat preview. No criterion or exclusion was silently
  dropped.
- OpenWiki update `5344bb55-8e8f-45e1-92f0-9d64b936f557` finished with status
  `complete`. It reconciled `openwiki/quickstart.md` and
  `openwiki/game-cartridges.md` through Ticket 059, rules v12, exact Level 7
  boundary/selection behavior, 17/8/51 combat initialization, and unchanged
  provider/cartridge/trusted-renderer authority. Its warnings are the two broad
  pages' pre-existing unresolved claims debt, not a Ticket 059 verification
  failure.
- Hand-maintained `docs/architecture/game-cartridges.md` now records Tickets
  047–059, rules v12, selectable levels one through seven, boundary record 60,
  accepted records 61–69, and Level 7's 17/8/51 combat initialization.
- AAR-059 is submitted and effective. It registers
  `BF-omarchy-gaming-system-cargo-target-directory-assumption-001` and
  `PR-omarchy-gaming-system-resolve-cargo-artifacts-from-metadata-001`: scripts
  that launch Cargo-built artifacts must resolve the exact manifest's actual
  target directory instead of assuming `<repo>/target`.
- Ticket 059 is closed and the spec/notes pair is archived. Production
  registration, admission, deployment, shared-realm persistence, level eight,
  packaging, public Usurper release, commit, and push remain unauthorized.
- The final post-archival `bin/gate.sh --diff` passed all 24 stages, printed
  `GATE GREEN [diff]`, and wrote a matching gate receipt for state hash
  `4008a486023fe1d3c477a30e4659018caa0a6ff8d4ead8ac55f523c6300a0b07`.
  The worktree-bound OpenWiki completion receipt matches the same state.
- Phase 5 exits PASS. The signed Level 7 preview remains open on workspace 8
  for visible review.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The live provider returned 401 even though the new binary had just built. | Ambient `CARGO_TARGET_DIR` moved v12 artifacts to `/mnt/fast/target`, while the script launched stale v11 output from `<repo>/target`. | Resolve each exact manifest's target directory with structured `cargo metadata`, require an absolute path, and invoke the quoted artifact. | `PR-omarchy-gaming-system-resolve-cargo-artifacts-from-metadata-001`. |
| 2 | The first platform gate had five environment failures; a second had only the analogous Door Legends target-layout failure. | System PostgreSQL owns 5432, existing drills assume that host port, hook cleanup assumes `/tmp`, and one clean-clone script assumes Cargo's default target directory. | Use a temporary Compose 55432 mapping, process-local loopback port adapter, `TMPDIR=/tmp`, and unset ambient `CARGO_TARGET_DIR`; do not alter the unrelated platform scripts in this game slice. | Preserve exact failed-stage evidence and isolate host-specific validation conditions without weakening the tested protocol. |
