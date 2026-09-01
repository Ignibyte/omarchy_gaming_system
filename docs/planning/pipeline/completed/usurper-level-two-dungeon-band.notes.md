---
title: Usurper Level-Two Dungeon Band — notes
pipeline_id: 9dc365e9-59e0-43fd-8d35-b64ba987a528
---

# Usurper Level-Two Dungeon Band — running notes

Chronological evidence and decisions. If a check did not run, these notes must
not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 053 completed rules/cartridge v6 with Gnoll poison across the
    existing level-one combat paths and left broader dungeon bands open;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires retaining the level-band rejection loop and its unused boundary
    record rather than simplifying the observed result set;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps this slice in solo non-classic normal dungeon combat;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001`
    applies to rejected encounter candidates because they still advance every
    later random result.
- Canonical observations:
  - `DUNGEONC.PAS` initializes the dungeon level from player level, exposes an
    in-dungeon change-level menu, and permits the original player-level through
    player-level-plus-ten range;
  - normal encounter selection calculates `(level-1)*10` and `level*10`, then
    repeats `Random(upper)` until the candidate is strictly greater than the
    lower boundary;
  - `EDMONST.PAS` stores ten level-two records at indices 10–19, all with
    reviewed base strength 12; record 10 is therefore present but unreachable
    through the normal level-two loop;
  - `PLVSMON.PAS` initializes each loaded monster's HP to strength times three.
- Scope decision:
  - translate level two, preserve both bands' exact record ordering and draw
    loop, and expose fixed level-one/level-two controls inside the dungeon;
  - compose the already translated combat, reward, spell, special, poison, and
    retreat paths without importing dungeon events or any shared-world state;
  - defer levels three-plus, special areas, packaging, admission, deployment,
    and publication.
- Preflight:
  - no active pipeline or open ticket existed; Ticket 054 was next;
  - pipeline tools were ready and the platform PostgreSQL container was
    healthy during the immediately preceding completed slice;
  - direct source review covered the full dungeon level-change branch,
    encounter loop, level-two editor records, and combat HP initialization;
    the external v6 model/rules/provider/cartridge and Ticket 053 notes supply
    the implementation baseline.
- Phase 1 exit: ticket, active spec/notes, AAR, scope, locked decisions, and
  six observable EARS requirements recorded. PASS.

## Phase 2 — Design

- Declared compatibility mode and source trace:
  - solo, non-classic normal dungeon combat with one player and one monster;
    no event dispatch, teammates, shared realm, or special-area branch;
  - canonical target remains Usurper v0.20e parentless commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, under
    `upstream/v0.20e/source-git/SOURCE` in the external repository;
  - `DUNGEONC.PAS:579-586` initializes the legacy dungeon to player level;
    `748-804` handles in-dungeon level changes and describes the original
    player-level through player-level-plus-ten access window;
  - `DUNGEONC.PAS:946-955` spends a fight, computes the band boundaries, and
    repeats `Random(level*10)` until the result is strictly greater than
    `(level-1)*10` before loading that record;
  - `EDITOR/EDMONST.PAS:2702-2800` defines indices 10–19 for level two, all at
    reviewed base strength 12, with the exact names and armor/weapon flags;
  - `PLVSMON.PAS:614-625` initializes every loaded monster's HP to strength
    times three with the original lower clamp, which is inactive for this
    positive-strength band.
- Level-two canonical data:
  - 10 Small Troll `(armor, weapon) = (true, true)`;
  - 11 Insane Ape `(false, false)`;
  - 12 Giant Gnoll `(true, true)`;
  - 13 Angry Centipede `(false, false)`;
  - 14 Small Spider `(false, false)`;
  - 15 Assassin `(true, true)`;
  - 16 Weak Dwarf `(true, true)`;
  - 17 Amazon `(true, true)`;
  - 18 Insane Eagle `(false, false)`;
  - 19 Rabid worm `(false, false)`.
- State, command, and RNG flow:
  1. Exact schema v7 accepts only dungeon levels one and two. A live monster
     must have the same level as the session, a selectable catalog index inside
     that level's stored band, its canonical name, positive bounded combat
     scalars/HP, bounded resistance, and existing poison state. Creation still
     derives exact strength, defence, and maximum HP from the canonical seed.
  2. `EnterDungeon { level }` is accepted from Main Street and Dungeon for
     levels one/two, clears any absent-by-phase encounter, changes location,
     and consumes no RNG. Other levels fail before cloning or RNG creation.
  3. Look clears encounter spells and spends one fight, then retains every
     rejected candidate in `last_rng_trace`; level two uses bound 20 and
     accepts only 11–19. The selected seed produces strength 12, defence 6,
     and 36 HP.
  4. Existing attack/spell/special/poison/reward code consumes only the active
     monster facts. Existing retreat already derives its damage bound from
     `dungeon_level`, so level two must prove the bound is 20.
  5. Provider fixed actions choose level one/two; the dungeon signed screen
     renders both level controls plus Look, combat navigation, Quick Heal, and
     Return using existing inert button and binding vocabulary.
- Implementation manifest:

  | Boundary | Planned change | Compatibility |
  |---|---|---|
  | external data | add exact level-two records and one 0–19 lookup | editor order and unused index 10 retained |
  | external rules | accept levels one/two, enforce level-consistent canonical monsters, preserve encounter rejection loop | state/rules v7; v6 deliberately rejected |
  | external provider | decode fixed level-two action and test generic/fixed/replay/view paths | provider protocol unchanged |
  | external cartridge | add two in-dungeon level controls, sign identity v7, and show level-two fixtures | presentation protocol and node types unchanged |
  | external provenance/docs/scripts | trace sources, exercise a level-two live profile twice, and reconcile port/test claims | no production/admission claim |
  | platform | no application, SDK, schema, migration, route, or QML source change | opaque provider state/view boundary already fits |
- Regression plan:

  | Case | Required evidence |
  |---|---|
  | level-two table | exact indices, names, base strength, and armor/weapon flags including record 10 |
  | level switch | Main Street and Dungeon accept one/two without RNG; zero/three rejected unchanged |
  | encounter selection | rejected 0–10 candidates remain in trace; accepted candidate is 11–19; record 10 never selected |
  | state validation | level/index/name/strength/defence/HP inconsistencies and schema v6 reject |
  | combat composition | attack and one spell/special/poison path retain behavior against a level-two seed |
  | retreat | failed level-two retreat records bounds 2 then 20 and applies result plus three |
  | provider | generic and fixed level-two commands match, replay is identical, projected status/narrative identify level two |
  | cartridge | all signed screens conform; level-two dungeon/combat fixtures render under trusted QML |
  | platform | existing provider starter remains opaque and full diff gate passes |
- Risks and alternatives:
  - direct sampling from a nine-row slice was rejected because it would erase
    the source's candidate rejections and shift later deterministic draws;
  - dropping record 10 was rejected because the editor record exists even
    though the normal selection branch cannot reach it;
  - accepting levels above two with placeholder monsters was rejected as a
    false compatibility claim;
  - importing `dungeon_event` was rejected because Ticket 052 already proved
    that its dispatcher is composite and partial event dispatch changes
    command semantics.
- CodeGraph design evidence inspected the platform `ProviderGame` seam,
  provider `GameState { status, state: Value }`, server game command storage,
  and provider routing. It confirms that game state remains opaque JSON and
  the trusted cartridge consumes bounded projected facts, limiting runtime
  edits to the external Usurper repository.
- Phase 2 exit: canonical source trace, version posture, state/action/view
  manifest, regression matrix, platform blast radius, and rejected alternatives
  are actionable. PASS.

## Phase 3 — Implement

- External data/rules changes:
  - advanced exact game-state/rules identity from v6 to v7;
  - added all ten level-two editor seeds at indices 10–19 with exact names,
    reviewed base strength 12, and armor/weapon-user flags;
  - extended the lookup across indices 0–19 while keeping source metadata on
    the normal editor records;
  - accepted only dungeon levels one/two from Main Street or Dungeon, retained
    ascend/descend/remain narration, cleared no active encounter by phase
    invariant, and consumed no RNG for a change;
  - retained the existing rejection loop unchanged, so level two records every
    `Random(20)` candidate through the first value 11–19 and never normally
    selects boundary record 10;
  - strengthened active-monster validation around level/band/index/name,
    scalar/view bounds, positive HP, resistance, and exact schema while
    preserving testable bounded live combat values;
  - composed existing attack, retreat, potion, spell, special, reward, and
    Gnoll poison paths with the level-two monster facts and bound-20 retreat.
- Provider/cartridge/provenance changes:
  - decoded fixed `enter_dungeon_level_2`, added generic/fixed equivalence,
    replay/view, deterministic death/re-entry, and bounded-state tests;
  - advanced the unadmitted cartridge to rules/cartridge v7, added inert
    level-one/level-two dungeon buttons, and changed dungeon/combat fixtures to
    visibly demonstrate the deeper band;
  - changed the live conformance profile to enter level two, cast as a Gnoll
    Cleric, exercise the deterministic failed-retreat death, re-enter, and
    still finish the BBS day twice across provider restart;
  - added separate canonical trace entries for the level-two table, level
    controls, and rejection loop, and reconciled the README, compatibility
    ledger, port map, and test summary.
- Focused checks:
  - `cargo test --workspace --all-features`: PASS after the permanent profile
    regression was added (45 tests across data/provider/rules/one-day);
  - focused permanent provider profile regression: PASS as part of that full
    workspace run;
  - `scripts/test-cartridge.sh`: PASS for all seventeen signed screens and
    trusted QML state smoke;
  - `scripts/test-provider.sh`: initially FAIL at 422 after a source-faithful
    level-two retreat killed the player and the inherited profile incorrectly
    issued `main_street`; after changing that driver step to `reenter`, PASS for
    the fixed 15-case corpus twice across restart.
- Deviations:
  - the original permits player level through player level plus ten, but this
    reviewed development release deliberately exposes only levels one/two and
    rejects unreviewed higher bands instead of fabricating their data;
  - no runtime `MONSTER.DAT` is claimed; source editor base values remain the
    explicit fixture, as in the prior level-one band.
- Phase 3 exit: the v7 data/rules/provider/cartridge path is implemented and
  visible with focused deterministic, signed-rendering, and restart evidence.
  PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Deterministic level transition | `EnterDungeon` accepts only one/two before clone or RNG creation, clears an encounter by phase invariant, and preserves exact state on rejection. | — | No issue found; reducer tests prove draw-free switching and immutable zero/three rejection. |
| 2 | Monster authority and bounds | An active monster must match the selected dungeon level, strict selectable catalog band, source-backed name, positive bounded HP/scalars, and schema v7. | — | No issue found; malformed level/index/name/scalar/schema cases reject before command or view use. |
| 3 | Rejection-loop availability and replay | Level-two selection can reject 0–10 repeatedly, but every draw checks the 64-entry cap and a failed reducer call returns no cloned state or RNG cursor. | — | No issue found; trace/replay tests cover rejected draws, accepted 11–19, and unreachable record 10. |
| 4 | Provider/action boundary | The exact fixed level-two action and generic command both reach the same reducer validation; current state stays provider-owned and output remains capped at 32 KiB. | — | No issue found; generic/fixed equivalence, replay, view, and full profile regressions pass. |
| 5 | Cartridge/rendering authority | The v7 cartridge adds inert fixed actions and bounded level-two facts only; it gains no executable code, filesystem, network, credential, or platform-identity authority. | — | No issue found; signed cartridge and trusted-QML smoke pass for all seventeen screens. |
| 6 | Complete security snapshot | The parentless 46-file repository snapshot and Ticket 054 source-to-sink paths were reviewed with an independent architecture map. | — | Zero reportable findings; after the final lint-only helper extraction, the exact validated snapshot was resealed at `/tmp/codex-security-scans/omarchygs_usurper/no-head_20260831T222727Z/report.md`. TAC availability could not be verified. |

- Fresh direct inspection covered the exact current data/rules/provider,
  cartridge, fixtures, scripts, docs, and provenance files. The prior sealed
  complete-snapshot scan supplied the unchanged-process baseline; no candidate
  survived discovery, so validation and attack-path phases were not applicable.
- The independent architecture review confirmed that the fixed cartridge list
  is intentionally narrower than generic provider ingress. Platform broker
  enforcement and constructor-side digest admission remain explicit
  out-of-scope prerequisites rather than claims made by this game repository.
- Phase 3.5 exit: complete finding ledger, sealed zero-finding security receipt,
  and independently checked trust/resource boundaries. PASS.

## Phase 4 — Validate

- Final external-game snapshot:
  - `scripts/test.sh`: PASS — rustfmt, Clippy with warnings denied, 45 Rust
    tests, rustdoc, pinned upstream checksum and clean-source checks, all 36
    provenance entries, privacy checks, seventeen signed cartridge screens,
    and trusted headless QML smoke;
  - `scripts/test-provider.sh`: PASS — the level-two Gnoll Cleric profile
    passed the fixed fifteen-case TLS, replay, timeout/fault, callback,
    reconciliation, privacy, and receipt corpus twice across provider restart
    and PostgreSQL state;
  - the final sealed security snapshot digest is
    `codex-security-snapshot/v1:sha256:45f02a27c22d45bcf8982d192879ed1bfebf5424a4f9d80fb70bd42c3b804cb5`
    with zero reportable findings.
- Platform validation:
  - fresh CodeGraph inspection traced `ProviderGame`, opaque
    `GameState.state: Value`, server provider routing, and the trusted cartridge
    boundary; no platform application, QML, SDK, schema, route, or migration
    edit is required;
  - `bin/gate.sh --diff`: `GATE GREEN [diff]` — all 24 canonical stages passed,
    including workspace Rust, database integration, QML smoke, deterministic
    client and both SDK releases, cartridge/renderer contracts, provider
    security/sidecar/authority drills, backup/restore, private-alpha admission,
    and server-module containment;
  - the gate wrote worktree-bound receipt
    `3a2915a2063b03f43369b56cfc5cb83c113c8ec1479ded60f23185f0a4091c42`.
- Acceptance audit:

  | Requirement | Result |
  |---|---|
  | REQ-001 | PASS — canonical readback and the 36-entry source trace identify dungeon default/change-level behavior, rejection selection, exact level-two rows, and HP initialization. |
  | REQ-002 | PASS — exact schema v7 and hostile state tests reject unsupported level, monster band/name/scalar, and prior-schema state before state/RNG advancement. |
  | REQ-003 | PASS — Main Street and Dungeon switch between levels one/two without draws, clear encounter state, project the selected level, and reject zero/three unchanged. |
  | REQ-004 | PASS — level two spends one fight, retains every bound-20 rejected draw, selects only 11–19, initializes 36 HP, and retains record 10 as unreachable source data. |
  | REQ-005 | PASS — attack, spell, special, potion, poison, reward, and failed-retreat/death/re-entry regressions compose on level two, including retreat bound 20. |
  | REQ-006 | PASS — fixed/generic provider actions, restart replay, signed screen conformance, trusted QML smoke, scope/security review, and the visible preview prove the bounded slice without deferred authority. |
- Skips or pre-existing failures: none in the required external or platform
  gates. Packaging, production admission, deployment, publication, higher
  dungeon levels, events, and shared realm remain intentionally out of scope.
- Phase 4 exit: every EARS requirement has executable, inspection, security,
  and cross-repository evidence; external and platform gates are green. PASS.

## Phase 5 — Complete

- OpenWiki lifecycle run `3036ea59-ac4d-420a-8cad-70b5ba19983e` returned
  `status: complete` after updating the quickstart and Game Cartridge page from
  Ticket 053/rules v6 through Ticket 054/rules v7. It retained the existing
  unresolved-Claims-debt warnings on those broad pages without reporting an
  incomplete lifecycle. After archival, reconciliation run
  `c06ebd97-808e-4380-877c-bf90aef5cf8c` also returned `status: complete` and
  rebound both pages' generated provenance to the completed notes and closed
  ticket paths, with the same pre-existing warnings.
- `scripts/show.sh dungeon` opened signed rules-v7 run
  `.preview/run.o5YzO7` through the production preview boundary. Render-plan
  readback showed `The Dungeons`, the level-two descent narrative, and inert
  `Dungeon level 1` / `Dungeon level 2` buttons bound to the exact fixed
  provider actions.
- The architecture page now records the current rules-v7 separate-game proof
  and unchanged provider/cartridge/platform authority boundary.
- Final post-archive `bin/gate.sh --fast`: `GATE GREEN [fast]`, including
  pipeline structure, hooks, secrets, workspace tests/docs, cartridge,
  renderer, both SDK/developer-kit paths, and server-module containment.
- AAR-054 records the conformance-driver correction, lint refactor, exact
  level-band/RNG proof, zero-finding security inspection, and validation
  receipts. No new knowledge ID was needed; three existing prevention rules
  proved effective.
- Ticket 054 is closed and the sole active spec/notes pair is archived under
  `docs/planning/pipeline/completed/`. Delivery remains absent: no commit,
  push, packaging, admission, deployment, or publication was requested.
- Phase 5 exit: requirements, visible behavior, durable docs, AAR, knowledge
  register, and archive agree; the source-faithful rules-v7 level-two dungeon
  slice is complete. PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first live v7 conformance run returned 422 after combat. | Level-two selection consumed additional rejected draws; the later failed retreat killed the low-HP Cleric, but the inherited profile assumed every retreat returned to Dungeon and next issued `main_street`. | Made the deterministic death explicit, changed the next profile command to `reenter`, and added a provider regression for the whole level-two cast/retreat/death/re-entry sequence. | Reuse `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`: assert the exact phase/outcome when earlier RNG work changes later command eligibility. |
| 2 | The first complete external gate failed Clippy because `reduce` reached 106 lines. | Adding the bounded level-change branch crossed the repository's 100-line function policy even though focused tests passed. | Extracted the accepted draw-free mutation into `enter_dungeon`, reran all external proofs, and resealed security against the final snapshot. | Run the complete lint gate before sealing final security evidence; keep command dispatch concise by isolating phase-specific mutations behind already completed preflight. |
