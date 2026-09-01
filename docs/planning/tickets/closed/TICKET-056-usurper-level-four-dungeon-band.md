---
title: TICKET-056-usurper-level-four-dungeon-band
status: closed
ticket_number: 056
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-four-dungeon-band.spec.md
---

# TICKET-056-usurper-level-four-dungeon-band

## Summary

Extend the separate Usurper v0.20e Rust port with the original level-four
dungeon monster band, four-level controls, exact encounter rejection loop,
and a signed visible level-four combat path.

## Why

The level-three milestone proved that additional source-backed dungeon depth
composes through the deterministic rules, remote provider, signed cartridge,
and trusted QML boundary. Level four is the next complete normal-monster band
and deepens the playable port without importing the original composite event
dispatcher or shared-world state.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e level-four editor records and retain the established dungeon change-level, encounter-selection, and monster-HP source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through four and exact rules schema v9, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one, two, three, or four from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a level-four encounter, the reducer shall spend one fight and repeat `Random(40)` until the result is greater than thirty, select exact editor record 31–39, initialize HP to three times reviewed base strength, and preserve record 30 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When level-four combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the level-four retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through four and visibly enter a signed level-four encounter through existing provider and trusted-QML protocols, without level five, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed-action/view tests, live profile twice across restart, signed cartridge conformance, QML smoke, scope review, security inspection, and visible preview. |

## Scope

- In:
  - exact level-four normal-monster records and rejection-loop selection;
  - bounded level-one through level-four selection inside the existing solo dungeon;
  - level-aware encounter validation, combat composition, provider actions,
    signed screens, provenance, tests, and visible preview.
- Out:
  - level five or higher, dungeon events, Uman/Ice caves, teams, PvP/NPCs,
    quests, finale, immortality, shared realm state, or platform gameplay code;
  - packaging, production registration, admission, deployment, or publication.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-level-four-dungeon-band.spec.md](../../pipeline/completed/usurper-level-four-dungeon-band.spec.md)
- Prior milestone: [Ticket 055](../closed/TICKET-055-usurper-level-three-dungeon-band.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
