# Provider deployment and operations

This runbook covers the reviewed remote and co-located sidecar profiles for an
already reviewed, exactly registered provider release. It does not register,
activate, discover, publish, or approve a provider. Door Legends v1 remains the
only production-admitted provider; use Relay Forge only for local conformance
and drills.

Read the [provider threat model](../security/provider-sidecar-threat-model.md),
[provider security runbook](provider-security.md), and
[Door Legends authority runbook](provider-authority-pilot.md) before changing a
deployment. The release registration is immutable: a different endpoint,
binary identity, game/rules/cartridge identity, or incompatible state schema
requires a reviewed new release.

## Invariants shared by both profiles

- OmarchyGS owns player authentication, provider registration/admission,
  grants, lifecycle, quotas, audit, the session envelope, and authenticated
  public projections. The provider owns rules state and revisions only.
- Use different OS users, PostgreSQL roles and databases, runtime secrets,
  configuration/writable directories, backups, and service lifecycles. Never
  give the provider the platform database URL or the platform service the
  provider database URL.
- The provider receives pairwise game-scoped subjects and one-scope expiring
  grants, never account/persona IDs, device credentials, client connectivity,
  arbitrary egress, or platform administration.
- Keep the platform grant/message signing seeds and pairwise secret only in the
  platform secret store. Give the provider only the corresponding public keys.
  Keep the provider message seed and TLS private key only with the provider.
- Preserve the canonical HTTPS DNS authority, registered TLS roots, fixed
  paths, message signatures, exact-v1 compatibility, request/response bounds,
  aggregate deadlines, replay receipts, quotas, and current lifecycle checks.
  Loopback port ownership is not authentication.
- A failed or unknown operation makes the affected session read-only. Do not
  retry a different intent or infer success from time. Restore service/state,
  then use authenticated reconciliation; never copy a cached view into rules
  state or select compiled fallback.
- OmarchyGS serializes command and reconciliation requests with a durable,
  deadline-bounded session reservation and a process-held PostgreSQL advisory
  fence around provider transport. A live fence rejects competing operations
  even if the visible reservation expires; a delayed original process must
  revalidate its reservation after acquiring the fence. An abandoned expired
  reservation first changes a formerly ready session to `reconciling`; only
  reconciliation may then reach the provider. Do not clear these fields or
  advisory locks by hand.

## Remote profile

### Provision and register

These steps make TLS identity, DNS/endpoint immutability, database ownership,
and secret custody explicit for a reviewed release.

1. Allocate a stable lowercase DNS name and explicit TLS port. The name must
   resolve only to globally routed provider addresses; private, loopback,
   link-local, metadata, special-use, and mixed DNS answers fail closed.
2. Terminate TLS in the provider process or a dedicated provider-owned proxy.
   Register only the bounded DER trust roots that authenticate that exact DNS
   name. Proxies and redirects are disabled by the OmarchyGS client, so the
   canonical endpoint must serve the four fixed provider paths directly.
3. Create one provider-only PostgreSQL role and database. Restrict its network
   and host authentication rules to the provider service. Apply provider
   migrations with the exact release artifact and verify that no OmarchyGS
   platform tables or credentials are present.
4. Place service secrets in the deployment secret manager with least-privilege
   read access. Files must be private, regular, non-symlink inputs; never store
   seeds, database URLs, grants, subjects, or TLS private keys in this
   repository, operator command JSON, logs, shell history, or tickets.
5. Register the immutable release, public message keys, TLS roots, active-
   session policy, four required scopes, and conservative quotas using the
   database-local provider administrator. Registration is not activation.
6. Run the public developer-kit and conformance suites against a staging
   deployment. Verify exact compatibility, TLS/message identities, separate
   database ownership, callbacks, replay, timeout/unknown-outcome recovery,
   outage/restart, and reconciliation before the separate admission decision.

### Remote network and monitoring

Allow inbound provider TLS only from expected OmarchyGS egress networks when
the deployment topology permits it. Allow provider outbound traffic only to
its PostgreSQL service, the exact platform callback origin, DNS/time services
required by the host, and explicit monitoring destinations. Do not add a
generic callback proxy or redirect.

