# Door Legends remote-authority pilot operations

Status: Ticket 019 enables one first-party, operator-pinned Door Legends v1
release. This is not a general provider onboarding path. External providers,
self-service registration, dynamic discovery, and direct client-provider
networking remain prohibited.

## Authority and failure rules

Door Legends owns its rules, private game state, revision, time, outcome,
operation receipts, and callback outbox in a separate PostgreSQL database.
OmarchyGS owns login, personas, the catalog decision, the platform session
envelope, provider grants, authenticated public view/result/achievement
projections, sync invalidations, and audit.

Never copy Door Legends gameplay tables into the OmarchyGS database. Never run
a provider-owned session through compiled rules. If the provider is unavailable
or its pilot is suspended, existing sessions are read-only until authenticated
reconciliation succeeds. Retirement is permanent.

## Platform runtime configuration

Provider support is optional and all-or-none. Leave all four values absent to
run compiled games only. Setting only some values, malformed base64url, a
secret of the wrong length, or a non-lowercase DNS callback authority prevents
server startup.

```text
OGS_PROVIDER_GRANT_SIGNING_SEED       unpadded base64url, exactly 32 bytes
OGS_PROVIDER_PAIRWISE_SECRET          unpadded base64url, exactly 32 bytes
OGS_PROVIDER_MESSAGE_SIGNING_SEED     unpadded base64url, exactly 32 bytes
OGS_PROVIDER_CALLBACK_AUTHORITY       lowercase DNS host, optional port
```

Generate and retain production secrets in the deployment secret manager. Do
not place them in this repository, operator JSON, shell history, logs, support
notes, or database backups. Supply the corresponding public grant and message
verification keys to the Door Legends deployment.

The Door Legends provider requires its own `DATABASE_URL`, TLS certificate and
private key, immutable release UUID and cartridge digest, registered endpoint
authority, both platform public keys, its provider message-signing seed, the
exact HTTPS callback URL, and the callback TLS root. The
`DOOR_LEGENDS_CALLBACK_SOCKET_OVERRIDE` variable is conformance-only; a
production build rejects it.

## Register and activate

First apply the exact release registration described in
[`provider-security.md`](provider-security.md). Pin the release UUID, game key
`door-legends`, rules version `1`, verified cartridge SHA-256, HTTPS DNS
endpoint, active-session policy, all four scopes, provider message key, TLS
root, and bounded quotas. Then activate the already-registered release:

```json
{
  "command": "activate_pilot",
  "actor": "oncall-operator",
  "reason": "enable the reviewed Door Legends v1 authority pilot",
  "pilot": {
    "release_id": "11111111-2222-4333-8444-555555555555",
    "display_name": "Door Legends",
    "min_human_players": 1,
    "max_human_players": 1,
    "achievements": [
      {
        "key": "first_escape",
        "display_name": "First Escape",
        "description": "Escape through the sunlit gate."
      }
    ]
  }
}
```

Apply it using `omarchygs-provider-admin` as documented in the security
runbook. Activation is an exact replay-safe operation. Only one pilot release
may be active, and its catalog policy and achievement definitions are
immutable. A different policy requires a new release and explicit review.

Before admitting players, confirm the public catalog shows exactly the
expected release and `authority: "registered_provider"`; confirm both services
trust the expected keys and TLS roots; then run:

```bash
scripts/test-provider-authority-pilot.sh
```

## Suspend, recover, and reconcile

To stop new launches while preserving a possible recovery, apply:

```json
{
  "command": "set_pilot_status",
  "actor": "oncall-operator",
  "reason": "contain incident INC-1234",
  "release_id": "11111111-2222-4333-8444-555555555555",
  "status": "suspended"
}
```

Also suspend or revoke the provider, release, scope, or key when that narrower
control is required. Preserve the platform and provider databases separately.
Restore the provider database first, start the exact pinned provider release,
verify its TLS/message identity, and reconcile affected sessions through the
participant API using their last authenticated revision. Do not infer command
success from time or from a cached view.

After health and audit evidence agree, set the pilot back to `active`. Review
`provider_security_audit_events`, provider operation attempts/receipts, callback
receipts, and the Door Legends outbox by stable IDs. Keep raw bodies, grants,
subjects, credentials, and keys out of logs and incident notes.

## Backup and restore drill

Back up OmarchyGS and Door Legends independently with database-native tooling.
Restore Door Legends into an isolated database and verify its sessions,
operation receipts, grant receipts, and callback outbox before any production
cutover. Restore OmarchyGS separately and verify only platform envelopes and
public projections exist there. The canonical gate performs this isolated
Door Legends `pg_dump`/`pg_restore` proof on every provider-boundary delivery.

## Permanent retirement

Retire only after confirming no recovery path is wanted:

```json
{
  "command": "set_pilot_status",
  "actor": "oncall-operator",
  "reason": "permanently end the Door Legends v1 pilot",
  "release_id": "11111111-2222-4333-8444-555555555555",
  "status": "retired"
}
```

Retirement is terminal and cannot be changed back to `active` or `suspended`.
The platform retains participant-authorized history and validated projections,
but never adopts provider gameplay state or activates a compiled fallback. A
future Door Legends service must be reviewed and registered as a new immutable
release.
