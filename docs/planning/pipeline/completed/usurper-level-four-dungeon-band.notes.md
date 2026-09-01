---
title: Usurper Level-Four Dungeon Band — notes
pipeline_id: 76cc326f-8c51-4775-824a-5c9231999c58
---

# Usurper Level-Four Dungeon Band — running notes

## Phase 1 — Plan

- User direction remains to continue building and visibly showing Usurper
  while deferring packaging and delivery.
- No active pipeline, open ticket, or blocking bulletin existed; Ticket 056
  was next. Pipeline tools are ready and PostgreSQL is healthy.
- Recalled knowledge:
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires retaining the level-band rejection loop and its unused boundary
    record separately;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps this slice in solo non-classic normal dungeon combat;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001`
    applies to rejected encounter candidates because they advance later draws;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`
    requires checking the full provider profile after altered encounter draws;
  - `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001`
    remains binding for the provider conformance harness.
- Ticket 055 supplies the current rules-v8 levels-one-through-three baseline,
  opaque provider state boundary, signed seventeen-screen cartridge, and
  complete combat composition.
- Canonical v0.20e readback establishes ten Level 4 editor records at indices
  30–39, reviewed base strength 14, the existing dungeon change-level path,
  the `Random(level*10)` rejection loop, and monster HP at strength times three.
- Phase 1 exit: scope, six EARS requirements, six locked decisions, Ticket 056,
  active spec/notes, and the open AAR are established.

## Phase 2 — Design

- Canonical roster mapping:

  | Index | Name | Base strength | Armor user | Weapon user | Normal Level 4 selection |
  |---:|---|---:|---|---|---|
  | 30 | Mad Troll | 14 | yes | yes | unreachable boundary record |
  | 31 | Baby Godzilla | 14 | no | no | reachable |
  | 32 | Baby Dragon | 14 | no | no | reachable |
  | 33 | Fat Mummy | 14 | yes | yes | reachable |
  | 34 | Freak | 14 | yes | yes | reachable |
  | 35 | Rabid ant | 14 | no | no | reachable |
  | 36 | Dwarf Zombie | 14 | yes | yes | reachable |
  | 37 | Dwarf Punk | 14 | yes | yes | reachable |
  | 38 | Mad Priest | 14 | yes | yes | reachable |
  | 39 | Psycho | 14 | yes | yes | reachable |

- Architecture and data flow:
  1. The signed inert dungeon screen emits fixed action
     `enter_dungeon_level_4`; the provider adapter maps it to the already
     public game-owned `Command::EnterDungeon { level: 4 }`.
  2. `reduce` validates current v9 state and command phase before cloning state
     or creating RNG, then `enter_dungeon` switches among levels one through
     four without a draw.
  3. `Look` clears encounter-local spells, spends one fight, and preserves
     each `Random(40)` candidate until one exceeds 30; `monster_seed` resolves
     that accepted exact record and initializes strength 14, defence 7, and
     42 HP.
  4. Existing attack, spell, class-special, potion, reward, retreat, poison,
     death, and re-entry reducers consume Level 4 unchanged; retreat derives
     its failure-damage draw bound from dungeon level as 40.
  5. The provider starter persists strict game-owned JSON and replay receipts;
     `view` exposes only bounded presentation facts. OmarchyGS carries the
     opaque action/state/view while its trusted QML renders signed inert data.
- Exact file manifest:

  | Repository/file | Purpose |
  |---|---|
  | external `crates/usurper-data/src/lib.rs` | add exact Level 4 records, lookup band, and table tests |
  | external `crates/usurper-rules/src/lib.rs` | advance v9, permit levels 1–4, label Level 4, and prove selection/state/combat bounds |
  | external `crates/usurper-provider/src/lib.rs` | decode fixed Level 4 action and prove generic/fixed/replay/view behavior |
  | external `cartridge/manifest.json` | pin rules/cartridge v9 |
  | external `cartridge/presentation.json` | add one inert Level 4 button/action |
  | external `fixtures/presentation/{dungeon,combat}.json` | visible Level 4 signed examples |
  | external `provenance/source-trace.json` | add exact Level 4 source/data proof and retarget strict-state proof |
  | external `scripts/test.sh` | require the expanded provenance floor and identify the Level 4 milestone |
  | external `scripts/test-provider.sh` | pin v9 and drive a deterministic Level 4 profile twice across restart |
  | external `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` | state exact implemented and deferred compatibility scope |
  | platform ticket/spec/notes/AAR/index | retain the auditable lifecycle record |
  | platform `docs/architecture/game-cartridges.md` and generated OpenWiki pages | reconcile the development proof at completion |
