---
title: TICKET-052-usurper-assassin-backstab-and-paladin-soul-strike
status: closed
ticket_number: 052
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-assassin-backstab-and-paladin-soul-strike.spec.md
---

# TICKET-052-usurper-assassin-backstab-and-paladin-soul-strike

## Summary

Extend the separate Usurper v0.20e Rust port with the original dungeon-combat
class-special slot: weapon-gated Assassin Backstab and hit-point-funded Paladin
Soul Strike, including their attack and monster-response order in the signed
combat view.

## Why

The current game makes the three spellcasting classes distinct in combat, but
the original Assassin and Paladin choices still fall back to an ordinary
attack. These two specials share one source menu seam and make two more classes
meaningfully playable without importing PvP, poison, teams, or random dungeon
events.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e creation defaults, dungeon-combat menu gates, Backstab branch, Soul Strike branch, soul-effect formula, normal-attack composition, and monster-response sources. | Canonical-source readback, source-trace validation, fixed constants, and compatibility documentation review. |
| REQ-002 | When a combat-special command is submitted, the reducer shall admit Backstab only for an Assassin wielding a weapon and Soul Strike only for a Paladin investing from one through current HP minus one, and shall reject every wrong-class, wrong-phase, unarmed, zero, overdrawn, or malformed command without changing state or RNG. | Strict command tables, complete-state/RNG equality, boundary tests, and malformed provider input tests. |
| REQ-003 | When an armed Assassin uses Backstab, the reducer shall roll `Random(3)` before the normal attack; on success it shall add half maximum HP to that attack, while on failure it shall suppress player damage and add level plus three to the same-turn monster response. | Success/failure/lethal fixtures, exact RNG-bound/index assertions, damage and counterattack assertions, and deterministic twins. |
| REQ-004 | When a Paladin uses Soul Strike, the reducer shall spend the selected HP before the original mental-health and addiction checks, preserve their conditional draw order, add `Random(invested HP) + level` to the normal attack on success, and still resolve the living monster's response in that turn. | Default and degraded-condition fixtures, HP conservation, success/failure/victory/death tests, exact RNG traces, and deterministic twins. |
| REQ-005 | When the combat view is rendered or driven through the provider, it shall expose only the source-eligible class special through one bounded fixed action that selects Backstab or a one-HP Soul Strike from authenticated current class state, and preserve exact revision, replay, restart, presentation, and trusted-QML behavior. | Provider generic/fixed equivalence tests, live Assassin profile twice across restart, signed cartridge conformance, QML smoke, and visible combat preview. |
| REQ-006 | When this slice completes, Usurper shall remain a separate unadmitted player-private provider/cartridge with no Gnoll poison, PvP/NPC specials, teams, mercy/fight-to-death, higher spells, dungeon events, shared-realm state, platform gameplay copy, schema/route change, packaging, or publication. | Cross-repository dependency/content review, CodeGraph inspection, scope scan, external checks, and full platform diff gate. |

## Scope

- In:
  - source-linked mental-health/addiction creation defaults needed by Soul Strike;
  - Assassin Backstab success/failure and Paladin Soul Strike investment/effect;
  - normal-attack and monster-response composition with strict commands;
  - provider actions, signed combat presentation, tests, and visible preview.
- Out:
  - Gnoll poison, PvP/NPC special use, teams, mercy, fight-to-death, higher
    spells, monster magic, random dungeon events, quests, and shared state;
  - packaging, production registration, admission, deployment, or publication.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-assassin-backstab-and-paladin-soul-strike.spec.md](../../pipeline/completed/usurper-assassin-backstab-and-paladin-soul-strike.spec.md)
- Prior milestone: [Ticket 051](../closed/TICKET-051-usurper-level-one-spellcasting-and-mana-loop.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
