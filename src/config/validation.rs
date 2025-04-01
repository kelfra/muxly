//! Configuration validation functionality
//!
//! This module provides functions for validating configuration values
//! and ensuring configuration consistency.

use anyhow::{Context, Result};
use tracing::{debug, warn};
use std::path::Path;

use super::Config;

/// Validate the entire configuration
pub fn validate_config(config: &Config) -> Result<()> {
    debug!("Validating configuration");
    
    // Validate app configuration
    validate_app_config(&config.app)?;
    
    // Validate connectors configuration
    validate_connectors_config(&config.connectors)?;
    
    // Validate router configuration
    validate_router_config(&config.router)?;
    
    // Validate scheduler configuration
    validate_scheduler_config(&config.scheduler)?;
    
    debug!("Configuration validation successful");
    Ok(())
}

/// Validate application configuration
fn validate_app_config(config: &super::models::app::AppConfig) -> Result<()> {
    debug!("Validating app configuration");
    
    // Validate port range
    if config.port == 0 {
        warn!("Port 0 might cause the OS to assign a random port");
    }
    
    // Validate log level
    match config.log_level.to_lowercase().as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => {}
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid log level: {}. Expected one of: trace, debug, info, warn, error",
                config.log_level
            ));
        }
    }
    
    // Validate database URL
    if config.database_url.is_empty() {
        return Err(anyhow::anyhow!("Database URL cannot be empty"));
    }
    
    // If auth is enabled, validate required fields
    if config.auth_enabled {
        if config.jwt_secret.is_none() && config.keycloak_url.is_none() {
            return Err(anyhow::anyhow!(
                "Authentication is enabled but neither JWT secret nor Keycloak URL is configured"
            ));
        }
        
        // If Keycloak is configured, validate required fields
        if let Some(url) = &config.keycloak_url {
            if url.is_empty() {
                return Err(anyhow::anyhow!("Keycloak URL cannot be empty"));
            }
            
            if config.keycloak_realm.is_none() || config.keycloak_realm.as_ref().unwrap().is_empty() {
                return Err(anyhow::anyhow!("Keycloak realm must be specified"));
            }
            
            if config.keycloak_client_id.is_none() || config.keycloak_client_id.as_ref().unwrap().is_empty() {
                return Err(anyhow::anyhow!("Keycloak client ID must be specified"));
            }
        }
    }
    
    Ok(())
}

/// Validate connectors configuration
fn validate_connectors_config(config: &super::models::connectors::ConnectorsConfig) -> Result<()> {
    debug!("Validating connectors configuration");
    
    // Validate global connector settings
    if config.global.timeout == 0 {
        return Err(anyhow::anyhow!("Connection timeout cannot be 0 seconds"));
    }
    
    // Validate backoff strategy
    match config.global.backoff_strategy.to_lowercase().as_str() {
        "linear" | "exponential" => {}
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid backoff strategy: {}. Expected 'linear' or 'exponential'",
                config.global.backoff_strategy
            ));
        }
    }
    
    // Validate BigQuery connector if enabled
    if config.bigquery.enabled {
        if config.bigquery.project_id.is_none() || config.bigquery.project_id.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("BigQuery project ID must be specified when BigQuery connector is enabled"));
        }
        
        if let Some(key_path) = &config.bigquery.service_account_key {
            if !Path::new(key_path).exists() {
                warn!("BigQuery service account key file does not exist: {}", key_path);
            }
        } else {
            warn!("BigQuery connector is enabled but no service account key is specified");
        }
    }
    
    // Validate GA4 connector if enabled
    if config.ga4.enabled {
        if config.ga4.property_id.is_none() || config.ga4.property_id.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("GA4 property ID must be specified when GA4 connector is enabled"));
        }
        
        // Check that either service account or OAuth is configured
        let has_service_account = config.ga4.service_account_key.is_some();
        let has_oauth = config.ga4.oauth_client_id.is_some() 
                        && config.ga4.oauth_client_secret.is_some()
                        && config.ga4.oauth_refresh_token.is_some();
        
        if !has_service_account && !has_oauth {
            return Err(anyhow::anyhow!(
                "GA4 connector is enabled but neither service account nor OAuth credentials are configured"
            ));
        }
    }
    
    // Validate HubSpot connector if enabled
    if config.hubspot.enabled {
        let has_api_key = config.hubspot.api_key.is_some();
        let has_oauth = config.hubspot.client_id.is_some() 
                        && config.hubspot.client_secret.is_some()
                        && config.hubspot.refresh_token.is_some();
                        
        if !has_api_key && !has_oauth && config.hubspot.access_token.is_none() {
            return Err(anyhow::anyhow!(
                "HubSpot connector is enabled but no authentication method is configured"
            ));
        }
    }
    
    // Validate custom connectors
    for (name, connector) in &config.custom {
        if connector.enabled {
            match connector.connector_type.to_lowercase().as_str() {
                "database" | "file" => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid connector type for custom connector '{}': {}. Expected 'database', or 'file'",
                        name, connector.connector_type
                    ));
                }
            }
            
            if connector.plugin_path.is_some() {
                let path = connector.plugin_path.as_ref().unwrap();
                if !Path::new(path).exists() {
                    warn!("Plugin file for custom connector '{}' does not exist: {}", name, path);
                }
            }
        }
    }
    
    Ok(())
}

