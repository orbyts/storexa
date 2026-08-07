use std::{
    env, fmt,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use sqlx::postgres::PgConnectOptions;

use crate::{DatabaseMetadata, PostgresProvider, Result, StorexaError};

const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_MIN_CONNECTIONS: u32 = 0;
const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const DEFAULT_MAX_LIFETIME: Duration = Duration::from_secs(30 * 60);
const DEFAULT_DATABASE_URL_VARIABLES: [&str; 2] = ["STOREXA_DATABASE_URL", "DATABASE_URL"];

/// Describes where a database configuration obtained its secret URL.
///
/// This metadata contains only an environment-variable name or file path. It
/// never contains the URL itself.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConfigSource {
    /// The URL was supplied directly by application code.
    Programmatic,
    /// The URL was read from a process environment variable.
    Environment { variable: String },
    /// The URL was read directly from a dotenv file without modifying the
    /// process environment.
    Dotenv { path: PathBuf, variable: String },
}

/// PostgreSQL connection and pool configuration.
#[derive(Clone)]
pub struct DatabaseConfig {
    database_url: String,
    max_connections: u32,
    min_connections: u32,
    acquire_timeout: Duration,
    idle_timeout: Option<Duration>,
    max_lifetime: Option<Duration>,
    source: ConfigSource,
    metadata: DatabaseMetadata,
}

impl DatabaseConfig {
    /// Creates configuration from a PostgreSQL connection URL.
    pub fn from_url(database_url: impl Into<String>) -> Result<Self> {
        Self::from_url_and_source(database_url.into(), ConfigSource::Programmatic)
    }

    fn from_url_and_source(database_url: String, source: ConfigSource) -> Result<Self> {
        validate_database_url(&database_url)?;

        Ok(Self {
            database_url,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            acquire_timeout: DEFAULT_ACQUIRE_TIMEOUT,
            idle_timeout: Some(DEFAULT_IDLE_TIMEOUT),
            max_lifetime: Some(DEFAULT_MAX_LIFETIME),
            source,
            metadata: DatabaseMetadata::default(),
        })
    }

    /// Loads `.env`, then reads `STOREXA_DATABASE_URL` or `DATABASE_URL`.
    ///
    /// `STOREXA_DATABASE_URL` takes precedence when both variables are set.
    pub fn from_env() -> Result<Self> {
        if let Some(config) = first_available_environment_config()? {
            return Ok(config);
        }

        if let Ok(path) = dotenvy::dotenv()
            && let Some(mut config) = first_available_environment_config()?
        {
            let variable = match &config.source {
                ConfigSource::Environment { variable } => variable.clone(),
                _ => unreachable!("environment lookup returns an environment source"),
            };
            config.source = ConfigSource::Dotenv { path, variable };
            return Ok(config);
        }

        Err(StorexaError::configuration(
            "set STOREXA_DATABASE_URL or DATABASE_URL",
        ))
    }

    /// Reads a database URL from an explicitly named environment variable.
    ///
    /// This is useful when a host application or secret manager uses names
    /// such as `PHOTARA_DEV_DATABASE_URL`.
    pub fn from_env_var(variable: impl Into<String>) -> Result<Self> {
        let variable = variable.into();
        if variable.trim().is_empty() {
            return Err(StorexaError::configuration(
                "environment variable name must not be empty",
            ));
        }

        let database_url = env::var(&variable).map_err(|_| {
            StorexaError::configuration(format!(
                "database URL environment variable {variable} is not set or is not valid Unicode"
            ))
        })?;

        Self::from_url_and_source(database_url, ConfigSource::Environment { variable })
    }

    /// Reads a named database URL directly from a dotenv file.
    ///
    /// Unlike [`dotenvy::from_path`], this does not add any values to the
    /// process environment.
    pub fn from_dotenv_var(path: impl AsRef<Path>, variable: impl Into<String>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let variable = variable.into();
        if variable.trim().is_empty() {
            return Err(StorexaError::configuration(
                "dotenv variable name must not be empty",
            ));
        }

        let entries = dotenvy::from_path_iter(&path).map_err(|_| {
            StorexaError::configuration(format!("could not read dotenv file {}", path.display()))
        })?;

        for entry in entries {
            let (key, value) = entry.map_err(|_| {
                StorexaError::configuration(format!(
                    "could not parse dotenv file {}",
                    path.display()
                ))
            })?;
            if key == variable {
                return Self::from_url_and_source(value, ConfigSource::Dotenv { path, variable });
            }
        }

        Err(StorexaError::configuration(format!(
            "database URL variable {variable} was not found in dotenv file {}",
            path.display()
        )))
    }

