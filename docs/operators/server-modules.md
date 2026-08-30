# Production server-module runbook

OmarchyGS ships one disabled-by-default, reviewed first-party server module:
`ignibyte.sentinel` release `10000000-0000-4000-8000-000000000001`. It also
supports up to eight explicitly admitted operator-custom module identities.
Every module observes only declared hooks and can return only typed proposals
that core policy reauthorizes. The first hook observes a minimized persona
report and the only capability is the core-owned `priority_review` label. A
module cannot read report detail, reporter or account identity, credentials,
arbitrary URLs/paths, provider authority, or client/game executable content.

Custom installation and lifecycle are database-local administrator actions.
There is no module upload, marketplace, HTTP, WebSocket, cartridge, or QML
administration route, and placing a Wasm file on disk cannot load it.

## Host prerequisites and opt-in

The server resolves only an executable named `omarchygs-module-host` beside the
running server binary. The service account needs a working systemd user manager
and `/usr/bin/systemd-run`, `/usr/bin/bwrap`, and `/usr/bin/prlimit`. Production
must provision stable, independently backed-up secrets:

```bash
export OGS_MODULE_ADMISSION_SIGNING_SEED='<unpadded-base64url-32-bytes>'
export OGS_MODULE_PAIRWISE_SECRET='<unpadded-base64url-32-bytes>'
# Optional: additionally register the packaged reviewed fixture.
export OGS_FIRST_PARTY_REPORT_MODULE=enabled
```

The two runtime secrets are all-or-none. They enable the generic dispatcher
for already-active custom modules; the optional selector additionally
registers the packaged first-party fixture and must be exactly `enabled`.
Padding, malformed/wrong-length secrets, partial configuration, a missing
sibling host, failed signature/digest/WIT binding, or failed containment
readiness stops the applicable operation. The admission seed signs
server-specific exact grants. The pairwise secret derives module-scoped persona
subjects and must not be reused for another purpose. Neither secret is stored
in PostgreSQL or a database dump.

With both secrets absent, the server still records a bounded
`runtime_unconfigured` observation gap for active custom subscriptions but
runs no module worker or host. This preserves core availability without
silently claiming that custom behavior executed.

## Install an operator-custom release

Treat the publisher release, component, public key, command, and provenance
private key as one reviewed custody set. All five paths must be absolute. Every
file must be a nonempty regular file owned by the invoking effective user,
mode 0600, single-link, and unchanged across the bounded no-follow read.
Components are capped at 2 MiB and must implement the exact production WIT as
a no-WASI WebAssembly component.

The publisher public-key document has format
`omarchygs.server-module-public-key/v1`; the operator provenance private-key
document has format `omarchygs.server-module-private-key/v1`. Both use
`algorithm: ed25519`, a bounded `key_id`, and respectively a canonical
`verifying_key` or private `signing_seed`. Confirm their lowercase SHA-256
fingerprints out of band, then create a canonical import command such as:

```json
{
  "format": "omarchygs.operator-custom-module-import-command/v1",
  "operation_id": "11111111-1111-4111-8111-111111111111",
  "server_id": "58ee076d-0216-422c-b1e2-48ee7fa648bb",
  "signed_release_path": "/srv/omarchygs/import/signed-release.json",
  "component_path": "/srv/omarchygs/import/component.wasm",
  "publisher_public_key_path": "/srv/omarchygs/import/publisher.public.json",
  "provenance_private_key_path": "/srv/omarchygs/secrets/custom-module.private.json",
  "publisher_key_sha256": "<lowercase-sha256>",
  "provenance_key_sha256": "<lowercase-sha256>",
  "granted_capabilities": ["moderation_add_label"],
  "initial_config": {},
  "initial_state": {},
  "acknowledgement": "I understand this module is unreviewed and unsupported by OmarchyGS.",
  "actor": "local-sysop",
  "reason": "Reviewed locally for this community"
}
```

Run the private command from the same build that supplies the packaged host:

```bash
DATABASE_URL="$DATABASE_URL" \
  omarchygs-admin custom-module-import /absolute/path/import-command.json
```

