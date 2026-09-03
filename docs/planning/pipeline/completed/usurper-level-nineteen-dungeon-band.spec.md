---
title: Usurper Level Nineteen Dungeon Band
pipeline_id: bea038ec-1d39-483d-b23f-7f1b8b2a8625
status: Phase 5 — Complete PASS
ticket: TICKET-073
ticket_doc: docs/planning/tickets/closed/TICKET-073-usurper-level-nineteen-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-073-usurper-level-nineteen-dungeon-band.md
created: 2026-09-03
---

# Usurper Level Nineteen Dungeon Band — spec

## Intent

Advance the separate source-linked Usurper port by one complete normal dungeon
band while preserving provider ownership, signed inert presentation, unique
non-overlapping controls, real-input proof, and all existing combat behavior.

## Scope

- In:
  - exact v0.20e Level 19 monster records 180–189;
  - normal Level 19 rejection-loop selection, three-times-strength HP, and
    bounded trace risk/size evidence;
  - draw-free level switching across levels one through nineteen;
  - rules/state/cartridge v24, bounded nineteenth-choice projection,
    deterministic reducers, provider projections, signed presentation,
    fixtures, provenance, documentation, and tests;
  - game-neutral trusted-QML regression ratchet for one non-overlapping,
    exactly-once control per current dungeon choice;
  - live provider restart conformance and provider-backed workspace-8 play.
- Out:
  - Level 20+, dungeon events, quests, finale, shared realm, or new combat
    systems;
  - platform rules/state, database, Provider SDK/protocol, renderer protocol,
    renderer compiler, or production trusted-QML behavior changes;
  - registration, admission, packaging, deployment, commit, push, or
    publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 19 editor records 180–189 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through nineteen and exact rules schema v24, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through nineteen from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 19 encounter, the reducer shall spend one fight and repeat `Random(190)` until the result is greater than one hundred eighty, select exact editor record 181–189, initialize HP to three times reviewed base strength, preserve record 180 as normally unreachable, and retain a bounded trace whose quantified tail risk and maximum state size satisfy the development-provider contract. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic long-run progression, serialized-state ceiling proof, and encounter-state assertions. |
| REQ-005 | When Level 19 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 19 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through nineteen and visibly enter a signed Level 19 encounter through provider-backed trusted QML, with every current choice occupying one non-overlapping delegate row and activating exactly once, without Level 20, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, 21-to-22 non-overlapping delegate replacement, unique-control and real-input checks, live profile twice across restart, signed cartridge conformance, local-play smoke, scope/security inspection, and visible workspace-8 play. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port only the normal Level 19 band and preserve event routing as out of scope. | `DUNGEONC.PAS` keeps ordinary monster loading in the event-false branch; events remain an independent unfinished subsystem. |
| 2 | Retain editor record 180 in canonical data but keep normal selection restricted to 181–189 through the original rejection loop. | Record inventory and reachable runtime behavior are separate compatibility facts. |
| 3 | Advance rules, state schema, and cartridge identities together to v24. | Older serialized state and signed action surfaces must not be relabeled as the deeper rules release. |
| 4 | Reuse the generic provider-owned dungeon/combat reducer and add only Level 19 data, bounds, actions, projections, and tests. | OmarchyGS remains transport/rendering authority, never a second game-rules owner. |
| 5 | Add one bounded nineteenth-choice string to the external game view/schema and bind it to exactly one Level 19 action. | The existing A–R fields are occupied on the dungeon screen; widening the data-only game view is smaller than overloading another semantic field or changing platform rendering. |
| 6 | Re-audit the 256-draw bounded RNG trace against Level 19's valid-tail probability and provider state ceiling before retaining it. | The source loop is unbounded and acceptance narrows with depth; the development cap must remain evidence-driven. |
| 7 | Ratchet the trusted-QML regression from twenty-one to twenty-two current controls without adding production UI behavior. | The user's inert/duplicate reports make delegate cardinality, geometry, enablement, and exactly-once real pointer and keyboard activation release-blocking evidence for every larger dungeon surface. |

## Linked artifacts

- Ticket: [TICKET-073](../../tickets/closed/TICKET-073-usurper-level-nineteen-dungeon-band.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: continuing goal after completing the Level 18 normal dungeon band.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | concrete shippable contract |
| 2 Design | Source/data/control-flow design and exact manifest | worktree-bound CodeGraph receipt |
| 3 Implement | External rules/data/provider/cartridge Level 19 slice | compile and self-review |
| 3.5 Inspect | Findings ledger and security review | lead disposition |
| 4 Validate | Focused/full tests, live corpus, and delivery gate | matching gate receipt |
| 5 Complete | AC audit, docs, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | explicit user authorization only |
