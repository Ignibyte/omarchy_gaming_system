---
title: TICKET-039-server-extension-isolation-and-typed-hook-architecture-spike
status: closed
ticket_number: 039
type: spike
created: 2026-08-27
closed: 2026-08-27
intake:
pipeline_spec: docs/planning/pipeline/completed/server-extension-isolation-and-typed-hook-architecture-spike.spec.md
---

# TICKET-039-server-extension-isolation-and-typed-hook-architecture-spike

## Summary

Select and prove the executable isolation model for future OmarchyGS server
modules, and define the versioned capability, typed-hook, state, lifecycle,
audit, compatibility, and recovery contracts that a production module base
must implement.

## Why

Owner-operated communities eventually need extensions beyond game backends,
but enabling executable modules before choosing a containment and upgrade model
would create an unbounded platform-authority bypass. Ticket 038 completed the
safe inert custom-cartridge path; this spike is the roadmap gate required before
administrator-installed server code or a general hook runtime can be built.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the spike inventories current extension seams, it shall distinguish inert Game Cartridges, registered game providers, compiled platform code, and general server modules, and shall identify every protected core authority a module may not bypass. | CodeGraph-backed current-flow map, direct source review, and architecture authority matrix. |
| REQ-002 | When the spike compares executable isolation models, it shall evaluate external-process RPC, Wasm, statically compiled modules, and any justified hybrid against containment, interface stability, resource control, async I/O, state, upgrade/rollback, observability, language portability, packaging, and operator burden using primary-source and measured local evidence. | Reproducible decision matrix, primary-source citations, local environment/proof measurements, and rejected-alternative rationale. |
| REQ-003 | When a module is described or loaded by the proof, the contract shall require one canonical versioned manifest binding immutable module identity, release/provenance, protocol compatibility, requested capabilities, subscribed hooks, resource budgets, configuration schema, state schema, and executable digest. | Canonical serialization/validation tests plus malformed, duplicate, unknown, downgrade, digest, and compatibility hostile cases. |
| REQ-004 | When core emits a hook or receives a module response, the contract shall use bounded typed events and typed intents under an explicit least-privilege grant; it shall expose no raw credential, account ownership, unrestricted database handle, arbitrary network destination, client executable bridge, or direct protected-state mutation. | Contract tests, capability-denial corpus, sensitive-field absence checks, and architecture review. |
| REQ-005 | When hooks execute concurrently, slowly, repeatedly, or fail, the contract shall define deterministic ordering, deadlines, bounded queues and payloads, idempotency/replay identity, retry policy, backpressure, crash containment, fail-open versus fail-closed classification, and a core-owned authorization/commit point for every accepted intent. | Selected-model proof covering normal, duplicate, reordered, timeout, crash, saturation, malformed, and unauthorized behavior. |
| REQ-006 | When modules use configuration or durable state, the architecture shall assign isolated namespaces, bounded quotas, optimistic or serializable revision rules, explicit schema migrations, backup/restore behavior, and removal/rollback semantics without granting access to OmarchyGS tables or credentials. | State-machine design, migration/rollback fixtures, namespace escape tests, and recovery rehearsal in the isolated proof. |
| REQ-007 | When an administrator installs, enables, disables, upgrades, rolls back, or removes a future module, the architecture shall require exact provenance and capability review, idempotent expected-state operations, immutable audit, health/readiness gates, compatible state transitions, and player-visible custom-server provenance where behavior affects players. | Lifecycle sequence, exact receipt/audit fixtures, concurrency and recovery cases, and operator threat review. |
| REQ-008 | When the selected isolation model is exercised, a worktree-bound isolated proof shall run core and module as their intended trust units, accept one allowlisted hook-to-intent flow, and reject undeclared capabilities, forged identity, tampered bytes, oversized input/output, forbidden I/O, hangs, crashes, and stale replay without changing production server authority. | Focused proof script, separate-process/runtime evidence, hostile conformance suite, process/resource observations, and production-route absence check. |
| REQ-009 | When module authors target the future base, the architecture shall define compatibility negotiation and deterministic conformance fixtures for both marketplace-vetted and operator-custom module provenance without equating signature, review, operator trust, capability grant, or runtime containment. | Version/provenance matrices, conformance fixture output, and trust-claim consistency review. |
| REQ-010 | When the spike completes, it shall record the selected model and residual risks in an ADR, update product/architecture/operator roadmap surfaces, sequence implementation tickets, pass focused/security/CodeGraph/OpenWiki evidence and the canonical diff gate, and leave production executable module loading disabled. | ADR and follow-up ticket audit, source/route/config absence checks, OpenWiki lifecycle, and `bin/gate.sh --diff`. |
| REQ-011 | When repository quality enforcement is configured, the system shall run build, test, documentation, packaging, security, and delivery checks locally, contain no hosted CI/CD workflow definition, and reject reintroduction of GitHub Actions or equivalent hosted automation. | GitHub workflow state readback, local-only automation checker hostile fixture, residual file/reference audit, and canonical local gate. |

## Scope

- In:
  - current-seam and authority inventory;
  - isolation/runtime comparison with primary-source and measured evidence;
  - canonical manifest, capability, typed hook/intent, compatibility, state,
    lifecycle, audit, failure, and recovery contracts;
  - an isolated non-production proof and hostile conformance fixtures;
  - ADR, operator/architecture documentation, and sequenced follow-up tickets.
  - user-directed removal of hosted CI/CD and durable local-only enforcement.
- Out:
  - production server module loading or administrator installation;
  - a network administration API, a production database migration, or module
    access to platform tables, credentials, arbitrary egress, or client code;
  - using general hooks as a game-backend alternative to the Provider SDK;
  - public marketplace review operations, third-party module approval, or
    claims that OS/runtime isolation makes untrusted code harmless.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/server-extension-isolation-and-typed-hook-architecture-spike.spec.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
