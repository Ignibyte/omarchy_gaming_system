---
title: Usurper healing-potion shop and combat-turn parity — notes
pipeline_id: a47cb42d-35b4-4137-914b-aae921ac99cc
---

# Usurper healing-potion shop and combat-turn parity — running notes

Chronological evidence and decisions. If a check did not run, these notes must
not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 049 is complete with a green matching diff-gate receipt and no
    active pipeline remained before Ticket 050 was opened;
  - `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` keeps this
    player-private state inside the existing separate provider;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    and `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    require an explicit source/config mode before translating adjacent units;
  - `PR-omarchy-gaming-system-verify-generated-provider-profiles-before-live-use-001`
    requires canonical-profile round-trip proof before live conformance;
  - Ticket 049 proved the rules-v2 model/economy, public provider adapter,
    independent PostgreSQL/restart path, sixteen-screen signed cartridge, and
    trusted platform-owned QML flow.
- Canonical observations:
  - `USERHUNC.PAS` initializes a new player with 150 healing potions;
  - `INIT.PAS` sets `MaxHeals` to 75 and `QuaffOpt` to 1 in the reviewed
    development configuration;
  - `MAGIC.PAS` prices a potion at `player.level * 5`, rejects purchases above
    the configured cap, debits gold, and increments the potion count;
  - `VARIOUS.PAS::Quick_Healing` uses the ceiling of missing hit points divided
    by five, capped by available potions, then heals five points per potion up
    to maximum health;
  - `PLVSMON.PAS` routes Q/H through quick healing and, for `QuaffOpt = 1`,
    continues directly into the existing player attack phase;
  - the current Rust reducer already matches the no-RNG healing calculation,
    but combat `QuickHeal` stops after healing and therefore misses the
    configured same-turn attack.
- Scope decision:
  - implement the Magic Shop healing-potion purchase and correct combat
    heal-then-attack transition as one bounded slice;
  - retain dungeon quick healing as heal-only;
  - defer spells and Magic Shop items, and defer the apparently disconnected
    Alchemist poison unit until reachability and combat semantics form a
    coherent source-mode decision.

## Phase 2 — Design

- Canonical mode and branch declaration:
  - the source baseline remains the parentless v0.20e commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, in non-classic mode with the
    reviewed development defaults from `INIT.PAS`;
  - `Config.MaxHeals = 75` governs only Magic Shop purchases. The independent
    `USERHUNC.PAS` character-creation assignment of 150 remains the launch
    balance and the durable-state upper bound; the port will not rewrite that
    original inconsistency into a cleaner invented rule;
  - `Config.QuaffOpt = 1` is fixed for this release. A Q/H selection at full
    health is returned to the combat choice loop, but a wounded player reaches
    quick healing and then the normal attack phase even when no potion is
    available. The latter oddity is retained in a distinguishing regression;
  - the Magic Shop's healing branch is reachable and self-contained. Spells,
    magic objects, and the separately defined but unproven Alchemist poison
    flow are not composed into it.
- Architecture and data flow:
  1. `usurper-model` adds `MagicShop` location/phase variants and two strict
     commands: visit the shop and buy a positive `u16` potion quantity. The
     existing healing-potion field remains the sole durable inventory scalar;
     no new identity, shared state, or platform-owned snapshot is introduced.
  2. `usurper-rules` declares the source-linked shop maximum (75), healing
     unit (5 HP), and configured quaff option (1). State validation continues
     to admit the canonical 150 launch value, while purchase validation checks
     the post-purchase value against 75 before mutation.
  3. Entering the Magic Shop is a no-RNG Main Street transition. Buying derives
     unit price from the authoritative character level (`level * 5`), validates
     positive quantity, cap and funds, computes cost with checked arithmetic,
     and atomically debits gold/increments potions. Invalid quantities, cap,
     or funds reject with the original state and RNG untouched.
  4. Quick healing remains the existing ceiling calculation: potions used are
     `min(ceil((max_hp - hp) / 5), available)`, health is capped at maximum,
     and the operation consumes no draw. In Dungeon phase that ends the
     transition. In Combat, a player who was wounded before healing continues
     directly through the existing `attack` function in the same reducer call;
     its first random draw therefore occurs at exactly the old attack position.
  5. Combat healing preserves both pieces of observable output by prefixing
     the healing result to the attack result. If the player was already at full
     health, it accepts the legacy no-effect menu outcome without attacking or
     moving RNG. If wounded with zero potions, it reports that no potion was
     available and still attacks, matching the selected source branch.
  6. The game view adds potion count to the bounded status, exposes Magic Shop
     price/count/cap facts, and uses the currently unused Main Street primary
     label for Magic Shop navigation rather than widening the v1 view schema.
  7. `usurper-provider` continues to implement only the public `ProviderGame`
     seam. Generic strict JSON accepts a bounded quantity; fixed cartridge
     actions expose visit and buy-one commands. The generated gameplay profile
     visits the shop but does not fake a purchase while the canonical launch
     balance remains above the shop cap; constructed adapter/rules tests prove
     purchase below the cap.
  8. The signed cartridge adds one inert `magic-shop` screen and host
     navigation/action declarations. It adds no QML, JavaScript, asset, URL,
     input parser, network access, credential, or publisher execution.
- API and compatibility:
  - Provider protocol v1, public `ProviderGame`, Cartridge format v1,
    presentation protocol v1, and the view schema remain unchanged;
  - the unadmitted development `rules_version` and `cartridge_version` advance
    from 2 to 3 because command enums, phase enums, and combat command behavior
    change. There is no production save migration or backward-compatibility
    promise for this local release;
  - command JSON remains deny-unknown-fields. `quantity` is an unsigned bounded
    scalar, with zero, over-cap, unaffordable, wrong-phase, and unknown-action
    inputs rejected before mutation;
  - provider revision/idempotency remains owned by the unchanged starter. An
    accepted no-effect full-health combat command still advances one provider
    revision, while an exact operation replay returns the stored response.
- Database and migration consequences:
  - no OmarchyGS schema, migration, route, compiled rule, catalog admission,
    or writable gameplay copy changes;
  - no provider migration is needed. The independent starter PostgreSQL store
    continues to persist opaque bounded v3 JSON state and operation receipts;
  - no shared Usurper realm table or transaction seam is introduced.
- Exact file manifest — adjacent Usurper repository:
  - `crates/usurper-model/src/lib.rs` — Magic Shop phase/location and bounded
    visit/purchase commands;
  - `crates/usurper-rules/src/lib.rs` — v3 constants, purchase reducer, view,
    heal-then-attack composition, and source-distinguishing unit tests;
  - `crates/usurper-provider/src/lib.rs` — fixed action decoding and adapter
    coverage while preserving the full generic command path;
  - `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `fixtures/presentation/{main-street,combat,magic-shop}.json` — v3 signed
    inert screen/action and representative view facts;
  - `scripts/{test.sh,test-cartridge.sh,test-provider.sh,show.sh}` — source-trace
    floor, seventeen-screen conformance/QML smoke, v3 live profile, and visible
    preview selection;
  - `README.md`, `docs/{COMPATIBILITY,RUST_PORT_MAP}.md`, and
    `provenance/source-trace.json` — milestone limits and four new trace links
    for launch/cap, purchase, healing quantity, and combat continuation;
  - Ticket 050 spec/notes/AAR/index and later generated OpenWiki reconciliation
    in the platform repository — workflow evidence only.
