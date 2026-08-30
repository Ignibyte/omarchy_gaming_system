---
title: INTAKE-official-marketplace-hosting-custody-and-operations
status: candidate
created: 2026-08-30
ticket:
pipeline_spec:
---

# INTAKE-official-marketplace-hosting-custody-and-operations

## Problem or opportunity

Ticket 037 completed deterministic marketplace publication, offline handoff,
immutable activation, guarded mirror verification, and compromise/rollback
drills. The repository still has no official domains, storage/CDN accounts,
production keys, offline media or HSM custody, monitoring/paging, review staff,
retention operation, or incident communications. Those external facts are the
remaining owner-operated marketplace roadmap item and cannot be inferred from
the local fixtures.

The current v1 trust protocol also cannot replace a possibly compromised
offline root in band. That event requires a separately reviewed client
bootstrap/package release and incident plan, so root-replacement engineering
must precede the live recovery exercise instead of being hidden inside a
hosting checklist.

## Proposed outcome

OmarchyGS operates an official, immutable HTTPS channel and marketplace across
independently hosted mirror pairs. Named people exercise separated online
catalog and offline-root custody, reviewed releases move through a recorded
dual-control ceremony, exact probes and paging continuously detect divergence,
retained evidence can be restored, and both catalog-key compromise and
offline-root replacement have real recovery records. No secret or mutable
hosting credential enters the repository.

## Candidate delivery sequence

Promotion should split this candidate into independently reviewable tickets:

1. design and implement the explicit client-bootstrap/root-replacement
   protocol and recovery drill;
2. define the official service ownership, review, retention, incident, and
   evidence policies;
3. provision the approved domains, accounts, storage/CDN, TLS, monitoring,
   paging, and custody systems;
4. execute the first dual-control production publication and mirror rollout;
5. operate through an approved observation window and perform catalog-key,
   restore, retention, and offline-root-replacement exercises.

No ticket may claim a later stage from local simulation of an earlier stage.

