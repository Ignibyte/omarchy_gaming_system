---
title: TICKET-057-usurper-level-five-dungeon-band
status: closed
ticket_number: 057
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-five-dungeon-band.spec.md
---

# TICKET-057-usurper-level-five-dungeon-band

## Summary

Extend the separate Usurper v0.20e Rust port with the original level-five
dungeon monster band, five-level controls, exact encounter rejection loop,
and a signed visible level-five combat path.

## Why

The level-four milestone proved that source-backed dungeon depth continues to
compose through the deterministic rules, remote provider, signed cartridge,
and trusted QML boundary. Level five is the next complete normal-monster band
and deepens the playable port without importing the original composite event
dispatcher or shared-world state.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e level-five editor records and retain the established dungeon change-level, encounter-selection, and monster-HP source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through five and exact rules schema v10, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one, two, three, four, or five from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a level-five encounter, the reducer shall spend one fight and repeat `Random(50)` until the result is greater than forty, select exact editor record 41–49, initialize HP to three times reviewed base strength, and preserve record 40 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When level-five combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the level-five retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through five and visibly enter a signed level-five encounter through existing provider and trusted-QML protocols, without level six, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed-action/view tests, live profile twice across restart, signed cartridge conformance, QML smoke, scope review, security inspection, and visible preview. |

## Scope

- In:
  - exact level-five normal-monster records and rejection-loop selection;
  - bounded level-one through level-five selection inside the existing solo dungeon;
  - level-aware encounter validation, combat composition, provider actions,
    signed screens, provenance, tests, and visible preview.
- Out:
  - level six or higher, dungeon events, Uman/Ice caves, teams, PvP/NPCs,
    quests, finale, immortality, shared realm state, or platform gameplay code;
  - packaging, production registration, admission, deployment, commit, push,
    or publication.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-level-five-dungeon-band.spec.md](../../pipeline/completed/usurper-level-five-dungeon-band.spec.md)
- Prior milestone: [Ticket 056](../closed/TICKET-056-usurper-level-four-dungeon-band.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
