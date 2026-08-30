---
title: Public Provider SDK contract, negotiation, and release
pipeline_id: fb5cf56b-6421-482c-badf-fc3e3b02a92e
status: Phase 5 — Complete PASS
ticket: TICKET-044
ticket_doc: docs/planning/tickets/closed/TICKET-044-public-provider-sdk-contract-negotiation-and-release.md
aar: docs/planning/knowledge/aar/AAR-044-public-provider-sdk-contract-negotiation-and-release.md
created: 2026-08-30
---

# Public Provider SDK contract, negotiation, and release — spec

## Intent

Ship the provider-facing half of the existing production provider seam as one
standalone, versioned, reproducible SDK contract and replace implicit v1
compatibility with an authenticated exact-capability handshake before provider
effects, without enabling external providers.

## Scope

- In:
  - provider-facing scope, errors, protocol types, signing and verification;
  - explicit authenticated protocol/capability negotiation;
  - a public-only crate plus exact SDK export, lock, schemas, fixtures, notices,
    checksums, and locally signed provenance;
  - internal compatibility re-exports and Door Legends clean-clone migration;
  - focused and canonical local evidence.
- Out:
  - starter/conformance-kit/second-game and sidecar follow-up slices;
  - provider onboarding or any new player/operator API;
  - hosted publication or CI/CD;
  - changing the sole authorized Door Legends release.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the Provider SDK is exported, it shall contain only documented provider-facing protocol, model, helper, schema, fixture, notice, and provenance files and shall exclude platform registry, broker, egress, database, administrator, secret, and repository-relative surfaces. | Exact export inventory, forbidden-surface scan, packaged dependency tree, and fresh-repository build. |
| REQ-002 | When the platform and a provider establish compatibility, they shall authenticate and bind one exact supported protocol version and the required capability set before launch, command, reconcile, or callback effects; missing, unknown, ambiguous, stripped, or downgrade-selected compatibility shall fail closed. | Negotiation unit matrix, grant/message binding tests, hostile vectors, and real broker/provider exercise. |
| REQ-003 | When an SDK consumer verifies a grant or signed HTTP message, public helpers shall bind the provider, release, game, rules, cartridge, session, pairwise subject, scope, expiry, replay, request context, negotiated compatibility, and exact bytes before returning parsed payloads. | Public API unit/interop tests and mismatch, stale/future, replay, digest, signature, schema, depth, value, and body-bound cases. |
| REQ-004 | When one reviewed SDK revision is exported twice or consumed from two fresh repositories, files, manifests, schemas, fixtures, checksums, release provenance, and builds shall be identical or verify against the same exact SDK identity without an OmarchyGS path dependency. | Deterministic export comparison, local release signature verification, two clean Git clones, packaged-crate builds, and source-path leak scan. |
| REQ-005 | When the SDK slice is delivered, existing compiled games and the Door Legends provider pilot shall retain their authority behavior, while no public route or SDK operation shall register, activate, discover, trust, or list an external provider. | Workspace/provider/server regressions, route and dependency diff, operator-boundary review, and canonical local gate. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | OmarchyGS owns and locally signs the preview SDK export; no external registry publication is part of this ticket. | It supplies exact release ownership and provenance without creating an external account, legal distribution promise, or hosted delivery dependency. |
| 2 | Protocol v1 is the only current version and launch, command, reconcile, and event are the exact required capability set. | An exact one-version window is the smallest honest compatibility policy and rejects ambiguity or downgrade. |
| 3 | Compatibility is authenticated before mutation and then bound into the grant and every operation, response, and callback body. | A metadata-only version field after an effect would not prove safe negotiation. |
| 4 | The dedicated SDK crate owns only provider-facing code; `omarchy-game-provider` remains the platform implementation and re-exports the public contract for internal source compatibility. | This prevents packaged dormant platform source from masquerading as a public SDK while minimizing migration risk. |
| 5 | Door Legends remains the only admitted provider and no SDK release action changes registry state. | SDK availability and platform trust/admission are separate authorities. |
| 6 | Starter/conformance/second-game and sidecar work remain Tickets 045 and 046 candidates, opened only after this slice is delivered. | One active spec/notes pair preserves the repository workflow and keeps each slice independently auditable. |

## Linked artifacts

- Ticket: [TICKET-044](../../tickets/closed/TICKET-044-public-provider-sdk-contract-negotiation-and-release.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Intake: [Public Provider SDK](../../intake/public-provider-sdk-starter-and-sidecar.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | autonomous scope lock plus tool readiness |
| 2 Design | Public boundary, handshake, release/export, file and regression manifests | worktree-bound CodeGraph receipt plus direct unsupported-file review |
| 3 Implement | SDK, negotiation integration, export/provenance, clean-repository proof | focused compile/tests and self-review |
| 3.5 Inspect | Correctness, security, compatibility, privacy, and simplification ledger | fixes plus fresh CodeGraph receipt |
| 4 Validate | Focused suites and complete local diff gate | matching worktree receipt |
| 5 Complete | AC audit, OpenWiki, AAR, roadmap/intake reconciliation, archive | no silent drops |
| Delivery | Staged review, authorized commit, push, and remote readback | matching local receipt and remote commit |
