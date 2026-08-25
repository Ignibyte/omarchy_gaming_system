---
aar: AAR-006-persona-lifecycle-and-privacy
ticket: TICKET-006
pipeline: persona-lifecycle-and-privacy
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-006-persona-lifecycle-and-privacy

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-bbs-opaque-revocable-sessions-001` | System overview, AAR-005, and completed session notes. | Yes — the authenticated principal comes from a server-validated session, never a client owner field. |
| `AD-omarchy-bbs-account-registration-boundary-001` | Knowledge register and architecture. | Yes — preserved account/persona separation and reused canonical identity patterns without exposing accounts. |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Knowledge register. | Yes — object-level flows will be paired with direct multi-account BOLA tests. |
| OWASP authorization and API property guidance | Current primary-source review. | Yes — locked per-object owner predicates and explicit input/output allowlists before design. |

## What happened

The third identity outcome added multiple public personas per private account.
Authenticated clients can create, inventory, and edit only their account's
personas, while anyone can resolve one exact canonical handle. Ownership comes
only from the validated device-session principal; updates predicate on both
account and persona IDs. Persistence rows are narrowed through domain and
transport models containing only seven public fields, so account/session data
cannot be serialized accidentally. Multi-account tests and live smoke prove
canonical conflicts, validation preservation, handle movement, private-field
absence, and indistinguishable absent/foreign object results.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| — | The first no-run compile found a temporary JSON borrow and a test literal that required an unenabled UUID Serde feature. Both were isolated to new test helpers. | Immediate Phase 3 all-target compile. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-bbs-owner-scope-account-resources-001` | Derive account ownership from the validated session and scope every account-owned list or mutation by that principal; mutations predicate on both owner and object IDs, with absent and foreign objects sharing the same result. | Unpredictable UUIDs and transport allowlists do not provide object authorization. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-bbs-public-persona-boundary-001` | Allow multiple personas per private account; expose only seven public profile fields, make exact canonical handle lookup public, and keep account ownership structurally absent from every persona response. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The prior account/session boundaries made persona ownership a narrow reuse
of the validated principal. Current OWASP authorization and property guidance
locked owner predicates and explicit response allowlists before implementation.
CodeGraph bounded the cross-module surface, while direct multi-account tests
covered the graph's advisory test gap and found a missing timestamp assertion
before the gate. OpenWiki reconciled the completed identity surface with zero
remaining claim issues, and the full 12-check gate passed.