## Candidate EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | Before provisioning begins, the project shall record approved service owners, release reviewers, online catalog custodians, offline-root custodians, hosting operators, security/incident contacts, budget authority, jurisdictions, and separation-of-duty rules. | Approved responsibility matrix and review record containing no personal secrets or credentials. |
| REQ-002 | When a client must replace a lost or suspected-compromised offline marketplace root, the supported bootstrap path shall authenticate the exact successor root independently of the old root, require explicit user/operator evidence, prevent silent server-origin substitution, preserve terminal old-root history, and reject rollback. | Separately ticketed threat model, implementation, hostile bootstrap/package tests, clean-client recovery drill, and signed incident procedure. |
| REQ-003 | When production key custody is established, catalog and offline-root private keys shall be distinct, non-exportable or owner-protected as approved, backed up through separate recovery custody, exercised under dual control, and absent from application hosts, repositories, tickets, logs, and publication artifacts. | Custody ceremony, public-key comparison, backup/recovery exercise, access review, and secret scan. |
| REQ-004 | When official hosting is provisioned, canonical channel and marketplace origins plus every approved mirror pair shall use valid HTTPS, immutable version prefixes, exact documented media types, transformation-free artifact delivery, and an atomic current-version selection that cannot expose a partial generation. | DNS/TLS/account ownership evidence, storage/CDN configuration review, authenticated remote probes, and partial-rollout failure drill. |
| REQ-005 | When the first production bundle is published, the online preparer, independent reviewers, offline custodian, finalizer, and hosting operators shall execute the existing separated ceremony over one exact reviewed cartridge and native package release. | Secret-free plan/request/response/finalization receipts, exact publication digest, role attestations, and remote mirror probe receipts. |
| REQ-006 | While the official service operates, monitoring shall continuously cover canonical and mirror availability, TLS expiry, authenticated publication identity and digest, bundle/snapshot freshness floors, split or stale generations, content length/type, storage capacity, probe failures, and unusual request/error volume. | Monitor definitions, synthetic probe history, alert routing, paging acknowledgement, and injected stale/tampered/expired failures. |
| REQ-007 | When a monitor detects alternate roots, stale or split publications, missing/extra/transformed bytes, invalid TLS, rollback, or an unexpected current digest, rollout shall stop and the incident path shall preserve evidence before any higher-version recovery. | Automated stop/paging evidence plus a timed tabletop or live fault exercise. |
| REQ-008 | When catalog-key compromise is exercised, operators shall revoke the affected key in retained history, append a distinct successor, publish a higher root-authorized bundle/snapshot, converge every mirror, deny stale activation, and verify client/server reconciliation before effects. | Real custody and hosted recovery transcript using the production controls and bounded public receipts. |
| REQ-009 | When an offline root is lost or suspected compromised, operators shall stop publication, invoke the independently authenticated replacement path, distribute the exact successor client bootstrap/package, verify clean-client enrollment, and preserve the terminal old-root incident record without treating the old root as replacement authority. | Root-replacement exercise across clean clients, exact package/root identities, rollback tests, and incident communications. |
| REQ-010 | Before the immutable local publication store reaches its sixteen-version ceiling, an approved retention procedure shall archive complete versions and receipts to protected evidence storage, verify restoration byte-for-byte, and require reviewed disposition before deletion. | Retention policy, archive manifest, restore/probe exercise, deletion authorization record, and refusal-at-ceiling regression retained. |
| REQ-011 | When a release is submitted for official review, assigned reviewers shall apply the approved source, licensing, publisher identity, reproducible artifact, conformance, malware/security, privacy/data, support, and lifecycle policy before signing bounded review facts. | Reviewer checklist, independent approvals, exact source/artifact/provenance identities, and rejection/withdrawal exercise. |
| REQ-012 | When an availability, integrity, custody, reviewer, or compromise incident occurs, the project shall communicate bounded impact, affected exact identities, containment, player/operator action, and recovery status through approved channels without publishing secrets or unsupported conclusions. | Incident template, contact/channel ownership, timed tabletop, redaction review, and public/private communication records. |
| REQ-013 | When backups or hosting state are restored, operators shall reconstruct the immutable store and serving pointer from authenticated evidence, probe every origin, and prove no unpublished, rolled-back, or mixed generation becomes current. | Real backup/restore exercise in isolated production-equivalent accounts followed by exact remote probes. |
| REQ-014 | Only after provisioning, first publication, monitoring/paging, custody recovery, retention restore, catalog compromise, root replacement, review staffing, and incident communication evidence pass through the approved observation window shall the official-hosting roadmap item be marked complete. | Requirement audit, linked evidence index, dated operational review, and narrowly scoped roadmap change. |

## Scope notes

- In:
  - separately gated root-replacement design and client-bootstrap support;
  - official domains, TLS, immutable object hosting/CDN, canonical and mirror
    pairs, continuous probes, monitoring, paging, and capacity controls;
  - production catalog/root custody, backups, dual control, restore exercises,
    review staffing/policy, retention, and incident communications;
  - real first publication plus an approved operating observation window.
- Out:
  - placing cloud, DNS, HSM, TLS, signing, paging, or monitoring secrets in Git;
  - treating local TLS fixtures or `scripts/test-marketplace-publication.sh` as
    proof that external services, people, or custody exist;
  - adding a mutable marketplace API or allowing clients to choose mirror
    fallback authorities;
  - using the possibly compromised old root to authorize its successor;
  - provisioning paid services, registering domains, contacting staff, or
    publishing incident messages without explicit authority.

## Promotion checklist

- [x] Ticket 042 delivery explicitly authorized and completed.
- [ ] Root-replacement product/security decision approved for its own ticket.
- [ ] Domain, account, budget, and external-write authority supplied.
- [ ] Named role owners and approved operating observation window supplied.
- [ ] Ticket sequence created and indexed one shippable slice at a time.
- [ ] First promoted ticket's pipeline spec/notes pair created.
- [ ] `ticket:` and `pipeline_spec:` filled for the promoted slice.
- [ ] Status changed to `promoted` only for the promoted slice.
