# Operator safety and platform recovery

Status: private-alpha local operator workflow. This is a narrow reversible
containment and recovery surface, not a remote administration system.

## Authority boundary

`omarchygs-admin` is a command-line program that uses the PostgreSQL authority
in `DATABASE_URL`. It does not start a listener, create an administrator token,
add an account role, or expose account ownership through the player API. Run it
only from a trusted operator environment with a reviewed database URL. Treat
that database credential and report inventory as sensitive operational data.

Every mutation requires a UUID operation identity, a 1–64 character operator
name, and a 1–500 character reason. Values must be trimmed and contain no
control characters. Successful actions append an immutable audit event in the
same database transaction. Standard output is JSON; failures write only a
stable `operator_*` code to standard error.

Build the local command with:

```bash
cargo build -p omarchy-gaming-system-server --bin omarchygs-admin
```

## Manage registration invitations

Private-alpha account creation requires one operator-issued invitation per
account. Issue, inventory, deliver, and revoke those codes through the
[private-alpha runbook](private-alpha.md). The first successful issue receipt
is the only output containing the raw code; PostgreSQL, audit, later operation
replays, and `invites` inventories retain metadata and a digest only. If first
delivery is lost, revoke that invitation and issue another rather than trying
to recover it from the database or a backup.

Invitation issue and revocation use the same bounded mode-0600 JSON command
files and immutable audit boundary as report/account actions. They do not add
a remote admin endpoint or grant an account administrator authority.

## Review reports

Players file persona-targeted reports from the Social screen. List up to 100
newest-first reports with:

```bash
DATABASE_URL="$DATABASE_URL" target/debug/omarchygs-admin reports open 100
```

The status is one of `open`, `resolved`, `dismissed`, or `all`; the limit is
1–100. The trusted local inventory contains category/detail, timestamps, the
reporter and subject public profiles, and `subject_account_id` so the operator
can target containment. It does not contain passwords, password hashes, raw
tokens, token digests, session inventory, or report idempotency keys. Do not
copy report detail into public logs or support channels without an appropriate
privacy and retention policy.

Disposition one open report by creating a mode-0600 command file:

```json
{
  "command": "set_report_status",
  "idempotency_key": "cc140a63-6acc-4b50-8f22-8726cbe497d5",
  "report_id": "edceff52-2e75-4e3c-ae92-ab09b1f510f0",
  "status": "resolved",
  "actor": "oncall-sysop",
  "reason": "Reviewed the report and completed the response"
}
```

Then apply it:

```bash
DATABASE_URL="$DATABASE_URL" target/debug/omarchygs-admin apply ./resolve-report.json
```

`dismissed` is the other terminal result. Report disposition preserves the
original report and cannot be changed through this command. An exact command
retry returns the original audit receipt; the same operation UUID with changed
intent conflicts.

## Suspend and reactivate an account

Use the subject account UUID from the local report inventory:

```json
{
  "command": "set_account_status",
  "idempotency_key": "b9d44e1c-819b-4700-bfbf-e1c73417b639",
  "account_id": "57d63f03-4ec5-4a67-97bd-7e16b60aa2d5",
  "status": "suspended",
  "actor": "oncall-sysop",
  "reason": "Temporary containment during report review"
}
```

Apply it with the same `apply` command. Suspension locks the account, changes
its state, revokes every currently live device session at one transaction
timestamp, and appends the audit event before commit. Authenticated HTTP and
WebSocket use is then denied. Personas, messages, reports, game history, MFA
configuration, and provider state are retained.

Reactivation uses a new operation UUID and the same document with `status` set
to `active`. It restores password/MFA login eligibility but never clears a
session's `revoked_at`; the player must authenticate again on each device.
`disabled` is a distinct stronger state and this reversible command cannot
change it.

Suspension is the rollback for an accidental reactivation. Reactivation is the
rollback for a suspension after review. Report dispositions and audit events
are intentionally not reversible; correct an operator mistake with a new
documented action where the supported state machine permits it, never by
editing audit or report rows.

## Backup and isolated restore

The platform database includes invitation digests and lifecycle metadata,
accounts, hashed credentials, encrypted MFA state, session digests, personas,
social/inbox data, game history, reports, the last verified marketplace
snapshot, reviewed release inventory, local cartridge selections, and immutable
operator/catalog audit. Protect dumps as secrets, encrypt them at rest,
restrict file permissions, retain off-host copies, and define a tested
retention/deletion policy. The provider authority pilot uses a separate
database and requires its own documented backup.

A production backup can use PostgreSQL custom format:

```bash
umask 077
pg_dump "$DATABASE_URL" --format=custom --file=omarchygs-platform.backup
```

Restore into a newly created isolated database, never over the running source:

```bash
pg_restore --exit-on-error --no-owner \
  --dbname="$RESTORE_DATABASE_URL" omarchygs-platform.backup
```

Before switching traffic, compare table counts and focused security/history
state, start the production server against the isolated restore, prove revoked
and suspended sessions remain denied, compare the public `server_id` from
`/.well-known/omarchygs`, compare marketplace/catalog rows and admission
revisions, and exercise the operator, catalog-audit, and identity immutability
guards. The separately stored cartridge root and marketplace/TLS key material
are not contained in PostgreSQL; restore them from their own protected backup
and verify exact digests before resuming sync or distribution.
A restore of the same community must retain that UUID. A database fork creates
two deployments claiming the same identity and is not a supported multi-server
setup. A deployment switch should be an explicit, monitored infrastructure
change with the old database retained read-only long enough for rollback.
Schema rollback uses a later forward migration, not a down script.

The repository's destructive-safety proof owns two generated databases and
drops only those validated names:

```bash
./scripts/test-operator-recovery.sh
```

It applies the embedded migrations through the production server, seeds
representative identity/social/inbox/game/report state, drives the real sysop
command, performs `pg_dump`/`pg_restore`, compares every public application
table, checks linked audit and immutability, and rejects a pre-suspension token
through the restored production server. It also proves the singleton server
UUID, marketplace snapshot, reviewed releases, local selection, and catalog
audit are exactly preserved before and after the dump/restore.

## External key custody

`OGS_MFA_ENCRYPTION_KEY` is not stored in PostgreSQL and is not present in a
database dump. Back up the exact base64url 32-byte key separately under
protected key-management procedures and restore it with the database. Losing
it prevents enrolled accounts from verifying TOTP; substituting another key
does not decrypt existing authenticators. Do not place this key in a dump,
operator JSON, shell history, repository, log, or support note.

The marketplace public key and explicit TLS root are configuration trust
anchors, while marketplace private signing keys never belong on a community
server. The descriptor-relative cartridge store is separate persistent state;
back it up with exact ownership/modes and content digests. Provider
grant/message secrets, TLS private keys, backup-encryption keys, and future
module signing keys are also separate custody domains.

## Current limitations

These controls do not provide permanent bans, remote administrator accounts,
roles/approval workflows, appeals, evidence attachments, content deletion,
automated moderation, report notifications, legal retention policy, scheduled
or encrypted backup infrastructure, federation, or cross-server moderation.
The project does not monitor or moderate independently operated communities.
Each operator remains responsible for TLS, host/database hardening, access
control, monitoring, incident response, privacy/retention policy, backups, and
restore practice.
