# Provider sidecar deployment templates

These files are reviewed inputs for the co-located profile, not an installer.
Replace every `@...@` token in a private staging directory, verify the exact
release registration and binary provenance, and follow
[`docs/operators/provider-deployment.md`](../../docs/operators/provider-deployment.md).

The provider and OmarchyGS must use different operating-system users,
PostgreSQL roles/databases, configuration directories, writable paths, and
runtime secrets. The sidecar socket is a routing decision only: canonical DNS
authority, TLS, signed messages, grants, lifecycle, quotas, and audit remain
mandatory.
