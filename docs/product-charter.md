# Omarchy Gaming System product charter

## Product promise

Omarchy Gaming System—OmarchyGS for short—is an API-first social gaming system
with a keyboard-first QML connector. It should feel alive for one or two people
and grow naturally into a small community.

The primary experience is playing and maintaining relationships through
connections, private inboxes, challenges, and persistent game history. A
public message board may become a complementary community surface later, but
it is not the current product identity or private-alpha focus.

## First playable outcome

A user can create an account and persona, connect with another persona, send a
game challenge through an inbox, complete an asynchronous match, and see the
result recorded.

## Private-alpha scope

- Accounts, opt-in TOTP two-factor authentication, and revocable sessions
- Persona creation and profiles
- Connection requests and blocking
- Inbox threads and typed game messages
- Game catalog, challenges, turns, and match history
- One original asynchronous game with solo bot and two-person challenge modes
- Cursor-based synchronization and live notifications
- Keyboard-first QML connector for Omarchy with one accessible semantic theme,
  deterministic focus/traversal, explicit plain-text presentation, and
  minimum-window containment across the complete player flow
- Sysop audit, suspension, and reporting basics

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
- Account and persona identities remain separate
- Every game session has one server-side authority. OmarchyGS owns compiled
  games; an operator-pinned registered provider may own one remote game's
  rules/state/revision while OmarchyGS retains identity, envelope, policy,
  projections, audit, and recovery.
- WebSockets are advisory; durable cursor sync recovers missed events
- Signal Siege remains a reviewed compiled Rust crate. Door Legends v1 is the
  sole first-party remote authority pilot. Signed declarative cartridges and
  the versioned conformance SDK remain the portable frontend contract;
  external provider onboarding is still gated.

## Definition of private-alpha done

Two clean Omarchy installations can connect to one server, create personas,
connect, exchange messages, finish a challenged match, reconnect after going
offline, and observe the correct match result without developer intervention.
