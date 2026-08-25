---
aar: AAR-021-signal-siege-compiled-game-and-solo-bot-matches
ticket: TICKET-021
pipeline: signal-siege-compiled-game-and-solo-bot-matches
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-021-signal-siege-compiled-game-and-solo-bot-matches

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-version-pinned-game-session-foundation-001` | Knowledge register, Ticket 012 AAR, and runtime code | Yes — Signal Siege must be an immutable compiled version and retained snapshots cannot depend on today's registry. |
| `AD-omarchy-gaming-system-idempotent-revision-command-boundary-001` | Knowledge register, Ticket 013 AAR, and command transaction | Yes — each human/bot round belongs in the existing snapshot/revision/receipt/sync transaction. |
| `PR-omarchy-gaming-system-check-replay-before-current-revision-001` | Standing rule and command implementation | Yes — the final command must replay after its first execution completes the session. |
| `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` | Ticket 020 AAR and standing rules | Yes — a committed solo-start retry must survive registry or active-cap drift. |
| Product charter and Constitution §10 | Product and architecture preflight | Yes — the first game must be original, asynchronous, deterministic, server-authoritative, and persona-facing without a bot account. |
| `BUL-001-initial-push-pending` | Bulletin preflight | Yes — it was resolved and archived after remote `main` was created and verified before Ticket 021 opened. |

## What happened

Ticket 021 turned the exact-version game foundation into the first production
playable server loop. Signal Siege v1 is a dedicated database-free Rust rules
crate: one human chooses strike, guard, or charge; the server-owned bot chooses
only from the durable pre-command state; both actions resolve simultaneously;
and core destruction or round 12 produces an explicit bounded outcome. The
runtime now returns a typed active/completed transition separately from game
JSON.

An authenticated account can launch an exact one-human definition for an owned
persona. The persona-root transaction checks an immutable durable receipt
before current registry and active-cap admission, enforces at most 25 active
solo starts, creates only the human seat, and commits the session, receipt, and
minimal sync invalidation together. No bot account, persona, credential, or
participant row exists. The command transaction resolves final replay before
lifecycle/revision checks and atomically stores state, revision, lifecycle,
completion time, receipt status, and invalidation. Completed sessions remain
participant-private list/detail history without consulting today's registry.

Inspection found one low-severity developer-smoke command-evaluation path: a
spoofed local responder could place strings into Bash arithmetic. Readiness is
now bound to the spawned server PID and its listening log, and JSON values must
be bounded integers before arithmetic. Independent cleanup also made Signal
Siege reject cross-field-inconsistent stored snapshots. Five rule tests, four
Signal Siege PostgreSQL scenarios, the complete 43-test database suite, the
live launch/play/completion/QML smoke, CodeGraph inspection, Codex Security
scan, OpenWiki lifecycle, and the canonical gate all passed.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-smoke-json-arithmetic-injection-001` | The live helper accepted JSON strings into Bash arithmetic and could be driven by an unrelated local responder bound to the configured port. | Codex Security diff scan and dynamic validation |
| `BF-omarchy-gaming-system-game-state-lifecycle-consistency-gap-001` | Initial Signal Siege parsing enforced structure and scalar bounds but not every relationship among phase, round, core, last-round evidence, and outcome. | Phase 3.5 skeptical game-state inspection |
| `BF-omarchy-gaming-system-openwiki-phase-receipt-sequencing-001` | The first successful OpenWiki finish did not write Ticket 021's completion receipt because the durable spec had not yet advanced from Phase 3.5 to Phase 4. | Completion-receipt readback |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-validate-shell-arithmetic-input-001` | Treat process-local HTTP as untrusted input: bind readiness to the intended child and require explicit numeric type/range before shell arithmetic. | Local port spoofing or malformed responses must fail closed without command evaluation. |
| `PR-omarchy-gaming-system-validate-game-state-cross-field-invariants-001` | At the compiled rules boundary, validate lifecycle relationships as well as JSON shape and scalar ranges before applying a command. | Persistence normally emits canonical state, but corrupt or adversarial snapshots must not become legal transitions. |
| `PR-omarchy-gaming-system-advance-durable-phase-before-phase-tools-001` | Record the completed validation phase in the active spec before invoking a phase-gated completion tool, then read back the tool receipt. | Hook evidence is intentionally keyed to the durable phase rather than chat history. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-signal-siege-solo-game-lifecycle-001` | Production registers Signal Siege v1 as a one-human deterministic compiled game; owner-scoped solo receipts and typed active/completed transitions provide its public launch, replay, and retained-history lifecycle without a bot identity. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. Every Ticket 021 requirement has direct focused evidence and integrated
PostgreSQL/live-smoke proof. The design reused the exact-version runtime,
participant-private resource, receipt-before-admission rule, and cursor
invalidation model without adding a bot principal, worker, random source, or
second session engine. Independent security and state inspection found two
real low-severity issues before delivery, both were fixed and regression
tested. The only workflow correction was receipt sequencing; its readback made
the mismatch immediately visible and the clean second OpenWiki lifecycle
issued the matching Ticket 021 receipt.
