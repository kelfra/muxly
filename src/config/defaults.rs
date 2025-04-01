use std::collections::HashMap;
use serde_json::{Value, json};
use crate::connectors::base::{
    ConnectorSettings, AuthSettings, RateLimitSettings, RetrySettings
};

/// Create default settings for a connector based on its type
pub fn default_connector_settings(connector_type: &str, id: &str, name: &str) -> ConnectorSettings {
    let mut settings = ConnectorSettings {
        id: id.to_string(),
        name: name.to_string(),
        connector_type: connector_type.to_string(),
        enabled: true,
        auth: default_auth_settings(connector_type),
        connection: default_connection_settings(connector_type),
        rate_limit: Some(default_rate_limit_settings(connector_type)),
        retry: Some(default_retry_settings(connector_type)),
    };
    
    settings
}

/// Default authentication settings based on connector type
fn default_auth_settings(connector_type: &str) -> AuthSettings {
    let params = HashMap::new();
    
    match connector_type {
        "bigquery" => AuthSettings {
            auth_type: "service_account".to_string(),
            params,
        },
        "ga4" => AuthSettings {
            auth_type: "oauth".to_string(),
            params,
        },
        "hubspot" => AuthSettings {
            auth_type: "api_key".to_string(),
            params,
        },
        "internal_api" => AuthSettings {
            auth_type: "none".to_string(),
            params,
        },
        _ => AuthSettings {
            auth_type: "none".to_string(),
            params,
        },
    }
}

/// Default connection settings based on connector type
fn default_connection_settings(connector_type: &str) -> HashMap<String, Value> {
    let mut settings = HashMap::new();
    
    match connector_type {
        "bigquery" => {
            settings.insert("project_id".to_string(), json!(""));
            settings.insert("location".to_string(), json!("US"));
            settings.insert("timeout_seconds".to_string(), json!(300));
        },
        "ga4" => {
            settings.insert("property_id".to_string(), json!(""));
            settings.insert("start_date".to_string(), json!("7daysAgo"));
            settings.insert("end_date".to_string(), json!("yesterday"));
            settings.insert("metrics".to_string(), json!(["activeUsers", "sessions", "screenPageViews"]));
            settings.insert("dimensions".to_string(), json!(["date"]));
        },
        "hubspot" => {
            settings.insert("base_url".to_string(), json!("https://api.hubapi.com"));
            settings.insert("fetch_contacts".to_string(), json!(true));
            settings.insert("fetch_companies".to_string(), json!(true));
            settings.insert("fetch_deals".to_string(), json!(true));
        },
        "internal_api" => {
            settings.insert("base_url".to_string(), json!("http://localhost:8000"));
            settings.insert("timeout_seconds".to_string(), json!(30));
            settings.insert("headers".to_string(), json!({}));
        },
        _ => {}
    }
    
    settings
}

/// Default rate limit settings based on connector type
fn default_rate_limit_settings(connector_type: &str) -> RateLimitSettings {
    match connector_type {
        "bigquery" => RateLimitSettings {
            max_requests: 100,
            period_seconds: 60,
            auto_adjust: true,
        },
        "ga4" => RateLimitSettings {
            max_requests: 100,
            period_seconds: 60,
            auto_adjust: true,
        },
        "hubspot" => RateLimitSettings {
            max_requests: 100,
            period_seconds: 10,
            auto_adjust: true,
        },
        // Internal APIs usually don't need rate limiting
        "internal_api" => RateLimitSettings {
            max_requests: 1000,
            period_seconds: 60,
            auto_adjust: false,
        },
        // Default to conservative settings
        _ => RateLimitSettings {
            max_requests: 60,
            period_seconds: 60,
            auto_adjust: true,
        },
    }
}

/// Default retry settings based on connector type
fn default_retry_settings(connector_type: &str) -> RetrySettings {
    match connector_type {
        "bigquery" => RetrySettings {
            max_attempts: 5,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30000,
            backoff_multiplier: 2.0,
            retryable_errors: vec!["5xx".to_string(), "timeout".to_string(), "quota_exceeded".to_string()],
        },
        "ga4" => RetrySettings {
            max_attempts: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
            retryable_errors: vec!["5xx".to_string(), "timeout".to_string(), "rate_limit_exceeded".to_string()],
        },
        "hubspot" => RetrySettings {
            max_attempts: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            backoff_multiplier: 1.5,
            retryable_errors: vec!["5xx".to_string(), "timeout".to_string(), "rate_limit_exceeded".to_string()],
        },
        // Less aggressive retries for internal APIs
        "internal_api" => RetrySettings {
            max_attempts: 2,
            initial_backoff_ms: 500,
            max_backoff_ms: 2000,
            backoff_multiplier: 2.0,
            retryable_errors: vec!["5xx".to_string(), "timeout".to_string()],
        },
        // Default to conservative settings
        _ => RetrySettings {
            max_attempts: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
            retryable_errors: vec!["5xx".to_string(), "timeout".to_string()],
        },
    }
}

