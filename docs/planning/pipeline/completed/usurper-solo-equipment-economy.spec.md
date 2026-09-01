---
title: Usurper solo equipment economy
pipeline_id: a7b78a98-56f4-4cde-aa66-24d13035405a
status: Phase 5 — Complete PASS
ticket: TICKET-049
ticket_doc: docs/planning/tickets/closed/TICKET-049-usurper-solo-equipment-economy.md
aar: docs/planning/knowledge/aar/AAR-049-usurper-solo-equipment-economy.md
created: 2026-08-31
completed: 2026-08-31
---

# Usurper solo equipment economy — spec

## Intent

Make the first Usurper day economically meaningful by adding source-linked
solo equipment, shops, haggling, bank/chest storage, combat effects, and inert
trusted presentation without crossing into the shared-town architecture.

## Scope

- In:
  - canonical bounded weapon/armor data and provenance;
  - inventory/equipment/private chest transitions;
  - weapon/armor buy, sell, and deterministic haggling;
  - solo bank deposits and withdrawals;
  - equipped normal-combat effects;
  - provider/cartridge/QML proof and full regression evidence.
- Out:
  - poison, spell, special, quest, finale, immortality, public-market, robbery,
    guard, NPC, king, team, social, or other shared-realm behavior;
  - platform rule/schema/route ownership or production admission/publication.

## Acceptance criteria (EARS)

The authoritative requirements and verification matrix are the seven EARS
requirements in [TICKET-049](../../tickets/closed/TICKET-049-usurper-solo-equipment-economy.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Split mapped Milestone 2 into a bounded equipment-economy slice before poison, spells, specials, quests, and finale work. | It creates a complete reward/spend/equip loop while keeping source and regression review tractable. |
| 2 | Implement only player-private bank and chest behavior; defer robbery, guards, treasury, market, and other cross-player effects. | Those features require the reviewed shared-realm seam reserved for Milestone 3. |
| 3 | Retain the session-bounded public Provider starter for this slice. | The state remains one-player/private and must still pass the existing 32 KiB and conformance boundaries. |
| 4 | Add only inert signed screen data and platform-owned trusted QML rendering. | It preserves the established cartridge authority boundary. |
| 5 | Use the non-classic v0.20e object/inventory path and limit armor to reviewed `Body` records. | It keeps shops, pack, equipment, and chest behavior on one real source branch instead of composing incompatible classic and non-classic modes. |
| 6 | Preserve source ordinals, including gaps, rather than renumbering the reviewed catalog subset. | Commands, fixtures, and provenance can continue to name the canonical record identity as later rows are added. |

## Linked artifacts

- Ticket: [TICKET-049](../../tickets/closed/TICKET-049-usurper-solo-equipment-economy.md)
- Prior milestone: [Ticket 048 notes](../completed/usurper-one-bbs-day-and-trusted-preview.notes.md)
- Port map: adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | canonical rule trace, state/command/view manifest, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | external model/data/rules/provider/cartridge expansion | focused checks and visible preview |
| 3.5 Inspect | cross-repository finding ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | external full checks and platform fast gate | evidence matches every requirement |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | full diff gate and authorized publication | matching receipt and explicit authorization |
