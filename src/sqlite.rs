//! SQLite-specific lifecycle without hiding SQL dialect or transaction behavior.

use std::{
    fmt,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use sqlx::{
    ConnectOptions, Sqlite, SqliteConnection, SqlitePool,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use thiserror::Error;
use tracing::{debug, instrument};

use crate::{DatabaseMetadata, DatabaseStats, MigrationReport};

/// Result returned by SQLite operations, including application SQL using `?`.
pub type SqliteResult<T> = std::result::Result<T, SqliteError>;

/// Classified SQLite failures. Display and Debug omit paths and driver details.
///
/// Wrapped sources may contain sensitive data; inspect them only deliberately.
#[derive(Error)]
#[non_exhaustive]
pub enum SqliteError {
    /// Invalid SQLite configuration.
    #[error("SQLite configuration error: {message}")]
    Configuration {
        /// Storexa-generated, path-free explanation.
        message: String,
    },
    /// Connecting or acquiring a pooled connection failed.
    #[error("failed to connect to SQLite")]
    Connection(#[source] sqlx::Error),
    /// A health query failed.
    #[error("SQLite health check failed")]
    HealthCheck(#[source] sqlx::Error),
    /// Beginning a transaction failed.
    #[error("failed to begin SQLite transaction")]
    TransactionBegin(#[source] sqlx::Error),
    /// Committing a transaction failed.
    #[error("failed to commit SQLite transaction")]
    TransactionCommit(#[source] sqlx::Error),
    /// Rolling back a transaction failed.
    #[error("failed to roll back SQLite transaction")]
    TransactionRollback(#[source] sqlx::Error),
    /// Application SQL failed.
    #[error("SQLite query failed")]
    Query(#[source] sqlx::Error),
    /// Application migrations failed.
    #[error("failed to run SQLite migrations")]
    Migration(#[source] sqlx::migrate::MigrateError),
}

impl SqliteError {
    fn configuration(message: &str) -> Self {
        Self::Configuration {
            message: message.to_owned(),
        }
    }
}

impl From<sqlx::Error> for SqliteError {
    fn from(error: sqlx::Error) -> Self {
        Self::Query(error)
    }
}

impl fmt::Debug for SqliteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration { message } => f
                .debug_struct("Configuration")
                .field("message", message)
                .finish(),
            Self::Connection(_) => f.write_str("Connection(..)"),
            Self::HealthCheck(_) => f.write_str("HealthCheck(..)"),
            Self::TransactionBegin(_) => f.write_str("TransactionBegin(..)"),
            Self::TransactionCommit(_) => f.write_str("TransactionCommit(..)"),
            Self::TransactionRollback(_) => f.write_str("TransactionRollback(..)"),
            Self::Query(_) => f.write_str("Query(..)"),
            Self::Migration(_) => f.write_str("Migration(..)"),
        }
    }
}

/// Explicit file or private in-memory SQLite configuration.
///
/// File paths are literal paths, not URLs. Parent directories and file permissions
/// belong to the application. Defaults: one connection, foreign keys enabled,
/// five-second busy timeout, FULL synchronous mode, and no journal-mode override.
#[derive(Clone)]
pub struct SqliteDatabaseConfig {
    path: Option<PathBuf>,
    name: String,
    create_if_missing: bool,
    max_connections: u32,
    min_connections: u32,
    acquire_timeout: Duration,
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
    busy_timeout: Duration,
    foreign_keys: bool,
    journal_mode: Option<SqliteJournalMode>,
    synchronous: SqliteSynchronous,
}

impl SqliteDatabaseConfig {
    /// Opens a literal file path; a missing file is an error unless opted in.
    /// Relative paths resolve against the process working directory at connect.
    pub fn from_path(path: impl AsRef<Path>) -> SqliteResult<Self> {
        let path = path.as_ref();
        let text = path.to_string_lossy();
        if path.as_os_str().is_empty()
            || path.file_name().is_none()
            || text == ":memory:"
            || text.starts_with("file:")
            || text.starts_with("sqlite:")
            || text.contains('\0')
        {
            return Err(SqliteError::configuration(
                "expected a nonempty literal database file path, not a SQLite URI or :memory:",
            ));
        }
        Ok(Self {
            path: Some(path.to_path_buf()),
            ..Self::in_memory()
        })
    }

    /// Creates a private, single-connection in-memory database.
    ///
    /// Idle and lifetime recycling are disabled. Independent pools are isolated;
    /// closing the pool or losing its physical connection loses all contents.
    pub fn in_memory() -> Self {
        Self {
            path: None,
            name: "default".to_owned(),
            create_if_missing: false,
            max_connections: 1,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: None,
            max_lifetime: None,
            busy_timeout: Duration::from_secs(5),
            foreign_keys: true,
            journal_mode: None,
            synchronous: SqliteSynchronous::Full,
        }
    }

    /// Assigns non-secret diagnostic identity; never put a path or secret here.
    pub fn with_name(mut self, name: impl Into<String>) -> SqliteResult<Self> {
        let metadata = DatabaseMetadata::named(name).map_err(|_| SqliteError::configuration(
            "database name must be nonempty, at most 128 characters, and contain no control characters"))?;
        self.name = metadata.name().to_owned();
        Ok(self)
    }
    /// Returns the application-defined diagnostic name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Allows creation of the database file, but never creates parent directories.
    pub fn with_create_if_missing(mut self, enabled: bool) -> Self {
        self.create_if_missing = enabled;
        self
    }
    /// Sets the pool maximum. In-memory databases require exactly one connection.
    pub fn with_max_connections(mut self, count: u32) -> Self {
        self.max_connections = count;
        self
    }
    /// Sets the pool minimum. In-memory databases require exactly one connection.
    pub fn with_min_connections(mut self, count: u32) -> Self {
        self.min_connections = count;
        self
    }
    /// Sets a nonzero acquisition timeout.
    pub fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }
    /// Sets idle recycling for file databases. In-memory databases require `None`.
    pub fn with_idle_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.idle_timeout = timeout;
        self
    }
    /// Sets lifetime recycling for file databases. In-memory databases require `None`.
    pub fn with_max_lifetime(mut self, timeout: Option<Duration>) -> Self {
        self.max_lifetime = timeout;
        self
    }
    /// Sets SQLite's lock wait, distinct from pool acquisition. Zero fails immediately.
    /// Values must fit SQLite's signed 32-bit millisecond timeout exactly.
    pub fn with_busy_timeout(mut self, timeout: Duration) -> Self {
        self.busy_timeout = timeout;
        self
    }
    /// Sets foreign-key enforcement on every new connection (enabled by default).
    pub fn with_foreign_keys(mut self, enabled: bool) -> Self {
        self.foreign_keys = enabled;
        self
    }
    /// Requests a journal mode on every connection. `None` preserves existing mode.
    ///
    /// WAL requires a local filesystem; Storexa does not detect network mounts.
    /// Changing journal mode may require exclusive access. Memory databases permit
    /// only `Memory` or no override. `Off` and `Memory` reduce file crash protection.
    pub fn with_journal_mode(mut self, mode: Option<SqliteJournalMode>) -> Self {
        self.journal_mode = mode;
        self
    }
    /// Sets synchronous policy on every connection. FULL is the default;
    /// less durable modes require an application-specific durability decision.
    pub fn with_synchronous(mut self, mode: SqliteSynchronous) -> Self {
        self.synchronous = mode;
        self
    }

    /// Validates pool limits and SQLite-specific configuration without doing I/O.
    pub fn validate(&self) -> SqliteResult<()> {
        if self.max_connections == 0 || self.min_connections > self.max_connections {
            return Err(SqliteError::configuration(
                "pool maximum must be positive and at least the minimum",
            ));
        }
        if self.acquire_timeout.is_zero()
            || self.idle_timeout.is_some_and(|t| t.is_zero())
            || self.max_lifetime.is_some_and(|t| t.is_zero())
        {
            return Err(SqliteError::configuration(
                "configured pool timeouts must be greater than zero",
            ));
        }
        if self.busy_timeout.as_millis() > i32::MAX as u128
            || !self.busy_timeout.subsec_nanos().is_multiple_of(1_000_000)
        {
            return Err(SqliteError::configuration(
                "busy_timeout must be whole milliseconds within SQLite's signed 32-bit range",
            ));
        }
        if self.path.is_none() {
            if self.max_connections != 1
                || self.min_connections != 1
                || self.idle_timeout.is_some()
                || self.max_lifetime.is_some()
            {
                return Err(SqliteError::configuration(
                    "in-memory databases require one retained connection without idle or lifetime recycling",
                ));
            }
            if self
                .journal_mode
                .is_some_and(|m| m != SqliteJournalMode::Memory)
            {
                return Err(SqliteError::configuration(
                    "in-memory databases support only the memory journal mode",
                ));
            }
        }
        Ok(())
    }

    fn options(&self) -> SqliteConnectOptions {
        let mut options = SqliteConnectOptions::new()
            .create_if_missing(self.create_if_missing)
            .foreign_keys(self.foreign_keys)
            .busy_timeout(self.busy_timeout)
            .synchronous(self.synchronous)
            .disable_statement_logging();
        if let Some(path) = &self.path {
            options = options.filename(path);
        } else {
            options = options.in_memory(true);
        }
        if let Some(mode) = self.journal_mode {
            options = options.journal_mode(mode);
        }
        options
    }
}

impl fmt::Debug for SqliteDatabaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteDatabaseConfig")
            .field("name", &self.name)
            .field("path", &"[REDACTED]")
            .field("in_memory", &self.path.is_none())
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("busy_timeout", &self.busy_timeout)
            .field("foreign_keys", &self.foreign_keys)
            .field("journal_mode", &self.journal_mode)
            .field("synchronous", &self.synchronous)
            .finish_non_exhaustive()
    }
}

/// A SQLite connection checked out from a pool.
pub type SqliteConnectionLease = PoolConnection<Sqlite>;

/// SQLite engine version and health-query duration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqliteHealthReport {
    /// SQLite's runtime engine version.
    pub sqlite_version: String,
    /// End-to-end health-query duration.
    pub latency: Duration,
}

/// Cloneable SQLite pool. Clones share lifecycle; closing one closes all.
#[derive(Clone)]
pub struct SqliteDatabase {
    pool: SqlitePool,
    name: String,
}

impl SqliteDatabase {
    /// Validates configuration and opens a verified pool. Does not run migrations.
    #[instrument(name = "storexa.sqlite.connect", skip(config), fields(db.system = "sqlite", db.connection.name = config.name()))]
    pub async fn connect(config: SqliteDatabaseConfig) -> SqliteResult<Self> {
        config.validate()?;
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect_with(config.options())
            .await
            .map_err(SqliteError::Connection)?;
        debug!("SQLite pool connected");
        Ok(Self {
            pool,
            name: config.name,
        })
    }
    /// Returns the non-secret diagnostic name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Exposes SQLx's concrete SQLite pool for application SQL.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    /// Acquires one connection, classifying pool errors.
    #[instrument(name = "storexa.sqlite.acquire", skip(self))]
    pub async fn acquire(&self) -> SqliteResult<SqliteConnectionLease> {
        self.pool.acquire().await.map_err(SqliteError::Connection)
    }
    /// Returns local pool utilization and lifecycle state.
    pub fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            closed: self.pool.is_closed(),
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }
    /// Queries the actual SQLite engine version and elapsed time.
    #[instrument(name = "storexa.sqlite.health", skip(self))]
    pub async fn health(&self) -> SqliteResult<SqliteHealthReport> {
        let started = Instant::now();
        let sqlite_version = sqlx::query_scalar("SELECT sqlite_version()")
            .fetch_one(&self.pool)
            .await
            .map_err(SqliteError::HealthCheck)?;
        Ok(SqliteHealthReport {
            sqlite_version,
            latency: started.elapsed(),
        })
    }
    /// Checks that SQLite executes a query, discarding version details.
    pub async fn health_check(&self) -> SqliteResult<()> {
        self.health().await.map(|_| ())
    }
    /// Begins a deferred SQLite transaction. Lock acquisition follows SQLite rules.
    #[instrument(name = "storexa.sqlite.begin", skip(self))]
    pub async fn begin(&self) -> SqliteResult<SqliteTransaction> {
        let inner = self
            .pool
            .begin()
            .await
            .map_err(SqliteError::TransactionBegin)?;
        Ok(SqliteTransaction { inner })
    }
    /// Validates and executes application-owned SQLite migrations.
    #[instrument(name = "storexa.sqlite.migrate", skip(self, migrator))]
    pub async fn run_migrations(
        &self,
        migrator: &sqlx::migrate::Migrator,
    ) -> SqliteResult<MigrationReport> {
        let started = Instant::now();
        migrator
            .run(&self.pool)
            .await
            .map_err(SqliteError::Migration)?;
        Ok(MigrationReport {
            available: migrator.iter().count(),
            elapsed: started.elapsed(),
        })
    }
    /// Gracefully closes the shared pool, waiting for outstanding leases/transactions.
    #[instrument(name = "storexa.sqlite.close", skip(self))]
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

impl fmt::Debug for SqliteDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteDatabase")
            .field("name", &self.name)
            .field("stats", &self.stats())
            .finish()
    }
}

