---
title: Usurper Level Ten Dungeon Band — notes
pipeline_id: ec0e8c0b-a5a8-4ea8-b484-2a9adaa3b7f6
---

# Usurper Level Ten Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - `BUL-002-pre-rebuild-delivery-handoff` remains informational: the ignored
    upstream corpus, provider kit, preview keys, database state, and workflow
    receipts are local evidence and remain outside source control.
  - Ticket 062 completed exact levels one through nine as rules/state/cartridge
    v14 and repaired authenticated next-screen selection, duplicate visible
    choices, and trusted activation auto-repeat in provider-backed play.
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires the generic dungeon calculation and enclosing event/registration
    branches to be read before translating the Level 10 outcome set.
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    record-90 rejection draws to remain observable in the deterministic trace.
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` requires
    the live profile to follow the actual post-Level-10 phase/HP trace rather
    than assuming the Level 9 command sequence still fits.
  - Ticket 062's live-shell rules require multi-screen transition proof,
    unique visible choices, and one activation per intended command.
- Source preflight:
  - authenticated upstream baseline remains the publisher-linked parentless
    commit `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`;
  - Git and source-archive copies of `EDMONST.PAS` are byte-identical with
    SHA-256 `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - `EDMONST.PAS` lines 3514–3613 define records 90–99 as Orc Lord, Grifter,
    Surpriser, Dark Ranger, Silver Mage, Medusa, Dripper, two distinct
    Mercenary records, and Bounty Hunter, all with base strength 20;
  - `DUNGEONC.PAS` lines 880–925 route events separately, while lines 937–955
    reset/load ordinary monsters and repeat `Random(level*10)` until the result
    exceeds `(level-1)*10`; Level 10 therefore retains record 90 but normally
    selects 91–99 through `Random(100)`;
  - the unregistered-release guard applies only above dungeon level 89/90,
    not to Level 10; Level 10 remains on the ordinary supported source branch;
  - the established `PLVSMON.PAS` trace initializes loaded monsters to
    strength-times-three HP and uses `Random(global_dungeonlevel*10)+3` for
    failed-retreat damage.
