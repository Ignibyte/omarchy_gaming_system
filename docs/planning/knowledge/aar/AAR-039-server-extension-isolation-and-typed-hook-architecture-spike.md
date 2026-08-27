---
aar: AAR-039-server-extension-isolation-and-typed-hook-architecture-spike
ticket: TICKET-039
pipeline: server-extension-isolation-and-typed-hook-architecture-spike
status: submitted
opened: 2026-08-27
submitted: 2026-08-27
effectiveness: effective
---

# AAR-039-server-extension-isolation-and-typed-hook-architecture-spike

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-owner-operated-server-extension-boundary-001` | ADR-0003, roadmap, and Ticket 027 recall | Yes; requires the isolation spike and keeps modules separate from providers and cartridges. |
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Ticket 014 architecture/proof recall | Yes; fixes trusted frontend and single-gameplay-authority boundaries that general hooks cannot bypass. |
| `AD-omarchy-gaming-system-remote-provider-security-foundation-001` | Ticket 018 provider recall | Yes; demonstrates bounded authenticated process separation, replay, quota, lifecycle, and audit patterns without making provider RPC the module contract. |
| `PR-omarchy-gaming-system-gate-every-nested-workspace-proof-001` | Ticket 014 delivery lesson | Yes; any isolated module proof must be included in the canonical gate, not merely hashed. |
| `PR-omarchy-gaming-system-gate-independent-executable-source-trees-001` | Ticket 019 delivery lesson | Yes; the proof module executable and fixtures must contribute to delivery state. |
| `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001` | Ticket 014/018 security recall | Yes by analogy; every module exchange must bind exact module identity, release, capability grant, hook, operation, and expiry/replay context. |
| `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001` | Ticket 014 hostile provider response lesson | Yes; process/runtime output must be bounded before buffering or decoding. |
| `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001` | Ticket 018 concurrency lesson | Yes; future durable hook receipts need a stable namespace root before first-delivery deduplication. |
| `AD-omarchy-gaming-system-operator-custom-cartridge-trust-boundary-001` | Ticket 038 nearest completed pipeline | Yes; provenance, operator trust, executable authority, and client containment remain independent. |
| Ticket 038 AAR terminal-state mismatch | Phase 1 pipeline validation | Yes; exposed that lifecycle frontmatter is an exact machine contract and must be validated after archive. |
| Owner decision: local-only quality enforcement | User-directed Phase 2 scope amendment | Yes; GitHub Actions was disabled remotely and its repository definition is being removed while retaining the canonical local gate and receipt. |

## What happened

Ticket 039 compared the credible extension boundaries and selected one exact
no-WASI WebAssembly Component Model release per dedicated OS-contained native
host process. The isolated nested workspace proves signed release, provenance,
and server admission contracts; exact WIT compatibility; typed hook-to-intent
flow; core reauthorization; bounded framing, fuel, memory, files, queues, and
state; lifecycle and recovery; and failure containment without adding a
production loader, route, migration, or configuration surface.

The inspection phase materially hardened the proof. It bound operator-custom
provenance to the admitted server, enforced artifact limits during the file
read, pruned empty dispatcher partitions, and resolved containment helpers to
reviewed absolute paths. The full local gate then exposed an unrelated but
important integration hazard: a new nested workspace's generated `target/`
tree entered the native-package source fixtures and exhausted temporary
storage. The fixture now streams only declared source trees and excludes
generated targets before extraction. GitHub Actions was disabled and its
workflow removed; a local checker and hostile fixture now reject hosted CI/CD
definitions while preserving the worktree-bound local gate as delivery proof.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-component-record-export-shim-001` | Hand-written component fixtures lifted named WIT records directly but omitted the type-export shim required for a valid exported Component Model function; contract-only tests passed while all runtime fixtures failed compilation. | First complete nested-workspace runtime test run. |
| `BF-omarchy-gaming-system-transient-scope-limit-nofile-001` | The local systemd user scope accepted memory/CPU/task properties but rejected `LimitNOFILE`, so the first contained process never reached readiness. | First real supervisor valid-flow smoke. |
| `BF-omarchy-gaming-system-supervisor-measured-launcher-rss-001` | The supervisor initially sampled the `systemd-run` launcher PID and reported that tiny value as host RSS. | Review of individual process-scenario measurements before Phase 3 exit. |
| `BF-omarchy-gaming-system-operator-provenance-server-binding-001` | The proof verified a separately signed operator-custom provenance statement but did not compare its server identity with the core admission server. | Phase 3.5 security binding trace. |
| `BF-omarchy-gaming-system-artifact-read-before-bound-001` | The supervisor and host allocated component files with `fs::read` before the declared artifact ceiling was enforced during the read. | Phase 3.5 resource-bound trace. |
| `BF-omarchy-gaming-system-dispatch-retained-empty-partitions-001` | Queue length was bounded, but popping the last event retained each empty partition map entry indefinitely. | Phase 3.5 state-machine review. |
| `BF-omarchy-gaming-system-module-supervisor-untrusted-search-path-001` | The unsandboxed supervisor selected `systemctl` and `systemd-run` through inherited `PATH` even though other containment helpers used absolute paths. | Phase 3.5 process-boundary review. |
| `BF-omarchy-gaming-system-source-fixture-copied-nested-build-products-001` | The native-client hostile source fixture copied a newly introduced nested workspace's complete `target/` tree and exhausted temporary-directory quota. | First complete Ticket 039 diff gate. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-generate-component-shims-from-exact-wit-001` | Build proof/author components deterministically from exact WIT metadata and validate the completed component; do not hand-maintain named-type export shims. | WIT type identity and Canonical ABI lowering are part of the contract and easy to misrepresent with plausible raw WAT. |
| `PR-omarchy-gaming-system-apply-host-limits-at-supported-layers-001` | Prove every resource ceiling on the actual target host and apply unsupported service-manager limits through an independently inherited OS primitive. | A documented sandbox property is not evidence that a particular service-manager scope accepts or enforces it. |
| `PR-omarchy-gaming-system-measure-inside-intended-trust-unit-001` | Collect resource measurements inside the intended process/trust unit or resolve its exact cgroup identity; never infer it from a wrapper PID. | Wrapper processes can produce plausible but materially false resource evidence. |
| `PR-omarchy-gaming-system-pin-signed-document-authorities-out-of-band-001` | Resolve signed-document keys from host/core-provisioned trust state and reject self-supplied public-key substitutions before signature verification. | A mathematically valid signature under an attacker-selected key is not an authenticated publisher, provenance authority, or core admission. |
| `PR-omarchy-gaming-system-bind-operator-provenance-to-admitted-server-001` | Bind operator-custom provenance to the exact server identity in the core admission, even when both documents have valid independent signatures. | Cross-document digests do not prove that custom-code trust granted by one owner-operated server applies to another. |
| `PR-omarchy-gaming-system-enforce-artifact-bounds-during-file-read-001` | Open executable artifacts once and enforce `MAX+1` while reading; do not rely on a metadata pre-check or a later parser limit. | Resource ceilings must apply before allocation and remain correct when a backing file changes. |
| `PR-omarchy-gaming-system-prune-empty-bounded-state-001` | When a bounded partitioned queue empties a partition, remove its container and test the retained-state cardinality. | Bounding live items does not bound memory when empty indexing state accumulates. |
| `PR-omarchy-gaming-system-use-absolute-containment-helper-paths-001` | Select supervisor/control-plane containment helpers by reviewed absolute path before clearing the guest environment. | Guest environment isolation occurs too late to protect unsandboxed executable lookup. |
| `PR-omarchy-gaming-system-exclude-generated-trees-from-source-fixtures-001` | Source-fixture copies must exclude generated build trees before extraction and prove the archive inventory does not contain them. | Repository growth can turn a previously small recursive source copy into an unbounded quota and latency hazard. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-process-isolated-wasm-server-modules-001` | Use one exact no-WASI Component Model release per OS-contained module-host process, with typed intents reauthorized by core; keep production loading disabled until its own ticket. | `docs/architecture/adr-0004-process-isolated-wasm-server-modules.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalled cartridge/provider authority rules prevented the general
module seam from becoming executable frontend content or a second gameplay
backend. The nested-workspace and streaming-bound lessons directly shaped the
proof and its gate integration. Inspection still found four proof-boundary
defects, showing that signatures, resource declarations, queue bounds, and a
sandbox command line each needed independent source-to-sink verification. The
new rules make those checks explicit before the production module base begins.
