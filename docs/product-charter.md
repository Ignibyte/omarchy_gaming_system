# Omarchy BBS product charter

## Product promise

Omarchy BBS is an API-first social gaming system with a keyboard-first QML
connector. It should feel alive for one or two people and grow naturally into a
small community.

## First playable outcome

A user can create an account and persona, connect with another persona, send a
game challenge through an inbox, complete an asynchronous match, and see the
result recorded.

## Private-alpha scope

- Accounts and revocable sessions
- Persona creation and profiles
- Connection requests and blocking
- Inbox threads and typed game messages
- Game catalog, challenges, turns, and match history
- One original asynchronous game with a bot opponent
- Cursor-based synchronization and live notifications
- Keyboard-first QML connector for Omarchy
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
- Games are deterministic and server-authoritative
- WebSockets are advisory; durable cursor sync recovers missed events
- Games begin as compiled Rust crates; a sandboxed SDK is a later decision

## Definition of private-alpha done

Two clean Omarchy installations can connect to one server, create personas,
connect, exchange messages, finish a challenged match, reconnect after going
offline, and observe the correct match result without developer intervention.