- Baseline:
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system ./scripts/test.sh`
    passed before implementation: formatting, warning-denying Clippy, all 79
    Rust tests, rustdoc, immutable source/provenance and privacy checks,
    seventeen-screen signed-QML smoke, and provider-backed local-play smoke.
- Decision: implement Level 10 as the next normal-dungeon band; defer Level 11,
  dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and data flow:
  - `usurper-data` owns exact immutable Level 10 editor fixtures;
  - `usurper-rules` remains the sole game authority for level switching,
    rejection-loop selection, combat, validation, and deterministic RNG;
  - `usurper-provider` decodes the generic `enter_dungeon` form and the fixed
    `enter_dungeon_level_10` action, then returns the ordinary bounded view;
  - the signed cartridge binds existing `option_j` to exactly one inert Level
    10 button; `GameView` and `schemas/view.schema.json` already define this
    required bounded field, and the dungeon screen remains below node limits;
  - OmarchyGS continues authenticating the signed zero-payload action,
    brokering it to the exact provider release, and rendering the returned
    typed plan without Usurper-specific rules or state.
- CodeGraph cannot inspect the separate Usurper repository because it has no
  `.codegraph/` index, so its Rust and JSON contracts were inspected directly.
  The required platform exploration traced cartridge action validation,
  provider session execution, binding resolution, and `RenderedNode::Button`
  consumption. A zero-payload provider-owned action and existing string view
  field require no platform application, schema, database, or migration
  change. Design receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `ec0e8c0b-a5a8-4ea8-b484-2a9adaa3b7f6`, state hash
  `797bd8ed751f93a3e624a3692a2f43fc45ee373ea86381739e00d6dbb303cb2f`.
- API/state and compatibility contract:
  - advance external state, rules, and cartridge identity from v14 to v15 and
    accept exact v15 only; no v14 state is silently migrated;
  - accept generic `enter_dungeon` levels 1–10 and fixed level actions 1–10;
    reject 0, 11, and larger unchanged and without RNG work;
  - require an active monster to belong to the selected implemented band,
    match the source-linked name, retain bounded scalars, and exclude every
    normally unreachable boundary record;
  - retain record 90 in immutable data while normal Level 10 selection accepts
    only records 91–99 after every source-order `Random(100)` rejection;
  - preserve Provider SDK/protocol v1, game key, provider ID, player-private
    state shape, and seventeen-screen presentation protocol.
- Exact canonical Level 10 data contract:

  | Index | Name | Base strength | Armor user | Weapon user | Normal Level 10 selection |
  |---:|---|---:|---|---|---|
  | 90 | Orc Lord | 20 | yes | yes | no — boundary record |
  | 91 | Grifter | 20 | no | no | yes |
  | 92 | Surpriser | 20 | no | no | yes |
  | 93 | Dark Ranger | 20 | yes | yes | yes |
  | 94 | Silver Mage | 20 | no | no | yes |
  | 95 | Medusa | 20 | yes | no | yes |
  | 96 | Dripper | 20 | no | no | yes |
  | 97 | Mercenary | 20 | yes | yes | yes |
  | 98 | Mercenary | 20 | yes | yes | yes |
  | 99 | Bounty Hunter | 20 | yes | yes | yes |

- Database and migration consequences: none in OmarchyGS. The external starter
  keeps its independent PostgreSQL state and operation receipts; strict v15
  identity uses fresh development sessions rather than mutating v14 rows.
- Planned external-provider file manifest:
  - `crates/usurper-data/src/lib.rs` — rows 90–99, lookup, and data tests;
  - `crates/usurper-rules/src/lib.rs` — v15 bounds, projection, encounter,
    retreat, hostile-state, and reducer regressions;
  - `crates/usurper-provider/src/lib.rs` — fixed action and generic/fixed,
    replay, view, and live-profile coverage;
  - `cartridge/manifest.json`, `cartridge/presentation.json` — v15 identity
    and the single inert Level 10 control;
  - `fixtures/presentation/dungeon.json`,
    `fixtures/presentation/combat.json` — signed Level 10 render facts;
  - `provenance/source-trace.json` — source-to-Rust Level 10 evidence;
  - `scripts/play.sh`, `scripts/test.sh`, `scripts/test-provider.sh` — visible
    Level 10 path, gate label, and exact v15 restart/replay assertions;
  - `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` — current
    scope, compatibility ledger, and port milestone.
- Platform files remain limited to Ticket 063 lifecycle and completion
  evidence. No server, SDK, QML, route, renderer vocabulary, database, or
  migration source change is designed.
- Regression table:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Exact ten-row order/flag test, source hashes, source-trace validator, and compatibility review. |
  | REQ-002 | v15 exact-schema test; unsupported level, boundary/unknown record, wrong name, malformed JSON, and state/RNG immutability checks. |
  | REQ-003 | Draw-free level 1–10 transitions, visible labels, phase/location/monster checks, and 0/11/max rejection. |
  | REQ-004 | Forced rejected/accepted `Random(100)` trace, record 90 exclusion, records 91–99 bound, 20/10/60 combat state, fight spend, and deterministic twin. |
  | REQ-005 | Exact `(2, 100)` retreat trace plus existing attack, quick-heal, spell, class-special, reward, poison, and complete-day suite. |
  | REQ-006 | Generic/fixed provider equality and replay, unique signed button label/action, all-screen QML smoke, local click path, live corpus twice across restart, platform diff gate, scope/security review, and provider-backed workspace-8 play. |
- Risk and rollback review:
  - wrong roster order/flags or exposing record 90 is covered by exact arrays,
    lookup boundaries, and forced rejection traces;
  - Level 10 RNG can shift later retreat/death outcomes, so provider and live
    drivers will be reconciled to observed authenticated state transitions;
  - a tenth dungeon control can regress layout or duplicate rendering, so
    signed-screen conformance, label uniqueness, the local click path, and
    visible workspace-8 play must exercise it;
  - strict v15 prevents mixed-schema interpretation; rollback is the unchanged
    v14 release/session identity, not an in-place conversion;
  - there is no new identity, credential, network, database, executable-code,
    shared-realm, or platform-authority surface.
- Alternatives rejected:
  - inferring Level 10 from arithmetic without exact rows would lose source
    order, duplicate-Mercenary identity, flags, and boundary-record proof;
  - combining Level 11 or dungeon events would cross a distinct content or
    control-flow boundary and weaken this slice's evidence;
  - adding a new view field, platform rule, or bespoke renderer control would
    widen contracts despite the existing `option_j` capacity.
- Phase 2 exit: source contract, architecture, compatibility boundary, exact
  file manifest, regression mapping, risks, baseline, and worktree-bound
  CodeGraph evidence are actionable.

## Phase 3 — Implement

- Built:
  - added exact Level 10 records 90–99 to `usurper-data`, preserving canonical
    order, both distinct Mercenary rows, source spelling, base strength 20,
    equipment flags, lookup coverage, and explicit record boundaries;
  - advanced rules/state identity to v15, accepted levels one through ten,
    exposed Level 10 through existing `option_j`, and retained rejected
    `Random(100)` draws until records 91–99 are selected with strength 20,
    defence 10, and 60 HP;
  - added draw-free Level 10 switching plus selection, deterministic replay,
    boundary, retreat-damage, hostile-state, generic/fixed provider, view, and
    replay regressions while preserving the lower-level suite;
  - added the fixed `enter_dungeon_level_10` provider action and one signed
    inert dungeon button, rules/cartridge v15 identity, Level 10 fixtures,
    provenance, local-play path, provider corpus identity, and current-scope
    compatibility documentation.
- Focused and complete implementation proof:
  - `cargo test -p usurper-data -p usurper-rules -p usurper-provider
    --all-features` passed after one fixture correction;
  - the first Level 10 failed-retreat fixture selected 18 damage, which killed
    the deterministic development character and correctly entered `Dead`
    rather than the asserted `Combat`; the fixture now selects 13 damage while
    retaining exact `(2, 100)` draw-bound proof;
  - focused Level 10 data, rules, and generic/fixed provider tests passed;
  - `scripts/test-cartridge.sh` passed every one of seventeen signed screens,
    duplicate visible-label rejection, and trusted-QML auto-repeat smoke;
  - `scripts/test-local-play.sh` passed Entry → creation → Main Street → Level
    1 → Level 10 → combat over authenticated provider-backed HTTP/QML state;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    ./scripts/test.sh` passed formatting, warning-denying Clippy, all 83 Rust
    tests, rustdoc, immutable source/provenance and privacy assertions, signed
    cartridge conformance, and provider-backed local-play smoke;
  - JSON parsing, shell syntax, and `git diff --check` passed.
