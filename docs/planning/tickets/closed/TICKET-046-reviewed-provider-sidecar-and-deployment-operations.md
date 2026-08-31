---
title: TICKET-046-reviewed-provider-sidecar-and-deployment-operations
status: closed
ticket_number: 046
type: feature
created: 2026-08-30
closed: 2026-08-30
intake: docs/planning/intake/public-provider-sdk-starter-and-sidecar.md
pipeline_spec: docs/planning/pipeline/completed/reviewed-provider-sidecar-and-deployment-operations.spec.md
---

# TICKET-046-reviewed-provider-sidecar-and-deployment-operations

## Summary

Ship a threat-modeled co-located provider sidecar profile and complete the
remote and co-located deployment/operations guide, with executable lifecycle,
rotation, restore, suspension, containment, and reconciliation evidence.

## Why

Tickets 044 and 045 made the Provider SDK, starter, conformance kit, and second
game independently consumable, but their loopback overrides are deliberately
test-only. Operators still lack one production-safe way to co-locate a
provider without weakening guarded egress or sharing platform state and
credentials, and the roadmap lacks a reviewed end-to-end operations profile.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an operator selects the sidecar profile, OmarchyGS shall bind one exact registered provider release and canonical HTTPS endpoint to one exact loopback socket while retaining DNS authority, TLS verification, signed messages, grants, scopes, quotas, deadlines, replay protection, audit, and current lifecycle checks. | Threat model, configuration/unit tests, real broker integration, and signed drill receipt. |
| REQ-002 | When sidecar configuration contains an IP-literal endpoint, non-loopback socket, endpoint/socket port mismatch, wrong release or endpoint, redirected path, hostile listener, or incomplete values, startup or transport shall fail closed without opening a general loopback/private-network egress exemption. | Hostile configuration and peer tests plus source inspection. |
| REQ-003 | When the co-located provider is installed or operated, the documented profile shall keep a separate process, operating identity, PostgreSQL database, TLS/message/grant credentials, writable paths, resource limits, service lifecycle, and network permissions from OmarchyGS. | Template validation and process-containment review. |
| REQ-004 | When the sidecar starts, stops, crashes, upgrades, or loses its database, OmarchyGS shall retain platform authority, deny new affected launches, keep existing affected sessions read-only, and recover only through authenticated reconciliation without shared state or compiled fallback. | Start/stop/crash/upgrade/backup-restore/reconciliation drill and database-separation assertions. |
| REQ-005 | When a provider is deployed remotely, the operations guide shall cover TLS identity, immutable DNS/endpoint registration, separate database, least-privilege secrets, rotation, quotas, health/monitoring, backup/restore, suspension/revocation, incident response, upgrades, and end-of-life. | Documentation contract assertions and operator walkthrough. |
| REQ-006 | When this ticket completes, the public Provider SDK roadmap item shall have local delivery evidence while external provider onboarding, hosted publication, and production admission remain unauthorized and visibly open. | Route/catalog/registry diff inspection, roadmap/intake audit, canonical gate, and delivery readback. |

## Scope

- In:
  - exact authenticated production sidecar transport and server configuration;
  - provider-starter callback support for the same exact sidecar profile;
  - co-located service/config templates, containment checks, and lifecycle drill;
  - complete remote and co-located deployment/operations guide;
  - focused tests, security inspection, local gate, and documentation closure.
- Out:
  - external/self-service provider registration, review, admission, or support;
  - public package-registry publication, hosted deployment, domains, or CI/CD;
  - shared platform/provider database, process, credentials, or compiled fallback;
  - a general loopback or private-network allowlist.

## Links

- Intake: [Public Provider SDK](../../intake/public-provider-sdk-starter-and-sidecar.md)
- Pipeline spec: [reviewed-provider-sidecar-and-deployment-operations.spec.md](../../pipeline/completed/reviewed-provider-sidecar-and-deployment-operations.spec.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- AAR: [AAR-046](../../knowledge/aar/AAR-046-reviewed-provider-sidecar-and-deployment-operations.md)
