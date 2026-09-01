---
title: Usurper Level-Five Dungeon Band — notes
pipeline_id: 977608e8-61c6-4ce9-a596-4cc73b8701b7
---

# Usurper Level-Five Dungeon Band — running notes

## Phase 1 — Recall and plan

- User direction remains to continue building and visibly showing Usurper
  while deferring packaging and delivery.
- No active pipeline, open ticket, or blocking bulletin existed; Ticket 057
  was next. Pipeline tools are ready, PostgreSQL is healthy, and the signed
  Level 4 preview remains open until a replacement Level 5 preview is proven.
- Recalled knowledge:
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires preserving the stored boundary row independently from normal
    reachability;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps the slice in solo non-classic normal dungeon combat;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001`
    makes every rejected `Random(50)` result observable deterministic behavior;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`
    requires replaying the complete provider profile after the new earlier RNG
    work changes later outcomes;
  - `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001`
    remains applicable to the provider conformance credential boundary;
  - Ticket 056 supplies the rules-v9 levels-one-through-four implementation,
    seventeen-screen cartridge, clean security scan, and visible baseline.
- Canonical v0.20e readback establishes ten Level 5 editor records at indices
  40–49 and the following exact base fixture values:

  | Index | Name | Base strength | Armor user | Weapon user | Normal Level 5 selection |
  |---:|---|---:|---|---|---|
  | 40 | Judge | 15 | yes | yes | unreachable boundary |
  | 41 | Unknown Reptile | 15 | no | no | accepted |
  | 42 | Micro Dragon | 15 | no | no | accepted |
  | 43 | Insane Ranger | 15 | yes | yes | accepted |
  | 44 | Grift Ghoul | 15 | no | no | accepted |
  | 45 | Grave Robber | 15 | yes | yes | accepted |
  | 46 | One-eyed Mutant | 15 | yes | yes | accepted |
  | 47 | Psycho | 15 | yes | yes | accepted |
  | 48 | Gnoll | 15 | yes | yes | accepted |
  | 49 | Disgusting Man | 15 | yes | yes | accepted |

- Source anchors:
  - `SOURCE/EDITOR/EDMONST.PAS:3005-3105` declares the Level 5 rows;
  - `SOURCE/EDITOR/ADDMONST.PAS:43-72` distinguishes editor base strength from
    initialized-world randomization, matching the prior explicit development
    fixture policy;
  - `SOURCE/USURPER/DUNGEONC.PAS:868-955` spends a fight and repeats
    `Random(level*10)` until the candidate exceeds `(level-1)*10`;
  - `SOURCE/USURPER/PLVSMON.PAS:603-625` sets HP to strength times three;
  - `SOURCE/USURPER/PLVSMON.PAS:68-138` makes failed retreat damage use
    `Random(global_dungeonlevel*10)+3`.
- Phase 1 exit: scope, six EARS requirements, six locked decisions, Ticket 057,
  pipeline UUID `977608e8-61c6-4ce9-a596-4cc73b8701b7`, and open AAR are settled.

## Phase 2 — Design

- Architecture and data flow:
  1. `usurper-data` extends the exact editor-backed monster catalog through
     index 49; no initialized `MONSTER.DAT` or invented live-world random
     scalar is claimed.
  2. `usurper-rules` advances to v10, admits only levels one through five,
     preserves the generic level-band rejection loop, and feeds the selected
     Level 5 seed into the existing combat, retreat, spell, special, potion,
     poison, and reward reducers.
  3. `usurper-provider` maps one new fixed inert action to the existing generic
     `EnterDungeon` command and continues to own private state/revision/replay
     through the public Provider Starter boundary.
  4. The signed cartridge adds one Level 5 button bound to existing `option_e`;
     the platform transports the action as opaque provider JSON, validates the
     bounded provider view, compiles the signed screen, and renders only trusted
     QML nodes.
- Exact file manifest:

  | File | Purpose |
  |---|---|
  | external `crates/usurper-data/src/lib.rs` | add exact Level 5 records, lookup band, and table tests |
  | external `crates/usurper-rules/src/lib.rs` | advance v10, permit levels 1–5, label Level 5, and prove selection/state/combat bounds |
  | external `crates/usurper-provider/src/lib.rs` | decode fixed Level 5 action and prove generic/fixed/replay/view behavior |
  | external `cartridge/manifest.json` | advance exact rules/cartridge identity to v10 |
  | external `cartridge/presentation.json` | add one inert Level 5 button/action using `option_e` |
  | external `fixtures/presentation/{dungeon,combat}.json` | provide visible Level 5 signed examples |
  | external `provenance/source-trace.json` | add exact Level 5 source/data proof and retarget strict-state proof |
  | external `scripts/test.sh` | require the expanded provenance floor and identify the Level 5 milestone |
  | external `scripts/test-provider.sh` | pin v10 and drive a deterministic Level 5 profile twice across restart |
  | external `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` | state the new exact implemented boundary and remaining Level 6+ work |
  | platform planning, AAR, architecture, and OpenWiki pages | retain durable workflow and boundary documentation only |
