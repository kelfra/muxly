mod api;
mod processors;
mod validation;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::transform::TransformPipeline;
use crate::router::Router;

/// Collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Enabled collectors
    pub enabled: bool,
    /// Validation rules for incoming data
    pub validation_rules: Vec<ValidationRule>,
}

/// Validation rule for incoming data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Field to validate
    pub field: String,
    /// Type of validation (required, min, max, regex, etc.)
    pub rule_type: String,
    /// Value for the validation rule
    pub value: Option<Value>,
    /// Error message if validation fails
    pub error_message: Option<String>,
}

/// Collection context
#[derive(Debug, Clone)]
pub struct CollectionContext {
    /// Source of the data
    pub source: String,
    /// Timestamp of the collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Collection ID
    pub collection_id: String,
}

/// Collector for receiving metrics
pub struct Collector {
    /// Configuration for the collector
    config: CollectorConfig,
    /// Router for sending data to destinations
    router: Arc<Router>,
    /// Transform pipeline for processing data
    transform: Arc<TransformPipeline>,
    /// Active collection count for monitoring
    active_collections: Arc<Mutex<u64>>,
}

impl Collector {
    /// Create a new collector
    pub fn new(
        config: CollectorConfig,
        router: Arc<Router>,
        transform: Arc<TransformPipeline>,
    ) -> Self {
        Self {
            config,
            router,
            transform,
            active_collections: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Collect data from an internal source
    pub async fn collect(&self, data: Value, context: CollectionContext) -> Result<()> {
        // Skip if the collector is disabled
        if !self.config.enabled {
            return Ok(());
        }
        
        // Increment active collections counter
        {
            let mut count = self.active_collections.lock().await;
            *count += 1;
        }
        
        // Process the collected data
        let result = self.process_data(data, &context).await;
        
        // Decrement active collections counter
        {
            let mut count = self.active_collections.lock().await;
            *count -= 1;
        }
        
        result
    }
    
    /// Process collected data
    async fn process_data(&self, data: Value, context: &CollectionContext) -> Result<()> {
        // Validate the data
        if !self.validate_data(&data) {
            return Err(anyhow::anyhow!("Data validation failed"));
        }
        
        // Add context to the data
        let mut enriched_data = data;
        if let Value::Object(ref mut obj) = enriched_data {
            obj.insert("_source".into(), Value::String(context.source.clone()));
            obj.insert("_timestamp".into(), Value::String(context.timestamp.to_rfc3339()));
            obj.insert("_collection_id".into(), Value::String(context.collection_id.clone()));
        }
        
        // Transform the data
        let transformed_data = self.transform.process(enriched_data)?;
        
        // Route the data to destinations
        self.router.route(transformed_data).await?;
        
        Ok(())
    }
    
    /// Validate data against the collector's validation rules
    fn validate_data(&self, data: &Value) -> bool {
        validation::validate_data(data, &self.config.validation_rules)
    }
    
    /// Get the current number of active collections
    pub async fn get_active_collections(&self) -> u64 {
        *self.active_collections.lock().await
    }
}

pub use api::{api_routes, Metric};
pub use processors::process_batch; 