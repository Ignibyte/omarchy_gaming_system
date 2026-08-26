# Owner-operated OmarchyGS servers

Status: product and trust direction accepted by
[`ADR-0003`](../architecture/adr-0003-owner-operated-server-and-extension-boundary.md);
the marketplace, custom-content, and module administration surfaces are not
implemented yet.

## Operating model

An OmarchyGS server is an independently operated community. Its administrator
deploys the standard Rust/PostgreSQL service, controls its hostname and TLS,
manages availability and backups, selects catalog releases, moderates its
community, and invites players. Accounts, personas, sessions, connections,
catalog policy, achievements, audit, and history belong to that deployment.
The same handle on another server is not the same identity.

The OmarchyGS project supplies software, signed release/provenance mechanisms,
and security guidance. It does not remotely operate, continuously monitor,
back up, moderate, or certify an independent administrator's deployment.

## Game provenance

Planned server catalogs distinguish at least these sources:

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

The implemented private-alpha invitation, report, suspension,
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
