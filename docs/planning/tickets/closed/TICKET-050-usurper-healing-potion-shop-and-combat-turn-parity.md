---
title: TICKET-050-usurper-healing-potion-shop-and-combat-turn-parity
status: closed
ticket_number: 050
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-healing-potion-shop-and-combat-turn-parity.spec.md
---

# TICKET-050-usurper-healing-potion-shop-and-combat-turn-parity

## Summary

Extend the separate Usurper v0.20e Rust port with the source-linked Magic Shop
healing-potion purchase flow and correct configured combat quaff behavior:
heal first, then immediately perform the player's normal attack in the same
accepted turn.

## Why

Ticket 049 made dungeon gold useful but left healing potions at their launch
balance and treated combat healing as a free standalone turn. The next bounded
slice closes that fidelity gap without importing the Magic Shop's unrelated
spell/item catalogs or the disconnected Alchemist poison subsystem.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Magic Shop, quick-healing, combat-quaff, launch-balance, and configured limit sources, including the deliberate distinction between the initial 150 potions and the shop's configured maximum of 75. | Canonical-source readback, source-trace validation, fixed constants and boundary fixtures, and documentation review. |
| REQ-002 | When a player buys healing potions in the Magic Shop, the reducer shall derive the unit price as five times player level, require a positive bounded quantity, sufficient gold, and room under the configured shop maximum, and shall conserve gold and potion count with checked arithmetic. | Price/quantity/funds/cap/overflow table tests, conservation assertions, deterministic state/view snapshots, and provider adapter tests. |
| REQ-003 | When a wounded player with potions chooses quick healing during combat under the declared `QuaffOpt = 1` development mode, the reducer shall consume exactly the required available potions at five hit points each, cap health at maximum, and then perform the existing normal player attack in that same transition without adding an RNG draw before the attack. | Exact heal-and-attack fixtures, partial-supply and lethal-hit cases, RNG-index assertions, deterministic twins, and prior combat regression tests. |
| REQ-004 | When quick healing is used outside combat, at full health, without potions, or with malformed purchase input, the reducer shall preserve the declared legacy phase behavior, reject or accept no-effect outcomes consistently, and never partially mutate gold, potions, health, encounter state, or RNG position. | Dungeon/full-health/empty-supply/invalid-input tests, state-before/after comparisons, replay/revision tests, and strict JSON fixtures. |
| REQ-005 | When the development cartridge opens the new Magic Shop and combat healing paths, trusted QML shall render a signed inert seventeenth screen and bounded declared actions while the provider shall preserve deterministic command, replay, restart, and presentation behavior. | Deterministic pack/conform, all-screen render-plan compilation, QML keyboard/state smoke, provider conformance twice across restart, and visible ready-state preview. |
| REQ-006 | When this slice completes, Usurper shall remain a separate unadmitted provider/cartridge with no spell catalog, magic-item catalog, poison/Alchemist behavior, shared-realm state, platform gameplay copy, schema/route change, publisher executable presentation, or production publication. | Cross-repository dependency/content review, CodeGraph inspection, scope scan, external checks, and full platform diff gate. |

## Scope

- In:
  - canonical healing-potion price, configured purchase cap, and launch-balance
    provenance;
  - a bounded Magic Shop phase, purchase command, state/view, and inert screen;
  - faithful quick-heal calculation in dungeon and combat;
  - configured combat quaff option 1: heal then normal attack in one transition;
  - strict provider action decoding, restart/replay proof, and visible preview.
- Out:
  - spells, Magic Shop item catalogs, poison, Alchemist access, special attacks,
    quests, finale, immortality, and shared-town behavior;
  - production registration, admission, deployment, publication, or new SDK
    surface.

## Links

- Intake: none; continuation of the user-authorized Usurper build
- Pipeline spec: [usurper-healing-potion-shop-and-combat-turn-parity.spec.md](../../pipeline/completed/usurper-healing-potion-shop-and-combat-turn-parity.spec.md)
- Prior milestone: [Ticket 049](../closed/TICKET-049-usurper-solo-equipment-economy.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
