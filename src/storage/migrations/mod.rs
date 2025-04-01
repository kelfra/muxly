use anyhow::Context;
use sqlx::migrate::MigrateDatabase;
use crate::storage::database::DatabasePool;
use std::path::Path;
use tracing::{info, warn};
use crate::error::{MuxlyError, Result};

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<()> {
    info!("Running database migrations...");
    
    // Get the migrations directory path
    // Using SQLx standard migrations format (timestamp_name)
    let migration_path = Path::new("./migrations");
    
    if !migration_path.exists() {
        warn!("Migrations directory not found at {}", migration_path.display());
        return Ok(());
    }
    
    // Run migrations using SQLx
    sqlx::migrate::Migrator::new(migration_path)
        .await
        .map_err(|e| MuxlyError::Database(e))?
        .run(pool)
        .await
        .map_err(|e| MuxlyError::Database(e))?;
    
    info!("Database migrations completed successfully");
    Ok(())
} 