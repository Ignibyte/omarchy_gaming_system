---
title: TICKET-051-usurper-level-one-spellcasting-and-mana-loop
status: closed
ticket_number: 051
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-one-spellcasting-and-mana-loop.spec.md
---

# TICKET-051-usurper-level-one-spellcasting-and-mana-loop

## Summary

Extend the separate Usurper v0.20e Rust port with the original level-one
Cleric, Magician, and Sage spell paths, their mana lifecycle, and a visible
combat action rendered by the existing trusted QML cartridge host.

## Why

The current game creates the original caster classes and displays mana but
cannot use either. This slice connects character creation, combat, daily
maintenance, provider replay, and the visible battle screen without waiting
for packaging or importing the unrelated higher-level spell catalog.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e creation, spell metadata, cast, combat-turn, encounter-reset, mana-maintenance, and development monster-resistance sources, while explicitly distinguishing the editor's source-backed development fixture from an unavailable initialized `MONSTER.DAT`. | Canonical-source readback, source-trace validation, fixed constants, and compatibility documentation review. |
| REQ-002 | When a character is created, the reducer shall preserve the original learned first-spell flag and class mana values, expose spellcasting only to Cleric, Magician, and Sage characters, and reject unknown, unlearned, wrong-phase, active, or unaffordable casts without changing state or RNG. | All-class creation table, strict command/state boundary tests, rejected-state equality, and provider malformed-input tests. |
| REQ-003 | When a caster uses the learned level-one spell in dungeon combat, the reducer shall spend exactly ten mana and preserve the original class effect: Cure Light restores `4 + Random(3)` HP up to maximum, Magic Missile rolls `4 + Random(3)` damage subject to the mapped monster resistance, and Fog of War activates three points of whole-fight absorption. | Source-distinguishing class fixtures, mana conservation, resistance pass/fail, HP caps, damage/victory, RNG-bound/index assertions, and deterministic twins. |
| REQ-004 | When an accepted spell leaves a monster alive, the monster shall take its normal response in the same combat turn; when a new encounter begins, temporary spell state shall reset before selection; and when the realm day advances, mana shall refill to the character's current maximum. | Turn-order/player-death/monster-victory fixtures, consecutive-encounter reset tests, sleep/re-entry tests, and full-day regression. |
| REQ-005 | When the combat view is rendered or driven through the provider, it shall expose current/max mana, the class-specific learned spell, and one bounded fixed cast action while preserving exact revision, replay, restart, presentation, and trusted-QML behavior. | Provider adapter tests, generated-profile round trip, live conformance twice across restart, deterministic cartridge conformance, QML smoke, and visible Magician-combat preview. |
| REQ-006 | When this slice completes, Usurper shall remain a separate unadmitted player-private provider/cartridge with no higher-level spells, Magic Shop object catalog, monster spells, teams, specials, shared-realm state, platform gameplay copy, schema/route change, packaging, or publication. | Cross-repository dependency/content review, CodeGraph inspection, scope scan, external checks, and full platform diff gate. |

## Scope

- In:
  - source-linked level-one spell metadata and learned/active state;
  - Cleric Cure Light, Magician Magic Missile, and Sage Fog of War;
  - mana cost, daily refill, combat response, resistance, and encounter reset;
  - strict provider action, signed combat presentation, and visible preview.
- Out:
  - spell levels two through twelve, monster spells, magic objects, poison,
    class/race specials, teams, events, quests, finale, and shared state;
  - packaging, production registration, admission, deployment, or publication.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-level-one-spellcasting-and-mana-loop.spec.md](../../pipeline/completed/usurper-level-one-spellcasting-and-mana-loop.spec.md)
- Prior milestone: [Ticket 050](../closed/TICKET-050-usurper-healing-potion-shop-and-combat-turn-parity.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
