use sqlx::{Pool, Sqlite, SqlitePool};
use std::time::Duration;
use anyhow::Result;
use crate::error::MuxlyError;
use tracing::{info, warn};

/// Database pool type
pub type DatabasePool = Pool<Sqlite>;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum connections in the pool
    pub max_connections: u32,
    /// Minimum connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime_seconds: u64,
    /// Idle timeout in seconds
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout_seconds: 30,
            max_lifetime_seconds: 1800, // 30 minutes
            idle_timeout_seconds: 600,  // 10 minutes
        }
    }
}

/// Initialize the SQLite database with the given URL and configuration
pub async fn init_sqlite_database(config: &DatabaseConfig) -> Result<DatabasePool> {
    info!("Initializing SQLite database: {}", config.url);
    
    // Configure the connection pool
    let pool = SqlitePool::builder()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(config.connection_timeout_seconds))
        .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
        .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
        .connect(&config.url)
        .await
        .map_err(|e| MuxlyError::Database(e))?;
    
    info!("Database connection pool initialized successfully");
    Ok(pool)
}

/// Check if the database is available
pub async fn check_database(pool: &DatabasePool) -> Result<bool> {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => Ok(true),
        Err(e) => {
            warn!("Database health check failed: {}", e);
            Err(MuxlyError::Database(e).into())
        }
    }
}

/// Gracefully shutdown the database connection pool
pub async fn shutdown_database(pool: &DatabasePool) -> Result<()> {
    info!("Shutting down database connections gracefully...");
    
    // Close the pool - this will wait for queries to complete
    pool.close().await;
    
    info!("Database connections shut down successfully");
    Ok(())
} 