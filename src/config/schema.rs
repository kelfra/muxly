//! Configuration schema module
//!
//! This module provides JSON Schema definitions for the configuration,
//! which can be used for validation and documentation.

use serde_json::Value;
use once_cell::sync::Lazy;

/// The full JSON Schema for the configuration
pub static CONFIG_SCHEMA: Lazy<Value> = Lazy::new(|| {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Muxly Configuration",
        "description": "Configuration for the Muxly data integration platform",
        "type": "object",
        "required": ["app"],
        "properties": {
            "app": APP_SCHEMA.clone(),
            "connectors": CONNECTORS_SCHEMA.clone(),
            "router": ROUTER_SCHEMA.clone(),
            "scheduler": SCHEDULER_SCHEMA.clone()
        }
    })
});

/// JSON Schema for application configuration
pub static APP_SCHEMA: Lazy<Value> = Lazy::new(|| {
    serde_json::json!({
        "type": "object",
        "title": "Application Configuration",
        "description": "General application settings",
        "required": ["name", "host", "port"],
        "properties": {
            "name": {
                "type": "string",
                "description": "Application name",
                "default": "Muxly"
            },
            "host": {
                "type": "string",
                "description": "Server host",
                "default": "127.0.0.1"
            },
            "port": {
                "type": "integer",
                "description": "Server port",
                "default": 3000,
                "minimum": 1,
                "maximum": 65535
            },
            "log_level": {
                "type": "string",
                "description": "Logging level",
                "enum": ["trace", "debug", "info", "warn", "error"],
                "default": "info"
            },
            "database_url": {
                "type": "string",
                "description": "Database connection URL",
                "default": "sqlite:muxly.db"
            },
            "max_connections": {
                "type": "integer",
                "description": "Maximum database connections",
                "default": 10,
                "minimum": 1
            },
            "connection_timeout": {
                "type": "integer",
                "description": "Connection timeout in seconds",
                "default": 30,
                "minimum": 1
            },
            "cors_enabled": {
                "type": "boolean",
                "description": "Enable CORS for API endpoints",
                "default": false
            },
            "cors_origins": {
                "type": "array",
                "description": "Allowed CORS origins",
                "items": {
                    "type": "string"
                },
                "default": ["http://localhost:3000"]
            },
            "auth_enabled": {
                "type": "boolean",
                "description": "Enable API authentication",
                "default": false
            },
            "jwt_secret": {
                "type": ["string", "null"],
                "description": "Secret for JWT token generation and validation"
            },
            "keycloak_url": {
                "type": ["string", "null"],
                "description": "Keycloak server URL for authentication"
            },
            "keycloak_realm": {
                "type": ["string", "null"],
                "description": "Keycloak realm"
            },
            "keycloak_client_id": {
                "type": ["string", "null"],
                "description": "Keycloak client ID"
            }
        }
    })
});

