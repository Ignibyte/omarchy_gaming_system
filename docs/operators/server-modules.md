# Production server-module runbook

OmarchyGS ships one disabled-by-default, reviewed first-party server module:
`ignibyte.sentinel` release `10000000-0000-4000-8000-000000000001`. It observes
new persona reports after their transaction commits and may propose only the
core-owned `priority_review` label. It cannot read report detail, reporter or
account identity, credentials, arbitrary URLs/paths, provider authority, or
client/game executable content.

There is no module marketplace/import path or HTTP administration route in
this release. Placing a Wasm file on disk cannot load it.

## Host prerequisites and opt-in

The server resolves only an executable named `omarchygs-module-host` beside the
running server binary. The service account needs a working systemd user manager
and `/usr/bin/systemd-run`, `/usr/bin/bwrap`, and `/usr/bin/prlimit`. Production
must provision stable, independently backed-up secrets:

```bash
export OGS_FIRST_PARTY_REPORT_MODULE=enabled
export OGS_MODULE_ADMISSION_SIGNING_SEED='<unpadded-base64url-32-bytes>'
export OGS_MODULE_PAIRWISE_SECRET='<unpadded-base64url-32-bytes>'
```

All three variables must be absent or present. The enable token is exactly
`enabled`; padding, malformed/wrong-length secrets, partial configuration, a
missing sibling host, failed signature/digest/WIT binding, or failed containment
readiness stops server startup. The admission seed signs server-specific exact
grants. The pairwise secret derives module-scoped persona subjects and must not
be reused for another purpose. Neither secret is stored in PostgreSQL or a
database dump.

With all variables absent, no module registration, host process, dispatcher,
route, or report behavior is added.

## Inventory and lifecycle

Use only the database-local administrator process:

```bash
omarchygs-admin modules
```

The bounded JSON inventory contains release/component identity, lifecycle and
data revisions, restore state, aggregate queue/receipt counts, saturating
observation-gap count/reason/time, and the count of upgrade-era receipts that
predate complete request evidence. It excludes signed bodies, event payloads,
state values, database configuration, and secrets.

Lifecycle commands are regular, single-link JSON files owned by the invoking
effective user with exact mode 0600. The administrator rejects symlinks,
hard-linked/shared files, and files that change while being read:

```json
{
  "format": "omarchygs.server-module-lifecycle-command/v1",
  "operation_id": "11111111-1111-4111-8111-111111111111",
  "module_id": "ignibyte.sentinel",
  "expected_revision": 2,
  "action": "suspend",
  "actor": "local-sysop",
  "reason": "Emergency stop while reviewing repeated host failures"
}
```

```bash
omarchygs-admin module-apply ./module-command.json
```

Actions are `disable`, `suspend`, `recover`, and terminal `retire`. Commands
require the current lifecycle revision and are idempotent only for an identical
operation body. Disable/suspend stop new claims and release an in-flight lease.
Recover clears a circuit/restore block into `disabled`; restart the configured
server to repeat exact host readiness and create a new admission before active
delivery. A configured server still starts while the persisted module is
disabled, degraded, suspended, retired, or pending restore review; it runs no
module host or dispatcher, and report transactions record `module_inactive`
gap evidence until an operator completes recovery. Never edit instance,
outbox, receipt, state, or audit rows manually.

## Delivery, gaps, and incidents

Events are durable and ordered per exact release/hook/pairwise subject. The
dispatcher makes at most three attempts with bounded backoff. A timeout, trap,
host exit, malformed response, or unavailable host becomes a stable retry and
then a dead letter; three consecutive failures degrade the module and pause new
claims. Already committed reports remain valid.

Dead letters count toward the 1,024-row outstanding cap. At capacity—or whenever
the module is not active—a new core report still commits, but no observation is
queued. The report transaction atomically increments the bounded inventory gap
counter and records `queue_saturated` or `module_inactive`. Treat any increase
as an operational incident: inventory the queue, suspend if needed, preserve
evidence, correct host/runtime availability, then use `recover` and a
readiness-checked restart. This release deliberately has no unsafe dead-letter
deletion or arbitrary replay command.

## Backup and restore

Normal PostgreSQL backup includes immutable release/admission/audit/receipt
evidence—including minimized request and response preimages for new delivery
receipts—mutable lifecycle/outbox rows, gap evidence, labels, namespaced state,
and retained state snapshots. It excludes both module secrets and compiled/JIT
process state. Back up the two module secrets separately under protected key
custody.

A raw database archive preserves the source lifecycle exactly; PostgreSQL does
not automatically identify that it was restored. After restoring into an
isolated database and before **any** server startup against it, create a private
restore command:

```json
{
  "format": "omarchygs.server-module-restore-command/v1",
  "operation_id": "22222222-2222-4222-8222-222222222222",
  "actor": "restore-operator",
  "reason": "Reconcile the 2026-08-27 isolated restore"
}
```

```bash
DATABASE_URL="$RESTORE_DATABASE_URL" \
  omarchygs-admin module-restore ./module-restore.json
DATABASE_URL="$RESTORE_DATABASE_URL" omarchygs-admin modules
```

Restore reconciliation forces every module to `disabled`, blocks automatic
activation, clears stale in-flight leases, and appends immutable audit. Verify
the exact release/component/WIT, receipts, dead letters, state, host package,
and external secrets. Apply `recover` at the resulting revision only after that
review, then start with the opt-in configuration and require readiness before
switching traffic.

## Local conformance

Run the fixed runtime/containment corpus with:

```bash
./scripts/test-server-modules.sh
```

The canonical delivery proof remains `bin/gate.sh --diff`, which also runs the
PostgreSQL transaction/concurrency/failure corpus and platform restore drill.
