#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Domain-agnostic PostgreSQL and SQLite persistence infrastructure.
//!
//! Storexa owns connections, pooling, health checks, transactions, and
//! migration execution. Applications remain responsible for their schemas and
//! SQL.
//!
//! ```no_run
//! use storexa::{Database, DatabaseConfig};
//!
//! # async fn example() -> storexa::Result<()> {
//! let config = DatabaseConfig::from_env()?;
//! let db = Database::connect(config).await?;
//! db.health_check().await?;
//! # Ok(())
//! # }
//! ```
//!
//! SQLite keeps an explicit engine-specific API and error type:
//!
//! ```no_run
//! use storexa::{SqliteDatabase, SqliteDatabaseConfig};
//!
//! # async fn example() -> storexa::SqliteResult<()> {
//! let db = SqliteDatabase::connect(SqliteDatabaseConfig::in_memory()).await?;
//! let mut transaction = db.begin().await?;
//! sqlx::query("CREATE TABLE example (id INTEGER PRIMARY KEY)")
//!     .execute(&mut *transaction).await?;
//! transaction.commit().await?;
//! db.close().await;
//! # Ok(())
//! # }
//! ```

mod config;
mod database;
mod error;
mod metadata;
mod sqlite;

pub use config::{ConfigSource, DatabaseConfig};
pub use database::{
    ConnectionLease, Database, DatabaseStats, HealthReport, MigrationReport, Transaction,
};
pub use error::{Result, StorexaError};
pub use metadata::{DatabaseMetadata, PostgresProvider};
pub use sqlite::{
    SqliteConnectionLease, SqliteDatabase, SqliteDatabaseConfig, SqliteError, SqliteHealthReport,
    SqliteResult, SqliteTransaction,
};
pub use sqlx::migrate::Migrator;
pub use sqlx::sqlite::{SqliteJournalMode, SqliteSynchronous};
pub use sqlx::{PgPool, Postgres};
pub use sqlx::{Sqlite, SqlitePool};
