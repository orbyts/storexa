# Storexa

Storexa is a small, domain-agnostic PostgreSQL persistence foundation for Rust
applications. It uses SQLx and deliberately does not generate application SQL
or model application entities.

Version `0.0.2` provides:

- environment and programmatic configuration
- an asynchronous SQLx PostgreSQL pool
- health checks
- transactions
- application-owned migration execution
- one consistent error type
- `tracing` instrumentation

Neon, Supabase, hosted PostgreSQL, and local PostgreSQL use the same Storexa
PostgreSQL implementation. Provider selection is configuration, not CRUD logic.

## Example

```rust,no_run
use storexa::{Database, DatabaseConfig};

# async fn example() -> storexa::Result<()> {
let config = DatabaseConfig::from_env()?;
let db = Database::connect(config).await?;

db.health_check().await?;

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

## Migrations

Applications own their migration files and pass a SQLx migrator to Storexa:

```rust,no_run
use sqlx::migrate::Migrator;
use storexa::Database;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

# async fn migrate(db: &Database) -> storexa::Result<()> {
db.run_migrations(&MIGRATOR).await
# }
```

## License

Storexa is licensed under the MIT License.
