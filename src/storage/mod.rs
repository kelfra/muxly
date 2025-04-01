mod database;
mod models;
mod migrations;
mod repositories;

use anyhow::Result;
use sqlx::{Pool, Sqlite, SqlitePool, postgres::PgPool};
use std::sync::Arc;

/// Database connection type
#[derive(Debug, Clone)]
pub enum DatabaseConnection {
    Sqlite(Pool<Sqlite>),
    #[allow(dead_code)]
    Postgres(PgPool),
}

/// Database connection pool
#[derive(Debug, Clone)]
pub struct DatabasePool {
    pub connection: DatabaseConnection,
}

impl DatabasePool {
    /// Create a new SQLite database pool
    pub async fn new_sqlite(path: &str) -> Result<Self> {
        let pool = SqlitePool::connect(path).await?;
        
        Ok(Self {
            connection: DatabaseConnection::Sqlite(pool),
        })
    }
    
    /// Create a new PostgreSQL database pool
    #[allow(dead_code)]
    pub async fn new_postgres(connection_string: &str) -> Result<Self> {
        let pool = PgPool::connect(connection_string).await?;
        
        Ok(Self {
            connection: DatabaseConnection::Postgres(pool),
        })
    }
    
    /// Run migrations
    pub async fn run_migrations(&self) -> Result<()> {
        match &self.connection {
            DatabaseConnection::Sqlite(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            },
            DatabaseConnection::Postgres(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            },
        }
        
        Ok(())
    }
}

/// Initialize the database connection
pub async fn init_database(config: &crate::config::DatabaseConfig) -> Result<Arc<DatabasePool>> {
    let pool = match config.db_type.as_str() {
        "sqlite" => {
            let path = format!("sqlite:{}", config.sqlite_path);
            DatabasePool::new_sqlite(&path).await?
        },
        "postgres" => {
            let connection_string = format!(
                "postgres://{}:{}@{}:{}/{}",
                config.postgres_user,
                config.postgres_password,
                config.postgres_host,
                config.postgres_port,
                config.postgres_database
            );
            DatabasePool::new_postgres(&connection_string).await?
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported database type: {}", config.db_type));
        }
    };
    
    // Run migrations if configured to do so
    if config.auto_migrate {
        pool.run_migrations().await?;
    }
    
    Ok(Arc::new(pool))
}

pub use models::*;
pub use repositories::*; 