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
The `conformance` feature permits only the exact callback socket override and
post-commit response delay required by the portable local fault corpus; do not
enable it for a production provider.
