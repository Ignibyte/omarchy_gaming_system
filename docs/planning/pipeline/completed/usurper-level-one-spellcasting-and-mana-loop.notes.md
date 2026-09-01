---
title: Usurper level-one spellcasting and mana loop — notes
pipeline_id: e5b12d13-82f9-49e9-8d57-2e9a083778bb
---

# Usurper level-one spellcasting and mana loop — running notes

Chronological evidence and decisions. If a check did not run, these notes must
not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 050 completed rules/cartridge v3 with potion purchases and faithful
    configured heal-then-attack turns; no active pipeline remained;
  - `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` keeps
    learned spells, mana, and temporary combat state in the existing
    player-private provider snapshot;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires tracing the spell effect through resistance, cast choice, later
    turn resolution, and encounter reset rather than copying one damage line;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    prevents higher spells, magic items, monster magic, or specials from being
    silently composed with this level-one slice;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` applies
    because a successful cast consumes the player's combat choice and includes
    the monster response in one reducer operation.
- Canonical observations:
  - `USERHUNC.PAS` clears the spell matrix, sets spell 1 learned, and assigns
    starting mana 20/40/40 to Cleric/Magician/Sage;
  - `SPELLSU.PAS` limits spell users to those classes and gives every first
    spell level 1, cost 10, and the names Cure Light, Magic Missile, and Fog of
    War;
  - `CAST.PAS` deducts mana before resolving the class effect; Cure Light
    restores `4 + Random(3)`, Magic Missile rolls the same range as damage,
    and Fog of War marks the spell active so its duration check adds three
    points of absorption each combat round;
  - `CAST.PAS` maps magic resistance 1–10 to a `Random(20)=0` resist chance;
    the release has no initialized monster data, while the GPL editor exposes
    10 as its explicit magic-resistance field default;
  - `PLVSMON.PAS` treats an accepted cast as the player's choice for the turn,
    skips the physical attack, resolves duration effects, then allows a living
    monster to respond;
  - `DUNGEONC.PAS` resets active spell flags for each encounter and
    `MAINT.PAS` refills mana to maximum each realm day.
- Scope decision:
  - implement the complete level-one player spell/mana loop and expose it in
    the existing combat cartridge screen;
  - defer levels 2–12, monster spells, the Magic Shop object catalog, poison,
    and specials to independently traced slices;
  - keep packaging and any delivery action deferred as the user requested.

## Phase 2 — Design

- Canonical branch and compatibility declaration:
  - the source baseline remains the parentless v0.20e commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, non-classic mode, one normal
    level-one dungeon monster, and the existing deterministic provider seed;
  - creation preserves the original twelve-entry learned/active spell matrix:
    entry 1 is learned for every class even though only Cleric, Magician, and
    Sage receive the combat/status spell menu. The Rust snapshot therefore
    stores the original flag and separately enforces caster eligibility;
  - the release contains no initialized `MONSTER.DAT`. This slice uses the GPL
    editor's explicit `Magic Res` field default of 10 as a named development
    record fixture. The original 1–10 mapping produces a resistance check of
    `Random(20)=0`; this is not claimed as the publisher's absent live-world
    record or as a reconstruction of `ADDMONST.PAS` startup RNG;
  - `DUNGEONC.PAS` resets active flags immediately before a new normal
    encounter, not when the preceding encounter ends. The provider retains
    that observable ordering and simply ignores duration flags outside combat;
  - only spell ordinal 1 is implemented. Spell metadata and commands remain
    ordinal-shaped so later source-backed levels do not require a parallel API.
- Architecture and data flow:
  1. `usurper-model` adds `MAX_PLAYER_SPELLS = 12`, fixed learned/active boolean
     arrays to `Character`, a bounded `CastSpell { spell: u8 }` command, and
     `magic_resistance` to the one active `MonsterState`. No account/persona,
     clock, network, database, or shared-realm value enters the snapshot.
  2. `usurper-data` adds one immutable `SpellRecord` for each original caster
     class at ordinal 1, all level 1 and cost 10, plus the named development
     monster resistance 10 with exact source references. Effect logic remains
     in rules rather than becoming mutable catalog data.
  3. Creation clears both arrays then marks index zero learned exactly as
     `USERHUNC.PAS` does. `stats.mana` remains maximum mana; `character.mana`
     remains current mana. The existing class table already carries the exact
     20/40/40 caster launch values and zero for the other classes.
  4. Validation admits `CastSpell` only during `Combat`, for the three caster
     classes, ordinal 1, a learned nonactive spell, and at least ten current
     mana. Unknown/unlearned/wrong-class/wrong-phase/active/unaffordable input
     rejects before cloning/mutation and consumes no RNG or provider revision.
  5. An accepted cast deducts ten mana, then performs the source resistance
     draw with bound 20. Cure Light deliberately ignores the target resistance
     result—as the original self-heal branch does—and restores `4 + Random(3)`
     capped at maximum. Magic Missile always rolls `4 + Random(3)`; on a passed
     resistance check it also preserves the otherwise redundant `Random(2)`
     single-target narration draw before applying damage. A resisted missile
     spends mana and rolls damage but neither narrates a hit nor mutates HP.
     Fog of War marks spell 1 active only on a passed resistance check.
  6. Physical attack and monster response are split internally without
     changing public commands. Normal `Attack` computes player damage then
     calls the shared response helper. `CastSpell` skips physical damage and,
     if the monster remains alive, calls the same response helper. Fog of War
     contributes three absorption points before the configured body-armor
     absorption. Victory uses the existing reward draw order; death uses the
     existing terminal transition.
  7. `enter_encounter` clears all active spell flags before installing the
     next monster and records development magic resistance 10. Daily sleep
     refills current mana from `stats.mana`, after the existing level-up code
     has increased both consistently; it does not invent between-fight regen.
  8. The view adds `Mana current/max` to status and uses combat `option_a` for
     the class-specific level-one spell label/cost. The existing combat screen
     gains one fixed `cast_level_one_spell` button; noncasters see an explicit
     unavailable label and receive a rejected command if a hostile client
     sends the action. No conditional-QML or publisher execution is added.
  9. `usurper-provider` maps the fixed action to ordinal 1 while retaining the
     strict generic JSON command. Its gameplay profile uses a deterministic
     Magician path proven against the reducer before live conformance, then the
     unchanged starter supplies expected revision, idempotency, replay,
     restart, fault, callback, and bounded-state behavior.
- API and compatibility:
  - public Provider protocol v1, `ProviderGame`, Cartridge format v1,
    presentation protocol v1, and the view schema remain unchanged;
  - the unadmitted development rules/cartridge identity advances from 3 to 4
    because the durable character/monster schema and command behavior change.
    No v3 save migration or production compatibility promise is made;
  - command JSON remains deny-unknown-fields. `spell` is a `u8`, but only the
    source-known and implemented ordinal 1 passes semantic validation;
  - an accepted resisted spell is still a real turn and advances one provider
    revision; exact operation replay returns the stored transition and never
    redraws. Invalid casts remain provider `InvalidInput` and do not commit.
- Database and migration consequences:
  - no OmarchyGS schema, migration, route, compiled game, catalog admission,
    or writable gameplay copy changes;
  - no provider migration is needed because this is an unadmitted exact v4
    release in the starter's opaque bounded JSON state. The independent
    provider PostgreSQL database continues to own state and receipts;
  - no shared Usurper table, subject-aware rule seam, or realm transaction is
    introduced.
- Exact file manifest — adjacent Usurper repository:
  - `crates/usurper-model/src/lib.rs` — learned/active arrays, monster
    resistance, and strict cast command;
  - `crates/usurper-data/src/lib.rs` — three source-linked level-one spell rows
    and the named development resistance fixture;
  - `crates/usurper-rules/src/lib.rs` — rules v4 creation/validation, spell
    reducer, shared monster response, Fog absorption, mana refill, views, and
    source-distinguishing tests;
  - `crates/usurper-rules/tests/one_day.rs` — deterministic caster-day
    regression in addition to the existing noncaster path when useful;
  - `crates/usurper-provider/src/lib.rs` — fixed cast decoder and adapter
    coverage without changing the SDK trait;
  - `cartridge/{manifest,presentation}.json` and
    `fixtures/presentation/combat.json` — v4 inert action and visible Magician
    spell/mana facts; no new executable frontend or screen is needed;
  - `scripts/{test.sh,test-cartridge.sh,test-provider.sh,show.sh}` — raised
    source-trace floor, unchanged 17-screen signed/QML proof, v4 live profile,
    and visible combat selection;
  - `README.md`, `docs/{COMPATIBILITY,RUST_PORT_MAP}.md`, and
    `provenance/source-trace.json` — milestone statement, caveat, and exact
    creation/metadata/cast/turn/reset/maintenance trace links;
  - Ticket 051 spec/notes/AAR/index and later OpenWiki reconciliation in the
    platform repository — workflow evidence only.
- CodeGraph design evidence:
  - fresh pipeline-bound exploration confirmed `ProviderGame::command` and
    `view` accept game-neutral bounded JSON and provider-owned state, while the
    starter owns durable revision/replay outside the game reducer;
  - exact rules/cartridge versions are immutable release identity and signed
    declared actions compile into bounded host-owned `RenderPlan` nodes. One
    v4 action and additional view text require no platform trait, route,
    schema, migration, or QML source change;
  - the adjacent game repository is not CodeGraph-indexed. Direct inspection
    covered its Rust producers/consumers, JSON, scripts, and the exact Pascal
    `USERHUNC`, `SPELLSU`, `CAST`, `PLVSMON`, `DUNGEONC`, `MAINT`, and editor
    default branches.
- Risks and controls:
  - absent live monster record: label resistance 10 as a source-backed editor
    development fixture and retain the no-live-record compatibility caveat;
  - RNG drift: assert the exact 20, 3, optional 2, and existing response/reward
    bounds and indices for each class/pass/fail/terminal branch;
  - double attack: share only the monster-response helper, never call normal
    player attack after a cast, and update every fixed driver accordingly;
  - partial mana/HP mutation: perform all semantic validation before mutation,
    use checked arithmetic, and compare complete state on every rejection;
  - active-state leakage: clear all flags at the exact next-encounter boundary
    and assert Fog remains visible but inert after the current combat ends;
  - replay/concurrency: retain starter expected revision and operation receipt
    corpus twice across process restart with an accepted cast in profile;
  - privacy/trust: arrays, mana, resistance, and labels add no platform
    identity/credential shape; the cartridge remains inert signed data;
  - rollback: exact v3 and v4 identities remain distinct; no platform or
    provider migration requires reversal.
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | exact spell/resistance constants; Pascal line readback; source-trace entries; upstream hash/tree and compatibility checks |
  | REQ-002 | all-class creation flags/mana; wrong class/phase/ordinal/unlearned/active/insufficient-mana table; state/RNG equality; strict JSON |
  | REQ-003 | Cleric capped heal, Magician hit/resist/victory, Sage pass/resist/absorption, exact mana and RNG trace, deterministic twins |
  | REQ-004 | spell-plus-response and death fixtures; no physical attack; consecutive encounter active reset; daily mana refill; full-day regression |
  | REQ-005 | dynamic view labels/status; fixed/generic provider commands; deterministic profile; signed 17-screen render/QML smoke; live corpus twice; visible combat preview |
  | REQ-006 | dependency/privacy/content/scope scans; direct external inspection plus platform CodeGraph; external full checks; OpenWiki; platform diff gate |
- Material alternatives rejected:
  - reconstructing absent `MONSTER.DAT` values from displaced `ADDMONST` RNG
    was rejected because it would claim a world initialization sequence the
    release does not supply; the explicit editor default is auditable;
  - implementing Magic Missile only was rejected because creation and spell 1
    already define three distinct caster branches on the same turn seam;
  - treating a cast as a physical attack modifier was rejected because the
    original marks `casted` and suppresses the normal player strike;
  - clearing Fog immediately at victory was rejected because the original
    clears active flags before the next encounter, not at combat exit;
  - adding higher spells, monster magic, magic items, or platform QML was
    deferred because each introduces separate source branches or authority.

## Phase 3 — Implement

- Adjacent game repository implementation:
  - advanced the durable rules/cartridge identity to v4;
  - added the twelve learned/active spell flags, generic ordinal cast command,
    monster resistance field, three immutable spell rows, and the explicitly
    labeled resistance-10 editor fixture;
  - implemented caster-only preflight validation, mana spend, all three class
    effects, resistance pass/fail, source draw order, shared living-monster
    response, Fog absorption/reset, and daily mana refill;
  - added mana and the learned spell to the projected combat view, a fixed
    `cast_level_one_spell` provider action, and one inert signed combat button;
  - updated the deterministic provider profile, combat fixture, compatibility
    docs, port map, README, and source trace from 22 to 28 entries.
- Implementation correction:
  - the new nonnegative mana invariant exposed an existing translation error
    in the Singuman level-gain adjustment: Rust `saturating_sub(5)` made a
    zero noncaster gain become `-5`, while Pascal executes `dec(cp,5)` only
    when `cp>0`. The reducer now preserves that source guard and the existing
    level-up regression passes.
- Focused behavior proof:
  - `cargo test --workspace --all-features`: PASS — 3 data, 5 provider, 21
    rules, and 1 complete-day integration test passed, plus all doc tests;
  - spell coverage includes creation flags/mana and view labels, complete-state
    rejection immutability, all effects, resistance pass/fail, exact RNG bounds,
    Fog absorption/reset, same-turn response/death/victory, daily refill, and
    generic/fixed provider equivalence.
- Full adjacent repository proof:
  - `scripts/test.sh`: PASS — rustfmt, workspace Clippy with warnings denied,
    all tests, rustdoc, upstream hashes/tree cleanliness, 28-entry source
    trace, privacy scan, signed 17-screen cartridge conformance, and trusted
    headless QML states all passed;
  - `scripts/test-provider.sh`: PASS — the v4 Magician cast profile passed the
    fixed 15-case TLS/replay/fault/callback corpus twice across provider
    restart with the independent PostgreSQL database;
  - no platform server, SDK, migration, route, or QML application source was
    changed. Packaging, admission, commit, and publication remain deferred.
- Phase 3 exit: implementation and focused/full adjacent checks are green;
  ready for fresh cross-repository inspection. PASS.

## Phase 3.5 — Inspect

- Correctness finding and disposition:
  - inspection found that creation marked spell 1 learned only when the class
    had an implemented `SpellRecord`. The v0.20e source instead stores that
    first learned flag for every new character and separately gates the spell
    menu to Cleric, Magician, and Sage;
  - fixed creation to set learned index zero for every class, retained
    class-specific cast rejection and unavailable view text, and strengthened
    the creation test to prove both stored legacy state and access control;
  - `cargo fmt --all -- --check` and the focused all-class creation/access test
    passed after the correction.
- Security inspection:
  - the `codex-security:security-diff-scan` preflight passed. The hosted scan
    could not resolve a baseline because this is a new repository with no
    `HEAD`, so the required terminal workflow froze and reviewed all 46 files
    as a deterministic directory snapshot;
  - the review covered strict provider JSON/action decoding, provider-owned
    state, cast preflight and RNG order, checked arithmetic and bounds, private
    config/TLS/database handling, cartridge authority, scripts, locked Cargo
    resolution, fixtures, and provenance;
  - no reportable security finding survived discovery. The sealed scan has
    complete coverage and validates at
    `/tmp/codex-security-scans/omarchygs_usurper/no-head_20260831T000000Z/report.md`,
    snapshot
    `codex-security-snapshot/v1:sha256:e05da6ebbde91c8cd79016dafb7aad50753e3c64daf22212321922e38e501eee`;
  - the architecture review was sequential because delegated scan workers were
    unavailable. TAC advisory status was unknown because its connector was not
    logged in; neither limitation reduced the complete source inventory.
- Cross-repository and trust-boundary inspection:
  - a fresh worktree-bound CodeGraph exploration confirmed that the platform
    still exposes only bounded opaque game JSON through `ProviderGame`, pins
    exact rules/cartridge identity, and compiles declared actions into a
    platform-owned `RenderPlan`;
  - direct inspection of the adjacent game verified there is no platform
    account/persona shape, database handle, route, migration, server gameplay
    implementation, executable publisher QML, script, native code, or network
    authority in the rules or cartridge;
  - Cargo parsed the locked graph, every JSON document parsed, every shell
    script passed `bash -n`, all 28 source-trace entries resolve to the pinned
    source tree, and the existing signed-cartridge/provider proofs exercise the
    changed action through rendering and restart/replay.
- Finding ledger:

  | # | Lens | Finding | Severity | Disposition |
  |---|---|---|---|---|
  | 1 | Legacy correctness | Non-caster creation dropped the original stored learned-spell-one flag by coupling storage to caster access. | medium | FIXED — all classes store flag 1; only source-backed caster classes can view/cast it; focused test green. |
  | 2 | Security | No exploitable command, state, credential, cartridge-authority, replay, script, or provenance gap was found in the 46-file snapshot. | none | PASS — sealed complete-coverage scan, zero findings. |
  | 3 | Platform isolation | The v4 slice adds no gameplay logic or authority to the OmarchyGS server/QML application. | none | PASS — fresh CodeGraph receipt plus direct game inspection. |

- Phase 3.5 exit: every inspection finding is disposed, the corrected snapshot
  is source-faithful and security-clean, and validation may begin. PASS.

## Phase 4 — Validate

- Final adjacent-game snapshot:
  - `scripts/test.sh`: PASS after the inspection correction — rustfmt, Clippy
    with warnings denied, 30 Rust tests, rustdoc, all upstream checks, the
    28-entry source trace, privacy scan, all seventeen signed screens, and
    trusted headless QML states passed;
  - `scripts/test-provider.sh`: PASS — the accepted Magician cast remained in
    the live gameplay profile and all fifteen TLS/replay/fault/callback cases
    passed twice across process restart and independent PostgreSQL persistence.
- Platform validation:
  - `bin/gate.sh --diff`: PASS — all 24 canonical stages passed, including
    workspace Rust, shell and secret gates, deterministic Cartridge and both
    SDK releases, QML renderer/package/live smoke, PostgreSQL integration,
    provider security/sidecar/authority drills, backup/restore, and server
    module containment;
  - no platform runtime source was changed for this slice. The platform gate
    validates that the existing Provider SDK/starter and trusted cartridge
    renderer still accept the separate game without gaining a second rules
    owner or executable publisher presentation.
- Acceptance audit:

  | Requirement | Result |
  |---|---|
  | REQ-001 | PASS — six new trace entries bind creation, spell metadata, resistance, effects, combat order, reset, and daily mana to the pinned source; resistance 10 remains explicitly an editor development fixture rather than an absent live record. |
  | REQ-002 | PASS — all classes preserve the original first learned flag, only Cleric/Magician/Sage expose or accept it, class mana is exact, and invalid casts preserve complete state and RNG. |
  | REQ-003 | PASS — ten mana, Cure Light, Magic Missile hit/resist/victory and redundant narration draw, Fog activation/resistance/three absorption, caps, and exact RNG order are tested. |
  | REQ-004 | PASS — accepted casts replace the physical strike, a living monster responds, lethal magic skips response, new encounters clear duration flags, and sleep refills current mana. |
  | REQ-005 | PASS — status and combat views expose mana/spell data, generic and fixed provider commands agree, restart/replay conformance is green, and the signed combat fixture renders through trusted QML. |
  | REQ-006 | PASS — direct/CodeGraph/security review and the platform gate confirm no higher-spell/shared-realm/platform-rule/route/migration/QML-authority/package/admission/publication expansion. |

- Phase 4 exit: all six requirements have executed evidence, the adjacent and
  platform suites are green, and no validation exception remains. PASS.

## Phase 5 — Complete

- OpenWiki lifecycle run `7d6fb6fc-54e9-44ca-bb26-5e4042778273` returned
  `status: complete` after updating the quickstart and Game Cartridge page from
  Tickets 048–050/rules v3 to Tickets 048–051/rules v4 spell/mana evidence;
  finalization retained warnings for pre-existing unresolved Claims debt on
  those two large pages, but did not report an incomplete lifecycle.
- AAR-051 records the two source-fidelity defects found by new invariants and
  inspection, their prevention rules, the terminal security-scan fallback, and
  the unchanged provider/cartridge architecture decision. Every new `BF-` and
  `PR-` identifier is appended to the knowledge register.
- Ticket 051 is closed and the sole active spec/notes pair is archived under
  `docs/planning/pipeline/completed/`. Delivery remains intentionally absent:
  no commit, push, package, admission, deployment, or publication was requested
  for this slice.
- Phase 5 exit: requirements, durable docs, AAR, knowledge, and archive agree;
  the source-faithful v4 combat preview is ready to show. PASS.