Monitor, without logging bodies or secrets:

- process/TLS endpoint health and exact-v1 compatibility latency;
- connect and aggregate request deadline failures;
- grant/request/callback quota exhaustion and concurrency lease recovery;
- signature, digest, context, redirect, body-limit, and reconciliation
  rejection counts by stable provider/release/key IDs;
- provider PostgreSQL health, connection saturation, migration version,
  backup freshness, restore verification, receipt/outbox backlog, disk usage,
  and replication status when used;
- platform `provider_security_audit_events` and affected session availability.

Page the provider operator on sustained unavailability, authentication
rejections, callback backlog, database errors, quota saturation, or restore
freshness failure. The OmarchyGS operator decides suspension/revocation; a
provider health endpoint does not grant launch authority.

## Co-located sidecar profile

The sidecar uses TLS over one exact loopback TCP socket. OmarchyGS maps only
the configured release UUID to that socket and still sends the registered DNS
URL as SNI, Host, and signed authority. Every other release uses normal guarded
public DNS. The socket port must equal the immutable registered endpoint port.

Reviewed templates live under [`deploy/provider-sidecar/`](../../deploy/provider-sidecar/):

- `platform.env.example` contains only the release UUID/socket routing pair;
- `provider-config.example.json` is the canonical compact Relay Forge/starter
  shape and keeps the conformance override null;
- `omarchygs-provider-sidecar@.service` supplies a separate user, read/write
  paths, restart bounds, resource ceilings, no capabilities, and loopback-only
  IP policy; and
- `platform-callback.Caddyfile.example` terminates the platform callback TLS
  identity on one loopback socket and forwards to the existing local HTTP
  server with Caddy's administrative API disabled. Replace Caddy with another
  reviewed proxy only if it preserves the exact authority, certificate, port,
  path, bounds, no-redirect behavior, and has no mutable management listener.

The callback proxy template is a complete Caddyfile, not a site fragment. If
an operator deliberately merges it into another complete Caddyfile, merge
`admin off` into that file's single global options block and re-review every
additional listener and route.

### Host preparation

For instance `door-legends`, provision a dedicated locked service account such
as `omarchygs-provider-door-legends`. Create:

```text
/etc/omarchygs/providers/door-legends/       provider user, mode 0700
  config.json                                provider user, mode 0600
  provider-cert.pem                          provider user, mode 0600
  provider-key.pem                           provider user, mode 0600
/var/lib/omarchygs/providers/door-legends/   provider user, mode 0700
```

Create a provider-only PostgreSQL role/database and store its URL only in the
private provider config. Use a Unix socket with database role authentication or
an exact loopback database port. Do not reuse the OmarchyGS role, database,
schema, connection pool, backup, or migration owner.

Materialize the service template with an absolute, reviewed provider binary.
The provider config's `authority`, `release_id`, cartridge digest, public
platform keys, TLS files, and callback target must exactly match registration.
For a starter-based provider, set `callback_sidecar_socket` to the local TLS
proxy, `callback_socket_override` to null, and
`command_response_delay_ms` to zero. For Door Legends set
`DOOR_LEGENDS_SIDECAR_CALLBACK_SOCKET`; never set its conformance-only callback
override or response delay in production. Both starter and Door Legends
callback clients ignore ambient HTTP(S)/all-proxy environment variables; do
not weaken that behavior or route callbacks through a generic proxy.

Configure OmarchyGS with its existing four provider runtime values plus:

```text
OGS_PROVIDER_SIDECAR_RELEASE_ID   exact registered release UUID
OGS_PROVIDER_SIDECAR_SOCKET       exact nonzero loopback provider TLS socket
```

The pair is all-or-none. A nil/malformed UUID, non-loopback/zero socket,
partial pair, or sidecar values without the complete provider runtime prevents
startup. A registered port mismatch, wrong TLS identity, hostile local
listener, wrong signed provider identity, ambient proxy, or redirected path fails transport
before provider effects. Do not add `/etc/hosts`, DNS, proxy, firewall, or CIDR
exceptions to make another private address work.

