---
title: Usurper Level Eleven Dungeon Band — notes
pipeline_id: af8cdc84-769f-4e5d-a917-6991d5e09209
---

# Usurper Level Eleven Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - `BUL-002-pre-rebuild-delivery-handoff` remains informational: the ignored
    upstream corpus, provider kit, preview state, database state, and workflow
    receipts are local evidence and remain outside source control.
  - Ticket 063 completed exact levels one through ten as rules/state/cartridge
    v15 and retained authenticated next-screen selection, unique visible
    choices, and trusted activation auto-repeat suppression.
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires the generic dungeon calculation and enclosing event/registration
    branches to be read before translating the Level 11 outcome set.
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    record-100 rejection draws to remain observable in the deterministic trace.
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` requires
    the live profile to follow actual post-Level-11 phase/HP state rather than
    assuming the Level 10 sequence still fits.
  - Ticket 062's live-shell rules continue to require multi-screen transition
    proof, unique visible choices, and one activation per intended command.
- Source preflight:
  - authenticated upstream baseline remains the publisher-linked parentless
    commit `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`;
  - Git and source-archive copies of `EDMONST.PAS` are byte-identical with
    SHA-256 `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - `EDMONST.PAS` lines 3615–3715 define records 100–109 as Medusas First Head,
    Medusas Second Head, Gnoll, Gnoll Chief, two Evil Gnoll records, Medusas
    Third Head, Catholic Noble, Protestant Noble, and Deathbringer, all with
    base strength 20;
  - `DUNGEONC.PAS` lines 880–925 route events separately, while lines 937–955
    reset/load ordinary monsters and repeat `Random(level*10)` until the result
    exceeds `(level-1)*10`; Level 11 therefore retains record 100 but normally
    selects 101–109 through `Random(110)`;
  - the unregistered-release guards apply only above dungeon levels 89/90, not
    to Level 11; Level 11 remains on the ordinary supported source branch;
  - `PLVSMON.PAS` lines 68–98 use `Random(global_dungeonlevel*10)+3` for failed
    retreat damage, and lines 603–625 initialize loaded monsters to
    strength-times-three HP.
- Existing boundary fit:
  - `GameView` and its signed schema already require bounded `option_k`, which
    is unused on the dungeon screen and can carry exactly one Level 11 control;
  - the provider already decodes generic level commands and fixed level
    actions, and the trusted renderer remains game-neutral.