/// Pool-owned SQLite transaction. Dropping queues rollback through SQLx.
/// Application SQL executes on the dereferenced SQLite connection.
pub struct SqliteTransaction {
    inner: sqlx::Transaction<'static, Sqlite>,
}
impl SqliteTransaction {
    /// Commits this transaction, classifying finalization failures.
    #[instrument(name = "storexa.sqlite.transaction.commit", skip(self))]
    pub async fn commit(self) -> SqliteResult<()> {
        self.inner
            .commit()
            .await
            .map_err(SqliteError::TransactionCommit)
    }
    /// Rolls back this transaction, classifying finalization failures.
    #[instrument(name = "storexa.sqlite.transaction.rollback", skip(self))]
    pub async fn rollback(self) -> SqliteResult<()> {
        self.inner
            .rollback()
            .await
            .map_err(SqliteError::TransactionRollback)
    }
}
impl Deref for SqliteTransaction {
    type Target = SqliteConnection;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for SqliteTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl fmt::Debug for SqliteTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteTransaction").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_driver_error_classes_do_not_disclose_sources() {
        for wrap in [
            SqliteError::Connection,
            SqliteError::HealthCheck,
            SqliteError::TransactionBegin,
            SqliteError::TransactionCommit,
            SqliteError::TransactionRollback,
            SqliteError::Query,
        ] {
            let error = wrap(sqlx::Error::Protocol("private-secret.sqlite".to_owned()));
            assert!(!format!("{error:?} {error}").contains("private-secret"));
        }
    }

    #[test]
    fn busy_timeout_boundary_and_memory_journal_are_valid() {
        assert!(
            SqliteDatabaseConfig::in_memory()
                .with_busy_timeout(Duration::from_millis(i32::MAX as u64))
                .with_journal_mode(Some(SqliteJournalMode::Memory))
                .validate()
                .is_ok()
        );
    }
}
