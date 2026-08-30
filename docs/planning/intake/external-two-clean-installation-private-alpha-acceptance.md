---
title: INTAKE-external-two-clean-installation-private-alpha-acceptance
status: candidate
created: 2026-08-30
ticket:
pipeline_spec:
---

# INTAKE-external-two-clean-installation-private-alpha-acceptance

## Problem or opportunity

The private-alpha software path is implemented and deterministically rehearsed,
but the product charter explicitly requires one real external acceptance event.
Two clean Omarchy installations operated by external testers must exercise one
reviewed server/client release without developer intervention. Local fixtures,
containers, virtual display tests, and repeated package builds are valuable
software evidence, but they cannot satisfy this human and deployment claim.

Ticket 042 is complete, validated, and delivered. This intake records the next
slice without claiming that unavailable external systems have been exercised.

## Proposed outcome

A dated, sanitized acceptance record identifies the exact server commit, client
package version and digest, HTTPS origin, operator, and tester labels; records
each runbook observation as pass or fail; links every discovered defect; and
contains no credentials, invitations, private content, or infrastructure
secrets. The roadmap and product status change only when every required
observation has direct evidence from the real external run.

## Candidate EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | Before invitations are issued, the operator shall record a reviewed server commit, matching native client package version and SHA-256 digest, a valid public HTTPS origin, a current green delivery receipt, a successful isolated restore drill, monitoring coverage, and operator/security contact details. | Sanitized preflight section plus direct receipt, package-digest, TLS, monitoring, backup, and restore observations. |
| REQ-002 | When the acceptance run begins, two external testers shall install the identified package on two clean Omarchy installations and independently connect, register with distinct operator-issued invitations, sign in, and create distinct personas without developer intervention. | Tester-attested checklist with timestamps, sanitized installation identities, operator invitation inventory, and no developer action recorded between start and completed personas. |
| REQ-003 | When either persona sends a connection request and the other accepts it, both clean clients shall show the authoritative connection after refresh. | Both testers record the resulting connection state after an explicit refresh. |
| REQ-004 | When both personas exchange private messages, each direction shall arrive and unread/read state shall transition correctly without exposing message contents in the acceptance record. | Sanitized tester observations for send, receive, unread, and read behavior in both directions. |
| REQ-005 | When one client is offline while activity occurs, reconnecting that client shall recover the correct inventory and history from durable REST state without depending on a missed WebSocket event. | Timed offline/activity/reconnect sequence and post-reconnect state observations. |
| REQ-006 | When one tester creates a Signal Siege Versus challenge and the other accepts it, both testers shall complete the match with keyboard-only controls and observe the same terminal outcome and history after reconnect. | Sanitized challenge/session identities, keyboard-only tester attestations, and matching terminal/history observations from both clients. |
| REQ-007 | When a tester submits a clearly labeled test report against the cooperating persona, the operator shall observe it through the local administrator boundary and dismiss it with an audited reason. | Sanitized tester submission observation plus operator queue and audit-event metadata without report contents. |
| REQ-008 | When each tester exits through the visible EXIT control and relaunches, each client shall close cleanly and permit an ordinary sign-in without developer repair. | Both testers record exit, relaunch, and sign-in outcomes. |
| REQ-009 | When the run ends, the operator shall verify both invitations are `used`, revoke every unused test invitation, complete a post-run backup, and confirm the documented restore and incident procedures remain available. | Metadata-only invitation inventory, revocation receipts when applicable, backup evidence, and runbook review. |
| REQ-010 | If any credential exposure, invalid TLS path, cross-persona disclosure, unauthorized mutation, authority-containment failure, data corruption, failed backup/restore, repeatable crash/data loss, or uncontainable abuse occurs, the operator shall stop new invitations and public ingress and record the run as failed. | Stop-condition ledger, containment timestamps/actions, sanitized incident link, and an explicitly failed acceptance result. |
| REQ-011 | When the record is submitted, it shall exclude passwords, invitation codes, bearer tokens, MFA material, database URLs, private message/report contents, signing keys, and unsanitized screenshots while preserving enough exact release and observation evidence for independent review. | Secret/privacy review of the completed record and changed-file secret scan. |
| REQ-012 | When all runbook observations pass on the real external systems, the project shall record the first acceptance event, close its ticket, and mark only the external private-alpha acceptance roadmap item complete; deterministic local rehearsal alone shall never satisfy this requirement. | Requirement-by-requirement audit against the external record, ticket closure, and narrowly scoped roadmap diff. |

## Evidence record contract

The promoted ticket should create one dated Markdown record under a dedicated
planning evidence directory. The record should contain:

- server commit and worktree-bound gate receipt;
- client package filename, package version, SHA-256 digest, and installation
  identity for each clean client;
- HTTPS origin, time window, operator label, and two sanitized tester labels;
- a preflight table and one result row for every candidate requirement above;
- timestamps and sanitized identifiers needed to correlate operator metadata;
- defect or incident links, containment actions, and rerun disposition;
- operator and tester attestations that no developer intervened; and
- an explicit overall `PASS` or `FAIL` that cannot be inferred from blanks.

The record must never include raw invitations, credentials, private content,
security secrets, database connection material, or evidence copied from local
simulation and represented as an external observation.

## External prerequisites

- Explicit authorization to deliver Ticket 042 before starting a new release
  slice in the primary worktree.
- One reviewed release deployed at a public HTTPS origin with operator-owned
  TLS, firewalling, rate limits, monitoring, protected PostgreSQL, off-host
  encrypted backup, and separate MFA-encryption-key custody.
- Two independently controlled clean Omarchy installations able to install the
  exact reviewed native package and verify its digest.
- Two external testers, one operator, protected one-to-one invitation delivery,
  a scheduled test window, and an agreed security/incident contact channel.
- Approval to create any paid hosted resources or contact external people; the
  general instruction to continue project work does not grant those external
  authorities.

## Scope notes

- In:
  - preflight and evidence-capture preparation;
  - one real two-installation external run of the existing private-alpha
    operator checklist;
  - sanitized results, defect linkage, stop/containment evidence, AAR, durable
    documentation reconciliation, and the exact roadmap status change when
    proven;
  - focused reruns after defects are fixed through separately scoped tickets.
- Out:
  - substituting local VMs, containers, QML fixtures, or
    `scripts/test-private-alpha.sh` for the external event;
  - provisioning official long-lived hosted marketplace infrastructure;
  - public provider SDK or external-provider onboarding;
  - recording credentials, invitation secrets, private conversations/reports,
    or unredacted screenshots;
  - marking the run passed after a stop condition or incomplete observation.

## Promotion checklist

- [x] Ticket 042 delivery explicitly authorized and completed.
- [ ] Public HTTPS server and exact reviewed release identified.
- [ ] Two clean external Omarchy installations and two testers scheduled.
- [ ] Operator, security contact, monitoring, backup, restore, and stop
      authority confirmed.
- [ ] Ticket created and indexed.
- [ ] Pipeline spec/notes pair created.
- [ ] `ticket:` and `pipeline_spec:` filled.
- [ ] Status changed to `promoted`.
