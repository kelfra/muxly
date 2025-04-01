mod database;
mod models;
mod migrations;
mod repositories;

use std::sync::Arc;

// Re-export database module for use elsewhere
pub use database::*;

/// Initialize database based on configuration
pub async fn init_database(config: &DatabaseConfig) -> crate::error::Result<Arc<DatabasePool>> {
    let pool = database::init_sqlite_database(config)
        .await
        .map_err(|e| crate::error::MuxlyError::Database(e.into()))?;
    
    // Run migrations
    migrations::run_migrations(&pool).await?;
    
    Ok(Arc::new(pool))
}

/// Gracefully shutdown the database
pub async fn shutdown() -> crate::error::Result<()> {
    // Shutdown operations can be added here as needed
    Ok(())
}

// Re-export model and repository types
pub use models::*;
pub use repositories::*; 