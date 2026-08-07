use thiserror::Error;

/// The result type returned by Storexa operations.
pub type Result<T> = std::result::Result<T, StorexaError>;

/// Errors produced by Storexa's persistence infrastructure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StorexaError {
    /// Configuration is missing or invalid.
    #[error("database configuration error: {message}")]
    Configuration { message: String },

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

impl StorexaError {
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
}