Start the callback TLS proxy, provider PostgreSQL, provider sidecar, and then
OmarchyGS. Confirm the provider user cannot read platform config/state, the
platform user cannot read provider config/state, both database URLs select
different databases/roles, the exact TLS certificates and message key IDs are
observed, and compatibility selects only v1 with all four capabilities.

## Lifecycle and recovery drills

Run `scripts/test-provider-sidecar.sh` after transport, provider runtime,
template, or operations changes. It uses ephemeral keys and databases to prove
the exact production sidecar transport, reject a hostile TLS peer holding the
port, crash/restart the provider, deny a new launch during outage, reconcile an
existing session after restart, verify callback recovery, restore the provider
database separately, validate containment templates, and verify a locally
signed secret-free bounded receipt. The full delivery gate also runs the Door
Legends player/API lifecycle, hostile ambient-proxy callback proof, concurrent
operation reservation proof, and independent restore proof.

Use this order in a real maintenance or incident exercise:

1. **Prepare:** record release/config/key IDs and current audit/session/outbox
   counts without copying secret values or bodies. Confirm recent independent
   backups and an isolated restore target.
2. **Suspend:** suspend the pilot/release or launch scope before planned outage.
   New launches stop. Existing affected sessions retain only their last
   authenticated view and reject commands while non-ready.
3. **Stop/crash:** stop the provider service and confirm a new launch cannot
   reach provider state. Confirm platform login, social, compiled games, and
   provider session reads remain platform-owned and available.
4. **Upgrade:** back up provider state, install an exact reviewed binary, run
   migrations from that artifact, and restart without changing the registered
   release identity. An incompatible endpoint, rules identity, state schema,
   or wire contract requires a new release and migration plan, not an in-place
   identity rewrite.
5. **Lost database/restore:** keep the provider stopped; restore its database
   into an isolated provider-only database; verify identity, sessions,
   consumed grants, operation receipts, and callback outbox; then cut the
   provider config to that database. Never restore provider tables into the
   platform database or rebuild them from platform projections.
6. **Recover:** start the exact provider and verify TLS/message identities and
   compatibility. Reconcile each affected session using its stable platform
   session, expected revision, and a fresh idempotency UUID. A mismatch stays
   read-only and escalates; timestamps do not choose a winner.
7. **Resume:** inspect authenticated response/callback and audit evidence, then
   reactivate the suspended layer. Confirm new launch and command only after
   existing sessions report `ready`.

## Rotation, incident response, and end-of-life

For provider message-key or TLS-root rotation, append the new public key/root
with a bounded validity window, deploy the corresponding provider secret,
retain an overlap, confirm authenticated compatibility/response/callback audit
under the new key, and only then suspend or revoke the old material. Rotate
platform grant/message keys through the provider's public-key configuration
with the same overlap discipline. Rotate database credentials separately and
verify that the old role can no longer connect. Never rewrite historical key
rows or operation/callback bytes.

On suspected compromise, preserve bounded audit/receipt IDs, suspend the
narrowest affected provider/release/scope/key, stop the sidecar if local
containment is required, revoke confirmed compromised keys, rotate database
credentials, and restore from known-good independent backups. Do not publish
incident details, contact external providers, or claim support coordination
without the required external authority. Repeated wrong-TLS, signature,
redirect, body-limit, quota, or reconciliation failures are security signals,
not reasons to weaken the profile.

For end-of-life, first deny new launch, notify affected players through the
separately authorized communication process, preserve required platform and
provider audit/backups, reconcile or explicitly close active sessions under
the pinned policy, revoke release/key authority, and retire the pilot/release.
Retirement is terminal. Remove the sidecar mapping, service, private secrets,
and provider database only after retention and recovery obligations are met.
The platform retains its envelope and authenticated public history; it never
adopts provider gameplay state or silently substitutes another release.
