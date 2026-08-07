use storexa::{Database, DatabaseConfig, Migrator};

static MIGRATOR: Migrator = sqlx::migrate!("./tests/migrations");

#[tokio::test]
#[ignore = "requires DATABASE_URL for a disposable PostgreSQL database"]
async fn direct_connection_migrations_and_transactions() -> storexa::Result<()> {
    initialize_tracing();

    let db = Database::connect(
        DatabaseConfig::from_env_var("DATABASE_URL")?
            .with_min_connections(0)
            .with_max_connections(2),
    )
    .await?;

    assert!(!db.stats().closed);
    drop(db.acquire().await?);

    let health = db.health().await?;
    assert!(!health.server_version.is_empty());

    let migration = db.run_migrations(&MIGRATOR).await?;
    assert_eq!(migration.available, 1);

    let value = format!("rollback-{}", std::process::id());
    let before = count_value(&db, &value).await?;

    let mut transaction = db.begin().await?;
    sqlx::query("INSERT INTO storexa_smoke_test (value) VALUES ($1)")
        .bind(&value)
        .execute(&mut *transaction)
        .await?;

    let inside: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM storexa_smoke_test WHERE value = $1")
            .bind(&value)
            .fetch_one(&mut *transaction)
            .await?;
    assert_eq!(inside, before + 1);

    transaction.rollback().await?;

    assert_eq!(count_value(&db, &value).await?, before);

    let committed_value = format!("commit-{}", std::process::id());
    let committed_before = count_value(&db, &committed_value).await?;
    let mut transaction = db.begin().await?;
    sqlx::query("INSERT INTO storexa_smoke_test (value) VALUES ($1)")
        .bind(&committed_value)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    assert_eq!(
        count_value(&db, &committed_value).await?,
        committed_before + 1
    );

    sqlx::query("DELETE FROM storexa_smoke_test WHERE value = $1")
        .bind(&committed_value)
        .execute(db.pool())
        .await?;
    assert_eq!(count_value(&db, &committed_value).await?, committed_before);

    db.close().await;
    assert!(db.stats().closed);
    Ok(())
}

#[tokio::test]
#[ignore = "requires STOREXA_TEST_POOLED_DATABASE_URL"]
async fn pooled_connection_health_check() -> storexa::Result<()> {
    initialize_tracing();

    let database_url = std::env::var("STOREXA_TEST_POOLED_DATABASE_URL").map_err(|_| {
        storexa::StorexaError::Configuration {
            message: "set STOREXA_TEST_POOLED_DATABASE_URL".to_owned(),
        }
    })?;

    let db = Database::connect(
        DatabaseConfig::from_url(database_url)?
            .with_min_connections(0)
            .with_max_connections(2),
    )
    .await?;
    db.health_check().await?;
    db.close().await;
    assert!(db.stats().closed);
    Ok(())
}

async fn count_value(db: &Database, value: &str) -> storexa::Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM storexa_smoke_test WHERE value = $1")
        .bind(value)
        .fetch_one(db.pool())
        .await
        .map_err(Into::into)
}

fn initialize_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "storexa=debug".into()),
        )
        .with_test_writer()
        .try_init();
}
