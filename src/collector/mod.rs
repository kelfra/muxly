mod api;
mod processors;
mod validation;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::Router;
use tracing::{info, debug, warn, error};
use chrono::Utc;
use uuid::Uuid;
use std::sync::RwLock;

use crate::transform::TransformPipeline;
// use crate::router::Router;

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
    /// Processed metrics counter
    processed_count: Arc<RwLock<i64>>,
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
            processed_count: Arc::new(RwLock::new(0)),
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
        let result = self.process_data(data, context).await;
        
        // Decrement active collections counter
        {
            let mut count = self.active_collections.lock().await;
            *count -= 1;
        }
        
        result
    }
    
    /// Process data through the collector pipeline
    pub async fn process(&self, metric: Metric) -> Result<()> {
        // Validate and normalize the data
        let data = serde_json::to_value(&metric)?;
        
        // Create collection context
        let context = CollectionContext {
            source: "api".to_string(),
            timestamp: Utc::now(),
            collection_id: Uuid::new_v4().to_string(),
        };
        
        // Process the data
        self.collect(data, context).await?;
        
        Ok(())
    }
    
    /// Process data from a specific source
    async fn process_data(&self, data: Value, context: CollectionContext) -> Result<()> {
        // Increment counter
        {
            let mut counter = self.active_collections.lock().await;
            *counter += 1;
        }
        
        // Add context info to the data if it's an object
        let mut data_with_context = data.clone();
        if let Value::Object(ref mut obj) = data_with_context {
            // Add source and timestamp
            obj.insert("_source".into(), Value::String(context.source.clone()));
            obj.insert("_timestamp".into(), Value::String(Utc::now().to_rfc3339()));
            obj.insert("_collection_id".into(), Value::String(context.collection_id.clone()));
        }
        
        // Validate the data
        if !self.validate_data(&data_with_context) {
            return Err(anyhow!("Data validation failed"));
        }
        
        // Process the data with the transform pipeline
        let processed_data = self.transform.process(data_with_context)?;
        
        // Instead of trying to route via Router which is different in axum,
        // we just log the data for now
        debug!("Processed data: {}", processed_data);
        
        // Decrement counter
        {
            let mut counter = self.active_collections.lock().await;
            *counter -= 1;
        }
        
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
    
    /// Process data through the pipeline
    async fn process_data(&self, data: Value, context: CollectionContext) -> Result<()> {
        // Increment metrics counter
        self.increment_processed_count();
        
        // Enrich data with context information
        let enriched_data = self.add_context_to_data(data, &context);
        
        // Validate the data
        self.validate_data(&enriched_data)?;
        
        // Process the data through transformation pipeline
        let processed_data = self.transform.process(enriched_data)?;
        
        // Log the processed data
        info!("Processed data: {:?}", processed_data);
        
        // Forward the data to output processors
        // TODO: Implement output processing
        
        Ok(())
    }
    
    /// Add context information to the data
    fn add_context_to_data(&self, data: Value, context: &CollectionContext) -> Value {
        // Create a new object with context and data
        let mut obj = serde_json::Map::new();
        
        // Add context information
        let mut ctx = serde_json::Map::new();
        ctx.insert("source".to_string(), json!(context.source));
        ctx.insert("timestamp".to_string(), json!(context.timestamp.to_rfc3339()));
        ctx.insert("collection_id".to_string(), json!(context.collection_id));
        
        // Add to the object
        obj.insert("context".to_string(), Value::Object(ctx));
        obj.insert("data".to_string(), data);
        
        Value::Object(obj)
    }
    
    /// Validate the data structure
    fn validate_data(&self, data: &Value) -> Result<()> {
        // Ensure data is an object
        if !data.is_object() {
            return Err(anyhow!("Data must be an object"));
        }
        
        // Ensure it has context and data fields
        let obj = data.as_object().unwrap();
        if !obj.contains_key("context") || !obj.contains_key("data") {
            return Err(anyhow!("Data must contain 'context' and 'data' fields"));
        }
        
        // Validate context
        let context = &obj["context"];
        if !context.is_object() {
            return Err(anyhow!("Context must be an object"));
        }
        
        // Validate data
        let data = &obj["data"];
        if !data.is_object() && !data.is_array() {
            return Err(anyhow!("Data must be an object or array"));
        }
        
        Ok(())
    }
    
    /// Transform the data using the transformation pipeline
    fn transform_data(&self, data: Value) -> Result<Value> {
        // TODO: Implement transformation pipeline
        Ok(data)
    }
    
    /// Increment the active collections counter
    fn increment_active_collections(&self) {
        let mut counter = self.active_collections.write().unwrap();
        *counter += 1;
    }
    
    /// Decrement the active collections counter
    fn decrement_active_collections(&self) {
        let mut counter = self.active_collections.write().unwrap();
        *counter -= 1;
    }
    
    /// Increment the processed metrics counter
    fn increment_processed_count(&self) {
        let mut counter = self.processed_count.write().unwrap();
        *counter += 1;
    }
    
    /// Get the total number of processed metrics
    pub fn get_processed_count(&self) -> i64 {
        *self.processed_count.read().unwrap()
    }
}

pub use api::{api_routes, Metric};
pub use processors::process_batch; 