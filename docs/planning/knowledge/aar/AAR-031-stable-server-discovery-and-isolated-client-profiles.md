---
aar: AAR-031-stable-server-discovery-and-isolated-client-profiles
ticket: TICKET-031
pipeline: stable-server-discovery-and-isolated-client-profiles
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: 5
---

# AAR-031-stable-server-discovery-and-isolated-client-profiles

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| ADR-0003 owner-operated server boundary | First executable ecosystem outcome after private-alpha software readiness | Yes — stable discovery and isolated profiles are explicitly next and must not imply federation. |
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | Knowledge search | Yes — identity persistence, public discovery, and the real QML consumer must run together. |
| Ticket 022 QML client rules | Existing onboarding and transport boundary | Yes — preserve exact response bounds, admitted origins, authority clearing, accessibility, and no persisted secrets. |
| Ticket 030 private-alpha delivery | Current registration and operator boundary | Yes — a server profile may retain public discovery metadata only; invitation and credential material remain ephemeral. |

## What happened

Every owner-operated database now receives one random singleton server UUID in
forward migration `0018`. Ordinary update, delete, and truncate are rejected,
and the operator recovery drill proves the UUID survives the real database dump
and restore. A bounded `OGS_SERVER_NAME` and public no-store
`GET /.well-known/omarchygs` document expose only that UUID, the public name,
protocol 1, and a deterministic set of implemented capabilities. `/health`
remains the separate operational-liveness contract.

The QML connector now treats discovery as admission. Players may connect once
or save and explicitly select up to sixteen exact public-only server profiles.
An already remembered canonical origin must present the same UUID before the
account screen appears, incompatible protocols/capabilities fail closed, and
switching origins clears the current request generation, bearer, MFA,
username, persona, social, inbox, challenge, and game authority before a new
request. Persisted state has an exact 16-KiB schema and cannot contain
credentials or cause automatic connection.

Two separate QML test-runner processes prove two profiles survive a client
restart, the 44-case fixture corpus covers compatible, future-capability,
incompatible, identity-replacement, malformed, slow, oversized, hostile-state,
and cross-authority paths, and the native package now contains the exact
38-file runtime. The canonical PostgreSQL suite passed 58 tests and the complete
22-stage diff gate passed before OpenWiki reconciliation. Codex Security sealed
a zero-finding review of the frozen implementation snapshot.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-qsettings-implicit-application-identity-001` | `QtCore.Settings` did not provide a reliable persistent namespace in headless test processes whose application identity was unset. | First two-process profile persistence run |
| `BF-omarchy-gaming-system-qsettings-url-prefix-assumption-001` | The first explicit settings path prepended `file://` even though QML `StandardPaths.writableLocation` already returned a URL, producing a `file://file/...` location. | Focused QML profile rerun |
| `BF-omarchy-gaming-system-database-test-portable-gate-marker-001` | The new real-PostgreSQL SQLx test lacked the repository-standard ignore marker, so the portable fast gate tried to run it without `DATABASE_URL`. | First `bin/gate.sh --fast` run |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-pin-qsettings-to-project-location-001` | Persistent QML state must use one explicit project-specific settings location and prove readback from a separate process under an isolated configuration root. | Test-runner or launcher application identifiers are not a stable persistence namespace. |
| `PR-omarchy-gaming-system-preserve-qml-standardpaths-url-type-001` | Treat a QML `StandardPaths.writableLocation` result as a URL and append only the relative filename; never add a second URL scheme without inspecting the returned type. | A syntactically plausible double prefix silently targets the wrong location. |
| `PR-omarchy-gaming-system-separate-database-tests-from-portable-loop-001` | Mark PostgreSQL-only tests with the repository's canonical ignore reason, then execute them through `scripts/test-database.sh` before relying on the portable fast gate. | The fast unit loop deliberately has no database authority, while delivery still requires the real migrated test. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-stable-server-discovery-and-isolated-profiles-001` | A community UUID belongs to durable PostgreSQL state; public discovery is an exact compatibility contract separate from health; saved client profiles contain public metadata only, pin origin to UUID, clear live authority before origin changes, and do not imply federation. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All ten EARS requirements have direct configuration, unit, PostgreSQL,
two-process QML persistence, hostile transport/settings, keyboard/accessibility,
package, backup/restore, live vertical-slice, security-inspection, and canonical
gate evidence. The initial failures were confined to test/persistence plumbing
and produced reusable rules rather than waived checks. OpenWiki completed and
reconciled the four affected pages; its warnings were the existing unrelated
Claims evidence debt already present on those broad pages. The final external
two-installation human acceptance event remains honestly open and independent
of this engineering outcome.
