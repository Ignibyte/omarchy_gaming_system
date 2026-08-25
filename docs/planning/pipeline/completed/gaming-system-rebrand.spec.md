---
title: Gaming system rebrand
pipeline_id: a1d313c9-e799-43f3-ab79-cd6e544c6308
status: Phase 5 — Complete PASS
ticket: TICKET-007
ticket_doc: docs/planning/tickets/closed/TICKET-007-gaming-system-rebrand.md
aar: docs/planning/knowledge/aar/AAR-007-gaming-system-rebrand.md
created: 2026-08-24
---

# Gaming system rebrand — spec

## Intent

Ship a coherent game-first identity as Omarchy Gaming System before further
account security or social/game slices make the old BBS name more expensive to
remove. Preserve only narrow compatibility that protects existing local
configuration and opaque sessions.

## Scope

- In: product positioning, current documentation and UI copy, service/package/
  configuration/token identifiers, local development resources, workflow
  tooling and receipts, regression checks, OpenWiki, and operator guidance.
- Out: new social or game behavior, public boards, 2FA, rewriting migrations or
  historical records, moving the worktree directory, renaming a remote
  repository, and delivery to Git.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a person reads a living product, architecture, workflow, or generated-wiki surface, the system shall identify itself as Omarchy Gaming System and describe games as the primary product, with public boards only as a possible later extension. | Scoped branding scan and documentation review |
| REQ-002 | When the server, QML connector, Cargo tooling, or development scripts expose a current product identifier, the system shall use the `omarchy-gaming-system`/`ogs` namespace and new sessions shall use an `ogs1_` token prefix. | Rust unit/integration tests, QML source review, and live smoke |
| REQ-003 | When an existing local client presents a structurally valid `bbs1_` session token or an operator supplies `BBS_BIND_ADDRESS` without `OGS_BIND_ADDRESS`, the system shall retain compatibility while preferring and documenting the new identifiers. | Focused Rust tests and configuration review |
| REQ-004 | When the local stack and Codex pipeline run after the rebrand, they shall use newly named development resources and receipts without rewriting forward-only migrations or historical evidence. | Compose validation, hook self-tests, pipeline checks, and diff gate |
| REQ-005 | When historical tickets, completed pipelines, AARs, and registered knowledge are inspected, the system shall preserve their original identifiers and claims while all new durable IDs use the gaming-system namespace. | Planning-record review and pipeline structure check |
| REQ-006 | When validation completes, the canonical diff gate shall pass the migrated PostgreSQL, Rust API, and QML path under the new product identity. | `bin/gate.sh --diff` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | The product name is **Omarchy Gaming System**, with `omarchy-gaming-system` as its technical slug and `ogs` as its short internal namespace. | This follows the user-approved game-first direction and separates the product from the existing community-board plugin. |
| 2 | Connections, inboxes, challenges, and server-authoritative games remain the first-playable center; a public message board may be considered later but is not part of this slice or current focus. | Differentiation comes from the product promise, not only its name. |
| 3 | New session tokens use `ogs1_`; structurally valid `bbs1_` tokens remain accepted until their normal revocation or expiry. `OGS_BIND_ADDRESS` takes precedence while `BBS_BIND_ADDRESS` remains a documented fallback. | Rebranding should not silently strand existing local clients or configuration. |
| 4 | Local Compose database, role, and volume identifiers move to the new namespace without editing existing migrations. The prior Docker volume is left untouched and recoverable but is no longer attached by default. | The project is pre-alpha, while forward-only migration history and recoverability remain binding. |
| 5 | Completed tickets, pipeline narratives, AARs, registered knowledge IDs, and migration contents remain historical. Living guides, templates, generated wiki pages, and new IDs move to the new name. | Historical evidence must stay truthful and link-stable. |
| 6 | Repository-directory and remote-hosting renames are delivery/operational actions outside this worktree slice. | No remote delivery has been authorized, and the current workspace root cannot rename itself safely mid-session. |

## Linked artifacts

- Ticket: [TICKET-007](../../tickets/closed/TICKET-007-gaming-system-rebrand.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope and naming locked |
| 2 Design | Architecture, file manifest, compatibility and regression plan | CodeGraph design receipt |
| 3 Implement | Coherent code, UI, docs, and tooling rebrand | focused checks |
| 3.5 Inspect | Findings ledger and post-change blast-radius review | CodeGraph inspection receipt |
| 4 Validate | Tests run and delivery gate green | matching gate receipt |
| 5 Complete | AC audit, OpenWiki, submitted AAR, archive | matching completion receipt |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
