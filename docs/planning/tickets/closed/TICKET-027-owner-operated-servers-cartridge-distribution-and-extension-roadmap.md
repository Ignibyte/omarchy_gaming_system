---
title: TICKET-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap
status: closed
ticket_number: 027
type: architecture
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/owner-operated-servers-cartridge-distribution-and-extension-roadmap.spec.md
---

# TICKET-027 — Owner-operated servers, cartridge distribution, and extension roadmap

## Summary

Define OmarchyGS as an owner-operated community server that acquires vetted,
frontend-only Game Cartridges for its players, while preserving a separately
gated path for administrator-sideloaded content, future game-provider SDKs, and
server-side modules and hooks.

## Why

The portable cartridge and remote-provider foundations exist, but the durable
product documents do not yet describe the intended ownership and distribution
experience: an individual runs the standard OmarchyGS server, chooses its
games, and invites friends to that server. The same documents must distinguish
marketplace trust from an operator's decision to run custom code and must not
let either route weaken the official client's executable-content boundary.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the product describes deployment ownership, it shall treat independently owner-operated OmarchyGS servers as first-class communities using the standard server architecture, with server-local accounts, personas, catalogs, policy, and game history unless a future federation design says otherwise. | Product charter, system architecture, and roadmap review |
| REQ-002 | When an operator acquires a vetted game, the architecture shall define a server-approved marketplace/catalog flow in which clients discover that server's installed games and obtain exact signed inert cartridges for local trusted rendering. | Cartridge architecture and ADR consistency review |
| REQ-003 | When a cartridge is described, the documentation shall define it as frontend presentation data rendered by platform-owned QML components, not raw publisher QML, executable server rules, or an independently networked backend. | Cartridge architecture, Constitution, and ADR consistency review |
| REQ-004 | When future backend portability is planned, the roadmap shall include a public game-provider SDK and starter server that let a game backend implement the brokered OmarchyGS protocol without coupling the core platform to its rules. | Roadmap and provider-boundary review |
| REQ-005 | When an operator bypasses the vetted marketplace, the architecture shall permit explicitly marked server-local cartridge and server-extension installation while preserving client-side inert-content validation, per-server trust disclosure, and operator responsibility. | Threat-boundary and operator-disclosure review |
| REQ-006 | When server extensibility is planned, the roadmap shall include a versioned module base, capability-scoped hooks, lifecycle/compatibility controls, auditability, and an explicit isolation decision without authorizing client plugins or an unstable in-process ABI. | Roadmap and ADR review |

## Scope

- In: product and architecture documentation, a scoped ADR, roadmap ordering,
  owner/operator trust boundaries, marketplace versus sideload semantics,
  future provider SDK and server module/hook outcomes, OpenWiki, and workflow
  evidence.
- Out: marketplace APIs or services, cartridge transfer or mounting code,
  persistent multi-server client profiles, federation, legal terms drafted as
  legal advice, provider self-service, a plugin runtime or ABI, database
  migrations, application behavior, and Git delivery.

## Links

- Intake: none
- Pipeline spec: [completed spec](../../pipeline/completed/owner-operated-servers-cartridge-distribution-and-extension-roadmap.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Operator guidance: [Owner-operated servers](../../../operators/owner-operated-servers.md)
