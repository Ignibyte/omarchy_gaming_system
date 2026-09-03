---
title: Usurper Level-Eight Dungeon Band — notes
pipeline_id: ae7ba576-c2df-471d-a162-e4a3bf30395e
---

# Usurper Level-Eight Dungeon Band — running notes

## Phase 1 — Plan

- Recall:
  - The prior goal turn completed Ticket 059 with rules/state/cartridge v12,
    levels one through seven, a full green gate, completed OpenWiki lifecycle,
    and a signed Level 7 preview on workspace 8.
  - `BUL-002-pre-rebuild-delivery-handoff` remains informational. The ignored
    upstream v0.20e corpus is reconstructed, its twelve recorded hashes pass,
    the publisher-linked source tree is detached and clean at tree
    `51624a9b0d259ac762b4c3eb5fb0672b1226923b`, and no publication is authorized.
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires stored boundary records and normal reachability to be proven
    separately.
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps the slice in solo non-classic normal dungeon combat.
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` makes
    every rejected `Random(80)` result observable deterministic behavior.
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` requires
    full-profile replay because earlier encounter draws affect later commands.
  - `PR-omarchy-gaming-system-resolve-cargo-artifacts-from-metadata-001`
    remains implemented in the live provider harness and must not regress.
  - The pipeline toolchain is ready and the project PostgreSQL container is
    healthy on the rebuilt machine's isolated host port.
- Canonical readback:
  - `SOURCE/EDITOR/EDMONST.PAS:3311-3410` declares Level 8 records 70–79:

    | Index | Name | Base strength | Armor user | Weapon user | Normal Level 8 selection |
    |---:|---|---:|---|---|---|
    | 70 | Unknown Reptile | 18 | no | no | no — boundary record |
    | 71 | Sandworm | 18 | no | no | yes |
    | 72 | Orc Officer | 18 | yes | yes | yes |
    | 73 | Orc General | 18 | yes | yes | yes |
    | 74 | Bronze Elf | 18 | yes | yes | yes |
    | 75 | Glowing Gnoll | 18 | yes | yes | yes |
    | 76 | Warrior | 18 | yes | yes | yes |
    | 77 | Strange Thing | 18 | no | no | yes |
    | 78 | Warrior | 18 | yes | yes | yes |
    | 79 | Elf Chieftain | 18 | yes | yes | yes |

  - `SOURCE/USURPER/DUNGEONC.PAS:748-804` owns in-dungeon level change;
    `:869-955` spends a fight and repeats `Random(level*10)` until the result
    exceeds `(level-1)*10`; `SOURCE/USURPER/PLVSMON.PAS:603-625` initializes
    HP to strength times three; and `:68-138` uses
    `Random(global_dungeonlevel*10)+3` for failed retreat damage.
- Decisions:
  - Ship exact Level 8 rows, selection, deterministic combat composition,
    fixed/generic provider controls, signed presentation, tests, provenance,
    compatibility documentation, and a visible workspace-8 preview as one
    shippable slice.
  - Advance exact state/rules/cartridge identity to v13; do not reinterpret
    v12 provider state.
  - Defer level nine, composite events, teams, shared realm, platform gameplay
    logic, migration, protocol, packaging, admission, deployment, delivery,
    and publication.

## Phase 2 — Design

- Architecture and data flow:
  - `usurper-data` owns the exact immutable Level 8 editor fixtures;
  - `usurper-rules` remains the sole game authority for level switching,
    rejection-loop selection, combat, validation, and deterministic RNG;
  - `usurper-provider` decodes only the generic `enter_dungeon` and fixed
    `enter_dungeon_level_8` forms, then returns the ordinary bounded view;
  - the signed cartridge binds the existing `option_h` view field to one inert
    `enter_dungeon_level_8` button; the trusted renderer already admits this
    node shape and the resulting dungeon screen remains far below its node
    budget;
  - OmarchyGS continues authenticating and translating the signed zero-payload
    action, brokering it to the exact registered provider release, and
    rendering the returned typed plan without Usurper-specific rules or state.