- CodeGraph design evidence:
  - the pipeline-bound exploration confirmed that `ProviderGame::command` and
    `view` carry game-neutral bounded JSON and opaque provider-owned state; the
    platform does not enumerate Usurper commands or phases;
  - `ProviderGameManifest` pins rules and cartridge identity, while signed
    screen actions compile into bounded `RenderPlan` nodes rendered by trusted
    host QML. One additional conforming screen/action set does not require a
    platform trait, route, schema, or QML source change;
  - the adjacent game repository has no CodeGraph index. Direct review covered
    its Rust producers/consumers, JSON, shell harness, and exact Pascal branches.
- Risks and controls:
  - source oddity normalization: retain separate launch and shop bounds and
    prove both; purchase cannot increase a player already above 75;
  - turn-order drift: one reducer operation calls healing before the unchanged
    attack function, with tape-index and outcome comparisons against direct
    attack fixtures;
  - accidental free combat turn: distinguish full-health menu rejection from
    wounded/no-potion autoattack and wounded/available-potion autoattack;
  - partial economy mutation: validate quantity, cap, unit price, total cost,
    and funds before either gold or potion count changes; assert conservation;
  - replay/concurrency: retain starter expected revision, operation receipt,
    process restart, and exact replay corpus with deterministic state;
  - privacy/trust: state and view gain no platform identity or credential
    shape, and the cartridge remains inert host-rendered data;
  - rollback: v2 and v3 identities remain distinct; this game is unadmitted
    and no platform/provider migration requires reversal.
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | exact constants and launch fixture; Pascal symbol/path readback; four new source-trace entries and GPL/content checks |
  | REQ-002 | level-price, successful 73→75 purchase, zero/over-cap/above-cap/insufficient-funds/wrong-phase tests, no-RNG and gold/potion conservation assertions |
  | REQ-003 | wounded full-supply, partial-supply, zero-supply, lethal-player-hit, lethal-monster-hit, direct-attack tape parity, and deterministic twin tests |
  | REQ-004 | dungeon heal-only, full-health combat no-effect, malformed JSON/unknown field, rejected-state equality, expected-revision/replay proof, and prior complete-day regression |
  | REQ-005 | deterministic v3 pack/conform, seventeen render plans, trusted QML ready/fixed-state smoke, provider adapter plus live corpus twice across restart, and visible Magic Shop preview |
  | REQ-006 | dependency/privacy/content/migration/route scans, direct external inspection plus platform CodeGraph, external full checks, OpenWiki reconciliation, and platform diff gate |
