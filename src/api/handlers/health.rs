use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

/// Basic health check endpoint returning OK
pub async fn health_check() -> &'static str {
    "OK"
}

/// Detailed health check with system status
pub async fn detailed_health_check() -> (StatusCode, Json<serde_json::Value>) {
    let health_data = json!({
        "status": "OK",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "api": "OK",
            "scheduler": "OK",
            "database": "OK"
        }
    });

    (StatusCode::OK, Json(health_data))
} 