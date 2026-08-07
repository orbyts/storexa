use std::fmt;

use sqlx::{PgPool, Postgres, postgres::PgPoolOptions};
use tracing::{debug, info, instrument};

use crate::{DatabaseConfig, Result, StorexaError};

/// A SQLx PostgreSQL transaction acquired from a [`Database`].
pub type Transaction<'connection> = sqlx::Transaction<'connection, Postgres>;

/// A cloneable handle to a PostgreSQL connection pool.
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Creates and verifies a PostgreSQL connection pool.
    #[instrument(
        name = "storexa.database.connect",
        skip(config),
        fields(
            db.system = "postgresql",
            db.max_connections = config.max_connections(),
            db.min_connections = config.min_connections()
        )
    )]
    pub async fn connect(config: DatabaseConfig) -> Result<Self> {
        config.validate()?;

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections())
            .min_connections(config.min_connections())
            .acquire_timeout(config.acquire_timeout())
            .idle_timeout(config.idle_timeout())
            .max_lifetime(config.max_lifetime())
            .connect(config.database_url())
            .await
            .map_err(StorexaError::Connection)?;

        info!("database pool connected");
        Ok(Self { pool })
    }

    /// Returns the underlying SQLx PostgreSQL pool for application-owned SQL.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Checks that PostgreSQL can execute a simple query.
    #[instrument(name = "storexa.database.health_check", skip(self))]
    pub async fn health_check(&self) -> Result<()> {
        let _: i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(StorexaError::HealthCheck)?;

        debug!("database health check succeeded");
        Ok(())
    }

    /// Begins a transaction from the pool.
    #[instrument(name = "storexa.database.begin", skip(self))]
    pub async fn begin(&self) -> Result<Transaction<'static>> {
        self.pool.begin().await.map_err(StorexaError::Transaction)
    }

    /// Runs application-owned SQLx migrations.
    #[instrument(name = "storexa.database.migrate", skip(self, migrator))]
    pub async fn run_migrations(&self, migrator: &sqlx::migrate::Migrator) -> Result<()> {
        migrator
            .run(&self.pool)
            .await
            .map_err(StorexaError::Migration)?;
        info!("database migrations completed");
        Ok(())
    }

    /// Gracefully closes the pool and waits for checked-out connections.
    #[instrument(name = "storexa.database.close", skip(self))]
    pub async fn close(self) {
        self.pool.close().await;
        debug!("database pool closed");
    }
}

impl fmt::Debug for Database {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Database")
            .field("closed", &self.pool.is_closed())
            .field("size", &self.pool.size())
            .field("idle", &self.pool.num_idle())
            .finish()
    }
}
