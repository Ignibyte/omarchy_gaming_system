---
aar: AAR-041-administrator-custom-server-module-installation-and-provenance
ticket: TICKET-041
pipeline: administrator-custom-server-module-installation-and-provenance
status: submitted
opened: 2026-08-27
submitted: 2026-08-29
effectiveness: effective
---

# AAR-041-administrator-custom-server-module-installation-and-provenance

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-process-isolated-wasm-server-modules-001` | ADR-0004 and Ticket 039 recall | Yes; fixes the no-WASI process, WIT, typed-intent, and core-reauthorization boundary. |
| `AD-omarchy-gaming-system-observation-only-production-server-module-base-001` | Ticket 040 AAR and nearest pipeline | Yes; Ticket 041 must extend custody/provenance without weakening the production observation path. |
| `PR-omarchy-gaming-system-pin-signed-document-authorities-out-of-band-001` | Ticket 039 trust review | Yes; a self-supplied publisher key is an explicit operator trust choice, never marketplace authentication. |
| `PR-omarchy-gaming-system-bind-operator-provenance-to-admitted-server-001` | Ticket 039 cross-document review | Yes; every custom provenance statement and admission must name the current stable server UUID. |
| `PR-omarchy-gaming-system-enforce-artifact-bounds-during-file-read-001` | Ticket 039 runtime review | Yes; component and signed-document bounds apply before allocation and parsing. |
| `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001` | Ticket 040 local-admin review | Yes; every custom mutation input must be owner-held, 0600, no-follow, single-link, bounded, and stable across the read. |
| `PR-omarchy-gaming-system-reauthorize-readiness-under-finalization-lock-001` | Ticket 040 readiness race | Yes; external readiness never authorizes a later lifecycle transition without rechecking all revisions under lock. |
| `PR-omarchy-gaming-system-reconcile-restored-modules-before-server-start-001` | Ticket 040 restore review | Yes; custom modules also remain disabled and review-blocked after raw database restore. |
| OpenWiki server-module and product-boundary pages | Generated evidence recall | Yes; preserves the local-admin, aggregate-disclosure, no-client-code, and no-game-authority limits. |

## What happened

Ticket 041 added the deliberate owner-operated escape hatch above the reviewed
Ticket 040 module base. A database-local administrator can now import an exact
publisher-signed Component Model artifact into immutable PostgreSQL custody,
explicitly acknowledge its unreviewed support boundary, select only a granted
subset of its requested powers, and lifecycle-manage it through revision-
checked enable, disable, suspend, recover, upgrade, one-step rollback, and
terminal evidence-retaining removal. Reviewed and operator-custom releases use
the same no-WASI WIT, isolated host, typed intent, state, dispatcher, receipt,
resource, and core-reauthorization boundaries.

The implementation also made operator-custom behavior visible without creating
an inventory or executable-delivery API. Discovery publishes only a stable-
server-bound count, behavior class, warning, and support boundary; trusted QML
validates, persists, identity-binds, and continuously renders that aggregate.
The server remains available when generic runtime secrets are absent and
records bounded `runtime_unconfigured` evidence rather than claiming execution.

Inspection found that final-component `O_NOFOLLOW` and stable descriptor checks
did not protect an operator-selected artifact from a symlink in an ancestor
directory. The import boundary was strengthened to absolute paths opened with
Linux `openat2` and `RESOLVE_NO_SYMLINKS|RESOLVE_NO_MAGICLINKS`, while retaining
the final no-follow, ownership, mode, link-count, byte-ceiling, and stable-
metadata checks. A nested-parent-symlink regression proves the repaired
boundary. The security diff scan then completed with zero reportable findings,
and the full 24-stage local gate passed.

The first OpenWiki finish was invoked before Phase 4 was durably recorded, so
the completion hook correctly left the prior pipeline receipt in place. The
existing phase-order rule was applied: Phase 4 was recorded, OpenWiki was
rerun cleanly, and readback proved a completion receipt for Ticket 041's exact
pipeline and gated state.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-private-artifact-ancestor-symlink-001` | Final-path no-follow checks did not independently prevent a symlinked ancestor from redirecting a privileged operator-selected custom-module import. | Phase 3.5 private executable-ingress review. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-reject-symlinked-ancestors-for-private-artifact-reads-001` | Resolve owner-selected private artifacts through an OS primitive that rejects symlinked and magic-link ancestors, then retain final no-follow and stable descriptor checks. | Final-inode validation cannot prove that every pathname ancestor remained inside the operator-reviewed directory chain. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-operator-custom-server-module-boundary-001` | Admit operator-custom modules only through bounded database-local immutable custody and explicit custom provenance; share the reviewed runtime authority model while keeping support claims and aggregate player warnings distinct. | `../../../architecture/adr-0004-process-isolated-wasm-server-modules.md`; `../../../architecture/server-modules.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. The prior isolation, authority-separation, descriptor-validation,
readiness-finalization, restore, and fail-open observation rules kept this work
from becoming a remote upload route, path-selected loader, second gameplay
authority, server-secret bridge, or client executable channel. The independent
inspection materially strengthened ancestor-path handling, and the focused
PostgreSQL, real CLI, QML, runtime-containment, security, OpenWiki, and full
local-gate evidence exercised the shipped boundaries rather than only their
types. The repeated OpenWiki phase-order mistake shows that the standing rule
still needs deliberate recall at the Phase 4/5 transition, but receipt readback
caught and corrected it before archival or delivery.
