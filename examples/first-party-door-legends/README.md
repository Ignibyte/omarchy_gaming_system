# Door Legends — first-party cartridge and provider fixture

This directory is source-only and intentionally has no path dependency on the
OmarchyGS repository. The Ticket 017 clean-room harness copies it into a fresh
Git repository, supplies an exported cartridge SDK and installed production
CLI through explicit environment variables, then builds a signed release
twice.

The cartridge is a small BBS-style lobby screen. Ticket 019 adds the standalone
Rust/Axum provider under `provider/`. It owns Door Legends rules, private state,
revision, operation receipts, and callback outbox in its own PostgreSQL
database. OmarchyGS retains only its platform session envelope and an
authenticated, bounded presentation cache; there is no compiled Door Legends
fallback in the platform server.

The authority gate exports `omarchy-game-provider` as a packaged public
protocol crate, initializes this directory as a new Git repository, clones it,
and builds the provider without access to a platform path dependency. It then
runs the real TLS broker, callback, restart, and backup/restore flow:

```bash
scripts/test-provider-authority-pilot.sh
```

Production deployment and lifecycle details are in
[`docs/operators/provider-authority-pilot.md`](../../docs/operators/provider-authority-pilot.md).