- Baseline:
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system ./scripts/test.sh`
    passed before implementation: formatting, warning-denying Clippy, all 83
    Rust tests, rustdoc, immutable source/provenance and privacy checks,
    seventeen-screen signed-QML smoke, duplicate-label and activation-repeat
    checks, and provider-backed local-play smoke.
- Decision: implement Level 11 as the next normal-dungeon band; defer Level 12,
  dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and data flow:
  - `usurper-data` owns the exact immutable Level 11 editor records;
  - `usurper-rules` remains the only game authority for draw-free level
    switching, rejection-loop selection, combat, state validation, and
    deterministic RNG;
  - `usurper-provider` decodes both generic `enter_dungeon` input and the fixed
    `enter_dungeon_level_11` action, then projects the ordinary bounded view;
  - the signed inert cartridge binds the already-required `option_k` view
    field to one Level 11 button, leaving the dungeon screen inside existing
    node and text limits;
  - OmarchyGS continues to authenticate the signed zero-payload action, broker
    it to the exact external provider release, and render only the resulting
    typed plan. It does not gain Usurper rules or state.
- The separate Usurper repository has no `.codegraph` index, so its Rust, JSON,
  and shell contracts were inspected directly. Platform CodeGraph exploration
  traced `RenderedNode::Button`, signed action validation, provider session
  dispatch/revision ownership, QML control construction, and the existing
  duplicate-label and trusted-input auto-repeat regressions. Adding one fixed
  zero-payload action over existing `option_k` requires no platform
  application, schema, database, or migration change. Design receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `af8cdc84-769f-4e5d-a917-6991d5e09209`, state hash
  `34c9b578d73a6cd19f1f9159935487b8a22c4faac201d7421213baa87df5d6a5`.
- API/state and compatibility contract:
  - advance external state, rules, and cartridge identity from v15 to v16 and
    accept exact v16 only; v15 serialized state is not silently relabeled;
  - accept generic dungeon levels 1–11 and fixed actions 1–11; reject zero,
    twelve, and larger levels without state or RNG changes;
  - require an active monster to belong to the selected implemented band,
    match the exact source name, retain bounded scalars, and exclude each
    normally unreachable band-boundary record;
  - retain record 100 as immutable source data while normal Level 11 selection
    accepts only records 101–109 after every source-order `Random(110)` draw;
  - preserve Provider SDK/protocol v1, game key, provider ID, player-private
    state shape, and the established seventeen-screen presentation protocol.
- Exact canonical Level 11 data contract:

  | Index | Name | Base strength | Armor user | Weapon user | Normal Level 11 selection |
  |---:|---|---:|---|---|---|
  | 100 | Medusas First Head | 20 | no | no | no — boundary record |
  | 101 | Medusas Second Head | 20 | no | no | yes |
  | 102 | Gnoll | 20 | yes | yes | yes |
  | 103 | Gnoll Chief | 20 | yes | yes | yes |
  | 104 | Evil Gnoll | 20 | yes | yes | yes |
  | 105 | Evil Gnoll | 20 | yes | yes | yes |
  | 106 | Medusas Third Head | 20 | no | no | yes |
  | 107 | Catholic Noble | 20 | yes | yes | yes |
  | 108 | Protestant Noble | 20 | yes | yes | yes |
  | 109 | Deathbringer | 20 | no | yes | yes |

- Database and migration consequences: none in OmarchyGS. The external starter
  retains its separate PostgreSQL state and operation receipts; strict v16
  identity uses fresh development sessions rather than rewriting v15 rows.
- Planned external-provider file manifest:
  - `crates/usurper-data/src/lib.rs` — records 100–109, lookup, and exact data
    regressions;
  - `crates/usurper-rules/src/lib.rs` — v16 bounds, projection, encounter,
    retreat, state-hostility, and reducer regressions;
  - `crates/usurper-provider/src/lib.rs` — fixed action plus generic/fixed,
    replay, view, and live-profile coverage;
  - `cartridge/manifest.json`, `cartridge/presentation.json` — v16 identity and
    exactly one inert Level 11 control;
  - `fixtures/presentation/dungeon.json`,
    `fixtures/presentation/combat.json` — signed Level 11 render facts;
  - `provenance/source-trace.json` — source-to-Rust Level 11 evidence;
  - `scripts/play.sh`, `scripts/test.sh`, `scripts/test-provider.sh` — visible
    Level 11 path, gate label, and exact v16 restart/replay assertions;
  - `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` — current
    scope, compatibility ledger, and port milestone.
- Platform files remain limited to Ticket 064 lifecycle/completion evidence.
  No server, SDK, QML, renderer vocabulary, database, route, migration,
  packaging, admission, deployment, or publication source change is designed.
- Regression table:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Exact ten-row order/name/strength/flag tests, authenticated source hashes, source-trace validation, and compatibility review. |
  | REQ-002 | Exact-v16 schema test; unsupported level, boundary/unknown record, wrong name/level, malformed JSON, and complete state/RNG immutability checks. |
  | REQ-003 | Draw-free level 1–11 transitions, visible labels, location/phase/monster assertions, and zero/twelve/max rejection. |
  | REQ-004 | Forced rejected/accepted `Random(110)` trace, record-100 exclusion, record 101–109 bound, 20/10/60 combat state, fight spend, and deterministic twins. |
  | REQ-005 | Exact failed-retreat `(2, 110)` trace plus existing attack, heal, spell, class-special, reward, poison, and complete-day regressions. |
  | REQ-006 | Generic/fixed provider equality and replay, unique signed label/action, all-screen QML smoke, one-click/one-revision path, live corpus twice across restart, platform diff gate, scope/security review, and provider-backed workspace-8 play. |

- Risk, privacy, concurrency, reconnect, and rollback review:
  - wrong roster order/flags, normal exposure of record 100, or accidentally
    deriving strength 21 from the level are covered by exact arrays, boundary
    lookup tests, forced rejection traces, and explicit 20/10/60 assertions;
  - duplicate `Evil Gnoll` rows and exact `Medusas` spelling are preserved as
    source identity rather than normalized away;
  - an eleventh control can regress layout or dispatch twice, so signed-screen
    conformance, label uniqueness, auto-repeat suppression, local click proof,
    and visible workspace-8 play all remain release gates;
  - Level 11 RNG can alter subsequent retreat/death outcomes, so the live
    driver must follow authenticated phase and HP instead of assuming the old
    Level 10 sequence;
  - provider revision checks and deterministic replay cover duplicate or
    reordered activation across reconnect/restart; no new shared mutable state
    or concurrent writer is introduced;
  - no new identity, credential, personal data, network listener, production
    configuration, database, executable-code, or shared-realm surface is added;
  - strict v16 prevents mixed-schema interpretation; rollback is the unchanged
    v15 release/session identity, not an in-place state conversion.
- Alternatives rejected:
  - inferring the roster from level arithmetic would lose exact row order,
    duplicate identities, flags, spelling, and boundary-record proof;
  - accepting record 100 because it appears in the editor table would contradict
    the ordinary dungeon rejection loop;
  - combining Level 12 or dungeon events would cross separate content and
    control-flow boundaries and weaken the proof for this slice;
  - adding a new view property, platform rule, or bespoke renderer control
    would widen contracts despite available `option_k` capacity.
- Phase 2 exit: the authenticated source contract, architecture, compatibility
  behavior, exact file manifest, regression plan, risks, and worktree-bound
  CodeGraph evidence are actionable.

## Phase 3 — Implement

- Built:
  - added exact records 100–109 to `usurper-data`, preserving source order,
    exact `Medusas` spelling, both distinct `Evil Gnoll` records, base strength
    20, equipment flags, and lookup boundaries;
  - advanced rules/state identity to v16, accepted dungeon levels one through
    eleven, projected Level 11 through existing `option_k`, and retained every
    rejected `Random(110)` draw until record 101–109 selection with strength
    20, defence 10, and 60 HP;
  - added draw-free Level 11 switching plus encounter, deterministic replay,
    boundary, retreat-damage, hostile-state, generic/fixed provider, view, and
    live-profile regressions without changing lower-level combat;
  - added strict fixed dungeon-action decoding, the
    `enter_dungeon_level_11` provider action, and exactly one signed inert
    Level 11 dungeon button;
  - advanced rules/cartridge identity and fixtures to v16 and updated
    provenance, local play, provider corpus, scope, compatibility, and port-map
    documentation.
- Focused and complete implementation proof:
  - `cargo test -p usurper-data -p usurper-rules -p usurper-provider
    --all-features` passed the data/rules/provider tests after the initial
    implementation;
  - `scripts/test-cartridge.sh` passed all seventeen signed screens, duplicate
    visible-label rejection, and trusted-QML auto-repeat smoke;
  - `scripts/test-local-play.sh` passed Entry → creation → Main Street → Level
    1 → Level 11 → combat through authenticated provider-backed HTTP/QML state,
    with each expected revision advancing exactly once;
  - the exact v16 provider corpus passed all fifteen TLS/replay/fault/callback
    cases twice across provider restart against a temporary isolated
    PostgreSQL container;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    ./scripts/test.sh` passed formatting, warning-denying Clippy, all 88 Rust
    tests, rustdoc, immutable source/provenance and privacy assertions, signed
    cartridge conformance, duplicate-label and activation-repeat checks, and
    provider-backed local-play smoke;
  - JSON parsing, shell syntax, and `git diff --check` passed.
