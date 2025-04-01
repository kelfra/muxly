#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use axum::http::StatusCode;
    use serde_json::json;

    #[test]
    fn test_error_status_code() {
        // Test that errors map to the correct HTTP status codes
        let not_found = MuxlyError::NotFound("Resource not found".to_string());
        assert_eq!(not_found.status_code(), StatusCode::NOT_FOUND);

        let bad_request = MuxlyError::BadRequest("Bad request".to_string());
        assert_eq!(bad_request.status_code(), StatusCode::BAD_REQUEST);

        let auth_error = MuxlyError::Authentication("Invalid token".to_string());
        assert_eq!(auth_error.status_code(), StatusCode::UNAUTHORIZED);

        let internal = MuxlyError::Internal("Server error".to_string());
        assert_eq!(internal.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        let external = MuxlyError::ExternalService("Service unavailable".to_string());
        assert_eq!(external.status_code(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_error_code() {
        // Test that errors map to the correct error codes
        let database = MuxlyError::Database(sqlx::Error::PoolClosed);
        assert_eq!(database.error_code(), "DATABASE_ERROR");

        let config = MuxlyError::Configuration("Invalid config".to_string());
        assert_eq!(config.error_code(), "CONFIGURATION_ERROR");

        let connector = MuxlyError::Connector("Connection failed".to_string());
        assert_eq!(connector.error_code(), "CONNECTOR_ERROR");

        let router = MuxlyError::Router("Routing error".to_string());
        assert_eq!(router.error_code(), "ROUTER_ERROR");
    }

    #[test]
    fn test_from_conversions() {
        // Test From trait implementations
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let muxly_error: MuxlyError = io_error.into();
        assert!(matches!(muxly_error, MuxlyError::Internal(_)));

        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let muxly_error: MuxlyError = json_error.into();
        assert!(matches!(muxly_error, MuxlyError::BadRequest(_)));
    }

    #[tokio::test]
    async fn test_into_response() {
        // Test IntoResponse implementation
        let error = MuxlyError::NotFound("Resource not found".to_string());
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        // Convert response body to bytes
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        
        // Parse body as JSON
        let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        // Verify JSON structure
        assert_eq!(error_json["error"]["code"], "NOT_FOUND");
        assert!(error_json["error"]["message"].as_str().unwrap().contains("Resource not found"));
    }
} 