    /// Overrides the database URL programmatically.
    pub fn with_database_url(mut self, database_url: impl Into<String>) -> Result<Self> {
        let database_url = database_url.into();
        validate_database_url(&database_url)?;
        self.database_url = database_url;
        self.source = ConfigSource::Programmatic;
        Ok(self)
    }

    /// Returns non-secret metadata describing where configuration was loaded.
    pub fn source(&self) -> &ConfigSource {
        &self.source
    }

    /// Assigns an application-defined name to this connection pool.
    pub fn with_name(mut self, name: impl Into<String>) -> Result<Self> {
        let provider = self.metadata.provider().clone();
        self.metadata = DatabaseMetadata::named(name)?.with_provider(provider)?;
        Ok(self)
    }

    /// Attaches descriptive PostgreSQL provider metadata.
    ///
    /// Provider metadata does not alter connection or SQL behavior.
    pub fn with_provider(mut self, provider: PostgresProvider) -> Result<Self> {
        self.metadata = self.metadata.with_provider(provider)?;
        Ok(self)
    }

    /// Returns the non-secret identity for this connection pool.
    pub fn metadata(&self) -> &DatabaseMetadata {
        &self.metadata
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
            .field("source", &self.source)
            .field("metadata", &self.metadata)
            .finish()
    }
}

fn first_available_environment_config() -> Result<Option<DatabaseConfig>> {
    for variable in DEFAULT_DATABASE_URL_VARIABLES {
        match env::var(variable) {
            Ok(database_url) => {
                return DatabaseConfig::from_url_and_source(
                    database_url,
                    ConfigSource::Environment {
                        variable: variable.to_owned(),
                    },
                )
                .map(Some);
            }
            Err(env::VarError::NotPresent) => {}
            Err(env::VarError::NotUnicode(_)) => {
                return Err(StorexaError::configuration(format!(
                    "database URL environment variable {variable} is not valid Unicode"
                )));
            }
        }
    }
    Ok(None)
}

fn validate_database_url(database_url: &str) -> Result<()> {
    if database_url.trim().is_empty() {
        return Err(StorexaError::configuration(
            "database URL must not be empty",
        ));
    }

    if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
        return Err(StorexaError::configuration(
            "database URL is not a valid PostgreSQL connection URL",
        ));
    }

    PgConnectOptions::from_str(database_url).map_err(|_| {
        StorexaError::configuration("database URL is not a valid PostgreSQL connection URL")
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use super::{ConfigSource, DatabaseConfig};

    #[test]
    fn rejects_empty_urls() {
        assert!(DatabaseConfig::from_url("  ").is_err());
    }

    #[test]
    fn rejects_non_postgresql_urls_without_echoing_them() {
        let error = DatabaseConfig::from_url("https://user:secret@example.com")
            .expect_err("non-PostgreSQL URLs must fail");
        let output = error.to_string();
        assert!(!output.contains("secret"));
        assert!(!output.contains("example.com"));
    }

    #[test]
    fn records_programmatic_source() {
        let config = DatabaseConfig::from_url("postgresql://localhost/test").expect("valid config");
        assert_eq!(config.source(), &ConfigSource::Programmatic);
    }

    #[test]
    fn reads_one_secret_from_dotenv_without_exporting_it() {
        let variable = format!("STOREXA_TEST_URL_{}", std::process::id());
        let path = std::env::temp_dir().join(format!("storexa-{}.env", std::process::id()));
        fs::write(
            &path,
            format!("UNRELATED=value\n{variable}=postgresql://localhost/test\n"),
        )
        .expect("write dotenv fixture");

        let config =
            DatabaseConfig::from_dotenv_var(&path, &variable).expect("load config from dotenv");
        assert_eq!(
            config.source(),
            &ConfigSource::Dotenv {
                path: path.clone(),
                variable: variable.clone(),
            }
        );
        assert!(std::env::var(&variable).is_err());

        fs::remove_file(path).expect("remove dotenv fixture");
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
