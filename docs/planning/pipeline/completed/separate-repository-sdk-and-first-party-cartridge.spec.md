---
title: Separate-repository OmarchyGS SDK and first-party cartridge
pipeline_id: 10b7eba4-c415-4551-87ff-75084d0f015c
status: Phase 5 — Complete PASS
ticket: TICKET-017
ticket_doc: docs/planning/tickets/closed/TICKET-017-separate-repository-omarchygs-sdk-and-first-party-cartridge.md
aar: docs/planning/knowledge/aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md
created: 2026-08-25
---

# Separate-repository OmarchyGS SDK and first-party cartridge — spec

## Intent

Turn the implemented cartridge contract and previewer into a reproducible,
version-pinned SDK release surface that an isolated game repository can consume
without platform source, database, credentials, or private interfaces. Prove
the boundary with one first-party retro cartridge release, signed provenance,
authoritative lifecycle decisions, and descriptor-relative store operations.

## Scope

- In: all five Ticket 017 EARS requirements; deterministic SDK export and lock;
  exact public schemas/tool compatibility; clean-room game-repository fixture;
  reproducible signed release attestation; platform release verification and
  import; active/deprecated/suspended/revoked/retired launch/session policy;
  Unix descriptor-relative content-addressed store; adversarial pathname and
  authoritative-revocation tests; focused and canonical gate integration;
  architecture, OpenWiki, and AAR reconciliation.
- Out: remote provider gameplay authority, public Internet publication,
  third-party onboarding, marketplace/download services, arbitrary publisher
  code, database schema changes, main-client catalog UI, compiled game-rule
  extraction, Constitution §10 changes, and Git delivery.

## Acceptance criteria (EARS)

The authoritative requirements are REQ-001 through REQ-005 in
[`TICKET-017`](../../tickets/open/TICKET-017-separate-repository-omarchygs-sdk-and-first-party-cartridge.md#ears-requirements).

## Initial decisions to validate in Phase 2

| # | Decision | Reason |
|---|---|---|
| 1 | Publish a deterministic, data-first SDK directory with a canonical lock and exact CLI version instead of exposing platform Rust internals as the SDK. | Keeps the public boundary language-neutral and aligned with the cartridge/renderer protocols. |
| 2 | Prove independence by materializing the first-party source as a fresh temporary Git repository and allowing it only the exported SDK plus installed CLI path. | A monorepo path dependency would not establish portability. |
| 3 | Sign a canonical release attestation that binds source revision, builder/tool and SDK identities, publisher/key, artifact digest, and conformance-report digest. | The platform must be able to verify what produced the exact consumed bytes. |
| 4 | Represent lifecycle state as a signed catalog-authority policy with explicit new-launch and active-session outcomes and no digest substitution. | Publisher signatures authenticate content; platform catalog authority decides whether it may launch. |
| 5 | Add an existing-root Unix store API anchored to directory descriptors for every descendant lookup and mutation; retain same-user path APIs only as explicitly weaker compatibility helpers. | A checked pathname and atomic rename do not contain an attacker who can replace ancestors. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-017-separate-repository-omarchygs-sdk-and-first-party-cartridge.md`
- Architecture: `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md`, `docs/architecture/game-cartridges.md`
- Predecessors: Ticket 015 package/verifier and Ticket 016 trusted renderer.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Active spec/notes/AAR and smallest independent release proof | scope and exclusions fixed |
| 2 Design | SDK/release/provenance/lifecycle/secure-store contracts, file manifest, regression map | CodeGraph receipt and actionable design |
| 3 Implement | Production contracts/tooling, clean-room first-party release, secure import, focused gate | focused loop green |
| 3.5 Inspect | Correctness, supply-chain, signature, filesystem, revocation, and isolation ledger | findings resolved plus fresh CodeGraph receipt |
| 4 Validate | Full matrix and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki, AAR, ticket archive | matching OpenWiki and delivery receipts |
| Delivery | Authorized commit/push only | explicit user authorization |
