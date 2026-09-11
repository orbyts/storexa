# Storexa

Storexa is a small, domain-agnostic PostgreSQL and SQLite persistence foundation for Rust
applications. It uses SQLx and deliberately does not generate application SQL
or model application entities.

Version `0.2.0` adds SQLite while preserving the PostgreSQL `0.1.0` API. It provides:

- environment and programmatic configuration
- explicit asynchronous SQLx PostgreSQL and SQLite pools
- health checks
- transactions
- application-owned migration execution
- classified errors for each database engine
- `tracing` instrumentation
- independently managed named connections with non-secret provider metadata

Neon, Supabase, hosted PostgreSQL, and local PostgreSQL use the same Storexa
PostgreSQL implementation. Provider selection is configuration, not CRUD logic.

## Example

```rust,no_run
use storexa::{Database, DatabaseConfig};

# async fn example() -> storexa::Result<()> {
let config = DatabaseConfig::from_env()?;
let db = Database::connect(config).await?;

let health = db.health().await?;
assert!(!health.server_version.is_empty());

let mut transaction = db.begin().await?;
// Applications execute their own SQL with SQLx here.
transaction.rollback().await?;

db.close().await;
# Ok(())
# }
```

`DatabaseConfig::from_env()` loads `.env` when present, then reads
`STOREXA_DATABASE_URL` or `DATABASE_URL`, in that order. Connection URLs are
redacted from debug output and are never emitted by Storexa's tracing spans.

Applications may select any exported secret explicitly without coupling
Storexa to the secret manager:

```rust,no_run
use storexa::DatabaseConfig;

let config = DatabaseConfig::from_env_var("PHOTARA_DEV_DATABASE_URL")?;
# Ok::<(), storexa::StorexaError>(())
```

Or read a value directly from a dotenv file without exporting every secret in
that file into the process:

```rust,no_run
use storexa::DatabaseConfig;

let config = DatabaseConfig::from_dotenv_var(
    "/run/secrets/photara.env",
    "PHOTARA_DEV_DATABASE_URL",
)?;
# Ok::<(), storexa::StorexaError>(())
```

See [Configuration and secrets](CONFIGURATION.md) for the full contract.
See [Database compatibility](COMPATIBILITY.md) for verified provider and
pooler modes, and [Data-transfer boundaries](TRANSFERS.md) for the intentionally
deferred migration layer.
The supported surface is listed in [Public API contract](API.md), and
security reporting guidance is in [SECURITY.md](SECURITY.md).

## Multiple connections and providers

Applications create and own as many independent `Database` handles as they
need. Names and providers are diagnostic metadata; they do not select a
different SQL implementation.

```rust,no_run
use storexa::{Database, DatabaseConfig, PostgresProvider};

# async fn connect_both() -> storexa::Result<()> {
let primary = Database::connect(
    DatabaseConfig::from_env_var("PHOTARA_DEV_DATABASE_URL")?
        .with_name("primary")?
        .with_provider(PostgresProvider::Neon)?,
).await?;

let archive = Database::connect(
    DatabaseConfig::from_env_var("PHOTARA_ARCHIVE_DATABASE_URL")?
        .with_name("archive")?
        .with_provider(PostgresProvider::Supabase)?,
).await?;

assert_eq!(primary.name(), "primary");
assert_eq!(archive.name(), "archive");
# Ok(())
# }
```

Storexa does not keep a global connection registry. The application decides
how handles are stored, shared, and shut down. Provider project creation,
branch management, and API credentials belong to a separate future control
plane; database URLs remain sufficient for PostgreSQL access.

## Migrations

Applications own their migration files and pass a SQLx migrator to Storexa:

```rust,no_run
use sqlx::migrate::Migrator;
use storexa::Database;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

# async fn migrate(db: &Database) -> storexa::Result<()> {
let report = db.run_migrations(&MIGRATOR).await?;
assert_eq!(report.available, MIGRATOR.iter().count());
# Ok(())
# }
```

## SQLite

```rust,no_run
use storexa::{SqliteDatabase, SqliteDatabaseConfig, SqliteJournalMode};

# async fn example() -> storexa::SqliteResult<()> {
let db = SqliteDatabase::connect(
    SqliteDatabaseConfig::from_path("application.sqlite")?
        .with_create_if_missing(true)
        .with_journal_mode(Some(SqliteJournalMode::Wal)),
).await?;
assert!(!db.health().await?.sqlite_version.is_empty());
db.close().await;
# Ok(())
# }
```

Use `SqliteDatabaseConfig::in_memory()` for an isolated ephemeral database.
SQLite has its own `SqliteTransaction`, `SqliteError`, `SqliteResult`, and concrete
`SqlitePool`; existing `Database`, `DatabaseConfig`, `Transaction`, and
`StorexaError` remain PostgreSQL-specific. No `Any` driver or ORM is involved.

Defaults are one connection, foreign keys enabled, a five-second busy timeout,
FULL synchronous mode, and preservation of the existing journal mode. Creating
a file requires explicit opt-in; parent directories and file permissions belong
to the application. WAL is an explicit choice for local filesystems, not shared
network storage. In-memory pools retain one connection with idle/lifetime
recycling disabled; closing or losing that connection destroys their contents.
See the executable [SQLite example](examples/sqlite_repository.rs).

SQLite uses the same `run_migrations` contract with SQLite-specific application
SQL. Storexa does not translate SQL or share migrations between engines.

## License

Storexa is licensed under the MIT License.
