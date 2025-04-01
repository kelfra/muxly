use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::collector::{Collector, CollectionContext};

/// Metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Name of the metric
    pub name: String,
    /// Value of the metric
    pub value: Value,
    /// Labels/dimensions associated with the metric
    pub labels: Option<Value>,
    /// Timestamp of the metric
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Batch of metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricBatch {
    /// Source of the metrics (application name)
    pub source: String,
    /// Metrics in the batch
    pub metrics: Vec<Metric>,
}

/// Response for a metric ingestion
#[derive(Debug, Serialize)]
pub struct MetricResponse {
    /// Status of the ingestion
    pub status: String,
    /// Collection ID
    pub collection_id: String,
    /// Number of metrics ingested
    pub count: usize,
}

/// Create API routes for the collector
pub fn api_routes(collector: Arc<Collector>) -> Router {
    Router::new()
        .route("/metrics", post(ingest_metrics))
        .route("/metrics/batch", post(ingest_metrics_batch))
        .route("/metrics/status", get(get_collector_status))
        .with_state(collector)
}

/// Handler for ingesting a single metric
async fn ingest_metrics(
    State(collector): State<Arc<Collector>>,
    Json(metric): Json<Metric>,
) -> Response {
    debug!("Ingesting metric: {}", metric.name);
    
    // Create a collection context
    let context = CollectionContext {
        source: "api".to_string(),
        timestamp: Utc::now(),
        collection_id: Uuid::new_v4().to_string(),
    };
    
    // Convert metric to JSON Value
    let data = match serde_json::to_value(&metric) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to serialize metric: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": format!("Invalid metric data: {}", e)
                })),
            )
                .into_response();
        }
    };
    
    // Collect the metric
    match collector.collect(data, context.clone()).await {
        Ok(_) => (
            StatusCode::ACCEPTED,
            Json(MetricResponse {
                status: "success".to_string(),
                collection_id: context.collection_id,
                count: 1,
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to collect metric: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to collect metric: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Handler for ingesting a batch of metrics
async fn ingest_metrics_batch(
    State(collector): State<Arc<Collector>>,
    Json(batch): Json<MetricBatch>,
) -> Response {
    info!("Ingesting batch of {} metrics from {}", batch.metrics.len(), batch.source);
    
    // Create a collection context
    let context = CollectionContext {
        source: batch.source,
        timestamp: Utc::now(),
        collection_id: Uuid::new_v4().to_string(),
    };
    
    // Convert batch to JSON Value
    let data = match serde_json::to_value(&batch.metrics) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to serialize metric batch: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": format!("Invalid metric batch data: {}", e)
                })),
            )
                .into_response();
        }
    };
    
    // Collect the metrics
    match collector.collect(data, context.clone()).await {
        Ok(_) => (
            StatusCode::ACCEPTED,
            Json(MetricResponse {
                status: "success".to_string(),
                collection_id: context.collection_id,
                count: batch.metrics.len(),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to collect metric batch: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to collect metric batch: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Handler for getting collector status
async fn get_collector_status(State(collector): State<Arc<Collector>>) -> Response {
    let active_collections = collector.get_active_collections().await;
    
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "active_collections": active_collections,
            "timestamp": Utc::now().to_rfc3339()
        })),
    )
        .into_response()
} 