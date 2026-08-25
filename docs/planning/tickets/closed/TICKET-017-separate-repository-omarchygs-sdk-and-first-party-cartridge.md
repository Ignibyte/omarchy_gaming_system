---
title: TICKET-017-separate-repository-omarchygs-sdk-and-first-party-cartridge
status: closed
ticket_number: 017
type: infrastructure
created: 2026-08-24
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/separate-repository-sdk-and-first-party-cartridge.spec.md
---

# TICKET-017-separate-repository-omarchygs-sdk-and-first-party-cartridge

## Summary

Publish the cartridge schemas/tooling as a versioned OmarchyGS SDK surface and
prove independent game development by consuming one first-party cartridge from
a separate repository through reproducible artifacts and conformance evidence.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the SDK is released, a game repository shall pin exact schema/tool versions, build and verify without the platform database, and receive documented compatibility and retirement policy. | Clean-room repository CI proof |
| REQ-002 | When a first-party game publishes a cartridge, provenance shall bind its source revision, builder/tool versions, publisher key, exact artifact digest, and conformance report. | Reproducible release attestation |
| REQ-003 | When OmarchyGS consumes the artifact, it shall use the same verifier and public contract intended for future publishers, with no private filesystem, database, credential, or source-tree integration. | Platform consumption integration test |
| REQ-004 | When an SDK or cartridge version is deprecated, suspended, revoked, or retired, the catalog and installed-artifact lifecycle shall follow an explicit new-launch and active-session policy without silent substitution. | Lifecycle matrix tests |
| REQ-005 | When cartridge import or installation runs with privileges or against a store root mutable by another principal, every lookup and mutation shall use descriptor-relative containment or an equivalent OS sandbox and shall consult authoritative revocation state without a pathname race or fail-open error. | Adversarial multi-user filesystem and revocation tests |

## Scope

- In: independent SDK artifact, documentation/fixtures, generated or
  hand-written adapters, clean-room CI, first-party repository release,
  provenance, platform import, privilege-aware store containment,
  suspension/retirement policy, and docs.
- Out: third-party onboarding, remote gameplay authority, marketplace,
  automatic Internet downloads, arbitrary plugins, and Git delivery.

## Links

- Depends on: `TICKET-015`, `TICKET-016`
- Pipeline: [completed spec](../../pipeline/completed/separate-repository-sdk-and-first-party-cartridge.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md)
