use serde_json::{Value, json};
use std::collections::HashMap;

/// Get a template configuration for a specific connector type
pub fn get_connector_template(connector_type: &str) -> Value {
    match connector_type {
        "bigquery" => bigquery_template(),
        "ga4" => ga4_template(),
        "hubspot" => hubspot_template(),
        "internal_api" => internal_api_template(),
        _ => json!({"error": "Unsupported connector type"}),
    }
}

/// BigQuery connector template
fn bigquery_template() -> Value {
    json!({
        "id": "bigquery_connector",
        "name": "BigQuery Connector",
        "connector_type": "bigquery",
        "enabled": true,
        "auth": {
            "auth_type": "service_account",
            "params": {
                "credentials_file": "", // Path to service account key file
                // Or use embedded JSON credentials:
                "credentials_json": ""  // Service account JSON as string
            }
        },
        "connection": {
            "project_id": "your-project-id",
            "location": "US",
            "timeout_seconds": 300
        },
        "query": {
            "sql": "SELECT * FROM `project.dataset.table` WHERE created_at >= @start_date AND created_at < @end_date LIMIT 1000",
            "parameters": {
                "start_date": "{{last_run_date}}",
                "end_date": "{{current_date}}"
            }
        },
        "schedule": {
            "schedule_type": "cron",
            "cron_expression": "0 */6 * * *",  // Every 6 hours
            "timezone": "UTC",
            "enabled": true
        },
        "transform": {
            "timestamp_field": "created_at",
            "mappings": {
                "id": "{{row.id}}",
                "timestamp": "{{row.created_at}}",
                "data": "{{row}}"
            },
            "flatten_nested": true,
            "remove_nulls": true
        },
        "comments": [
            "This template connects to BigQuery and runs a SQL query with date parameters.",
            "To use this template:",
            "1. Fill in your project_id",
            "2. Provide credentials via credentials_file or credentials_json",
            "3. Update the SQL query to match your data",
            "4. Adjust the schedule as needed"
        ]
    })
}

/// GA4 connector template
fn ga4_template() -> Value {
    json!({
        "id": "ga4_connector",
        "name": "Google Analytics 4 Connector",
        "connector_type": "ga4",
        "enabled": true,
        "auth": {
            "auth_type": "oauth",
            "params": {
                "client_id": "your-client-id.apps.googleusercontent.com",
                "client_secret": "your-client-secret",
                "access_token": "", // Optional if using refresh token
                "refresh_token": "", // Recommended for production use
                "token_uri": "https://oauth2.googleapis.com/token"
            }
        },
        "connection": {
            "property_id": "123456789", // Your GA4 property ID
            "start_date": "7daysAgo",   // Relative or YYYY-MM-DD
            "end_date": "yesterday",    // Relative or YYYY-MM-DD
            "metrics": ["activeUsers", "sessions", "screenPageViews"],
            "dimensions": ["date", "deviceCategory", "country"],
            "filters": "" // Optional filter expression
        },
        "schedule": {
            "schedule_type": "cron",
            "cron_expression": "0 1 * * *",  // Daily at 1 AM
            "timezone": "UTC",
            "enabled": true
        },
        "transform": {
            "timestamp_field": "date",
            "mappings": {
                "id": "ga4_{{row.dimensions.0}}",
                "timestamp": "{{row.dimensions.0}}",
                "metrics": "{{row.metrics}}",
                "dimensions": "{{row.dimensions}}"
            },
            "flatten_nested": false,
            "remove_nulls": true
        },
        "comments": [
            "This template fetches data from Google Analytics 4.",
            "To use this template:",
            "1. Create OAuth credentials in Google Cloud Console",
            "2. Update client_id and client_secret",
            "3. Complete the OAuth flow to get refresh_token",
            "4. Set your GA4 property_id",
            "5. Adjust metrics and dimensions as needed"
        ]
    })
}

/// HubSpot connector template
fn hubspot_template() -> Value {
    json!({
        "id": "hubspot_connector",
        "name": "HubSpot Connector",
        "connector_type": "hubspot",
        "enabled": true,
        "auth": {
            "auth_type": "api_key", // Or "oauth"
            "params": {
                // For API key authentication:
                "api_key": "your-api-key",
                
                // For OAuth authentication (alternative):
                "client_id": "",
                "client_secret": "",
                "access_token": "",
                "refresh_token": "",
                "token_uri": "https://api.hubapi.com/oauth/v1/token"
            }
        },
        "connection": {
            "base_url": "https://api.hubapi.com",
            "fetch_contacts": true,
            "fetch_companies": true,
            "fetch_deals": true,
            "start_date": "{{last_success - 7days}}",
            "limit": 100
        },
        "schedule": {
            "schedule_type": "cron",
            "cron_expression": "*/30 * * * *", // Every 30 minutes
            "timezone": "UTC",
            "enabled": true
        },
        "transform": {
            "timestamp_field": "updatedAt",
            "mappings": {
                "id": "hs_{{row.id}}",
                "timestamp": "{{row.updatedAt}}",
                "data": "{{row}}"
            },
            "flatten_nested": false,
            "remove_nulls": true
        },
        "comments": [
            "This template connects to HubSpot and syncs contacts, companies, and deals.",
            "To use this template:",
            "1. Choose authentication type (API key or OAuth)",
            "2. For API key: Add your HubSpot API key",
            "3. For OAuth: Complete OAuth flow to get tokens",
            "4. Adjust which objects to fetch (contacts, companies, deals)",
            "5. Configure the sync schedule as needed"
        ]
    })
}

