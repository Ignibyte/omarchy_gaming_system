# Remote provider security operations

Status: Ticket 018 installs the operator control plane and broker foundation.
Ticket 019 uses it for one operator-pinned first-party Door Legends release.
Ticket 044 extracts the provider-facing contract into the public-only
`omarchygs-provider-sdk` preview and adds authenticated exact-v1 compatibility
preflight before provider effects. The SDK does not change registration or
pilot admission.
Registration alone still does not transfer gameplay authority or make a
release catalog-visible; the separate pilot activation command is required.
See [`provider-authority-pilot.md`](provider-authority-pilot.md) for the exact
activation, recovery, and retirement procedure.

## Operating model

An operator controls four independent lifecycle layers: provider, exact
release, capability scope, and operational key. Each starts `active`, may be
`suspended`, and may be terminally `revoked`. Every mutation requires a bounded
operator identity and reason, locks the provider/release root transactionally,
and appends immutable audit evidence.

An exact release permanently pins:

- provider ID, release UUID, game key, rules version, and cartridge SHA-256;
- lowercase DNS host, explicit TLS port, and canonical base path;
- active-session policy (`terminate`, `read_only`, or `continue`);
- launch, command, reconciliation, and event scopes;
- immutable Ed25519 message verification keys and DER TLS roots; and
- grant/request/callback rates, cross-process concurrency, body ceilings, and
  connect/total deadlines.

Changing endpoint or game/release identity requires registering a new release.
Key rotation appends a new key with a validity window; it never rewrites an old
key. Keep old and new keys active during a deliberate overlap, confirm the new
key in authenticated audit evidence, then suspend or revoke the old key.

Before each new network attempt, the platform sends a signed compatibility
offer to the release's fixed `compatibility` path. The provider authenticates
that exact provider/release/message context and returns a signed selection.
Current SDK v1 permits exactly protocol version 1 with launch, command,
reconcile, and event capabilities. Negotiation failure releases the request
lease, records a safe failure audit, issues no grant, creates no attempt, and
must not reach provider gameplay state. The selected profile is mandatory in
the later signed grant and every operation, response, and callback body. Grant
issuance also requires the same release configuration revision and the active
message key that authenticated the selection, then returns the freshly locked
security material for the operation. A rotation, revocation, or quota change
during preflight therefore conflicts before a grant or attempt is created.
Durable attempt creation repeats the locked revision, lifecycle, scope, and key
admission and its returned material is the snapshot used for the outbound POST.

## Applying commands

Build or run the adapter from a trusted operator environment:

```bash
export DATABASE_URL='postgres://...'
mise exec -- cargo run -p omarchy-game-provider \
  --bin omarchygs-provider-admin -- apply ./operator-command.json
```

The command file is a strict tagged JSON object no larger than 256 KiB. Restrict
it to the operator because it expresses security policy, even though it must
contain only public provider material. A lifecycle command has this shape:

```json
{
  "command": "set_release_status",
  "actor": "oncall-operator",
  "reason": "contain provider incident INC-1234",
  "release_id": "11111111-2222-4333-8444-555555555555",
  "status": "suspended"
}
```

Rotation uses `command: "rotate_key"`, a `key_kind` of
`message_ed25519` or `tls_root_der`, and this public key object:

```json
{
  "key_id": "provider-message-2026-08",
  "public_material_base64": "<standard-base64-public-bytes>",
  "valid_from": 1787616000,
  "valid_until": null
}
```

Registration additionally supplies the complete `registration` object. Use
`cargo doc -p omarchy-game-provider --no-deps --open` for the platform
registry model and `cargo doc -p omarchygs-provider-sdk --no-deps --open` for
the public protocol contract. Validation is all-or-nothing; unknown fields,
IP literals, local/special hostnames, noncanonical paths, malformed key bytes,
missing scopes, and out-of-range quotas fail before mutation.

## Suspension and revocation

- Suspend a provider or release first when an investigation may be reversible.
  New launches stop immediately. Existing command/reconciliation behavior
  follows the release's pinned active-session policy.
- Suspend one scope when the incident is capability-specific.
- Revoke a compromised message key or TLS root immediately. A revoked key is
  excluded from the next admission even if its validity window remains open.
- Revoke the provider or release when trust is permanently withdrawn.
  Revocation is terminal and stops every new grant and request.
- Do not depend on WebSocket delivery for containment. Admission reads current
  PostgreSQL lifecycle/key/scope state transactionally.

Completed operation receipts remain replayable as exact historical evidence;
revocation prevents new network attempts. This distinction lets callers learn
the already-recorded outcome of an idempotent operation without reopening
provider authority. The platform and Door Legends up-convert only their own
persisted pre-negotiation v1 response/outbox rows to the fixed exact-v1
compatibility profile. New network messages remain strict and reject a missing
compatibility field. The sole exception is an authenticated byte-exact retry
whose message, event, session, revision, and legacy body digest already match
an immutable callback receipt; it resolves only as a duplicate and cannot
project again. Door Legends sends a retained legacy outbox body once for that
lost-ack recovery and upgrades it only after an explicit rejection.

## Failure and recovery

Public broker failures are limited to stable non-disclosing codes such as
`provider_denied`, `provider_quota_exceeded`, `provider_protocol_rejected`, and
`provider_unavailable`. Remote bodies, URLs, database errors, subjects, grants,
and key bytes are not placed in these errors or safe audit details.

A timeout is an unknown outcome. Retry the same semantic operation with its
original release, platform session, idempotency UUID, expected revision, and
payload. The broker creates a fresh short-lived grant/message attempt while
retaining the original intent. A changed intent under the same idempotency key
is a conflict. If the provider remains unavailable, suspend launch admission
and use authenticated reconciliation after recovery; do not infer success from
timestamps.

Quota counters and request leases live in PostgreSQL. Compatibility, grant
preparation, and the provider operation share one aggregate
`total_timeout_ms` deadline, so live provider traffic cannot outlast the
request lease by receiving a second full transport budget. Expired leases
recover after a crashed process. Repeated quota exhaustion, signature
rejection, redirect attempts, body-limit failures, or reconciliation mismatch
should be treated as provider health/security signals and may justify
suspension.

## Audit and validation

`provider_security_audit_events` is append-only. Query it by provider/release
and descending `sequence`; do not copy raw request/response bodies or secrets
into incident notes. Durable operations, attempts, grants, quota windows,
leases, and message receipts provide the correlated evidence needed for retry
and incident reconstruction.

Run the canonical proof after provider-boundary changes:

```bash
scripts/test-provider-conformance.sh
```

This proof uses a separate TLS process and ephemeral keys. It covers real TLS
trust, grants and message signatures, pairwise privacy, revision conflict,
exact replay, commit-then-timeout recovery, signed event deduplication,
redirect/oversized/signature failures, wrong trust roots, outage, restart, and
reconciliation.
