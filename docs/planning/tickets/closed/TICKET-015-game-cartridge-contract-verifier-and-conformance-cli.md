---
title: TICKET-015-game-cartridge-contract-verifier-and-conformance-cli
status: closed
ticket_number: 015
type: feature
created: 2026-08-24
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/game-cartridge-contract-verifier-and-conformance-cli.spec.md
---

# TICKET-015-game-cartridge-contract-verifier-and-conformance-cli

## Summary

Turn the Ticket 014 data-only proof into a versioned local cartridge contract,
deterministic pack/verify tooling, and conformance CLI without enabling remote
providers or publisher code execution.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a game repository builds a cartridge, the tool shall emit a deterministic canonical archive and signed integrity identity covering every manifest, presentation, schema, localization, and declared asset byte. | Reproducibility and signature tests |
| REQ-002 | When OmarchyGS verifies an untrusted cartridge, it shall reject traversal, links, duplicate/colliding names, unknown file types, executable content, malformed schemas, unsupported required capabilities, invalid signatures, and every compressed/expanded resource-limit violation before installation. | Negative corpus and archive-limit tests |
| REQ-003 | When a cartridge declares SDK, protocol, required, or optional capability ranges, the verifier shall return a stable compatibility result and require a declared fallback for every optional visual/audio capability. | Version/capability matrix tests |
| REQ-004 | When the conformance CLI completes, it shall produce a machine-readable provenance report without installing a cartridge, contacting a provider, reading platform credentials, or requiring the OmarchyGS database. | CLI integration and isolation tests |
| REQ-005 | When a verified cartridge is installed locally, OmarchyGS shall use a content-addressed read-only location and atomic activation/revocation metadata without adding executable permission. | Filesystem lifecycle tests |

## Scope

- In: package/schema versioning, canonical archive, signature/key identity,
  strict parser, resource limits, capability negotiation/fallbacks, negative
  corpus, conformance report, local content-addressed install, docs, and gate.
- Out: QML rendering, production publisher onboarding, network download,
  remote providers, marketplace policy, custom code/shaders, and Git delivery.

## Links

- Pipeline: [completed spec](../../pipeline/completed/game-cartridge-contract-verifier-and-conformance-cli.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md)
- Contract: [Game Cartridges](../../../architecture/game-cartridges.md)