/// Internal API connector template
fn internal_api_template() -> Value {
    json!({
        "id": "internal_api_connector",
        "name": "Internal API Connector",
        "connector_type": "internal_api",
        "enabled": true,
        "auth": {
            "auth_type": "bearer", // Or "basic", "api_key", "none"
            "params": {
                "token": "your-auth-token",
                // For basic auth:
                "username": "",
                "password": "",
                // For API key:
                "api_key": "",
                "api_key_header": "X-API-Key"
            }
        },
        "connection": {
            "base_url": "http://your-internal-api.example.com",
            "endpoint": "/api/metrics",
            "method": "GET",
            "timeout_seconds": 30,
            "headers": {
                "Content-Type": "application/json",
                "Accept": "application/json"
            },
            "query_params": {
                "from": "{{last_run_date}}",
                "to": "{{current_date}}"
            }
        },
        "schedule": {
            "schedule_type": "webhook",
            "webhook_path": "/trigger/internal_api",
            "enabled": true
        },
        "transform": {
            "timestamp_field": "timestamp",
            "mappings": {
                "id": "{{row.id}}",
                "timestamp": "{{row.timestamp}}",
                "data": "{{row}}"
            },
            "flatten_nested": true,
            "remove_nulls": true
        },
        "comments": [
            "This template connects to an internal API endpoint.",
            "To use this template:",
            "1. Update the base_url and endpoint to point to your API",
            "2. Configure authentication if needed",
            "3. Adjust headers and query parameters",
            "4. Modify the schedule or use webhook trigger",
            "5. Update the transformation mappings to match your data"
        ]
    })
}

/// Get a full template for a complete setup
pub fn get_full_setup_template(name: &str) -> Value {
    match name {
        "basic" => basic_setup_template(),
        "complete" => complete_setup_template(),
        _ => json!({"error": "Unsupported template name"}),
    }
}

/// Basic setup template with minimal configuration
fn basic_setup_template() -> Value {
    json!({
        "muxly": {
            "server": {
                "host": "0.0.0.0",
                "port": 3000
            },
            "database": {
                "type": "sqlite",
                "sqlite": {
                    "path": "./data/muxly.db"
                }
            },
            "connectors": [
                internal_api_template()
            ],
            "outputs": [
                {
                    "type": "file",
                    "enabled": true,
                    "config": {
                        "path": "./data/output",
                        "format": "json",
                        "filename_template": "{{connector_id}}_{{date}}.json"
                    }
                }
            ]
        }
    })
}

/// Complete setup template with all features configured
fn complete_setup_template() -> Value {
    json!({
        "muxly": {
            "server": {
                "host": "0.0.0.0",
                "port": 3000,
                "cors": {
                    "allowed_origins": ["*"],
                    "allowed_methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
                    "allowed_headers": ["*"]
                },
                "log_level": "info"
            },
            "database": {
                "type": "sqlite",
                "sqlite": {
                    "path": "./data/muxly.db",
                    "journal_mode": "WAL"
                },
                "auto_migrate": true
            },
            "connectors": [
                bigquery_template(),
                ga4_template(),
                hubspot_template(),
                internal_api_template()
            ],
            "outputs": [
                {
                    "type": "file",
                    "enabled": true,
                    "config": {
                        "path": "./data/output",
                        "format": "json",
                        "filename_template": "{{connector_id}}_{{date}}.json"
                    }
                },
                {
                    "type": "prometheus",
                    "enabled": true,
                    "config": {
                        "metrics_endpoint": "/metrics",
                        "metric_name_template": "muxly_{{connector_id}}_{{metric_name}}"
                    }
                },
                {
                    "type": "webhook",
                    "enabled": false,
                    "config": {
                        "url": "http://example.com/webhook",
                        "method": "POST",
                        "headers": {
                            "Content-Type": "application/json",
                            "Authorization": "Bearer {{webhook_token}}"
                        }
                    }
                }
            ]
        }
    })
}

/// Create a simple template file that can be saved to disk
pub fn create_template_file(template_type: &str, template_name: &str) -> std::io::Result<String> {
    let template = match template_type {
        "connector" => get_connector_template(template_name),
        "setup" => get_full_setup_template(template_name),
        _ => json!({"error": "Unsupported template type"}),
    };
    
    Ok(serde_json::to_string_pretty(&template)?)
}
