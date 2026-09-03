---
title: TICKET-059-usurper-level-seven-dungeon-band
status: closed
ticket_number: 059
type: feature
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-seven-dungeon-band.spec.md
---

# TICKET-059-usurper-level-seven-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and inert cartridge with
the exact normal level-seven dungeon band from the authenticated v0.20e source,
then prove it through provider replay and a visible trusted-QML preview.

## Why

Ticket 058 stopped deliberately at level six. Level seven is the next
source-complete playable increment and extends the established dungeon path
without introducing events, shared-realm state, or platform-owned game rules.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e level-seven editor records and retain the established dungeon change-level, encounter-selection, and monster-HP source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through seven and exact rules schema v12, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through seven from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a level-seven encounter, the reducer shall spend one fight and repeat `Random(70)` until the result is greater than sixty, select exact editor record 61–69, initialize HP to three times reviewed base strength, and preserve record 60 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When level-seven combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the level-seven retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through seven and visibly enter a signed level-seven encounter through existing provider and trusted-QML protocols, without level eight, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed-action/view tests, live profile twice across restart, signed cartridge conformance, QML smoke, scope review, security inspection, and visible preview. |

## Scope

- In:
  - v0.20e monster records 60–69 with reviewed strength and equipment flags;
  - source-order `Random(70)` rejection draws, normal selection 61–69, and
    source-derived 51 HP;
  - strict rules/state v12, levels one through seven, provider action/replay,
    signed inert cartridge, fixtures, provenance, tests, documentation, and
    visible preview.
- Out:
  - level eight or higher and composite dungeon event/team/shared-world paths;
  - platform gameplay logic, migrations, new provider protocol, executable
    cartridge content, packaging, admission, deployment, commit, push, or
    publication.

## Links

- Intake: none; the user asked to continue the established Usurper build.
- Pipeline spec: [usurper-level-seven-dungeon-band.spec.md](../../pipeline/completed/usurper-level-seven-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