- CodeGraph could not index the separate Usurper repository because it has no
  `.codegraph/` database, so its Rust sources and non-Rust contracts were
  inspected directly. The worktree-bound platform graph traced
  `ValidatedSessionCartridgeAction`, registered-provider command translation,
  `ProviderGame::command`/`view`, and `RenderPlan`. It confirms that the action
  schema remains cartridge-owned, the provider game receives neither platform
  identity nor credentials, and the generic renderer consumes only bounded
  nodes. Design receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `ae7ba576-c2df-471d-a162-e4a3bf30395e`, state hash
  `4008a486023fe1d3c477a30e4659018caa0a6ff8d4ead8ac55f523c6300a0b07`.
- API/state and compatibility contract:
  - advance external state, rules, and cartridge identity from v12 to v13 and
    accept exact v13 only; no v12 state is silently migrated;
  - accept generic `enter_dungeon` levels 1–8 and fixed level actions 1–8;
    reject 0, 9, and larger unchanged and without RNG work;
  - require active monsters to belong to the selected implemented band, match
    the exact source-linked name, retain bounded scalars, and exclude every
    normally unreachable boundary record; encounter initialization still uses
    the exact reviewed base-strength fixture;
  - retain record 70 in immutable data while normal Level 8 selection accepts
    only 71–79 after all source-order `Random(80)` rejections;
  - preserve both source-distinct `Warrior` records 76 and 78 even though their
    current stored name, strength, and equipment flags are identical;
  - preserve Provider SDK/protocol v1, the existing game key, provider ID,
    player-private state shape, and seventeen-screen presentation protocol.
- Database and migration consequences: none in OmarchyGS. The external starter
  continues owning its independent PostgreSQL state and operation receipts;
  strict v13 identity means validation uses fresh development sessions rather
  than mutating v12 rows.
- Planned implementation files in the external provider:
  - `crates/usurper-data/src/lib.rs` — exact records 70–79, lookup, and data
    tests;
  - `crates/usurper-rules/src/lib.rs` — v13 validation, level selection/view,
    encounter/retreat behavior, and reducer regressions;
  - `crates/usurper-provider/src/lib.rs` — fixed action plus generic/fixed and
    replay coverage;
  - `cartridge/manifest.json`, `cartridge/presentation.json` — exact v13
    identity and inert Level 8 control;
  - `fixtures/presentation/dungeon.json`,
    `fixtures/presentation/combat.json` — signed Level 8 render facts;
  - `provenance/source-trace.json` — source-to-Rust Level 8 evidence;
  - `scripts/test.sh`, `scripts/test-provider.sh` — human-readable gate label,
    exact v13 live profile, and composite replay assertions;
  - `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` — current
    scope, compatibility ledger, and port milestone.
- Platform changes remain limited to Ticket 060 planning, architecture/wiki
  reconciliation, and completion evidence. No server, SDK, QML, route,
  database, migration, or renderer-vocabulary source change is designed.
- Regression table:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Exact ten-row data/order/flag test, including both Warrior indices; source hashes; source-trace validator; compatibility review. |
  | REQ-002 | v13 exact-schema test; unsupported level, boundary/unknown record, wrong name, oversized scalar, malformed JSON, and state/RNG immutability checks. |
  | REQ-003 | Draw-free level 1–8 transitions, visible labels, phase/location/monster checks, and 0/9/max rejection. |
  | REQ-004 | Forced rejected/accepted `Random(80)` trace, record 70 exclusion, records 71–79 bound, 18/9/54 combat state, fight spend, and deterministic twin. |
  | REQ-005 | Exact `(2, 80)` retreat trace plus existing attack, quick-heal, spell, class-special, reward, poison, and complete-day suite. |
  | REQ-006 | Generic/fixed provider equality and replay, signed cartridge conformance, all-screen QML smoke, fixed live corpus twice across restart, platform diff gate, scope/security review, and visible Level 8 preview. |
- Risk and rollback review:
  - wrong roster order/flags, collapsing the two distinct Warrior records, or
    exposing record 70 is covered by exact arrays, lookup boundaries, and
    forced rejection traces;
  - added encounter RNG can shift later retreat/death profile outcomes, so the
    complete live command driver must be replayed and adjusted only to match
    the deterministic source-composed phases;
  - an eighth dungeon control could regress layout or action declaration, so
    the signed all-screen QML smoke and visible preview must exercise it;
  - strict v13 prevents mixed-schema interpretation; rollback is the
    unmodified v12 release/session identity, not in-place state conversion;
  - no new identity, credential, network, database, executable-content, shared
    realm, or platform-authority surface is introduced.
