---
title: TICKET-054-usurper-level-two-dungeon-band
status: closed
ticket_number: 054
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-two-dungeon-band.spec.md
---

# TICKET-054-usurper-level-two-dungeon-band

## Summary

Extend the separate Usurper v0.20e Rust port with the original level-two
dungeon monster band, change-level controls, exact encounter selection loop,
and a signed visible level-two combat path.

## Why

The first dungeon level now composes character creation, equipment, potions,
spells, class specials, combat, and Gnoll poison. A second source-backed band
is the smallest coherent slice that turns those systems into a deeper playable
dungeon without importing shared-world state or the original composite event
dispatcher.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e dungeon default, change-level range, encounter-selection loop, level-two editor records, and monster-HP initialization sources. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with the implemented levels one and two and exact rules schema v7, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one or two from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a level-two encounter, the reducer shall spend one fight and repeat `Random(20)` until the result is greater than ten, select exact editor record 11–19, initialize HP to three times reviewed base strength, and preserve record 10 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When level-two combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll poison behavior shall compose unchanged except for source-defined level/monster inputs, including the level-two retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch between levels one and two and visibly enter a signed level-two encounter through existing provider and trusted-QML protocols, without level three, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed-action/view tests, live profile twice across restart, signed cartridge conformance, QML smoke, scope review, security inspection, and visible preview. |

## Scope

- In:
  - exact level-two normal-monster records and rejection-loop selection;
  - bounded level-one/level-two selection inside the existing solo dungeon;
  - level-aware encounter validation, combat composition, provider actions,
    signed screens, provenance, tests, and visible preview.
- Out:
  - level three or higher, dungeon events, Uman/Ice caves, teams, PvP/NPCs,
    quests, finale, immortality, shared realm state, or platform gameplay code;
  - packaging, production registration, admission, deployment, or publication.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-level-two-dungeon-band.spec.md](../../pipeline/completed/usurper-level-two-dungeon-band.spec.md)
- Prior milestone: [Ticket 053](../closed/TICKET-053-usurper-gnoll-poisonous-bite.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
