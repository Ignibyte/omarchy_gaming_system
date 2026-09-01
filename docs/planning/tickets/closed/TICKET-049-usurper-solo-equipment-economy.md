---
title: TICKET-049-usurper-solo-equipment-economy
status: closed
ticket_number: 049
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-solo-equipment-economy.spec.md
---

# TICKET-049-usurper-solo-equipment-economy

## Summary

Extend the separate Usurper v0.20e Rust port from its first complete BBS day
into the source-linked solo equipment/economy loop: canonical starter weapons
and armor, bounded inventory/equipment, buying/selling and haggling, bank
deposit/withdrawal, private chest storage, combat effects, and trusted inert
cartridge screens.

## Why

Ticket 048 proves the real provider and trusted presentation path. The next
playable slice should make dungeon rewards meaningful while remaining entirely
inside one player's bounded provider session, before spells/class specials and
the separately gated shared-town topology.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When Milestone 2A data is compiled, the game shall expose only reviewed source-linked v0.20e weapon, armor, inventory, shop, bank, chest, and haggling rules with stable legacy ordinals and machine-readable provenance. | Canonical-source readback, table hashes/counts, ordinal fixtures, provenance trace validation, and rights scan. |
| REQ-002 | When a player receives, equips, swaps, sells, or stores an item, the reducer shall preserve bounded original slot/equipment behavior, reject invalid or full-capacity operations before mutation, and apply each accepted transition at one exact revision. | Table-driven inventory/equipment/chest tests, boundary and stale-command tests, deterministic state/view snapshots. |
| REQ-003 | When a player uses the weapon or armor shop, the reducer shall derive source-linked prices, enforce available gold and sale ownership, and resolve bounded haggling with the original attempt and decision order without duplicating money or items. | Buy/sell/haggle fixtures, insufficient-funds/full-inventory/kickout cases, no-RNG assertions, and replay tests. |
| REQ-004 | When a player deposits or withdraws bank gold or moves an item through the private chest, the reducer shall conserve value and ownership, enforce legacy scalar and capacity bounds, and keep robbery/shared-bank behavior unavailable. | Conservation/property tests, min/max/overflow fixtures, exact replay/revision tests, and privacy scan. |
| REQ-005 | When equipped weapon or armor affects a normal solo dungeon encounter, combat shall use the source-linked derived attack/defence behavior and preserve the previously established draw order, reward, death, and day semantics. | Differential equipped/unequipped combat fixtures, deterministic twins, draw-order assertions, and prior one-day regression suite. |
| REQ-006 | When the development cartridge opens the expanded loop, trusted QML shall render signed inventory, weapon shop, armor shop, bank, and chest screens plus the existing screens, with only bounded declared actions and fixed unavailable states. | Pack/conform, all-screen render-plan compilation, QML keyboard/state smoke, and visible ready-state preview. |
| REQ-007 | When this slice completes, Usurper shall remain a separate unadmitted provider/cartridge with no platform gameplay copy, migration, route, shared-realm claim, publisher executable presentation, historical art, or production publication. | Cross-repository dependency/content review, CodeGraph inspection, provider conformance/restart run, external checks, and full platform diff gate. |

## Scope

- In:
  - a reviewed bounded subset of canonical v0.20e weapon and armor catalogs;
  - player inventory, equipped weapon/armor, private chest, and safe swaps;
  - weapon/armor shops, buy/sell, deterministic haggling, and exact gold flow;
  - solo bank deposit/withdrawal only;
  - equipment-aware normal combat and reward loop;
  - signed inert inventory/shop/bank/chest cartridge screens and visible proof;
  - provider restart/conformance and complete regression validation.
- Out:
  - poison, spells, consumable item breadth, race/class special attacks, quests,
    finale, and immortality, which remain later Milestone 2 slices;
  - bank robbery, guards, town treasury, public market, NPCs, mail/social
    ownership, or any other shared-realm transition;
  - production registration, admission, deployment, publication, or SDK
    hosting/licensing decisions.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-solo-equipment-economy.spec.md](../../pipeline/completed/usurper-solo-equipment-economy.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
