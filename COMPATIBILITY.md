# Database compatibility

Storexa 0.2.0 uses SQLx 0.9's native PostgreSQL and SQLite drivers through
separate public types. PostgreSQL provider metadata never selects different CRUD,
transaction, or migration code.

## Compatibility contract

A database is compatible when it accepts a `postgres://` or `postgresql://`
connection URL supported by SQLx and provides the PostgreSQL behavior required
by the application's own SQL and migrations. Provider-specific HTTP APIs are
not part of this contract.

| Target | Status | Connection guidance |
| --- | --- | --- |
| PostgreSQL 18 | CI verified | Direct connection; also represents local and self-hosted PostgreSQL |
| Neon | Integration verified | Direct and Neon pooled URLs both connect; use a direct URL for migrations and native transfer tools |
| Generic hosted PostgreSQL | Expected compatible | Use a standard PostgreSQL URL and verify required extensions |
| Supabase direct | Expected compatible | Prefer direct access for migrations and native transfer tools; network IPv6 availability may matter |
| Supabase session pooler | Expected compatible | Suitable when direct access is unavailable from an IPv4-only runtime |
| Supabase transaction pooler | Not yet supported | It does not support prepared statements; Storexa has not yet exposed and tested the required SQLx tuning |

“Expected compatible” is deliberately weaker than “verified.” It means the
provider exposes PostgreSQL, but Storexa's integration suite has not yet run
against that target.

## Client and server pooling

Every `Database` contains a client-side SQLx pool. Hosted services may add a
server-side pooler such as PgBouncer or Supavisor. When both are used, keep the
SQLx pool limits proportionate to the application's actual concurrency.

Transaction poolers do not preserve session state between transactions. Avoid
depending on session-level `SET` behavior, temporary tables spanning
transactions, or other session affinity. Storexa does not rewrite application
SQL to hide those differences.

Migrations and native transfer tools should use a direct connection unless the
provider and tool explicitly document pooler support. Runtime queries may use a
verified pooled endpoint.

## Extensions and versions

Wire-protocol compatibility does not guarantee that every provider enables the
same extensions, roles, collation versions, or administrative privileges.
Applications own those requirements and must encode portable assumptions in
their migrations.

## SQLite compatibility

SQLite uses SQLx's bundled SQLite engine. In-memory and local file databases are
covered by the ordinary integration suite: migration application/checksums,
commit/rollback/drop rollback, file reopen, independent memory pools, connection
settings, lock contention, acquisition timeout, and closed-pool failures.

WAL is opt-in and intended for a local filesystem. SQLite documents that WAL does
not work over network filesystems. Storexa does not detect mounts or add locking,
leases, or durability protocols for SMB/NFS. Merely choosing rollback journaling
does not establish support for shared network storage.

File journal mode is preserved unless explicitly configured; SQLite may persist
WAL mode in the file. Changes can need exclusive access. Connection settings are
applied to every connection. More pooled connections do not create simultaneous
SQLite writers. Busy timeout controls lock waiting; acquire timeout controls
waiting for a pool connection. SQLite migrations should be coordinated by the
application before serving work.

PostgreSQL and SQLite need engine-specific schemas/migrations. Storexa does not
promise equivalent SQL types, isolation levels, locking, or migration concurrency.

## Primary references

- [SQLx PostgreSQL driver](https://docs.rs/sqlx/0.9.0/sqlx/postgres/)
- [Neon connection pooling](https://neon.com/docs/connect/connection-pooling)
- [Supabase PostgreSQL connections](https://supabase.com/docs/guides/database/connecting-to-postgres)
- [SQLx SQLite driver](https://docs.rs/sqlx/0.9.0/sqlx/sqlite/)
- [SQLite WAL restrictions](https://www.sqlite.org/wal.html)
- [SQLite network filesystem caveats](https://www.sqlite.org/useovernet.html)
