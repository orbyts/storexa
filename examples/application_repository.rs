use storexa::{Database, DatabaseConfig, PostgresProvider};

/// An application-owned repository built from Storexa and SQLx primitives.
struct ApplicationRepository<'a> {
    database: &'a Database,
}

impl<'a> ApplicationRepository<'a> {
    fn new(database: &'a Database) -> Self {
        Self { database }
    }

    async fn server_time(&self) -> storexa::Result<String> {
        sqlx::query_scalar("SELECT current_timestamp::text")
            .fetch_one(self.database.pool())
            .await
            .map_err(Into::into)
    }
}

#[tokio::main]
async fn main() -> storexa::Result<()> {
    let config = DatabaseConfig::from_env_var("APPLICATION_DATABASE_URL")?
        .with_name("primary")?
        .with_provider(PostgresProvider::Generic)?;
    let database = Database::connect(config).await?;

    let repository = ApplicationRepository::new(&database);
    let _server_time = repository.server_time().await?;

    database.close().await;
    Ok(())
}
