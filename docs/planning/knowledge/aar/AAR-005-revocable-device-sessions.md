---
aar: AAR-005-revocable-device-sessions
ticket: TICKET-005
pipeline: revocable-device-sessions
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-005-revocable-device-sessions

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-bbs-account-registration-boundary-001` | System overview, AAR-004, and completed registration notes. | Yes — sessions authenticate account credentials but remain separate from public personas. |
| `PR-omarchy-bbs-build-runtime-targets-after-dependency-changes-001` | Knowledge register and AAR-004. | Yes — the plan includes a plain production build after adding token dependencies. |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Knowledge register. | Yes — token/auth topology will be paired with direct security and PostgreSQL tests. |
| OWASP session/authentication guidance and RFC 6750 | Current primary-source review. | Yes — set entropy, storage, timeout, error, cache, logging, and header contracts before design. |

## What happened

The second identity outcome added password-authenticated, account-scoped device
sessions. Login now performs generic, comparable Argon2 work across missing,
wrong-password, suspended, and disabled cases; issues a 256-bit opaque token;
and stores only its SHA-256 digest. Each Bearer-authenticated request enforces
active account status plus idle, absolute, and revocation rules in PostgreSQL.
Accounts can inventory and idempotently revoke their own sessions without
learning whether another account owns a supplied UUID. Inspection also bounded
shared Argon2 concurrency and made device inventory explicitly non-cacheable.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-bbs-unbounded-argon2-concurrency-001` | `spawn_blocking` protected Tokio workers but allowed request concurrency to multiply 19 MiB Argon2 allocations without a bound. | Phase 3.5 security/resource inspection. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-bbs-bound-memory-hard-credential-work-001` | Share a concurrency bound across every registration/login path that performs memory-hard credential work. | Moving work to blocking threads prevents async starvation but does not prevent memory exhaustion. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-bbs-opaque-revocable-sessions-001` | Use server-stored device sessions with 256-bit opaque Bearer tokens, digest-only persistence, server-enforced idle/absolute timeouts, and owner-scoped inventory/revocation; keep account identity out of session JSON. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The registration boundary made credential reuse a small extraction rather
than a second password implementation. Current OWASP/RFC review locked token,
timeout, cache, error, and header behavior before code. The runtime-target rule
kept dependency verification explicit, and the graph/direct-test pairing found
the one material resource gap—unbounded memory-hard concurrency—before the
canonical gate.
