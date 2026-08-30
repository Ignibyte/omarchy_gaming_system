# ADR-0004: Process-isolated Wasm server modules

- Status: accepted; production observation slice and operator-custom admission implemented
- Date: 2026-08-27
- Knowledge ID: `AD-omarchy-gaming-system-process-isolated-wasm-server-modules-001`

## Context

Owner-operated OmarchyGS communities will eventually need general server
extensions for moderation, integrations, community automation, and other
behavior that is neither a game's presentation nor its authoritative rules.
ADR-0003 deliberately separated this family from inert Game Cartridges,
registered game providers, and reviewed first-party compiled code.

Executable modules can affect a server's confidentiality, integrity,
availability, moderation, and correctness. A signature proves who signed
bytes; marketplace review describes one review event; operator trust records a
local choice; a capability grant authorizes named effects; and containment
limits compromise. None of those claims substitutes for another.

The architecture spike compared dynamic/in-process Rust, statically compiled
modules, native external-process RPC, in-process WebAssembly, OCI processes,
and a process/Wasm hybrid. It also exercised the selected trust units against
traps, infinite work, excessive memory, forbidden imports, wrong interfaces,
tampered bytes, forged context, unauthorized intents, process exit, timeout,
and restart.

## Decision

The portable module artifact is one WebAssembly Component Model component with
an exact WIT package/world major and WIT SHA-256. One exact module release runs
in one dedicated native `omarchygs-module-host` process. The host embeds a
pinned Wasmtime release, links no WASI or other guest import, creates a fresh
resource-limited Store/instance per invocation, and never deserializes a
publisher-supplied native/AOT cache.

The native host is independently contained by an OmarchyGS-owned OS policy:

- a dedicated service identity and cgroup;
- bounded memory, CPU, tasks, and file descriptors;
- no network namespace beyond loopback and no socket hostcall;
- no home, server configuration, database credential, device, or writable
  component path;
- `NoNewPrivileges`/capability removal and a read-only exact component; and
- separate startup and per-invocation deadlines controlled outside Wasmtime.

Core and module exchange a bounded, length-prefixed canonical JSON control
protocol. The Component Model boundary itself uses typed WIT records. A hook
contains only allowlisted pairwise/public domain data plus immutable bounded
configuration and module-state snapshots. A component may return only typed
intent proposals. Core re-verifies release, provenance, admission, event,
capability, lifecycle, target, and expected revision before a core domain
service transaction can commit an effect and immutable receipt.
Operator-custom provenance must name that same admitted server, component
artifacts must be bounded while they are read, and bounded dispatchers must
discard empty partition state.

Observation hooks execute from a durable transactional outbox after the
original platform operation commits. Their failure cannot undo that operation.
Any future admission hook must evaluate an immutable snapshot outside a
database transaction and re-lock/revalidate authoritative state before commit.
The first production slice will implement observation hooks only.

Public module mutation routes, additional hook/capability vocabulary, module
egress, and gameplay authority are not authorized by this ADR. Production
discovery may disclose only bounded behavior/provenance aggregates; private
inventory and exact local mutation stay outside the network API.

## Alternatives

### Dynamic in-process Rust libraries

Rejected. Rust has no stable ABI for this boundary, and publisher code would
share the server's address space, allocator, runtime, credentials, and crash
domain. Safe unload and independent rollback are not credible.

### Statically compiled modules

Retained for reviewed first-party platform features. They are simple to
operate but require a full OmarchyGS rebuild and have full core authority, so
they are not the independently installed module ecosystem.

### Native external-process RPC

Process separation and lifecycle are sound, but arbitrary native artifacts
retain every syscall allowed by the OS sandbox and require architecture-
specific packaging. This may be a specialized future deployment boundary, not
the baseline portable artifact.

### In-process Wasm

Rejected as the sole boundary. WIT, linear memory, no ambient I/O, fuel, and
Store limits are useful, but a runtime escape, abort, or host resource defect
would still share the core server process.

### OCI/container module

Deferred as an optional deployment wrapper. It adds image, daemon, policy,
patching, and operator burden without replacing the typed component contract
or core reauthorization requirement.

### One host process for many modules

Rejected for the baseline. One runtime compromise or abort would cross module
trust domains and make resource attribution, disablement, and rollback less
reliable.

## Consequences

- The portable author contract is language-neutral WIT, not a Rust ABI.
- OmarchyGS must ship and patch the native host/runtime even though components
  release independently.
- The first core dispatcher uses a durable bounded outbox, self-contained
  receipt ledger, partition ordering, retry/dead-letter policy, circuit breaker,
  and explicit fail-open observation classification. Later hook classes must
  make their availability policy equally explicit.
- Configuration and state require core-owned module namespaces, quotas,
  compare-and-set revisions, staged forward migrations, retained rollback
  snapshots, and disabled-on-restore activation.
- Marketplace-vetted and operator-custom modules use the same conformance and
  sandbox. Their provenance and player warnings differ; their runtime power
  does not silently differ.
- Game Cartridges remain inert trusted-renderer data. Game rules remain either
  compiled platform code or one registered provider authority. A general hook
  cannot become a second gameplay authority or client executable bridge.
- Wasmtime and the OS sandbox are maintained security boundaries with residual
  kernel/runtime risk. Prompt patching, exact release admission, telemetry,
  emergency suspension, recovery, and operator disclosures remain required.

## Evidence

- `crates/server-module-spike/` and
  `scripts/test-server-module-spike.sh` implement the isolated proof without
  linking it to the production workspace.
- `crates/server-module-runtime/`, migration `0025_server_modules.sql`, and
  `scripts/test-server-modules.sh` implement and gate the first-party
  production observation base. Migration
  `0027_operator_custom_server_modules.sql` and the database-local
  `custom-module-*` commands add immutable custom artifact custody,
  server-bound provenance, lifecycle, shared dispatch, and bounded public
  disclosure without a public administration surface.
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
  defines the WIT and Canonical ABI foundation.
- [Wasmtime security](https://docs.wasmtime.dev/security.html) describes its
  sandbox assumptions and security process.
- [Wasmtime resource limiting](https://docs.rs/wasmtime/latest/wasmtime/struct.Store.html)
  documents Store fuel and limiter integration.
- [Bubblewrap](https://github.com/containers/bubblewrap) constructs the proof's
  filesystem, user, process, and network namespaces; OmarchyGS owns the exact
  policy rather than treating Bubblewrap defaults as a sandbox profile.
- The [Rust Reference ABI chapter](https://doc.rust-lang.org/reference/type-layout.html#the-rust-representation)
  does not provide the stable publisher ABI required by dynamic Rust modules.
