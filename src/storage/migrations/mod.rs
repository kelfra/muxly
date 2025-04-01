use anyhow::{Result, Context};
use sqlx::migrate::MigrateDatabase;
use crate::storage::database::DatabasePool;
use std::path::Path;
use tracing::{info, warn};

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<()> {
    info!("Running database migrations...");
    
    // Get the migrations directory path
    let migration_path = Path::new("./migrations");
    
    if !migration_path.exists() {
        warn!("Migrations directory not found at {}", migration_path.display());
        return Ok(());
    }
    
    // Run migrations using SQLx
    sqlx::migrate::Migrator::new(migration_path)
        .await
        .context("Failed to create migrator")?
        .run(pool)
        .await
        .context("Failed to run migrations")?;
    
    info!("Database migrations completed successfully");
    Ok(())
} 