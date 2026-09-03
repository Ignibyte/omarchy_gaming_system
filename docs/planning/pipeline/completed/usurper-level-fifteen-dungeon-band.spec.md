---
title: Usurper Level Fifteen Dungeon Band
pipeline_id: ad75250b-4fc4-4cd3-80e0-5cfdf006e4f7
status: Phase 5 — Complete PASS
ticket: TICKET-069
ticket_doc: docs/planning/tickets/closed/TICKET-069-usurper-level-fifteen-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-069-usurper-level-fifteen-dungeon-band.md
created: 2026-09-02
---

# Usurper Level Fifteen Dungeon Band — spec

## Intent

Advance the separate source-linked Usurper port by one complete normal dungeon
band while preserving provider ownership, signed inert presentation, unique
controls, real-input proof, and all existing combat behavior.

## Scope

- In:
  - exact v0.20e Level 15 monster records 140–149;
  - normal Level 15 rejection-loop selection and three-times-strength HP;
  - draw-free level switching across levels one through fifteen;
  - rules/state/cartridge v20, bounded fifteenth-choice projection,
    deterministic reducers, provider projections, signed presentation,
    fixtures, provenance, documentation, and tests;
  - live provider restart conformance and provider-backed workspace-8 play.
- Out:
  - Level 16+, dungeon events, quests, finale, shared realm, or new combat
    systems;
  - platform rules/state, database, Provider SDK/protocol, or renderer changes;
  - registration, admission, packaging, deployment, commit, push, or
    publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 15 editor records 140–149 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through fifteen and exact rules schema v20, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through fifteen from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 15 encounter, the reducer shall spend one fight and repeat `Random(150)` until the result is greater than one hundred forty, select exact editor record 141–149, initialize HP to three times reviewed base strength, and preserve record 140 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When Level 15 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 15 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through fifteen and visibly enter a signed Level 15 encounter through provider-backed trusted QML, with each choice rendered and activated exactly once and without Level 16, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, unique-control and real-input checks, live profile twice across restart, signed cartridge conformance, local-play click smoke, scope/security inspection, and visible workspace-8 play. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port only the normal Level 15 band and preserve event routing as out of scope. | `DUNGEONC.PAS` keeps ordinary monster loading in the event-false branch; events remain an independent unfinished subsystem. |
| 2 | Retain editor record 140 in canonical data but keep normal selection restricted to 141–149 through the original rejection loop. | Record inventory and reachable runtime behavior are separate compatibility facts. |
| 3 | Advance rules, state schema, and cartridge identities together to v20. | Older serialized state and signed action surfaces must not be relabeled as the deeper rules release. |
| 4 | Reuse the generic provider-owned dungeon/combat reducer and add only Level 15 data, bounds, actions, projections, and tests. | OmarchyGS remains transport/rendering authority, never a second game-rules owner. |
| 5 | Add one bounded fifteenth-choice string to the external game view/schema and bind it to exactly one Level 15 action. | The existing A–N fields are occupied on the dungeon screen; widening the data-only game view is smaller and safer than overloading another semantic field or changing platform rendering. |

## Linked artifacts

- Ticket: [TICKET-069](../../tickets/open/TICKET-069-usurper-level-fifteen-dungeon-band.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: continuing goal after completing the Level 14 normal dungeon band.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | concrete shippable contract |
| 2 Design | Source/data/control-flow design and exact manifest | worktree-bound CodeGraph receipt |
| 3 Implement | External rules/data/provider/cartridge Level 15 slice | compile and self-review |
| 3.5 Inspect | Findings ledger and security review | lead disposition |
| 4 Validate | Focused/full tests, live corpus, and delivery gate | matching gate receipt |
| 5 Complete | AC audit, docs, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | explicit user authorization only |
