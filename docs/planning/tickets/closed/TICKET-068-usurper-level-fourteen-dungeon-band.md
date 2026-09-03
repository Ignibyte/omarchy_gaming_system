---
title: TICKET-068-usurper-level-fourteen-dungeon-band
status: done
ticket_number: 068
type: feature
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-fourteen-dungeon-band.spec.md
---

# TICKET-068-usurper-level-fourteen-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and signed inert cartridge
with the exact normal Level 14 dungeon band from authenticated v0.20e source,
then prove it through provider replay and provider-backed trusted-QML play.

## Why

Ticket 067 completed Level 13 and strengthened the actual-delegate regression
at the trusted QML boundary. Level 14 is the next source-complete ordinary
dungeon slice and fits the established provider-owned reducer, with one bounded
external view-schema extension for its fourteenth visible level choice.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 14 editor records 130–139 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through fourteen and exact rules schema v19, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through fourteen from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 14 encounter, the reducer shall spend one fight and repeat `Random(140)` until the result is greater than one hundred thirty, select exact editor record 131–139, initialize HP to three times reviewed base strength, and preserve record 130 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When Level 14 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 14 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through fourteen and visibly enter a signed Level 14 encounter through provider-backed trusted QML, with each choice rendered and activated exactly once and without Level 15, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, unique-control and real-input checks, live profile twice across restart, signed cartridge conformance, local-play click smoke, scope/security inspection, and visible workspace-8 play. |

## Scope

- In:
  - v0.20e monster records 130–139 with reviewed base strength and equipment
    flags;
  - source-order `Random(140)` rejection draws, normal selection 131–139, and
    source-derived 60 HP;
  - strict rules/state/cartridge v19, levels one through fourteen, `option_n`
    view/schema capacity, provider action/replay, signed inert cartridge,
    fixtures, provenance, tests, documentation, and provider-backed play;
  - regression proof that visible controls remain unique and real activation
    produces one provider revision.
- Out:
  - Level 15 or higher and composite dungeon event/team/shared-world paths;
  - new spells, specials, equipment, poison variants, quests, or social state;
  - platform gameplay logic, migrations, new provider protocol, packaging,
    admission, deployment, commit, push, or publication.

## Links

- Intake: the active goal is to continue building the established Usurper game.
- Pipeline spec: [usurper-level-fourteen-dungeon-band.spec.md](../../pipeline/completed/usurper-level-fourteen-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