/// JSON Schema for connectors configuration
pub static CONNECTORS_SCHEMA: Lazy<Value> = Lazy::new(|| {
    serde_json::json!({
        "type": "object",
        "title": "Connectors Configuration",
        "description": "Configuration for data source connectors",
        "properties": {
            "global": {
                "type": "object",
                "description": "Global connector settings",
                "properties": {
                    "timeout": {
                        "type": "integer",
                        "description": "Default connection timeout in seconds",
                        "default": 30,
                        "minimum": 1
                    },
                    "max_retries": {
                        "type": "integer",
                        "description": "Default maximum retries for failed requests",
                        "default": 3,
                        "minimum": 0
                    },
                    "backoff_strategy": {
                        "type": "string",
                        "description": "Default backoff strategy for retries",
                        "enum": ["linear", "exponential"],
                        "default": "exponential"
                    },
                    "concurrent_syncs": {
                        "type": "integer",
                        "description": "Default concurrent sync limit",
                        "default": 5,
                        "minimum": 1
                    },
                    "check_interval": {
                        "type": "integer",
                        "description": "Default connection check interval in seconds",
                        "default": 300,
                        "minimum": 1
                    }
                }
            },
            "bigquery": {
                "type": "object",
                "description": "BigQuery connector configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable the BigQuery connector",
                        "default": false
                    },
                    "project_id": {
                        "type": ["string", "null"],
                        "description": "Google Cloud project ID"
                    },
                    "service_account_key": {
                        "type": ["string", "null"],
                        "description": "Path to the service account JSON key file"
                    },
                    "max_rows": {
                        "type": "integer",
                        "description": "Maximum rows per batch",
                        "default": 10000,
                        "minimum": 1
                    },
                    "location": {
                        "type": "string",
                        "description": "Default BigQuery location/region",
                        "default": "US"
                    }
                }
            },
            "ga4": {
                "type": "object",
                "description": "Google Analytics 4 connector configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable the GA4 connector",
                        "default": false
                    },
                    "property_id": {
                        "type": ["string", "null"],
                        "description": "GA4 property ID"
                    },
                    "service_account_key": {
                        "type": ["string", "null"],
                        "description": "Path to the service account JSON key file"
                    },
                    "oauth_client_id": {
                        "type": ["string", "null"],
                        "description": "OAuth client ID"
                    },
                    "oauth_client_secret": {
                        "type": ["string", "null"],
                        "description": "OAuth client secret"
                    },
                    "oauth_refresh_token": {
                        "type": ["string", "null"],
                        "description": "OAuth refresh token"
                    },
                    "default_date_range": {
                        "type": "integer",
                        "description": "Default date range in days",
                        "default": 30,
                        "minimum": 1
                    }
                }
            },
            "hubspot": {
                "type": "object",
                "description": "HubSpot connector configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable the HubSpot connector",
                        "default": false
                    },
                    "api_key": {
                        "type": ["string", "null"],
                        "description": "HubSpot API key (legacy)"
                    },
                    "access_token": {
                        "type": ["string", "null"],
                        "description": "OAuth access token"
                    },
                    "refresh_token": {
                        "type": ["string", "null"],
                        "description": "OAuth refresh token"
                    },
                    "client_id": {
                        "type": ["string", "null"],
                        "description": "OAuth client ID"
                    },
                    "client_secret": {
                        "type": ["string", "null"],
                        "description": "OAuth client secret"
                    },
                    "batch_size": {
                        "type": "integer",
                        "description": "Default sync batch size",
                        "default": 100,
                        "minimum": 1
                    },
                    "enable_webhooks": {
                        "type": "boolean",
                        "description": "Enable webhook registration",
                        "default": true
                    }
                }
            },
            "custom": {
                "type": "object",
                "description": "Custom connector configurations",
                "additionalProperties": {
                    "type": "object",
                    "properties": {
                        "enabled": {
                            "type": "boolean",
                            "description": "Enable the custom connector",
                            "default": false
                        },
                        "connector_type": {
                            "type": "string",
                            "description": "Type of connector",
                            "enum": ["api", "database", "file"]
                        },
                        "plugin_path": {
                            "type": ["string", "null"],
                            "description": "Path to the connector plugin file"
                        },
                        "api_url": {
                            "type": ["string", "null"],
                            "description": "API base URL for API-type connectors"
                        },
                        "auth_type": {
                            "type": "string",
                            "description": "Authentication type",
                            "enum": ["none", "basic", "bearer", "apikey", "oauth"],
                            "default": "none"
                        },
                        "auth_credentials": {
                            "type": "object",
                            "description": "Authentication credentials",
                            "additionalProperties": {
                                "type": "string"
                            }
                        },
                        "parameters": {
                            "type": "object",
                            "description": "Custom configuration parameters",
                            "additionalProperties": true
                        }
                    },
                    "required": ["connector_type"]
                }
            }
        }
    })
});