- Rebuilt-machine baseline evidence:
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system scripts/test.sh`
    passed before implementation: formatting, warning-denying Clippy, 66 Rust
    tests, rustdoc, immutable upstream hashes, provenance/privacy assertions,
    signed cartridge conformance, and all seventeen trusted-QML screens;
  - the live provider harness retains the prior metadata-resolved artifact
    paths. Its full post-change TLS/replay/fault/callback run remains Phase 4
    evidence rather than being inferred from the Level 7 result.
- Phase 2 exit: architecture, compatibility/state contract, exact file
  manifest, regression mapping, risks, baseline, and CodeGraph evidence are
  actionable.

## Phase 3 — Implement

- Built:
  - Added exact Level 8 rows 70–79 to `usurper-data`, including both distinct
    source indices named `Warrior`, reviewed base strength 18, equipment flags,
    lookup coverage, and explicit record-70/record-79/table assertions.
  - Advanced rules identity to v13. State and commands now accept levels 1–8,
    the dungeon view exposes `option_h`, and encounter selection preserves
    every rejected `Random(80)` result until records 71–79 are selected. Level
    8 monsters initialize at strength 18, defence 9, and 54 HP.
  - Added Level 8 rejection-trace, deterministic-twin, boundary, draw-free
    switching, retreat-bound, hostile-state, generic/fixed provider, view, and
    replay regressions while retaining the complete lower-level suite.
  - Added the fixed `enter_dungeon_level_8` provider decoder,
    rules/cartridge v13 identity, signed inert dungeon button/action, Level 8
    fixtures, provenance, live-profile selection, and compatibility/port-map
    descriptions.
  - Reconciled the composite live driver with actual deterministic behavior:
    after Level 8 Look and Cure Light, the first failed retreat is lethal, so
    re-entry follows immediately instead of issuing the now-invalid second
    Look used by the Level 7 trace.
  - Applied the existing metadata-resolved Cargo artifact rule to the provider
    key helper, signed-cartridge harness, and visible preview launcher. Each now
    executes the exact platform target directory reported by its manifest.
- Focused and complete proof after formatting:
  - `cargo test -p usurper-data`: 10 passed;
  - `cargo test -p usurper-rules`: 43 unit plus 1 integration passed;
  - `cargo test -p usurper-provider`: 17 passed, including the Level 8 live
    death/re-entry profile;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    scripts/test-cartridge.sh`: all seventeen signed screens and trusted-QML
    state smoke passed;
  - `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system scripts/test.sh`:
    formatting, warning-denying Clippy, all 71 Rust tests, rustdoc, immutable
    upstream/provenance and privacy assertions, plus the signed-screen suite
    passed;
  - `bash -n` for the four affected scripts and `git diff --check`: passed.
- Deviations:
  - The approved manifest expanded from thirteen to fifteen external files to
    include `scripts/test-cartridge.sh` and `scripts/show.sh`; direct inspection
    found that both violated the newly registered artifact-resolution rule.
    `scripts/test-provider.sh` was already in scope and its remaining hardcoded
    provider-key helper was fixed at the same time. This strengthens the exact
    Level 8 proof without changing product authority or behavior.
  - No platform source, protocol, migration, database, SDK, QML, renderer
    vocabulary, packaging, admission, deployment, or publication surface
    changed.
- Phase 3 exit: the approved implementation, justified harness expansion, and
  focused evidence are ready for independent inspection.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Canonical fidelity | Exact editor rows 70–79, duplicate Warrior records 76/78, equipment flags, `Random(80)` rejection semantics, 54 HP initialization, and the level-derived retreat bound match the verified v0.20e sources. | none | PASS — direct source, implementation, provenance, and deterministic test inspection agree. |
