---
title: Usurper solo equipment economy — notes
pipeline_id: a7b78a98-56f4-4cde-aa66-24d13035405a
---

# Usurper solo equipment economy — running notes

Chronological evidence and decisions. If a check did not run, these notes must
not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 048 is complete with a green matching diff-gate receipt and no
    active pipeline remained before Ticket 049 was opened;
  - the authenticated port map defines Milestone 2 as solo progression and
    equipment economy before the separately gated shared town;
  - `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` permits
    the current starter only while state and transactions remain player-local;
  - Ticket 048 proved the packaged public SDK/starter, deterministic reducer,
    provider PostgreSQL/restart path, signed cartridge, and trusted QML flow;
  - the source-fidelity lessons require canonical branch/data declaration
    evidence rather than proximity-based translations.
- Scope decision:
  - implement the closed solo reward → buy/sell/haggle → equip → combat loop
    plus private bank/chest conservation;
  - defer poison, spells, specials, quests/finale and every shared bank/town
    effect to later tickets.

## Phase 2 — Design

- Canonical branch declaration:
  - this slice uses v0.20e's non-classic object path, not the mutually
    exclusive classic direct-equipment path;
  - weapon rows are reviewed shop-enabled, side-effect-free records 2 through
    9 from `EDITOR/EDWEAP.PAS` (`Dagger` through `Short Bow`); body-armor rows
    are sparse canonical records 1, 3, 4, 7, and 14 from `EDITOR/EDBODY.PAS`
    (`Grass Coat`, `Cloth`, `Leather Vest`, `Plate Mail`, and `Banded Mail`);
  - the gaps are intentional: they exclude records with extra HP/stat/cure or
    later-milestone behavior while retaining their original type-local IDs;
  - `INIT.PAS` fixes the pack at 15 slots. Home chest capacity is operator
    configuration in v0.20e, so this development profile explicitly fixes it
    at 15 rather than claiming a universal historical default.
- Architecture and data flow:
  1. `usurper-model` adds only bounded serializable value types: item kind and
     canonical catalog ID, fifteen pack slots, fifteen private chest slots,
     one right-hand weapon, one body-armor slot, solo bank gold, shop-specific
     remaining haggles, locations/phases, and strict commands. Equipped items
     are removed from the pack; swapping puts the prior item into the selected
     pack slot, matching `INVENT.PAS::Use_Item`.
  2. `usurper-data` owns the reviewed catalogs with canonical price, combat
     power, shop flag, and source reference. Lookup never trusts a command to
     supply price or power. Sparse ordinals are stable and unknown/unreviewed
     IDs reject.
  3. `usurper-rules` remains a pure launch/reduce/view boundary. The reducer
     validates phase, slot, catalog, capacity, ownership, funds, and checked
     scalar arithmetic before any successful item or money transfer. A failed
     legacy haggle is an accepted in-game outcome: it consumes one shop attempt
     and advances the provider revision while transferring no gold or item.
  4. Haggling follows `HAGGLEC.PAS` without RNG: creation begins with three
     attempts per shop; daily maintenance resets weapon to three and armor to
     four, with one additional attempt for Jesters. An offer must be positive,
     below list price, and at least the rounded 80% floor. Its rounded discount
     must fit the original charisma band (4/7/10/13/17/20 percent). The command
     itself is the player's final acceptance when that offer succeeds.
  5. Buying uses list price or the accepted offer, requires a free pack slot,
     and conserves gold. Selling is limited to a matching unequipped pack item
     in the current shop and pays `value div 2`. Equipped items must be
     unequipped first, as the non-classic shop does.
  6. Solo banking preserves the `BANK.PAS` hand/bank transfer and two-billion
     scalar ceiling. Chest transfers preserve ownership between the fixed pack
     and player-private development chest. Robbery, guards, interest, public
     records/news, and other shared effects are absent.
  7. Player attack begins with equipped weapon power, then consumes exactly the
     established strength draw and conditional strength bonus. Normal defence
     subtracts the v0.20e configured development base of 25 percent of equipped
     body-armor power, rounded by one isolated positive-rational helper. The
     existing monster response, reward, death, sleep, and RNG order otherwise
     remain unchanged.
  8. `usurper-provider` still implements only the public `ProviderGame` seam.
     Generic JSON commands expose the full bounded reducer contract; fixed
     Cartridge actions cover representative buys, haggles, slot operations,
     and transfers without adding trusted text/numeric input.
  9. The signed inert cartridge adds inventory, weapon shop, armor shop, bank,
     and chest screens to the existing eleven-screen package. It contains only
     schemas and presentation data; the platform-owned QML renderer remains the
     sole executable presentation surface.
