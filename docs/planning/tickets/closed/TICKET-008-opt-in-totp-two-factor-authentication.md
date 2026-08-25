---
title: TICKET-008-opt-in-totp-two-factor-authentication
status: closed
ticket_number: 008
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/opt-in-totp-two-factor-authentication.spec.md
---

# TICKET-008-opt-in-totp-two-factor-authentication

## Summary

Add optional TOTP two-factor authentication to OmarchyGS accounts, including
safe enrollment, one-time recovery codes, MFA-gated device login, account-wide
attempt limits, replay resistance, status, and secure disablement.

## Why

The user requested opt-in 2FA before the social and game surfaces expand the
impact of an account takeover. The current password and revocable-session
boundary is the right place to add it without coupling authentication to public
personas or later game behavior.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an authenticated active account supplies its correct password to begin enrollment, the system shall generate an account-unique TOTP secret, persist it only under authenticated encryption, and return the provisioning secret and URI once under `Cache-Control: no-store` without enabling MFA yet. | Unit and router/PostgreSQL tests |
| REQ-002 | When the account confirms enrollment with a valid unused TOTP, the system shall enable MFA and return ten independently random one-time recovery codes exactly once; invalid or replayed codes shall not enable it. | Unit and router/PostgreSQL tests |
| REQ-003 | When correct primary credentials belong to an MFA-enabled account, the system shall return one of at most ten independent short-lived opaque MFA challenges without creating a device session; when the live challenge budget is exhausted, the system shall return HTTP 429 without invalidating an existing challenge. Accounts without MFA shall retain the existing `201 Created` login contract and all primary-credential failures shall remain generic. | Multi-account and challenge-budget router/PostgreSQL tests |
| REQ-004 | When a valid unexpired challenge is completed within its attempt limit using an unused TOTP or recovery code, the system shall consume the factor and challenge atomically and create one device session; reused, expired, malformed, locked, or inactive-account attempts shall create none. | Transactional router/PostgreSQL tests |
| REQ-005 | When TOTP is verified, the system shall use RFC 6238's 30-second, six-digit HMAC-SHA-1 profile with a one-step drift window, accept each time step only once, and apply failed-attempt throttling across challenges for the account. | RFC test vectors, deterministic unit tests, and PostgreSQL concurrency/security tests |
| REQ-006 | When an authenticated account reads MFA status or disables MFA with its password and a valid current TOTP or recovery code, the system shall expose no secret material and shall remove MFA enforcement and outstanding challenges only after both factors succeed. | Router/PostgreSQL tests and response audit |
| REQ-007 | When the canonical diff gate validates the slice, it shall exercise enrollment, MFA-gated login, recovery, replay rejection, disablement, the migrated PostgreSQL path, and the existing QML health connector without leaking secrets. | `bin/gate.sh --diff` and live smoke |

## Scope

- In: optional TOTP, encrypted authenticator secrets, pending enrollment,
  one-time recovery codes, short-lived login challenges, replay prevention,
  account-wide failed-attempt throttling, MFA status/disable APIs, configuration,
  API/operator docs, PostgreSQL tests, and live smoke.
- Out: SMS/email codes, WebAuthn/passkeys, mandatory organization policies,
  multiple simultaneous TOTP authenticators, QR image rendering, password reset,
  remote key management/HSM integration, distributed edge throttling, QML login
  screens, social/game behavior, commits, pushes, and pull requests.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/opt-in-totp-two-factor-authentication.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