/// Validate router configuration
fn validate_router_config(config: &super::models::router::RouterConfig) -> Result<()> {
    debug!("Validating router configuration");
    
    // Check global router settings
    if config.global.flush_interval == 0 {
        warn!("Router flush interval is set to 0, this may cause high CPU usage");
    }
    
    // Validate Prometheus output if enabled
    if config.prometheus.enabled {
        if !config.prometheus.metrics_path.starts_with('/') {
            return Err(anyhow::anyhow!("Prometheus metrics path must start with a '/'"));
        }
        
        match config.prometheus.default_metric_type.to_lowercase().as_str() {
            "counter" | "gauge" | "histogram" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid Prometheus metric type: {}. Expected 'counter', 'gauge', or 'histogram'",
                    config.prometheus.default_metric_type
                ));
            }
        }
    }
    
    // Validate webhook output if enabled
    if config.webhook.enabled {
        if config.webhook.endpoints.is_empty() {
            warn!("Webhook output is enabled but no endpoints are configured");
        }
        
        for (i, endpoint) in config.webhook.endpoints.iter().enumerate() {
            if endpoint.url.is_empty() {
                return Err(anyhow::anyhow!("Webhook endpoint URL cannot be empty (endpoint #{})", i + 1));
            }
            
            match endpoint.method.to_uppercase().as_str() {
                "POST" | "PUT" => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid HTTP method for webhook endpoint #{}: {}. Expected 'POST' or 'PUT'",
                        i + 1, endpoint.method
                    ));
                }
            }
        }
    }
    
    // Validate file output if enabled
    if config.file.enabled {
        if config.file.output_dir.is_empty() {
            return Err(anyhow::anyhow!("File output directory cannot be empty"));
        }
        
        match config.file.format.to_lowercase().as_str() {
            "json" | "csv" | "parquet" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid file format: {}. Expected 'json', 'csv', or 'parquet'",
                    config.file.format
                ));
            }
        }
        
        match config.file.rotation.to_lowercase().as_str() {
            "hourly" | "daily" | "weekly" | "monthly" | "none" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid file rotation: {}. Expected 'hourly', 'daily', 'weekly', 'monthly', or 'none'",
                    config.file.rotation
                ));
            }
        }
    }
    
    // Validate S3 output if enabled
    if config.s3.enabled {
        if config.s3.bucket.is_none() || config.s3.bucket.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("S3 bucket name must be specified when S3 output is enabled"));
        }
        
        if config.s3.region.is_none() || config.s3.region.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("AWS region must be specified when S3 output is enabled"));
        }
        
        let has_credentials = config.s3.access_key_id.is_some() && config.s3.secret_access_key.is_some();
        if !has_credentials && !config.s3.use_instance_profile {
            return Err(anyhow::anyhow!(
                "S3 output is enabled but neither AWS credentials nor instance profile is configured"
            ));
        }
    }
    
    // Validate database output if enabled
    if config.database.enabled {
        if config.database.connection_url.is_none() || config.database.connection_url.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("Database connection URL must be specified when database output is enabled"));
        }
        
        if config.database.table.is_none() || config.database.table.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("Target table must be specified when database output is enabled"));
        }
        
        match config.database.db_type.to_lowercase().as_str() {
            "postgres" | "mysql" | "sqlite" | "sqlserver" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid database type: {}. Expected 'postgres', 'mysql', 'sqlite', or 'sqlserver'",
                    config.database.db_type
                ));
            }
        }
    }
    
    // Validate Slack notifications if enabled
    if config.slack.enabled {
        if config.slack.webhook_url.is_none() || config.slack.webhook_url.as_ref().unwrap().is_empty() {
            return Err(anyhow::anyhow!("Slack webhook URL must be specified when Slack notifications are enabled"));
        }
    }
    
    // Validate custom outputs
    for (name, output) in &config.custom {
        if output.enabled {
            if output.plugin_path.is_some() {
                let path = output.plugin_path.as_ref().unwrap();
                if !Path::new(path).exists() {
                    warn!("Plugin file for custom output '{}' does not exist: {}", name, path);
                }
            }
        }
    }
    
    Ok(())
}

