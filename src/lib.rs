//! Domain-agnostic PostgreSQL persistence infrastructure.
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

mod config;
mod database;
mod error;

pub use config::{ConfigSource, DatabaseConfig};
pub use database::{
    ConnectionLease, Database, DatabaseStats, HealthReport, MigrationReport, Transaction,
};
pub use error::{Result, StorexaError};
pub use sqlx::migrate::Migrator;
pub use sqlx::{PgPool, Postgres};
