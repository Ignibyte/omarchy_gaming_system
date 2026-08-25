---
aar: AAR-012-game-registry-and-versioned-sessions
ticket: TICKET-012
pipeline: game-registry-and-versioned-sessions
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-012-game-registry-and-versioned-sessions

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-first-identity-001` | Knowledge-register and product-charter recall | Yes — game surfaces remain persona-facing and central, while boards stay out of this slice. |
| `PR-omarchy-gaming-system-scope-public-cursors-to-resource-001` | Knowledge-register search | Yes — a game-session identifier may be exposed only within participant-authorized session and sync resources. |
| `PR-omarchy-gaming-system-verify-retained-cursor-continuity-001` | Ticket 011 AAR and knowledge register | Yes — reuse the established transaction-coupled persona feed rather than creating a game-global activity cursor. |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Knowledge-register search | Yes — CodeGraph will map the Axum/domain blast radius, while direct registry and PostgreSQL tests remain required. |
| Constitution §10/§14 and system overview | Architecture recall | Yes — compiled games are deterministic server-owned modules with no database access, and future commands must be revision-aware. |

## What happened

OmarchyGS gained the first executable game foundation without advertising a
placeholder game. A new database-free runtime validates compiled manifests,
orders the public catalog deterministically, resolves only an exact rules
version, and bounds deterministic object initialization. Production injects an
honestly empty registry until a playable definition ships; tests inject two
compiled fixture versions.

PostgreSQL now owns immutable game key/version identity, revision-zero active
snapshots, and ordered persona seats. Trusted internal orchestration initializes
and persists a session inside its caller's transaction, locking participants in
canonical UUID order and appending one minimal persona sync invalidation per
participant. Authenticated REST inventory/detail require both an owned acting
persona and durable session membership, expose only public participant profiles,
and read stored versions and snapshots without substituting today's registry.
Public creation and commands remain intentionally absent for the challenge and
revision-command slices.

Inspection found no game-boundary security issue, but the full frozen diff scan
reported two low weaknesses in the shared Codex secret guard. It omitted OpenAI
project/service-account key families, and a dash-prefixed Git filename could be
parsed as a `grep` option. Both were fixed at the shared scanner and covered by
the actual hook self-test. Official Codex documentation and OpenAI source also
resolved the scan's deferred trust question: persisted trust hashes the
normalized hook definition, not referenced script bytes. That separate
transitive-integrity limitation is recorded for a dedicated hardening slice;
hooks remain guardrails and the independently executed worktree gate remains
delivery proof.

The canonical gate passed 29 local server/runtime tests, all 30 migrated
PostgreSQL tests, the live empty-catalog API contract, and the unchanged visible
QML health connector. OpenWiki reconciled five affected pages and completed
with no stale Claims.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-test-fixture-preempted-boundary-001` | A malformed-state fixture attempted object indexing before returning the intended array, so the test scaffold panicked before the runtime boundary could reject it. | Focused runtime unit test. |
| `BF-omarchy-gaming-system-openai-secret-family-omission-001` | The shared changed-file scanner omitted high-signal OpenAI project and service-account key prefixes. | Codex Security frozen-diff scan. |
| `BF-omarchy-gaming-system-secret-path-option-injection-001` | A valid changed root filename beginning with `-` could be parsed as a `grep` option and skipped. | Codex Security synthetic scanner reproduction. |
| `BF-omarchy-gaming-system-transitive-hook-trust-gap-001` | Codex persists trust for the normalized hook definition but does not incorporate bytes of referenced repository scripts into that identity. | Official Codex hooks documentation and OpenAI `hooks/src/engine/discovery.rs` inspection after the offline scan. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-pin-durable-game-rules-version-001` | Persist an exact immutable game key/version with every snapshot and never relabel stored state through the current process registry. | Durable history must remain interpretable after compiled definitions evolve or disappear. |
| `PR-omarchy-gaming-system-build-negative-fixtures-through-boundary-001` | Construct negative fixtures so malformed input reaches the owning validation boundary without the fixture itself coercing, indexing, or panicking first. | A scaffold failure can produce misleading coverage and prevent the intended invariant from being tested. |
| `PR-omarchy-gaming-system-align-secret-families-with-integrations-001` | Reconcile high-signal secret-scanner families with every active repository integration and regression-test the real shared enforcement entrypoint. | Provider drift can create a deterministic false green in both Stop and delivery checks. |
| `PR-omarchy-gaming-system-terminate-options-before-repository-paths-001` | Pass an explicit option terminator before every repository-derived pathname supplied to a command-line parser. | Valid Git names can otherwise alter parsing and bypass a security or quality control. |
| `PR-omarchy-gaming-system-treat-hook-trust-as-transitive-code-trust-001` | After branch or referenced-script changes, treat persisted project-hook trust as a grant over the transitive local code and independently rely on the worktree-bound gate for delivery proof. | Current Codex trust hashes the normalized hook definition rather than every referenced script byte. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-version-pinned-game-session-foundation-001` | Use validated compiled exact-version definitions for deterministic initialization, persist immutable key/version snapshots with persona seats, and expose only an empty-until-playable public catalog plus participant-private reads in this slice. | `docs/architecture/system-overview.md` and `openwiki/runtime-foundation.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. Recalled persona privacy, cursor scope, deterministic server authority, and
graph-coverage rules directly shaped the smallest honest foundation. The design
avoided both a misleading placeholder game and a temporary creation endpoint
that would bypass challenge policy. CodeGraph covered route/domain blast radius;
direct runtime, SQL migration, and PostgreSQL tests covered unsupported edges.
Security inspection caught two inherited workflow bypasses and documented one
separate Codex trust limitation. All seven requirements and both reportable
findings were dispositioned; commands, challenges, playable rules, QML game UI,
the hook-integrity follow-up, and Git delivery remained out of scope.