- Material alternatives rejected:
  - lowering the launch inventory to 75 or raising the shop cap to 150 was
    rejected because either would erase a source-proven compatibility oddity;
  - treating combat quick heal as a standalone turn was rejected because it is
    the current fidelity defect under configured option 1;
  - adding spells, poison, or a generalized consumable-item framework was
    deferred because those require separate catalogs, reachability, targeting,
    and combat-mode evidence;
  - adding platform application code was rejected because the existing opaque
    provider and signed-data contracts already carry the complete slice.

## Phase 3 — Implement

- Built in the separate `omarchygs_usurper` workspace:
  - advanced the unadmitted rules/cartridge identity to v3 and added bounded
    Magic Shop location/phase plus strict visit/purchase commands;
  - retained the 150-potion character-creation balance and separately enforced
    the configured 75-potion shop purchase ceiling;
  - added checked level-times-five unit pricing, quantity/funds/cap validation,
    atomic gold/potion conservation, and no-RNG shop transitions;
  - composed combat quick healing with the existing normal attack function for
    configured quaff option 1, while preserving full-health no-effect and the
    wounded/no-potion autoattack oddity;
  - added potion facts to the bounded view, one signed inert Magic Shop screen,
    Main Street navigation, fixed buy-one action, and explicit combat label;
  - expanded the public-provider profile to visit the shop, fixed adapter tests
    to prove both generic-quantity and signed fixed actions below the cap, and
    kept the unchanged fifteen-case transport/replay/fault corpus;
  - added four source-trace records and updated the compatibility ledger, port
    map, README, and seventeen-screen test/preview scripts.
