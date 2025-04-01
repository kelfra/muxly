use anyhow::Result;
use sqlx::Pool;
use sqlx::Sqlite;

/// Database connection pool type
pub type DatabasePool = Pool<Sqlite>;

/// Initialize a new SQLite database connection
pub async fn init_sqlite_database(database_url: &str) -> Result<DatabasePool> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    Ok(pool)
} 