/// JSON Schema for router configuration
pub static ROUTER_SCHEMA: Lazy<Value> = Lazy::new(|| {
    serde_json::json!({
        "type": "object",
        "title": "Router Configuration",
        "description": "Configuration for data output routing",
        "properties": {
            "global": {
                "type": "object",
                "description": "Global router settings",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable routing system",
                        "default": true
                    },
                    "buffer_size": {
                        "type": "integer",
                        "description": "Buffer size for output queues",
                        "default": 1000,
                        "minimum": 1
                    },
                    "batch_size": {
                        "type": "integer",
                        "description": "Batch size for sending data",
                        "default": 100,
                        "minimum": 1
                    },
                    "flush_interval": {
                        "type": "integer",
                        "description": "Flush interval in seconds",
                        "default": 5,
                        "minimum": 0
                    },
                    "max_retries": {
                        "type": "integer",
                        "description": "Number of retries for failed deliveries",
                        "default": 3,
                        "minimum": 0
                    },
                    "enable_transformation": {
                        "type": "boolean",
                        "description": "Enable routing transformation",
                        "default": true
                    }
                }
            },
            "prometheus": {
                "type": "object",
                "description": "Prometheus metrics output configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable Prometheus output",
                        "default": false
                    },
                    "metrics_path": {
                        "type": "string",
                        "description": "Metrics path",
                        "default": "/metrics"
                    },
                    "include_labels": {
                        "type": "boolean",
                        "description": "Include labels in metrics",
                        "default": true
                    },
                    "default_metric_type": {
                        "type": "string",
                        "description": "Default metric type",
                        "enum": ["counter", "gauge", "histogram"],
                        "default": "gauge"
                    }
                }
            },
            "webhook": {
                "type": "object",
                "description": "Webhook output configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable webhook output",
                        "default": false
                    },
                    "endpoints": {
                        "type": "array",
                        "description": "Webhook endpoints",
                        "items": {
                            "type": "object",
                            "properties": {
                                "url": {
                                    "type": "string",
                                    "description": "Endpoint URL"
                                },
                                "secret": {
                                    "type": ["string", "null"],
                                    "description": "Secret for signing requests"
                                },
                                "method": {
                                    "type": "string",
                                    "description": "HTTP method",
                                    "enum": ["POST", "PUT"],
                                    "default": "POST"
                                },
                                "authorization": {
                                    "type": ["string", "null"],
                                    "description": "Authorization header"
                                },
                                "event_types": {
                                    "type": "array",
                                    "description": "Event types to send to this endpoint",
                                    "items": {
                                        "type": "string"
                                    }
                                }
                            },
                            "required": ["url"]
                        }
                    },
                    "max_concurrency": {
                        "type": "integer",
                        "description": "Maximum concurrency for webhook deliveries",
                        "default": 10,
                        "minimum": 1
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Connection timeout in seconds",
                        "default": 30,
                        "minimum": 1
                    }
                }
            },
            "file": {
                "type": "object",
                "description": "File output configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable file output",
                        "default": false
                    },
                    "output_dir": {
                        "type": "string",
                        "description": "Output directory",
                        "default": "./output"
                    },
                    "format": {
                        "type": "string",
                        "description": "File format",
                        "enum": ["json", "csv", "parquet"],
                        "default": "json"
                    },
                    "append": {
                        "type": "boolean",
                        "description": "Append to existing files",
                        "default": true
                    },
                    "rotation": {
                        "type": "string",
                        "description": "File rotation period",
                        "enum": ["hourly", "daily", "weekly", "monthly", "none"],
                        "default": "daily"
                    },
                    "max_file_size": {
                        "type": "integer",
                        "description": "Maximum file size in MB before rotation",
                        "default": 100,
                        "minimum": 1
                    }
                }
            },
            "s3": {
                "type": "object",
                "description": "S3 output configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable S3 output",
                        "default": false
                    },
                    "region": {
                        "type": ["string", "null"],
                        "description": "AWS region"
                    },
                    "bucket": {
                        "type": ["string", "null"],
                        "description": "S3 bucket name"
                    },
                    "prefix": {
                        "type": "string",
                        "description": "S3 path prefix",
                        "default": "muxly/"
                    },
                    "format": {
                        "type": "string",
                        "description": "File format",
                        "enum": ["json", "csv", "parquet"],
                        "default": "json"
                    },
                    "access_key_id": {
                        "type": ["string", "null"],
                        "description": "AWS access key ID"
                    },
                    "secret_access_key": {
                        "type": ["string", "null"],
                        "description": "AWS secret access key"
                    },
                    "use_instance_profile": {
                        "type": "boolean",
                        "description": "Use AWS instance profile",
                        "default": false
                    }
                }
            },
            "database": {
                "type": "object",
                "description": "Database output configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable database output",
                        "default": false
                    },
                    "connection_url": {
                        "type": ["string", "null"],
                        "description": "Database connection URL"
                    },
                    "db_type": {
                        "type": "string",
                        "description": "Database type",
                        "enum": ["postgres", "mysql", "sqlite", "sqlserver"],
                        "default": "postgres"
                    },
                    "table": {
                        "type": ["string", "null"],
                        "description": "Target table name"
                    },
                    "schema": {
                        "type": ["string", "null"],
                        "description": "Schema name"
                    },
                    "create_table": {
                        "type": "boolean",
                        "description": "Create table if not exists",
                        "default": false
                    },
                    "batch_size": {
                        "type": "integer",
                        "description": "Batch size for inserts",
                        "default": 100,
                        "minimum": 1
                    }
                }
            },
            "slack": {
                "type": "object",
                "description": "Slack notification configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable Slack notifications",
                        "default": false
                    },
                    "webhook_url": {
                        "type": ["string", "null"],
                        "description": "Webhook URL"
                    },
                    "channel": {
                        "type": ["string", "null"],
                        "description": "Default channel"
                    },
                    "username": {
                        "type": "string",
                        "description": "Bot username",
                        "default": "Muxly Bot"
                    },
                    "icon": {
                        "type": "string",
                        "description": "Icon emoji or URL",
                        "default": ":chart_with_upwards_trend:"
                    },
                    "event_types": {
                        "type": "array",
                        "description": "Event types to notify about",
                        "items": {
                            "type": "string"
                        }
                    }
                }
            },
            "custom": {
                "type": "object",
                "description": "Custom output configurations",
                "additionalProperties": {
                    "type": "object",
                    "properties": {
                        "enabled": {
                            "type": "boolean",
                            "description": "Enable the custom output",
                            "default": false
                        },
                        "output_type": {
                            "type": "string",
                            "description": "Output type"
                        },
                        "plugin_path": {
                            "type": ["string", "null"],
                            "description": "Path to the output plugin file"
                        },
                        "parameters": {
                            "type": "object",
                            "description": "Custom configuration parameters",
                            "additionalProperties": true
                        }
                    },
                    "required": ["output_type"]
                }
            }
        }
    })
});

