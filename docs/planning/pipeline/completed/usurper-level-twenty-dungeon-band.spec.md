---
title: Usurper Level Twenty Dungeon Band
pipeline_id: b14f7737-8d10-47ed-9f34-5f376c56a0f0
status: Phase 5 — Complete PASS
ticket: TICKET-074
ticket_doc: docs/planning/tickets/closed/TICKET-074-usurper-level-twenty-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-074-usurper-level-twenty-dungeon-band.md
created: 2026-09-03
---

# Usurper Level Twenty Dungeon Band — spec

## Intent

Advance the separate source-linked Usurper port by one complete normal dungeon
band while preserving provider ownership, signed inert presentation, unique
non-overlapping controls, real-input proof, and all existing combat behavior.

## Scope

- In:
  - exact v0.20e Level 20 monster records 190–199;
  - normal Level 20 rejection-loop selection, three-times-strength HP, and
    bounded trace risk/size evidence;
  - draw-free level switching across levels one through twenty;
  - rules/state/cartridge v25, bounded twentieth-choice projection,
    deterministic reducers, provider projections, signed presentation,
    fixtures, provenance, documentation, and tests;
  - game-neutral trusted-QML regression ratchet for one non-overlapping,
    exactly-once control per current dungeon choice, plus a provider-backed
    multi-screen action/plan-replacement smoke path;
  - live provider restart conformance and provider-backed workspace-8 play.
- Out:
  - Level 21+, dungeon events, quests, finale, shared realm, or new combat
    systems;
  - platform rules/state, database, Provider SDK/protocol, renderer protocol,
    renderer compiler, or production trusted-QML behavior changes;
  - registration, admission, packaging, deployment, commit, push, or
    publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 20 editor records 190–199 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through twenty and exact rules schema v25, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through twenty from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 20 encounter, the reducer shall spend one fight and repeat `Random(200)` until the result is greater than one hundred ninety, select exact editor record 191–199, initialize HP to three times reviewed base strength, preserve record 190 as normally unreachable, and retain a bounded trace whose quantified tail risk and maximum state size satisfy the development-provider contract. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic 256-draw progression, serialized-state ceiling proof, and encounter-state assertions. |
| REQ-005 | When Level 20 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 20 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through twenty and visibly enter a signed Level 20 encounter through provider-backed trusted QML, with every current choice occupying one non-overlapping delegate row and activating exactly once, without Level 21, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, 22-to-23 non-overlapping delegate replacement, unique-control and real-input checks, a seven-action provider-backed QML lifecycle, live profile twice across restart, signed cartridge conformance, scope/security inspection, and visible workspace-8 play launched only after the compositor exposes a real output. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port only the normal Level 20 band and preserve event routing as out of scope. | `DUNGEONC.PAS` keeps ordinary monster loading in the event-false branch; events remain an independent unfinished subsystem. |
| 2 | Retain editor record 190 in canonical data but keep normal selection restricted to 191–199 through the original rejection loop. | Record inventory and reachable runtime behavior are separate compatibility facts. |
| 3 | Advance rules, state schema, and cartridge identities together to v25. | Older serialized state and signed action surfaces must not be relabeled as the deeper rules release. |
| 4 | Reuse the generic provider-owned dungeon/combat reducer and add only Level 20 data, bounds, actions, projections, and tests. | OmarchyGS remains transport/rendering authority, never a second game-rules owner. |
| 5 | Add one bounded twentieth-choice string to the external game view/schema and bind it to exactly one Level 20 action. | The existing A–S fields are occupied on the dungeon screen; widening the data-only game view is smaller than overloading another semantic field or changing platform rendering. |
| 6 | Retain the 256-draw trace only if Level 20 has a valid at-cap deterministic progression and remains below the Provider Starter's 32 KiB state ceiling. | The source loop is unbounded and acceptance narrows with depth; the development cap must remain evidence-driven. |
| 7 | Ratchet the trusted-QML regression from twenty-two to twenty-three current controls and drive a provider-confirmed seven-action lifecycle without adding production UI behavior. | The user's inert/duplicate reports make delegate cardinality, geometry, enablement, plan replacement, and exactly-once real pointer, keyboard, and provider activation release-blocking evidence for every larger dungeon surface. |

## Linked artifacts

- Ticket: [TICKET-074](../../tickets/closed/TICKET-074-usurper-level-twenty-dungeon-band.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: continuing goal after completing the Level 19 normal dungeon band.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | concrete shippable contract |
| 2 Design | Source/data/control-flow design and exact manifest | worktree-bound CodeGraph receipt |
| 3 Implement | External rules/data/provider/cartridge Level 20 slice | compile and self-review |
| 3.5 Inspect | Findings ledger and security review | lead disposition |
| 4 Validate | Focused/full tests, live corpus, and delivery gate | matching gate receipt |
| 5 Complete | AC audit, docs, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | explicit user authorization only |
