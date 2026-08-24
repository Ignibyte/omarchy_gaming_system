# System overview

Omarchy BBS begins as a modular Rust monolith backed by PostgreSQL. The QML
connector is the flagship client but uses the same versioned public API as any
future terminal, web, mobile, or embedded connector.

```text
QML connector
  ├─ HTTPS/JSON commands and queries
  └─ WebSocket notifications
             ↓
Rust application
  ├─ auth and accounts
  ├─ personas and social connections
  ├─ conversations and notifications
  └─ server-authoritative game runtime
             ↓
PostgreSQL
  ├─ durable domain state
  ├─ game events and snapshots
  └─ synchronization cursors
```

## Boundaries

- HTTP and WebSocket handlers translate transport data and call domain code.
- Domain modules own authorization and invariants.
- Game modules accept validated commands and return deterministic transitions;
  they do not query the database directly.
- Transactions append events, update snapshots/revisions, and create durable
  notifications atomically.
- WebSockets signal that data changed. A cursor API repairs missed events after
  reconnect and remains the synchronization source of truth.

## Current slice

The current executable connects to PostgreSQL, applies embedded migrations,
and exposes `/health`. The QML connector consumes that endpoint and displays a
connected, offline, or protocol-error state.
