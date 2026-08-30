---
aar: AAR-045-provider-starter-conformance-and-second-game
ticket: TICKET-045
pipeline: provider-starter-conformance-and-second-game
status: submitted
opened: 2026-08-30
submitted: 2026-08-30
effectiveness: effective
---

# AAR-045-provider-starter-conformance-and-second-game

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Tickets 018, 019, and 044 notes and AARs | Provider starter, conformance, replay, and clean-clone knowledge search | Yes — supplied the production boundary, pilot behavior, public protocol, and failure controls. |
| Provider binding, dedupe, callback, quota, authority-refresh, deadline, durable-preimage, and native-inventory prevention rules | Knowledge register search | Yes — fixed the security and recovery invariants before design. |
| Public Provider SDK intake, ADR-0003, and affected OpenWiki pages | Roadmap promotion review | Yes — selected the second slice while preserving sidecar/onboarding exclusions. |

## What happened

Ticket 045 turned the public protocol preview into a reusable provider
developer surface. `omarchygs-provider-starter` now owns the exact four-route
Axum/TLS runtime, authentication, compatibility and grant admission, separate
PostgreSQL sessions and one-use grants, stable operation receipts, callback
outbox, and process lifecycle. A narrow `ProviderGame` trait keeps deterministic
launch, command, view, and event logic free of transport, credentials, database
handles, platform identity, and admission authority.

`omarchygs-provider-conformance` adds a fixed fifteen-case TLS/fault runner with
bounded machine-readable receipts and a deterministic signed release that
contains the SDK, starter, and conformance packages. Relay Forge supplied the
second game: two clean Git clones consumed packaged public artifacts only,
produced independent builds, and exercised distinct rules, keys, process, and
PostgreSQL state through both the public conformance CLI and the real platform
broker. Door Legends remains the sole production-admitted provider.

Inspection found two low-risk evidence-boundary defects. The conformance target
did not initially bind URL and socket ports exactly, and callback success could
be observed without binding every semantic fact under test. Both were repaired
and covered by focused negative cases. The complete Codex Security diff scan
reported zero reportable vulnerabilities, the OpenWiki lifecycle completed,
and every canonical local gate stage passed. No hosted publication, external
onboarding, production sidecar transport, or new registration/player route was
introduced.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-conformance-socket-port-identity-gap-001` | A loopback conformance resolver override was not required to use the endpoint's exact port, so an outage probe could still reach the healthy provider. | Live outage-recovery conformance and security inspection. |
| `BF-omarchy-gaming-system-conformance-callback-observation-underbinding-001` | A validly signed callback unrelated to the exercised session could satisfy duplicate-delivery evidence because the sink checked transport identity but not all attested semantics. | Security inspection of callback receipt evidence. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bind-resolver-overrides-to-exact-authority-port-001` | Bind a test-only DNS/socket override to the URL's DNS host, canonical authority, and exact port, and reject IP-literal bypasses. | Loopback classification alone does not prove which authenticated endpoint a conformance request reached. |
| `PR-omarchy-gaming-system-bind-test-observations-to-attested-semantics-001` | A test observation must bind every immutable identity, session, revision, and body fact that its pass result attests, independently of transport authentication. | Valid traffic can otherwise satisfy the wrong semantic test case and create false assurance. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-provider-starter-capability-seam-001` | The public starter owns provider-side protocol, persistence, receipts, callbacks, and lifecycle behind a capability-minimized deterministic game trait; it carries no platform registration, broker, egress, admission, or player-route authority. | `docs/architecture/adr-0003-owner-operated-server-and-extension-boundary.md`; `docs/architecture/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective (5/5). Earlier provider identity, replay-before-revision, durable
preimage, clean-source, and admission-separation rules directly shaped the
starter transaction and public package boundary. Independent CodeGraph and
security inspection exposed two ways the test harness could overstate evidence;
the repaired port identity and callback semantic bindings then passed focused,
live TLS, restart, real-broker, PostgreSQL, clean-clone, and full-gate coverage.
The result demonstrates reusable provider implementation without widening
production trust, while leaving the sidecar/operations threat model as the next
separately reviewable slice.
