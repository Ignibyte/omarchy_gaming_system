---
title: TICKET-062-usurper-level-nine-dungeon-band
status: closed
ticket_number: 062
type: feature
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-nine-dungeon-band.spec.md
---

# TICKET-062-usurper-level-nine-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and signed inert cartridge
with the exact normal Level 9 dungeon band from authenticated v0.20e source,
then prove it through provider replay and provider-backed trusted-QML play.

## Why

Ticket 060 stopped deliberately at Level 8, and Ticket 061 made visible testing
honestly interactive. Level 9 is the next source-complete progression slice and
can reuse the established dungeon reducer without pulling in events, shared
realm state, or another combat subsystem.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 9 editor records 80–89 and retain the established dungeon selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through nine and exact rules schema v14, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through nine from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 9 encounter, the reducer shall spend one fight and repeat `Random(90)` until the result is greater than eighty, select exact editor record 81–89, initialize HP to three times reviewed base strength, and preserve record 80 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When Level 9 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 9 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through nine and visibly enter a signed Level 9 encounter through provider-backed trusted QML, without Level 10, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, live profile twice across restart, signed cartridge conformance, local-play click smoke, scope/security inspection, and visible workspace-8 play. |

## Scope

- In:
  - v0.20e monster records 80–89 with reviewed base strength and equipment
    flags;
  - source-order `Random(90)` rejection draws, normal selection 81–89, and
    source-derived 57 HP;
  - strict rules/state v14, levels one through nine, provider action/replay,
    signed inert cartridge, fixtures, provenance, tests, documentation, and
    provider-backed visible play.
- Out:
  - Level 10 or higher and composite dungeon event/team/shared-world paths;
  - new spells, specials, equipment, poison variants, quests, or social state;
  - platform gameplay logic, migrations, new provider protocol, packaging,
    admission, deployment, commit, push, or publication.

## Links

- Intake: none; the user asked to continue building the established Usurper game.
- Pipeline spec: [usurper-level-nine-dungeon-band.spec.md](../../pipeline/completed/usurper-level-nine-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)

## Outcome

Completed. The separate provider/cartridge now implements the exact normal
Level 9 band as strict v14, including records 80–89, source-order rejection
draws, normal selection 81–89, 57-HP combat, replay/restart proof, and visible
provider-backed workspace-8 play. The validation pass also repaired stale
post-mutation screen selection, duplicate command/navigation controls, and
trusted activation auto-repeat across plan replacement. All focused and full
external checks, the restarted provider corpus, final zero-finding security
scans, OpenWiki lifecycle, and the complete platform gate passed. No delivery
or publication was performed.
