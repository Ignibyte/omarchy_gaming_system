---
title: Usurper Gnoll Poisonous Bite — notes
pipeline_id: 29c361f7-4873-417d-8b2a-6fae36585000
---

# Usurper Gnoll Poisonous Bite — running notes

Chronological evidence and decisions. If a check did not run, these notes must
not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 052 completed rules/cartridge v5 with Backstab and Soul Strike on
    the factored player-strike/monster-response path and explicitly deferred
    the independent Gnoll poison duration branch;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires tracing the bite beyond its headline roll through tick timing,
    response suppression, and encounter completion;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps this slice on solo non-classic level-one dungeon monsters;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` applies
    because bite, tick, ordinary action, and monster response are one provider
    command;
  - `PR-omarchy-gaming-system-separate-legacy-state-from-access-gates-001`
    keeps the encounter poison flag independent from the player's race gate.
- Canonical observations:
  - `USERHUNC.PAS` advertises the Gnoll poisonous bite and allows every class
    pairing except the existing Troll/Orc Paladin exclusions;
  - `VARIOUS.PAS` resets every created monster's `poisoned` flag to false;
  - `PLVSMON.PAS` consumes `Random(4)+1` for an unpoisoned target after normal
    attack calculation and before player-strike resolution, poisoning only on
    the displayed value three;
  - each still-living poisoned monster then loses `Random(5)+1` HP after player
    and team actions but before its response; an already-poisoned monster does
    not consume another bite roll;
  - a poison-lethal monster cannot answer and ends the fight, but the reward
    helper has already been passed, so the source grants no immediate XP/gold;
  - accepted casts, configured quick healing, Backstab, and Soul Strike all
    enter this shared phase, making the passive orthogonal to class choice.
- Scope decision:
  - add one transient monster poison flag and integrate passive bite/tick stages
    across the existing solo combat commands;
  - expose poison through existing bounded narrative/status strings, not a new
    button or provider vocabulary;
  - defer weapon/spell/PvP poison, disease, teams, multiple monsters, dungeon
    events, packaging, admission, deployment, and publication.
- Preflight:
  - no active pipeline or critical bulletin existed; Ticket 053 was next;
  - pipeline tools reported CodeGraph 1.5.0 and OpenWiki 0.3.3 ready;
  - the platform PostgreSQL container was healthy;
  - direct source review covered `USERHUNC.PAS`, `VARIOUS.PAS`, and the complete
    `PLVSMON.PAS` bite/tick/response/completion branch; the external v5 model,
    rules, provider, cartridge, scripts, and Ticket 052 notes supplied the
    implementation baseline.
- Phase 1 exit: ticket, active spec/notes, AAR, scope, locked decisions, and
  six observable EARS requirements recorded. PASS.

## Phase 2 — Design

- Declared legacy mode and baseline:
  - solo, non-classic, level-one normal dungeon encounter with one player and
    one monster; no teammates, additional monsters, event branch, or shared
    realm state;
  - canonical target is Usurper v0.20e parentless commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, under
    `upstream/v0.20e/source-git/SOURCE/USURPER` in the external game repository;
  - the development RNG preserves translated bounds and draw order, not an
    unproven Borland sequence.
- Exact source trace:
  - `USERHUNC.PAS:603` labels Gnoll's racial trait `poisonous bite`;
  - `VARIOUS.PAS:578-652` clears `monster.poisoned` on reset and accepts the
    encounter's initial boolean through `Create_Monster`;
  - `PLVSMON.PAS:1186-1209` calculates the ordinary attack first, then rolls
    `Random(4)+1` only for a Gnoll facing an unpoisoned monster and poisons on
    displayed result three;
  - `PLVSMON.PAS:1214-1266` applies Backstab/Soul Strike composition and direct
    strike/death handling after the bite attempt;
  - `PLVSMON.PAS:1542-1548` rolls `Random(5)+1` once for every still-active
    poisoned monster after player/team actions and before monster processing;
  - `PLVSMON.PAS:1814-1820` ends the encounter when that later tick leaves no
    living monster. Because `Has_Monster_Died` ran earlier at line 1263, a
    poison-lethal turn does not use the immediate reward path.
- Command and RNG data flow:
  1. Attack and configured wounded Quick Heal calculate ordinary power, try
     the conditional bite, resolve the direct strike, tick poison if the
     monster remains active, then allow its response if it still lives.
  2. Backstab keeps its bound-3 success draw first, calculates ordinary power,
     tries the bite, then applies its success strike or failure response bonus;
     the poison tick still precedes either living response.
  3. Soul Strike spends HP and performs only applicable mental/addiction
     checks, calculates ordinary power and optional soul effect, tries the
     bite, resolves the direct strike, then ticks before response.
  4. A nonlethal accepted cast keeps spell resistance/effect draws, consumes
     the source's otherwise discarded ordinary-attack calculation, tries the
     bite, ticks poison, then allows response. A spell-lethal monster has
     already left the source attack phase and keeps its existing reward path.
  5. Already-poisoned monsters skip the bound-4 bite draw but continue one
     bound-5 tick per completed offensive turn. Direct lethal attacks keep
     their reward path and never tick; poison-lethal attacks consume no
     response or reward draws.
- Implementation manifest:

  | Boundary | Planned change | Compatibility |
  |---|---|---|
  | external model | add required `MonsterState.poisoned: bool` | rules/state schema v6; v5 state deliberately rejected |
  | external rules | initialize false and factor direct action, bite, tick, response, and encounter-completion stages | existing command enum and deterministic provider method unchanged |
  | external view | append bounded poison state/narration to existing strings | no new action, binding, or QML node |
  | external provider/tests | exercise Gnoll command/replay and exact draw order | launch/command/view protocols unchanged |
  | external cartridge | sign rules/cartridge v6 and show a Gnoll combat fixture | presentation protocol remains v1 |
  | platform | no source, SDK, protocol, database, migration, or QML application change | opaque provider state already carries game-owned fields |

- API and data compatibility:
  - the provider's public launch, command, view, manifest, signed callback, and
    operation-replay shapes remain byte-compatible at the protocol boundary;
  - the game-owned state intentionally advances from exact schema v5 to v6,
    and `serde(deny_unknown_fields)` plus the required boolean prevents silent
    cross-version acceptance;
  - no PostgreSQL table, platform migration, SDK type, or credential flow is
    involved. The platform continues to store and return the provider's opaque
    signed state.
- Regression matrix:

  | Case | Required evidence |
  |---|---|
  | unpoisoned Gnoll Attack | bound-4 draw after ordinary-power draws; poison only on result 2; same-turn bound-5 tick |
  | already poisoned turn | no bound-4 draw; one bound-5 tick before response |
  | non-Gnoll turn | no bite draw or poison mutation; prior combat outcomes retained |
  | Backstab success/failure | bound-3 and ordinary-power order retained; tick precedes normal/bonus response |
  | Soul Strike success/failure | HP/check/effect order retained; bite/tick compose before response |
  | caster spell | resistance/effect, discarded ordinary-power, bite/tick, response order |
  | direct lethal | existing XP/gold reward; no tick or response |
  | poison lethal | encounter completes; no XP/gold or response draw |
  | replay/restart/view | identical operation result and poison status; no fabricated race command |
  | platform rendering | trusted signed v6 fixture renders through existing combat nodes |

- Risks and alternatives:
  - rejected a separate Gnoll command because the source trait is passive and
    its apparent menu-key allowance has no matching Gnoll handler;
  - rejected player-owned or platform-owned poison state because the source
    resets it on each monster and the provider owns game semantics;
  - rejected reusing the immediate victory helper for poison lethal because it
    would invent XP/gold absent from the traced branch;
  - the discarded normal-attack computation after a living accepted cast is a
    source-order correction required before placing the Gnoll roll, even
    though the resulting punch is not applied.
- CodeGraph design evidence inspected the platform's provider starter
  `GameState { status, state: Value }`, `ProviderGame::command`, and
  `ProviderGame::view` flow plus renderer consumption. It confirmed that game
  state remains opaque JSON and existing strings/nodes accept this slice, so
  the blast radius is limited to the external Usurper repository and platform
  documentation evidence.
- Phase 2 exit: exact legacy branch, composition order, architecture boundary,
  compatibility posture, test matrix, and alternatives are actionable. PASS.

## Phase 3 — Implement

- External game workspace changes:
  - advanced exact game-state/rules identity and signed cartridge identity from
    v5 to v6;
  - added required `MonsterState.poisoned`, initialized false for every normal
    encounter and surfaced through the existing bounded status string;
  - factored the living-turn boundary so direct strike, conditional Gnoll bite,
    persistent poison tick, poison completion, and monster response occur once
    in the traced order;
  - integrated the passive with Attack, configured wounded Quick Heal,
    Backstab success/failure, Soul Strike success/failure, and all three
    accepted level-one spell paths without adding a command or cartridge node;
  - corrected living accepted-cast RNG order by calculating and discarding the
    source's ordinary attack before the bite check;
  - kept direct-lethal reward handling unchanged while poison lethal clears the
    encounter without XP/gold, news, reward draws, or a monster response.
- Provider/cartridge/provenance changes:
  - added deterministic provider replay/view coverage for a successful Gnoll
    bite and required explicit schema-v6 poison state;
  - changed the live provider profile to Gnoll Cleric and inserted an accepted
    combat cast, exercising the composite turn through the real operation path;
  - updated the signed combat fixture to visibly show the Gnoll race, bite,
    persistent `Poisoned` status, and existing spell/attack controls;
  - added three canonical source-trace entries and reconciled the README,
    compatibility ledger, port map, test floor, and test summary.
- Focused implementation checks:
  - `cargo test --workspace --no-fail-fast`: PASS after implementation (39
    total unit/integration tests: 3 data, 7 provider, 28 rules, 1 one-day);
  - focused provider replay test for Gnoll poison: PASS;
  - `scripts/test.sh`: PASS, including format, clippy `-D warnings`, all-feature
    tests/docs, canonical source/checksum checks, seventeen signed cartridge
    screens, and headless trusted QML smoke;
  - `scripts/test-provider.sh`: PASS, fixed 15-case TLS/replay/fault/callback
    corpus twice across provider restart with durable PostgreSQL receipts.
- Phase 3 exit: game-owned v6 behavior, provider replay, signed visible
  cartridge, provenance, and focused/full external checks are implemented.
  PASS.

## Phase 3.5 — Inspect

- Manual/source-fidelity review:
  - verified the shared command flow retains pre-RNG state/command validation
    and post-transition validation;
  - verified a newly poisoned monster ticks in that same turn, a persistent
    poison skips the bite reroll, direct lethal reaches exactly one reward
    helper, and poison lethal reaches neither reward nor response;
  - verified no racial action was added and existing class/spell preflight is
    still authoritative;
  - checked manifest/provider/rules identity agreement, fixture bindings,
    source paths, state-size/message limits, and absence of platform identity
    or credential shapes in game state/presentation.
- CodeGraph:
  - the external Usurper repository has no `.codegraph` index, so the tool
    explicitly declined that target and direct source/test inspection supplied
    game-side evidence;
  - a fresh worktree-bound platform inspection traced `ProviderGame`, opaque
    `GameState.state: Value`, starter apply/receipt flow, and inert `RenderPlan`
    bindings. No platform schema, SDK protocol, database migration, server, or
    QML application change is required for the provider-owned boolean/status.
- Security diff review:
  - project policy prohibited delegation; preflight returned `ready` with that
    sequential-parent warning and available goal controls;
  - TAC status could not be verified because its connector was unavailable;
  - because the external parentless repository has no resolvable HEAD, the
    terminal workflow inventoried all 19 source-like working-tree files against
    the empty-tree baseline, generated a repository-specific threat model,
    reviewed provider state/commands, combat/rewards, cartridge/view,
    standalone secrets/config, dependencies/static data/scripts, and normalized
    zero candidate rows;
  - deterministic finalization sealed a complete no-findings report at
    `/tmp/omarchygs-usurper-t053-security.SIB5yi/report.md`.
- Finding ledger: no correctness, security, source-fidelity, compatibility,
  regression, documentation, or scope findings remained open after review.
- Phase 3.5 exit: fresh structural evidence, complete sequential security
  coverage, and a zero-open finding ledger are recorded. PASS.

## Phase 4 — Validate

- Final adjacent-game snapshot:
  - `cargo test --workspace --no-fail-fast`: PASS — 39 unit/integration tests
    across data, provider, rules, and complete-day coverage;
  - `scripts/test.sh`: PASS — rustfmt, Clippy with warnings denied, Rust tests
    and docs, pinned upstream integrity, all source-trace entries, privacy
    checks, seventeen signed cartridge screens, and trusted headless QML;
  - `scripts/test-provider.sh`: PASS — the Gnoll Cleric spell/combat profile
    passed the fixed fifteen-case TLS, replay, fault, callback, reconciliation,
    and receipt corpus twice across provider restart and PostgreSQL state;
  - no external game source changed after those complete proofs; the subsequent
    security and source-fidelity inspections matched the validated rules-v6
    snapshot.
- Platform validation:
  - `bin/gate.sh --diff`: `GATE GREEN [diff]` — all 24 canonical stages passed,
    including workspace Rust, secret/shell/hook policy, deterministic
    Cartridge and both SDK releases, trusted QML renderer/package/live smoke,
    PostgreSQL integration, provider TLS/replay/sidecar/authority drills,
    backup/restore, private-alpha admission, and server-module containment;
  - the gate wrote receipt
    `0fb855df06efb495176835287431bac20f260d431fc63dc7f9bdc070f2514737`
    before Phase 5 documentation reconciliation;
  - the game-owned boolean, status string, and narrative continue through the
    existing opaque provider and inert render-plan boundaries without a server,
    SDK protocol, migration, or trusted QML application change.
- Acceptance audit:

  | Requirement | Result |
  |---|---|
  | REQ-001 | PASS — direct canonical readback and source trace identify Gnoll eligibility, poison initialization, bite, tick, response, and completion branches in the pinned clean v0.20e tree. |
  | REQ-002 | PASS — every encounter initializes explicit false poison state; required schema-v6 state and hostile JSON reject without state/RNG advancement. |
  | REQ-003 | PASS — Attack, Quick Heal, Backstab, Soul Strike, and accepted casts preserve ordinary-power/effect work before the conditional bound-4 bite draw; non-Gnoll and already-poisoned controls skip it. |
  | REQ-004 | PASS — first and later poison ticks consume the exact bound-5 draw before response; persistent status, lethal response suppression, and no-immediate-reward completion are deterministic. |
  | REQ-005 | PASS — provider replay/restart/view, live Gnoll profile, signed cartridge conformance, QML smoke, and the visible combat view use existing actions with no racial-special command. |
  | REQ-006 | PASS — security/scope review and fresh platform inspection show no broader poison/team/shared-realm feature, platform gameplay copy, route/schema change, packaging, admission, or publication. |

- Phase 4 exit: all six EARS requirements have executable, inspection, and
  cross-repository evidence; external and platform validation are green. PASS.

## Phase 5 — Complete

- OpenWiki lifecycle run `2ebd15b5-a6a5-47ba-82c4-73ddfc47bfd9` returned
  `status: complete` after updating the quickstart and Game Cartridge page from
  Tickets 048–052/rules v5 through Ticket 053/rules v6 Gnoll poison. After
  archival, reconciliation run `b511a7fb-42af-43c0-8e76-f6e81accbd1e` also
  completed and rebound generated provenance to the completed notes and closed
  ticket paths. Both finalizations retained the existing unresolved-Claims-debt
  warnings on those two broad pages but did not report an incomplete lifecycle.
- `scripts/show.sh combat` opened a newly signed rules-v6 cartridge through the
  production trusted preview boundary. Render-plan readback showed `Dungeon
  Combat`, the `Gnoll Cleric` status, `Poisonous Gnollbite!` narrative, and the
  persistent `Poisoned` marker with the existing Attack, Quick Heal, Cure
  Light, and class-special bindings.
- AAR-053 records the source-order correction, zero-finding security review,
  unchanged provider/cartridge boundary, and new standing rule
  `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001`.
- Ticket 053 is closed and this sole active spec/notes pair is archived under
  `docs/planning/pipeline/completed/`. Delivery remains absent: no commit, push,
  package, admission, deployment, or publication was requested for this slice.
- Phase 5 exit: requirements, visible behavior, durable docs, AAR, knowledge
  register, and archive agree; the source-faithful rules-v6 Gnoll poison slice
  is complete. PASS.
