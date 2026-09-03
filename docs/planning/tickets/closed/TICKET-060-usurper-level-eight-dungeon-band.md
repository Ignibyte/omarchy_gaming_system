---
title: TICKET-060-usurper-level-eight-dungeon-band
status: closed
ticket_number: 060
type: feature
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-eight-dungeon-band.spec.md
---

# TICKET-060-usurper-level-eight-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and inert cartridge with
the exact normal level-eight dungeon band from the authenticated v0.20e source,
then prove it through provider replay and a visible trusted-QML preview.

## Why

Ticket 059 stopped deliberately at level seven. Level eight is the next
source-complete playable increment and extends the established dungeon path
without introducing events, shared-realm state, or platform-owned game rules.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e level-eight editor records and retain the established dungeon change-level, encounter-selection, and monster-HP source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through eight and exact rules schema v13, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through eight from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a level-eight encounter, the reducer shall spend one fight and repeat `Random(80)` until the result is greater than seventy, select exact editor record 71–79, initialize HP to three times reviewed base strength, and preserve record 70 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When level-eight combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the level-eight retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through eight and visibly enter a signed level-eight encounter through existing provider and trusted-QML protocols, without level nine, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed-action/view tests, live profile twice across restart, signed cartridge conformance, QML smoke, scope review, security inspection, and visible preview. |

## Scope

- In:
  - v0.20e monster records 70–79 with reviewed strength and equipment flags;
  - source-order `Random(80)` rejection draws, normal selection 71–79, and
    source-derived 54 HP;
  - strict rules/state v13, levels one through eight, provider action/replay,
    signed inert cartridge, fixtures, provenance, tests, documentation, and
    visible preview.
- Out:
  - level nine or higher and composite dungeon event/team/shared-world paths;
  - platform gameplay logic, migrations, new provider protocol, executable
    cartridge content, packaging, admission, deployment, commit, push, or
    publication.

## Links

- Intake: none; the user asked to continue the established Usurper build.
- Pipeline spec: [usurper-level-eight-dungeon-band.spec.md](../../pipeline/completed/usurper-level-eight-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)

## Outcome

Completed as rules/state/cartridge v13 in the separate development repository.
Level 8 preserves source record 70 as normally unreachable, selects records
71–79 through the original `Random(80)` rejection loop, and initializes combat
at strength 18, defence 9, and 54 HP. All external checks, live provider restart
conformance, security inspection, platform diff gate, trusted-QML screen smoke,
OpenWiki reconciliation, and visible workspace-8 rendering passed. Packaging,
admission, deployment, commit, push, and publication remain deferred.
