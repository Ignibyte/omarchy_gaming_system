---
title: INTAKE-public-provider-sdk-starter-and-sidecar
status: promoted
created: 2026-08-30
ticket: TICKET-044 (delivered slice); next candidate TICKET-045
pipeline_spec: docs/planning/pipeline/completed/public-provider-sdk-contract-negotiation-and-release.spec.md
---

# INTAKE-public-provider-sdk-starter-and-sidecar

## Problem or opportunity

The production `omarchy-game-provider` crate, fixed v1 protocol, separately
built Door Legends provider, TLS broker, and conformance suite prove the
backend authority seam. They are still project-internal proof surfaces rather
than a supported public developer product: compatibility is a fixed implicit
v1 contract, the example is game-specific, fault fixtures are gate-oriented,
and no reviewed co-located deployment profile exists.

Publishing only the existing crate would expose platform registry/broker
internals without delivering a stable starter, version negotiation, portable
conformance interface, release provenance, or safe operations story. The SDK
must package the provider-facing contract while keeping registration,
admission, player routes, and platform authority inside OmarchyGS.

## Proposed outcome

An independently consumable, reproducibly exported OmarchyGS Provider SDK
offers versioned provider-facing models and authenticated protocol helpers, a
game-agnostic starter backend with separate durable state, public conformance
and fault fixtures, an explicit negotiated compatibility contract, and remote
plus co-located sidecar deployment guidance. Two clean external source trees
build identical SDK-dependent providers without a path dependency or access to
platform credentials/database state. Publishing the SDK authorizes no provider
release by itself.

## Candidate delivery sequence

This roadmap item is likely safest as three consecutive tickets:

1. [delivered by Ticket 044] extract and version the provider-facing SDK plus
   explicit negotiation and deterministic export/release provenance;
2. build the game-agnostic starter backend and public conformance/fault kit,
   then prove a second clean-room game integration;
3. design, implement, and drill the authenticated co-located sidecar deployment
   profile and complete the deployment/operations guide.

Each ticket must leave external-provider admission disabled.

