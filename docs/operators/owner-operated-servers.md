# Owner-operated OmarchyGS servers

Status: stable identity/discovery, isolated flagship-client profiles, and the
marketplace-vetted server catalog administration boundary are implemented. The
direction is accepted by
[`ADR-0003`](../architecture/adr-0003-owner-operated-server-and-extension-boundary.md);
client cartridge acquisition, operator-custom content, and module
administration remain follow-up work.

## Operating model

An OmarchyGS server is an independently operated community. Its administrator
deploys the standard Rust/PostgreSQL service, controls its hostname and TLS,
manages availability and backups, selects catalog releases, moderates its
community, and invites players. Accounts, personas, sessions, connections,
catalog policy, achievements, audit, and history belong to that deployment.
The same handle on another server is not the same identity.

Migration `0018` gives the deployment one immutable random UUID in PostgreSQL.
`GET /.well-known/omarchygs` publishes that UUID, the bounded public
`OGS_SERVER_NAME`, protocol 1, and the deterministic set of implemented
capabilities. Backups and restored deployments retain the UUID. Changing a
hostname, TLS certificate, or public name does not rotate it; identity rotation
and database-fork handling remain future operator workflows.

The flagship QML client stores at most 16 public profiles in
`omarchygs-server-profiles.ini` under the platform configuration directory.
Each record contains only canonical origin, UUID, public name, protocol, and
capabilities. Selecting or replacing a server clears live session/persona and
dependent social/game authority before another-origin traffic. If a remembered
origin presents a different UUID, the client refuses account access and
requires explicit removal of the prior profile.

This UUID pin is a continuity check, not a cryptographic identity proof. Remote
origins still require valid HTTPS, and a player must verify a newly entered
origin through an appropriate trusted channel. Multiple profiles are choices
among independent communities; they do not federate accounts or data.

The OmarchyGS project supplies software, signed release/provenance mechanisms,
and security guidance. It does not remotely operate, continuously monitor,
back up, moderate, or certify an independent administrator's deployment.

## Game provenance

Server catalogs distinguish these provenance classes; the first is implemented
and the other two remain staged work:

- **marketplace-vetted:** the exact publisher release and review/provenance
  record came through the OmarchyGS marketplace, then the local operator chose
  to import and activate it;
- **first-party:** the release ships with or is operated directly by the
  OmarchyGS project under its documented lifecycle; and
- **operator-custom:** the local administrator signed/imported the release or
  installed supporting server code without marketplace review.

Marketplace publication never forces a server to list a game. Conversely, a
local operator's decision never turns custom content into marketplace-vetted
content. Player-facing catalog and launch surfaces must keep the source and
operator identity visible and must not collapse all three into a generic
"verified" badge.

## Marketplace synchronization and local admission

The database-local `omarchygs-admin marketplace-sync` command synchronizes one
operator-pinned marketplace. Its configuration is four independent trust
inputs: a canonical HTTPS origin, an exact marketplace Ed25519 public-key
document, an explicit DER TLS root, and a pre-provisioned descriptor-relative
cartridge store. The guarded client rejects proxies, redirects, referers,
transparent decompression, private or mixed DNS answers, non-success status,
unbounded bodies, and release paths outside the configured origin.

Each monotonic signed snapshot binds its marketplace authority, review facts,
publisher public key, exact immutable release identities, relative component
paths, and signed lifecycle policy. The command verifies production release
and SDK conformance before staging accepted bytes. PostgreSQL publishes the
reviewed inventory atomically only after all entries succeed; a failed or
downgraded snapshot cannot partially replace the prior catalog.

Synchronization is not admission. `omarchygs-admin cartridges` displays the
reviewed inventory and effective state. `omarchygs-admin catalog-apply` uses an
idempotency UUID plus exact expected and desired selections to activate,
deactivate, upgrade, or explicitly roll back one game. Every successful change
increments its admission revision and appends an immutable audit event in the
same transaction. A stale expectation or changed replay conflicts.

The authenticated `GET /v1/cartridges` response exposes only effective
marketplace-vetted metadata. Marketplace suspension, removal from the current
snapshot, local incompatibility, or loss of admission hides the selected
release immediately and never falls back to another digest. Selecting a
different release or rolling back always requires an explicit operator
command. If a newer authenticated marketplace policy permits the same exact
still-selected release again, it becomes effective without rewriting the
server's retained selection. Marketplace outage does not prevent use of the
last valid database snapshot or local catalog changes, but operators cannot
treat stale review state as a fresh sync.

The current command stages server-side immutable bytes as evidence and future
distribution input. It does not yet make the QML client download, cache, mount,
or launch a cartridge. Those player-side steps remain a separate trust-boundary
slice.

## Custom cartridges and code

An administrator will be able to enable a local trust domain and import custom
cartridges. Marketplace bypass does not mean parser, package, or client-safety
bypass: a custom cartridge remains signed inert data, is content-addressed,
passes the same byte/media/schema/capability checks, and renders only through
platform-owned QML components. It cannot ship raw QML, JavaScript, native
client code, credentials, or an arbitrary network client.

Custom game backend code uses the registered provider contract. General
server behavior uses the future server module system. Both are executable
server-side trust decisions made by the operator and may affect the
confidentiality, integrity, availability, moderation, and correctness of that
server. They must be independently inventoryable, auditable, disableable, and
recoverable; a general module hook must not masquerade as a game provider or
mutate protected state outside core authorization.

A future Provider SDK may let an administrator run a custom game backend as a
separate service on the same infrastructure. Co-location does not grant shared
database credentials or remove provider identity, authentication, quotas,
audit, or lifecycle controls; its local transport profile requires its own
security design.

## Administrator responsibilities

Before inviting players, an operator is responsible for at least:

- production TLS, secrets, database access, host/network hardening, upgrades,
  monitoring, capacity, backups, restore drills, and incident response;
- reviewing every custom cartridge, provider, module, configuration change,
  requested capability, and data destination they enable;
- maintaining publisher/provider/module keys and acting on suspension,
  revocation, vulnerability, and end-of-life notices;
- presenting accurate provenance and custom-content warnings to players;
- defining moderation, retention, privacy, account recovery, support, and
  acceptable-use policies appropriate to their deployment; and
- complying with the laws and contractual obligations that apply to their
  jurisdiction, users, and data.

The implemented private-alpha invitation, report, suspension, catalog,
immutable-audit, and platform restore workflows are documented in the
[private-alpha runbook](private-alpha.md) and
[operator safety and platform recovery](operator-safety-and-recovery.md).

This document is engineering and product guidance, not legal advice. Public
self-hosting distribution requires reviewed terms, privacy and telemetry
disclosures, warranty/support boundaries, a security contact policy, and
clear language for operator-custom content. Those terms allocate expectations;
they do not replace technical containment or honest provenance.

## Project safety boundary

Connecting to a custom server necessarily trusts that server with the account,
social, and game data created there. It does not grant the server authority
over the player's device. The official client retains HTTPS requirements,
origin-scoped sessions, bounded response validation, inert cartridge parsing,
trusted rendering, and no direct cartridge/provider credential path regardless
of the server's catalog source.

Federation, cross-server accounts, cross-server challenges, shared moderation,
and global recovery are separate future systems. An administrator must not
promise those properties based only on protocol compatibility.