- Implementation correction:
  - the first workspace test run exposed the old `one_day` integration driver
    issuing `Attack` unconditionally after `QuickHeal`. Under the corrected
    v3 semantics, the quick-heal transition had already attacked and sometimes
    ended combat, so the extra command rejected as invalid. Changed both the
    unit-day loop and integration driver to choose exactly one combat command
    per iteration; no production rule was weakened.
- Focused evidence:
  - `cargo test --workspace --all-features` — PASS after the driver correction
    (2 data, 4 provider, 15 rules, and 1 full-day integration test; doc tests
    PASS);
  - `scripts/test-cartridge.sh` — PASS for all seventeen signed render plans
    and trusted QML ready/fixed-state smoke;
  - `scripts/test.sh` — PASS including fmt, warnings-denied Clippy, all tests,
    rustdoc, authenticated upstream hashes/tree, 22 source-trace entries,
    privacy scan, signed cartridge, and trusted QML smoke;
  - the cartridge contains 17 unique screens and 71 declared actions.
- No platform application source, migration, route, schema, provider trait, or
  trusted QML source changed for Ticket 050.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Source/config fidelity | v0.20e initializes 150 potions but configures a 75-potion shop ceiling; merging those into one limit would erase an observable legacy oddity. | high | PASS: durable state admits 150, creation retains 150, purchase alone enforces 75, and tests distinguish above-cap, 73→75, and rejected 75→76 paths. |
| 2 | Combat turn order | The source's Q-selection guard rejects full-health healing, but a wounded player with zero potions still reaches `QuaffOpt = 1` and normal attack. | high | PASS: full health moves no RNG/monster/player state; wounded full/partial/zero supply uses the exact direct-attack tape; both player-death and monster-victory terminal branches pass. |
| 3 | Economy/input integrity | Quantity, cap, unit-price, total-cost, and funds checks could partially debit or overflow if ordered after mutation. | medium | PASS: all validation and checked arithmetic precede one atomic gold/potion update; zero, cap, insufficient-funds, wrong-phase, extra-field, generic, and fixed-action tests pass with rejected-state equality. |
| 4 | Determinism/replay | Composing heal and attack must not add a random draw or create a second provider operation. | informational | PASS: quick healing consumes no draw, calls the unchanged attack inside one reducer transition, matches direct attack's complete trace/state index, and passes the provider expected-revision/replay corpus twice across restart. |
| 5 | Version/state compatibility | Existing v2 state can deserialize structurally in some phases but command semantics and enum surface changed. | informational | Accepted only as an explicitly unadmitted v3 development release; rules and cartridge identities both advance to 3 and no production save/migration compatibility is claimed. |
| 6 | Trusted presentation | Main Street and the new shop could widen executable/frontend authority or exceed host bounds. | informational | PASS: the cartridge remains JSON/schema-only, all 17 screen IDs and 71 action IDs are unique/declared, the largest screen has 26 nodes, all render plans conform, and trusted QML ready/fixed-state smoke passes. |
| 7 | Privacy/dependencies | Potion state/view additions could introduce platform identity or backend coupling. | informational | PASS: model/data/rules retain only serialization/error dependencies and no SQL/network/filesystem/clock/entropy; the privacy/content scan finds no identity, credential, URL, script, or publisher QML shape. |
| 8 | Fresh platform blast radius | Post-implementation CodeGraph traced provider identity/state, exact release lookup, render requests, signed plan compilation, and trusted navigation consumers. | informational | PASS: the existing opaque game-neutral JSON, exact v3 release pins, bounded render plan, and host navigation already carry this slice; no platform caller, schema, route, migration, trait, or QML source needs a change. Direct inspection covered the unindexed adjacent game. |

- The sole implementation-time regression was the stale full-day test driver
  recorded in Phase 3; it was fixed and the complete external check reran green.
- No unresolved inspection finding remains.

## Phase 4 — Validate

