# Public API contract

Version 0.1.0 defines Storexa's first supported public surface. Storexa applies
semantic-versioning compatibility to this contract throughout the 0.1 series.

## Configuration

- `DatabaseConfig` constructs validated PostgreSQL pool configuration from a
  URL, conventional environment variables, an explicitly named environment
  variable, or one selected dotenv entry.
- Pool builders configure maximum and minimum connections, acquisition timeout,
  idle timeout, and maximum lifetime.
- `ConfigSource` reports non-secret configuration provenance.
- `DatabaseMetadata` and `PostgresProvider` attach validated diagnostic identity
  without changing SQL behavior.

Applications own their TOML or other configuration format. Storexa consumes the
resolved values and does not read an implicit global config file.

## Database lifecycle

- `Database::connect` creates and verifies a cloneable SQLx PostgreSQL pool.
- `pool` exposes `&PgPool` for application-owned SQL.
- `acquire` returns a classified `ConnectionLease`.
- `stats`, `health`, and `health_check` expose lifecycle diagnostics.
- `close` gracefully closes the shared pool.

Cloning `Database` clones the pool handle, not the underlying connections.
Closing any clone closes the shared pool.

## Transactions and migrations

- `begin` returns a pool-owned `Transaction` that dereferences to
  `PgConnection` for SQLx queries.
- `Transaction::commit` and `rollback` classify finalization errors.
- `run_migrations` accepts an application-owned SQLx `Migrator` and returns a
  `MigrationReport`.

Applications continue to own SQL, schemas, migration files, domain models, and
repository types.

## Errors and diagnostics

- Storexa operations return `Result<T, StorexaError>`.
- Storexa-produced error display and debug output do not include connection
  URLs or delegated SQLx error details.
- Wrapped SQLx and migration errors remain available through
  `std::error::Error::source()` for deliberate inspection.
- Storexa emits diagnostics through `tracing`; it never initializes a global
  subscriber and never prints directly.

## SQLx interoperability

Storexa re-exports `PgPool`, `Postgres`, and `Migrator` as conveniences. These
are not abstractions over a homemade ORM. SQLx remains the query, type, and
migration engine.

## Explicitly deferred

- Generated CRUD and application entities
- A global database registry
- Provider project or branch creation
- A public cross-provider transfer trait
- Supabase transaction-pooler support
- SQLite and non-PostgreSQL backends