- Database/migration consequences: none. Provider Starter persistence already
  stores bounded opaque JSON and exact rules identity; v9 state is deliberately
  rejected rather than migrated. OmarchyGS gains no game table or migration.
- API and compatibility behavior:
  - public Provider SDK/starter protocol remains exact v1 with the same four
    capabilities and fixed fifteen-case security corpus;
  - `enter_dungeon_level_5` is a parameterless cartridge action translated to
    `Command::EnterDungeon { level: 5 }`; generic `{action:"enter_dungeon",
    level:5}` remains equivalent;
  - v10 accepts only schema v10 and levels 1–5. It rejects v9, zero, six,
    maximum, boundary record 40 in combat, unknown records, mismatched names,
    and oversized scalars before RNG mutation;
  - signed-screen/view vocabulary and the seventeen-screen count do not change.
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | exact data-table name/order/strength/flags test; upstream checksum/clean-source check; new provenance source trace; compatibility/port-map review |
  | REQ-002 | v10 hostile state/JSON suite covering prior schema, unsupported/wrong level, boundary/unknown record, name mismatch, and scalar ceiling with state/RNG equality |
  | REQ-003 | draw-free sequential 1→2→3→4→5→1 switching, projected labels A–E, unchanged same-level path, and rejection of 0/6/max |
  | REQ-004 | deterministic `Random(50)` two-draw rejection trace, accepted 41–49 band, exact 15/7/45 state, one-fight spend, and byte-identical replay |
  | REQ-005 | Level 5 failed retreat trace `(2, _), (50, _)`; attack plus representative spell/special/poison/potion/reward composition through the existing complete suite |
  | REQ-006 | fixed/generic provider equivalence, bounded view/replay/restart, v10 signed cartridge/QML matrix, standard security inspection, full platform gate, and visible Level 5 preview |
- Risks and treatment:
  - rejected Level 5 candidates advance the later deterministic tape; rerun and
    reconcile both the unit profile and live conformance driver rather than
    preserving a stale death/success assumption;
  - record 40 must remain present but invalid as an active normal Level 5
    monster; validate stored data and encounter reachability separately;
  - the Level 5 monster named `Gnoll` must remain ordinary source data and must
    not be confused with the player's Gnoll passive-poison race branch;
  - a new control must stay within the existing inert action/view schema and
    trusted renderer; no publisher QML or platform vocabulary is added;
  - invalid v9 or hostile state must fail before RNG construction/effects;
  - credential/TLS/database controls stay owned by the external Provider SDK,
    starter, conformance harness, and deployment boundary and will be reviewed
    without attributing them to the pure game reducer.
- Material alternatives rejected:
  - no generated formula for monster names or equipment flags; the ten source
    rows remain explicit and reviewable;
  - no selection of record 40 merely because it exists in the table;
  - no state migration or loose multi-version parser inside the v10 game;
  - no Level 6, event dispatcher, shared realm, platform rule copy, new SDK
    feature, or packaging work in this slice.
- CodeGraph design receipt:
  - traced `session_cartridges::translate_command` through the registered
    provider path, bounded provider view, signed render-plan boundary, and QML;
  - registered-provider actions become opaque JSON with the action string plus
    allowlisted signed payload, while the cartridge owns its exact action schema;
  - provider views remain non-empty safe objects under 64 KiB and rules version
    remains release/catalog metadata;
  - no platform application, SDK, QML, database, route, renderer, or migration
    change is needed. The external repository has no CodeGraph index, so its
    Rust, JSON, shell, fixtures, and tests were inspected directly.
- Phase 2 exit: source map, exact file manifest, compatibility behavior,
  regression evidence, risk treatment, and platform blast radius are
  actionable.

## Phase 3 — Implement

- External data and lookup:
  - added exact Level 5 indices 40–49, names, base strength 15, and source
    armor/weapon-user flags;
  - extended `monster_seed` only through index 49 and added exact data tests.
