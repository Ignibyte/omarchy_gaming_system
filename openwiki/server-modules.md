---
type: "Reference"
title: "Server modules and typed hook boundary"
openwiki_generated: true
sources:
  - id: openwiki-source-ba203ea2e600f294ab58ef02
    resource: repo://crates/server/src/bin/omarchygs-admin.rs
  - id: openwiki-source-a13fe4db1eee073d0a7e2c4d
    resource: repo://crates/server/src/main.rs
  - id: openwiki-source-2d8ea93c101c36a0e0974581
    resource: repo://crates/server/src/server_modules.rs
  - id: openwiki-source-e9c32af872bdfcc1f392d212
    resource: repo://docs/architecture/server-modules.md
  - id: openwiki-source-6d761b854f0836930f612db4
    resource: repo://docs/operators/server-modules.md
  - id: openwiki-source-dc62400b0039f0daf5073bd4
    resource: repo://migrations/0026_server_module_observation_evidence.sql
  - id: openwiki-source-e08dc6155c081d7928029e27
    resource: repo://scripts/test-operator-recovery.sh
  - id: openwiki-source-8128bd5b86e858053bc20c68
    resource: repo://scripts/test-server-module-spike.sh
  - id: openwiki-source-5f564ae64057cbe621fc587a
    resource: repo://scripts/test-server-modules.sh
generated: {by: "codex", at: "2026-08-28T00:35:08.763Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-28T01:06:32.318Z
---

# Server modules and typed hook boundary

## Current status

ADR-0004 is now implemented for one disabled-by-default, reviewed first-party
module. When the exact opt-in configuration is present, production registers
the compiled-in `ignibyte.sentinel` release, proves a fresh packaged host under
OS containment, creates a server-specific admission, and starts a durable
dispatcher. Ticket 039 remains the independent architecture proof; Ticket 040
owns the production observation slice.

There is still no module discovery marketplace, operator-supplied executable
path, arbitrary install/import, public administration route, admission hook,
network egress, or gameplay authority. Ticket 041 separately gates custom
server-module installation and provenance.

The extension families stay deliberately separate:

| Family | Supplied artifact | Authority |
|---|---|---|
| Game Cartridge | Signed inert schemas and assets | Presentation only through trusted OmarchyGS QML |
| Compiled game | Reviewed Rust linked into the platform | Exact-version platform game rules |
| Registered provider | Independent authenticated service | Sole remote rules and private-state authority for its pinned sessions |
| Server module | Process-isolated no-WASI component | Observe admitted hooks and propose typed effects |

A module cannot supply client QML, access a provider grant, implement a second
gameplay authority, or receive a database handle.

## Trust and runtime boundary

Release, provenance, server admission, and measured runtime containment are
independent claims. A publisher signs the exact component and contract. A
marketplace or server operator separately records provenance. Core admits exact
hooks, capabilities, budgets, configuration/state revisions, lifecycle, and
server identity. Operator-custom provenance must name that same admitted
server. The host then measures the actual containment used to execute it.

```text
core transaction + durable outbox event
                 ↓ bounded partitioned dispatch
exact release + admission + typed event
                 ↓ bounded local RPC
dedicated OS-contained module host
                 ↓ exact no-WASI WIT component
              typed intent
                 ↓
core reauthorization + domain transaction + immutable receipt
```

The component sees only bounded typed hook data, pairwise/public identifiers,
immutable configuration, and a state snapshot. It returns a no-op or typed
intent. Core rechecks the exact release, admission, capability, lifecycle,
target, policy, idempotency identity, and expected revision before any protected
effect commits.

The first production observation is emitted by report creation. While Sentinel
is active and below its 1,024-row outstanding ceiling, the authoritative report
transaction appends one privacy-minimized `persona_reported` event. Exact report
replay returns before emission, so it cannot enqueue a duplicate. The event
contains the report UUID, bounded category, pairwise subject, and exact
configuration/state snapshots; it excludes report detail, reporter/account
identity, credentials, arbitrary paths or URLs, and provider authority.

Observation is optional rather than admission. If the module is inactive or
the queue is saturated, the report still commits and the same transaction
increments a saturating aggregate gap count with `module_inactive` or
`queue_saturated` plus its timestamp. After enqueue, timeout, trap, malformed
output, crash, or retry cannot roll back the completed report. A later
admission hook would have to run against an immutable snapshot outside a
database transaction, then re-lock and revalidate authoritative state before
commit.

## State, lifecycle, and operations

Configuration and state remain in core-owned bounded namespaces with
compare-and-set revisions. Upgrade uses an isolated candidate namespace,
explicit forward migrations, quota/schema validation, and retained rollback
snapshots. Backups include manifests, admissions, audit, outbox, namespaced
state, aggregate gap evidence, and immutable delivery receipts. New receipts
retain the bounded attempt-normalized canonical request, exact response, their
digests, and target report even after delivered outbox pruning. Upgrade-era
rows that predate migration 0026 remain explicitly identifiable as incomplete
rather than receiving fabricated evidence.

The database-local `omarchygs-admin` process implements bounded inventory plus
expected-revision disable, suspend, recover, terminal retire, and restore
commands. Command files must be regular, owner-held, single-link, exact-mode
0600 files; the shared reader uses no-follow open and verifies descriptor
identity and metadata stability around the bounded read.

PostgreSQL cannot infer that a raw archive has been restored. The operator must
therefore run the audited `module-restore` command before any restored server
startup. It disables every module, clears stale leases, blocks activation, and
requires explicit review plus recovery before fresh readiness. A configured
server still starts while the persisted module is inactive and records
aggregate `module_inactive` gaps without starting its dispatcher. Install,
upgrade, custom provenance, and removal remain future operations.

## Proof and next implementation slice

`scripts/test-server-module-spike.sh` retains the independent exact-WIT hostile
proof. `scripts/test-server-modules.sh` is the production entrypoint: it runs
runtime tests and rustdoc, executes the packaged host under real systemd-user,
Bubblewrap, prlimit, Wasmtime memory/fuel, and outer-time containment, and
asserts the fixed sibling loader, absent custom artifact inputs, absent public
module routes, and absent network/database client dependencies.

The migrated PostgreSQL corpus proves atomic private emission, ordering,
receipt replay and retention, fail-open gap accounting, bounded failure and
circuit behavior, readiness configuration/state races, lifecycle/state CAS,
restore, and legacy receipt semantics. Gate 21 proves pre-start restore
reconciliation with module configuration still present; gate 24 runs the
production conformance script. Ticket 041 keeps administrator custom
installation and provenance separate, and additional hook classes remain
separately reviewed work.

## Change map

| Intent | Read first | Narrow evidence |
|---|---|---|
| Change release, WIT, signatures, admission, framing, host, or limits | ADR-0004, `docs/architecture/server-modules.md`, and `crates/server-module-runtime` | `scripts/test-server-modules.sh` plus deterministic and contained-host conformance |
| Change report observation, dispatch, receipts, gaps, intents, state, lifecycle, or restore | `crates/server/src/server_modules.rs`, `reports.rs`, migrations `0025`–`0026`, and `docs/operators/server-modules.md` | Focused ignored server-module and operator-CLI tests, gate 21 recovery drill, then gate 24 |
| Change the independent architecture proof | `crates/server-module-spike` | `scripts/test-server-module-spike.sh` and gate 23 |
| Add custom installation or another hook/intent | Ticket 041 and Constitution extension boundaries | New ticketed threat/authority review, CodeGraph inspection, database/process evidence, and canonical local diff gate |
