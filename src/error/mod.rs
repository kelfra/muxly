use std::fmt;
use sqlx::Error as SqlxError;
use thiserror::Error;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;
use serde::{Serialize, Deserialize};

#[cfg(test)]
mod tests;

/// The core error type for the Muxly application.
/// This provides a unified error handling approach across all modules.
#[derive(Error, Debug)]
pub enum MuxlyError {
    #[error("Database error: {0}")]
    Database(#[from] SqlxError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Connector error: {0}")]
    Connector(String),

    #[error("Router error: {0}")]
    Router(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

/// Human-readable error type for API responses
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Details about the error
#[derive(Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl MuxlyError {
    /// Converts the error into an appropriate HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            MuxlyError::NotFound(_) => StatusCode::NOT_FOUND,
            MuxlyError::BadRequest(_) | MuxlyError::Validation(_) => StatusCode::BAD_REQUEST,
            MuxlyError::Authentication(_) => StatusCode::UNAUTHORIZED,
            MuxlyError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Converts the error to an error code string
    pub fn error_code(&self) -> String {
        match self {
            MuxlyError::Database(_) => "DATABASE_ERROR",
            MuxlyError::Configuration(_) => "CONFIGURATION_ERROR",
            MuxlyError::Connector(_) => "CONNECTOR_ERROR",
            MuxlyError::Router(_) => "ROUTER_ERROR",
            MuxlyError::Scheduler(_) => "SCHEDULER_ERROR",
            MuxlyError::Authentication(_) => "AUTHENTICATION_ERROR",
            MuxlyError::Validation(_) => "VALIDATION_ERROR",
            MuxlyError::NotFound(_) => "NOT_FOUND",
            MuxlyError::BadRequest(_) => "BAD_REQUEST",
            MuxlyError::Internal(_) => "INTERNAL_ERROR",
            MuxlyError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
        }.to_string()
    }
}

/// Custom result type to make error handling more concise
pub type Result<T> = std::result::Result<T, MuxlyError>;

// Implement From for common error types
impl From<anyhow::Error> for MuxlyError {
    fn from(err: anyhow::Error) -> Self {
        MuxlyError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for MuxlyError {
    fn from(err: serde_json::Error) -> Self {
        MuxlyError::BadRequest(format!("Invalid JSON: {}", err))
    }
}

impl From<std::io::Error> for MuxlyError {
    fn from(err: std::io::Error) -> Self {
        MuxlyError::Internal(format!("IO error: {}", err))
    }
}

impl From<reqwest::Error> for MuxlyError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            MuxlyError::ExternalService(format!("Request timeout: {}", err))
        } else if err.is_connect() {
            MuxlyError::ExternalService(format!("Connection error: {}", err))
        } else {
            MuxlyError::ExternalService(format!("HTTP client error: {}", err))
        }
    }
}

// Make errors usable with Axum's response system
impl IntoResponse for MuxlyError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: self.error_code(),
                message: self.to_string(),
                details: None,
            },
        };

        // Log server errors
        if status.is_server_error() {
            tracing::error!("{:?}", self);
        } else {
            tracing::debug!("{:?}", self);
        }

        // Build the error response
        (status, Json(error_response)).into_response()
    }
} 