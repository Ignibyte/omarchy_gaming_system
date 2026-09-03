---
title: TICKET-066-usurper-level-twelve-dungeon-band
status: done
ticket_number: 066
type: feature
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-twelve-dungeon-band.spec.md
---

# TICKET-066-usurper-level-twelve-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and signed inert cartridge
with the exact normal Level 12 dungeon band from authenticated v0.20e source,
then prove it through provider replay and provider-backed trusted-QML play.

## Why

Ticket 064 completed Level 11 and Ticket 065 proved the repaired real pointer,
keyboard, delegate, and one-action paths. Level 12 is the next source-complete
ordinary dungeon slice and fits the established provider-owned reducer, with
one bounded view-schema extension for its twelfth visible level choice.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 12 editor records 110–119 and retain the established dungeon selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through twelve and exact rules schema v17, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through twelve from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 12 encounter, the reducer shall spend one fight and repeat `Random(120)` until the result is greater than one hundred ten, select exact editor record 111–119, initialize HP to three times reviewed base strength, and preserve record 110 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When Level 12 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 12 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through twelve and visibly enter a signed Level 12 encounter through provider-backed trusted QML, with each choice rendered and activated exactly once and without Level 13, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, unique-control and real-input checks, live profile twice across restart, signed cartridge conformance, local-play click smoke, scope/security inspection, and visible workspace-8 play. |

## Scope

- In:
  - v0.20e monster records 110–119 with reviewed base strength and equipment
    flags;
  - source-order `Random(120)` rejection draws, normal selection 111–119, and
    source-derived 60 HP;
  - strict rules/state/cartridge v17, levels one through twelve, `option_l`
    view/schema capacity, provider action/replay, signed inert cartridge,
    fixtures, provenance, tests, documentation, and provider-backed play;
  - regression proof that visible controls remain unique and real activation
    produces one provider revision.
- Out:
  - Level 13 or higher and composite dungeon event/team/shared-world paths;
  - new spells, specials, equipment, poison variants, quests, or social state;
  - platform gameplay logic, migrations, new provider protocol, packaging,
    admission, deployment, commit, push, or publication.

## Links

- Intake: the active goal is to continue building the established Usurper game.
- Pipeline spec: [usurper-level-twelve-dungeon-band.spec.md](../../pipeline/completed/usurper-level-twelve-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