/// Validate scheduler configuration
fn validate_scheduler_config(config: &super::models::scheduler::SchedulerConfig) -> Result<()> {
    debug!("Validating scheduler configuration");
    
    // Validate API scheduler if enabled
    if config.api.enabled {
        if config.api.max_concurrent_jobs == 0 {
            return Err(anyhow::anyhow!("API scheduler max_concurrent_jobs cannot be 0"));
        }
        
        if config.api.job_timeout == 0 {
            warn!("API scheduler job_timeout is set to 0, jobs will never time out");
        }
    }
    
    // Validate cron scheduler if enabled
    if config.cron.enabled {
        if let Some(cron_expression) = &config.cron.cron_expression {
            if !is_valid_cron_expression(cron_expression) {
                return Err(anyhow::anyhow!(
                    "Invalid default cron expression: {}", cron_expression
                ));
            }
        }
        
        if let Some(timezone) = &config.cron.timezone {
            if !is_valid_timezone(timezone) {
                return Err(anyhow::anyhow!(
                    "Invalid timezone: {}", timezone
                ));
            }
        }
    }
    
    // Validate webhook scheduler if enabled
    if config.webhook.enabled {
        if config.webhook.validate_signatures && config.webhook.secret.is_none() {
            warn!("Webhook signature validation is enabled but no secret is configured");
        }
        
        if config.webhook.max_payload_size == 0 {
            return Err(anyhow::anyhow!("Webhook max_payload_size cannot be 0"));
        }
    }
    
    Ok(())
}

/// Check if a cron expression is valid
fn is_valid_cron_expression(expr: &str) -> bool {
    // This is a simplified validation - in a real implementation,
    // we would use the cron crate to parse and validate the expression
    
    // Check if expression has 5 or 6 fields (standard cron or with seconds)
    let fields: Vec<&str> = expr.split_whitespace().collect();
    !(fields.len() != 5 && fields.len() != 6)
}

/// Check if a timezone is valid
fn is_valid_timezone(timezone: &str) -> bool {
    // This is a simplified validation - in a real implementation,
    // we would use the chrono-tz crate to validate the timezone
    
    !timezone.is_empty()
}