Import verifies the publisher signature, exact component digest, WIT and
budgets, requested/granted subset, fingerprints, stable server UUID, and a
fresh contained readiness probe before retaining immutable component and trust
evidence. It creates a new module identity in `disabled`, or stages an
immutable later release without changing the selected one. Reusing an
operation UUID replays only the exact same canonical command; changed intent
conflicts. Importing the exact release again must name the same immutable grant.

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

Reviewed-fixture lifecycle commands keep their existing format. Custom module
lifecycle commands use the same private-file custody rules as import and bind
the exact instance plus all three mutable revisions:

```json
{
  "format": "omarchygs.operator-custom-module-lifecycle-command/v1",
  "action": "enable",
  "operation_id": "22222222-2222-4222-8222-222222222222",
  "instance_id": "<inventory-instance-uuid>",
  "expected_lifecycle_revision": 1,
  "expected_config_revision": 1,
  "expected_state_revision": 0,
  "actor": "local-sysop",
  "reason": "Enable after local review"
}
```

```bash
DATABASE_URL="$DATABASE_URL" \
  omarchygs-admin custom-module-apply /absolute/path/module-command.json
```

The example must contain one `action`; shown actions are `enable`, `disable`,
`suspend`, `recover`, `upgrade`, `rollback`, and terminal `remove`. Enable and
recover run contained readiness and publish a fresh exact admission. Upgrade
also requires `target_release_id` and a complete bounded `candidate_state`;
it atomically retains the immediate predecessor snapshot and terminalizes stale
work. Rollback can restore only that immediate predecessor, once. Remove keeps
all audit, receipt, artifact, and state evidence but can never be reversed.

Commands require matching lifecycle, configuration, and state revisions and
are idempotent only for an identical operation body. Disable/suspend stop new
claims and release in-flight leases. A configured server still starts while a
module is disabled, degraded, suspended, retired, or pending restore review.
Never edit instance, release, outbox, receipt, state, or audit rows manually.

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
retained state snapshots, custom component bytes, and public trust evidence. It
excludes the admission/pairwise secrets, every provenance private key, and
compiled/JIT process state. Back those private keys and secrets up separately
under protected key custody; do not add them to a database dump.

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

Restore reconciliation leaves retired modules terminal and forces every other
module to `disabled`, blocks automatic activation, clears stale in-flight
leases and admission selection, and appends immutable audit. Verify
the exact release/component/WIT, receipts, dead letters, state, host package,
and external secrets. Apply `recover` at the resulting revision only after that
review. Custom recovery itself repeats readiness and publishes a new admission;
the reviewed fixture repeats readiness on configured startup. Switch traffic
only after inventory shows the intended lifecycle.

## Disclosure, policy, and support boundary

While any custom module is active or degraded, public discovery exposes only a
server-bound aggregate count, the `moderation_labels` behavior class, and the
permanent warning that the code is unreviewed and unsupported by OmarchyGS.
The official QML client preserves that warning before and after sign-in. It is
not a module inventory and contains no release, path, key, component, config,
state, or operator identity.

An operator who installs custom executable code owns its security, privacy,
availability, moderation, legal-policy, and support consequences. Before
inviting players, publish applicable terms and acceptable-use/privacy rules,
describe the module's data use, designate a security contact and vulnerability
intake, monitor and patch the server/host/component, and rehearse suspension,
backup, restore, and key-compromise response. OmarchyGS sends no project
telemetry on a module's behalf; operator logging and monitoring must remain
bounded and must not capture hook payloads, credentials, or private keys.

On suspected compromise, suspend the module first, preserve inventory/audit/
receipt evidence, rotate affected external keys, patch or stage a new exact
release, repeat containment/readiness, and recover only after review. Project
support covers the OmarchyGS core boundary, not an operator's custom module or
the behavior it requested.

## Local conformance

Run the fixed runtime/containment corpus with:

```bash
./scripts/test-server-modules.sh
```

The canonical delivery proof remains `bin/gate.sh --diff`, which also runs the
PostgreSQL transaction/concurrency/failure corpus and platform restore drill.
