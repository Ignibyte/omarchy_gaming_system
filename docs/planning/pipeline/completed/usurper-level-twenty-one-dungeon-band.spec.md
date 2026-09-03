---
title: Usurper Level Twenty-One Dungeon Band
pipeline_id: 6f6945fc-f320-4e6d-931b-15a042c659eb
status: Phase 5 — Complete PASS
ticket: TICKET-075
ticket_doc: docs/planning/tickets/closed/TICKET-075-usurper-level-twenty-one-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-075-usurper-level-twenty-one-dungeon-band.md
created: 2026-09-03
---

# Usurper Level Twenty-One Dungeon Band — spec

## Intent

Advance the separate source-linked Usurper port by one complete normal dungeon
band while preserving provider ownership, signed inert presentation, unique
non-overlapping controls, real-input proof, and all existing combat behavior.

## Scope

- In:
  - exact v0.20e Level 21 monster records 200–209;
  - normal Level 21 rejection-loop selection, three-times-strength HP, and
    bounded trace risk/size evidence;
  - draw-free level switching across levels one through twenty-one;
  - rules/state/cartridge v26, bounded twenty-first-choice projection,
    deterministic reducers, provider projections, signed presentation,
    fixtures, provenance, documentation, and tests;
  - game-neutral trusted-QML regression ratchet for one non-overlapping,
    exactly-once control per current dungeon choice, plus the provider-backed
    multi-screen action/plan-replacement smoke path;
  - production trusted-QML control materialization and button semantics needed
    to retire replaced accessibility objects before exposing the next plan,
    publish one native press action per button, and retain pointer/keyboard
    activation exactly once;
  - live provider restart conformance and provider-backed workspace-8 play.
- Out:
  - Level 22+, dungeon events, quests, finale, shared realm, or new combat
    systems;
  - platform rules/state, database, Provider SDK/protocol, renderer protocol,
    renderer compiler, or unrelated production trusted-QML behavior changes;
  - registration, admission, packaging, deployment, commit, push, or
    publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 21 editor records 200–209 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through twenty-one and exact rules schema v26, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through twenty-one from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 21 encounter, the reducer shall spend one fight and repeat `Random(210)` until the result is greater than two hundred, select exact editor record 201–209, initialize HP to three times reviewed base strength, preserve record 200 as normally unreachable, and retain a bounded trace whose quantified tail risk and maximum state size satisfy the development-provider contract. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic 256-draw progression, serialized-state ceiling proof, and encounter-state assertions. |
| REQ-005 | When Level 21 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 21 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through twenty-one and visibly enter a signed Level 21 encounter through provider-backed trusted QML, with every current choice occupying one non-overlapping delegate row, exposing one native accessibility press action with current bounds, and activating exactly once, without Level 22, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, a one-turn 23-to-24 delegate retirement/materialization replacement, unique-control and real-input checks, native accessibility action/bounds audit, a seven-action provider-backed QML lifecycle, live profile twice across restart, signed cartridge conformance, scope/security inspection, and visible workspace-8 play launched only after the compositor exposes a real output. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port only the normal Level 21 band and preserve event routing as out of scope. | `DUNGEONC.PAS` keeps ordinary monster loading in the event-false branch; events remain an independent unfinished subsystem. |
| 2 | Retain editor record 200 in canonical data but keep normal selection restricted to 201–209 through the original rejection loop. | Record inventory and reachable runtime behavior are separate compatibility facts. |
| 3 | Advance rules, state schema, and cartridge identities together to v26. | Older serialized state and signed action surfaces must not be relabeled as the deeper rules release. |
| 4 | Reuse the generic provider-owned dungeon/combat reducer and add only Level 21 data, bounds, actions, projections, and tests. | OmarchyGS remains transport/rendering authority, never a second game-rules owner. |
| 5 | Add one bounded twenty-first-choice string to the external game view/schema and bind it to exactly one Level 21 action. | The existing A–T fields are occupied on the dungeon screen; widening the data-only game view is smaller than overloading another semantic field or changing platform rendering. |
| 6 | Retain the 256-draw trace only if Level 21 has a valid at-cap deterministic progression and remains below the Provider Starter's 32 KiB state ceiling. | The source loop is unbounded and acceptance narrows with depth; the development cap must remain evidence-driven. |
| 7 | Ratchet the trusted-QML regression from twenty-three to twenty-four current controls, retire the old delegates for one event-loop turn before materializing a replacement plan, and use native button accessibility semantics without a second manual press action. | The user's duplicate/inert reports and the reproduced stale AT-SPI bounds/duplicate press action make delegate lifecycle, native geometry/action exposure, enablement, and exactly-once real pointer, keyboard, and provider activation release-blocking evidence for every larger dungeon surface. |

## Linked artifacts

- Ticket: [TICKET-075](../../tickets/closed/TICKET-075-usurper-level-twenty-one-dungeon-band.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: continuing goal after completing the Level 20 normal dungeon band.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | concrete shippable contract |
| 2 Design | Source/data/control-flow design and exact manifest | worktree-bound CodeGraph receipt |
| 3 Implement | External rules/data/provider/cartridge Level 21 slice | compile and self-review |
| 3.5 Inspect | Findings ledger and security review | lead disposition |
| 4 Validate | Focused/full tests, live corpus, and delivery gate | matching gate receipt |
| 5 Complete | AC audit, docs, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | explicit user authorization only |
