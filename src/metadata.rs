use std::fmt;

use crate::{Result, StorexaError};

const DEFAULT_DATABASE_NAME: &str = "default";
const MAX_METADATA_LENGTH: usize = 128;

/// Describes the PostgreSQL service behind a Storexa connection.
///
/// This value is diagnostic metadata only. Every variant uses the same SQLx
/// PostgreSQL implementation and application-owned SQL.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PostgresProvider {
    /// PostgreSQL without provider-specific identification.
    #[default]
    Generic,
    /// Neon PostgreSQL.
    Neon,
    /// Supabase PostgreSQL.
    Supabase,
    /// A PostgreSQL server running in the application's local environment.
    Local,
    /// Another PostgreSQL-compatible provider.
    Other(String),
}

impl PostgresProvider {
    fn validate(&self) -> Result<()> {
        if let Self::Other(provider) = self {
            validate_metadata_value("provider name", provider)?;
        }
        Ok(())
    }
}

impl fmt::Display for PostgresProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic => formatter.write_str("postgresql"),
            Self::Neon => formatter.write_str("neon"),
            Self::Supabase => formatter.write_str("supabase"),
            Self::Local => formatter.write_str("local-postgresql"),
            Self::Other(provider) => formatter.write_str(provider),
        }
    }
}

/// Non-secret identity attached to one database connection pool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatabaseMetadata {
    name: String,
    provider: PostgresProvider,
}

impl DatabaseMetadata {
    /// Creates metadata for a named PostgreSQL connection.
    pub fn named(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_metadata_value("database name", &name)?;
        Ok(Self {
            name,
            provider: PostgresProvider::Generic,
        })
    }

    /// Sets descriptive provider metadata.
    pub fn with_provider(mut self, provider: PostgresProvider) -> Result<Self> {
        provider.validate()?;
        self.provider = provider;
        Ok(self)
    }

    /// Returns the application-defined connection name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the descriptive PostgreSQL provider.
    pub fn provider(&self) -> &PostgresProvider {
        &self.provider
    }
}

impl Default for DatabaseMetadata {
    fn default() -> Self {
        Self {
            name: DEFAULT_DATABASE_NAME.to_owned(),
            provider: PostgresProvider::Generic,
        }
    }
}

fn validate_metadata_value(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(StorexaError::configuration(format!(
            "{label} must not be empty"
        )));
    }
    if value.chars().count() > MAX_METADATA_LENGTH {
        return Err(StorexaError::configuration(format!(
            "{label} must not exceed {MAX_METADATA_LENGTH} characters"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(StorexaError::configuration(format!(
            "{label} must not contain control characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DatabaseMetadata, PostgresProvider};

    #[test]
    fn defaults_to_generic_postgresql() {
        let metadata = DatabaseMetadata::default();
        assert_eq!(metadata.name(), "default");
        assert_eq!(metadata.provider(), &PostgresProvider::Generic);
    }

    #[test]
    fn creates_named_provider_metadata() {
        let metadata = DatabaseMetadata::named("development")
            .expect("valid name")
            .with_provider(PostgresProvider::Neon)
            .expect("valid provider");

        assert_eq!(metadata.name(), "development");
        assert_eq!(metadata.provider(), &PostgresProvider::Neon);
    }

    #[test]
    fn rejects_unsafe_metadata() {
        assert!(DatabaseMetadata::named("\n").is_err());
        assert!(
            DatabaseMetadata::default()
                .with_provider(PostgresProvider::Other("\rprovider".to_owned()))
                .is_err()
        );
    }
}
