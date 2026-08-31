---
title: INTAKE-reviewed-external-provider-onboarding
status: candidate
created: 2026-08-30
ticket:
pipeline_spec:
---

# INTAKE-reviewed-external-provider-onboarding

## Problem or opportunity

Door Legends v1 is intentionally the sole first-party registered-provider
authority pilot. The platform has exact registration, lifecycle, keys, scopes,
quotas, guarded egress, replay, audit, reconciliation, and recovery, but it has
no policy or operational authority to admit third-party providers. A public SDK
alone cannot establish publisher identity, release review, support, monitoring,
incident response, data practices, or achievement trust.

The final roadmap item must remain gated until the public Provider SDK and real
operations prove those controls. It should begin with one reviewed external
pilot, not a self-service marketplace or an automatic activation path.

## Proposed outcome

One independently operated external provider passes a documented identity,
source/build, protocol, security, privacy, reliability, recovery, support, and
lifecycle review; is registered and activated by an operator for one exact
game release; exposes accurate player provenance; survives monitored
operations and recovery exercises; and can be suspended or revoked without
losing platform authority. Only after that pilot is reviewed may the project
decide whether broader onboarding is justified.

## Candidate EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | Before review begins, a provider applicant shall supply verifiable publisher/operator identity, security and support contacts, exact source and build provenance, dependency/license inventory, data-flow and retention disclosures, service locations, lifecycle dates, and vulnerability/incident commitments. | Application schema validation, identity/contact verification, provenance rebuild, policy review, and deliberate incomplete/false application rejection. |
| REQ-002 | When a release is reviewed, independent reviewers shall bind one immutable provider/release/game/rules/cartridge identity to the public SDK/protocol version, endpoint/TLS roots, message keys, scopes, quotas, active-session policy, achievements, data policy, support window, and review evidence. | Signed bounded review record, exact registration comparison, independent approvals, and changed-byte/identity rejection. |
| REQ-003 | When conformance and security review run, the external release shall pass the public valid/fault corpus, clean-room reproducible build, dependency/license scan, source review, guarded-egress/TLS tests, replay/concurrency tests, callback/result policy tests, and separate-database backup/restore exercise. | Machine-readable conformance receipts, review report, build attestations, real broker run, and restore/reconciliation evidence. |
| REQ-004 | When the platform operator registers or activates the reviewed release, authority shall remain database-local, exact, audited, replay-safe, and independent of marketplace publication or SDK availability; no network self-service caller shall gain registration authority. | Administrator command tests, API/discovery negative review, immutable audit, and exact review-to-registration comparison. |
| REQ-005 | When players discover or launch an externally provided game, the platform shall disclose the provider identity, exact release, external authority/availability, review status, support/security contact boundary, and material data handling without exposing its endpoint, keys, or direct client transport. | API/QML contract and keyboard/accessibility tests plus privacy review. |
| REQ-006 | While an external release operates, monitoring shall cover authenticated availability, latency/error/quota trends, signature or protocol rejection, replay/reconciliation health, callback lag, database capacity, TLS/key expiry, backup freshness, support response, and policy violations. | Dashboards/alerts, synthetic operations, paging exercise, and observation-period report. |
| REQ-007 | When provider, release, scope, message key, or TLS trust is suspended or revoked, new authority shall stop according to exact lifecycle policy, affected sessions shall become bounded read-only/terminated as specified, and no WebSocket, cached state, alternative provider, or compiled rules shall restore gameplay authority. | Live suspension/revocation matrix, player behavior observations, audit, and no-fallback assertions. |
| REQ-008 | When an operation has an unknown outcome or the provider recovers from outage, retries shall preserve semantic idempotency and expected revision, and session recovery shall use authenticated reconciliation rather than inferred time or platform-owned gameplay state. | Timeout/retry/outage/restart exercises and durable receipt/reconciliation audit. |
| REQ-009 | When the provider or platform restores from backup, each database shall remain independent, exact keys/releases shall be re-established, platform projections shall not become gameplay snapshots, and service shall resume only after authenticated reconciliation and review. | Dual backup/isolated-restore drill, schema/data inventory, key comparison, and post-restore session exercise. |
| REQ-010 | When provider result or achievement claims arrive, OmarchyGS shall authenticate and deduplicate them, reauthorize the exact current release/policy, accept only reviewed bounded definitions, and atomically publish platform projections/invalidations without trusting arbitrary claims. | Policy and callback integration tests, hostile claims, lifecycle races, and projection/audit assertions. |
| REQ-011 | When a security, privacy, availability, support, or end-of-life obligation fails, operators shall stop new launches, suspend or revoke the narrowest affected authority, preserve evidence, communicate bounded impact, and execute the documented player/session disposition. | Incident tabletop/live exercise, response-time evidence, operator receipts, communications review, and recovery/retirement outcome. |
| REQ-012 | Only after the Provider SDK, hosted operational dependencies, review staffing/policy, one external pilot, monitored observation window, suspension, incident, backup/restore, and retirement exercises pass shall the reviewed-external-provider roadmap item be marked complete. | Dependency receipts, pilot acceptance audit, dated operational review, ticket closure, and narrowly scoped roadmap change. |

## Scope notes

- In:
  - one reviewed external provider pilot and exact-release review evidence;
  - publisher/operator identity, provenance, security/privacy/support/lifecycle
    policy, player disclosure, monitoring, recovery, suspension, and retirement;
  - existing database-local registration and broker authority, extended only
    where independently justified by the pilot's reviewed requirements.
- Out:
  - self-service or automatic provider registration/activation;
  - arbitrary provider discovery, direct client-provider traffic, provider UI,
    shared credentials/databases, or fallback to compiled rules;
  - treating an SDK signature or protocol pass as marketplace review, support,
    security, or operational approval;
  - general server-module installation or cartridge code execution;
  - contacting providers, signing agreements, provisioning services, or making
    public claims without explicit external authority.

## Promotion checklist

- [x] Ticket 042 delivered and preceding roadmap dependencies dispositioned.
- [x] Public Provider SDK/starter/conformance/sidecar roadmap item complete.
- [ ] Official operations, recovery, suspension, support, and review staffing
      evidence approved for external-provider use.
- [ ] Candidate external provider and exact pilot release selected with consent.
- [ ] Legal/privacy/security/support review ownership supplied.
- [ ] Ticket created and indexed.
- [ ] Pipeline spec/notes pair created.
- [ ] `ticket:` and `pipeline_spec:` filled.
- [ ] Status changed to `promoted`.
