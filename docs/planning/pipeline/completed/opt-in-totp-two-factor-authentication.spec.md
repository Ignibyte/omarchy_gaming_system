---
title: Opt-in TOTP two-factor authentication
pipeline_id: b5b83a39-3fca-4351-a192-509b5b9ffa20
status: Phase 5 — Complete PASS
ticket: TICKET-008
ticket_doc: docs/planning/tickets/closed/TICKET-008-opt-in-totp-two-factor-authentication.md
aar: docs/planning/knowledge/aar/AAR-008-opt-in-totp-two-factor-authentication.md
created: 2026-08-24
---

# Opt-in TOTP two-factor authentication — spec

## Intent

Protect OmarchyGS accounts that opt in with a second factor before connections,
inboxes, and games increase account-takeover impact. Preserve the existing
password-only login contract for accounts that have not enrolled.

## Scope

- In: optional TOTP, encrypted authenticator secrets, pending enrollment,
  one-time recovery codes, short-lived login challenges, replay prevention,
  account-wide failed-attempt throttling, MFA status/disable APIs, configuration,
  API/operator docs, PostgreSQL tests, and live smoke.
- Out: SMS/email codes, WebAuthn/passkeys, mandatory organization policies,
  multiple simultaneous TOTP authenticators, QR image rendering, password reset,
  remote key management/HSM integration, distributed edge throttling, QML login
  screens, social/game behavior, commits, pushes, and pull requests.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an authenticated active account supplies its correct password to begin enrollment, the system shall generate an account-unique TOTP secret, persist it only under authenticated encryption, and return the provisioning secret and URI once under `Cache-Control: no-store` without enabling MFA yet. | Unit and router/PostgreSQL tests |
| REQ-002 | When the account confirms enrollment with a valid unused TOTP, the system shall enable MFA and return ten independently random one-time recovery codes exactly once; invalid or replayed codes shall not enable it. | Unit and router/PostgreSQL tests |
| REQ-003 | When correct primary credentials belong to an MFA-enabled account, the system shall return one of at most ten independent short-lived opaque MFA challenges without creating a device session; when the live challenge budget is exhausted, the system shall return HTTP 429 without invalidating an existing challenge. Accounts without MFA shall retain the existing `201 Created` login contract and all primary-credential failures shall remain generic. | Multi-account and challenge-budget router/PostgreSQL tests |
| REQ-004 | When a valid unexpired challenge is completed within its attempt limit using an unused TOTP or recovery code, the system shall consume the factor and challenge atomically and create one device session; reused, expired, malformed, locked, or inactive-account attempts shall create none. | Transactional router/PostgreSQL tests |
| REQ-005 | When TOTP is verified, the system shall use RFC 6238's 30-second, six-digit HMAC-SHA-1 profile with a one-step drift window, accept each time step only once, and apply failed-attempt throttling across challenges for the account. | RFC test vectors, deterministic unit tests, and PostgreSQL concurrency/security tests |
| REQ-006 | When an authenticated account reads MFA status or disables MFA with its password and a valid current TOTP or recovery code, the system shall expose no secret material and shall remove MFA enforcement and outstanding challenges only after both factors succeed. | Router/PostgreSQL tests and response audit |
| REQ-007 | When the canonical diff gate validates the slice, it shall exercise enrollment, MFA-gated login, recovery, replay rejection, disablement, the migrated PostgreSQL path, and the existing QML health connector without leaking secrets. | `bin/gate.sh --diff` and live smoke |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Use optional six-digit TOTP with HMAC-SHA-1, a 30-second step, and a one-step past/future drift window. | This is the interoperable RFC 6238 profile used by common authenticator applications; a bounded window balances clock drift and attack surface. |
| 2 | Require a valid device session plus the account password to begin enrollment, and do not enforce MFA until a TOTP confirmation succeeds. | A stolen bearer alone must not be able to lock the account owner out, and a mistyped provisioning step must not activate an unusable authenticator. |
| 3 | Encrypt each random 160-bit TOTP secret with AES-256-GCM under an operator-supplied `OGS_MFA_ENCRYPTION_KEY`, binding ciphertext to the account ID. | TOTP verification requires recoverable symmetric key material, so digest-only storage is impossible; authenticated encryption and installation-level key separation reduce database-only compromise impact. |
| 4 | Return ten 120-bit recovery codes once after confirmation and store only SHA-256 digests; each code is consumed atomically. | Recovery is required before opt-in can safely lock future logins, while high-entropy codes allow fast digest lookup without retaining bearer material. |
| 5 | Correct primary credentials on an enabled account create a five-minute opaque challenge and no device session. A challenge permits at most five failed factor attempts, while the authenticator also tracks failures across challenges and temporarily locks verification. | A bounded two-stage flow preserves the current JSON session API while preventing challenge churn from bypassing OTP throttling. |
| 6 | Consume a successfully used TOTP time step, recovery code, and login challenge in the same database transaction before issuing the device session. | RFC 6238 and current NIST guidance require one-time acceptance; row locks and a transaction prevent concurrent replay. |
| 7 | Disabling MFA requires an authenticated session, the current password, and a valid unused TOTP or recovery code. Existing device sessions remain independently revocable. | Security-setting removal must prove both current factors without silently changing the established device-session lifecycle. |
| 8 | “OmarchyGS” is the human shorthand for Omarchy Gaming System. Existing `omarchy-gaming-system`, `ogs`, and `ogs1_` technical namespaces remain canonical. | This records the user's terminology without another runtime compatibility break. |
| 9 | Preserve up to ten independent live MFA login challenges per account and reject further issuance with the existing `mfa_rate_limited` contract until a challenge is consumed or expires. | The bounded set prevents a password-only actor from invalidating another device's in-progress challenge, supports legitimate overlap, and avoids a schema migration while the account-row lock serializes issuance. |
| 10 | Bootstrap the exact OpenWiki-declared pnpm release only after verifying its pinned SHA-512 tarball digest, install with the tracked frozen pnpm lock and lifecycle scripts disabled, and bind the ignored build to checked provenance. | This closes the registry-to-developer execution gap without vendoring generated dependency state into the product repository. |
| 11 | Canonicalize hooked edit paths with `realpath -m` before gated-path classification and exempt only exact standalone non-mutating commit commands. | This closes the two confirmed Codex guardrail aliases while keeping hooks cooperative rather than claiming a hostile shell sandbox. |
| 12 | Temporarily retain the explicit public `username_taken` registration conflict for private alpha and revisit it only with a verifiable private registration channel. | The user accepted the low enumeration risk for this slice; changing the response without a trustworthy delivery/claim channel would silently alter registration usability rather than complete the missing deployment design. |

## Linked artifacts

- Ticket: [TICKET-008](../../tickets/closed/TICKET-008-opt-in-totp-two-factor-authentication.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled standards | factor, recovery, challenge, and key boundaries recorded |
| 2 Design | Transaction flows, migration, file manifest, regression plan | actionable design and CodeGraph evidence |
| 3 Implement | MFA domain/API/config/migration/tests/smoke/docs | focused checks and self-review |
| 3.5 Inspect | Security and concurrency ledger plus fixes | fresh post-edit CodeGraph analysis |
| 4 Validate | Unit/integration tests and canonical delivery gate | matching gate receipt |
| 5 Complete | OpenWiki reconciliation, AC audit, submitted AAR, archive | OpenWiki receipt and no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |
