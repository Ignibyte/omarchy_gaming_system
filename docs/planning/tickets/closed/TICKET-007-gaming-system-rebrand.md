---
title: TICKET-007-gaming-system-rebrand
status: closed
ticket_number: 007
type: architecture
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/gaming-system-rebrand.spec.md
---

# TICKET-007-gaming-system-rebrand

## Summary

Rebrand the game-first product and its living technical surfaces from Omarchy
BBS to Omarchy Gaming System while preserving deliberate compatibility for
existing local configuration and session credentials.

## Why

An independently shipped `thoughtlesslabs/omarchy-bbs` plugin already owns the
Omarchy-native community-board identity. This project is instead centered on
connections, private inboxes, challenges, and server-authoritative games, so a
distinct game-first name prevents product and installation confusion before
private alpha.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a person reads a living product, architecture, workflow, or generated-wiki surface, the system shall identify itself as Omarchy Gaming System and describe games as the primary product, with public boards only as a possible later extension. | Scoped branding scan and documentation review |
| REQ-002 | When the server, QML connector, Cargo tooling, or development scripts expose a current product identifier, the system shall use the `omarchy-gaming-system`/`ogs` namespace and new sessions shall use an `ogs1_` token prefix. | Rust unit/integration tests, QML source review, and live smoke |
| REQ-003 | When an existing local client presents a structurally valid `bbs1_` session token or an operator supplies `BBS_BIND_ADDRESS` without `OGS_BIND_ADDRESS`, the system shall retain compatibility while preferring and documenting the new identifiers. | Focused Rust tests and configuration review |
| REQ-004 | When the local stack and Codex pipeline run after the rebrand, they shall use newly named development resources and receipts without rewriting forward-only migrations or historical evidence. | Compose validation, hook self-tests, pipeline checks, and diff gate |
| REQ-005 | When historical tickets, completed pipelines, AARs, and registered knowledge are inspected, the system shall preserve their original identifiers and claims while all new durable IDs use the gaming-system namespace. | Planning-record review and pipeline structure check |
| REQ-006 | When validation completes, the canonical diff gate shall pass the migrated PostgreSQL, Rust API, and QML path under the new product identity. | `bin/gate.sh --diff` |

## Scope

- In: product positioning, living documentation, QML branding, health-service
  identity, Cargo package/log target, configuration namespace, session-token
  issuance prefix, local Compose resources, scripts, Codex hooks/receipts,
  project skills, OpenWiki, and rebrand regression evidence.
- Out: connections, inboxes, games, public message-board implementation, 2FA,
  production deployment, remote repository rename, commit, push, and pull
  request.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/gaming-system-rebrand.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
