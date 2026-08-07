use std::{env, fmt, time::Duration};

use crate::{Result, StorexaError};

const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_MIN_CONNECTIONS: u32 = 0;
const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const DEFAULT_MAX_LIFETIME: Duration = Duration::from_secs(30 * 60);

/// PostgreSQL connection and pool configuration.
#[derive(Clone)]
pub struct DatabaseConfig {
    database_url: String,
    max_connections: u32,
    min_connections: u32,
    acquire_timeout: Duration,
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
}

impl DatabaseConfig {
    /// Creates configuration from a PostgreSQL connection URL.
    pub fn from_url(database_url: impl Into<String>) -> Result<Self> {
        let database_url = database_url.into();
        if database_url.trim().is_empty() {
            return Err(StorexaError::configuration(
                "database URL must not be empty",
            ));
        }

        Ok(Self {
            database_url,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            acquire_timeout: DEFAULT_ACQUIRE_TIMEOUT,
            idle_timeout: Some(DEFAULT_IDLE_TIMEOUT),
            max_lifetime: Some(DEFAULT_MAX_LIFETIME),
        })
    }

    /// Loads `.env`, then reads `STOREXA_DATABASE_URL` or `DATABASE_URL`.
    ///
    /// `STOREXA_DATABASE_URL` takes precedence when both variables are set.
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let database_url = env::var("STOREXA_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .map_err(|_| StorexaError::configuration("set STOREXA_DATABASE_URL or DATABASE_URL"))?;

        Self::from_url(database_url)
    }

    /// Overrides the database URL programmatically.
    pub fn with_database_url(mut self, database_url: impl Into<String>) -> Result<Self> {
        let database_url = database_url.into();
        if database_url.trim().is_empty() {
            return Err(StorexaError::configuration(
                "database URL must not be empty",
            ));
        }
        self.database_url = database_url;
        Ok(self)
    }

    /// Sets the maximum number of connections in the SQLx pool.
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Sets the minimum number of idle connections maintained by the pool.
    pub fn with_min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = min_connections;
        self
    }

    /// Sets how long pool acquisition may wait before timing out.
    pub fn with_acquire_timeout(mut self, acquire_timeout: Duration) -> Self {
        self.acquire_timeout = acquire_timeout;
        self
    }

    /// Sets the idle connection timeout. `None` disables it.
    pub fn with_idle_timeout(mut self, idle_timeout: Option<Duration>) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    /// Sets the maximum connection lifetime. `None` disables it.
    pub fn with_max_lifetime(mut self, max_lifetime: Option<Duration>) -> Self {
        self.max_lifetime = max_lifetime;
        self
    }

    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    pub(crate) fn max_connections(&self) -> u32 {
        self.max_connections
    }

    pub(crate) fn min_connections(&self) -> u32 {
        self.min_connections
    }

    pub(crate) fn acquire_timeout(&self) -> Duration {
        self.acquire_timeout
    }

    pub(crate) fn idle_timeout(&self) -> Option<Duration> {
        self.idle_timeout
    }

    pub(crate) fn max_lifetime(&self) -> Option<Duration> {
        self.max_lifetime
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.max_connections == 0 {
            return Err(StorexaError::configuration(
                "max_connections must be greater than zero",
            ));
        }
        if self.min_connections > self.max_connections {
            return Err(StorexaError::configuration(
                "min_connections must not exceed max_connections",
            ));
        }
        if self.acquire_timeout.is_zero() {
            return Err(StorexaError::configuration(
                "acquire_timeout must be greater than zero",
            ));
        }
        Ok(())
    }
}

impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DatabaseConfig")
            .field("database_url", &"[REDACTED]")
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("idle_timeout", &self.idle_timeout)
            .field("max_lifetime", &self.max_lifetime)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::DatabaseConfig;

    #[test]
    fn rejects_empty_urls() {
        assert!(DatabaseConfig::from_url("  ").is_err());
    }

    #[test]
    fn redacts_url_in_debug_output() {
        let config = DatabaseConfig::from_url("postgresql://user:secret@example.com/database")
            .expect("valid config");

        let output = format!("{config:?}");
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("secret"));
        assert!(!output.contains("example.com"));
    }

    #[test]
    fn validates_pool_limits() {
        let config = DatabaseConfig::from_url("postgresql://localhost/test")
            .expect("valid config")
            .with_min_connections(2)
            .with_max_connections(1);

        assert!(config.validate().is_err());
    }

    #[test]
    fn validates_acquire_timeout() {
        let config = DatabaseConfig::from_url("postgresql://localhost/test")
            .expect("valid config")
            .with_acquire_timeout(Duration::ZERO);

        assert!(config.validate().is_err());
    }
}
