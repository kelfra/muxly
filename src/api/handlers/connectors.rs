use axum::{
    extract::{Path, State, Json as AxumJson},
    response::IntoResponse,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// List all connectors
pub async fn list_connectors() -> (StatusCode, Json<Value>) {
    let connectors = json!([]);
    (StatusCode::OK, Json(connectors))
}

/// Get a specific connector by ID
pub async fn get_connector(Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"id": id, "name": "Sample Connector"})))
}

#[derive(Deserialize)]
pub struct CreateConnectorRequest {
    name: String,
    connector_type: String,
    enabled: Option<bool>,
    auth: Value,
    connection: Value,
}

/// Create a new connector
pub async fn create_connector(
    AxumJson(payload): AxumJson<CreateConnectorRequest>
) -> (StatusCode, Json<Value>) {
    (StatusCode::CREATED, Json(json!({"id": "new-connector", "status": "created"})))
}

#[derive(Deserialize)]
pub struct UpdateConnectorRequest {
    name: Option<String>,
    enabled: Option<bool>,
    auth: Option<Value>,
    connection: Option<Value>,
}

/// Update an existing connector
pub async fn update_connector(
    Path(id): Path<String>,
    AxumJson(payload): AxumJson<UpdateConnectorRequest>
) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"id": id, "status": "updated"})))
}

/// Delete a connector
pub async fn delete_connector(Path(id): Path<String>) -> StatusCode {
    StatusCode::NO_CONTENT
}

/// Test a connector's connection
pub async fn test_connector(Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({"id": id, "connection": "successful"})))
}

/// Trigger a manual sync for a connector
pub async fn trigger_sync(Path(id): Path<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::ACCEPTED, Json(json!({"id": id, "sync_id": "sync-123", "status": "started"})))
} 