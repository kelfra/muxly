mod pipeline;
mod mapping;
mod normalization;
mod filtering;

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Transform pipeline for processing data
pub struct TransformPipeline {
    /// Settings for the transformation
    pub settings: TransformSettings,
}

/// Settings for the transformation pipeline
#[derive(Debug, Clone)]
pub struct TransformSettings {
    /// Field to use as the timestamp
    pub timestamp_field: String,
    /// Mappings from source fields to destination fields
    pub mappings: HashMap<String, String>,
    /// Whether to flatten nested objects
    pub flatten_nested: bool,
    /// Whether to remove null values
    pub remove_nulls: bool,
}

impl TransformPipeline {
    /// Create a new transform pipeline
    pub fn new(settings: TransformSettings) -> Self {
        Self { settings }
    }
    
    /// Process a data row through the transformation pipeline
    pub fn process(&self, data: Value) -> Result<Value> {
        // In a real implementation, this would apply the transformations
        // For now, just return the data as-is
        Ok(data)
    }
    
    /// Process a batch of data rows
    pub fn process_batch(&self, data: Vec<Value>) -> Result<Vec<Value>> {
        let mut result = Vec::with_capacity(data.len());
        
        for row in data {
            result.push(self.process(row)?);
        }
        
        Ok(result)
    }
}

pub use pipeline::*;
pub use mapping::*;
pub use normalization::*;
pub use filtering::*; 