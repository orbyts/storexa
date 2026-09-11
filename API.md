# Public API contract

Version 0.2.0 preserves the PostgreSQL 0.1.0 surface and adds explicit SQLite
types. Storexa applies semantic-versioning compatibility throughout the 0.2 series.

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

## SQLite capability

- `SqliteDatabaseConfig::from_path` accepts literal file paths, never SQLite URLs.
  `in_memory` creates a private database. File creation is explicit; directories
  are application-owned. `validate` checks options without I/O.
- Builders configure pool limits/timeouts, file creation, foreign-key enforcement,
  busy timeout, optional journal mode, synchronous policy, and diagnostic name.
- `SqliteDatabase` provides `connect`, `name`, `pool`, `acquire`, `stats`, `health`,
  `health_check`, `begin`, `run_migrations`, and `close`. Clone/close semantics
  match PostgreSQL. `SqliteHealthReport` reports `sqlite_version` and latency.
- `SqliteTransaction` dereferences to `SqliteConnection` and provides `commit`
  and `rollback`. `begin` uses SQLite's deferred transaction semantics; it does
  not claim that a writer lock has already been acquired.
- `SqliteResult<T>` uses `SqliteError`, including SQLite-specific query conversion
  through `From<sqlx::Error>`. The PostgreSQL conversion remains unchanged.
- `SqlitePool`, `Sqlite`, `SqliteJournalMode`, `SqliteSynchronous` are convenience
  SQLx re-exports. `DatabaseStats` and `MigrationReport` are shared value reports.

In-memory pools require exactly one retained connection with no idle/lifetime
recycling. They are ephemeral even while a pool handle exists if its connection
is lost or explicitly closed through SQLx. File pools may grow, but SQLite still
has only one writer at a time. Raw SQLx access lets applications change connection
state; Storexa settings describe initialization, not enforced isolation from
application SQL. Applications own migration serialization across independent
processes/pools and any backup, recovery, or synchronization protocol.

SQLite paths are redacted in Storexa debug/tracing/error display; SQLx statement
logging is disabled for Storexa-created SQLite connections. Error sources remain
available for deliberate inspection, and direct SQLx query errors are not redacted
unless converted into `SqliteError`. Diagnostic names must contain no secrets.

## Explicitly deferred

- Generated CRUD and application entities
- A global database registry
- Provider project or branch creation
- A public cross-provider transfer trait
- Supabase transaction-pooler support
- Engines other than PostgreSQL and SQLite
- SQLite network-filesystem coordination, synchronization, backup, and SQL translation
