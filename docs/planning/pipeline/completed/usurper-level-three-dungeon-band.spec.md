---
title: Usurper Level-Three Dungeon Band
pipeline_id: eca339a1-38b9-40e1-b844-2138df71ae1f
status: Phase 5 — Complete PASS
ticket: TICKET-055
ticket_doc: docs/planning/tickets/closed/TICKET-055-usurper-level-three-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-055-usurper-level-three-dungeon-band.md
created: 2026-08-31
---

# Usurper Level-Three Dungeon Band — spec

## Intent

Make the solo dungeon visibly deeper by translating its exact level-three
normal monster band and extending the existing deterministic provider and
trusted cartridge level path.

## Scope

- In:
  - source-linked level-three monster records and encounter draw order;
  - bounded level-one-through-three switching and level-consistent state;
  - existing combat subsystem composition, provider replay, signed QML view,
    provenance, compatibility docs, and regression coverage.
- Out:
  - dungeon level four or higher, random dungeon events, special areas,
    teams, PvP/NPCs, quests, shared realm state, and platform application code;
  - packaging, production registration, admission, deployment, or publication.

## Acceptance criteria (EARS)

The authoritative requirements and verification matrix are the six EARS
requirements in [TICKET-055](../../tickets/closed/TICKET-055-usurper-level-three-dungeon-band.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Preserve the original per-level rejection loop rather than replacing it with a direct nine-record choice. | Rejected candidates consume RNG and determine every later deterministic draw. |
| 2 | Retain level-three record 20 (`Medium Troll`) in canonical data while proving that normal selection returns only 21–29. | The source stores ten records but requires the selected ordinal to be greater than the lower band boundary. |
| 3 | Bound this development release to selectable levels one through three and reject every other value before RNG. | Only those normal-monster bands have been reviewed and translated; accepting higher levels would manufacture missing data. |
| 4 | Reuse `EnterDungeon { level }` from Main Street and Dungeon, with fixed cartridge actions for levels one through three. | It extends the already tested game-owned protocol while matching the original in-dungeon change-level behavior. |
| 5 | Advance the unadmitted game/rules/cartridge identity to v8 and keep provider/presentation protocols at v1. | Accepted state invariants and visible command outcomes expand, while the platform already treats game state and presentation as opaque bounded data. |

## Linked artifacts

- Ticket: [TICKET-055](../../tickets/closed/TICKET-055-usurper-level-three-dungeon-band.md)
- Prior milestone: [Ticket 054 notes](usurper-level-two-dungeon-band.notes.md)
- Port map: adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md`
- Architecture: [ADR-0002](../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../architecture/game-cartridges.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | canonical level/roster trace, state/action/view manifest, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | external data/rules/provider/cartridge expansion | focused checks and visible level-three fixture |
| 3.5 Inspect | cross-repository finding ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | external full checks and platform diff gate | evidence matches every requirement |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | authorized commit/publication only | explicitly deferred |
