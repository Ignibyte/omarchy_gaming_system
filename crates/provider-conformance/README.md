# OmarchyGS provider conformance

This public preview crate provides an exact loopback-only TLS runner,
authenticated callback sink, finite fault inventory, secret-free receipt, and
deterministic developer-kit exporter. The corpus covers compatibility, normal
operations, whole-operation replay, changed intent, stale revision,
commit-timeout recovery, transport outage/recovery, signature/digest/context
mismatch, malformed and oversized input, callback retry/deduplication, and
authoritative reconciliation.

`ConformanceTarget` defaults to the Relay Forge sample sequence for backward
compatibility. Providers with another command vocabulary can supply a bounded
`ConformanceGameplayProfile`: one launch payload, one retry-safe timeout
command, a finite continuation, and an active or completed final status. This
changes only game-owned payloads; the fixed 15-case security, fault, callback,
and receipt contract remains intact. The final continuation command must emit
the provider callback being tested, whether that is a persistent `turn_ready`
fact or a terminal result.

The runner creates only ephemeral platform test grants and signatures. A
provider sees pairwise game-scoped subjects and scoped grants, never account or
persona identity, reusable device credentials, platform database access,
arbitrary egress, client executable privilege, or direct client connectivity.
The crate has no registration, activation, discovery, trust, admission, or
publication operation.

The CLI accepts one absolute, canonical, mode-0600 configuration file. Its
socket override must be loopback. Production network policy and co-located
sidecar transport are deliberately outside this package.