- Two implementation corrections were made from actual test evidence:
  - the host's system PostgreSQL already occupied port 5432, so the provider
    test harness now starts the platform Compose database only when no explicit
    securely loaded admin-URL file is supplied; the corpus ran against an
    ephemeral database on a separate loopback port and that container and
    credential file were removed afterward;
  - the Level 11 deterministic path leaves the test Cleric at 2 HP after Cure
    Light and kills the player on the first failed retreat. The live corpus now
    authenticates `Dead`, performs exactly one `reenter`, and continues instead
    of issuing the stale Level 10 second-combat sequence that correctly
    returned HTTP 422;
  - adding an eleventh inline fixed-action arm crossed the repository's
    100-line Clippy limit, so strict exact action matching was factored into
    `fixed_dungeon_level` rather than suppressing the warning.
- Implementation stayed within the Phase 2 external manifest. No platform
  server, SDK, protocol, QML, renderer vocabulary, route, database, migration,
  packaging, admission, deployment, delivery, or publication source changed.
- Phase 3 exit: the v16 external-provider slice and focused evidence are ready
  for independent inspection.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Source/rules correctness | Exact records 100–109, record-100 rejection, bounded `Random(110)` trace, v16 hostile-state handling, and Level 11 combat arithmetic were internally consistent and covered by focused tests. | None | No change required. |
