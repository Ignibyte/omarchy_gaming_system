---
title: Reviewed provider sidecar and deployment operations
pipeline_id: 35105398-ffdc-433f-b83a-86e418471a07
status: Phase 5 — Complete PASS
ticket: TICKET-046
ticket_doc: docs/planning/tickets/closed/TICKET-046-reviewed-provider-sidecar-and-deployment-operations.md
aar: docs/planning/knowledge/aar/AAR-046-reviewed-provider-sidecar-and-deployment-operations.md
created: 2026-08-30
completed: 2026-08-30
---

# Reviewed provider sidecar and deployment operations — spec

## Intent

Complete the locally actionable Provider SDK roadmap by shipping one explicit,
authenticated co-located transport and a drill-backed remote/co-located
operations profile without granting any new provider admission authority.

## Scope

- In:
  - exact release/endpoint/loopback-socket production sidecar binding;
  - unchanged TLS, signed-message, grant, quota, deadline, replay, audit, and
    lifecycle enforcement;
  - separate provider process, database, credentials, paths, and supervision;
  - lifecycle, key rotation, suspension, backup/restore, upgrade, and
    reconciliation drills;
  - remote and co-located operator guidance and local delivery evidence.
- Out:
  - external provider onboarding or a second production-admitted provider;
  - public hosting, domains, package publication, or hosted automation;
  - shared process/state/credentials or compiled gameplay fallback;
  - general loopback/private-network egress exceptions.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an operator selects the sidecar profile, OmarchyGS shall bind one exact registered provider release and canonical HTTPS endpoint to one exact loopback socket while retaining DNS authority, TLS verification, signed messages, grants, scopes, quotas, deadlines, replay protection, audit, and current lifecycle checks. | Threat model, configuration/unit tests, real broker integration, and signed drill receipt. |
| REQ-002 | When sidecar configuration contains an IP-literal endpoint, non-loopback socket, endpoint/socket port mismatch, wrong release or endpoint, redirected path, hostile listener, or incomplete values, startup or transport shall fail closed without opening a general loopback/private-network egress exemption. | Hostile configuration and peer tests plus source inspection. |
| REQ-003 | When the co-located provider is installed or operated, the documented profile shall keep a separate process, operating identity, PostgreSQL database, TLS/message/grant credentials, writable paths, resource limits, service lifecycle, and network permissions from OmarchyGS. | Template validation and process-containment review. |
| REQ-004 | When the sidecar starts, stops, crashes, upgrades, or loses its database, OmarchyGS shall retain platform authority, deny new affected launches, keep existing affected sessions read-only, and recover only through authenticated reconciliation without shared state or compiled fallback. | Start/stop/crash/upgrade/backup-restore/reconciliation drill and database-separation assertions. |
| REQ-005 | When a provider is deployed remotely, the operations guide shall cover TLS identity, immutable DNS/endpoint registration, separate database, least-privilege secrets, rotation, quotas, health/monitoring, backup/restore, suspension/revocation, incident response, upgrades, and end-of-life. | Documentation contract assertions and operator walkthrough. |
| REQ-006 | When this ticket completes, the public Provider SDK roadmap item shall have local delivery evidence while external provider onboarding, hosted publication, and production admission remain unauthorized and visibly open. | Route/catalog/registry diff inspection, roadmap/intake audit, canonical gate, and delivery readback. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | The sidecar uses an exact TLS-over-loopback socket binding for one configured release and canonical DNS endpoint; it is a production profile distinct from the conformance-only override. | This preserves the established HTTPS/SNI/Host/signature protocol while avoiding DNS-based private-address admission and a broad network exemption. |
| 2 | The sidecar remains a separately supervised provider-starter process with its own PostgreSQL database and credentials. | Co-location changes transport latency and operations, not authority or failure domains. |
| 3 | Door Legends remains the sole production-admitted release; the drill uses ephemeral registration and state only. | Transport proof must not silently authorize Relay Forge or external providers. |
| 4 | Remote deployment is documentation/template work in this ticket; no hosted system, domain, account, or production key is provisioned. | Those actions need external authority and real operators that local evidence cannot counterfeit. |

## Linked artifacts

- Ticket: [TICKET-046](../../tickets/closed/TICKET-046-reviewed-provider-sidecar-and-deployment-operations.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Intake: [Public Provider SDK](../../intake/public-provider-sdk-starter-and-sidecar.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled constraints | autonomous scope lock plus tool readiness |
| 2 Design | Separate threat model, transport/config/template/drill manifest | worktree-bound CodeGraph receipt plus unsupported-file review |
| 3 Implement | Sidecar transport, operations assets, hostile and lifecycle coverage | focused compile/tests and self-review |
| 3.5 Inspect | Correctness, security, containment, lifecycle, compatibility | fixes plus fresh CodeGraph/security evidence |
| 4 Validate | Focused suites and complete local diff gate | matching worktree receipt |
| 5 Complete | AC audit, OpenWiki, AAR, roadmap/intake reconciliation, archive | no silent drops |
| Delivery | Staged review, authorized commit, push, and remote readback | matching local receipt and remote commit |
