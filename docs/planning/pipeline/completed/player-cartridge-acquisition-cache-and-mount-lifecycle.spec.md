---
title: Player cartridge acquisition, cache, and mount lifecycle
pipeline_id: f7e13a3c-e3e9-4d8a-b4b0-a48cf0ef02d4
status: Phase 5 — Complete PASS
ticket: TICKET-033
ticket_doc: docs/planning/tickets/closed/TICKET-033-player-cartridge-acquisition-cache-and-mount-lifecycle.md
aar: docs/planning/knowledge/aar/AAR-033-player-cartridge-acquisition-cache-and-mount-lifecycle.md
created: 2026-08-26
---

# Player cartridge acquisition, cache, and mount lifecycle — specification

## Outcome

An authenticated player can use the flagship client to install, update, and
remove the selected owner-operated server's exact admitted signed cartridge.
Trusted Rust verifies and caches inert bytes; QML receives bounded status and
provenance only and never becomes a credential-bearing filesystem installer or
an executable-content loader.

## Locked decisions

- The selected OmarchyGS server is the only distribution destination. The
  client does not follow marketplace, publisher, provider, or cartridge URLs.
- The server returns one exact bounded acquisition bundle only after device
  session authentication and a fresh effective-admission check. It never
  silently substitutes a newer, older, or fallback digest.
- Publisher integrity, marketplace review/lifecycle, and server admission stay
  separately verifiable. Ticket 033 persists the current exact signed
  marketplace snapshot because Ticket 032 retained its digest and derived
  facts but not the signed review document needed by a player verifier.
- Marketplace review is anchored outside the selected server. The companion
  loads one exact client-controlled marketplace public key before accepting an
  acquisition, requires the complete envelope key to equal it, and never
  learns that trust root from discovery, catalog metadata, QML, or the
  acquisition request. Missing trust leaves the social client usable but
  disables marketplace-vetted mounts.
- A small Rust companion owns download verification, cache writes, mount
  records, and removal. The launcher gives it a random loopback endpoint and
  per-process credential; repository-owned QML remains the trusted player UI
  but not the filesystem authority.
- The companion may receive the selected device bearer only for the duration
  of an explicit acquisition request. It does not persist the bearer, expose it
  in output/logs, forward it outside the selected origin, or give it to a
  render plan or cartridge.
- Immutable content is shared by digest; admission and provenance bindings are
  scoped by stable server UUID. Cross-profile byte reuse never means
  cross-profile trust reuse.
- Install and update are explicit player actions. A failed update leaves the
  prior good mount intact; a server policy denial prevents a new mount without
  deleting authoritative state or silently selecting another release.
- Removal first unmounts one server-profile binding. Exact cached content may
  be reclaimed only when no mount references it, while monotonic denial
  evidence and all server/account/game state remain intact.
- The package remains inert: no cartridge raw QML, JavaScript, native library,
  shell command, Web content, arbitrary path, executable bit, or direct network
  authority is introduced.
- Live session/render-plan binding is a later slice. This ticket mounts a
  verified exact presentation release and makes its lifecycle visible, but it
  does not claim a dynamically supplied game is playable before the server
  session contract pins and supplies a compatible view model.

## EARS acceptance criteria

