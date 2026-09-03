---
title: TICKET-063-usurper-level-ten-dungeon-band
status: closed
ticket_number: 063
type: feature
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-ten-dungeon-band.spec.md
---

# TICKET-063-usurper-level-ten-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and signed inert cartridge
with the exact normal Level 10 dungeon band from authenticated v0.20e source,
then prove it through provider replay and provider-backed trusted-QML play.

## Why

Ticket 062 completed Level 9 and repaired the live shell's stale, duplicate,
and auto-repeated controls. Level 10 is the next source-complete progression
slice and uses the established normal-encounter branch without requiring
dungeon events, shared realm state, or another combat subsystem.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 10 editor records 90–99 and retain the established dungeon selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through ten and exact rules schema v15, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through ten from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 10 encounter, the reducer shall spend one fight and repeat `Random(100)` until the result is greater than ninety, select exact editor record 91–99, initialize HP to three times reviewed base strength, and preserve record 90 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When Level 10 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 10 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through ten and visibly enter a signed Level 10 encounter through provider-backed trusted QML, with each choice rendered and activated exactly once and without Level 11, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, unique-control checks, live profile twice across restart, signed cartridge conformance, local-play click smoke, scope/security inspection, and visible workspace-8 play. |

## Scope

- In:
  - v0.20e monster records 90–99 with reviewed base strength and equipment
    flags;
  - source-order `Random(100)` rejection draws, normal selection 91–99, and
    source-derived 60 HP;
  - strict rules/state v15, levels one through ten, provider action/replay,
    signed inert cartridge, fixtures, provenance, tests, documentation, and
    provider-backed visible play;
  - regression proof that visible controls remain unique and one activation
    produces one provider revision.
- Out:
  - Level 11 or higher and composite dungeon event/team/shared-world paths;
  - new spells, specials, equipment, poison variants, quests, or social state;
  - platform gameplay logic, migrations, new provider protocol, packaging,
    admission, deployment, commit, push, or publication.

## Links

- Intake: none; the user asked to continue building the established Usurper game.
- Pipeline spec: [usurper-level-ten-dungeon-band.spec.md](../../pipeline/completed/usurper-level-ten-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
