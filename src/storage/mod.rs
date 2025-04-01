mod database;
mod models;
mod migrations;
mod repositories;

use anyhow::Result;
use std::sync::Arc;

// Re-export database module for use elsewhere
pub use database::*;

/// Initialize database based on configuration
pub async fn init_database(database_url: &str) -> Result<Arc<database::DatabasePool>> {
    let pool = database::init_sqlite_database(database_url).await?;
    
    // Run migrations
    migrations::run_migrations(&pool).await?;
    
    Ok(Arc::new(pool))
}

// Optionally re-export model and repository types
pub use models::*;
pub use repositories::*; 