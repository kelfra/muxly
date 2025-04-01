use axum::{
    extract::{Path, Json as JsonExtractor},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

/// List all outputs
pub async fn list_outputs() -> (StatusCode, Json<Value>) {
    let outputs = json!([]);
    (StatusCode::OK, Json(outputs))
}

/// Create a new output
pub async fn create_output(
    JsonExtractor(payload): JsonExtractor<Value>
) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(json!({"id": "new-output", "status": "created"})))
}

/// Get a specific output by ID
pub async fn get_output(Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"id": id, "name": "Sample Output"})))
}

/// Update an output by ID
pub async fn update_output(
    Path(id): Path<String>,
    JsonExtractor(payload): JsonExtractor<Value>
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"id": id, "status": "updated"})))
}

/// Delete an output by ID
pub async fn delete_output(Path(id): Path<String>) -> StatusCode {
    StatusCode::NO_CONTENT
} 