/// JSON Schema for scheduler configuration
pub static SCHEDULER_SCHEMA: Lazy<Value> = Lazy::new(|| {
    serde_json::json!({
        "type": "object",
        "title": "Scheduler Configuration",
        "description": "Configuration for job scheduling",
        "properties": {
            "api": {
                "type": "object",
                "description": "API scheduler configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable API scheduler",
                        "default": true
                    },
                    "max_concurrent_jobs": {
                        "type": "integer",
                        "description": "Maximum concurrent jobs",
                        "default": 10,
                        "minimum": 1
                    },
                    "job_timeout": {
                        "type": "integer",
                        "description": "Job execution timeout in seconds",
                        "default": 300,
                        "minimum": 0
                    },
                    "max_history_size": {
                        "type": "integer",
                        "description": "Maximum execution history size per job",
                        "default": 100,
                        "minimum": 1
                    }
                }
            },
            "cron": {
                "type": "object",
                "description": "Cron scheduler configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable cron scheduler",
                        "default": true
                    },
                    "catch_up": {
                        "type": "boolean",
                        "description": "Default catch-up behavior for jobs",
                        "default": false
                    },
                    "cron_expression": {
                        "type": ["string", "null"],
                        "description": "Default cron expression"
                    },
                    "timezone": {
                        "type": ["string", "null"],
                        "description": "Default timezone for cron jobs"
                    }
                }
            },
            "webhook": {
                "type": "object",
                "description": "Webhook scheduler configuration",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable webhook scheduler",
                        "default": true
                    },
                    "secret": {
                        "type": ["string", "null"],
                        "description": "Secret for validating webhook requests"
                    },
                    "max_payload_size": {
                        "type": "integer",
                        "description": "Maximum payload size in bytes",
                        "default": 1048576,
                        "minimum": 1
                    },
                    "validate_signatures": {
                        "type": "boolean",
                        "description": "Whether to validate webhook signatures",
                        "default": true
                    }
                }
            }
        }
    })
});

/// Get the JSON Schema in string format
pub fn get_schema_string() -> String {
    serde_json::to_string_pretty(&*CONFIG_SCHEMA).unwrap()
} 