| 2 | Provider/revision behavior | Generic and fixed Level 11 actions converge on one reducer command; strict action decoding, idempotency, expected revisions, render-before-commit behavior, and restart replay preserve one accepted state transition per activation. | None | No change required. |
| 3 | Signed presentation/QML controls | The seventeen-screen declaration has unique node IDs and action bindings, exactly one Level 11 choice, bounded empty payloads, and existing duplicate-label plus trusted auto-repeat regressions. | None | No change required. |
| 4 | Secrets/network/test lifecycle | The explicit PostgreSQL admin URL is read from a caller-supplied secure file, is not emitted, and suppresses only test-owned Compose startup; the isolated test container and temporary credential file were removed. | None | No change required. |

- Post-implementation CodeGraph exploration retraced the current platform
  path through signed action definitions, render-plan `Button` nodes,
  provider broker input validation, expected revisions, and trusted QML
  activation controls. No Level 11 platform-rule or renderer change was
  present. Inspect receipt:
  `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`, pipeline
  `af8cdc84-769f-4e5d-a917-6991d5e09209`, state hash
  `34c9b578d73a6cd19f1f9159935487b8a22c4faac201d7421213baa87df5d6a5`.
- Codex Security diff scan
  `65d7e259-877b-40c4-881a-7b6345f296f7` inspected all fifteen frozen
  implementation surfaces against baseline
  `bb31caa122de669d72a265860b19969fcd28505f`, reported zero findings, and
  completed its full 15/15 worklist. Threat-model coverage included player
  input, signed presentation, provider command/replay, local test capability,
  data/provenance, and shell lifecycle boundaries. The optional Trust and
  Authorization Context connector was unavailable, so no TAC-only output is
  claimed; repository and scan evidence were otherwise complete.
- Phase 3.5 exit: no blocking, high, medium, low, or advisory finding remained;
  the lead disposition is PASS for validation.

## Phase 4 — Validate

- Tests run:
  - final `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    ./scripts/test.sh` passed in the external repository: formatting, strict
    Clippy, all 88 Rust tests, rustdoc, immutable upstream/provenance and
    privacy checks, all seventeen signed screens, provider-backed local play,
    duplicate-label checks, and trusted input metrics `expected=1`,
    `exercised=1`, `repeats_blocked=1`, `focus=true`;
  - the exact v16 fifteen-case TLS/replay/fault/callback provider corpus had
    already passed twice across restart against an isolated PostgreSQL
    instance, and no implementation source changed afterward;
  - the fresh development app was rebuilt, the stale preview was terminated,
    and Hyprland readback verified the new QML window visible and accepting
    input on workspace 8. The desktop was locked at final visual inspection,
    so no keystroke was injected into the password prompt; the equivalent
    current-build QML action path was exercised by the trusted smoke above.
