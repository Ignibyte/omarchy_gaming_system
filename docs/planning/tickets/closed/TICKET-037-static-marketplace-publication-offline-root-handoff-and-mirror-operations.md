---
title: TICKET-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations
status: closed
ticket_number: 037
type: feature
created: 2026-08-27
closed: 2026-08-27
intake:
pipeline_spec: docs/planning/pipeline/completed/static-marketplace-publication-offline-root-handoff-and-mirror-operations.spec.md
---

# TICKET-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations

## Summary

Turn the existing signed cartridge, marketplace snapshot, trust-channel, and
native-package contracts into one deterministic static publication workflow
with a narrow online catalog-signing boundary, an explicit offline-root
handoff, exact mirror verification, monitoring receipts, and a rehearsed
rotation/revocation incident path.

## Why

Ticket 036 gave players and owner-operated servers a root-authenticated trust
and package channel, but repository fixtures still assemble hosted marketplace
state by hand. There is no production-shaped tool that verifies reviewed
release inputs, creates the exact static tree consumed by servers and clients,
separates catalog signing from offline root custody, proves mirrors identical,
or records a recoverable publication/incident ceremony.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an operator describes a publication candidate, the tooling shall accept one canonical bounded exact-schema plan that pins channel, marketplace, snapshot, bundle, validity, key, package, release-review, and output identities without embedding private material. | Contract unit tests and canonical/hostile plan corpus. |
| REQ-002 | When a reviewed cartridge release enters the workflow, the online publication phase shall verify its publisher signature, SDK identity, conformance, host compatibility, exact components, review metadata, and monotonic lifecycle intent before any marketplace signature or output becomes publishable. | Verified-release fixtures and tamper/incompatibility/path/limit tests. |
| REQ-003 | When marketplace policy and snapshot bytes are produced, only an explicit absolute mode-0600 catalog private-key file shall authorize signing, and the output shall be the canonical exact `snapshot.signed.json` plus release component paths already consumed by server synchronization. | CLI filesystem/key tests and existing server sync against produced output. |
| REQ-004 | When a trust-channel update is requested, the online phase shall emit a canonical public offline-signing request that binds the exact catalog snapshot version/key lifecycle, package artifacts, channel origin, marketplace origin, validity window, prior trust digest, and next bundle version without carrying any private key. | Handoff schema, transition, secret-absence, and reproducibility tests. |
| REQ-005 | When the offline ceremony signs a request, it shall require an explicit absolute mode-0600 root private-key file, revalidate the complete request and previous trust transition, write only a new root-signed trust document and public receipt, and never contact a network destination. | Offline command isolation, filesystem, rollback, tamper, expiry, and network-absence tests. |
| REQ-006 | When a signed offline response returns online, finalization shall independently verify its root signature, exact request identity, channel/origin/version/freshness invariants, catalog keyring, snapshot version, package inventory, and prior transition before accepting it. | Import/finalization matrix with substituted, stale, split-brain, and wrong-root responses. |
| REQ-007 | When a publication set is finalized, the tooling shall build one deterministic immutable static tree containing exact marketplace snapshot/release components, trust document, native packages, public manifests, and audit receipts with fixed relative paths, media types, sizes, digests, and no extra or mutable inputs. | Double-build comparison and exact-tree/type/mode/inventory checks. |
| REQ-008 | When publication replaces a currently served tree, it shall stage and verify a complete candidate before one atomic local pointer transition, retain bounded previous versions for rollback evidence, and preserve the prior current version on any failure or interruption. | Descriptor/path hostile tests plus failure, concurrency, and crash-boundary integration. |
| REQ-009 | When an operator verifies a local publication or mirror, the verifier shall authenticate every root/catalog/publisher claim and exact artifact byte, reject missing/extra/symlink/hardlink/path/permission/divergence/rollback state, and emit a bounded secret-free machine-readable receipt. | Local and copied-mirror conformance corpus. |
| REQ-010 | When monitoring a hosted origin, the probe shall use bounded guarded HTTPS without proxy, redirects, ambient credentials, private destinations, or decompression and shall verify the current public manifest, trust, snapshot, releases, and package artifacts before reporting healthy. | Spawned TLS origin tests covering DNS, TLS, media type, size, timeout, tamper, and mirror drift. |
| REQ-011 | When more than one mirror is configured for an operations drill, the system shall require the same authenticated publication identity at every mirror and shall report partial rollout or split-brain without teaching clients to trust an alternate authority. | Multi-origin local TLS drill and exact identity comparison. |
| REQ-012 | When catalog-key compromise or publication rollback is simulated, the incident workflow shall create a higher root-signed bundle that terminally revokes the affected key, introduces a valid successor when needed, advances package bootstrap floors, denies stale publication, and preserves bounded historical evidence. | End-to-end rotation/revocation/rollback drill through server and client verifiers. |
| REQ-013 | When operators inspect publication history, every prepare, offline sign, finalize, activate, verify, probe, and incident step shall expose stable public provenance and timestamps without private keys, credentials, absolute secret paths, or untrusted rich text. | Exact receipt schemas and secret/path-absence scan. |
| REQ-014 | When this operational tooling is absent or unused, existing manual/channel server configuration, client enrollment/staging, cartridge SDK identity, package build, and owner-operated deployment behavior shall remain compatible. | Existing marketplace, package, SDK, server, client, QML, and recovery suites. |
| REQ-015 | Before delivery, the operational runbook, architecture, roadmap, generated engineering wiki, fixtures, and implementation shall describe the same custody/publication/mirror/incident boundary and pass the canonical worktree-bound diff gate. | Clean-room operations drill, documentation review, OpenWiki lifecycle, and `bin/gate.sh --diff`. |

## Scope

- In:
  - a non-SDK publication contract and CLI for reviewed release/snapshot
    assembly, public offline-root handoff, static-tree finalization, local
    activation, exact mirror verification, and guarded monitoring;
  - deterministic local HTTPS/mirror and key-rotation/revocation drills;
  - operator custody, review, rollout, rollback, monitoring, and incident
    runbooks that expose no real secrets.
- Out:
  - provisioning a production domain, cloud bucket/CDN, HSM/KMS, escrow vendor,
    monitoring account, pager service, or real production signing key;
  - a public upload/review portal, automated approval, arbitrary package
    repositories, client mirror fallback, marketplace billing, or analytics;
  - operator-local custom-cartridge trust, Provider SDK onboarding, server
    modules/hooks, federation, or publisher executable code.

## Links

- Intake: none; next locally actionable owner-ecosystem roadmap slice after the
  externally controlled two-clean-installation acceptance run.
- Pipeline spec: [active spec](../../pipeline/active/static-marketplace-publication-offline-root-handoff-and-mirror-operations.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [system overview](../../../architecture/system-overview.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
