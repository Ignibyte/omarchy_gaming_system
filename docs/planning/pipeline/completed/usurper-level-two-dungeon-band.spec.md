---
title: Usurper Level-Two Dungeon Band
pipeline_id: 9dc365e9-59e0-43fd-8d35-b64ba987a528
status: Phase 5 — Complete PASS
ticket: TICKET-054
ticket_doc: docs/planning/tickets/closed/TICKET-054-usurper-level-two-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-054-usurper-level-two-dungeon-band.md
created: 2026-08-31
---

# Usurper Level-Two Dungeon Band — spec

## Intent

Make the solo dungeon visibly deeper by translating its exact level-two normal
monster band and change-level path through the deterministic provider and
trusted cartridge.

## Scope

- In:
  - source-linked level-two monster records and encounter draw order;
  - bounded level-one/level-two switching and level-consistent state;
  - existing combat subsystem composition, provider replay, signed QML view,
    provenance, compatibility docs, and regression coverage.
- Out:
  - dungeon level three or higher, random dungeon events, special areas,
    teams, PvP/NPCs, quests, shared realm state, and platform application code;
  - packaging, production registration, admission, deployment, or publication.

## Acceptance criteria (EARS)

The authoritative requirements and verification matrix are the six EARS
requirements in [TICKET-054](../../tickets/closed/TICKET-054-usurper-level-two-dungeon-band.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Preserve the original per-level rejection loop rather than replacing it with a direct nine-record choice. | Rejected candidates consume RNG and determine every later deterministic draw. |
| 2 | Retain level-two record 10 (`Small Troll`) in canonical data while proving that normal selection returns only 11–19. | The source stores ten records but requires the selected ordinal to be greater than the lower band boundary. |
| 3 | Bound this development release to selectable levels one and two and reject every other value before RNG. | Only those normal-monster bands have been reviewed and translated; accepting higher levels would manufacture missing data. |
| 4 | Reuse `EnterDungeon { level }` from Main Street and Dungeon, with fixed cartridge actions for levels one and two. | It keeps the game-owned protocol bounded while matching the original in-dungeon change-level behavior. |
| 5 | Advance the unadmitted game/rules/cartridge identity to v7 and keep provider/presentation protocols at v1. | Accepted state invariants and visible command outcomes expand, while the platform already treats game state and presentation as opaque bounded data. |

## Linked artifacts

- Ticket: [TICKET-054](../../tickets/closed/TICKET-054-usurper-level-two-dungeon-band.md)
- Prior milestone: [Ticket 053 notes](../completed/usurper-gnoll-poisonous-bite.notes.md)
- Port map: adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md`
- Architecture: [ADR-0002](../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../architecture/game-cartridges.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | canonical level/roster trace, state/action/view manifest, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | external data/rules/provider/cartridge expansion | focused checks and visible level-two fixture |
| 3.5 Inspect | cross-repository finding ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | external full checks and platform diff gate | evidence matches every requirement |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | authorized commit/publication only | explicitly deferred |