- Adjacent Usurper workspace:
  - `scripts/test.sh` — PASS after the last game-code edit: fmt, Clippy with
    warnings denied, all Rust tests, rustdoc, canonical upstream hashes/tree,
    22-entry source trace, privacy scan, signed seventeen-screen cartridge,
    and trusted QML state smoke;
  - `scripts/test-provider.sh` — PASS after the final profile edit: fixed
    fifteen-case TLS/authentication/replay/fault/callback/reconciliation corpus
    completed twice across provider process restart with rules version 3;
  - direct JSON/shell checks confirmed 17 unique screens, 71 unique declared
    actions, no undeclared node action, valid fixtures, and valid shell syntax.
- Platform workspace:
  - `bin/gate.sh --diff` — `GATE GREEN [diff]` across all 24 stages, including
    deterministic SDK/starter releases, signed cartridge/trusted renderer,
    reproducible native package, PostgreSQL integration, live QML, remote
    provider, backup/restore, admission, and module isolation/conformance;
  - gate receipt and current gated-state hash both equal
    `38c50ac9cdbfbfb5589f6788ae89fd5263d1dfd38fa6acb9c6e5d56c7fb3c31c`.
- Phase 4 PASS. No requirement is skipped and no pre-existing red check is
  being claimed as green.

## Phase 5 — Complete

- Acceptance audit:

  | Requirement | Result | Concrete evidence |
  |---|---|---|
  | REQ-001 | satisfied | v3 constants/creation tests preserve 150 launch and 75 shop bounds; four new canonical source-trace records identify `USERHUNC`, `INIT`, `MAGIC`, `VARIOUS`, and `PLVSMON`; trace count/checks pass. |
  | REQ-002 | satisfied | price/cap/funds/quantity/wrong-phase tests prove the 73→75 purchase, 10-gold debit, conservation, checked rejection, and no RNG; generic and fixed provider commands pass. |
  | REQ-003 | satisfied | full/partial/zero-supply quaff fixtures match direct attack's complete RNG trace, consume exact five-HP units, and cover player death plus monster victory in the same transition. |
  | REQ-004 | satisfied | dungeon heal-only, full-health combat no-effect, strict extra-field rejection, state-before/after assertions, corrected full-day regression, and live provider replay/revision evidence pass. |
  | REQ-005 | satisfied | deterministic v3 package has 17 unique screens/71 actions; all-screen render-plan and trusted QML state smoke pass; live provider corpus passes twice across restart. |
  | REQ-006 | satisfied | dependency/privacy/content scans and fresh CodeGraph inspection show no spell/item/poison/shared-realm/platform gameplay/schema/route/QML change or production publication; external checks and platform diff gate pass. |
- Hand-maintained `docs/architecture/game-cartridges.md` now records Tickets
  047–050 as the separate rules-v3 Usurper proof and its explicit non-admission
  boundary.
- OpenWiki update lifecycle completed under run
  `2a51c15e-7eba-4717-aeed-385c68e91fce`; Grounded Claims were added for the
  rules-v3/17-screen proof on `quickstart.md` and `game-cartridges.md`, and both
  pages were reconciled. Finalization reported the pages' pre-existing
  unresolved evidence debt as warnings but returned `status=complete` and
  issued a matching completion receipt for state hash
  `b1435770c0b6398b50767bae54bf78f60ad21a28138186d1918e0f20a7e48c24`.
- AAR 050 was submitted. New indexed lessons:
  - `BF-omarchy-gaming-system-usurper-composite-quaff-double-attack-001`;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`.
- Post-OpenWiki `bin/gate.sh --diff` — `GATE GREEN [diff]`; the gate receipt,
  OpenWiki completion receipt, and current gated-state hash all equal
  `b1435770c0b6398b50767bae54bf78f60ad21a28138186d1918e0f20a7e48c24`.
- `scripts/show.sh magic-shop` opened the signed Magic Shop fixture in the
  trusted QML renderer and remained live after its ready window appeared;
  the headless EGL fallback warnings were nonfatal.
- No requirement needs a follow-up ticket. Ticket 050 and this pipeline pair
  were archived complete on 2026-08-31; no delivery publication was performed.
