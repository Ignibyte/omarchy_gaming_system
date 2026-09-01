---
title: TICKET-047-usurper-v020e-provenance-and-rust-port-map
status: closed
ticket_number: 047
type: spike
created: 2026-08-30
closed: 2026-08-30
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-v020e-provenance-and-rust-port-map.spec.md
---

# TICKET-047-usurper-v020e-provenance-and-rust-port-map

## Summary

Acquire and authenticate the original Usurper v0.20e release and source in a
separate local game workspace, then produce a build-ready map from the Pascal
rules, persistence, data, maintenance, and presentation boundaries to a Rust
OmarchyGS provider and inert cartridge.

## Why

Usurper is the selected first historical BBS game. Its original source is
GPL-2.0-or-later, but the platform charter requires source and asset provenance
to be verified before a port. A bounded mapping slice prevents a 200,000-line
Pascal codebase, bundled third-party units, DOS I/O, and historical assets from
being copied wholesale without an exact compatibility and licensing plan.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the upstream baseline is acquired, the project shall identify the original unmodified source commit and original v0.20e release archive by canonical origin, exact SHA-256 digest, byte size, and immutable local path outside the platform repository. | Independent hash/readback commands, upstream commit inspection, and repository-status review. |
| REQ-002 | When the upstream corpus is inventoried, the build map shall classify source, executable, data, text, ANSI/ASCII, documentation, bundled library, and generated artifacts and shall record the controlling license notice or an explicit unresolved-provenance marker for each class. | File/type/license inventory and sampled file-level review. |
| REQ-003 | When the original Pascal implementation is mapped, the build map shall identify gameplay domains, authoritative state, persistence records, random and time behavior, daily maintenance, terminal presentation, BBS integration, and cross-domain call order needed for a faithful Rust port. | Source topology, direct inspection, and cross-reference tables. |
| REQ-004 | When the Rust delivery map is complete, it shall define a deterministic rules core, independent provider database/runtime, Provider SDK seam, inert cartridge screens, compatibility ledger, and a first playable one-day vertical slice with requirement-to-evidence coverage. | Architecture review against ADR-0002/0003, provider starter contracts, and a milestone regression matrix. |
| REQ-005 | When this mapping ticket completes, no upstream game bytes, Rust game implementation, provider admission, platform migration, public route, marketplace publication, or production registration shall have been added to the OmarchyGS platform repository. | Git diff/status inspection, route/migration inventory, and canonical documentation gate. |

## Scope

- In:
  - original v0.20e binary/source acquisition and cryptographic provenance;
  - separate adjacent game-workspace layout;
  - source, data, asset, bundled-code, and license inventory;
  - Pascal domain/state/flow map and compatibility policy;
  - Rust provider/cartridge architecture and incremental milestone map;
  - one complete BBS-day first-playable definition.
- Out:
  - a Rust game implementation or database schema;
  - committing upstream archives or source into the platform repository;
  - claiming unresolved third-party code or art is cleared for redistribution;
  - public provider admission, marketplace publication, hosted deployment, or
    production signing keys;
  - preserving crashes, unsafe file access, data corruption, or security bugs
    as compatibility behavior.

## Links

- Intake: none; user selected Usurper directly after product exploration
- Pipeline spec: [usurper-v020e-provenance-and-rust-port-map.spec.md](../../pipeline/completed/usurper-v020e-provenance-and-rust-port-map.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