| 2 | State and command integrity | Level 8 enters through either strict generic JSON or one payload-free fixed action, then the reducer enforces phase, level 1–8, schema v13, catalog band/name, scalar, and RNG-trace bounds before and after a cloned transition. | none | PASS — unknown, out-of-phase, old-schema, level 0/9/max, boundary record 70, unknown record 80, and wrong-name state are rejected without accepted mutation. |
| 3 | Provider and cartridge boundary | The starter owns authenticated current state, pairwise subject, revision, persistence, and replay receipts; the external adapter receives no platform identity or credentials. The signed cartridge only requests declared inert actions and bounded view fields. | none | PASS — no platform gameplay authority, protocol, route, migration, SDK, QML, or renderer-vocabulary change is required. |
| 4 | Harness artifact resolution | `show.sh`, `test-cartridge.sh`, and `test-provider.sh` resolve the platform and game target directories with Cargo metadata, require nonempty absolute paths, quote every execution, and build before launch. | none | PASS — the invoking operator already owns Cargo environment authority; no lower-privilege input reaches a new execution sink. |
| 5 | Security diff scan | Complete native review covered all ten changed source-like files, the five remaining fixtures/docs, and the supporting starter-owned state/replay controls. | none | PASS — scan `305eca4b-953f-43e6-96e7-c67a55659312` sealed with zero findings and complete coverage. TAC advisory status remained unverified because the access connector was not signed in. |
| 6 | Platform blast radius | Fresh CodeGraph inspection traced the provider game boundary and signed-cartridge verification/rendering path; its query returned only existing generic provider/cartridge symbols and no Level 8-specific platform dependency. | informational | PASS — the adjacent Usurper repository remains unindexed and was inspected directly. Worktree-bound inspect receipt matches pipeline `ae7ba576-c2df-471d-a162-e4a3bf30395e` and state hash `4008a486023fe1d3c477a30e4659018caa0a6ff8d4ead8ac55f523c6300a0b07`. |

- Security report:
  `/mnt/fast/tmp/codex-security-scans-t0HL23/omarchygs_usurper/bb31caa122de669d72a265860b19969fcd28505f_20260902T191132Z_s2w3q0rw/report.md`.
- The security architecture pass was performed sequentially because this task's
  project instructions did not authorize delegation. No reportable or deferred
  candidate remains.
- Phase 3.5 exit: the full current external diff is source-backed, the approved
  scope deviation is justified, all inspection hypotheses are disposed, and no
  finding requires a code change.

## Phase 4 — Validate

- Tests run:
  - final external `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
    scripts/test.sh`: PASS — rustfmt, warning-denying Clippy, 71 Rust tests,
    rustdoc, all canonical-source hashes, provenance/privacy checks, signed
    cartridge conformance, and seventeen trusted-QML screen/state smokes;
  - live external `scripts/test-provider.sh`: PASS — the fixed 15-case
    TLS/replay/fault/callback corpus passed twice across an actual provider
    restart with rules v13 and the Level 8 death/re-entry sequence;
  - full platform `bin/gate.sh --diff`: GREEN through all 24/lettered checks,
    including PostgreSQL integration, real API/QML smoke, provider conformance,
    sidecar, first-party authority, backup/restore, private-alpha, reproducible
    package, and contained server-module drills.
- Gate run:
  - `.git/omarchy-gaming-system-gate-receipt` contains
    `4008a486023fe1d3c477a30e4659018caa0a6ff8d4ead8ac55f523c6300a0b07`,
    exactly matching a fresh `ogs_gate_state_hash` readback;
  - the host's unrelated PostgreSQL instance owns port 5432, so the project
    container remained on 55432 and a temporary loopback-only test adapter
    redirected canonical hardcoded 5432 integration clients to that container.
    No repository or production configuration changed.
- Visible validation:
  - replaced the prior preview with one signed rules/cartridge v13 combat
    preview on Hyprland workspace 8; direct screenshot inspection confirms
    `A level 8 Sandworm blocks your way`, player HP 19, monster HP 54, and the
    existing Attack, Retreat, Quick heal, spell, and class-special controls;
  - the preview is intentionally fixture-only. Button presses emit
    `requested=<action> confirmed=false` and do not mutate state because
    `scripts/show.sh` does not attach a live provider/session. The live
    conformance corpus proves the actual action path separately. A playable
    local end-to-end shell is follow-up scope, not evidence supplied by this
    preview.
