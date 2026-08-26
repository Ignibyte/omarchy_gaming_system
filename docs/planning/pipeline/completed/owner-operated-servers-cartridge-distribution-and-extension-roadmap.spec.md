---
title: Owner-operated servers, cartridge distribution, and extension roadmap
pipeline_id: c2474cc0-716a-4db5-8223-1f67cea48059
status: Phase 5 — Complete PASS
ticket: TICKET-027
ticket_doc: docs/planning/tickets/closed/TICKET-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md
aar: docs/planning/knowledge/aar/AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md
created: 2026-08-26
---

# Owner-operated servers, cartridge distribution, and extension roadmap — spec

## Intent

Capture the approved direction in which individuals operate standard OmarchyGS
community servers, curate games for their players, and may deliberately step
outside the vetted marketplace on their own server without granting custom
content executable authority inside the official client.

## Scope

- In: all six Ticket 027 requirements and the documentation/architecture
  surfaces needed to make the direction durable and internally consistent.
- Out: application code, API/schema implementation, federation, marketplace
  operations, legal drafting, plugin/runtime implementation, and Git delivery.

## Acceptance criteria (EARS)

The authoritative acceptance criteria are REQ-001 through REQ-006 in
[`TICKET-027`](../../tickets/closed/TICKET-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Owner-operated servers are independent communities running the standard OmarchyGS architecture; federation is not implied. | This supports friends-and-family hosting without merging unrelated identity or trust domains. |
| 2 | The marketplace distributes vetted, signed, inert frontend cartridges; game backend code is not embedded in the cartridge. | Presentation portability and gameplay authority are separate security and release concerns. |
| 3 | The official client always renders a bounded declarative cartridge through trusted QML, including for operator-sideloaded games. | Choosing an unvetted server must not grant that server raw client-code execution. |
| 4 | A future provider SDK makes independently hosted game backends conform to the brokered server protocol; the selected OmarchyGS server remains the platform authority and network broker. | The core stays game-agnostic while credentials, policy, and audit remain centralized. |
| 5 | Server operators may install custom server extensions outside the marketplace, but those extensions are server-local operator trust and must be visibly distinguishable from vetted releases. | Administrators control their own machines, while players need honest provenance and unchanged client safety. |
| 6 | The module/hook mechanism is a future separately threat-modeled system; this slice specifies outcomes but does not choose an in-process ABI, Wasm runtime, or external-process protocol. | The isolation and compatibility choice requires evidence beyond a roadmap update. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md`
- Architecture: `docs/architecture/game-cartridges.md`, `docs/architecture/system-overview.md`, `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md`
- Product: `docs/product-charter.md`, `docs/planning/ROADMAP.md`
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, EARS scope, active spec/notes, open AAR | bounded documentation-only slice |
| 2 Design | ownership/trust model, exact manifest, review matrix, CodeGraph receipt | consistent architecture direction |
| 3 Implement | product, ADR, architecture, and roadmap updates | focused link/structure checks |
| 3.5 Inspect | authority, client-safety, operator-risk, compatibility, and terminology ledger | resolved contradictions and fresh CodeGraph receipt |
| 4 Validate | focused checks and canonical diff gate | matching delivery receipt |
| 5 Complete | AC audit, OpenWiki, submitted AAR, ticket/archive | matching completion receipt |
| Delivery | staged review and separately authorized commit/push | explicit delivery authorization |