- API and compatibility:
  - provider protocol v1, Cartridge format v1, presentation protocol v1, and
    `ProviderGame` are unchanged;
  - the development game `rules_version` advances from 1 to 2 because the
    strict serialized state gains required fields. This unadmitted build has
    no production save migration or compatibility promise;
  - commands use one-based legacy slot numbers and type-local catalog IDs;
    out-of-range slots, mismatched shops, unknown items, extra JSON fields, and
    forbidden phases reject without state or RNG movement;
  - fixed QML actions are conveniences over the same commands and do not widen
    the generic provider contract.
- Database and migration consequences:
  - no OmarchyGS migration, game table, route, or platform gameplay copy;
  - no new provider migration: the starter continues to durably store the
    opaque bounded v2 JSON state and revision in its independent PostgreSQL
    database;
  - no shared Usurper realm schema is introduced. That remains blocked on the
    reviewed pre-Milestone-3 state-topology seam.
- Exact file manifest — adjacent Usurper repository:
  - `crates/usurper-model/src/lib.rs` — equipment/economy state, phases,
    commands, limits, and error surface;
  - `crates/usurper-data/src/lib.rs` — reviewed sparse weapon/body catalog and
    lookups with canonical references;
  - `crates/usurper-rules/src/lib.rs` — pure pack/equipment/shop/haggle/bank/
    chest transitions, equipment-aware combat, daily counters, and views;
  - crate-local rule/data/provider tests — tables, boundaries, conservation,
    determinism, no-RNG haggling, combat differential, adapter strictness;
  - `crates/usurper-provider/src/lib.rs` — fixed cartridge action decoding and
    v2 adapter tests;
  - `cartridge/{manifest.json,presentation.json}` and five new presentation
    fixtures — inert sixteen-screen/action expansion;
  - `scripts/{test-cartridge.sh,test-provider.sh,show.sh}` — all-screen proof,
    representative economy conformance sequence, restart, and visible preview;
  - `README.md`, `docs/COMPATIBILITY.md`, and
    `provenance/source-trace.json` — milestone description, explicit limits,
    source-to-Rust/test trace;
  - current Ticket 049 spec/notes/AAR/index and later OpenWiki pages — workflow
    evidence only; no platform application source is expected to change.
- CodeGraph design evidence:
  - pipeline-bound exploration traced `ConformanceGameplayProfile` through the
    public runner and brokered provider session path. The new deterministic
    command sequence fits its bounded payload/count/status contract without a
    platform conformance change;
  - a second exploration traced `ProviderGame::{launch,command,view,event}` and
    cartridge session presentation. The opaque state remains provider-owned,
    while the authenticated bounded view and signed render plan retain their
    existing platform authority;
  - the adjacent game repository has no CodeGraph index. Direct review covers
    its Rust, JSON, shell, and canonical Pascal producers/consumers.