- Database and migration consequences: none. The provider starter already
  persists bounded game-owned JSON and exact operation receipts in its own
  PostgreSQL database; no schema, transaction, identity, or platform database
  change is introduced.
- API and compatibility behavior:
  - game/state/rules/cartridge identity advances from v8 to v9; v8 state is
    deliberately rejected rather than silently migrated;
  - `Command::EnterDungeon { level }`, Provider SDK v1, presentation protocol
    v1, view schema, and trusted node vocabulary remain unchanged;
  - one new fixed zero-field action is strictly decoded, and all levels above
    four or below one reject before state/RNG mutation;
  - no production release, admission, or compatibility promise is made.
- Regression map:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | exact ten-row data names/order/strength/flags test, canonical checksum/clean-source checks, and new source-trace entry |
  | REQ-002 | v8 rejection; zero/five/maximum level rejection; wrong monster level, boundary index 30, unknown index/name/scalar/schema JSON cases |
  | REQ-003 | Main Street/Dungeon level 1–4 transitions, narration, empty monster, exact RNG equality, and four projected labels |
  | REQ-004 | seeded rejected candidate then accepted 31–39, all bound 40, one fight spent, 14/7/42 scalars, deterministic twin |
  | REQ-005 | Level 4 failed-retreat trace `(2, _), (40, _)`; attack plus representative spell/special/poison/potion/reward composition through the existing complete combat suite |
  | REQ-006 | generic/fixed provider equivalence, replay, restart profile, signed cartridge checks, trusted QML smoke, visible dungeon/combat preview, platform gate |
- Security, privacy, concurrency, reconnect, and rollback risks:
  - malformed persisted level/monster pairs must fail before any command or
    projection can use them;
  - added rejection draws change later deterministic results, so the complete
    live profile must assert actual phase outcomes and restart replay;
  - the button remains inert data and gains no executable, network,
    filesystem, credential, account, or persona authority;
  - state remains under 32 KiB and authenticated provider operations retain
    expected revision, idempotency, replay, callback, and reconciliation;
  - the hardened private-file credential loader is preserved unchanged;
  - rollback is removal of the uncommitted external/platform changes; v9 does
    not accept v8 state and no migration is claimed.
- Alternatives rejected:
  - directly sampling nine records would erase rejected RNG work;
  - accepting Level 5 with placeholder data would create false parity;
  - importing `dungeon_event` would combine a larger composite dispatcher with
    this bounded normal-encounter slice;
  - adding platform game logic or executable QML would violate the established
    single-authority and trusted-renderer boundaries.
- CodeGraph design inspection traced `session_cartridges::translate_command`,
  provider execution/projection, the 64-KiB view ceiling, the public
  `ProviderGame` seam, render-plan compilation, and trusted QML consumers. It
  confirms the fixed action remains opaque provider-owned JSON and no platform
  application-code change is needed. The separate Usurper repository has no
  CodeGraph index, so its Rust, JSON, shell, fixtures, and tests were inspected
  directly.
- Phase 2 exit: the source map, exact file manifest, compatibility behavior,
  risk treatment, regression matrix, and CodeGraph blast-radius evidence are
  actionable.

## Phase 3 — Implement

- External data and lookup:
  - added exact Level 4 indices 30–39, names, base strength 14, and source
    armor/weapon-user flags;
  - extended `monster_seed` only through index 39 and added exact data tests.
- External rules/state:
  - advanced the exact state/rules identity to v9 and accepted only dungeon
    levels one through four;
  - extended the draw-free level switch and dungeon view with Level 4 while
    rejecting zero, five, and maximum unchanged;
  - retained the generic rejection loop, so Level 4 records every
    `Random(40)` candidate through the first result greater than 30, excludes
    boundary record 30, and initializes an accepted monster to strength 14,
    defence 7, and 42 HP;
  - added exact Level 4 failed-retreat draw/bound/damage coverage and v9 hostile
    state cases for prior schema, unsupported/wrong level, boundary, unknown
    record, wrong name, and oversized scalar.
- Provider and persistent profile:
  - decoded fixed `enter_dungeon_level_4`, added generic/fixed equivalence,
    replay, projected view, bounded-state, and return-to-Level-3 tests;
  - moved the permanent Gnoll/Cleric profile to Level 4 and retained the actual
    failed-retreat death followed by `reenter` across provider restart.
