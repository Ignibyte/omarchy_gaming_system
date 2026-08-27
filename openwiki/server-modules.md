---
type: "Reference"
title: "Server modules and typed hook boundary"
openwiki_generated: true
sources:
  - id: openwiki-source-b8ce6b5ac0e4d708b3fff1af
    resource: repo://crates/server-module-spike/tests/contracts.rs
  - id: openwiki-source-1b0c20715f9f2a7a8217634f
    resource: repo://crates/server-module-spike/tests/runtime.rs
  - id: openwiki-source-f629e6aac25104e4390f424c
    resource: repo://crates/server-module-spike/tests/state_lifecycle.rs
  - id: openwiki-source-0fa8a0670e40aca3d14c3478
    resource: repo://docs/architecture/adr-0004-process-isolated-wasm-server-modules.md
  - id: openwiki-source-e9c32af872bdfcc1f392d212
    resource: repo://docs/architecture/server-modules.md
  - id: openwiki-source-f866d4e4132782d86cee8049
    resource: repo://docs/planning/tickets/open/TICKET-040-production-server-module-base-and-observation-hooks.md
  - id: openwiki-source-7d17fced59b7740c185d58fc
    resource: repo://docs/planning/tickets/open/TICKET-041-administrator-custom-server-module-installation-and-provenance.md
  - id: openwiki-source-8128bd5b86e858053bc20c68
    resource: repo://scripts/test-server-module-spike.sh
generated: {by: "codex", at: "2026-08-27T21:56:27.195Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-27T21:56:27.195Z
---

# Server modules and typed hook boundary

## Current status

ADR-0004 selects the architecture for future general server extensions: one
exact WebAssembly Component Model release runs in one dedicated, OS-contained
host process with no WASI or other guest imports. Ticket 039 proves that
boundary in an isolated nested workspace. Production module discovery,
installation, persistence, routes, configuration, administration, and startup
are not implemented or authorized.

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

Observation hooks are the first production class. They consume a future
durable post-commit outbox, so timeout, trap, malformed output, crash, or retry
cannot roll back the completed platform action. A later admission hook would
have to run against an immutable snapshot outside a database transaction, then
re-lock and revalidate authoritative state before commit.

## State, lifecycle, and operations

Configuration and state remain in core-owned bounded namespaces with
compare-and-set revisions. Upgrade uses an isolated candidate namespace,
explicit forward migrations, quota/schema validation, and retained rollback
snapshots. Backups include manifests, admissions, audit, outbox/receipts, and
namespaced state; restore starts modules disabled until every artifact and
pending receipt is reverified.

Install, enable, disable, upgrade, rollback, suspend, and remove remain future
database-local administrator operations. Marketplace-vetted and operator-custom
modules must use the same WIT, conformance, capability, and containment rules;
their provenance and player disclosures differ, not their runtime power.

## Proof and next implementation slice

`scripts/test-server-module-spike.sh` regenerates the exact-WIT component
fixtures twice, runs 21 contract/runtime/state tests, and runs 13 contained
process scenarios. The hostile matrix includes forbidden imports, wrong
interfaces, excessive memory, infinite work, traps, tamper, forged context,
unauthorized intent, host exit, outer timeout, and clean restart. The proof also
checks deterministic artifacts, local-only quality automation, containment
signals, and the absence of a production loader.

The next production slice is Ticket 040: the versioned module base, durable
observation outbox/dispatcher, one safe typed observation hook and intent,
state/lifecycle base, and conformance tooling. Ticket 041 keeps administrator
custom installation and provenance separate. Additional hook classes remain
separately reviewed work.

## Change map

| Intent | Read first | Narrow evidence |
|---|---|---|
| Change artifact, WIT, signatures, provenance, or admission | ADR-0004 and `docs/architecture/server-modules.md` | Contract tests plus deterministic fixture comparison |
| Change host, supervisor, limits, or local framing | `crates/server-module-spike/src/bin/` | Runtime tests and all process scenarios |
| Change intents, ordering, state, or lifecycle | `crates/server-module-spike/src/lib.rs` | Runtime and state/lifecycle suites |
| Add production loading or hooks | Tickets 040–041 and Constitution extension boundaries | New ticketed threat/authority review, CodeGraph inspection, database/process evidence, and canonical local diff gate |
