use axum::{
    routing::get,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Define routes
pub fn routes() -> Router {
    Router::new().route("/health", get(health_check))
}

// Health check response model
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
}

/// Get health status
/// 
/// Returns the current health status of the API
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Health check successful", body = HealthResponse)
    )
)]
pub async fn health_check() -> Json<HealthResponse> {
    // Get uptime (placeholder implementation)
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() % 86400; // Just for demonstration purposes
    
    // Return health status
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
    })
} 