- External rules/state:
  - advanced the exact state/rules identity to v10 and accepted only dungeon
    levels one through five;
  - extended the draw-free level switch and dungeon view with Level 5 while
    rejecting zero, six, and maximum unchanged;
  - retained the generic rejection loop, so Level 5 records every
    `Random(50)` candidate through the first result greater than 40, excludes
    boundary record 40, and initializes an accepted monster to strength 15,
    defence 7, and 45 HP;
  - added exact Level 5 failed-retreat draw/bound/damage coverage and v10
    hostile state cases for prior schema, unsupported/wrong level, boundary,
    unknown record, wrong name, and oversized scalar.
- Provider and persistent profile:
  - decoded fixed `enter_dungeon_level_5`, added generic/fixed equivalence,
    replay, projected view, bounded-state, and return-to-Level-4 tests;
  - moved the permanent Gnoll/Cleric profile to Level 5. Its source-faithful
    later retreat remains death followed by `reenter` across provider restart.
- Cartridge, fixtures, provenance, and docs:
  - advanced manifest rules/cartridge identity to v10;
  - added one inert Level 5 button and zero-field action to the existing signed
    dungeon screen using `option_e`;
  - changed dungeon/combat fixtures to visible Level 5 examples;
  - added the Level 5 canonical source trace, raised the checked trace floor to
    39, and reconciled README, compatibility limits, and the port map.
- Focused evidence:
  - data tests: PASS — 7;
  - rules tests: PASS — 37 unit plus 1 complete-day integration;
  - provider tests: PASS — 12;
  - `scripts/test-cartridge.sh`: PASS — seventeen signed screens and trusted
    QML state smoke;
  - `scripts/test.sh`: PASS — formatting, Clippy with warnings denied, 57 Rust
    tests, rustdoc, upstream checksums/clean source, 39-entry provenance,
    privacy scan, signed cartridge, and trusted QML;
  - `scripts/test-provider.sh`: PASS — fixed fifteen-case TLS/replay/fault/
    callback/reconciliation corpus twice across process restart.
- Implementation deviation/failure: none. The added Level 5 rejection draws
  were reconciled through the complete provider profile; the later retreat
  remains the source-faithful death/re-entry path already expected at Level 4.
- No platform application, SDK, database, migration, route, or QML source was
  changed.
- Phase 3 exit: the complete designed external file manifest is implemented
  and all focused plus external full/provider checks are green.

## Phase 3.5 — Inspect ledger

- Direct source/fidelity review:
  - reread the canonical Level 5 editor rows and confirmed the Rust indices,
    names, base strength, and armor/weapon-user flags exactly match records
    40–49;
  - reread the normal dungeon rejection loop, monster HP initializer, and
    retreat procedure and confirmed the Level 5 implementation retains
    record 40 while normally accepting only 41–49, sets 15/7/45 combat
    scalars, and uses the failed-retreat bound 50;
  - searched the complete current source for stale rules-v9, Level-4-only,
    and incorrect Level 6 range claims; none remain in implementation or
    current product documentation.
- Correctness and scope review:
  - `git diff --check`: PASS;
  - exact rules v10 state and command boundaries, fixed/generic provider
    decoding, inert action registration, Level 5 fixtures, and 39-entry
    provenance were reviewed without a defect;
  - no platform application, SDK, QML, database, route, renderer, or migration
    source changed, and packaging/admission/deployment/delivery remain out of
    scope.
- Fresh CodeGraph inspection:
  - traced `session_cartridges::translate_command` through the registered
    provider path, provider view boundary, signed render plan, and trusted
    renderer after implementation;
  - a zero-payload `enter_dungeon_level_5` action remains opaque provider JSON
    whose exact schema is cartridge-owned, so the platform blast radius is
    still documentation/planning only.
- Standard Codex Security scan:
  - scan `3fb70ee4-818a-4d0a-ac9d-6e546863e031` sealed complete coverage of all
    46 tracked/unignored source files with zero reportable findings;
  - snapshot digest
    `codex-security-snapshot/v1:sha256:8663fdc2bf98878a46fb85f334ff24cfde6eacc375ba8475a0fb14f39d390fcf`;
  - manifest SHA-256
    `4809fb6f5a1f0831598d1012939678ae6ba1bf8b9095426e26915ae80978d554`;
    report SHA-256
    `52bfe7f71be676a214926fbb4f0e3688db48dac112e572d9ac742791a8770d83`;
  - the preflight warned that three usable worker slots were below the
    preferred six, but the scan retained complete scope. One independent
    baseline reviewed all 46 files; an additional independent architecture
    worker could not start at the host thread limit, so the parent verified
    and advanced the previous source-backed model to rules v10/Level 5;
  - explicit limits remain honest: the ignored packaged Provider SDK/starter
    owns TLS/message/grant/revision/replay/database controls, deployed service
    and TLS-file permissions are operator concerns, and the fixed development
    RNG is not suitable for future shared or competitive outcomes.
