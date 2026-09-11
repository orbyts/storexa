use std::{error::Error, time::Duration};

use storexa::{
    Migrator, SqliteDatabase, SqliteDatabaseConfig, SqliteError, SqliteJournalMode, SqliteResult,
    SqliteSynchronous,
};

static MIGRATOR: Migrator = sqlx::migrate!("./tests/sqlite_migrations");

#[tokio::test]
async fn memory_migrations_transactions_and_lifecycle() -> SqliteResult<()> {
    let db = SqliteDatabase::connect(SqliteDatabaseConfig::in_memory().with_name("test")?).await?;
    assert_eq!(db.name(), "test");
    assert!(!db.health().await?.sqlite_version.is_empty());
    assert_eq!(db.run_migrations(&MIGRATOR).await?.available, 1);
    assert_eq!(db.run_migrations(&MIGRATOR).await?.available, 1);
    for (value, commit) in [("commit", true), ("rollback", false)] {
        let mut tx = db.begin().await?;
        sqlx::query("INSERT INTO records(value) VALUES (?)")
            .bind(value)
            .execute(&mut *tx)
            .await?;
        if commit {
            tx.commit().await?;
        } else {
            tx.rollback().await?;
        }
    }
    {
        let mut tx = db.begin().await?;
        sqlx::query("INSERT INTO records(value) VALUES ('dropped')")
            .execute(&mut *tx)
            .await?;
    }
    let values: Vec<String> = sqlx::query_scalar("SELECT value FROM records ORDER BY id")
        .fetch_all(db.pool())
        .await?;
    assert_eq!(values, ["commit"]);
    assert!(
        sqlx::query("INSERT INTO children(record_id) VALUES (999)")
            .execute(db.pool())
            .await
            .is_err()
    );
    let another = SqliteDatabase::connect(SqliteDatabaseConfig::in_memory()).await?;
    assert!(
        sqlx::query("SELECT * FROM records")
            .execute(another.pool())
            .await
            .is_err()
    );
    another.close().await;
    let clone = db.clone();
    db.close().await;
    assert!(clone.stats().closed);
    assert!(matches!(
        clone.acquire().await,
        Err(SqliteError::Connection(_))
    ));
    assert!(matches!(
        clone.health().await,
        Err(SqliteError::HealthCheck(_))
    ));
    assert!(matches!(
        clone.begin().await,
        Err(SqliteError::TransactionBegin(_))
    ));
    Ok(())
}

#[tokio::test]
async fn file_persistence_and_each_connection_configuration() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("private database.sqlite");
    let config = SqliteDatabaseConfig::from_path(&path)?;
    assert!(matches!(
        SqliteDatabase::connect(config.clone()).await,
        Err(SqliteError::Connection(_))
    ));
    assert!(!path.exists());
    let db = SqliteDatabase::connect(
        config
            .clone()
            .with_create_if_missing(true)
            .with_max_connections(2)
            .with_min_connections(2)
            .with_journal_mode(Some(SqliteJournalMode::Wal))
            .with_busy_timeout(Duration::from_millis(1234))
            .with_synchronous(SqliteSynchronous::Normal),
    )
    .await?;
    db.run_migrations(&MIGRATOR).await?;
    let mut first = db.acquire().await?;
    let mut second = db.acquire().await?;
    for lease in [&mut first, &mut second] {
        let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&mut **lease)
            .await?;
        let fk: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&mut **lease)
            .await?;
        let busy: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(&mut **lease)
            .await?;
        let sync: i64 = sqlx::query_scalar("PRAGMA synchronous")
            .fetch_one(&mut **lease)
            .await?;
        assert_eq!((journal.as_str(), fk, busy, sync), ("wal", 1, 1234, 1));
    }
    drop(first);
    drop(second);
    sqlx::query("INSERT INTO records(value) VALUES ('persistent')")
        .execute(db.pool())
        .await?;
    assert!(!format!("{db:?}").contains("private database"));
    db.close().await;
    let reopened = SqliteDatabase::connect(config).await?;
    let value: String = sqlx::query_scalar("SELECT value FROM records")
        .fetch_one(reopened.pool())
        .await?;
    assert_eq!(value, "persistent");
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(reopened.pool())
        .await?;
    assert_eq!(
        journal, "wal",
        "default must preserve the existing journal mode"
    );
    let sync: i64 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(reopened.pool())
        .await?;
    assert_eq!(sync, 2, "default must request FULL synchronous");
    reopened.close().await;
    Ok(())
}

