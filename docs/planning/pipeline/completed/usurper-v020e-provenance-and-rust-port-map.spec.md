---
title: Usurper v0.20e provenance and Rust port map
pipeline_id: 376a6f08-d054-47ec-ac17-70ad4fa36dd7
status: Phase 5 — Complete PASS
ticket: TICKET-047
ticket_doc: docs/planning/tickets/closed/TICKET-047-usurper-v020e-provenance-and-rust-port-map.md
aar: docs/planning/knowledge/aar/AAR-047-usurper-v020e-provenance-and-rust-port-map.md
created: 2026-08-30
completed: 2026-08-30
---

# Usurper v0.20e provenance and Rust port map — spec

## Intent

Establish one authenticated, legally reviewable Usurper baseline and turn its
historical Pascal implementation into an actionable, deterministic Rust
provider/cartridge build map before any game implementation begins.

## Scope

- In:
  - original unmodified source commit and v0.20e release archive;
  - separate local game workspace and immutable provenance manifest;
  - source, data, asset, bundled-code, and license classification;
  - gameplay/state/persistence/RNG/time/maintenance/presentation flow map;
  - Rust rules/provider/cartridge boundaries and phased implementation plan.
- Out:
  - Rust application code, SQL migrations, or production package bytes;
  - upstream corpus in the OmarchyGS platform repository;
  - public provider registration/admission, hosting, or marketplace release;
  - a blanket redistribution conclusion for unverified third-party material.

## Acceptance criteria (EARS)

The binding acceptance criteria are the five requirements in
[`TICKET-047`](../../tickets/closed/TICKET-047-usurper-v020e-provenance-and-rust-port-map.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Pin source commit `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, labeled upstream as “Original, unmodified source,” and the original `usurp020e.zip` release as independent evidence. | The commit is the root source snapshot linked by the Usurper archive for v0.20e, while the release archive preserves the actually distributed game/data shape. |
| 2 | Keep acquisition and future game work in `/home/cpeppers/Projects/omarchygs_usurper`; commit only provenance and mapping documentation to this platform repository. | ADR-0002 requires BBS game sources to live in separate repositories and prevents GPL/reference bytes from coupling platform release history. |
| 3 | Treat v0.20e, the last release by Jakob Dangarden, as the canonical behavior baseline; use later FreePascal ports and fixes only as annotated secondary evidence. | This best matches the requested original logic while making every later correction explicit. |
| 4 | Preserve formulas, integer semantics, RNG ordering, data tables, state transitions, and maintenance order; do not preserve unsafe I/O, crashes, corruption, credential exposure, or undefined behavior. | Fidelity is observable game behavior, not historical vulnerability reproduction. |

## Linked artifacts

- Ticket: [TICKET-047](../../tickets/closed/TICKET-047-usurper-v020e-provenance-and-rust-port-map.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: none; selected directly from the preceding Usurper/LORD exploration

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, exact baseline decision | autonomous scope lock and tool readiness |
| 2 Design | Authenticated corpus inventory, license matrix, Pascal topology, Rust architecture, exact documentation manifest, regression plan | worktree-bound CodeGraph receipt plus direct upstream review |
| 3 Implement | Provenance manifest and complete build-map documentation only | deterministic inventory checks and self-review |
| 3.5 Inspect | Correctness, provenance, licensing, completeness, architecture, and scope ledger | fixes plus fresh CodeGraph evidence |
| 4 Validate | Reproducible acquisition/inventory checks and local diff gate | matching worktree receipt |
| 5 Complete | AC audit, OpenWiki, submitted AAR, ticket/archive closure | no silent drops or unresolved provenance claims |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt; no push without authorization |
