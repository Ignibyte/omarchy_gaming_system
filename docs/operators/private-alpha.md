# Invite-only private-alpha runbook

Status: software admission workflow ready; the first external two-installation
run must still be executed and recorded honestly.

This runbook admits a small reviewed group to one owner-operated OmarchyGS
server. It does not turn the project into a hosted service or make the project
responsible for an independently operated community.

## Go/no-go preconditions

Do not invite a player until all of these are true:

- the server and native client come from one identified reviewed release, and
  its canonical `bin/gate.sh --diff` receipt is green;
- the public server origin uses valid HTTPS. The official client rejects
  non-loopback plaintext HTTP, but the operator still owns TLS termination,
  renewal, firewalling, host hardening, and safe reverse-proxy limits;
- PostgreSQL access is restricted, backups are encrypted and stored off-host,
  and an isolated restore has passed `scripts/test-operator-recovery.sh`;
- the exact `OGS_MFA_ENCRYPTION_KEY` is in protected separate custody and can
  be restored with the database;
- the operator can run `omarchygs-admin`, review reports, suspend/reactivate
  accounts, revoke invitations, and contact the invited testers;
- monitoring covers availability, storage, database health, TLS expiry, and
  unusual request volume; and
- testers have been told who operates the server, what data it holds, how to
  report a safety/security problem, and that this is pre-release software.

The application does not supply distributed registration/login throttling,
DDoS protection, automated TLS, telemetry, crash upload, support ticketing, or
legal/privacy-policy approval. Put appropriate controls at the public edge and
do not treat an unguessable invitation as a substitute for request limits.

## Issue one invitation

Build the database-local command and confirm `DATABASE_URL` names the intended
community database:

```bash
cargo build -p omarchy-gaming-system-server --bin omarchygs-admin
```

Create a mode-0600 command document. `valid_for_hours` is 1–720; use the
shortest practical delivery window. The raw invitation is not an input:

```json
{
  "command": "issue_registration_invite",
  "idempotency_key": "773d4b6e-1fe4-47f4-a474-c5b13e478389",
  "label": "alpha tester 01",
  "valid_for_hours": 72,
  "actor": "oncall-sysop",
  "reason": "Admit one reviewed private-alpha tester"
}
```

```bash
umask 077
DATABASE_URL="$DATABASE_URL" \
  target/debug/omarchygs-admin apply ./issue-invite.json
```

The first exact JSON receipt contains an `ogsi_...` code and
`first_delivery: true`. Capture it without placing it in shell history, logs,
the command document, the repository, a ticket, or a shared channel. An exact
operation retry returns the same durable metadata with
`first_delivery: false` and no `invite_code`; raw codes are deliberately not
recoverable from PostgreSQL or backups.

List bounded metadata without secrets:

```bash
DATABASE_URL="$DATABASE_URL" \
  target/debug/omarchygs-admin invites issued 100
```

The filter is `issued`, `used`, `expired`, `revoked`, or `all`; the limit is
1–100. At most 500 unexpired issued codes can coexist. Inventory shows the
label, derived state, timestamps, and—after use—the private account username.
It never returns a raw code, digest, password, token, session, account UUID, or
persona mapping.

Deliver each code one-to-one over a protected channel already associated with
the intended tester. Do not reuse a code, post it to a group, or ask a tester
to send it back. The code is a temporary account-creation bearer, not a login
credential and not a cross-server identity.

## Revoke an invitation

If delivery went to the wrong person, first output was lost, the tester
declines, or the window should close early, use a new operation UUID:

```json
{
  "command": "revoke_registration_invite",
  "idempotency_key": "c1d7636b-f914-472e-b2b1-72327a9da999",
  "invite_id": "c86268fb-123a-41fa-8e08-2ded86c2d35d",
  "actor": "oncall-sysop",
  "reason": "Invitation delivery channel was no longer trusted"
}
```

Apply it with `omarchygs-admin apply`. Revocation is idempotent under the same
operation UUID and is audited. A used, expired, absent, or already-revoked
invitation cannot be made usable through this command. Revoking a consumed
code does not suspend its account; use the separately audited account
suspension workflow when containment is required.

## Clean-client onboarding

Give the tester the HTTPS server URL, native Omarchy client package/install
instructions, one invitation code, the operator/security contact, and the test
window. Never send a starter password. The tester should:

1. launch the packaged OmarchyGS client and enter the exact HTTPS origin;
2. choose **Create Account**, enter the invitation, choose a unique private
   account username and a new 12–128 byte password, then submit;
3. sign in explicitly and create a public persona; and
4. opt into authenticator-app TOTP when practical, storing the ten one-time
   recovery codes outside the client device.

The invitation and password are masked and cleared by the client. A lost
registration response can be retried with the same three values; exact proof
returns the original receipt. Any changed intent receives the same invalid-
invitation message as an expired, revoked, unknown, or consumed code.

## First external acceptance run

Use two clean Omarchy installations and two distinct invitations. Record the
server release/commit, client package version and digest, origin, date/time,
operator, sanitized tester labels, and pass/fail for each item—never passwords,
invitations, Bearers, MFA secrets/codes, database URLs, or private message
contents.

- Both clean clients connect over HTTPS, register independently, sign in, and
  create distinct personas without developer intervention.
- One persona requests the other; the second accepts; both see the connection
  after an authoritative refresh.
- Both directions exchange private messages and unread state behaves as
  expected.
- One client goes offline, activity occurs, it reconnects, and durable REST
  state—not a WebSocket assumption—recovers the correct inventory/history.
- One persona creates a Signal Siege Versus challenge, the other accepts, both
  finish the match with keyboard-only controls, and both see the same terminal
  outcome/history after reconnect.
- A tester submits a clearly labeled test report against the cooperating test
  persona; the operator sees it locally and dismisses it with an audit reason.
- Each client closes through the visible EXIT control and can sign in again.
- The operator verifies both invitations are `used`, no unused test codes
  remain, a post-run backup completes, and a restore/incident procedure is
  still available.

The repository’s deterministic software rehearsal is:

```bash
./scripts/test-private-alpha.sh
```

It proves the real CLI → PostgreSQL → production server invitation boundary in
an isolated generated database, including issue, registration, exact replay,
changed-intent denial, login, revocation, metadata-only inventory, and audit.
It is not evidence that external humans or two physical installations ran the
acceptance checklist.

## Feedback and stop conditions

Ask for reproducible steps, expected/actual behavior, release/package
identity, timestamps, screen/state names, and sanitized screenshots. Player
safety reports belong in the in-product report flow. Suspected vulnerabilities
or credential exposure go directly to the operator/security contact, not a
public issue containing secrets or private data.

Stop new invitations and close public ingress while investigating any of:

- leaked database, MFA, session, invitation, TLS, provider, or signing secret;
- invalid TLS or traffic reaching the server outside the intended origin;
- cross-account/persona data exposure or unauthorized mutation;
- suspended/revoked authority remaining usable;
- database corruption, failed backup, or failed isolated restore;
- repeatable crash/data loss in the first-playable path; or
- abuse volume that the current manual moderation and edge controls cannot
  contain.

Revoke every unused code, suspend affected accounts when appropriate, retain
audit and backups, document the incident without secrets, and resume only
after the cause and recovery evidence are understood. The operator remains
responsible for availability, moderation, retention, privacy, support, and
applicable law for that deployment.
