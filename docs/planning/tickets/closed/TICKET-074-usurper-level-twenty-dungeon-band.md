---
title: TICKET-074-usurper-level-twenty-dungeon-band
status: closed
ticket_number: 074
type: feature
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-twenty-dungeon-band.spec.md
---

# TICKET-074-usurper-level-twenty-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and signed inert cartridge
with the exact normal Level 20 dungeon band from authenticated v0.20e source,
then prove it through provider replay and provider-backed trusted-QML play.

## Why

Ticket 073 completed Level 19 and strengthened the trusted-QML lifecycle proof
after the user's duplicate and inert-control report. Level 20 is the next
source-complete ordinary dungeon slice and fits the established provider-owned
path with one bounded external view-schema field.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 20 editor records 190–199 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through twenty and exact rules schema v25, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through twenty from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 20 encounter, the reducer shall spend one fight and repeat `Random(200)` until the result is greater than one hundred ninety, select exact editor record 191–199, initialize HP to three times reviewed base strength, preserve record 190 as normally unreachable, and retain a bounded trace whose quantified tail risk and maximum state size satisfy the development-provider contract. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic 256-draw progression, serialized-state ceiling proof, and encounter-state assertions. |
| REQ-005 | When Level 20 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 20 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through twenty and visibly enter a signed Level 20 encounter through provider-backed trusted QML, with every current choice occupying one non-overlapping delegate row and activating exactly once, without Level 21, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, 22-to-23 non-overlapping delegate replacement, unique-control and real-input checks, live profile twice across restart, signed cartridge conformance, local-play smoke, scope/security inspection, and visible workspace-8 play. |

## Scope

- In:
  - v0.20e monster records 190–199 with reviewed base strength and equipment
    flags;
  - source-order `Random(200)` rejection draws, normal selection 191–199,
    source-derived 72 HP, and explicit bounded-trace risk/size evidence;
  - strict rules/state/cartridge v25, levels one through twenty, bounded
    twentieth-choice view/schema capacity, provider action/replay, signed inert
    cartridge, fixtures, provenance, tests, documentation, and provider-backed
    play;
  - regression proof that twenty-three visible dungeon controls remain unique,
    non-overlapping, enabled, and produce exactly one action each.
- Out:
  - Level 21 or higher and composite dungeon event/team/shared-world paths;
  - new spells, specials, equipment, poison variants, quests, or social state;
  - platform gameplay logic, migrations, new provider protocol, packaging,
    admission, deployment, commit, push, or publication.

## Links

- Intake: the active goal is to continue building the established Usurper game.
- Pipeline spec: [usurper-level-twenty-dungeon-band.spec.md](../../pipeline/completed/usurper-level-twenty-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
