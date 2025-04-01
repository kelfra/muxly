use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::collector::{Collector, CollectionContext, Metric};

/// Process a batch of metrics
pub async fn process_batch(
    collector: &Arc<Collector>,
    metrics: Vec<Metric>,
    source: &str,
) -> Result<String> {
    info!("Processing batch of {} metrics from {}", metrics.len(), source);
    
    // Create a collection context
    let context = CollectionContext {
        source: source.to_string(),
        timestamp: Utc::now(),
        collection_id: Uuid::new_v4().to_string(),
    };
    
    // Skip if no metrics
    if metrics.is_empty() {
        warn!("Empty batch of metrics, skipping");
        return Ok(context.collection_id);
    }
    
    // Convert to Value
    let metrics_value = serde_json::to_value(metrics)?;
    
    // Wrap in batch object
    let batch_value = json!({
        "source": source,
        "metrics": metrics_value,
        "metadata": {
            "batch_size": metrics_value.as_array().map(|a| a.len()).unwrap_or(0),
            "timestamp": Utc::now().to_rfc3339(),
        }
    });
    
    // Process the batch
    collector.collect(batch_value, context.clone()).await?;
    
    Ok(context.collection_id)
}

/// Convert metrics to a standard format
pub fn standardize_metric(metric: &Metric) -> Value {
    let mut standardized = json!({
        "name": metric.name,
        "value": metric.value,
    });
    
    // Add labels if present
    if let Some(labels) = &metric.labels {
        standardized["labels"] = labels.clone();
    }
    
    // Add timestamp if present, otherwise use current time
    let timestamp = match &metric.timestamp {
        Some(ts) => ts.clone(),
        None => Utc::now().to_rfc3339(),
    };
    standardized["timestamp"] = json!(timestamp);
    
    standardized
}

/// Extract numeric value from a metric
pub fn extract_numeric_value(metric: &Metric) -> Option<f64> {
    match &metric.value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
} 