- Skips or pre-existing failures:
  - none. Tests that the ordinary Cargo pass marks ignored for external
    services were exercised by their canonical live scripts later in the same
    green gate.
- Phase 4 exit: all six requirements have executable, source, inspection,
  security, receipt, and visible-render evidence. Phase 5 documentation and
  archival remain.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — exact rows 70–79, source hashes, provenance trace, and
    compatibility/port-map readback agree;
  - REQ-002 PASS — strict v13 hostile-state and malformed-JSON tests reject old,
    out-of-band, boundary, wrong-name, and oversized inputs without accepted RNG
    advancement;
  - REQ-003 PASS — generic and fixed level controls switch draw-free among
    levels 1–8 and reject 0, 9, and larger values unchanged;
  - REQ-004 PASS — forced rejection traces preserve every `Random(80)` draw,
    exclude record 70, accept records 71–79, and initialize 18/9/54 combat;
  - REQ-005 PASS — attack, retreat, potion, spell, class-special, reward, poison,
    and exact Level 8 retreat behavior pass the full reducer/provider suites;
  - REQ-006 PASS — fixed action/view/replay, the live fifteen-case corpus twice
    across restart, signed cartridge/QML smoke, security and scope inspection,
    the platform diff gate, and the workspace-8 preview all passed without
    production admission or platform gameplay changes.
- Docs: compatibility, Rust port map, external README, durable cartridge
  architecture, and the affected OpenWiki pages now describe rules v13 and the
  fixture-preview/live-provider distinction.
- AAR: `AAR-060-usurper-level-eight-dungeon-band` submitted effective; no new
  knowledge ID was needed.
- Archive: ticket, spec, and notes moved to their closed/completed locations;
  delivery remains unauthorized and deferred.
- Final post-documentation gate: `bin/gate.sh --diff` GREEN with receipt and
  fresh state hash
  `f6c27243492f58e2fcd7712e9af59398b67d2be08636d48a110874b2cb7a5fcd`.
  The successful rebuilt-host run used the project database's isolated port,
  `TMPDIR=/tmp`, and no ambient `CARGO_TARGET_DIR`; no repository or production
  configuration was changed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first focused provider test issued a second Look after the Level 8 trace had already killed the low-HP Gnoll Cleric. | The new monster/retreat inputs changed the deterministic driver length, not the rules contract. | Removed the now-invalid extra turn and re-entered immediately after the proven lethal retreat. | Assert the exact intermediate phase/HP trace when advancing a live conformance profile to a new band. |
| 2 | Three external harness execution sites still assumed the platform used `<repo>/target`. | The prior artifact-resolution repair covered only the main binaries used by that ticket. | Resolved both manifests' Cargo target directories, required absolute paths, and executed the just-built binaries from those paths. | Apply the existing metadata-resolved artifact rule to every helper added to an affected harness manifest. |
| 3 | The visible preview's controls looked playable but did not change state. | The signed preview intentionally renders a fixed fixture and exposes only unconfirmed action requests; it is not attached to a provider session. | Clarified the validation claim and retained live provider conformance as the mutation proof. | Label fixture previews explicitly and plan a separate interactive local-play harness before claiming end-to-end visible gameplay. |
| 4 | The first archived spec failed the pipeline structure check. | Its descriptive completion status did not match the checker's exact accepted value. | Changed the status to `Phase 5 — Complete PASS` and reran pipeline and hook checks. | Copy lifecycle status values exactly from `scripts/check-pipeline.sh`. |
| 5 | The first post-documentation gate reported hook and Door Legends authority failures after their substantive work passed. | The rebuilt shell supplied `/mnt/fast/tmp` to cleanup code constrained to `/tmp`, and an ambient Cargo target override to a legacy drill that still resolves `<clone>/target`. | Reproduced both failures, traced their exact exit sites, then ran with `TMPDIR=/tmp` and the ambient Cargo override removed; the focused drill and complete gate passed. | Normalize those test-only host variables until the remaining platform helper adopts the existing metadata-resolution rule. |
