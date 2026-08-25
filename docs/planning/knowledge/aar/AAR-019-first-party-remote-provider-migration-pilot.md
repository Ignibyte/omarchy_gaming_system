---
aar: AAR-019-first-party-remote-provider-migration-pilot
ticket: TICKET-019
pipeline: first-party-remote-provider-migration-pilot
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-019-first-party-remote-provider-migration-pilot

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Knowledge register, ADR-0002, and cartridge architecture | Yes — fixes the broker-only network, inert frontend, pairwise identity, platform-envelope, and single-gameplay-owner boundaries. |
| `AD-omarchy-gaming-system-remote-provider-security-foundation-001` | Ticket 018 AAR, implementation, and operator runbook | Yes — supplies the exact registered release, grant/message, egress, replay, quota, lifecycle, and audit controls this pilot must reuse. |
| `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001` | Standing rules and provider protocol | Yes — every operation, event, result, and projection must bind the complete immutable provider context. |
| `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` | Standing rules and current solo/challenge flows | Yes — committed player retries must resolve before mutable pilot/catalog admission, while current revocation remains fail-closed for new work. |
| `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001` | Ticket 018 callback finding | Yes — atomic callback projection must lock the extant session/release root before first receipt insertion. |
| Ticket 017 Door Legends clean-room proof | Completed notes and `examples/first-party-door-legends` | Yes — provides an already proven separate-repository cartridge identity for the first remote-only pilot. |
| Constitution §§10, 14, 15, and 18 | Workflow preflight | Yes — requires the explicit scoped authority amendment, forward-only migration, real separate-process evidence, and independent inspection. |

## What happened

Ticket 019 moved Door Legends v1 across the authority boundary without moving
the trusted frontend or creating a platform shadow engine. Migration 0015
assigns every session exactly one `platform_compiled` or
`registered_provider` authority. Compiled Signal Siege keeps its local object
state and `GameRegistry` path. A Door Legends session pins one immutable
provider release, stores no writable local rules state, and exposes only the
platform-owned envelope, authenticated bounded view, availability, allowlisted
result, achievements, participants, and timestamps.

The optional production provider runtime is all-or-none at startup. When
enabled, it merges the singleton active pilot into the catalog and routes
launch, command, and reconcile through `ProviderBroker` after committing a
durable participant-private envelope and receipt root. Door Legends builds from
a clean clone against the packaged public protocol, runs as a separate TLS
process with its own PostgreSQL database, owns revisions/receipts/outbox, and
delivers signed callbacks. OmarchyGS keeps authentication, pairwise grant
issuance, launch policy, audit, platform projections, lifecycle, and REST/cursor
recovery. Suspension is read-only with explicit reconciliation, restoration
requires reconciliation, and retirement is terminal; there is no compiled
failback.

Independent inspection found five reportable security issues and two additional
correctness/delivery gaps before delivery. Invalid callbacks could consume the
shared authenticated quota; pilot lifecycle was not part of every admission;
provider clients admitted ambient TLS roots and redirects; callback/response
transactions could invert release/session locks; and the independently built
Door Legends sources were absent from the delivery state hash. A separate code
review also found that exact callback replay could be reclassified by current
projection policy. Each issue was fixed and covered by focused regression
evidence before the final gate.

The first full 18-stage gate was honestly red on a warning-denied Clippy borrow
and an old smoke catalog shape. Both were corrected. The rerun passed all
stages, including 44 PostgreSQL server tests, the API/QML smoke, provider
security conformance, and the clean-clone authority/restore drill. OpenWiki
completed after updating the five affected engineering pages; it retained four
pre-existing evidence-debt warnings but returned a complete lifecycle receipt.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-provider-callback-replay-reclassification-001` | An exact authenticated callback replay could be evaluated against already-advanced current policy and conflict with its original accepted/ignored disposition. | Phase 3.5 correctness review and callback replay regression |
| `BF-omarchy-gaming-system-provider-callback-preauth-quota-001` | A caller possessing a release UUID could consume the shared callback quota before proving an authentic provider message. | Codex Security diff scan, medium finding |
| `BF-omarchy-gaming-system-provider-pilot-lifecycle-admission-gap-001` | The first-party pilot lifecycle was absent from general provider admission and was not rechecked inside callback projection. | Codex Security diff scan, medium finding |
| `BF-omarchy-gaming-system-provider-client-trust-expansion-001` | Provider HTTP clients merged ambient roots with registered roots and permitted redirects beyond the exact registered request target. | Codex Security diff scan, two low findings |
| `BF-omarchy-gaming-system-provider-lock-order-inversion-001` | Response and callback effect transactions could acquire release and session roots in inverse order. | Codex Security diff scan, low finding |
| `BF-omarchy-gaming-system-first-party-provider-gate-state-omission-001` | The independently compiled Door Legends provider tree did not contribute to the canonical gated-state hash. | Phase 3.5 delivery-integrity review |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-preserve-first-callback-disposition-001` | Once an authenticated callback identity is durably accepted or ignored, an exact replay must preserve that first disposition instead of being reclassified by mutable current projection policy. | Replay is recovery of an immutable first-delivery decision; re-evaluation can poison deduplication after the original event advanced state. |
| `PR-omarchy-gaming-system-charge-authenticated-quota-after-authentication-001` | Charge a shared authenticated-message quota only after exact signature/context/body verification, then recheck current key, lifecycle, and bounds before committing the charge. | Pre-authentication charging lets unauthenticated traffic exhaust capacity intended for authenticated providers. |
| `PR-omarchy-gaming-system-layer-pilot-lifecycle-into-every-admission-001` | When a narrow activation lifecycle overlays a general provider release, lock and evaluate it at every launch, command, reconcile, event, and projection boundary. | A lifecycle control that protects only discovery or one call path does not contain existing sessions or asynchronous effects. |
| `PR-omarchy-gaming-system-use-one-provider-effect-lock-order-001` | Provider effect transactions acquire release, pilot, and session roots in one documented canonical order before receipts or projections. | Cross-path lock inversion converts otherwise valid concurrent operations into database deadlocks. |
| `PR-omarchy-gaming-system-gate-independent-executable-source-trees-001` | Every source tree that contributes an independently compiled executable or delivery proof must participate in the canonical gated-state hash. | A green receipt is not evidence for code that can change without invalidating it. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-first-party-remote-authority-pilot-001` | Door Legends v1 is the sole operator-enabled registered-provider authority pilot. It owns only its scoped rules/private state/revision/outcome; OmarchyGS owns the trusted cartridge frontend, identity, catalog, session envelope, broker, public projections, audit, lifecycle, and recovery. Every session has exactly one durable gameplay authority, and external providers remain unauthorized. | `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All seven Ticket 019 requirements have executable evidence at their real
boundaries: forward-only schema constraints, optional production startup,
separate TLS process and database, player REST routes, stable replay/revision
semantics, callback projection, lifecycle/reconciliation, privacy allowlists,
and independent backup/restore. The complete gate preserves compiled Signal
Siege and every existing platform regression while making the Door Legends
pilot a mandatory stage. Inspection found and closed meaningful security,
concurrency, replay, and delivery-proof defects before commit. The scope remains
deliberately narrow: no external provider, direct client network, executable
cartridge frontend, or compiled failback was authorized.
