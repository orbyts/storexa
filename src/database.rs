use std::{
    fmt,
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

use sqlx::{PgConnection, PgPool, Postgres, pool::PoolConnection, postgres::PgPoolOptions};
use tracing::{debug, info, instrument};

use crate::{DatabaseConfig, Result, StorexaError};

/// A PostgreSQL connection checked out from a [`Database`] pool.
pub type ConnectionLease = PoolConnection<Postgres>;

/// A snapshot of local SQLx pool state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatabaseStats {
    /// Whether the pool has been closed.
    pub closed: bool,
    /// Total connections currently managed by the pool.
    pub size: u32,
    /// Connections currently idle and ready for acquisition.
    pub idle: usize,
}

/// Details returned by a successful database health check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthReport {
    /// PostgreSQL's server version string.
    pub server_version: String,
    /// End-to-end duration of the health query.
    pub latency: Duration,
}

/// Details returned after migrations are validated and applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationReport {
    /// Number of application migrations known to the migrator.
    pub available: usize,
    /// Time spent validating and applying migrations.
    pub elapsed: Duration,
}

/// A pool-owned PostgreSQL transaction.
///
/// Storexa classifies begin, commit, and rollback failures while exposing the
/// underlying PostgreSQL connection for application-owned SQL.
pub struct Transaction {
    inner: sqlx::Transaction<'static, Postgres>,
}

impl Transaction {
    /// Commits all work performed in this transaction.
    #[instrument(name = "storexa.transaction.commit", skip(self))]
    pub async fn commit(self) -> Result<()> {
        self.inner
            .commit()
            .await
            .map_err(StorexaError::TransactionCommit)?;
        debug!("database transaction committed");
        Ok(())
    }

    /// Rolls back all work performed in this transaction.
    #[instrument(name = "storexa.transaction.rollback", skip(self))]
    pub async fn rollback(self) -> Result<()> {
        self.inner
            .rollback()
            .await
            .map_err(StorexaError::TransactionRollback)?;
        debug!("database transaction rolled back");
        Ok(())
    }
}

impl Deref for Transaction {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Transaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Transaction")
            .finish_non_exhaustive()
    }
}

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

    /// Acquires one PostgreSQL connection from the pool.
    #[instrument(name = "storexa.database.acquire", skip(self))]
    pub async fn acquire(&self) -> Result<ConnectionLease> {
        self.pool.acquire().await.map_err(StorexaError::Connection)
    }

    /// Returns a local snapshot of pool lifecycle and utilization state.
    pub fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            closed: self.pool.is_closed(),
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }

    /// Checks that PostgreSQL can execute a query.
    #[instrument(name = "storexa.database.health", skip(self))]
    pub async fn health(&self) -> Result<HealthReport> {
        let started = Instant::now();
        let server_version: String = sqlx::query_scalar("SELECT current_setting('server_version')")
            .fetch_one(&self.pool)
            .await
            .map_err(StorexaError::HealthCheck)?;
        let latency = started.elapsed();

        debug!(
            latency_ms = latency.as_millis(),
            "database health check succeeded"
        );
        Ok(HealthReport {
            server_version,
            latency,
        })
    }

    /// Checks that PostgreSQL is reachable, discarding diagnostic details.
    pub async fn health_check(&self) -> Result<()> {
        self.health().await.map(|_| ())
    }

    /// Begins a pool-owned transaction.
    #[instrument(name = "storexa.database.begin", skip(self))]
    pub async fn begin(&self) -> Result<Transaction> {
        let inner = self
            .pool
            .begin()
            .await
            .map_err(StorexaError::TransactionBegin)?;
        debug!("database transaction begun");
        Ok(Transaction { inner })
    }

    /// Validates and runs application-owned SQLx migrations.
    #[instrument(
        name = "storexa.database.migrate",
        skip(self, migrator),
        fields(db.migrations.available = migrator.iter().count())
    )]
    pub async fn run_migrations(
        &self,
        migrator: &sqlx::migrate::Migrator,
    ) -> Result<MigrationReport> {
        let available = migrator.iter().count();
        let started = Instant::now();
        migrator
            .run(&self.pool)
            .await
            .map_err(StorexaError::Migration)?;
        let elapsed = started.elapsed();

        info!(
            available,
            elapsed_ms = elapsed.as_millis(),
            "database migrations completed"
        );
        Ok(MigrationReport { available, elapsed })
    }

    /// Gracefully closes the pool and waits for checked-out connections.
    ///
    /// Closing one clone closes the shared pool for every clone.
    #[instrument(name = "storexa.database.close", skip(self))]
    pub async fn close(&self) {
        self.pool.close().await;
        debug!("database pool closed");
    }
}

impl fmt::Debug for Database {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Database")
            .field("stats", &self.stats())
            .finish()
    }
}
