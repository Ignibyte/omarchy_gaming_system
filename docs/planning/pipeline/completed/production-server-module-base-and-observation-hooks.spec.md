---
title: Production server-module base and observation hooks
pipeline_id: 49248bf8-87d9-4cfe-886c-492133c4a89c
status: Phase 5 — Complete PASS
ticket: TICKET-040
ticket_doc: docs/planning/tickets/closed/TICKET-040-production-server-module-base-and-observation-hooks.md
aar: docs/planning/knowledge/aar/AAR-040-production-server-module-base-and-observation-hooks.md
created: 2026-08-27
---

# Production server-module base and observation hooks — spec

## Intent

Turn ADR-0004's isolated architecture proof into the smallest production
server-module vertical slice: an exact reviewed release and admission, one
post-commit observation event, one bounded typed intent reauthorized by core,
durable dispatch and receipts, isolated host execution, namespaced state,
lifecycle, recovery, and conformance. Existing deployments remain unchanged
unless an exact first-party fixture is explicitly configured.

## Scope

- In:
  - the eight EARS requirements in Ticket 040;
  - production registry, PostgreSQL persistence, observation outbox,
    dispatcher, host/service boundary, typed hook and intent, state/lifecycle,
    recovery, telemetry, and deterministic conformance;
  - one exact reviewed first-party fixture used only for end-to-end evidence.
- Out:
  - administrator-uploaded or operator-custom package installation;
  - admission or fail-closed hooks, arbitrary egress, SQL/native/WASI
    hostcalls, general provider/game authority, marketplace module review,
    client code, or raw server-supplied QML/JavaScript.

## Acceptance criteria (EARS)

The binding acceptance criteria are REQ-001 through REQ-008 in
[`TICKET-040`](../../tickets/closed/TICKET-040-production-server-module-base-and-observation-hooks.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | The production artifact and runtime follow ADR-0004: one exact no-WASI component release per dedicated OS-contained native host process. | Ticket 039 selected and hostile-tested this defense-in-depth boundary. |
| 2 | This slice exposes only one post-commit observation hook and one fixed typed intent; no synchronous admission hook is authorized. | External execution must not hold or decide the original domain transaction. |
| 3 | Core owns every release/admission/state/outbox/receipt/lifecycle row and reauthorizes every proposed effect under current state. | A module signature, capability request, or host response is not authority to mutate platform state. |
| 4 | Only a checked-in, exact reviewed first-party fixture may be configured in this slice. | Operator-custom installation, provenance acknowledgement, and package custody remain Ticket 041. |
| 5 | Disabled-by-default startup must preserve all current route, process, database-domain, client, cartridge, compiled-game, and provider behavior. | A new extension seam cannot silently change existing owner-operated servers. |
| 6 | Observation delivery is durable, bounded, partition-ordered, at least once, and never executes while the originating domain transaction is open. | Failures must not roll back committed platform work or create unbounded memory/lock pressure. |

## Linked artifacts

- Ticket: [TICKET-040](../../tickets/closed/TICKET-040-production-server-module-base-and-observation-hooks.md)
- Architecture: [ADR-0004](../../../architecture/adr-0004-process-isolated-wasm-server-modules.md), [Server modules](../../../architecture/server-modules.md)
- Predecessor: [Ticket 039 completed pipeline](server-extension-isolation-and-typed-hook-architecture-spike.spec.md)
- Intake: none; Ticket 040 was sequenced and approved by Ticket 039.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Bound ticket, spec/notes, and open AAR | scope and authority boundaries settled |
| 2 Design | Production data flow, schema/API contracts, file manifest, regression map, and CodeGraph evidence | actionable design plus worktree-bound receipt |
| 3 Implement | Registry, outbox/dispatcher, isolated host base, one hook/intent, state/lifecycle/recovery, and conformance | focused Rust/PostgreSQL/process tests pass |
| 3.5 Inspect | Correctness, security, privacy, concurrency, recovery, operations, and simplification ledger | findings resolved plus fresh CodeGraph receipt |
| 4 Validate | Focused suites and complete local delivery gate | matching worktree gate receipt |
| 5 Complete | AC audit, OpenWiki, hand docs, submitted AAR/knowledge, ticket close, and archive | no silent drops |
| Delivery | Fresh receipt, staged review, authorized commit/push/readback | remote `main` matches the delivered commit |
