//! Application-owned SQLite schema and SQL over Storexa lifecycle primitives.
use storexa::{SqliteDatabase, SqliteDatabaseConfig, SqliteResult};

#[tokio::main]
async fn main() -> SqliteResult<()> {
    let db = SqliteDatabase::connect(SqliteDatabaseConfig::in_memory()).await?;
    sqlx::query("CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT NOT NULL)")
        .execute(db.pool())
        .await?;
    let mut transaction = db.begin().await?;
    sqlx::query("INSERT INTO notes(body) VALUES (?)")
        .bind("Application-owned data")
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes")
        .fetch_one(db.pool())
        .await?;
    assert_eq!(count, 1);
    db.close().await;
    Ok(())
}
