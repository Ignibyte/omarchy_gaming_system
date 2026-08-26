---
aar: AAR-029-operator-reporting-suspension-audit-and-recovery-drill
ticket: TICKET-029
pipeline: operator-reporting-suspension-audit-and-recovery-drill
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: 5
---

# AAR-029-operator-reporting-suspension-audit-and-recovery-drill

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Private-alpha roadmap and product charter | First unchecked outcome after native packaging | Yes — establishes operator safety and recovery as the final engineering gate before external invites. |
| Existing inactive-account authentication behavior | Identity migration plus session/MFA/sync tests | Yes — supplies a fail-closed enforcement boundary but not an audited operator mutation. |
| Provider operator/audit and restore proof | Tickets 018–019 documentation and tests | Yes — demonstrates useful patterns while confirming provider and platform authority/databases must remain separate. |
| Owner-operated server guide | Pre-invite responsibility checklist | Yes — backup, restore, moderation, key custody, and incident response need executable evidence rather than prose alone. |

## What happened

The platform now has one complete private-alpha safety loop. An authenticated
owned persona can submit a bounded, retry-safe report from the keyboard-first
Social screen. A trusted operator can review the private queue and apply only
reversible account suspension/reactivation or terminal report disposition
through a database-local CLI; there is no remote administrator API or reusable
administrator credential. Suspension revokes every live device session in the
same transaction, reactivation never restores old tokens, and every mutation
appends immutable audit evidence.

The platform recovery drill applies the complete migration set, drives the real
operator CLI, dumps representative identity/social/inbox/game/report/audit
state, restores it into isolation, compares all application-table counts, and
proves the restored production server rejects a pre-suspension token. The
21-stage diff gate passed, the final security review found no reportable
vulnerability, and OpenWiki completed with no unresolved claims. Private-alpha
limitations remain explicit: this is not remote administration, permanent
banning, appeals, evidence attachment, content deletion, scheduled backup
infrastructure, or cross-server moderation.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-private-route-error-cache-policy-gap-001` | Report success responses were private, but route errors initially missed `Cache-Control: no-store`. | First report API PostgreSQL test |
| `BF-omarchy-gaming-system-recovery-fixture-cumulative-schema-drift-001` | The direct restore fixture omitted the authority column added after the game-session creating migration. | First platform recovery drill |
| `BF-omarchy-gaming-system-cargo-multi-binary-default-run-ambiguity-001` | Adding the sysop executable made the server package's existing `cargo run` launch path ambiguous. | First live development smoke after the second binary was added |
| `BF-omarchy-gaming-system-idempotent-creation-replay-mutable-projection-001` | Report replay reconstructed its creation receipt from mutable current status after operator disposition. | Phase 3.5 correctness inspection |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-apply-private-cache-policy-at-route-boundary-001` | Apply private response cache policy around the bounded route so success and every error path inherit it, then test both. | Handler-only wrappers can leave extractor, authentication, and domain errors cacheable. |
| `PR-omarchy-gaming-system-build-recovery-fixtures-against-cumulative-schema-001` | Design direct backup/restore fixtures from the final cumulative migrated schema, not only the table's creating migration. | Later forward migrations can add required columns and cross-field constraints that a stale fixture silently misses. |
| `PR-omarchy-gaming-system-pin-default-run-when-adding-package-binary-001` | When a Cargo package gains another binary, pin `default-run` or update every launch consumer in the same change. | Single-binary inference is an implicit compatibility dependency for developer and service scripts. |
| `PR-omarchy-gaming-system-reconstruct-idempotent-creation-receipts-from-immutable-fields-001` | Reconstruct creation replays only from immutable creation fields, never from a resource's mutable current projection. | An exact retry must return the original committed creation result even after later lifecycle transitions. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-database-local-operator-safety-boundary-001` | Private-alpha platform moderation uses a PostgreSQL-local bounded command with mandatory immutable audit; network administrator identity and authorization remain a separate future design. Suspension is reversible containment that revokes current sessions, while report disposition is terminal and retained. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All nine EARS requirements have direct API, QML, PostgreSQL transaction,
real-CLI, restore, documentation, inspection, and canonical-gate evidence.
Inspection found and fixed the mutable replay projection before delivery; the
route-level no-store test, cumulative-schema restore proof, and Cargo
`default-run` regression all remain executable. OpenWiki completed with no
unresolved claims, the security diff scan retained no findings, and the
operator boundary remains local, narrow, audited, and explicit about the
controls that private alpha still lacks.
