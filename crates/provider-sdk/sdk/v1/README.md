# OmarchyGS Provider SDK v1 preview

This is the provider-facing Rust contract for one registered OmarchyGS game
release. It contains protocol models, exact-v1 compatibility negotiation,
pairwise identity, signed grants, RFC 9421/9530 message helpers, bounded JSON
validation, deterministic export verification, schemas, fixtures, and signed
local release provenance.

The platform sends a signed compatibility offer to the fixed
`compatibility` path before every new launch, command, or reconcile attempt.
The provider verifies the platform signature and returns a provider-signed
selection. SDK v1 accepts only protocol version 1 with all four capabilities:
`game.launch`, `game.command`, `game.reconcile`, and `game.event`. Missing,
unknown, reordered, partial, extra, duplicate, or downgrade profiles reject.
The exact selection is then mandatory in the signed grant and every operation,
response, and callback body.

Upgrades may use `ProviderOperationResponse::from_persisted_v1_bytes` and
`ProviderEvent::from_persisted_v1_bytes` only for authenticated local receipts
or outbox rows written before compatibility became mandatory. These helpers do
not relax network decoding: a new request, response, or event without the exact
compatibility field is rejected.

Verify signatures and exact context before parsing or mutating state. Providers
receive only pairwise game-scoped subjects and one-scope short-lived grants.
They never receive account/persona identity, device credentials, platform
database access, arbitrary egress, client executable privilege, or direct
client connectivity.

This SDK contains no registry, broker, egress policy, administrator operation,
database migration, provider key, or automatic admission. Possessing or
building it does not register, activate, discover, trust, list, or publish a
provider. Door Legends remains the sole authorized provider until a later
reviewed onboarding pipeline completes.

`sdk-lock.json` pins every compiled-owned byte. `sdk-release.json` binds that
lock to one project release authority, key, source revision, and builder digest
with a domain-separated Ed25519 signature. Use `export_sdk` and
`verify_sdk_directory`; it accepts only the exact native file/directory
inventory under fixed traversal budgets before authenticating every required
byte. Never trust provenance instead of re-verifying bytes.

See `LICENSES.md` before any redistribution or production use.
