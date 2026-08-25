---
aar: AAR-004-account-registration
ticket: TICKET-004
pipeline: account-registration
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-004-account-registration

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-bbs-server-authoritative-state-001` | Knowledge register, system overview, and completed foundation notes. | Yes — registration validation and persistence belong behind a thin transport handler. |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Knowledge register and AAR-003. | Yes — the plan requires direct unit and PostgreSQL integration evidence in addition to graph analysis. |
| `PR-omarchy-bbs-quality-gates-include-untracked-001` | Knowledge register and completed workflow notes. | Yes — new migrations, modules, tests, and docs remain in the canonical worktree hash before delivery. |
| `BUL-001-initial-push-pending` | Bulletin preflight. | Yes — validation is local and no remote CI or delivery claim will be made. |

## What happened

The first roadmap identity outcome shipped locally as `POST /v1/accounts`.
Registration now canonicalizes a deliberately narrow ASCII username namespace,
bounds passwords and request bodies, hashes credentials with explicit salted
Argon2id parameters off Tokio workers, and lets PostgreSQL authoritatively
resolve uniqueness races. Focused review removed credential-bearing `Debug`
implementations and added the request cap. The full gate proved the behavior
through unit tests, isolated migrated databases, a live API duplicate check,
and the existing QML health consumer.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-bbs-dev-dependency-runtime-mask-001` | `cargo test` compiled after the OS RNG crate was placed under dev-dependencies, but the production server could not import it. | The first live `scripts/dev.sh --smoke-test` production build. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-bbs-build-runtime-targets-after-dependency-changes-001` | Run a plain non-test binary build after changing runtime dependency boundaries. | Cargo test targets can see dev-dependencies and therefore mask a production dependency misclassification. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-bbs-account-registration-boundary-001` | Keep account registration behind a thin versioned handler; the account domain owns canonicalization, bounded Argon2id hashing, and persistence errors, while sessions and public personas remain separate slices. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The recalled server-authority and live-vertical-slice rules prevented the
schema from being mistaken for a feature and drove tests through the real
router and PostgreSQL. The graph-coverage limitation was correctly treated as
advisory, while direct tests exposed the actual behavior. The canonical live
smoke then found a production dependency defect that the unit target concealed.
