---
title: Usurper Assassin Backstab and Paladin Soul Strike — notes
pipeline_id: 207dd49f-1088-4058-bde5-f25cf2145a87
---

# Usurper Assassin Backstab and Paladin Soul Strike — running notes

Chronological evidence and decisions. If a check did not run, these notes must
not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 051 completed rules/cartridge v4 with the three original level-one
    caster branches, strict cast preflight, and same-turn monster responses;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires tracing each special from its menu/preflight branch through
    normal attack, monster response, and terminal outcomes;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps this slice on the existing solo, non-classic, normal-monster path;
  - `PR-omarchy-gaming-system-preserve-legacy-guards-before-safe-arithmetic-001`
    applies to Soul Strike HP investment and the conditional mental/addiction
    checks;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` applies
    because each accepted special includes the normal strike and a living
    monster's response in one reducer operation.
- Canonical observations:
  - `USERHUNC.PAS` initializes mental health to 100 and addiction to 0;
  - `PLVSMON.PAS` offers menu option `1` only to Paladins and Assassins;
  - Backstab requires a weapon and rolls `Random(3)` before normal attack;
    success adds half maximum HP, while failure zeroes player damage and adds
    level plus three to the monster's punch;
  - Soul Strike accepts 1 through current HP minus 1, deducts it immediately,
    conditionally checks weak mental health then addiction, and on success adds
    `Random(soul) + level` after normal attack is calculated;
  - both branches continue through the ordinary living-monster response.
- Scope decision:
  - implement both class specials on the existing solo dungeon-combat seam;
  - preserve generic variable Soul Strike investment and expose a bounded
    one-HP fixed cartridge action;
  - defer Gnoll poison, PvP/NPC variants, teams, mercy, dungeon events, and all
    packaging/admission/publication work.
- Preflight:
  - no active pipeline or critical bulletin existed;
  - pipeline tools reported CodeGraph 1.5.0 and OpenWiki 0.3.3 ready;
  - the platform PostgreSQL container was healthy;
  - direct source review covered `USERHUNC.PAS`, `PLVSMON.PAS`, and
    `VARIOUS.PAS`; the external v4 model/rules/provider/cartridge and Ticket
    051 notes supplied the implementation baseline.
- Phase 1 exit: ticket, active spec/notes, AAR, scope, locked decisions, and
  six observable EARS requirements recorded. PASS.

## Phase 2 — Design

- Canonical branch and compatibility declaration:
  - the source baseline remains the parentless v0.20e commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, non-classic mode, solo player,
    one normal level-one dungeon monster, and the existing deterministic
    provider seed;
  - `USERHUNC.PAS:845-895` initializes `mental=100` and `addict=0`. Those
    scalars are part of the original durable player record and must be stored
    separately from the Soul Strike access gate;
  - `PLVSMON.PAS:722-989` exposes one menu key for either Assassin or Paladin.
    The Rust API keeps distinct typed commands because Backstab has no player
    parameter while Soul Strike accepts an HP amount;
  - Backstab's `Random(3)` occurs before the ordinary attack is calculated.
    Success adds `maxhps div 2` to that already calculated attack. Failure
    still calculates the ordinary attack, then discards all player damage,
    skips the monster-defence display draw, and increases the monster's same-
    turn punch by `player.level+3`;
  - Soul Strike deducts the selected HP before conditional failure checks. A
    mental value below 50 may consume bounds `3, mental, mental`; if the strike
    survives, addiction above 50 may consume `3, addict, addict`. The ordinary
    attack is calculated next and successful soul damage then consumes
    `Random(investment)` and adds level before the monster-defence draw;
  - the initial 100/0 values skip both failure checks. Direct deterministic
    fixtures will exercise degraded values without claiming an implemented
    drug, steroid, or recovery system;
  - accepted special commands own the complete player choice and same-turn
    response. They are not modifiers that can be separately replayed or
    followed by another attack command at the same provider revision.
- Architecture and data flow:
  1. `usurper-model` adds bounded `mental_health` and `addiction` fields to
     `Character`, plus `Backstab` and `SoulStrike { hit_points: i32 }` command
     variants. No account/persona, clock, database, or shared-realm value enters
     the snapshot.
  2. Creation stores mental health 100 and addiction 0. State validation keeps
     both in `0..=100`; Soul Strike validation separately enforces Paladin,
     combat, and `1..current_hp` exclusive of the current-HP endpoint.
  3. The existing attack reducer is split into an ordinary power calculation,
     a hit/defence/victory resolution, and a monster response that accepts a
     source-backed extra punch. Plain attack and configured quick-heal retain
     byte-identical draw order and outcomes.
  4. Backstab first rolls bound 3. A successful result 0 computes ordinary
     power, checked-adds half maximum HP, and resolves the ordinary defence,
     victory, or living response. A failed result computes and discards
     ordinary power, does not draw defence, and resolves a response with
     `level+3` extra pre-absorption monster punch.
  5. Soul Strike immediately subtracts the accepted investment. Conditional
     mental and addiction checks run in source order and may reduce the bonus
     to zero without refund. Ordinary power is always computed; a surviving
     investment then draws its bound and adds the current level. The combined
     punch uses the same defence/victory/response helper as plain attack.
  6. Monster extra punch is applied before existing Fog and body-armor
     absorption, matching the original mutation of `monster.punch` before
     defensive effects. Arithmetic remains checked until the established
     nonnegative damage saturation point.
  7. The combat view uses `option_b` for the class special: Backstab for an
     armed Assassin, an explicit weapon-required label for an unarmed
     Assassin, one-HP Soul Strike for a Paladin with at least two HP, and an
     unavailable label otherwise. Static hostile clicks still fail reducer
     preflight.
  8. `usurper-provider` maps one fixed `use_class_special` action from the
     already authenticated/decoded current character class to the same typed
     Backstab or one-HP Soul Strike command accepted through strict generic
     JSON. The live conformance profile becomes an Assassin, retains its
     existing dagger purchase/equip path, and executes Backstab before
     retreating if combat remains active.
  9. The signed inert combat screen adds one bounded button. The development
     combat fixture visibly shows an armed Assassin and Backstab result while
     all seventeen existing screen/QML states remain covered.
- API and compatibility:
  - public Provider protocol v1, `ProviderGame`, Cartridge format v1,
    presentation protocol v1, and the view schema remain unchanged;
  - the unadmitted development rules/cartridge identity advances from 4 to 5
    because durable character state and command behavior change. No v4 save
    migration or production compatibility promise is made;
  - generic command JSON remains deny-unknown-fields. Soul Strike uses signed
    `i32` input at decode but semantic preflight accepts only the source range;
  - rejected specials remain provider `InvalidInput` and consume no RNG or
    revision. An accepted special advances one revision, and exact operation
    replay returns the stored complete transition without redrawing.
- Database and migration consequences:
  - no OmarchyGS schema, migration, route, compiled game, catalog admission,
    or writable gameplay copy changes;
  - no provider migration is needed because this remains an unadmitted exact
    v5 release stored as bounded opaque JSON in the independent provider
    PostgreSQL database;
  - no shared Usurper table, subject-aware rule seam, or realm transaction is
    introduced.
- Exact file manifest — adjacent Usurper repository:
  - `crates/usurper-model/src/lib.rs` — two source scalars and two typed special
    commands;
  - `crates/usurper-rules/src/lib.rs` — rules v5 creation/validation, factored
    attack stages, Backstab, Soul Strike, response bonus, view labels, and
    source-distinguishing unit tests;
  - `crates/usurper-provider/src/lib.rs` — state-routed fixed action decoder and strict
    generic/fixed equivalence tests without changing the SDK trait;
  - `cartridge/{manifest,presentation}.json` and
    `fixtures/presentation/combat.json` — v5 identity, inert actions, and
    visible armed-Assassin facts; no executable frontend code;
  - `scripts/{test.sh,test-provider.sh}` — raised source-trace floor, updated
    milestone wording, and an Assassin special in the unchanged security
    corpus twice across restart;
  - `README.md`, `docs/{COMPATIBILITY,RUST_PORT_MAP}.md`, and
    `provenance/source-trace.json` — milestone statement, caveats, and exact
    creation/menu/Backstab/Soul-Effect/turn trace links;
  - Ticket 052 spec/notes/AAR/index and later OpenWiki reconciliation in the
    platform repository — workflow evidence only.
- CodeGraph design evidence:
  - fresh pipeline-bound exploration confirmed `ProviderGame::command`
    receives only authenticated current opaque game state plus bounded JSON,
    while the starter owns durable revisions/replay outside the reducer;
  - exact provider/game/rules/cartridge identity is pinned around that narrow
    trait, and the generic conformance profile can add a new game-owned command
    without changing platform vocabulary or security cases;
  - the adjacent game repository is not CodeGraph-indexed. Direct inspection
    covered its Rust producers/consumers, JSON and shell drivers, plus the exact
    Pascal `USERHUNC`, `PLVSMON`, and `VARIOUS` branches.
- Risks and controls:
  - attack-order regression: factor helpers without changing plain-attack
    draws and compare complete normal/quick-heal transitions before and after;
  - failed-Backstab drift: assert ordinary power draws still occur, defence
    does not, player damage is zero, and the response bonus is absorbed only
    after it is added;
  - Soul Strike refund/order drift: assert investment remains spent across
    mental/addiction failure, all conditional bounds/indices, and ordinary
    attack continuation;
  - invalid-state injection: validate both new scalars before every command or
    view and reject out-of-range persisted JSON as internal corruption;
  - lethal ordering: separately prove special victory skips monster response
    and post-investment low HP can die to a living response;
  - replay/concurrency: retain starter expected-revision and operation-receipt
    corpus twice across restart with an accepted Backstab in the profile;
  - UI ambiguity: use class-specific bound labels and reducer rejection rather
    than implying that every class can use both static action nodes;
  - privacy/trust: the two gameplay scalars and labels add no identity or
    credential shape; the cartridge remains inert signed data;
  - rollback: exact v4 and v5 identities remain distinct and no platform or
    provider schema must be reversed.
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | creation/menu/special constants; Pascal line readback; source-trace entries; upstream hash/tree and compatibility checks |
  | REQ-002 | class/phase/weapon/investment rejection table; state/RNG equality; persisted scalar bounds; strict generic/fixed JSON |
  | REQ-003 | Backstab success/failure/victory/response fixtures; exact bound order; normal-power consumption; response bonus/absorption |
  | REQ-004 | default and degraded mental/addiction paths; HP spend/no refund; bonus order; victory/death; deterministic twins |
  | REQ-005 | dynamic combat labels; provider generic/fixed commands; deterministic Assassin profile; signed 17-screen/QML smoke; live corpus twice; visible preview |
  | REQ-006 | dependency/privacy/content/scope scans; direct external inspection plus platform CodeGraph; external full checks; OpenWiki; platform diff gate |
- Material alternatives rejected:
  - a single untyped reducer `class_special` command was rejected because it
    would make Backstab silently accept an irrelevant HP parameter. The inert
    fixed action may safely route to distinct typed commands from the already
    authenticated current snapshot before reducer preflight;
  - implementing only Backstab was rejected because the original assigns the
    same menu seam to Paladin and Assassin and both compose through the same
    attack stages;
  - omitting mental/addiction state was rejected because it would preserve only
    the current default outcome while making the translated Soul Strike logic
    structurally incompatible with later original systems;
  - adding Gnoll poison was deferred because its monster duration state and
    later poison effect are a separate passive race branch, not menu option 1;
  - starting random dungeon events was deferred because `Dungeon_Event`
    composes a wishing-well probe with a second eleven-way encounter probe; a
    partial event dispatcher would silently alter original look/fight behavior.
- Phase 2 exit: architecture, exact file manifest, compatibility boundary,
  risks, alternatives, and every requirement-to-evidence mapping are
  actionable; a fresh CodeGraph design receipt exists for this pipeline. PASS.

## Phase 3 — Implement

- Adjacent game repository implementation:
  - advanced the durable rules/cartridge identity to v5;
  - added bounded mental-health/addiction state with the exact 100/0 creation
    defaults, typed Backstab and variable-HP Soul Strike commands, and strict
    class/phase/weapon/HP preflight;
  - split normal attack into source-order power, hit/defence/victory, and
    response stages without changing the existing plain-attack path;
  - implemented Backstab's pre-attack bound-3 roll, half-max-HP success bonus,
    failed strike suppression, and level-plus-three response bonus;
  - implemented immediate Soul Strike HP spend, conditional mental then
    addiction checks, no-refund failures, ordinary attack continuation, and
    `Random(investment)+level` successful bonus before defence;
  - added a dynamic class-special label and one inert `use_class_special`
    button, state-routed by the provider adapter to typed Backstab or one-HP
    Soul Strike while generic commands retain variable investment;
  - changed the deterministic live profile to an Assassin that buys/equips a
    dagger and uses Backstab, and updated the visible combat fixture, README,
    compatibility ledger, port map, and source trace from 28 to 31 entries.
- Implementation correction:
  - the initial model edit placed mental/addiction fields in the reusable
    `Stats` row rather than `Character`; the first compile rejected every
    existing `Stats` initializer. The fields were moved to the correct durable
    player record before behavior testing;
  - Clippy then reported `validate_command` above the project line budget. The
    special-specific checks were extracted into `validate_combat_special`
    rather than suppressing the lint.
- Focused behavior proof:
  - `cargo test --workspace --all-features`: PASS — 3 data, 6 provider, 24
    rules, and 1 complete-day integration test passed, plus all doc tests;
  - coverage includes exact creation defaults, complete rejection immutability,
    Backstab success/failure/victory and draw order, Soul Strike default and
    degraded mental/addiction paths, HP spend/no refund, victory/death,
    dynamic labels, malformed JSON, and generic/fixed provider equivalence;
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
    PASS after the validation-helper extraction.
- Full adjacent repository proof:
  - `scripts/test.sh`: PASS — rustfmt, Clippy with warnings denied, 34 Rust
    tests, rustdoc, upstream hashes/tree cleanliness, 31-entry source trace,
    privacy scan, signed 17-screen cartridge conformance, and trusted headless
    QML states passed;
  - `scripts/test-provider.sh`: PASS — the v5 armed-Assassin Backstab profile
    passed the fixed 15-case TLS/replay/fault/callback corpus twice across
    provider restart with the independent PostgreSQL database;
  - no platform server, SDK, migration, route, or QML application source was
    changed. Packaging, admission, commit, and publication remain deferred.
- Phase 3 exit: implementation plus focused/full adjacent checks are green;
  ready for fresh cross-repository inspection. PASS.

## Phase 3.5 — Inspect

- Legacy correctness inspection:
  - direct readback of `PLVSMON.PAS:1084-1143` and `1186-1220` confirmed the
    implemented shared menu seam: Backstab rolls before ordinary attack,
    success adds half maximum HP, failure discards ordinary power and adds
    level plus three to the monster's same-turn punch; Soul Strike deducts HP
    before mental/addiction checks and a failed bonus still falls through to
    the ordinary attack;
  - direct readback of `VARIOUS.PAS:1423-1429` confirmed successful Soul Strike
    contributes `Random(investment)+level`; `USERHUNC.PAS:850-870` confirmed
    exact creation values mental 100 and addiction 0;
  - the factored Rust stages preserve those source-relative boundaries:
    Backstab failure consumes ordinary-power draws but no defence draw, the
    response bonus is added before existing armor/Fog absorption, and Soul
    Strike never refunds its accepted HP investment.
- Security inspection:
  - `codex-security:security-diff-scan` preflight passed. The repository still
    has no resolvable `HEAD`, so the required terminal workflow reviewed all 46
    files as a deterministic directory snapshot using the prior sealed full
    review as its baseline and a focused source-to-sink review of every Ticket
    052 surface;
  - review covered strict generic/fixed JSON, provider-owned state routing,
    phase/class/weapon/HP preflight, mental/addiction invariants, checked
    arithmetic, bounded deterministic RNG, provider private configuration,
    inert cartridge authority, shell/JSON inputs, locked Cargo resolution, and
    source provenance;
  - no reportable security finding survived discovery. The sealed complete-
    coverage report is
    `/tmp/codex-security-scans/omarchygs_usurper/no-head_20260831T192524Z/report.md`,
    snapshot
    `codex-security-snapshot/v1:sha256:1e591e299d8152e0b4eaa4d57d190839db5ba0c56c0dc7e0d2d50764597bf269`;
  - scan limitations are explicit: the no-`HEAD` repository required terminal
    snapshot review, delegated scan workers were unavailable, TAC availability
    could not be verified, and the adjacent Provider SDK/starter internals were
    exercised by conformance rather than rescanned.
- Cross-repository and trust-boundary inspection:
  - a fresh worktree-bound CodeGraph exploration confirmed `ProviderGame`
    receives only authenticated current opaque JSON state plus a JSON command,
    while the starter owns revision/replay and exact provider/game/rules/
    cartridge identity;
  - the existing signed presentation contract remains a closed list of data
    screens/actions. Adding the external `use_class_special` action and its
    provider-owned reducer behavior requires no platform protocol, SDK,
    server, migration, or QML application change;
  - direct inspection covered the unindexed adjacent Rust, JSON, shell, and
    Pascal surfaces. Cargo parsed the locked graph, all JSON documents parsed,
    every shell script passed `bash -n`, no forbidden key/certificate artifact
    was present, and the exact v5 identity/31-entry trace remained consistent.
- Finding ledger:

  | # | Lens | Finding | Severity | Disposition |
  |---|---|---|---|---|
  | 1 | Legacy correctness | No mismatch found in Backstab/Soul Strike eligibility, spend, draw, attack, response, victory, or death order. | none | PASS — direct Pascal readback plus deterministic Rust fixtures. |
  | 2 | Command integrity | A fixed caller payload cannot select a foreign class special; it is routed from provider-owned current class and rechecked by the reducer before mutation/RNG. | none | PASS — malformed/wrong-class/equipment/HP tests and generic/fixed equivalence. |
  | 3 | Arithmetic and availability | Mental/addiction, HP investment, response bonus, state size, and RNG trace remain bounded; no new loop, allocation, filesystem, network, or credential authority exists. | none | PASS — manual source-to-sink review and sealed security scan. |
  | 4 | Platform isolation | The v5 game behavior fits the existing opaque provider and declarative cartridge boundaries. | none | PASS — fresh CodeGraph receipt; no Ticket 052 platform application source change. |

- Phase 3.5 exit: every inspection lens is disposed with no correction needed;
  the sealed snapshot is source-faithful and security-clean, and validation may
  begin. PASS.

## Phase 4 — Validate

- Final adjacent-game snapshot:
  - `scripts/test.sh`: PASS — rustfmt, Clippy with warnings denied, 34 Rust
    tests, rustdoc, pinned upstream hashes/tree cleanliness, all 31 source-
    trace entries, privacy scan, all seventeen signed screens, and trusted
    headless QML states passed;
  - `scripts/test-provider.sh`: PASS — the armed-Assassin Backstab profile
    passed all fifteen TLS/replay/fault/callback cases twice across provider
    restart and independent PostgreSQL persistence;
  - no game file changed after those full proofs; the subsequent complete
    security snapshot, shell/JSON/Cargo checks, and direct inspection all
    matched the tested v5 digest.
- Platform validation:
  - `bin/gate.sh --diff`: GATE GREEN — all 24 canonical stages passed,
    including workspace Rust, secret/shell/hook gates, deterministic Cartridge
    and both SDK releases, trusted QML renderer/package/live smoke, PostgreSQL
    integration, provider TLS/replay/sidecar/authority drills, backup/restore,
    private-alpha admission, and server-module containment;
  - the reproducible native client package produced identical hashes in both
    builds, and the live PostgreSQL suite passed 66 server API cases plus the
    provider, operator, QML, and isolation drills;
  - Ticket 052 changes only the separate unadmitted Usurper game repository
    and platform workflow evidence. The existing opaque Provider SDK/starter
    and signed declarative cartridge renderer accept the feature without a new
    platform rules owner or executable publisher presentation.
- Acceptance audit:

  | Requirement | Result |
  |---|---|
  | REQ-001 | PASS — the 31-entry trace and direct readback bind exact creation defaults, shared menu gate, Backstab, Soul Strike, Soul Effect, ordinary attack, and response sources to the pinned clean v0.20e tree. |
  | REQ-002 | PASS — phase/class/weapon/investment boundaries and hostile JSON reject without mutation/RNG; persisted mental/addiction values are bounded and generic/fixed actions are strict. |
  | REQ-003 | PASS — success, failure, lethal victory, ordinary-power draw consumption, skipped defence draw, half-max-HP bonus, and level-plus-three response bonus are deterministic and source ordered. |
  | REQ-004 | PASS — HP spend precedes conditional mental/addiction draws, failures receive no refund and retain ordinary attack, successful bonus order is exact, and victory/death terminal paths are covered. |
  | REQ-005 | PASS — dynamic labels, state-routed fixed action, generic/fixed equivalence, signed 17-screen/QML smoke, and the live restart/replay profile all passed. |
  | REQ-006 | PASS — complete security/scope review and fresh CodeGraph show no Gnoll/PvP/team/event/shared-realm/platform gameplay expansion, migration, packaging, admission, or publication. |

- Phase 4 exit: every EARS requirement has executable and inspection evidence;
  the adjacent game proofs and full platform delivery gate are green. PASS.

## Phase 5 — Complete

- OpenWiki lifecycle run `9e013672-41c4-4e2c-92fd-978dc2981bef` returned
  `status: complete` after updating the quickstart and Game Cartridge page from
  Tickets 048–051/rules v4 to Tickets 048–052/rules v5 Backstab/Soul Strike
  evidence; finalization retained warnings for unresolved Claims debt on those
  two broad pages, but did not report an incomplete lifecycle.
- The signed combat preview opened successfully through `scripts/show.sh
  combat` and remains visible on the desktop. Its provider-owned current class
  routes the one inert special action to Backstab for an armed Assassin or a
  one-HP Soul Strike for an eligible Paladin; ineligible classes do not gain a
  fabricated command.
- AAR-052 records the implementation corrections, the complete terminal
  security review, and the unchanged opaque-provider/declarative-cartridge
  architecture. No new durable `BF-`, `PR-`, or `AD-` identifier was needed.
- Ticket 052 is closed and the sole active spec/notes pair is archived under
  `docs/planning/pipeline/completed/`. Delivery remains intentionally absent:
  no commit, push, package, admission, deployment, or publication was requested
  for this slice.
- Phase 5 exit: requirements, visible behavior, durable docs, AAR, and archive
  agree; the source-faithful v5 class-special combat slice is complete. PASS.