- Implementation stayed inside the Phase 2 manifest. No platform server, SDK,
  protocol, QML, route, renderer vocabulary, database, migration, packaging,
  admission, deployment, delivery, or publication surface changed.
- Phase 3 exit: the v15 external-provider slice and focused evidence are ready
  for inspection.

## Phase 3.5 — Inspect

- CodeGraph re-traced the platform action-validation, provider-brokering,
  binding-resolution, and `RenderedNode::Button` path after implementation that
  Level 10 remains an external provider-owned zero-payload action. No platform
  application, schema, database, or migration change is warranted. Inspect
  receipt: `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`, pipeline
  `ec0e8c0b-a5a8-4ea8-b484-2a9adaa3b7f6`, state hash
  `797bd8ed751f93a3e624a3692a2f43fc45ee373ea86381739e00d6dbb303cb2f`.
- Codex Security completed an offline diff review of all fifteen generated
  changed-file items in the accumulated external-provider working-tree
  snapshot. Exact Level 10 data/bounds, generic and fixed action routing,
  signed presentation, unique/current controls, local HTTP/assets, production
  configuration, test harnesses, and provenance all received full-file or
  supporting-boundary evidence.
- One development-tool hypothesis was dynamically reproduced: on this host, a
  different local UID can read the QML process arguments and recover the
  loopback bearer. Attack-path policy rejected it as non-reportable because it
  reaches only one ephemeral, in-memory, developer-only game session and no
  production identity, provider key, database credential, or persistent state.
  No Level 10 finding survived validation/reportability.
- Sealed security scan: `72315a57-35f5-4ed7-8111-c346cf5455a5`; report:
  `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260902T231145Z_2vm58j19/report.md`;
  complete measured usage: 17,366,290 total tokens, 17,306,856 input tokens,
  and 16,760,832 cached input tokens across four review threads.
- Inspection ledger: zero reportable findings, zero deferred candidates, and
  no requested-scope change. Level 11+, dungeon events, shared realm,
  registration/admission, deployment, and publication remain explicitly out
  of scope.
- Phase 3.5 exit: architecture and security inspection pass; proceed to the
  isolated provider restart corpus, complete gates, and visible workspace-8
  play proof.

## Phase 4 — Validate

- The first exact v15 provider-corpus run reached the real application and
  returned a bounded 422 in the gameplay profile. A temporary deterministic
  trace showed that Level 10 selected Grifter, cast to 4 HP, retreated safely,
  selected Mercenary, cast to 2 HP, then took 17 failed-retreat damage and
  entered `Dead`; the old lower-level driver incorrectly expected another
  living Main Street transition.
- The profile was reconciled to the actual provider state machine: it now
  inserts Look, cast, retreat, and re-entry once, resumes after the absorbed
  transition, and does not submit a redundant Main Street command after
  `reenter` already reaches Main Street. This applies the recalled
  `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` instead of
  creating a duplicate rule.
- With the isolated PostgreSQL Compose override, `scripts/test-provider.sh`
  passed the fixed fifteen-case TLS/replay/fault/callback corpus twice across
  process restart using the exact v15 Usurper profile.
