# OmarchyGS remote-provider security foundation

`omarchy-game-provider` is the dormant production trust boundary for future
registered game providers. It does not expose a player route or transfer game
authority. Current compiled games and PostgreSQL game snapshots remain
authoritative until the separate Ticket 019 architecture and migration gates
are accepted.

The crate provides:

- operator-only exact-release registration, lifecycle, scope, key, and quota
  changes with append-only PostgreSQL audit evidence;
- 60-second Ed25519 grants containing one scope and a provider/game pairwise
  persona subject, never an account ID or reusable device credential;
- a fixed RFC 9421-shaped Ed25519 HTTP Message Signature profile with RFC 9530
  `Content-Digest` over exact request, response, and callback bytes;
- HTTPS-only, proxy-free, redirect-free egress that rejects special/private
  DNS results, pins accepted addresses, trusts only registered DER roots, and
  streams under registered body/deadline limits;
- durable idempotency, message receipts, quotas, concurrency leases, and safe
  failure audit; and
- an opt-in, separately spawned TLS provider fixture. The compile-time
  conformance transport admits only the exact generated loopback socket and is
  not linked into the platform server.

The broker is the only future network principal. Cartridges and clients never
receive provider endpoints, provider keys, platform signing material, database
access, or a direct provider transport.

## Operator adapter

The admin binary reads one bounded, strict JSON command and emits one safe JSON
receipt:

```bash
export DATABASE_URL='postgres://...'
mise exec -- cargo run -p omarchy-game-provider \
  --bin omarchygs-provider-admin -- apply ./operator-command.json
```

Supported command tags are `register_release`, `rotate_key`,
`set_provider_status`, `set_release_status`, `set_scope_status`,
`set_key_status`, and `update_quotas`. Registration and rotation accept public
Ed25519 verification keys or public DER TLS roots only. Provider private keys,
platform grant/message signing seeds, pairwise secrets, database URLs, and
account credentials do not belong in command documents.

Operational procedures and command shapes are documented in
[`docs/operators/provider-security.md`](../../docs/operators/provider-security.md).

## Validation

From the repository root:

```bash
mise exec -- cargo test -p omarchy-game-provider
scripts/test-provider-conformance.sh
```

The second command starts PostgreSQL, compiles the conformance-only fixture,
generates all TLS/signing material in a private temporary directory, spawns the
fixture as a distinct process, exercises the hostile/failure corpus, and
removes the ephemeral state when the test exits.
