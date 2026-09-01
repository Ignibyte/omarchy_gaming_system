---
title: TICKET-053-usurper-gnoll-poisonous-bite
status: closed
ticket_number: 053
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-gnoll-poisonous-bite.spec.md
---

# TICKET-053-usurper-gnoll-poisonous-bite

## Summary

Extend the separate Usurper v0.20e Rust port with the original Gnoll
poisonous-bite passive and persistent per-encounter monster poison damage,
including its exact combat-turn order in the signed visible combat view.

## Why

The current game offers Gnoll during character creation but does not implement
the racial ability advertised by the original. This slice makes that choice
meaningfully distinct without importing Alchemist weapon poison, Sage poison,
PvP, teams, or broader dungeon events.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Gnoll race description, bite eligibility and roll, monster poison initialization, poison-tick timing/damage, and encounter completion sources. | Canonical-source readback, source-trace validation, fixed constants, and compatibility documentation review. |
| REQ-002 | When a new monster encounter begins, its poison state shall start false and remain a bounded provider-owned encounter fact until that encounter ends, while malformed persisted state shall fail without advancing RNG. | Creation/reset fixtures, state validation, hostile JSON tests, and complete-state/RNG equality. |
| REQ-003 | When a Gnoll completes an offensive combat turn against an unpoisoned monster, the reducer shall consume `Random(4)+1` at the original point after ordinary attack calculation and applicable spell or Soul Strike effects but before strike resolution, and shall poison only when the result equals three. | Attack, quick-heal, spell, Backstab, and Soul Strike draw-trace fixtures plus non-Gnoll/already-poisoned controls. |
| REQ-004 | When a living monster is poisoned after the player action, the reducer shall apply `Random(5)+1` poison damage before the monster response on that and later turns, preserve the status without rerolling the bite, skip the response if poison is lethal, and preserve the original no-immediate-reward poison-lethal oddity. | First/subsequent tick, cumulative damage, response-order, lethal, reward-conservation, and deterministic-twin tests. |
| REQ-005 | When combat is driven through the provider or rendered through the cartridge, the existing attack and other eligible commands shall expose poisoned status and bite/tick narration without adding a fabricated race-special action, while preserving exact revision, replay, restart, presentation, and trusted-QML behavior. | Provider transition/replay tests, live Gnoll profile twice across restart, signed cartridge conformance, QML smoke, and visible combat preview. |
| REQ-006 | When this slice completes, Usurper shall remain a separate unadmitted player-private provider/cartridge with no Alchemist weapon poison, Sage poison spell, PvP/NPC poison, teams, disease, multi-monster expansion, shared realm, platform gameplay copy, schema/route change, packaging, or publication. | Cross-repository dependency/content review, CodeGraph inspection, security scan, scope scan, external checks, and full platform diff gate. |

## Scope

- In:
  - source-linked Gnoll bite eligibility, exact roll, and poison persistence;
  - poison ticks before same-turn monster response, including lethal behavior;
  - deterministic provider state, existing combat commands, signed view data,
    tests, provenance, and visible preview.
- Out:
  - Alchemist weapon poison, Sage poison, PvP/NPC poison, teams, diseases,
    multiple monsters, events, quests, shared state, and unrelated races;
  - packaging, production registration, admission, deployment, or publication.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-gnoll-poisonous-bite.spec.md](../../pipeline/completed/usurper-gnoll-poisonous-bite.spec.md)
- Prior milestone: [Ticket 052](../closed/TICKET-052-usurper-assassin-backstab-and-paladin-soul-strike.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
