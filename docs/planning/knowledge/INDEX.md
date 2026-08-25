# Knowledge register — Omarchy Gaming System

Search this file before planning or implementation, then read the linked AAR,
pipeline notes, or architecture document. New IDs belong both in the run's AAR
and in this register.

## Standing rules

| ID | Rule | Source |
|---|---|---|
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | A foundation is not complete until the real database migration, HTTP endpoint, and QML consumer run together. | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-secret-fixtures-must-not-match-source-001` | Secret-scanner fixtures must be assembled inside the sandbox rather than stored as a matching literal in source. | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-quality-gates-include-untracked-001` | Pre-staging quality gates must inspect committable untracked files explicitly. | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Structural graph coverage hints supplement but never replace direct test inspection and executed gate evidence. | `aar/AAR-003-codex-pipeline-intelligence-enforcement.md` |
| `PR-omarchy-bbs-build-runtime-targets-after-dependency-changes-001` | After changing runtime dependency boundaries, compile a non-test binary target because test builds can make dev-dependencies visible. | `aar/AAR-004-account-registration.md` |
| `PR-omarchy-bbs-bound-memory-hard-credential-work-001` | Moving password work off async workers is insufficient; bound concurrent memory-hard jobs across every credential entrypoint. | `aar/AAR-005-revocable-device-sessions.md` |
| `PR-omarchy-bbs-owner-scope-account-resources-001` | Derive account ownership from the validated session and scope every account-owned list or mutation by that principal; mutation SQL must predicate on both owner and object IDs. | `aar/AAR-006-persona-lifecycle-and-privacy.md` |
| `PR-omarchy-gaming-system-separate-live-identity-from-history-001` | For product-identity changes, inventory emitted/living identifiers separately from migrations, completed evidence, and registered historical IDs; test every intentional compatibility exception explicitly. | `aar/AAR-007-gaming-system-rebrand.md` |
| `PR-omarchy-gaming-system-preserve-independent-mfa-challenges-001` | Bound concurrent MFA challenges per account without deleting or replacing another unexpired challenge; consume only the selected challenge or clear all challenges during explicit MFA disablement. | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-bind-generated-tools-to-lock-provenance-001` | Install generated developer tools with the reviewed package manager and frozen lock, disable dependency lifecycle scripts when possible, and fail closed unless versions, patches, dependencies, and build output match a local provenance receipt. | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-canonicalize-hook-paths-001` | Canonicalize every hook-observed edit path against the Git worktree before classifying it, and treat outside or unresolved paths as gated failures. | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-exact-command-exemptions-001` | Exempt a non-mutating shell command from enforcement only when the entire normalized command matches the reviewed standalone form. | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-lock-social-pairs-before-state-001` | For relationship state shared by two personas, lock both extant persona roots in canonical order before authorizing or reading and mutating relationship/block state; prove competing outcomes against PostgreSQL. | `aar/AAR-009-persona-connections-and-blocking.md` |
| `PR-omarchy-gaming-system-bound-owner-inventories-at-write-001` | For an owner-scoped collection without pagination, enforce a stored-cardinality ceiling in the mutation transaction and serialize boundary races on an existing domain root. | `aar/AAR-010-private-inbox-conversations-and-messages.md` |
| `PR-omarchy-gaming-system-scope-public-cursors-to-resource-001` | Scope a public cursor or sequence to the resource whose history it orders unless cross-resource activity disclosure is an explicit documented contract. | `aar/AAR-010-private-inbox-conversations-and-messages.md` |
| `PR-omarchy-gaming-system-verify-retained-cursor-continuity-001` | After fetching a retained incremental page, verify that its first event is exactly the requested successor; otherwise require a baseline reset. | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `PR-omarchy-gaming-system-bound-live-transports-by-principal-001` | Bound long-lived transports by authenticated principal as well as resource and process, and release every counter through one lifetime-owned permit. | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `PR-omarchy-gaming-system-reauthorize-live-transports-without-touch-001` | Retain non-secret session identity for long-lived transports and periodically reauthorize it without extending idle lifetime; close fail-closed on invalidity or uncertainty. | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `PR-omarchy-gaming-system-pin-durable-game-rules-version-001` | Persist an exact immutable game key/version with every snapshot and never relabel stored state through the current process registry. | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-build-negative-fixtures-through-boundary-001` | Construct negative fixtures so malformed input reaches the owning validation boundary without the fixture itself failing first. | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-align-secret-families-with-integrations-001` | Reconcile high-signal secret-scanner families with active repository integrations and regression-test the real shared enforcement entrypoint. | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-terminate-options-before-repository-paths-001` | Pass an explicit option terminator before every repository-derived pathname supplied to a command-line parser. | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-treat-hook-trust-as-transitive-code-trust-001` | After branch or referenced-script changes, treat persisted project-hook trust as a grant over the transitive local code and rely independently on the worktree-bound gate for delivery proof. | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-check-replay-before-current-revision-001` | Under a serialized mutation boundary, resolve an idempotency receipt before enforcing the current revision so a genuine retry can replay after its first attempt advances durable state. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-preserve-monotonic-persisted-timestamps-001` | When a transaction can wait on a lock, derive mutation time after lock acquisition and preserve monotonicity against the stored value. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-use-nul-git-path-inventories-001` | Carry repository path inventories as NUL-delimited byte records through enumeration, sorting, hashing, and enforcement. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001` | Authenticate every wrapper and platform artifact before installing a repository tool, then revalidate its installed tree, executable link, and provenance before use. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-derive-digests-with-verifier-encoding-001` | Derive reviewed aggregate digests with the exact byte-record encoding used by the verifier. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |

## Register

| ID | Kind | Source |
|---|---|---|
| `BF-omarchy-bbs-postgres18-volume-layout-001` | failure | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `BF-omarchy-bbs-secret-selftest-self-match-001` | failure | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `BF-omarchy-bbs-untracked-whitespace-blindspot-001` | failure | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | rule | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-secret-fixtures-must-not-match-source-001` | rule | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-quality-gates-include-untracked-001` | rule | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `AD-omarchy-bbs-agent-work-pipeline-001` | decision | `../../architecture/adr-0001-agent-work-pipeline.md` |
| `BF-omarchy-bbs-openwiki-release-source-drift-001` | failure | `aar/AAR-003-codex-pipeline-intelligence-enforcement.md` |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | rule | `aar/AAR-003-codex-pipeline-intelligence-enforcement.md` |
| `AD-omarchy-bbs-codex-pipeline-intelligence-001` | decision | `../../architecture/adr-0001-agent-work-pipeline.md` |
| `BF-omarchy-bbs-dev-dependency-runtime-mask-001` | failure | `aar/AAR-004-account-registration.md` |
| `PR-omarchy-bbs-build-runtime-targets-after-dependency-changes-001` | rule | `aar/AAR-004-account-registration.md` |
| `AD-omarchy-bbs-account-registration-boundary-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-bbs-unbounded-argon2-concurrency-001` | failure | `aar/AAR-005-revocable-device-sessions.md` |
| `PR-omarchy-bbs-bound-memory-hard-credential-work-001` | rule | `aar/AAR-005-revocable-device-sessions.md` |
| `AD-omarchy-bbs-opaque-revocable-sessions-001` | decision | `../../architecture/system-overview.md` |
| `PR-omarchy-bbs-owner-scope-account-resources-001` | rule | `aar/AAR-006-persona-lifecycle-and-privacy.md` |
| `AD-omarchy-bbs-public-persona-boundary-001` | decision | `../../architecture/system-overview.md` |
| `PR-omarchy-gaming-system-separate-live-identity-from-history-001` | rule | `aar/AAR-007-gaming-system-rebrand.md` |
| `AD-omarchy-gaming-system-game-first-identity-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-mfa-challenge-invalidation-001` | failure | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `BF-omarchy-gaming-system-openwiki-lock-provenance-001` | failure | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `BF-omarchy-gaming-system-hook-path-alias-001` | failure | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `BF-omarchy-gaming-system-commit-exemption-bypass-001` | failure | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-preserve-independent-mfa-challenges-001` | rule | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-bind-generated-tools-to-lock-provenance-001` | rule | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-canonicalize-hook-paths-001` | rule | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `PR-omarchy-gaming-system-exact-command-exemptions-001` | rule | `aar/AAR-008-opt-in-totp-two-factor-authentication.md` |
| `AD-omarchy-gaming-system-opt-in-totp-mfa-001` | decision | `../../architecture/system-overview.md` |
| `AD-omarchy-gaming-system-registration-enumeration-risk-001` | decision | `../../../openwiki/product-boundaries.md` |
| `PR-omarchy-gaming-system-lock-social-pairs-before-state-001` | rule | `aar/AAR-009-persona-connections-and-blocking.md` |
| `AD-omarchy-gaming-system-persona-social-pair-model-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-unbounded-pending-inventory-001` | failure | `aar/AAR-010-private-inbox-conversations-and-messages.md` |
| `BF-omarchy-gaming-system-global-private-message-cursor-001` | failure | `aar/AAR-010-private-inbox-conversations-and-messages.md` |
| `PR-omarchy-gaming-system-bound-owner-inventories-at-write-001` | rule | `aar/AAR-010-private-inbox-conversations-and-messages.md` |
| `PR-omarchy-gaming-system-scope-public-cursors-to-resource-001` | rule | `aar/AAR-010-private-inbox-conversations-and-messages.md` |
| `AD-omarchy-gaming-system-private-inbox-model-001` | decision | `../../architecture/system-overview.md` |
| `AD-omarchy-gaming-system-block-interaction-inference-policy-001` | decision | `../../../openwiki/product-boundaries.md` |
| `BF-omarchy-gaming-system-sync-retention-read-race-001` | failure | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `BF-omarchy-gaming-system-websocket-decoder-defaults-001` | failure | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `BF-omarchy-gaming-system-websocket-principal-exhaustion-001` | failure | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `BF-omarchy-gaming-system-websocket-session-lifetime-001` | failure | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `PR-omarchy-gaming-system-verify-retained-cursor-continuity-001` | rule | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `PR-omarchy-gaming-system-bound-live-transports-by-principal-001` | rule | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `PR-omarchy-gaming-system-reauthorize-live-transports-without-touch-001` | rule | `aar/AAR-011-durable-persona-sync-and-websocket-notifications.md` |
| `AD-omarchy-gaming-system-persona-sync-boundary-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-test-fixture-preempted-boundary-001` | failure | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `BF-omarchy-gaming-system-openai-secret-family-omission-001` | failure | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `BF-omarchy-gaming-system-secret-path-option-injection-001` | failure | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `BF-omarchy-gaming-system-transitive-hook-trust-gap-001` | failure | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-pin-durable-game-rules-version-001` | rule | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-build-negative-fixtures-through-boundary-001` | rule | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-align-secret-families-with-integrations-001` | rule | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-terminate-options-before-repository-paths-001` | rule | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `PR-omarchy-gaming-system-treat-hook-trust-as-transitive-code-trust-001` | rule | `aar/AAR-012-game-registry-and-versioned-sessions.md` |
| `AD-omarchy-gaming-system-version-pinned-game-session-foundation-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-transaction-start-timestamp-regression-001` | failure | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `BF-omarchy-gaming-system-newline-path-enforcement-bypass-001` | failure | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `BF-omarchy-gaming-system-codegraph-artifact-integrity-gap-001` | failure | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `BF-omarchy-gaming-system-digest-record-encoding-mismatch-001` | failure | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-check-replay-before-current-revision-001` | rule | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-preserve-monotonic-persisted-timestamps-001` | rule | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-use-nul-git-path-inventories-001` | rule | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001` | rule | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-derive-digests-with-verifier-encoding-001` | rule | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `AD-omarchy-gaming-system-idempotent-revision-command-boundary-001` | decision | `../../architecture/system-overview.md` |
