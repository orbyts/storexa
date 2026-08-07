# PostgreSQL compatibility

Storexa 0.0.5 has one data-plane implementation: SQLx 0.9's native PostgreSQL
driver. Provider metadata is descriptive and never selects different CRUD,
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

## Primary references

- [SQLx PostgreSQL driver](https://docs.rs/sqlx/0.9.0/sqlx/postgres/)
- [Neon connection pooling](https://neon.com/docs/connect/connection-pooling)
- [Supabase PostgreSQL connections](https://supabase.com/docs/guides/database/connecting-to-postgres)
