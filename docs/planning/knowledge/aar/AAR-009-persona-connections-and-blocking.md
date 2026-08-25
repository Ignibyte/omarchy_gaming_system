---
aar: AAR-009-persona-connections-and-blocking
ticket: TICKET-009
pipeline: persona-connections-and-blocking
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-009-persona-connections-and-blocking

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-bbs-owner-scope-account-resources-001` | Knowledge-register search before planning | Yes — acting personas must be constrained by both authenticated account and persona UUID. |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Knowledge-register search before planning | Yes — multi-account and concurrency claims require direct PostgreSQL evidence. |
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | Knowledge-register search before planning | Yes — the live smoke must cross migration, API, and connector, even before the QML social UI exists. |
| Persona lifecycle pipeline and system overview | Nearest completed notes and architecture recall | Yes — reuse the seven-field public profile boundary and non-disclosing object authorization. |
| OpenWiki product/runtime pages | Generated evidence recall | Yes — confirms connections must use persona identity and must not claim future inbox or notification behavior. |

## What happened

OmarchyGS gained its first persona-to-persona social primitive without crossing
the private account boundary. Owned personas can create and inventory
directional requests, the addressee can accept one mutual connection, either
participant can cancel or remove state, and a private directional block
atomically removes the relationship and prevents requests in either direction
until explicitly unblocked. Every social response embeds only the established
seven-field public persona model.

The schema keeps one canonical UUID-ordered row per pending or accepted pair
and a separate directional block row. Every pair mutation locks both persona
roots in that same order before checking ownership, target policy, blocks, or
relationship state. This made the opposite-request, concurrent-acceptance, and
request-versus-block outcomes linear and testable without introducing the
future inbox or event contract early.

The frozen 56-item Codex Security diff scan completed with no reportable
findings. One CodeGraph install-provenance candidate was rejected after
validation because the exact published npm version, HTTPS/integrity metadata,
and threat model did not establish a realistic lower-privileged artifact
substitution path. CodeGraph design and inspection receipts matched, OpenWiki
completed after updating the runtime, product, quickstart, and validation
claims, and the Phase 4 gate passed 23 fast tests, all 16 migrated PostgreSQL
tests, and the complete two-account social/MFA/session/QML smoke. The final
post-wiki state is
`926f3408135d7cbaceee484d279a4b02a0ddf8489f719dca3df45bce81884caf`.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| — | An early formatting check ran after `main.rs` declared `connection_api_tests` but before the new module file existed. | Focused implementation loop; adding the file before rerunning Cargo resolved it. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-lock-social-pairs-before-state-001` | For relationship state shared by two personas, lock both extant persona roots in a canonical order before authorizing the actor or reading and mutating relationship/block tables; prove competing outcomes against real PostgreSQL. | Cross-table uniqueness alone cannot serialize request, acceptance, removal, and block races or prevent deadlock when callers name the pair in opposite order. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-persona-social-pair-model-001` | Model each cross-account persona pair as one canonical pending-or-accepted relationship row, retain requester/addressee direction only while pending, and model private blocks as separate directional rows that delete relationship state atomically. | `docs/architecture/system-overview.md` and `openwiki/runtime-foundation.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. Prior account/persona privacy and advisory-graph lessons directly shaped
the design, while the required structural, security, database, live-smoke, and
OpenWiki evidence all exercised different failure modes. The concurrency model
stayed small enough to reason about and four migrated tests proved the races
that static inspection could not. No accepted requirement or inspection issue
was dropped, no future inbox/WebSocket behavior was misrepresented as present,
and no commit, push, or pull request was performed.