| ID | EARS requirement | Required evidence |
|---|---|---|
| REQ-001 | When cartridge distribution is configured for the normal server, the process shall require one pre-provisioned secure store and the exact current marketplace public key, shall advertise the acquisition capability truthfully, and shall fail closed without exposing a partial distribution surface when configuration is absent or invalid. | Server configuration, discovery, and startup tests |
| REQ-002 | When a marketplace snapshot is published, the server shall durably retain the bounded exact signed snapshot and public verification key that produced the current reviewed inventory, without storing private signing material or weakening snapshot monotonicity. | Migration and PostgreSQL synchronization/replay tests |
| REQ-003 | When an authenticated player requests an exact cartridge acquisition, the server shall reauthorize that the requested game and digest are the currently effective local admission and shall return only a bounded, no-store bundle assembled from immutable server-staged bytes and the separate publisher, marketplace, and server-admission claims. | Axum authentication, lifecycle, exact-response, and size tests |
| REQ-004 | When the selected release changes, is omitted, becomes incompatible, or receives a denied lifecycle policy, acquisition of the formerly advertised digest shall fail closed and shall never substitute another release; an in-flight client shall recheck the exact current admission before mounting. | PostgreSQL transition/race and client TOCTOU tests |
| REQ-005 | When the flagship client starts, its launcher shall start one same-user Rust cartridge companion on a random loopback endpoint protected by an unguessable per-process credential, shall keep cache mutation outside QML, and shall stop the companion when the shell exits. | Companion API, launcher cleanup, hostile local-request, and packaged-client smoke tests |
| REQ-006 | When the QML shell asks to acquire a cartridge, the companion shall accept only the selected canonical server origin, stable server UUID, current device bearer, exact game key, digest, and admission revision; remote servers shall require HTTPS and neither the server nor cartridge may redirect or choose a different destination. | Request validation, TLS/redirect, server-identity, and authority-isolation tests |
| REQ-007 | When an acquisition bundle is received, the companion shall require its complete marketplace key to equal an independently provisioned client trust anchor—even when authority and key labels match—then shall verify the signed marketplace snapshot/review entry, publisher release signature, conformance reconstruction, lifecycle policy, exact digest and identity tuple, supported SDK, host compatibility, and selected server's exact admission before publishing a local mount. | Contract hostile corpus, substituted-key rejection, and end-to-end acquisition tests |
| REQ-008 | When verified bytes are installed, the companion shall atomically retain them in a private descriptor-relative content-addressed read-only cache with no executable permission, and identical digests may be reused without duplicating bytes. | Linux filesystem, symlink/race, mode, atomicity, and deduplication tests |
| REQ-009 | When a cartridge is mounted, the companion shall record a bounded server-UUID-scoped binding to the exact digest, client-trusted marketplace-key fingerprint, and admission revision, so restarts and independent server profiles never reuse admission or marketplace provenance authority even when they share immutable content bytes. | Multi-profile, trust-key substitution, and restart tests |
| REQ-010 | When a player refreshes the cartridge library, installs an available release, explicitly updates to a newly admitted release, or removes a local mount, the keyboard-first QML shell shall show bounded loading, ready, update, deprecated, unavailable, and error states and shall preserve the prior good mount on any failed acquisition. | QML fixture and live companion interaction tests |
| REQ-011 | When a player removes a mounted cartridge, the client shall remove only that profile binding and may reclaim exact immutable bytes only when no profile references them; it shall not delete account, persona, social, session, save, achievement, server catalog, or provider state. | Removal/reference-count and domain-absence tests |
| REQ-012 | When the native client package is built, it shall contain the exact reviewed QML payload and native companion/launcher with reproducible provenance, least-privilege modes, no embedded credentials, and a clean extracted-package acquisition smoke path. | Source contract, two-build comparison, payload inspection, and extracted-package smoke |
| REQ-013 | When Ticket 033 is delivered, the canonical diff gate shall pass the acquisition/database/QML/package path together with every existing cartridge, provider, recovery, and admission gate. | `bin/gate.sh --diff` receipt |

## Failure behavior

- Missing or invalid server distribution configuration omits the capability
  and route authority; startup never invents a writable store or trust key.
- Invalid sessions, exact identities, admission revisions, lifecycle states,
  signed records, digests, schemas, compatibility, sizes, paths, redirects, or
  server UUIDs fail without publishing a new mount.
- An interrupted download or cache write leaves no partial authoritative file.
- A companion authentication failure returns no cache, profile, server, or
  bearer information.
- A missing, invalid, symlinked, or substituted client marketplace trust key
  prevents install, inventory, and removal from treating any mount as
  marketplace-vetted; the rest of the social client remains available.
- A failed explicit update preserves the previous verified profile mount.
- Removal never calls a server mutation and never removes authoritative
  account, persona, social, gameplay, save, result, or achievement state.

## Observable non-goals

- A mounted cartridge does not yet make a new server game session playable.
- The companion is not a privileged system service and does not defend against
  a fully compromised same-user desktop session.
- No automatic update, background marketplace contact, cross-server trust,
  direct provider traffic, or offline server-authority replacement is added.
- No public marketplace-key enrollment or rotation channel is claimed; the
  private-alpha key is provisioned through a client-controlled local file.
- No operator-custom cartridge can claim marketplace-vetted provenance.

## Linked artifacts

- Ticket: [TICKET-033](../../tickets/closed/TICKET-033-player-cartridge-acquisition-cache-and-mount-lifecycle.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | confirmed shippable boundary |
| 2 Design | Acquisition protocol, trust/ownership model, file manifest, regression plan | CodeGraph receipt and design PASS |
| 3 Implement | Server bundle, client companion/cache/mounts, QML/package integration | focused compile/tests and self-review |
| 3.5 Inspect | Correctness/security/authority/filesystem/QML findings ledger and fixes | CodeGraph plus independent security disposition |
| 4 Validate | Focused suites and canonical gate | worktree-bound green gate receipt |
| 5 Complete | AC audit, OpenWiki, docs, submitted AAR, archive | no silent drops and completion receipt |
| Delivery | Fresh gate, staged review, commit/push verification | matching local/remote commit and tree |