- Cartridge, fixtures, provenance, and docs:
  - advanced manifest rules/cartridge identity to v9;
  - added one inert Level 4 button and zero-field action to the existing signed
    dungeon screen;
  - changed dungeon/combat fixtures to visible Level 4 examples;
  - added the Level 4 canonical source trace, raised the checked trace floor to
    38, and reconciled README, compatibility limits, and the port map.
- Focused evidence:
  - data tests: PASS — 6;
  - rules tests: PASS — 35 unit plus 1 complete-day integration;
  - provider tests: PASS — 11;
  - `scripts/test-cartridge.sh`: PASS — seventeen signed screens and trusted
    QML state smoke;
  - `scripts/test.sh`: PASS — formatting, Clippy with warnings denied, 53 Rust
    tests, rustdoc, upstream checksums/clean source, 38-entry provenance,
    privacy scan, signed cartridge, and trusted QML;
  - `scripts/test-provider.sh`: PASS — fixed fifteen-case TLS/replay/fault/
    callback/reconciliation corpus twice across process restart.
- Implementation deviation/failure:
  - preserving Level 4's rejected encounter draws shifted the deterministic
    live profile's later retreat from Ticket 055's success to death; the
    source-faithful result was retained, and both unit/live drivers now assert
    `dead` then `reenter` rather than issuing `main_street` from an invalid
    phase.
- No platform application, SDK, database, migration, route, or QML source was
  changed.
- Phase 3 exit: the complete designed external file manifest is implemented
  and all focused plus external full/provider checks are green.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Determinism/correctness | Preserving Level 4 encounter rejection draws changes the permanent provider profile's later failed retreat into death. | expected behavioral delta | Retained the source-faithful result and asserted `dead` followed by `reenter` in both unit and live conformance drivers. |
| 2 | EARS/state integrity | Exact v9 Level 4 records, state/monster consistency, draw-free switching, accepted 31–39 band, 14/7/42 combat state, and retreat bound 40 all have direct tests. | none | PASS; all six requirements retain direct evidence and hostile-state rejection occurs before RNG mutation. |
| 3 | Simplification/authority | Level 4 reuses the generic table lookup, rejection loop, combat reducers, provider adapter, and signed dungeon screen. | none | PASS; no second game path or platform gameplay logic was added. |
| 4 | Security/privacy | Standard current-snapshot scan reviewed all 46 external files. | none | PASS; scan `cdfee76f-0634-4a52-9469-31ac1cbfad02` completed with zero reportable findings; report SHA-256 `b2d2b124f43b4b4095e2c59a95660255683133c16b3d138f224f04c536e73006`. |
| 5 | Dependency/deployment boundary | Authentication, replay, revision, session isolation, PostgreSQL transactionality, callback delivery, and TLS target-file controls are implemented by the excluded Provider SDK/starter or deployment. | review boundary | Preserved as explicit scan limitations/open questions, not misattributed to the game reducer and not treated as evidence of an in-scope vulnerability. |
| 6 | Cartridge/QML | The new control is a parameterless action in inert JSON; the tracked source becomes signed only when the platform packer produces an archive. | none | PASS; trusted rendering and signature enforcement remain platform-owned. |
| 7 | Platform blast radius | Fresh CodeGraph traced `translate_command` through provider execution, bounded view projection, render-plan preparation, and QML. | none | PASS; `enter_dungeon_level_4` remains opaque provider JSON, rules v9 remains catalog/provider metadata, and no platform code, SDK, QML, database, route, renderer, or migration change is needed. |

- The independent security architecture review reconciled the concrete
  provider listener, private configuration, PostgreSQL, callback, TLS,
  ProviderGame, cartridge, preview, provider-kit, and credential-test
  resources against their actual consumers. It confirmed the current game is
  the documented solo Provider Starter seam; the future shared-realm store is
  not silently claimed by this slice.
- The complete scan retained three follow-up boundaries for their owning
  release/deployment reviews: external SDK/starter controls, effective
  production topology and TLS target-file protections, and starter-owned
  pairwise-subject/receipt privacy. No in-scope attacker path or reportable
  game-repository finding survived validation.
- Phase 3.5 exit: inspection is clean, every finding or boundary question has
  an explicit disposition, and fresh CodeGraph plus security evidence covers
  the changed game/platform seam.

## Phase 4 — Validate

