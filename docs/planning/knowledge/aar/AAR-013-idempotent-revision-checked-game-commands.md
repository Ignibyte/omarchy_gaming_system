---
aar: AAR-013-idempotent-revision-checked-game-commands
ticket: TICKET-013
pipeline: idempotent-revision-checked-game-commands
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-013-idempotent-revision-checked-game-commands

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-pin-durable-game-rules-version-001` | Knowledge-register and Ticket 012 recall | Yes — command execution must resolve the stored exact version and fail rather than substitute. |
| `AD-omarchy-gaming-system-version-pinned-game-session-foundation-001` | Knowledge register and OpenWiki runtime page | Yes — command mutation extends the existing persona seats, snapshot, revision, and sync transaction boundary. |
| `PR-omarchy-gaming-system-scope-public-cursors-to-resource-001` | Knowledge-register search | Yes — idempotency identity and revision remain scoped to one game session rather than exposing global command volume. |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Knowledge-register search | Yes — CodeGraph maps callers and state flow, while receipt semantics and races require direct SQL/test inspection. |
| Constitution §10/§14 and system overview | Architecture recall | Yes — commands are deterministic, idempotent, revision-aware, transaction-coupled, REST-authoritative, and persona-facing. |

## What happened

Ticket 013 added the first durable game mutation boundary. A participant command
now executes through the session's exact compiled game version, advances one
optimistic revision, persists a session-wide idempotency receipt, and appends
minimal participant invalidations in one PostgreSQL transaction. Runtime and
router tests cover bounds, stable rejection, privacy, semantic replay,
collisions, revision conflicts, rollback, and concurrent one-winner behavior.

Transaction inspection caught a timestamp regression risk after lock waiting,
and test inspection found two missing isolation cases. Those were corrected
before validation. The frozen Codex Security diff scan found no command-path
vulnerability, but it reproduced two low-severity workflow weaknesses:
newline-bearing Git paths could bypass enforcement, and the pinned CodeGraph
version did not authenticate the executable artifacts. Both approved fixes now
fail closed and have positive and negative regression evidence.

The canonical diff gate passed the complete local, PostgreSQL, documentation,
hook, and visible QML smoke suite. OpenWiki run
`c456c89a-bb36-4327-bcc2-d170501cc92c` reconciled the runtime, product,
validation, and Codex-workflow pages before the pipeline was archived.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-transaction-start-timestamp-regression-001` | Transaction-start time could regress `game_sessions.updated_at` after lock waiting. | Phase 3.5 transaction inspection. |
| `BF-omarchy-gaming-system-newline-path-enforcement-bypass-001` | Newline-bearing Git paths bypassed delivery hashing and high-signal secret scanning. | Codex Security dynamic reproduction. |
| `BF-omarchy-gaming-system-codegraph-artifact-integrity-gap-001` | CodeGraph exact-version setup lacked repository-reviewed artifact integrity. | Codex Security supply-chain trace. |
| `BF-omarchy-gaming-system-digest-record-encoding-mismatch-001` | The first reviewed CodeGraph tree digest used a different checksum record encoding than the verifier and failed closed. | Remediation setup verification. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-check-replay-before-current-revision-001` | Under a serialized mutation boundary, resolve an idempotency receipt before enforcing the current revision. | A genuine retry carries the original revision after its first attempt has advanced durable state. |
| `PR-omarchy-gaming-system-preserve-monotonic-persisted-timestamps-001` | When a transaction can wait on a lock, derive mutation time after lock acquisition and preserve monotonicity against the stored value. | PostgreSQL transaction-start time can predate a competing transaction's committed timestamp. |
| `PR-omarchy-gaming-system-use-nul-git-path-inventories-001` | Carry repository path inventories as NUL-delimited byte records through enumeration, sorting, hashing, and enforcement. | Newlines and other valid Git path bytes must not split or hide a changed file. |
| `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001` | Authenticate every wrapper and platform artifact before installing a repository tool, then revalidate its installed tree, executable link, and provenance before use. | An exact package version alone does not prove the bytes that will execute. |
| `PR-omarchy-gaming-system-derive-digests-with-verifier-encoding-001` | Derive reviewed aggregate digests with the exact byte-record encoding used by the verifier. | Equivalent filenames and checksums serialized differently produce different aggregate digests and a fail-closed setup rejection. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-idempotent-revision-command-boundary-001` | Participant commands use REST as the durable boundary; a locked session resolves replay before revision, executes the exact compiled version, and commits snapshot, revision, receipt, and minimal sync invalidations atomically. WebSockets remain hints. | `../../architecture/system-overview.md`; `../../../openwiki/runtime-foundation.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The pipeline produced the intended command slice, caught and repaired one
transaction correctness issue and two workflow security issues, proved the
race and retry contracts against PostgreSQL, reconciled durable documentation,
and completed without silently dropping any Ticket 013 requirement.
