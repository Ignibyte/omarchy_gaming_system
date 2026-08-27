---
title: Static marketplace publication, offline-root handoff, and mirror operations
pipeline_id: e02178df-dc45-4ddb-b2bd-43bc01a11e24
status: Phase 5 — Complete PASS
ticket: TICKET-037
ticket_doc: docs/planning/tickets/closed/TICKET-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations.md
aar: docs/planning/knowledge/aar/AAR-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations.md
created: 2026-08-27
---

# Static marketplace publication, offline-root handoff, and mirror operations — spec

## Intent

Convert the contracts delivered through Tickets 032–036 into a deterministic,
auditable, static-host publication workflow that keeps the root offline,
verifies every reviewed release and package byte, activates complete trees
atomically, proves mirrors identical, and rehearses rollback/key-compromise
response without claiming that this repository provisions a real production
host or stores a real production secret.

## Scope

- In:
  - canonical bounded publication plans, reviewed release/snapshot production,
    and deterministic static-tree layout;
  - explicit online catalog-signing versus offline root-signing commands and
    public request/response receipts;
  - exact local activation, verification, guarded hosted probes, mirror
    consistency, rotation/revocation drills, and operator documentation.
- Out:
  - real production infrastructure, domains, accounts, HSM/KMS/escrow, pager
    integrations, release staff, or production keys;
  - a public intake portal, automated review, arbitrary package repository,
    mirror fallback in clients, custom-cartridge trust, provider onboarding,
    modules/hooks, or federation.

## Acceptance criteria (EARS)

The binding requirements are the fifteen `REQ-*` rows in
[TICKET-037](../../tickets/open/TICKET-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations.md).

## Locked planning decisions

| # | Decision | Why |
|---|---|---|
| 1 | Deliver a static-host publication protocol and local operations drill, not a bespoke always-on marketplace application server. | Current consumers already require immutable HTTPS files; adding a mutable service would expand authority and operations without playable value. |
| 2 | Keep publisher, catalog, offline root, hosting, server admission, and client installation as distinct authorities/steps. | Compromise or convenience in one boundary must not silently manufacture the claims of another. |
| 3 | The online phase may use a catalog private key but can only prepare a public request for the root; only the offline phase reads the root private key and it performs no network work. | Root custody is meaningful only if routine hosted publication cannot wield it. |
| 4 | A final publication is a complete content-addressed/versioned tree selected by one atomic local pointer. | Clients and servers must never observe a half-written combination of trust, snapshot, releases, and packages. |
| 5 | A mirror is valid only when it serves the same authenticated publication identity; it never becomes a second trust root or client-selected fallback. | Availability replication must not weaken provenance or create ambiguous authority. |
| 6 | Production provisioning and a real root ceremony remain explicit external rollout work. | Local deterministic evidence cannot honestly prove custody vendors, human separation, domains, or live incident response. |

## Linked artifacts

- Ticket: [TICKET-037](../../tickets/closed/TICKET-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations.md)
- Prior pipeline: [Ticket 036 completed spec](../completed/public-marketplace-trust-enrollment-rotation-and-client-package-channel.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [system overview](../../../architecture/system-overview.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | autonomous approved-continuation scope review |
| 2 Design | Static layout, custody protocol, threat model, file/test manifest | actionable design plus CodeGraph receipt |
| 3 Implement | Contracts, CLI, drills, docs matching design | focused compilation/tests and self-review |
| 3.5 Inspect | Correctness/security/operations findings ledger | verified dispositions plus fresh CodeGraph receipt |
| 4 Validate | Focused drills and complete delivery gate | matching worktree gate receipt |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, commit/push | matching receipt and remote readback |
