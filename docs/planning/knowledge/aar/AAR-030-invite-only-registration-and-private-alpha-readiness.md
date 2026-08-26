---
aar: AAR-030-invite-only-registration-and-private-alpha-readiness
ticket: TICKET-030
pipeline: invite-only-registration-and-private-alpha-readiness
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: 5
---

# AAR-030-invite-only-registration-and-private-alpha-readiness

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Private-alpha roadmap and product charter | Sole unfinished private-alpha outcome after Ticket 029 | Yes — external admission is next, but real external execution must remain distinct from software readiness. |
| `AD-omarchy-gaming-system-database-local-operator-safety-boundary-001` | Ticket 029 architecture and operator CLI | Yes — invitation lifecycle belongs in the same local, audited authority boundary. |
| `AD-omarchy-gaming-system-registration-enumeration-risk-001` | Knowledge register and product boundaries | Yes — preserves the accepted username-conflict behavior while requiring invitation lifecycle failures to remain uniform. |
| Ticket 022 QML onboarding notes and prevention rules | Nearest registration client pipeline | Yes — invitation codes must follow the existing secret lifetime, exact-schema, admitted-origin, and keyboard/accessibility controls. |
| Owner-operated server responsibilities | Pre-invite operator checklist | Yes — the alpha runbook must connect admission with TLS, backup, moderation, incident response, and honest unsupported boundaries. |

## What happened

Open account creation is now an owner-controlled private-alpha admission path.
The existing account route requires a 256-bit, expiring, one-account invitation
issued through the database-local operator executable. PostgreSQL retains only
the code digest; issue and revocation are idempotent audited transactions;
inventory is bounded and secret-free; and the account insert consumes the
invitation in the same row-locked transaction.

An uncertain first response can be retried safely. The original two-field
account receipt is returned with HTTP 200 only when the used code's canonical
username and Argon2id password both match. Every other unavailable or changed
invitation receives the same HTTP 403 shape, and the credential comparison has
no early username exit. The keyboard-first QML client masks the invitation,
sends it only to the admitted registration origin, and clears it on every
credential-lifetime boundary.

The real local CLI, migrated production server, HTTP path, and ordinary login
now run together in an isolated private-alpha drill at canonical gate stage 22.
The complete 22-stage diff gate passed after its cold-database readiness bounds
were calibrated. The runbook makes the remaining boundary explicit: this
delivery proves software readiness, not that the first external two-installation
human event has occurred.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-blocked-lock-joined-projection-stale-001` | A contender that waited on the invitation row lock observed the updated invitation state but a stale joined account projection from its earlier transaction snapshot. | Concurrent invitation-consumption PostgreSQL test |
| `BF-omarchy-gaming-system-qml-editable-accessible-role-gap-001` | The shared styled text-field wrapper exposed a name but no explicit editable accessibility role for the new invitation control. | First QML accessibility fixture run |
| `BF-omarchy-gaming-system-registration-contract-caller-drift-001` | One secondary live-smoke registration caller retained the old two-field request after the invite-only schema break. | First complete migrated development smoke |
| `BF-omarchy-gaming-system-used-invite-username-timing-oracle-001` | A leaked used invitation returned before Argon2id on username mismatch, revealing whether the submitted canonical username matched the linked account. | Codex Security diff scan and real timing reproduction |
| `BF-omarchy-gaming-system-cold-migration-readiness-deadline-001` | The final recovery and admission drills used a ten-second fixed startup deadline that was too close to cold 17-migration startup under full-gate load. | First canonical diff gate, stages 21 and 22 |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-refresh-dependent-projections-after-blocked-lock-001` | After a row-lock wait changes the root lifecycle, end the stale transaction and reload dependent joined projections before replay authorization. | A locked row can be current while a joined row remains invisible to the transaction snapshot used before the wait. |
| `PR-omarchy-gaming-system-declare-editable-qml-accessibility-role-001` | Shared styled text inputs must declare and fixture-test their explicit editable accessibility role. | Visual inheritance and an accessible name do not prove assistive technology receives the correct control semantics. |
| `PR-omarchy-gaming-system-inventory-callers-after-exact-contract-break-001` | After an intentional exact-schema break, inventory every production, fixture, script, and peer caller and execute the complete vertical slice. | Focused primary-path tests can miss a secondary caller that still emits the retired request shape. |
| `PR-omarchy-gaming-system-equalize-secret-replay-credential-work-001` | Once a bearer secret resolves a credential-linked row, perform the same password-verification work before combining any attacker-controlled identity predicate into denial. | Identical status and body are insufficient when response timing reveals linked identity. |
| `PR-omarchy-gaming-system-budget-readiness-for-measured-cold-path-001` | Bound process readiness with a deadline that covers measured cold migration under full-gate load plus margin, while retaining immediate process-death detection. | A tight healthy-path timeout creates late-gate flakes without detecting a real server failure. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-invite-only-account-admission-001` | Private-alpha account admission uses expiring one-account digest-backed bearer invitations issued, inventoried, and revoked only through the existing database-local audited operator boundary. Account creation and consumption are atomic; only credential-proven exact replay recovers a used invitation's public receipt. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All nine EARS requirements have direct unit, PostgreSQL concurrency,
real-CLI, API, QML keyboard/accessibility, migrated vertical-slice, isolated
private-alpha, documentation, inspection, security, and canonical-gate
evidence. Inspection found and fixed the used-code timing oracle before
delivery, and read-only verification retained exact replay while making wrong
username and wrong password denial timing equivalent. OpenWiki completed and
updated the four affected pages; it warned that their broad pre-existing Claims
sidecars still contain unrelated unresolved evidence debt, so it preserved
those sidecars rather than falsely marking them verified. The human external
alpha event remains correctly open on the roadmap.
