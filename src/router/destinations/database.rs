use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, Postgres, query, query_builder::QueryBuilder};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;

use crate::router::Destination;

/// Configuration for the database destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDestinationConfig {
    /// Type of database (postgres, mysql, sqlite)
    pub db_type: String,
    /// Connection string
    pub connection_string: String,
    /// Table to insert into
    pub table: String,
    /// Schema to use (optional)
    pub schema: Option<String>,
    /// Key to use for upsert operations (optional)
    pub upsert_key: Option<String>,
    /// Batch size for inserts
    pub batch_size: usize,
    /// Whether to create the table if it doesn't exist
    pub create_if_not_exists: bool,
    /// Column mappings (from JSON key to DB column)
    pub column_mappings: Option<HashMap<String, String>>,
}

/// Destination that writes data to a database
pub struct DatabaseDestination {
    /// Unique identifier
    pub id: String,
    /// Configuration for the database destination
    pub config: DatabaseDestinationConfig,
    /// Database connection pool
    pool: Option<Pool<Postgres>>,
}

impl DatabaseDestination {
    /// Create a new database destination
    pub fn new(id: String, config: DatabaseDestinationConfig) -> Self {
        Self { 
            id, 
            config,
            pool: None,
        }
    }
    
    /// Initialize the database connection pool
    pub async fn initialize(&mut self) -> Result<()> {
        if self.config.db_type != "postgres" {
            return Err(anyhow::anyhow!("Only PostgreSQL is currently supported"));
        }
        
        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.config.connection_string)
            .await?;
        
        self.pool = Some(pool);
        
        // Create table if needed
        if self.config.create_if_not_exists {
            self.ensure_table_exists().await?;
        }
        
        Ok(())
    }
    
    /// Ensure the target table exists
    async fn ensure_table_exists(&self) -> Result<()> {
        if let Some(pool) = &self.pool {
            // This is a placeholder - in a real implementation, 
            // we would create the table based on the column mappings
            // Here, we just check if the table exists
            
            let schema = self.config.schema.as_deref().unwrap_or("public");
            let query = format!(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_schema = $1 
                    AND table_name = $2
                )",
            );
            
            let exists: (bool,) = sqlx::query_as(&query)
                .bind(schema)
                .bind(&self.config.table)
                .fetch_one(pool)
                .await?;
            
            if !exists.0 {
                // Table doesn't exist, create a minimal table
                // In a real implementation, this would be more sophisticated
                let create_query = format!(
                    "CREATE TABLE {}.{} (
                        id SERIAL PRIMARY KEY,
                        data JSONB NOT NULL,
                        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
                    )",
                    schema, self.config.table
                );
                
                sqlx::query(&create_query)
                    .execute(pool)
                    .await?;
            }
        }
        
        Ok(())
    }
    
    /// Insert a single record to the database
    async fn insert_record(&self, data: &Value) -> Result<()> {
        if let Some(pool) = &self.pool {
            let schema = self.config.schema.as_deref().unwrap_or("public");
            
            // Simple insert using JSONB
            let query = format!(
                "INSERT INTO {}.{} (data) VALUES ($1)",
                schema, self.config.table
            );
            
            sqlx::query(&query)
                .bind(data)
                .execute(pool)
                .await?;
        }
        
        Ok(())
    }
    
    /// Insert multiple records to the database in a batch
    async fn insert_batch(&self, data: &[Value]) -> Result<()> {
        if let Some(pool) = &self.pool {
            if data.is_empty() {
                return Ok(());
            }
            
            let schema = self.config.schema.as_deref().unwrap_or("public");
            
            // Build a batch insert query
            let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                format!("INSERT INTO {}.{} (data) ", schema, self.config.table)
            );
            
            query_builder.push_values(data, |mut b, item| {
                b.push_bind(item);
            });
            
            // Execute the query
            let query = query_builder.build();
            query.execute(pool).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl Destination for DatabaseDestination {
    fn get_type(&self) -> &str {
        "database"
    }
    
    fn get_id(&self) -> &str {
        &self.id
    }
    
    async fn send(&self, data: Value) -> Result<()> {
        self.insert_record(&data).await
    }
    
    async fn send_batch(&self, data: Vec<Value>) -> Result<()> {
        // Process in batches of the specified size
        for chunk in data.chunks(self.config.batch_size) {
            self.insert_batch(chunk).await?;
        }
        
        Ok(())
    }
    
    async fn check_availability(&self) -> Result<bool> {
        if let Some(pool) = &self.pool {
            // Try to execute a simple query to check connection
            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_) => Ok(true),
                Err(e) => {
                    tracing::error!("Database connection check failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }
} 