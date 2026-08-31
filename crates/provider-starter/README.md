# OmarchyGS provider starter

This public preview crate owns provider-side exact HTTP/TLS verification,
PostgreSQL sessions and whole-operation receipts, expected revisions, consumed
grants, callback outbox delivery, and lifecycle. Games implement only the
small deterministic `ProviderGame` rules seam.

The starter receives pairwise game-scoped subjects and scoped grants. It has
no account or persona identity, reusable device credentials, platform database
access, arbitrary egress, client executable privilege, direct client
connectivity, compiled gameplay fallback, or registration, activation,
discovery, trust, admission, and publication authority.

One database is pinned to one provider/release/game/rules/cartridge identity.
Production co-location may use `CallbackConfig::sidecar` to map the exact
canonical HTTPS callback authority to one equal-port loopback TLS socket. It
retains the platform DNS identity, TLS root, signed path/body, and bounds; it
does not create a general private-network exception. The `conformance` feature
permits only the separately named callback socket override and post-commit
response delay required by the portable local fault corpus; do not enable it
for a production provider.

See the repository
[`provider-deployment.md`](../../docs/operators/provider-deployment.md) runbook
for remote and sidecar deployment, lifecycle, rotation, restore, and incident
procedures. Neither profile grants registration or admission authority.