#[tokio::test]
async fn migration_checksum_failure_is_classified() -> SqliteResult<()> {
    let db = SqliteDatabase::connect(SqliteDatabaseConfig::in_memory()).await?;
    db.run_migrations(&MIGRATOR).await?;
    sqlx::query("UPDATE _sqlx_migrations SET checksum = X'00'")
        .execute(db.pool())
        .await?;
    let error = db
        .run_migrations(&MIGRATOR)
        .await
        .expect_err("reject changed history");
    assert!(matches!(error, SqliteError::Migration(_)));
    assert!(error.source().is_some());
    assert_eq!(format!("{error:?}"), "Migration(..)");
    db.close().await;
    Ok(())
}

#[tokio::test]
async fn lock_contention_and_acquisition_failures_are_bounded() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let db = SqliteDatabase::connect(
        SqliteDatabaseConfig::from_path(directory.path().join("locks.sqlite"))?
            .with_create_if_missing(true)
            .with_max_connections(2)
            .with_min_connections(2)
            .with_busy_timeout(Duration::ZERO)
            .with_acquire_timeout(Duration::from_millis(50)),
    )
    .await?;
    db.run_migrations(&MIGRATOR).await?;
    let mut writer = db.begin().await?;
    sqlx::query("INSERT INTO records(value) VALUES ('writer')")
        .execute(&mut *writer)
        .await?;
    let mut reader = db.acquire().await?;
    let error = sqlx::query("INSERT INTO records(value) VALUES ('contended')")
        .execute(&mut *reader)
        .await
        .expect_err("writer holds lock");
    assert!(error.as_database_error().is_some());
    assert!(matches!(
        db.acquire().await,
        Err(SqliteError::Connection(sqlx::Error::PoolTimedOut))
    ));
    drop(reader);
    writer.rollback().await?;
    db.close().await;
    Ok(())
}

#[tokio::test]
async fn no_parent_creation_and_foreign_keys_can_be_disabled() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("absent").join("secret.sqlite");
    let error = SqliteDatabase::connect(
        SqliteDatabaseConfig::from_path(&path)?.with_create_if_missing(true),
    )
    .await
    .expect_err("parent is application owned");
    assert!(matches!(error, SqliteError::Connection(_)));
    assert!(!path.parent().expect("parent").exists());
    assert!(!format!("{error:?} {error}").contains("secret"));
    let db =
        SqliteDatabase::connect(SqliteDatabaseConfig::in_memory().with_foreign_keys(false)).await?;
    db.run_migrations(&MIGRATOR).await?;
    sqlx::query("INSERT INTO children(record_id) VALUES (999)")
        .execute(db.pool())
        .await?;
    db.close().await;
    Ok(())
}

#[test]
fn configuration_and_debug_redaction() {
    for path in [
        "",
        ":memory:",
        "file:secret.sqlite?mode=memory",
        "sqlite://secret.sqlite",
        "bad\0secret",
        ".",
        "..",
    ] {
        let error = SqliteDatabaseConfig::from_path(path).expect_err("reject nonliteral path");
        assert!(!format!("{error:?} {error}").contains("secret"));
    }
    let config = SqliteDatabaseConfig::from_path("private/secret.sqlite").unwrap();
    assert!(!format!("{config:?}").contains("secret"));
    assert!(SqliteDatabaseConfig::in_memory().validate().is_ok());
    for invalid in [
        config.clone().with_max_connections(0),
        config.clone().with_min_connections(3),
        config.clone().with_acquire_timeout(Duration::ZERO),
        config.clone().with_idle_timeout(Some(Duration::ZERO)),
        config
            .clone()
            .with_busy_timeout(Duration::from_millis(i32::MAX as u64 + 1)),
        config.with_busy_timeout(Duration::from_nanos(1)),
        SqliteDatabaseConfig::in_memory().with_max_connections(2),
        SqliteDatabaseConfig::in_memory().with_min_connections(0),
        SqliteDatabaseConfig::in_memory().with_max_lifetime(Some(Duration::from_secs(1))),
        SqliteDatabaseConfig::in_memory().with_idle_timeout(Some(Duration::from_secs(1))),
        SqliteDatabaseConfig::in_memory().with_journal_mode(Some(SqliteJournalMode::Wal)),
    ] {
        assert!(invalid.validate().is_err());
    }
    assert!(SqliteDatabaseConfig::in_memory().with_name("\n").is_err());
    let error = SqliteError::from(sqlx::Error::Protocol("private/secret.sqlite".to_owned()));
    assert!(matches!(error, SqliteError::Query(_)));
    assert!(error.source().is_some());
    assert!(!format!("{error:?} {error}").contains("secret"));
}
