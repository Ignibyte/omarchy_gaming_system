---
title: TICKET-040-production-server-module-base-and-observation-hooks
status: closed
ticket_number: 040
type: feature
created: 2026-08-27
closed: 2026-08-27
intake:
pipeline_spec: docs/planning/pipeline/completed/production-server-module-base-and-observation-hooks.spec.md
---

# TICKET-040-production-server-module-base-and-observation-hooks

## Summary

Implement the first production server-module base selected by ADR-0004: exact
release/admission registry, one safe post-commit observation hook, one bounded
typed intent, process-isolated component hosting, durable delivery/receipts,
namespaced state, lifecycle, recovery, and conformance. Do not enable arbitrary
operator package installation in this slice.

## Why

Ticket 039 proves the containment and contract. Production needs the smallest
observation-only vertical slice before administrators can admit custom
executable packages or admission hooks can affect pending platform commands.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When production starts without module configuration, the system shall preserve the current route, process, database, and domain behavior and start no module host. | Existing workspace/integration/QML suites and startup inventory. |
| REQ-002 | When an exact reviewed fixture release is registered, the system shall verify canonical release/provenance/admission/WIT/component bindings and persist an immutable exact-release inventory without granting requested capabilities implicitly. | PostgreSQL integration and hostile contract corpus. |
| REQ-003 | When an allowlisted domain transaction emits the first observation, the system shall append it to a bounded durable module outbox in the same transaction when delivery is active; when the optional module is inactive or saturated, the system shall commit the core transaction and atomically increment bounded aggregate gap evidence without exposing account ownership, credentials, arbitrary paths/URLs, or ungranted private data. | Transaction commit/gap, privacy, bounds, and direct SQL tests. |
| REQ-004 | When the dispatcher delivers events, the system shall preserve order per exact release/hook/subject partition, apply at-least-once replay receipts with retained request/response evidence, bounded retries/backoff/dead-letter state, a hard queue ceiling, and an outer deadline without holding a domain database transaction during module execution. | Concurrency, replay, evidence retention/pruning, outage, timeout, saturation, and lock-duration tests. |
| REQ-005 | When a component proposes the first typed intent, a core domain service shall reauthorize current lifecycle, capability, subject, target, policy, expected revision, and idempotency before committing the effect and immutable receipt atomically. | Authorization and transaction-concurrency corpus. |
| REQ-006 | When the module host runs, the system shall use one exact no-WASI component release per dedicated OS-contained process with pinned runtime, read-only bytes, no server secrets/database/network hostcall, independent memory/CPU/task/file/fuel limits, health, circuit breaking, disablement, and emergency suspension. | Real process containment and hostile component drill. |
| REQ-007 | When module configuration/state/lifecycle changes or backup/restore occurs, the system shall enforce core-owned namespaces, quotas, CAS revisions through readiness finalization, explicit migrations, expected-state audit, retained rollback, canonical pre-start restore reconciliation into a disabled review state, and receipt/outbox reconciliation. | State/readiness-race/lifecycle/migration/backup/isolated-restore tests. |
| REQ-008 | When the slice completes, it shall publish deterministic author/operator conformance fixtures, document compatibility and operations, pass security/CodeGraph/OpenWiki/local gate evidence, and retain the prohibition on arbitrary custom installation, admission hooks, game authority, and client executable content. | Conformance clean-room proof, source/route audit, docs, and local diff gate. |

## Scope

- In:
  - ADR-0004 production registry, observation outbox/dispatcher, one safe
    typed hook/intent, host/service base, namespaced state, lifecycle,
    recovery, telemetry, and conformance;
  - one exact reviewed test/first-party fixture for end-to-end evidence.
- Out:
  - administrator-uploaded/custom module installation;
  - admission/fail-closed hooks, arbitrary egress, database/plugin/native
    hostcalls, provider/game authority, client code, or public marketplace
    review operations.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/production-server-module-base-and-observation-hooks.spec.md)
- Architecture: [ADR-0004](../../../architecture/adr-0004-process-isolated-wasm-server-modules.md), [Server modules](../../../architecture/server-modules.md)