- Gate run:
  - `TMPDIR=/tmp bin/gate.sh --fast` passed every source, contract, renderer,
    packaging-source, architecture, secret, whitespace, hook, and module
    check;
  - `bin/gate.sh --diff` passed the same code checks plus deterministic native
    package generation, but ended red because seven database drills could not
    own their hard-coded `127.0.0.1:5432` endpoint and the first hook run used
    the host's non-`/tmp` `TMPDIR`. The hook self-test passed when given its
    documented `/tmp` cleanup domain; the database collision is unrelated to
    this external-only game slice.
- Skips or pre-existing failures:
  - the host's independently running system PostgreSQL owns port 5432 and does
    not contain the platform test role. It was deliberately not stopped or
    modified. This prevented a green database-bearing platform receipt, while
    the Level 11 provider corpus itself passed on a separate loopback port and
    cleaned up its isolated container and credential file;
  - no production registration, admission, deployment, publication, commit,
    or push was attempted.
- Phase 4 exit: all Level 11 implementation and UI regression evidence passed;
  the only full-platform failures are documented host-environment collisions
  outside this ticket's unchanged platform/database surface.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — authenticated v0.20e source hashes, exact records 100–109,
    provenance, compatibility docs, and data tests agree;
  - REQ-002 PASS — exact-v16 hostile state, malformed JSON, wrong
    level/name, boundary, unknown-record, and state/RNG immutability tests pass;
  - REQ-003 PASS — levels 1–11 switch without draws and zero, twelve, and
    larger levels reject unchanged;
  - REQ-004 PASS — forced `Random(110)` traces retain rejected record 100,
    accept records 101–109, spend one fight, and prove 20/10/60 combat state;
  - REQ-005 PASS — Level 11 retreat damage plus existing attack, heal, spell,
    class-special, reward, poison, replay, and complete-day regressions pass;
  - REQ-006 PASS — generic/fixed provider equality, one signed Level 11
    control, duplicate-label rejection, one activation/revision, restart
    corpus, trusted-QML combat path, inspection, and workspace-8 launch pass.
- Docs: the external README, compatibility ledger, Rust port map, provenance,
  fixtures, and scripts describe v16/Level 11. OpenWiki quickstart and Game
  Cartridges pages now carry the same milestone and retained non-production
  boundary. `openwiki_finish` returned `status: complete`; both pages retained
  an explicit warning for their pre-existing unresolved Claims debt.
- AAR: `AAR-064-usurper-level-eleven-dungeon-band` is submitted and marks the
  recalled branch, discarded-RNG, composite-driver, and visible-control rules
  effective. No duplicate standing rule or new ADR was created.
- Archive: Ticket 064 is closed, the ticket index is advanced to an empty open
  queue, and this single spec/notes pair is moved to `pipeline/completed`.
- Phase 5 exit: pipeline complete. Delivery remains unauthorized, so no commit,
  push, pull request, deployment, admission, registration, or publication was
  performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Default provider-corpus setup could not bind port 5432. | A host PostgreSQL service began occupying the Compose-published port. | Honor the existing secure admin-URL-file override without starting Compose; validate against an isolated loopback database. | Keep database selection explicit and never stop an unrelated listener to make a test pass. |
| 2 | The first v16 corpus returned HTTP 422 during its second scripted encounter. | The Level 11 `Random(110)` path killed the Cleric on the first failed retreat, while the old Level 10 driver assumed a Dungeon phase. | Prove the actual phase/HP in a provider regression and replace the stale command with one `reenter`. | Composite drivers must follow authenticated provider state after every changed RNG band. |
| 3 | Full Clippy rejected `decode_command` at 101 lines. | One additional fixed action pushed an already-large exact-action match past the enforced limit. | Extract strict fixed dungeon-action matching into `fixed_dungeon_level`. | Keep bounded action families in small exact decoders rather than growing one dispatcher indefinitely. |
