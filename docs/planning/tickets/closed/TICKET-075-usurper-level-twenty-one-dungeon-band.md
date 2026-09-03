---
title: TICKET-075-usurper-level-twenty-one-dungeon-band
status: closed
ticket_number: 075
type: feature
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-level-twenty-one-dungeon-band.spec.md
---

# TICKET-075-usurper-level-twenty-one-dungeon-band

## Summary

Extend the separate deterministic Usurper provider and signed inert cartridge
with the exact normal Level 21 dungeon band from authenticated v0.20e source,
then prove it through provider replay and provider-backed trusted-QML play.

## Why

Ticket 074 completed Level 20 and corrected the invalid placeholder-output and
trusted-QML delegate evidence behind the user's earlier duplicate/inert-control
report. Level 21 is the next source-complete ordinary dungeon slice. Its
twenty-four-control dungeon surface must preserve the clean v25 build's unique
rows and working real input.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 21 editor records 200–209 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through twenty-one and exact rules schema v26, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through twenty-one from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 21 encounter, the reducer shall spend one fight and repeat `Random(210)` until the result is greater than two hundred, select exact editor record 201–209, initialize HP to three times reviewed base strength, preserve record 200 as normally unreachable, and retain a bounded trace whose quantified tail risk and maximum state size satisfy the development-provider contract. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic 256-draw progression, serialized-state ceiling proof, and encounter-state assertions. |
| REQ-005 | When Level 21 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 21 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through twenty-one and visibly enter a signed Level 21 encounter through provider-backed trusted QML, with every current choice occupying one non-overlapping delegate row, exposing one native accessibility press action with current bounds, and activating exactly once, without Level 22, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, a one-turn 23-to-24 delegate retirement/materialization replacement, unique-control and real-input checks, native accessibility action/bounds audit, live profile twice across restart, signed cartridge conformance, local-play smoke, scope/security inspection, and visible workspace-8 play. |

## Scope

- In:
  - v0.20e monster records 200–209 with reviewed base strength and equipment
    flags;
  - source-order `Random(210)` rejection draws, normal selection 201–209,
    source-derived 75 HP, and explicit bounded-trace risk/size evidence;
  - strict rules/state/cartridge v26, levels one through twenty-one, bounded
    twenty-first-choice view/schema capacity, provider action/replay, signed
    inert cartridge, fixtures, provenance, tests, documentation, and
    provider-backed play;
  - regression proof that twenty-four visible dungeon controls remain unique,
    non-overlapping, enabled, and produce exactly one action each.
- Out:
  - Level 22 or higher and composite dungeon event/team/shared-world paths;
  - new spells, specials, equipment, poison variants, quests, or social state;
  - platform gameplay logic, migrations, new provider protocol, packaging,
    admission, deployment, commit, push, or publication.

## Links

- Intake: the active goal is to continue building the established Usurper game.
- Pipeline spec: [usurper-level-twenty-one-dungeon-band.spec.md](../../pipeline/completed/usurper-level-twenty-one-dungeon-band.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