- Risks and controls:
  - source-mode drift: every row and rule names the non-classic producer, and
    tests retain sparse IDs; classic Troll discount/direct equipment is not
    silently mixed in;
  - duplication/loss: every mutation uses one ownership move between pack,
    equipped slot, and chest plus checked gold conservation assertions;
  - malformed durable state: validation rejects unknown IDs, wrong equipped
    kinds, excess balances/counters, impossible phase state, and oversized
    presentation before processing commands. Repeated catalog IDs remain
    valid because v0.20e permits multiple ordinary copies and has no per-item
    instance identity;
  - arithmetic: all money uses checked `i32` bounded at two billion; rounding
    is isolated and covered by non-tie source fixtures because no DOS oracle is
    available for a stronger floating-point parity claim;
  - replay/concurrency: the pure reducer is supplemented by the existing
    provider expected-revision, operation-receipt, restart, and replay corpus;
  - privacy/trust: no platform identity enters rules or views; cartridge data
    gains no code, URL, credential, or direct provider access;
  - rollback: the game is still separate and unadmitted; rules v1 artifacts
    remain distinguishable and no platform/database migration needs reversal.
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | exact sparse catalog tests/hash/readback; source-trace paths and ordinal fixtures; GPL/SPDX and content scans |
  | REQ-002 | one-based slot, full pack/chest, equip/swap/unequip/store/retrieve, unknown item, malformed/duplicate state, deterministic snapshot tests |
  | REQ-003 | direct/accepted/rejected/exhausted haggle, charisma bands, half-price sale, wrong shop, funds/full pack, no-RNG and gold/item conservation tests |
  | REQ-004 | zero/negative/maximum/overflow bank transfers, full/empty chest, total-gold and item-multiset property loops, provider replay/revision proof |
  | REQ-005 | same-seed equipped/unequipped attack and armor absorption fixtures, exact trace bounds/order, prior complete-day/retreat/death/reward suite |
  | REQ-006 | deterministic pack/conform, sixteen render plans, trusted QML ready/failure/keyboard smoke, visible equipment-economy preview |
  | REQ-007 | cross-repository dependency/migration/route/content scans, direct external inspection plus platform CodeGraph, live conformance twice across restart, external checks and full platform gate |
- Material alternatives rejected:
  - mixing the classic weapon/armor files with non-classic pack/chest behavior
    was rejected because those are mutually exclusive source modes;
  - porting every object/equipment slot was deferred because stat modifiers,
    cures, curses, restrictions, alignment, and special effects belong to later
    bounded slices;
  - a new platform route/schema or publisher QML was rejected because the
    existing provider and signed-data seams already carry this player-local
    behavior safely.

## Phase 3 — Implement

- Built in the separate `omarchygs_usurper` workspace:
  - advanced the unadmitted development rules/cartridge identity to v2 and
    added strict bounded item/equipment/economy state to `usurper-model`;
  - transcribed non-classic weapon records 2-9 and body records 1, 3, 4, 7,
    and 14 with canonical prices/powers and sparse ordinal lookup;
  - added pack/equipped/chest ownership moves, safe swaps, half-value sales,
    list/offer purchases, no-RNG charisma haggling, solo bank conservation,
    daily attempt resets, and equipment-aware normal combat to the pure reducer;
  - added fixed Cartridge action mappings on top of the full strict JSON
    command contract without changing the public Provider trait;
  - expanded the inert cartridge from eleven to sixteen screens with inventory,
    weapon shop, armor shop, bank, and chest views/actions plus updated fixtures;
  - expanded the live provider profile to exercise buy/haggle/equip/unequip,
    chest store/retrieve, bank deposit/withdraw, equipped dungeon entry, the
    prior day loop, callback, and persistent active status;
  - updated the compatibility ledger, port map, README, and machine-readable
    source trace with five equipment/economy source-to-test entries.
- Implementation correction:
  - the first live conformance attempt rejected its generated configuration as
    non-canonical because two new command objects wrote `kind` before
    `catalog_index`. Reordered those JSON fields lexicographically; no protocol
    or runner change was needed.