- Tests run:
  - external `scripts/test.sh`: PASS — formatting, Clippy with warnings
    denied, 53 Rust tests, rustdoc, upstream checksum/clean-source validation,
    38-entry provenance, privacy checks, seventeen signed screens, and the
    trusted-QML ready/loading/offline/empty/protocol-error matrix;
  - external `scripts/test-provider.sh`: PASS — the fixed fifteen-case TLS,
    replay, fault, callback, reconciliation, and restart corpus completed
    twice against PostgreSQL.
- Gate run:
  - platform `bin/gate.sh --diff`: PASS — all 24 numbered checks, including
    Rust/PostgreSQL/QML smoke, deterministic SDK and client packages, remote
    provider/sidecar security and durability, backup/restore, admission, and
    server-module isolation; worktree-bound receipt
    `20a4ae492fd338ef0be9f273c4276773b8c3ba95fb5a43871e5873503eac753c`.
- Visible preview:
  - a fresh signed Level 4 dungeon preview remains open from external run
    directory `.preview/run.SN1oZp`;
  - verified plan `omarchygs.render-plan/v1` is Core/ready, cartridge v9,
    title `The Dungeons`, narrates descent to dungeon level 4, and exposes
    `enter_dungeon_level_1` through `enter_dungeon_level_4` via the trusted QML
    renderer; the prior Level 3 preview was closed.
- Skips or pre-existing failures: none. The platform gate intentionally runs
  PostgreSQL-, QML-, provider-, backup-, and process-isolation-specific tests
  in their dedicated stages after Cargo's ordinary ignored-test pass.
- Requirement audit:

  | Requirement | Result |
  |---|---|
  | REQ-001 | PASS — exact Level 4 editor records and source links remain in data, provenance, compatibility docs, and checksum/clean-source checks. |
  | REQ-002 | PASS — strict v9 state rejects prior schema, unsupported levels, boundary/unknown/mismatched monsters, and oversized scalars before RNG mutation. |
  | REQ-003 | PASS — levels one through four switch draw-free, clear encounters, and expose exact labels; all other levels reject unchanged. |
  | REQ-004 | PASS — Level 4 preserves each `Random(40)` rejection, accepts only 31–39, spends one fight, and creates exact 14/7/42 combat state. |
  | REQ-005 | PASS — attack, retreat, potion, spell, class-special, rewards, and Gnoll poison compose through the existing suite with retreat bound 40. |
  | REQ-006 | PASS — provider replay/restart, signed cartridge, trusted QML, security scan, platform gate, and visible preview prove the bounded four-level slice without platform game logic or delivery. |

- Phase 4 exit: every EARS requirement has direct passing evidence and the
  signed Level 4 screen is visibly running; ready for documentation
  reconciliation and lifecycle completion.

## Phase 5 — Complete

- Acceptance-criteria audit: REQ-001 through REQ-006 remain PASS with no
  omitted scope or silent delivery expansion.
- Docs:
  - OpenWiki lifecycle `8a3f9f98-a07e-406c-9744-97e8fdb738ad` completed and
    updated `openwiki/quickstart.md` plus `openwiki/game-cartridges.md` for
    Tickets 048–056, rules v9, levels one through four, exact Level 4
    boundary/selection/combat initialization, and unchanged platform authority;
  - its warnings concerned pre-existing unresolved claim debt on the two broad
    pages, not a failed lifecycle or an unverified Ticket 056 requirement;
  - final permanent-source lifecycle
    `c6b31719-17e7-43ec-84dc-dcb3f9b1b29f` also completed after archive and
    reconciled the Level 4 facts against `docs/architecture/game-cartridges.md`
    plus the completed pipeline notes; it reported the same pre-existing claim
    debt warnings;
  - authoritative `docs/architecture/game-cartridges.md` now records Tickets
    047–056, rules v9, all four levels, boundary records 10/20/30, accepted
    bands 11–19/21–29/31–39, and Level 4's 14/7/42 combat state.
- AAR: AAR-056 is submitted and effective. Existing legacy-branch,
  discarded-RNG, composite-driver, private-command-file, and provider-authority
  knowledge covered the slice; no new knowledge ID was necessary.
- Archive: Ticket 056 is closed and its ticket, spec, and notes are under their
  closed/completed paths. Packaging, registration, admission, commit, push,
  deployment, and publication remain explicitly deferred.
- Phase 5 exit: documentation and AAR are reconciled, all six requirements pass,
  the pipeline is archived, and the visible signed Level 4 preview remains open.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Level 4 changed the live profile's retreat from success to death. | Preserved `Random(40)` rejection draws advanced the later deterministic tape. | Assert the source-faithful death and issue `reenter` in both unit and conformance drivers. | Reconcile full composite command profiles after adding earlier RNG work. |
