use std::fmt;

use thiserror::Error;

/// The result type returned by Storexa operations.
pub type Result<T> = std::result::Result<T, StorexaError>;

/// Errors produced by Storexa's persistence infrastructure.
#[derive(Error)]
#[non_exhaustive]
pub enum StorexaError {
    /// Configuration is missing or invalid.
    #[error("database configuration error: {message}")]
    Configuration {
        /// Secret-safe explanation of the configuration problem.
        message: String,
    },

    /// The PostgreSQL pool could not connect.
    #[error("failed to connect to PostgreSQL")]
    Connection(#[source] sqlx::Error),

    /// A connected database failed its health check.
    #[error("PostgreSQL health check failed")]
    HealthCheck(#[source] sqlx::Error),

    /// A transaction could not begin.
    #[error("failed to begin PostgreSQL transaction")]
    TransactionBegin(#[source] sqlx::Error),

    /// A transaction could not commit.
    #[error("failed to commit PostgreSQL transaction")]
    TransactionCommit(#[source] sqlx::Error),

    /// A transaction could not roll back.
    #[error("failed to roll back PostgreSQL transaction")]
    TransactionRollback(#[source] sqlx::Error),

    /// Application-owned SQL returned an error through a Storexa primitive.
    #[error("PostgreSQL query failed")]
    Query(#[source] sqlx::Error),

    /// Application-owned migrations could not run.
    #[error("failed to run PostgreSQL migrations")]
    Migration(#[source] sqlx::migrate::MigrateError),
}

impl From<sqlx::Error> for StorexaError {
    fn from(error: sqlx::Error) -> Self {
        Self::Query(error)
    }
}

impl fmt::Debug for StorexaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration { message } => formatter
                .debug_struct("Configuration")
                .field("message", message)
                .finish(),
            Self::Connection(_) => formatter.write_str("Connection(..)"),
            Self::HealthCheck(_) => formatter.write_str("HealthCheck(..)"),
            Self::TransactionBegin(_) => formatter.write_str("TransactionBegin(..)"),
            Self::TransactionCommit(_) => formatter.write_str("TransactionCommit(..)"),
            Self::TransactionRollback(_) => formatter.write_str("TransactionRollback(..)"),
            Self::Query(_) => formatter.write_str("Query(..)"),
            Self::Migration(_) => formatter.write_str("Migration(..)"),
        }
    }
}

impl StorexaError {
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StorexaError;

    #[test]
    fn redacts_driver_details_from_debug_output() {
        let error = StorexaError::Connection(sqlx::Error::Protocol(
            "postgresql://user:secret@example.com/database".to_owned(),
        ));

        let debug = format!("{error:?}");
        assert_eq!(debug, "Connection(..)");
        assert!(!debug.contains("secret"));
        assert!(!debug.contains("example.com"));
    }
}