/// Default scheduler settings for different connector types
pub fn default_scheduler_settings(connector_type: &str) -> Value {
    match connector_type {
        "bigquery" => json!({
            "schedule_type": "cron",
            "cron_expression": "0 */6 * * *",  // Every 6 hours
            "timezone": "UTC",
            "enabled": true,
        }),
        "ga4" => json!({
            "schedule_type": "cron",
            "cron_expression": "0 1 * * *",  // Once per day at 1 AM
            "timezone": "UTC",
            "enabled": true,
        }),
        "hubspot" => json!({
            "schedule_type": "cron",
            "cron_expression": "*/30 * * * *",  // Every 30 minutes
            "timezone": "UTC",
            "enabled": true,
        }),
        "internal_api" => json!({
            "schedule_type": "webhook",
            "webhook_path": "/trigger/internal_api",
            "enabled": true,
        }),
        _ => json!({
            "schedule_type": "cron",
            "cron_expression": "0 */3 * * *",  // Every 3 hours
            "timezone": "UTC",
            "enabled": true,
        }),
    }
}

/// Default transformation settings for different connector types
pub fn default_transformation_settings(connector_type: &str) -> Value {
    match connector_type {
        "bigquery" => json!({
            "timestamp_field": "created_at",
            "mappings": {
                "id": "{{row.id}}",
                "timestamp": "{{row.created_at}}",
                "data": "{{row}}"
            },
            "flatten_nested": true,
            "remove_nulls": true
        }),
        "ga4" => json!({
            "timestamp_field": "date",
            "mappings": {
                "id": "ga4_{{row.dimensions.0}}",
                "timestamp": "{{row.dimensions.0}}",
                "metrics": "{{row.metrics}}",
                "dimensions": "{{row.dimensions}}"
            },
            "flatten_nested": false,
            "remove_nulls": true
        }),
        "hubspot" => json!({
            "timestamp_field": "updatedAt",
            "mappings": {
                "id": "hs_{{row.id}}",
                "timestamp": "{{row.updatedAt}}",
                "data": "{{row}}"
            },
            "flatten_nested": false,
            "remove_nulls": true
        }),
        "internal_api" => json!({
            "timestamp_field": "timestamp",
            "mappings": {
                "id": "{{row.id}}",
                "timestamp": "{{row.timestamp}}",
                "data": "{{row}}"
            },
            "flatten_nested": true,
            "remove_nulls": true
        }),
        _ => json!({
            "timestamp_field": "timestamp",
            "mappings": {
                "id": "{{row.id}}",
                "timestamp": "{{row.timestamp}}",
                "data": "{{row}}"
            },
            "flatten_nested": true,
            "remove_nulls": true
        }),
    }
}

/// Default output/router settings
pub fn default_output_settings() -> Value {
    json!({
        "outputs": [
            {
                "type": "file",
                "enabled": true,
                "config": {
                    "path": "./data/output",
                    "format": "json",
                    "filename_template": "{{connector_id}}_{{date}}.json",
                    "max_file_size_mb": 10,
                    "rotate_files": true
                }
            },
            {
                "type": "prometheus",
                "enabled": false,
                "config": {
                    "metrics_endpoint": "/metrics",
                    "metric_name_template": "muxly_{{connector_id}}_{{metric_name}}",
                    "include_labels": true
                }
            },
            {
                "type": "webhook",
                "enabled": false,
                "config": {
                    "url": "http://localhost:8080/webhook",
                    "method": "POST",
                    "headers": {},
                    "batch_size": 100,
                    "timeout_seconds": 30
                }
            }
        ]
    })
}

/// Default server settings
pub fn default_server_settings() -> Value {
    json!({
        "host": "0.0.0.0",
        "port": 3000,
        "cors": {
            "allowed_origins": ["*"],
            "allowed_methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
            "allowed_headers": ["*"],
            "max_age_seconds": 86400
        },
        "request_timeout_seconds": 30,
        "max_json_payload_size": "5MB",
        "enable_compression": true,
        "log_level": "info"
    })
}

/// Default database settings
pub fn default_database_settings() -> Value {
    json!({
        "type": "sqlite",
        "sqlite": {
            "path": "./data/muxly.db",
            "journal_mode": "WAL",
            "synchronous": "NORMAL",
            "cache_size": 2000,
            "foreign_keys": true
        },
        "postgres": {
            "host": "localhost",
            "port": 5432,
            "username": "muxly",
            "password": "",
            "database": "muxly",
            "ssl": false,
            "max_connections": 5
        },
        "migrations_path": "./migrations",
        "auto_migrate": true
    })
}
