---
title: TICKET-044-public-provider-sdk-contract-negotiation-and-release
status: closed
ticket_number: 044
type: feature
created: 2026-08-30
closed: 2026-08-30
intake: docs/planning/intake/public-provider-sdk-starter-and-sidecar.md
pipeline_spec: docs/planning/pipeline/completed/public-provider-sdk-contract-negotiation-and-release.spec.md
---

# TICKET-044-public-provider-sdk-contract-negotiation-and-release

## Summary

Extract the provider-facing contract into a public-only OmarchyGS Provider SDK,
add authenticated exact-v1 capability negotiation before provider effects, and
produce deterministic locally signed SDK exports with fresh-repository proof.

## Why

The current protocol is proven by Door Legends but is packaged inside the
platform implementation, treats v1 compatibility as implicit, and has no
standalone release identity. This is the first independently shippable slice
of the public Provider SDK roadmap item and leaves starter, conformance-kit,
sidecar, and external-onboarding work separately gated.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the Provider SDK is exported, it shall contain only documented provider-facing protocol, model, helper, schema, fixture, notice, and provenance files and shall exclude platform registry, broker, egress, database, administrator, secret, and repository-relative surfaces. | Exact export inventory, forbidden-surface scan, packaged dependency tree, and fresh-repository build. |
| REQ-002 | When the platform and a provider establish compatibility, they shall authenticate and bind one exact supported protocol version and the required capability set before launch, command, reconcile, or callback effects; missing, unknown, ambiguous, stripped, or downgrade-selected compatibility shall fail closed. | Negotiation unit matrix, grant/message binding tests, hostile vectors, and real broker/provider exercise. |
| REQ-003 | When an SDK consumer verifies a grant or signed HTTP message, public helpers shall bind the provider, release, game, rules, cartridge, session, pairwise subject, scope, expiry, replay, request context, negotiated compatibility, and exact bytes before returning parsed payloads. | Public API unit/interop tests and mismatch, stale/future, replay, digest, signature, schema, depth, value, and body-bound cases. |
| REQ-004 | When one reviewed SDK revision is exported twice or consumed from two fresh repositories, files, manifests, schemas, fixtures, checksums, release provenance, and builds shall be identical or verify against the same exact SDK identity without an OmarchyGS path dependency. | Deterministic export comparison, local release signature verification, two clean Git clones, packaged-crate builds, and source-path leak scan. |
| REQ-005 | When the SDK slice is delivered, existing compiled games and the Door Legends provider pilot shall retain their authority behavior, while no public route or SDK operation shall register, activate, discover, trust, or list an external provider. | Workspace/provider/server regressions, route and dependency diff, operator-boundary review, and canonical local gate. |

## Scope

- In:
  - dedicated public provider SDK crate and deterministic export/verification;
  - exact v1 protocol and required capability negotiation;
  - compatibility binding in grants, operations, responses, and events;
  - platform-crate compatibility re-exports and Door Legends SDK consumption;
  - signed local release provenance and two-clean-repository proof.
- Out:
  - starter backend abstraction, portable conformance CLI/fault kit, or second game;
  - co-located sidecar transport or deployment/operations guide;
  - external provider registration, approval, activation, discovery, or support;
  - crates.io, marketplace, package-registry, or hosted CI/CD publication.

## Links

- Intake: [Public Provider SDK](../../intake/public-provider-sdk-starter-and-sidecar.md)
- Pipeline spec: [public-provider-sdk-contract-negotiation-and-release.spec.md](../../pipeline/completed/public-provider-sdk-contract-negotiation-and-release.spec.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