## Candidate EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the Provider SDK is exported, it shall contain only documented provider-facing protocol/model/helper sources, licenses, schemas, fixtures, templates, and provenance; it shall exclude platform registry, broker, egress policy, database migrations, administrator authority, private keys, and repository-relative dependencies. | Deterministic export manifest, public-only source scan, clean repository build, and forbidden-surface assertions. |
| REQ-002 | When platform and provider establish compatibility, both shall authenticate and bind one explicit supported protocol version and capability set before launch, command, reconciliation, or callback traffic; unknown, ambiguous, missing, or downgrade-selected versions shall fail closed without game mutation. | Negotiation unit/interop matrix, signed-context tests, downgrade/stripping cases, and real broker-provider exercise. |
| REQ-003 | When an SDK consumer handles a grant or signed HTTP message, public helpers shall verify the existing provider/release/game/rules/cartridge/session/subject/scope/expiry/replay and originating-request bindings over exact bytes before exposing parsed payloads. | Public API tests and hostile vectors for mismatches, stale/future time, replay, signature, digest, schema, body bounds, and request/response context. |
| REQ-004 | When a starter backend receives launch, command, or reconcile operations, it shall demonstrate separate durable gameplay state, expected revisions, whole-operation idempotency, stable timeout retry receipts, authenticated callbacks, bounded payloads, and no compiled-platform failback. | Starter PostgreSQL integration tests plus complete broker/callback/outage/restart/reconciliation exercise. |
| REQ-005 | When developers implement game rules, the starter abstraction shall keep game-specific deterministic state transitions separate from transport, grant/signature handling, persistence, callback delivery, configuration, and process lifecycle. | Example game implementation review, unit tests without network/database, and alternate-game substitution test. |
| REQ-006 | When the public conformance kit runs against a provider endpoint, it shall exercise valid launch/command/reconcile/callback flow and the published fault corpus for replay, changed intent, stale revision, timeout/unknown outcome, outage/restart, signature/digest/context mismatch, oversized/malformed bodies, callback deduplication, and recovery. | Standalone conformance CLI/library results against the starter and clean-room provider, with bounded machine-readable receipts. |
| REQ-007 | When SDK artifacts are produced twice from one reviewed revision, their bytes, manifests, documentation, schemas, fixtures, checksums, and provenance shall be identical and verifiable from fresh repositories without access to the OmarchyGS source tree. | Two clean exports/builds, byte comparison, source-revision binding, SBOM/license review, and public-only dependency scan. |
| REQ-008 | When a second clean-room backend consumes the SDK, it shall implement a game distinct from Door Legends, use its own PostgreSQL database and keys, pass public conformance, and integrate through the real OmarchyGS broker without platform path dependencies or private integration hooks. | Two fresh clones/builds, exact release comparison, real TLS/broker run, database separation check, and forbidden-import scan. |
| REQ-009 | When a provider is deployed remotely, templates and guidance shall cover TLS identity, DNS/endpoint immutability, separate database, least-privilege secrets, key rotation, quotas, health/monitoring, backup/restore, suspension/revocation, incident response, upgrades, and end-of-life. | Template validation, operator walkthrough, restore/rotation/suspension drills, and documentation audit. |
| REQ-010 | When an operator selects the co-located profile, provider traffic shall use one explicitly configured authenticated local transport that retains exact provider/release identity, signatures, scopes, quotas, deadlines, audit, separate process/state/credentials, and lifecycle controls without a general loopback/private-network exemption. | Separate threat model, platform/provider configuration tests, hostile socket/path/peer cases, process containment review, and end-to-end sidecar drill. |
| REQ-011 | When the sidecar starts, stops, crashes, upgrades, or loses its database, OmarchyGS shall retain its platform authority, deny new affected launches, keep existing provider sessions read-only as policy requires, and recover only through authenticated reconciliation—never shared state or compiled fallback. | Service lifecycle, outage, restart, upgrade, backup/restore, and reconciliation tests. |
| REQ-012 | When SDK docs describe identity or data flow, they shall state that providers receive only pairwise game-scoped subjects and grants, never account identity, reusable device credentials, platform database access, arbitrary egress, client executable privilege, or direct client connectivity. | Documentation contract assertions, serialized traffic scan, and clean-room integration review. |
| REQ-013 | When the SDK is published, the server shall not automatically register, activate, discover, trust, or list any external provider; exact operator registration and a later reviewed-onboarding pipeline shall remain mandatory. | Public API/discovery diff review, negative server tests, and operator-boundary inspection. |
| REQ-014 | When this roadmap item completes, the SDK, starter, negotiation, public conformance kit, second clean-room integration, remote deployment guidance, and reviewed sidecar profile shall all have delivery receipts; unresolved external onboarding policy shall remain a separate unchecked item. | Per-ticket requirement audits, release manifests, canonical gates, OpenWiki/AAR evidence, and narrowly scoped roadmap change. |

## Scope notes

- In:
  - public provider-facing model/protocol packages and deterministic export;
  - explicit authenticated protocol/capability negotiation;
  - signing/grant/message helpers with safe secret boundaries;
  - game-agnostic starter service, separate persistence, callbacks, and public
    conformance/fault fixtures;
  - a second clean-room backend proof;
  - remote deployment and an explicitly authenticated co-located sidecar
    profile with operational drills.
- Out:
  - external/self-service provider approval or public registration routes;
  - direct client-provider networking, provider-hosted UI, executable
    cartridges, account/persona identity disclosure, shared platform database,
    or compiled gameplay fallback;
  - exporting platform registry/broker/administrator implementations as the
    developer SDK;
  - reusing the conformance-only loopback socket override in production;
  - general server modules or hooks as a substitute for game authority.

## Promotion checklist

- [x] Ticket 042 delivery explicitly authorized and completed.
- [x] SDK package/repository and release-signing ownership approved for the
  project-owned locally signed preview export; no hosted publication implied.
- [x] Protocol-version compatibility window and negotiation policy approved as
  exact v1 with launch, command, reconcile, and event capabilities required.
- [ ] Second clean-room game scope selected.
- [ ] Co-located transport profile receives a dedicated threat-model decision.
- [x] Ticket sequence is promoted one shippable slice at a time; Ticket 044 is
  completed and later slices remain unopened until its delivery readback.
- [x] First promoted ticket's pipeline spec/notes pair completed and archived.
- [x] `ticket:` and `pipeline_spec:` point to the delivered slice.
- [x] Status changed to `promoted` only for the Ticket 044 slice; remaining
  candidate sequence items retain their unchecked decisions.
