# Omarchy Gaming System product charter

## Product promise

Omarchy Gaming System—OmarchyGS for short—is an API-first social gaming system
with a keyboard-first QML connector. It should feel alive for one or two people
and grow naturally into a small community.

The primary experience is playing and maintaining relationships through
connections, private inboxes, challenges, and persistent game history. A
public message board may become a complementary community surface later, but
it is not the current product identity or private-alpha focus.

The long-term deployment unit is an owner-operated OmarchyGS community. An
individual or group runs the standard server, curates its game library, and
invites players into that server-local identity and social world. Independent
servers may implement the same public protocol without implicitly sharing
accounts, personas, policy, or history.

## First playable outcome

A user can create an account and persona, connect with another persona, send a
game challenge through an inbox, complete an asynchronous match, and see the
result recorded.

## Private-alpha scope

- Operator-invited accounts, opt-in TOTP two-factor authentication, and
  revocable sessions
- Persona creation and profiles
- Connection requests and blocking
- Inbox threads and typed game messages
- Game catalog, challenges, turns, and match history
- One original asynchronous game with solo bot and two-person challenge modes
- Cursor-based synchronization and live notifications
- Keyboard-first QML connector for Omarchy with one accessible semantic theme,
  deterministic focus/traversal, explicit plain-text presentation, and
  minimum-window containment across the complete player flow
- Native client-only Arch package with an Omarchy application entry, exact
  trusted-QML payload, and extracted-artifact launch proof
- Persona reporting plus a database-local sysop queue, reversible account
  suspension with session containment, immutable audit, and isolated platform
  backup/restore proof
- Expiring one-account invitation codes, local audited issue/revoke/inventory,
  masked QML registration, and an external-alpha operator/tester runbook

## Explicit non-goals for the first alpha

- Federation between servers
- Public message boards or activity feeds
- Shared currencies and item trading
- User-supplied native plugins
- Mobile or browser clients
- Real-time action games
- Ports without verified source and asset licensing

## Architectural commitments

- Rust modular monolith backed by PostgreSQL
- Versioned JSON API plus WebSocket notifications
- A native Omarchy player package consumes the system Qt QML runtime and ships
  no server, credential, provider executable, or publisher-supplied code
- Account and persona identities remain separate
- Every game session has one server-side authority. OmarchyGS owns compiled
  games; an operator-pinned registered provider may own one remote game's
  rules/state/revision while OmarchyGS retains identity, envelope, policy,
  projections, audit, and recovery.
- WebSockets are advisory; durable cursor sync recovers missed events
- Player reports are bounded platform data. Private-alpha operator mutations
  stay outside the network API in a PostgreSQL-local command, revoke current
  authority transactionally, and retain immutable audit.
- Signal Siege remains a reviewed compiled Rust crate. Door Legends v1 is the
  sole first-party remote authority pilot. Signed declarative cartridges and
  the versioned conformance SDK remain the portable frontend contract;
  external provider onboarding is still gated.
- A Game Cartridge is a signed inert frontend release rendered through
  platform-owned QML components. It contains no publisher QML, server rules,
  backend executable, credential, or independent network client.
- Each owner-operated server controls its admitted catalog. The planned vetted
  marketplace supplies reviewed exact releases; an operator may instead
  establish an explicitly marked local trust domain for custom cartridges and
  future server extensions without weakening official-client validation.
- Portable game backends use the brokered provider protocol and a future public
  Provider SDK. General server modules and hooks are a separate, versioned,
  capability-scoped extension family whose executable isolation must be
  approved before implementation.

## Definition of private-alpha done

Two clean Omarchy installations can connect to one server, create personas,
register with distinct operator-issued invitations, connect, exchange messages,
finish a challenged match, reconnect after going offline, and observe the
correct match result without developer intervention. Deterministic software
rehearsal does not replace recording that real external acceptance run.
