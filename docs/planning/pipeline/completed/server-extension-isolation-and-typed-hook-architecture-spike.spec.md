---
title: Server extension isolation and typed-hook architecture spike
pipeline_id: 144295f2-9300-4fcc-96e0-2e25d910f99e
status: Phase 5 — Complete PASS
ticket: TICKET-039
ticket_doc: docs/planning/tickets/closed/TICKET-039-server-extension-isolation-and-typed-hook-architecture-spike.md
aar: docs/planning/knowledge/aar/AAR-039-server-extension-isolation-and-typed-hook-architecture-spike.md
created: 2026-08-27
---

# Server extension isolation and typed-hook architecture spike — spec

## Intent

Choose and exercise the containment boundary for future general server modules
before any executable extension loader is authorized. The result must make
typed hooks useful to owner-operated communities while keeping core domain
authorization, protected state, credentials, client safety, and game-provider
authority outside module control.

## Scope

- In:
  - the eleven EARS requirements in Ticket 039;
  - runtime/isolation research and a reproducible comparison;
  - exact contracts and an isolated hostile proof for the selected model;
  - ADR, conformance direction, operational consequences, and follow-up work.
- Out:
  - production module installation, loading, persistence, routes, or discovery;
  - external-provider authorization, Provider SDK publication, federation, or
    executable cartridge/client content.

## Acceptance criteria (EARS)

The binding acceptance criteria are REQ-001 through REQ-011 in
[`TICKET-039`](../../tickets/closed/TICKET-039-server-extension-isolation-and-typed-hook-architecture-spike.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | General server modules, game providers, inert cartridges, and compiled core code remain separate extension families. | A convenient hook must not bypass the provider protocol or turn frontend data into executable authority. |
| 2 | This ticket may build only an isolated worktree-bound proof; production server startup, routes, database schema, and operator module installation remain unchanged. | The spike must earn an executable architecture decision before activation. |
| 3 | Every module effect is a typed intent re-authorized and committed by a core domain service; modules receive neither database handles nor raw mutable domain objects. | Capability declaration and process isolation are not substitutes for core authorization. |
| 4 | Dynamic Rust ABI loading and arbitrary in-process third-party native code are not candidates for authorization. | ABI instability and server-wide compromise conflict with ADR-0003. |
| 5 | The comparison must include operator usability, lifecycle, recovery, and observability as well as sandbox strength and benchmark results. | A secure runtime that cannot be upgraded, diagnosed, or restored safely is not an operable module system. |
| 6 | Module provenance, code integrity, marketplace review, operator trust, capability grants, and runtime containment remain independent claims. | A valid signature or reviewed package does not itself authorize effects or guarantee safety. |
| 7 | Repository quality checks and delivery evidence run locally; hosted CI/CD definitions are prohibited. | The owner explicitly chose local enforcement, and the worktree-bound gate already provides the canonical proof. |

## Linked artifacts

- Ticket: [TICKET-039](../../tickets/closed/TICKET-039-server-extension-isolation-and-typed-hook-architecture-spike.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Prior pipelines: [Ticket 014](../completed/portable-games-sdk-and-remote-hosting-spike.spec.md), [Ticket 018](../completed/production-remote-provider-security-foundation.spec.md), [Ticket 027](../completed/owner-operated-servers-cartridge-distribution-and-extension-roadmap.spec.md), [Ticket 038](../completed/operator-custom-cartridge-trust-import-and-player-warnings.spec.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, bounded EARS scope, spec/notes, and open AAR | scope and authority boundaries settled |
| 2 Design | Model research/matrix, threat model, exact contracts, proof manifest, and requirement-to-evidence map | actionable design plus CodeGraph receipt |
| 3 Implement | Isolated selected-model proof, hostile fixtures, ADR, and documentation | focused proof passes without production activation |
| 3.5 Inspect | Correctness, security, containment, lifecycle, operations, and simplification ledger | findings resolved plus fresh CodeGraph receipt |
| 4 Validate | Focused conformance/security suites and complete delivery gate | matching worktree gate receipt |
| 5 Complete | AC audit, OpenWiki, AAR/knowledge, follow-up sequence, and archive | no silent drops |
| Delivery | Fresh receipt, staged review, commit/push/readback | matching receipt and remote SHA |
