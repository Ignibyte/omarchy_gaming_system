---
title: TICKET-041-administrator-custom-server-module-installation-and-provenance
status: closed
ticket_number: 041
type: feature
created: 2026-08-27
closed: 2026-08-29
intake:
pipeline_spec: docs/planning/pipeline/completed/administrator-custom-server-module-installation-and-provenance.spec.md
---

# TICKET-041-administrator-custom-server-module-installation-and-provenance

## Summary

After Ticket 040 proves the production module base, add database-local
administrator installation, review, enable/disable, upgrade/rollback, removal,
and recovery for exact operator-custom module releases with permanent
player-facing custom-server provenance and no client executable bridge.

## Why

Owner-operated communities need a deliberate escape hatch without letting
unreviewed executable code claim marketplace provenance, expand its own
capabilities, inherit server secrets, or become OmarchyGS-supported code.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an administrator imports a custom module, the system shall require a bounded private database-local command, exact publisher integrity, operator-custom provenance, explicit unreviewed-code acknowledgement, requested/granted capability review, compatible WIT/component bytes, immutable staging, actor/reason, and an idempotent operation UUID. | Admin integration plus malformed/path/key/capability/replay hostile corpus. |
| REQ-002 | When a custom release is enabled, upgraded, rolled back, disabled, suspended, removed, or recovered, the system shall require expected lifecycle/state revisions, host readiness, compatible atomic namespace migration, retained rollback, immutable audit, and no delivery outside the exact active admission. | Concurrent lifecycle, migration, crash, rollback, and recovery drills. |
| REQ-003 | When a custom module affects server behavior visible to players, discovery and the trusted client shall disclose stable server identity, operator-custom status, module-behavior capability summary, and support boundary without sending component bytes, raw code, private inventory, or signing authority to the client. | API/QML schema, privacy, keyboard/accessibility, and hostile provenance tests. |
| REQ-004 | When custom and marketplace-vetted modules execute, the system shall apply the same WIT, capability, resource, state, dispatcher, receipt, sandbox, and conformance rules while keeping their attestations, warnings, audit, and support claims distinct. | Shared conformance matrix and trust-claim audit. |
| REQ-005 | When an operator bypasses marketplace review, the system shall not claim OmarchyGS review/support, expose server/database credentials, permit arbitrary native/QML/JavaScript/client delivery, allow direct protected-state mutation, or let a general hook become a game provider. | Security review, source/route/client inventory, and real containment drill. |
| REQ-006 | When the slice completes, it shall document terms, privacy/telemetry, security contact, patching, backup/recovery, incident, and operator responsibility expectations and pass security/CodeGraph/OpenWiki/local gate evidence. | Documentation audit and local diff gate. |

## Scope

- In:
  - operator-custom exact module import and lifecycle over the Ticket 040 base;
  - explicit trust/provenance, player disclosure, conformance, operations,
    recovery, and responsibility terms.
- Out:
  - public self-service marketplace approval, remote administrator API,
    admission hooks, arbitrary egress/hostcalls, game-provider substitution,
    or client executable content.

## Links

- Intake:
- Pipeline spec: [administrator-custom-server-module-installation-and-provenance.spec.md](../../pipeline/completed/administrator-custom-server-module-installation-and-provenance.spec.md)
- Architecture: [ADR-0004](../../../architecture/adr-0004-process-isolated-wasm-server-modules.md), [Server modules](../../../architecture/server-modules.md)
