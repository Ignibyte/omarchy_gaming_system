---
title: TICKET-048-usurper-one-bbs-day-and-trusted-preview
status: closed
ticket_number: 048
type: feature
created: 2026-08-31
closed: 2026-08-31
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-one-bbs-day-and-trusted-preview.spec.md
---

# TICKET-048-usurper-one-bbs-day-and-trusted-preview

## Summary

Build the first executable Rust slice of Usurper v0.20e in its separate game
repository: one deterministic character day implemented behind the public
provider seam and shown through a signed inert cartridge in the trusted QML
preview surface.

## Why

Ticket 047 authenticated and mapped the original release. The next useful proof
must connect source-linked rules, durable provider behavior, declarative game
screens, and the real trusted renderer so the port is visibly playable instead
of remaining documentation or disconnected scaffolding.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the separate Usurper repository is prepared for implementation, it shall build as a GPL-2.0-or-later Rust workspace, keep authenticated upstream bytes ignored and immutable, and trace every translated rule and seed value in this slice to the canonical v0.20e source. | Repository inventory, license/provenance scan, source-trace ledger, formatting, Clippy, tests, and upstream hash/readback checks. |
| REQ-002 | When a new deterministic Usurper session launches, the game shall create the explicitly labeled fixed-alias development character, accept one of the original race and class choices, apply source-linked initial scalar behavior, and enter Main Street with a bounded revisioned view. | Table-driven creation fixtures, invalid-command tests, deterministic twin-run comparison, and view-schema validation. |
| REQ-003 | When the character enters the dungeon, the game shall select an allowed level and resolve a source-linked normal encounter with attack and retreat actions, HP/death handling, XP/gold reward, fight consumption, and an indexed bound/result trace for every random draw. | Reducer unit tests covering victory, retreat, death, stale/invalid input, draw ordering, and scalar boundaries. |
| REQ-004 | When the character visits the healer or level master and then sleeps, the game shall apply eligible healing or advancement, end the visit, advance one realm day exactly once, reset the first-slice daily counters, expose bounded mail/news facts, and allow re-entry. | End-to-end one-day fixture, replay/revision tests, maintenance idempotency tests, and byte-identical state/view/event/RNG traces across clean runs. |
| REQ-005 | When Usurper is exercised as an OmarchyGS provider, it shall implement only the public `ProviderGame` rules seam while the starter owns transport, signed protocol, operation receipts, provider PostgreSQL persistence, callbacks, and lifecycle; serialized state and views shall stay within the starter and cartridge bounds and contain no platform identity or credential. | Packaged-SDK build, provider conformance/fault run, PostgreSQL restart/replay test, bounded serialization checks, and privacy scans. |
| REQ-006 | When a development player opens the Usurper cartridge, the trusted renderer shall show signed multi-screen entry, race, class, Main Street, status, dungeon, combat, healer, level-master, mail/news, and sleep presentation using only declared bounded nodes/actions and plain replacement text. | Cartridge pack/conformance, render-plan compilation for every screen/state, QML loading/offline/empty/error and keyboard smoke, plus a captured visible ready-state preview. |
| REQ-007 | When this slice completes, Usurper shall remain a separate process/repository and game-state authority with no platform rule copy, platform database migration, raw publisher QML/JavaScript, historical ANSI asset, external provider admission, marketplace publication, or compiled fallback. | Cross-repository diff and dependency scan, platform route/migration review, cartridge inventory, full local gate, and authority review. |

## Scope

- In:
  - buildable separate Rust workspace and source-trace/compatibility ledger;
  - deterministic one-player state, commands, injected clock/RNG, and first-day
    reducers;
  - public Provider SDK/starter adapter and development conformance evidence;
  - signed Core cartridge covering the mapped first-day screens;
  - real trusted QML preview and visible smoke evidence;
  - focused external-repository checks plus the complete platform diff gate.
- Out:
  - complete solo economy, shops, magic, quests, finale, or immortality;
  - shared king/market/NPC/PvP/family state or a generalized realm SDK seam;
  - arbitrary alias input before a trusted platform input capability exists;
  - public provider onboarding, production registration, hosted deployment,
    marketplace publication, or packaging-channel work;
  - historical binaries, DDPlus/Borland/SWAG code, or uncleared ANSI art.

## Links

- Intake: none; the user authorized implementation after Ticket 047
- Pipeline spec: [usurper-one-bbs-day-and-trusted-preview.spec.md](../../pipeline/completed/usurper-one-bbs-day-and-trusted-preview.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