- Findings ledger: no correctness, security, architecture, or scope finding
  required a code change. Phase 3.5 exits PASS.

## Phase 4 — Validate

- External production evidence:
  - `./scripts/test.sh`: PASS — 7 data, 12 provider, 37 rules, and 1
    complete-day integration test (57 Rust tests total), rustdoc, canonical
    upstream checksum/clean-source checks, 39 source traces, privacy scan,
    signed seventeen-screen cartridge, and trusted-QML smoke;
  - `./scripts/test-provider.sh`: PASS — the fixed fifteen-case
    TLS/replay/fault/callback/reconciliation corpus ran twice across provider
    restart with rules v10 and the Level 5 profile.
- Platform regression evidence:
  - `bin/gate.sh --diff`: PASS — all 24 stages, including Rust formatting,
    Clippy, tests, rustdoc, production cartridge/renderer contracts, both SDK
    releases, two clean provider builds, reproducible native package
    (`889fcf77cad0f561824527704f28639338d27953c40639b4e416590ba64076c2`),
    PostgreSQL integration, live QML smoke, remote-provider security,
    first-party provider authority, backup/restore, private-alpha admission,
    and server-module isolation;
  - the matching worktree receipt records state hash
    `cc165c1d6e99d397dbb330682ecd86ff63441fb9755e030a2e0d3d2ccc5ed045`.
- Visible acceptance evidence:
  - `./scripts/show.sh dungeon`: PASS and remains open for the user in preview
    run `.preview/run.Dpv0rn`;
  - inspected `prepared/render-plan.json`: exact
    `omarchygs.render-plan/v1`, `ready`, `core`, title `The Dungeons`, visible
    Level 5 narrative/status, and inert Level 1–5 controls including exact
    `enter_dungeon_level_5`;
  - the superseded Level 4 preview was closed only after this Level 5 plan was
    verified.
- Requirement audit: REQ-001 through REQ-006 are each satisfied by their
  designed source-readback, reducer/provider/cartridge tests, security scan,
  full platform gate, and visible signed preview evidence. No requirement was
  narrowed or silently dropped.
- Scope audit: no Level 6/event/shared-realm behavior, platform gameplay code,
  migration, packaging, admission, deployment, publication, commit, or push
  was introduced. Phase 4 exits PASS.

## Phase 5 — Complete

- Acceptance-criteria audit: REQ-001 through REQ-006 remain PASS with no
  omitted scope or silent delivery expansion.
- Docs:
  - OpenWiki lifecycle `555384bd-08bd-4ca7-bc20-0e5b5d9f63d8` completed and
    updated `openwiki/quickstart.md` plus `openwiki/game-cartridges.md` for
    Tickets 048–057, rules v10, levels one through five, exact Level 5
    boundary/selection/combat initialization, and unchanged platform authority;
  - its warnings concerned pre-existing unresolved claim debt on the two broad
    pages, not a failed lifecycle or an unverified Ticket 057 requirement;
  - authoritative `docs/architecture/game-cartridges.md` now records Tickets
    047–057, rules v10, all five levels, boundary records 10/20/30/40, accepted
    bands 11–19/21–29/31–39/41–49, and Level 5's 15/7/45 combat state.
- AAR: AAR-057 is submitted and effective. Existing legacy-branch,
  discarded-RNG, composite-driver, private-command-file, and provider-authority
  knowledge covered the slice; no new knowledge ID was necessary.
- Archive: Ticket 057 is closed and its ticket, spec, and notes are under their
  closed/completed paths. Packaging, registration, admission, commit, push,
  deployment, and publication remain explicitly deferred.
- Phase 5 exit: documentation and AAR are reconciled, all six requirements pass,
  the pipeline is archived, and the visible signed Level 5 preview remains open.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | No defect occurred; Level 5 retained the previously reconciled death/re-entry profile. | The generic band extension composed without changing the later expected phase. | Kept exact full-profile assertions and reran both unit and live conformance drivers. | Continue reconciling complete command profiles after earlier RNG work changes. |
