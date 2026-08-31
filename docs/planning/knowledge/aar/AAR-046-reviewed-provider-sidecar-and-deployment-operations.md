---
aar: AAR-046-reviewed-provider-sidecar-and-deployment-operations
ticket: TICKET-046
pipeline: reviewed-provider-sidecar-and-deployment-operations
status: submitted
opened: 2026-08-30
submitted: 2026-08-30
effectiveness: effective
---

# AAR-046-reviewed-provider-sidecar-and-deployment-operations

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Tickets 018, 019, 044, and 045 notes and AARs | Provider sidecar, egress, lifecycle, replay, and operations knowledge search | Yes — supplied the exact authority, transport, failure, and evidence boundaries. |
| `PR-omarchy-gaming-system-bind-resolver-overrides-to-exact-authority-port-001` | Ticket 045 callback/transport inspection | Yes — prevents a co-located mapping from becoming an under-bound loopback bypass. |
| Public Provider SDK intake, ADR-0003, architecture guide, and OpenWiki | Roadmap promotion review | Yes — requires a separate threat model, independent state/process/credentials, and no external admission. |

## What happened

Ticket 046 completed the locally actionable Provider SDK roadmap. Production
can now map one exact registered provider release to one exact TLS-over-loopback
socket without changing the canonical HTTPS URL, DNS/SNI/Host identity,
registered roots, signed messages, grants, exact-v1 negotiation, quotas,
lifecycle, replay, or audit. Server configuration treats the release/socket
pair as all-or-none, and provider callbacks have a separately named exact
sidecar mode that ignores ambient proxies. The reviewed templates keep the
provider in a separate service identity, PostgreSQL role/database, private
configuration and state, backup, resource, network, and lifecycle boundary.

The focused drill builds and runs a clean-room provider, rejects a hostile TLS
peer on the configured local port, denies work during crash, restarts and
reconciles, recovers callbacks, restores the provider database independently,
checks the service/config/Caddy templates, and signs a bounded receipt that is
free of credentials and database URLs. The operator runbook covers both remote
and co-located TLS identity, immutable registration, secrets, limits,
monitoring, rotation, suspension, incident response, restore, upgrade, and EOL.

Security inspection found an enabled Caddy administration listener and an
ambient-proxy path in Door Legends callback delivery. Both were removed. A
correctness review then found that readiness checking did not linearize command
or reconciliation across external I/O. Migration 0029 now retains a bounded
operation reservation, while a transaction-scoped PostgreSQL advisory fence
spans provider transport and a reservation UUID fences projection. An
independent patch review caught two follow-on defects: visible expiry could
reclaim a still-live operation, and failure cleanup could overwrite a newer
operator suspension or retirement. Post-lock revalidation and lifecycle-safe
cleanup fixed both.

The sealed Codex Security scan reported one medium and one low finding, both
fixed. Focused server, sidecar, starter, authority-pilot, hostile-proxy,
concurrency, lifecycle, and restore evidence passed. The complete local diff
gate also passed every stage once after implementation and security fixes;
closeout documentation intentionally invalidated that receipt, so authorized
delivery uses a final matching gate after the archive is complete. OpenWiki was
reconciled, and external onboarding remains visibly separate and unauthorized.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-provider-sidecar-caddy-admin-listener-001` | The reviewed callback-proxy template retained Caddy's default mutable loopback administration API, which the separately sandboxed provider user could reach. | Codex Security diff scan and deployment-template review. |
| `BF-omarchy-gaming-system-provider-callback-ambient-proxy-001` | Door Legends callback delivery could consult ambient HTTP(S)/all-proxy configuration before applying its exact loopback socket mapping. | Codex Security diff scan and hostile-proxy authority pilot. |
| `BF-omarchy-gaming-system-provider-operation-admission-transport-race-001` | Command readiness was checked before waiting for external provider execution, so a queued command could outlive another operation's transition into recovery. | Correctness/security-policy inspection and concurrent command/reconcile proof. |
| `BF-omarchy-gaming-system-provider-operation-live-expiry-reclaim-001` | A visible durable reservation could expire while its original platform process was still executing, allowing another process to reclaim the session. | Independent patch review and forced-expiry live-operation test. |
| `BF-omarchy-gaming-system-provider-operation-failure-lifecycle-overwrite-001` | Delayed failure cleanup could replace a newer operator `suspended` or `retired` availability with a transport recovery state. | Independent patch review and suspension-versus-failure race test. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-treat-local-sockets-as-routing-not-authentication-001` | A co-located transport may replace only one exact registered release's socket destination; retain the canonical HTTPS/TLS/message identity and reject broad private-network exceptions. | Loopback port ownership is availability and routing evidence, never provider authentication. |
| `PR-omarchy-gaming-system-linearize-external-effects-with-reservation-and-live-fence-001` | When authorization spans external I/O, pair a durable crash-recovery reservation with a process-held database fence from final admission through transport and response revalidation. | A durable expiry alone cannot distinguish an abandoned operation from a slow live process. |
| `PR-omarchy-gaming-system-preserve-newer-lifecycle-during-async-cleanup-001` | Asynchronous response and failure cleanup may clear only its exact operation identity and must preserve newer terminal or operator-controlled lifecycle state. | An older request must not revive or weaken a later suspension or retirement. |
| `PR-omarchy-gaming-system-disable-ambient-sidecar-management-and-proxy-planes-001` | Sidecar service and callback-proxy templates must explicitly disable ambient proxy selection and mutable administration listeners unless a separately reviewed control plane requires them. | Co-location otherwise introduces hidden local authorities outside the signed protocol and service boundary. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-exact-release-tls-sidecar-profile-001` | The production co-located provider profile maps one exact registered release to one exact loopback TCP socket while retaining the registered DNS HTTPS identity and every existing protocol and lifecycle control. | `docs/architecture/adr-0003-owner-operated-server-and-extension-boundary.md`; `docs/architecture/game-cartridges.md`; `docs/security/provider-sidecar-threat-model.md` |
| `AD-omarchy-gaming-system-provider-session-operation-fence-001` | Provider command and reconciliation use a durable bounded session reservation plus a transaction-scoped PostgreSQL advisory fence held across transport, followed by exact reservation revalidation before projection. | `docs/architecture/game-cartridges.md`; `docs/security/provider-sidecar-threat-model.md`; migration `0029_provider_operation_reservations.sql` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective (5/5). Earlier provider endpoint, locked-current-trust, aggregate-
deadline, durable-replay, clean-source, and exact-port rules constrained the
sidecar into a transport-only profile. Security and independent patch review
found four material gaps that focused tests had not yet exposed; the resulting
admin/proxy hardening and reservation/advisory/lifecycle fixes passed hostile
peer, hostile proxy, forced-expiry, concurrent operation, suspension race,
crash/restart, callback, independent restore, and full-gate evidence. The
result completes local SDK deployment operations without manufacturing an
external provider, marketplace review, production host, key-custody process,
or support organization.
