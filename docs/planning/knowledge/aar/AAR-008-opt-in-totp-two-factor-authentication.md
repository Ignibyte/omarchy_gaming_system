---
aar: AAR-008-opt-in-totp-two-factor-authentication
ticket: TICKET-008
pipeline: opt-in-totp-two-factor-authentication
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-008-opt-in-totp-two-factor-authentication

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-bbs-opaque-revocable-sessions-001` | Knowledge register, system overview, AAR-005, and completed session notes. | Yes — MFA must gate issuance while preserving opaque, revocable device sessions. |
| `PR-omarchy-bbs-bound-memory-hard-credential-work-001` | Knowledge register and AAR-005. | Yes — enrollment and disablement password checks must share the existing Argon2 concurrency bound. |
| `PR-omarchy-bbs-owner-scope-account-resources-001` | Knowledge register and AAR-006. | Yes — enrollment, status, and disablement derive account ownership from the validated session. |
| `PR-omarchy-gaming-system-separate-live-identity-from-history-001` | Knowledge register and AAR-007. | Yes — OmarchyGS is a living shorthand while established technical identifiers and historical IDs retain their defined roles. |
| RFC 6238, RFC 4226, and NIST SP 800-63B-4 | Current primary-source review. | Yes — set interoperable TOTP, replay, key-protection, recovery, and cross-challenge throttling contracts before design. |

## What happened

OmarchyGS gained optional account-level TOTP MFA without coupling credentials
to public personas or changing password-only accounts. An authenticated account
must re-enter its password to begin a ten-minute encrypted enrollment, confirm
one RFC 6238 code, and retain ten one-time recovery codes whose digests are the
only stored form. Future valid-password logins create bounded, digest-only
five-minute challenges and no session; factor verification consumes the
challenge and TOTP step or recovery code in the same transaction that creates
the device session. Status is deliberately narrow, and disablement requires a
valid device session, current password, and unused factor.

The independent security inspection found three low-severity issues. Challenge
churn could invalidate another device's legitimate in-progress login; OpenWiki
was built from an npm dependency graph instead of its reviewed pnpm lock; and
public registration exposes whether a private username exists. The first two
were fixed and regression-tested. The user accepted the registration oracle for
the private-alpha slice until a verifiable private registration channel can be
designed. The same review found two workflow defects: path aliases could bypass
pre-design gated-path classification, and help text in a compound command could
exempt a real commit. Both hook defects were hardened and adversarially tested.

CodeGraph design and inspection evidence, a frozen Codex Security scan,
OpenWiki Grounded Claims reconciliation, 22 fast tests, all 12 migrated
PostgreSQL tests, and the live registration/session/persona/MFA/QML path passed.
The final post-wiki gate receipt and OpenWiki completion receipt match worktree
state `6ef93e06b0e0dc9f6501add66dbea4536396a26846a8a4a31ec7e0b93adc41c9`.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-mfa-challenge-invalidation-001` | A correct-password login deleted every live MFA challenge before issuing its replacement, so one password holder could repeatedly invalidate another device's in-progress second-factor attempt. | Independent security scan and direct transaction review. |
| `BF-omarchy-gaming-system-openwiki-lock-provenance-001` | OpenWiki declared pnpm and tracked `pnpm-lock.yaml`, but local setup used npm and executed an ignored dependency graph that was not bound to the reviewed lock. | Independent security scan and generated-tool inspection. |
| `BF-omarchy-gaming-system-hook-path-alias-001` | Pre-design gated-path classification inspected lexical paths, allowing `..`, absolute, or symlink aliases to name gated files through a documentation-looking path. | Adversarial hook review. |
| `BF-omarchy-gaming-system-commit-exemption-bypass-001` | Any help or dry-run token in a compound shell command exempted a real `git commit` from active-pipeline and receipt enforcement. | Adversarial hook review. |
| — | The first expiry test fixture moved challenge expiry before creation and was rejected by the intentional database constraint before endpoint behavior ran. | Initial PostgreSQL test run; fixed by aging both timestamps while preserving their order. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-preserve-independent-mfa-challenges-001` | Bound concurrent MFA challenges per account without deleting or replacing another unexpired challenge; consume only the selected challenge or clear all challenges during explicit MFA disablement. | Password proof is not second-factor proof, so challenge issuance must not let a password holder deny another device's legitimate factor attempt. |
| `PR-omarchy-gaming-system-bind-generated-tools-to-lock-provenance-001` | Install generated developer tools with the reviewed package manager and frozen lock, disable dependency lifecycle scripts when possible, and fail closed unless versions, patches, dependencies, and build output match a local provenance receipt. | Pinning only a repository commit does not bind the executable transitive dependency graph. |
| `PR-omarchy-gaming-system-canonicalize-hook-paths-001` | Canonicalize every hook-observed edit path against the Git worktree before classifying it, and treat outside or unresolved paths as gated failures. | Lexical prefix checks do not identify the file the filesystem will actually mutate. |
| `PR-omarchy-gaming-system-exact-command-exemptions-001` | Exempt a non-mutating shell command from enforcement only when the entire normalized command matches the reviewed standalone form. | A harmless token elsewhere in a compound command must not mask an authorized-state mutation. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-opt-in-totp-mfa-001` | Keep optional TOTP inside private account authentication: encrypt recoverable secrets, hash bearer-like recovery/challenge material, gate only new session issuance, and leave personas plus existing device sessions independent. | `docs/architecture/system-overview.md` |
| `AD-omarchy-gaming-system-registration-enumeration-risk-001` | Temporarily retain the explicit public `username_taken` registration conflict for private alpha; revisit it only with a separately designed verifiable private registration channel. | `openwiki/product-boundaries.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The pipeline added a security-sensitive identity slice and then used its
independent inspection gate to find availability, supply-chain provenance, and
workflow-enforcement defects before completion. The remediation preserved
legitimate overlapping device logins, converted OpenWiki setup to a verified
frozen install, and closed both confirmed hook aliases without claiming that
cooperative hooks are a hostile shell sandbox. The one deferred security issue
is explicit, low severity, user-approved, and bounded to the current
private-alpha registration contract. OpenWiki completed with no remaining claim
issues, and the final canonical gate passed against the exact post-wiki state.
