---
title: TICKET-014-portable-games-sdk-and-remote-hosting-spike
status: closed
ticket_number: 014
type: spike
created: 2026-08-24
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/portable-games-sdk-and-remote-hosting-spike.spec.md
---

# TICKET-014-portable-games-sdk-and-remote-hosting-spike

## Summary

Determine a secure, portable game integration architecture in which OmarchyGS
owns platform identity, personas, avatars, social features, achievements,
discovery, and launch policy while separately versioned game providers can own
their server-side rules and gameplay state. Prove the smallest useful boundary
and decide how a game's frontend can safely participate in the OmarchyGS
experience.

## Why

The current compiled Rust runtime correctly serves the private-alpha trust
model, but it assumes the OmarchyGS process owns all gameplay state and rules.
The intended long-term product needs old BBS-inspired games and later
third-party games to live in separate repositories, target an OmarchyGS SDK,
and potentially execute on remote provider servers. Settling the authority,
credential, protocol, packaging, and frontend boundaries now avoids coupling
the challenge flow and first game to an architecture that cannot evolve safely.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the spike evaluates portable execution models, it shall compare the current compiled-in runtime, a separately versioned compiled first-party game, a sandboxed local package, and a remote game service across authority, isolation, deployability, latency, offline behavior, compatibility, and operations, then recommend a staged target architecture. | Reviewed decision matrix and recommendation |
| REQ-002 | When the spike models a remote game launch, it shall define a trust bootstrap and authorization flow using a registered game identity and short-lived audience-bound, persona-scoped grants that disclose no account ownership, reusable device token, credential, or direct database access, including key rotation, revocation, capability scopes, and endpoint allowlisting. | Protocol sequence, claim schema, and threat-model review |
| REQ-003 | When a remote provider owns gameplay, the proposed contract shall assign durable authority for state, commands, revisions, idempotency, time, randomness, results, achievement claims, callbacks, retries, outages, and reconciliation without treating WebSockets as durable truth. | Authority matrix, failure-mode analysis, and protocol contract |
| REQ-004 | When the spike specifies frontend delivery, it shall make a signed immutable OmarchyGS Game Cartridge rendered by trusted platform components the baseline, prohibit raw game-supplied QML/JavaScript/native execution, and compare sandboxed executable or provider-hosted web escape hatches across keyboard/accessibility behavior, isolation, bridge permissions, version pinning, updates, and malicious-provider containment. | Cartridge contract, frontend option matrix, and recommended launch flow |
| REQ-005 | When the spike defines the OmarchyGS SDK boundary, it shall specify separate-repository manifests, schemas, capability negotiation, compatibility/version policy, generated or hand-written adapters, conformance tests, artifact provenance, local development, registration, rollout, suspension, and retirement for both first-party and future third-party games. | SDK/package lifecycle proposal and conformance plan |
| REQ-006 | When the preferred boundary is selected, the spike shall exercise it with an isolated cross-process proof that launches a persona into a fixture game, exchanges one revision-aware idempotent command, and returns one authenticated result or platform event without exposing platform credentials or database access. | Runnable proof, captured commands, and negative security checks |
| REQ-007 | When the spike completes, it shall record an ADR that reconciles the recommendation with the current server-authoritative constitution, identifies every required future amendment and migration seam, and opens a sequenced set of implementation tickets without enabling production remote access in this spike. | ADR, current-code gap map, and linked follow-up tickets |
| REQ-008 | When a cartridge declares presentation capabilities, OmarchyGS shall define core, rich-2D, advanced, and future 3D/web profiles with explicit compatibility and fallback behavior plus measured limits for archive expansion, decoded assets, scene complexity, animation/effects, audio, payloads, memory, and frame time. | Graphics capability matrix, profiled proof, and conformance-limit plan |

## Scope

- In: current runtime and API seam analysis; control-plane versus game-provider
  authority; first-party games in separate repositories; remote provider
  identity and scoped session grants; command/result/event reliability;
  achievement claim trust; frontend hosting and sandbox options; SDK manifest,
  versioning, conformance, provenance, registration, suspension, and retirement;
  Game Cartridge format and graphics profiles; an isolated cross-process proof;
  an ADR; and sequenced follow-up tickets.
- Out: a production SDK release, a public developer program or marketplace,
  billing, federation, production remote-provider access, arbitrary native
  plugin loading, a production browser client, durable schema migration,
  changing the constitution inside this spike, porting a specific BBS game, or
  changing the current compiled runtime before the recommendation is accepted.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/portable-games-sdk-and-remote-hosting-spike.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Cartridge proposal: [Game Cartridges](../../../architecture/game-cartridges.md)
