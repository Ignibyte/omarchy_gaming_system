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
| `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` | After authenticating and owner-scoping the actor, resolve and validate a durable idempotency identity before current admission checks that apply only to new work; return it through the normal resource authorization path. | `aar/AAR-020-game-challenges-turn-notifications-history-and-expiration.md` |
| `PR-omarchy-gaming-system-preserve-monotonic-persisted-timestamps-001` | When a transaction can wait on a lock, derive mutation time after lock acquisition and preserve monotonicity against the stored value. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-use-nul-git-path-inventories-001` | Carry repository path inventories as NUL-delimited byte records through enumeration, sorting, hashing, and enforcement. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001` | Authenticate every wrapper and platform artifact before installing a repository tool, then revalidate its installed tree, executable link, and provenance before use. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-derive-digests-with-verifier-encoding-001` | Derive reviewed aggregate digests with the exact byte-record encoding used by the verifier. | `aar/AAR-013-idempotent-revision-checked-game-commands.md` |
| `PR-omarchy-gaming-system-parse-the-bytes-that-were-authenticated-001` | Parse security-sensitive package records from the exact byte buffers whose lengths and digests were verified. | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001` | Enforce untrusted response and archive limits during streaming, before buffering or decoding the complete input. | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-make-untrusted-text-format-explicit-001` | Render untrusted text through explicit plain-text mode unless a separately sanitized markup contract is intended and tested. | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-bound-package-traversal-work-001` | Bound package entries, directory depth, and accepted directory names in addition to accepted-file count and bytes. | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-gate-every-nested-workspace-proof-001` | Make every nested workspace that supplies required ticket evidence part of the canonical gate before relying on a worktree receipt. | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001` | Bind cartridge, grant, request, receipt, and event validation to registered principal and exact game, release, session, subject, scope, and expiry context. | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-validate-decoder-profile-not-headers-001` | Before accepting an asset, validate the exact decoder profile and every source of decoded work, not only dimensions and magic bytes. | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-bind-presentation-nodes-to-capabilities-001` | Bind every presentation node and effect to the exact required host capability before compatibility evaluation. | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-read-bounded-input-from-checked-handle-001` | Read untrusted filesystem input through the same checked regular-file handle with an enforced streaming byte ceiling. | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-distinguish-not-found-from-denial-001` | Treat only an explicit `NotFound` as absence; propagate or deny every other lookup failure at an authorization or revocation boundary. | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-require-descriptor-relative-privileged-store-001` | Before a cartridge store crosses a user or privilege boundary, use descriptor-relative containment or an equivalent OS sandbox plus authoritative revocation. | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-make-expensive-authentication-unique-001` | Cache expensive authentication work by immutable authenticated identity and publish retained bytes only after the referencing object passes admission. | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-enforce-render-budgets-during-construction-001` | Charge retained render-plan bytes with checked arithmetic before keeping each node, then preserve a final exact envelope check. | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001` | Bind each declarative interactive node to one exact platform-emitted payload shape before a dispatcher exists. | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001` | Independently recount cheap aggregate profile budgets when a serialized plan crosses into the trusted UI runtime. | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-validate-retained-directory-authority-001` | Validate type, expected owner, and group/other write permissions on every retained directory descriptor before treating it as a security boundary. | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001` | Hold one cross-process lock across the complete authenticated read, compare, and replace of monotonic policy state, then re-read beneath that lock. | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Persist the highest authenticated policy before applying its allow or deny decision. | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-charge-decoded-media-at-render-admission-001` | Charge per-instance decoded-media work against the selected render profile before publishing its node or asset, and exercise the maximum accepted decoder path. | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-validate-shell-arithmetic-input-001` | Treat process-local HTTP as untrusted input: bind readiness to the intended child and require explicit numeric type/range before shell arithmetic. | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `PR-omarchy-gaming-system-validate-game-state-cross-field-invariants-001` | At the compiled rules boundary, validate lifecycle relationships as well as JSON shape and scalar ranges before applying a command. | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `PR-omarchy-gaming-system-advance-durable-phase-before-phase-tools-001` | Record the completed validation phase in the active spec before invoking a phase-gated completion tool, then read back the tool receipt. | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001` | When a deduplication receipt may not exist, lock a guaranteed durable domain root before the first-read/insert decision and prove simultaneous first delivery. | `aar/AAR-018-production-remote-provider-security-foundation.md` |
| `PR-omarchy-gaming-system-classify-provider-egress-by-global-allocation-001` | Classify provider destinations from the positively allocated global address space, then exclude current special-purpose ranges and test translation/reserved prefixes. | `aar/AAR-018-production-remote-provider-security-foundation.md` |
| `PR-omarchy-gaming-system-preserve-first-callback-disposition-001` | Once an authenticated callback identity is durably accepted or ignored, an exact replay must preserve that first disposition instead of being reclassified by mutable current projection policy. | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-charge-authenticated-quota-after-authentication-001` | Charge shared authenticated-message quota only after exact signature/context/body verification, then recheck current key, lifecycle, and bounds before committing the charge. | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-layer-pilot-lifecycle-into-every-admission-001` | When a narrow activation lifecycle overlays a general provider release, lock and evaluate it at every launch, command, reconcile, event, and projection boundary. | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-use-one-provider-effect-lock-order-001` | Provider effect transactions acquire release, pilot, and session roots in one documented canonical order before receipts or projections. | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-gate-independent-executable-source-trees-001` | Every source tree that contributes an independently compiled executable or delivery proof must participate in the canonical gated-state hash. | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-retire-qml-xhr-after-generation-invalidation-001` | Invalidate the current QML request generation before retiring an XHR, detach its callback, retain it briefly, and abort outside the active callback. | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-protect-test-secret-file-handoffs-001` | For test-only credential handoffs, use a mode-0700 directory and mode-0600 non-symlink file, keep secrets out of argv and logs, and remove the exact file on every exit. | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-mirror-authoritative-client-response-bounds-001` | Client success validators and form limits must mirror the authoritative server contract exactly and reject empty required values or expired authority. | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-reconcile-regression-claims-with-executed-cases-001` | Reconcile every claimed hostile fixture outcome with an invoked test case before accepting the inspection gate. | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001` | Instantiate the production QML root after shared-control contract edits instead of relying only on isolated component assumptions. | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-own-headless-qt-test-environment-001` | Headless QML gate entrypoints must set their platform and rendering backend unconditionally. | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-preserve-bodyless-qml-requests-001` | When a QML request has no document, call `XMLHttpRequest.send()` with no argument; assert zero request bytes and immediately reuse the connection in a stateful fixture. | `aar/AAR-023-keyboard-first-qml-connections-and-private-inbox.md` |
| `PR-omarchy-gaming-system-observe-delivery-before-requeue-001` | Before manually requeueing a durable outbox row in a replay test, wait for the producer's original attempt to reach its committed delivered state. | `aar/AAR-023-keyboard-first-qml-connections-and-private-inbox.md` |
| `PR-omarchy-gaming-system-bind-presentation-cardinality-before-indexing-001` | Before trusted client presentation indexes participants or state arrays, bind uniqueness, actor membership, exact game version, and exact version-specific cardinality. | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `PR-omarchy-gaming-system-assert-minimum-layout-after-settle-001` | Exercise the production root at every supported minimum size and assert actual child geometry only after the asynchronous layout has settled. | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `PR-omarchy-gaming-system-stage-new-paths-before-final-security-scan-001` | Run the delivery security scan against a staged snapshot so newly created paths are included in the immutable inventory, then make no repository-content changes before delivery. | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `PR-omarchy-gaming-system-restore-focus-after-qml-materialization-001` | When a routed QML focus target depends on asynchronous data or delegate creation, restore focus only after the enabled target materializes and prove that handoff through the production root. | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `PR-omarchy-gaming-system-scope-style-policy-to-the-trusted-visual-boundary-001` | A centralized UI contract and its source policy must inventory every platform-owned visual surface, including trusted cartridge renderer nodes, rather than only the main application routes. | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `PR-omarchy-gaming-system-require-plain-text-on-every-qml-text-object-001` | Parse every in-scope QML `Text` object and require explicit `Text.PlainText`; rejecting named rich formats alone is insufficient. | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `PR-omarchy-gaming-system-assert-explicit-accessible-role-for-shell-actions-001` | Declare and production-root test the explicit accessible role for every persistent shell action, even when its shared control usually supplies one. | `aar/AAR-026-explicit-qml-application-exit-control.md` |
| `PR-omarchy-gaming-system-wait-for-deferred-qml-focus-before-input-001` | When a QML mode change schedules a deferred focus handoff, wait for the documented target to own active focus before injecting test input. | `aar/AAR-026-explicit-qml-application-exit-control.md` |
| `PR-omarchy-gaming-system-reconcile-foundation-docs-when-activated-001` | When a dormant foundation becomes an executable product path, reconcile every foundation architecture/current-state summary in the same delivery. | `aar/AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md` |
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Model publisher integrity, marketplace review, and server admission as separate attestations with independent issuers, meanings, and absence states. | `aar/AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md` |

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
| `BF-omarchy-gaming-system-authenticated-cartridge-reopen-001` | failure | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `BF-omarchy-gaming-system-provider-response-post-buffer-bound-001` | failure | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `BF-omarchy-gaming-system-qml-auto-text-untrusted-markup-001` | failure | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `BF-omarchy-gaming-system-cartridge-directory-budget-gap-001` | failure | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `BF-omarchy-gaming-system-nested-proof-gate-omission-001` | failure | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `BF-omarchy-gaming-system-qml-proof-log-routing-001` | failure | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-parse-the-bytes-that-were-authenticated-001` | rule | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001` | rule | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-make-untrusted-text-format-explicit-001` | rule | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-bound-package-traversal-work-001` | rule | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-gate-every-nested-workspace-proof-001` | rule | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001` | rule | `aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md` |
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | decision | `../../architecture/adr-0002-game-cartridge-and-provider-boundary.md` |
| `BF-omarchy-gaming-system-png-decoded-profile-underbound-001` | failure | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `BF-omarchy-gaming-system-presentation-capability-confusion-001` | failure | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `BF-omarchy-gaming-system-path-read-after-metadata-bound-gap-001` | failure | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `BF-omarchy-gaming-system-revocation-lookup-fail-open-001` | failure | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `BF-omarchy-gaming-system-pathname-store-containment-boundary-001` | failure | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-validate-decoder-profile-not-headers-001` | rule | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-bind-presentation-nodes-to-capabilities-001` | rule | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-read-bounded-input-from-checked-handle-001` | rule | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-distinguish-not-found-from-denial-001` | rule | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `PR-omarchy-gaming-system-require-descriptor-relative-privileged-store-001` | rule | `aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md` |
| `AD-omarchy-gaming-system-canonical-game-cartridge-v1-001` | decision | `../../architecture/game-cartridges.md` |
| `AD-omarchy-gaming-system-same-user-cartridge-store-001` | decision | `../../architecture/game-cartridges.md` |
| `BF-omarchy-gaming-system-repeated-asset-authentication-amplification-001` | failure | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `BF-omarchy-gaming-system-late-render-plan-byte-budget-001` | failure | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-make-expensive-authentication-unique-001` | rule | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-enforce-render-budgets-during-construction-001` | rule | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001` | rule | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001` | rule | `aar/AAR-016-trusted-cartridge-renderer-and-previewer.md` |
| `AD-omarchy-gaming-system-trusted-cartridge-renderer-v1-001` | decision | `../../architecture/game-cartridges.md` |
| `BF-omarchy-gaming-system-store-directory-authority-gap-001` | failure | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `BF-omarchy-gaming-system-policy-cache-rollback-race-001` | failure | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `BF-omarchy-gaming-system-denied-policy-not-persisted-001` | failure | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `BF-omarchy-gaming-system-render-raster-availability-gap-001` | failure | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-validate-retained-directory-authority-001` | rule | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001` | rule | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | rule | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `PR-omarchy-gaming-system-charge-decoded-media-at-render-admission-001` | rule | `aar/AAR-017-separate-repository-sdk-and-first-party-cartridge.md` |
| `AD-omarchy-gaming-system-portable-cartridge-sdk-release-v1-001` | decision | `../../architecture/game-cartridges.md` |
| `BF-omarchy-gaming-system-challenge-replay-current-policy-order-001` | failure | `aar/AAR-020-game-challenges-turn-notifications-history-and-expiration.md` |
| `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` | rule | `aar/AAR-020-game-challenges-turn-notifications-history-and-expiration.md` |
| `AD-omarchy-gaming-system-durable-game-challenge-orchestration-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-smoke-json-arithmetic-injection-001` | failure | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `BF-omarchy-gaming-system-game-state-lifecycle-consistency-gap-001` | failure | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `BF-omarchy-gaming-system-openwiki-phase-receipt-sequencing-001` | failure | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `PR-omarchy-gaming-system-validate-shell-arithmetic-input-001` | rule | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `PR-omarchy-gaming-system-validate-game-state-cross-field-invariants-001` | rule | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `PR-omarchy-gaming-system-advance-durable-phase-before-phase-tools-001` | rule | `aar/AAR-021-signal-siege-compiled-game-and-solo-bot-matches.md` |
| `AD-omarchy-gaming-system-signal-siege-solo-game-lifecycle-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-provider-callback-absent-row-race-001` | failure | `aar/AAR-018-production-remote-provider-security-foundation.md` |
| `BF-omarchy-gaming-system-provider-ipv6-special-use-egress-gap-001` | failure | `aar/AAR-018-production-remote-provider-security-foundation.md` |
| `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001` | rule | `aar/AAR-018-production-remote-provider-security-foundation.md` |
| `PR-omarchy-gaming-system-classify-provider-egress-by-global-allocation-001` | rule | `aar/AAR-018-production-remote-provider-security-foundation.md` |
| `AD-omarchy-gaming-system-remote-provider-security-foundation-001` | decision | `../../architecture/game-cartridges.md` |
| `BF-omarchy-gaming-system-provider-callback-replay-reclassification-001` | failure | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `BF-omarchy-gaming-system-provider-callback-preauth-quota-001` | failure | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `BF-omarchy-gaming-system-provider-pilot-lifecycle-admission-gap-001` | failure | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `BF-omarchy-gaming-system-provider-client-trust-expansion-001` | failure | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `BF-omarchy-gaming-system-provider-lock-order-inversion-001` | failure | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `BF-omarchy-gaming-system-first-party-provider-gate-state-omission-001` | failure | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-preserve-first-callback-disposition-001` | rule | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-charge-authenticated-quota-after-authentication-001` | rule | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-layer-pilot-lifecycle-into-every-admission-001` | rule | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-use-one-provider-effect-lock-order-001` | rule | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `PR-omarchy-gaming-system-gate-independent-executable-source-trees-001` | rule | `aar/AAR-019-first-party-remote-provider-migration-pilot.md` |
| `AD-omarchy-gaming-system-first-party-remote-authority-pilot-001` | decision | `../../architecture/adr-0002-game-cartridge-and-provider-boundary.md` |
| `BF-omarchy-gaming-system-qml-xhr-abort-lifetime-crash-001` | failure | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `BF-omarchy-gaming-system-qml-test-secret-path-authority-gap-001` | failure | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `BF-omarchy-gaming-system-qml-client-contract-bound-drift-001` | failure | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `BF-omarchy-gaming-system-qml-regression-claim-coverage-gap-001` | failure | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `BF-omarchy-gaming-system-qml-textarea-limit-api-assumption-001` | failure | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `BF-omarchy-gaming-system-qml-headless-platform-inheritance-001` | failure | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-retire-qml-xhr-after-generation-invalidation-001` | rule | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-protect-test-secret-file-handoffs-001` | rule | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-mirror-authoritative-client-response-bounds-001` | rule | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-reconcile-regression-claims-with-executed-cases-001` | rule | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001` | rule | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `PR-omarchy-gaming-system-own-headless-qt-test-environment-001` | rule | `aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md` |
| `AD-omarchy-gaming-system-qml-onboarding-authority-boundary-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-qml-bodyless-xhr-null-payload-001` | failure | `aar/AAR-023-keyboard-first-qml-connections-and-private-inbox.md` |
| `BF-omarchy-gaming-system-provider-replay-requeue-race-001` | failure | `aar/AAR-023-keyboard-first-qml-connections-and-private-inbox.md` |
| `PR-omarchy-gaming-system-preserve-bodyless-qml-requests-001` | rule | `aar/AAR-023-keyboard-first-qml-connections-and-private-inbox.md` |
| `PR-omarchy-gaming-system-observe-delivery-before-requeue-001` | rule | `aar/AAR-023-keyboard-first-qml-connections-and-private-inbox.md` |
| `AD-omarchy-gaming-system-qml-social-inbox-authority-boundary-001` | decision | `../../architecture/system-overview.md` |
| `BF-omarchy-gaming-system-qml-game-session-cardinality-gap-001` | failure | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `BF-omarchy-gaming-system-qml-home-action-overflow-001` | failure | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `BF-omarchy-gaming-system-security-scan-untracked-inventory-gap-001` | failure | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `PR-omarchy-gaming-system-bind-presentation-cardinality-before-indexing-001` | rule | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `PR-omarchy-gaming-system-assert-minimum-layout-after-settle-001` | rule | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `PR-omarchy-gaming-system-stage-new-paths-before-final-security-scan-001` | rule | `aar/AAR-024-signal-siege-versus-and-keyboard-first-game-flow.md` |
| `AD-omarchy-gaming-system-signal-siege-versus-version-boundary-001` | decision | `../../architecture/system-overview.md` |
| `AD-omarchy-gaming-system-platform-compiled-presenter-provenance-001` | decision | `../../architecture/game-cartridges.md` |
| `BF-omarchy-gaming-system-asynchronous-qml-focus-handoff-001` | failure | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `BF-omarchy-gaming-system-partial-trusted-visual-policy-scope-001` | failure | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `BF-omarchy-gaming-system-qml-plain-text-policy-default-gap-001` | failure | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `PR-omarchy-gaming-system-restore-focus-after-qml-materialization-001` | rule | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `PR-omarchy-gaming-system-scope-style-policy-to-the-trusted-visual-boundary-001` | rule | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `PR-omarchy-gaming-system-require-plain-text-on-every-qml-text-object-001` | rule | `aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md` |
| `AD-omarchy-gaming-system-host-owned-semantic-qml-theme-001` | decision | `../../architecture/system-overview.md`; `../../architecture/game-cartridges.md` |
| `BF-omarchy-gaming-system-qml-inherited-accessible-role-gap-001` | failure | `aar/AAR-026-explicit-qml-application-exit-control.md` |
| `BF-omarchy-gaming-system-qml-mode-focus-test-race-001` | failure | `aar/AAR-026-explicit-qml-application-exit-control.md` |
| `PR-omarchy-gaming-system-assert-explicit-accessible-role-for-shell-actions-001` | rule | `aar/AAR-026-explicit-qml-application-exit-control.md` |
| `PR-omarchy-gaming-system-wait-for-deferred-qml-focus-before-input-001` | rule | `aar/AAR-026-explicit-qml-application-exit-control.md` |
| `BF-omarchy-gaming-system-provider-activation-documentation-drift-001` | failure | `aar/AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md` |
| `BF-omarchy-gaming-system-cartridge-distribution-trust-conflation-001` | failure | `aar/AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md` |
| `PR-omarchy-gaming-system-reconcile-foundation-docs-when-activated-001` | rule | `aar/AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md` |
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | rule | `aar/AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap.md` |
| `AD-omarchy-gaming-system-owner-operated-server-extension-boundary-001` | decision | `../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md` |
