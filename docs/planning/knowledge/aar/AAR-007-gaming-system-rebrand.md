---
aar: AAR-007-gaming-system-rebrand
ticket: TICKET-007
pipeline: gaming-system-rebrand
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-007-gaming-system-rebrand

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `docs/product-charter.md` | Local knowledge recall | Yes — already establishes a social-game product and excludes public boards from private alpha. |
| `docs/architecture/system-overview.md` | Architecture recall | Yes — the rename does not alter server authority, REST/WebSocket roles, or account/persona separation. |
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | Knowledge register | Yes — requires the final product identity to be proven through PostgreSQL, API, and QML together. |
| `PR-omarchy-bbs-quality-gates-include-untracked-001` | Knowledge register | Yes — much of the current accumulated work remains untracked and must stay inside validation. |
| `thoughtlesslabs/omarchy-bbs` | User-directed external comparison | Yes — demonstrates direct name/position collision with a shipped community-board plugin. |

## What happened

The project moved from the conflicting Omarchy BBS identity to Omarchy Gaming
System without changing its modular-monolith architecture or pretending planned
games already exist. Living product, architecture, workflow, API, QML, Cargo,
health, Compose, script, hook, and generated-wiki surfaces now use the game-
first name and `omarchy-gaming-system`/`ogs` namespaces. New opaque sessions use
`ogs1_`, while exact legacy `bbs1_` tokens and `BBS_BIND_ADDRESS` retain narrow
transition support. Migrations and historical evidence remain unchanged; the
old Docker volume remains recoverable and detached rather than deleted.

Focused checks, a full security-sensitive diff review, two current-worktree
CodeGraph inspections, the OpenWiki update lifecycle, all eight migrated
PostgreSQL integration tests, and the live account/session/persona/QML smoke
path completed successfully. No reportable security finding survived review.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| — | The first scoped branding scan left one current product sentence in the workflow ADR under the old name because the file also contains a historical `AD-omarchy-bbs-*` identifier that had to remain stable. | Phase 3.5 living-surface branding scan; fixed before validation. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-separate-live-identity-from-history-001` | For product-identity changes, inventory emitted/living identifiers separately from migrations, completed evidence, and registered historical IDs; test every intentional compatibility exception explicitly. | A global rename would falsify history, while an overbroad historical exclusion can leave current product prose or runtime compatibility unverified. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-game-first-identity-001` | Name the product Omarchy Gaming System, keep connections/inboxes/challenges/server-authoritative games at the center, treat public boards only as a possible later complement, and use the `omarchy-gaming-system`/`ogs` runtime namespace with narrow local transition compatibility. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The rebrand remained a bounded identity transition rather than a domain
rewrite. The compatibility design preserved existing local sessions and bind
configuration without weakening entropy, hashing, expiry, revocation, or owner
scoping. The independent scan caught the only living-doc miss. OpenWiki
finished with refreshed claims and navigation, and the final canonical diff
gate passed all 12 tiers against the exact post-wiki gated worktree.