- Focused evidence already run while implementing:
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` and
    `cargo test --workspace --all-features` — PASS (2 data, 3 provider, 12
    rules, and 1 complete-day integration test; doc tests PASS);
  - `scripts/test-cartridge.sh` — PASS for all sixteen signed render plans and
    trusted QML ready/fixed-state smoke;
  - `scripts/test-provider.sh` — PASS after the canonical-order correction,
    with the fixed fifteen-case TLS/PostgreSQL/replay/fault/callback corpus run
    twice across provider restart;
  - `scripts/test.sh` — PASS including fmt, Clippy, tests, rustdoc, authenticated
    upstream hashes/tree, eighteen source-trace entries, privacy scan, signed
    cartridge, and trusted QML smoke.
- No platform application source, migration, route, schema, or gameplay copy
  was added for Ticket 049.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Canonical provider input | Two new `buy_item` objects in the generated conformance configuration placed `kind` before `catalog_index`, so the public CLI correctly rejected the non-canonical compact JSON. | medium | Reordered both objects lexicographically, recorded the correction, and reran the full live corpus twice across restart successfully. |
| 2 | Source branch fidelity | Classic direct-equipment tables and non-classic inventory/object tables are mutually exclusive; combining them would create a plausible but nonexistent legacy mode. | high | PASS after selecting only non-classic `EDWEAP`/`EDBODY`, `INVENT`, and non-classic shop branches. Sparse type-local ordinals and exclusions are explicit in code/docs/tests. |
| 3 | Ownership and arithmetic | Every accepted buy/sell/equip/swap/unequip/store/retrieve/deposit/withdraw path was checked for a single source and destination, checked money bounds, and rejection-before-transfer behavior. Repeated catalog IDs are valid ordinary copies, not unique object identities. | informational | PASS through table-driven transitions, full pack/chest, empty slot, wrong shop, insufficient source, zero, maximum, overflow, and conservation fixtures. |
| 4 | Determinism and combat | Haggling consumes no RNG, equipment adds no draw, and normal combat must retain the prior tape even while weapon damage and armor absorption change results. | informational | PASS: all six charisma bands, exhausted attempts, empty trace/state index, same-tape equipped/unequipped differential, and prior day/retreat/death/reward tests pass. |
| 5 | Provider/state compatibility | Required v2 fields cannot decode an old v1 state. | informational | Accepted for this explicitly unadmitted development release; rules/cartridge versions both advance to 2 and there is no production save or migration claim. |
| 6 | Authority, privacy, and dependencies | Model/data/rules still contain only Serde/error dependencies and no SQL, network, filesystem, clock, entropy, platform identity, or credentials; only the adapter consumes packaged public Provider crates. | informational | PASS via Cargo tree, forbidden-shape scan, independent database/restart run, and no platform application diff for Ticket 049. |
| 7 | Trusted presentation | The five new views could have widened execution or navigation authority. | informational | PASS: cartridge inventory is JSON/schema only, no executable files/assets/URLs/provider code exist, all 68 actions are unique/declared, all 16 screens conform, maximum screen size is 24 nodes, and trusted QML smoke passes. |
| 8 | Fresh platform blast radius | Post-implementation CodeGraph traced the profile through broker/provider state and the authenticated view/action through session presentation and trusted rendering. | informational | PASS: the existing opaque `ProviderGame` state, bounded public view, signed screen/action, and immutable release pins already support rules v2 and five additional inert screens; no platform caller/schema/migration needs a change. Direct inspection covered the unindexed adjacent repository. |

- Visible inspection evidence:
  - `scripts/show.sh weapon-shop` packed and conformed the signed v2 cartridge,
    prepared the trusted render plan, and opened the platform-owned QML window;
    the process remained active after the ready-state screen appeared. The
    software stack emitted only the known non-fatal DRI2 EGL warnings.

## Phase 4 — Validate

- Adjacent Usurper workspace:
  - `scripts/test.sh && scripts/test-provider.sh` — PASS after the final test
    additions;
  - Rust formatting, Clippy with warnings denied, all workspace tests and doc
    tests, rustdoc, authenticated upstream hashes and clean source tree,
    eighteen-entry source trace, privacy/content checks, deterministic signed
    cartridge, all sixteen render plans, and trusted QML smoke all passed;
  - the fixed fifteen-case TLS/PostgreSQL/replay/fault/callback conformance
    corpus passed twice across provider restart with the rules-v2 gameplay
    profile.
- Visible proof:
  - `scripts/show.sh weapon-shop` opened the signed rules-v2 weapon-shop screen
    in the platform-owned QML preview and remained active; the only diagnostics
    were known non-fatal DRI2 EGL warnings.
- Platform repository:
  - `bin/gate.sh --diff` — `GATE GREEN [diff]` across all 24 stages, including
    deterministic native packaging, PostgreSQL/API/QML smoke, provider
    security and authority proofs, backup/restore, private-alpha admission,
    and server-module isolation/conformance;
  - the gate wrote worktree receipt
    `a2b9494249a9a1481b2e3d6cccd6743ed17d65725b8ece47036275cb2b693576`.
    Phase 5 documentation changes require one final matching diff gate before
    the archived milestone can be considered delivery-ready.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 — PASS: the authenticated v0.20e non-classic branch, reviewed
    sparse weapon/body records, stable ordinals, eighteen-entry source trace,
    upstream hashes, clean source tree, rights declarations, and content scans
    all passed;
  - REQ-002 — PASS: table-driven rules cover one-based slots, full/empty pack
    and chest, equip/swap/unequip, ownership movement, unknown IDs, malformed
    state, deterministic snapshots, and revision-safe provider replay;
  - REQ-003 — PASS: list and accepted-offer buys, failed and exhausted
    haggling, all six charisma bands, half-price sales, wrong shop, funds and
    capacity denial, no-RNG behavior, and money/item conservation all passed;
  - REQ-004 — PASS: zero, maximum, negative and overflow bank cases plus
    private chest capacity and multiset/value conservation passed, while scans
    confirmed that robbery and shared-bank state remain absent;
  - REQ-005 — PASS: equipped/unequipped differential fixtures preserve the
    exact RNG tape and prior reward, death, retreat, sleep, and daily semantics;
  - REQ-006 — PASS: the deterministic signed cartridge conforms, all sixteen
    screens and 68 declared actions validate, trusted QML ready/failure smoke
    passes, and the visible weapon-shop preview opened successfully;
  - REQ-007 — PASS: dependency/content inspection, fresh platform CodeGraph,
    external checks, provider restart conformance, and the complete platform
    gate found no Usurper gameplay copy, route, migration, shared-realm claim,
    executable cartridge content, or production publication.
- Docs:
  - OpenWiki update run `c0b94930-9801-4aa9-a0fc-58ea50e06661` updated
    `quickstart.md` and `game-cartridges.md` and completed with their known
    pre-existing claims-debt warnings;
  - completion reconciliation run `597a0d26-fe43-4121-b853-cac294030109`
    returned `status: complete` without warnings and wrote the matching
    pipeline completion receipt.
- AAR:
  - submitted `AAR-049` with one concrete failure, two prevention rules, and
    the governing existing Usurper architecture decision;
  - every new `BF-` and `PR-` ID was appended to the knowledge register.
- Archive:
  - Ticket 049 closed and the spec/notes pair moved to `pipeline/completed/`;
    no active pipeline pair remains.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first rules-v2 live provider run rejected two otherwise valid `buy_item` fixtures. | Their nested JSON object fields were emitted in a non-canonical order. | Reordered the fields lexicographically and reran the complete corpus twice across restart. | `PR-omarchy-gaming-system-verify-generated-provider-profiles-before-live-use-001` |
| 2 | The legacy source exposes classic direct-equipment and non-classic object/inventory implementations with overlapping names. | Version proximity alone does not establish that mutually exclusive build branches can be composed. | Declared the non-classic mode first and admitted only producers and consumers from that branch. | `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` |
