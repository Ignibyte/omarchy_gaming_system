# Relay Forge provider

Relay Forge is the second clean-room OmarchyGS provider proof. Its rules depend
only on the public `ProviderGame` seam. The public starter owns protocol
authentication, exact grants, PostgreSQL sessions and receipts, callback
delivery, TLS routing, and process shutdown.

This preview grants no registration or activation authority. The process
receives only pairwise game-scoped subjects and scoped grants—never account or
persona identity, device credentials, platform database access, arbitrary
egress, direct client connectivity, client executable privilege, or compiled
gameplay fallback.

The binary accepts one absolute, mode-0600, effective-user-owned, single-link
canonical JSON configuration path. Configuration contains the provider's own
database/TLS/signing material plus public platform verification keys. Run the
portable conformance kit rather than exposing the conformance-only socket and
post-commit-delay controls in a production build.