- `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system ./scripts/test.sh`
  passed after that final external-provider change: formatting,
  warning-denying Clippy, all 83 Rust tests, rustdoc, immutable
  source/provenance and privacy checks, all seventeen signed screens,
  duplicate-label rejection, trusted-QML activation-repeat smoke, and the
  provider-backed Entry → creation → Main Street → Level 1 → Level 10 → combat
  local-play path.
- The complete platform `bin/gate.sh --diff` passed all 24 stages with the
  isolated PostgreSQL override and process-local port redirection. Its receipt
  matches state hash
  `34c9b578d73a6cd19f1f9159935487b8a22c4faac201d7421213baa87df5d6a5`.
- The exact development window, title `OmarchyGS Usurper Local Play —
  Development`, class `org.qt-project.qml`, was opened and verified mapped on
  workspace 8. The desktop was already locked, so no keystrokes were sent into
  the password prompt; the authenticated trusted-QML path and one-control /
  one-revision behavior were exercised by the automated local-play and
  renderer smoke instead. The application remains open for user inspection.
- Phase 4 exit: exact behavior, complete external tests, provider restart
  conformance, trusted-QML interaction regression coverage, workspace-8 window
  placement, security inspection, and the platform gate pass.

## Phase 5 — Complete

- Acceptance audit:

  | Requirement | Final evidence | Result |
  |---|---|---|
  | REQ-001 | Authenticated v0.20e readback, exact records 90–99 tests, immutable source hashes, provenance validation, and compatibility documentation. | PASS |
  | REQ-002 | Strict v15 schema plus hostile state/JSON, cross-field, level/record/name, and RNG-immutability tests. | PASS |
  | REQ-003 | Draw-free generic/fixed level 1–10 transitions and unchanged rejection for 0, 11, and larger values. | PASS |
  | REQ-004 | Forced `Random(100)` rejection trace, record-90 exclusion, records 91–99 bounds, deterministic twins, and exact 20/10/60 combat state. | PASS |
  | REQ-005 | Full lower-level attack, retreat, potion, spell, class-special, reward, poison, and complete-day regressions plus exact Level 10 retreat bounds and observed lethal trace. | PASS |
  | REQ-006 | One unique phase-valid control per visible choice, one activation per revision, seventeen signed screens, multi-phase provider-backed HTTP/QML smoke through Level 10 combat, provider corpus twice across restart, workspace-8 app placement, zero reportable security findings, and full platform gate. | PASS |

- OpenWiki lifecycles `817915fd-f84a-47a9-80b7-15a65f41a4c6`,
  `94f48805-4de7-4116-bcf8-bab7523be12a`, and
  `3eab6db4-8e99-47d8-8932-6e341f541831` completed the Ticket 063 update, prose
  correction, and completed-evidence relocation for `openwiki/quickstart.md`
  and `openwiki/game-cartridges.md`. All reported the pages' pre-existing
  unresolved Claims evidence debt and therefore left their Claims sidecars
  unchanged; the new Level 10 prose and repository evidence remain present.
- AAR 063 is submitted and effective. The existing composite-driver prevention
  rule covered the only implementation-time profile correction, so no
  duplicate BF, prevention-rule, or architecture-decision ID was added.
- Ticket 063 is closed, its index and artifact links are reconciled, and the
  spec/notes are ready for the completed pipeline archive.
- Scope audit found no silent drops: Level 11+, dungeon events, quests, finale,
  shared realm, new combat systems, platform gameplay/schema/database changes,
  registration, admission, packaging, deployment, commit, push, and
  publication remain out of scope.
- Phase 5 exit: all six acceptance criteria pass; documentation, AAR, ticket,
  OpenWiki, inspection, validation, and archival evidence are reconciled.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first v15 live profile received a bounded 422 after deeper Level 10 combat. | The driver assumed the lower-level living phase/HP trace after a cast and retreat; the actual deterministic Level 10 sequence killed the character. | Trace the exact provider sequence, insert one re-entry path, resume after the absorbed transition, and remove the redundant Main Street command. | Reuse `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`: composite drivers follow authenticated post-command phase and issue one command per iteration. |
| 2 | The requested workspace-8 visual pass encountered the existing desktop lock screen. | The user session was locked independently of the game. | Map and verify the exact window on workspace 8, leave it open, and avoid sending input into the password prompt; rely on the authenticated trusted-QML regression path for interaction proof. | Treat a lock screen as an input-safety boundary, not permission to type credentials or blind keystrokes. |
| 3 | OpenWiki could not rewrite the two broad pages' Claims sidecars. | Both pages already contain unrelated stale/unresolved evidence debt. | Complete the lifecycle with explicit warnings and preserve the grounded Level 10 prose/source metadata. | Do not misreport an OpenWiki content update as clean Claims verification when pre-existing debt prevents sidecar finalization. |
