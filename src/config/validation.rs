use anyhow::{Result, anyhow};
use serde_json::Value;
use tracing::warn;
use crate::connectors::base::ConnectorSettings;

/// Validate a connector's configuration settings
pub fn validate_connector_settings(settings: &ConnectorSettings) -> Result<()> {
    // Validate common fields
    if settings.id.is_empty() {
        return Err(anyhow!("Connector ID cannot be empty"));
    }
    
    if settings.name.is_empty() {
        return Err(anyhow!("Connector name cannot be empty"));
    }
    
    // Validate connector-specific fields based on type
    match settings.connector_type.as_str() {
        "bigquery" => validate_bigquery_settings(settings),
        "ga4" => validate_ga4_settings(settings),
        "hubspot" => validate_hubspot_settings(settings),
        "internal_api" => validate_internal_api_settings(settings),
        _ => Err(anyhow!("Unsupported connector type: {}", settings.connector_type)),
    }
}

/// Validate BigQuery connector settings
fn validate_bigquery_settings(settings: &ConnectorSettings) -> Result<()> {
    // Validate authentication
    if settings.auth.auth_type != "service_account" {
        return Err(anyhow!("BigQuery connector requires service_account authentication"));
    }
    
    // Check for required connection parameters
    let project_id = settings.connection.get("project_id")
        .ok_or_else(|| anyhow!("Missing required field: project_id"))?;
    
    if project_id.as_str().map_or(true, |s| s.is_empty()) {
        return Err(anyhow!("project_id cannot be empty"));
    }
    
    // Validate credentials are present in authentication params
    if !settings.auth.params.contains_key("credentials_json") && 
       !settings.auth.params.contains_key("credentials_file") {
        return Err(anyhow!("BigQuery connector requires either credentials_json or credentials_file"));
    }
    
    Ok(())
}

/// Validate GA4 connector settings
fn validate_ga4_settings(settings: &ConnectorSettings) -> Result<()> {
    // Validate authentication
    if settings.auth.auth_type != "oauth" {
        return Err(anyhow!("GA4 connector requires oauth authentication"));
    }
    
    // Check for required connection parameters
    let property_id = settings.connection.get("property_id")
        .ok_or_else(|| anyhow!("Missing required field: property_id"))?;
    
    if property_id.as_str().map_or(true, |s| s.is_empty()) {
        return Err(anyhow!("property_id cannot be empty"));
    }
    
    // Check for metrics array
    if !settings.connection.contains_key("metrics") {
        return Err(anyhow!("Missing required field: metrics"));
    }
    
    // OAuth credentials checks
    if !settings.auth.params.contains_key("client_id") || 
       !settings.auth.params.contains_key("client_secret") {
        return Err(anyhow!("GA4 connector requires client_id and client_secret"));
    }
    
    Ok(())
}

/// Validate HubSpot connector settings
fn validate_hubspot_settings(settings: &ConnectorSettings) -> Result<()> {
    // Validate authentication
    if settings.auth.auth_type != "api_key" && settings.auth.auth_type != "oauth" {
        return Err(anyhow!("HubSpot connector requires api_key or oauth authentication"));
    }
    
    // Check authentication parameters
    match settings.auth.auth_type.as_str() {
        "api_key" => {
            if !settings.auth.params.contains_key("api_key") {
                return Err(anyhow!("HubSpot connector with api_key auth requires an api_key parameter"));
            }
        },
        "oauth" => {
            if !settings.auth.params.contains_key("access_token") || 
               !settings.auth.params.contains_key("refresh_token") || 
               !settings.auth.params.contains_key("client_id") || 
               !settings.auth.params.contains_key("client_secret") {
                return Err(anyhow!("HubSpot connector with oauth auth requires access_token, refresh_token, client_id, and client_secret parameters"));
            }
        },
        _ => {}
    }
    
    Ok(())
}

/// Validate Internal API connector settings
fn validate_internal_api_settings(settings: &ConnectorSettings) -> Result<()> {
    // Check for required connection parameters
    let base_url = settings.connection.get("base_url")
        .ok_or_else(|| anyhow!("Missing required field: base_url"))?;
    
    if base_url.as_str().map_or(true, |s| s.is_empty()) {
        return Err(anyhow!("base_url cannot be empty"));
    }
    
    // Validate URL format
    if let Some(url_str) = base_url.as_str() {
        if let Err(e) = url::Url::parse(url_str) {
            return Err(anyhow!("Invalid base_url: {}", e));
        }
    } else {
        return Err(anyhow!("base_url must be a string"));
    }
    
    Ok(())
}

/// Validate scheduler settings
pub fn validate_scheduler_settings(settings: &Value) -> Result<()> {
    // Extract schedule type
    let schedule_type = settings.get("schedule_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid schedule_type"))?;
    
    match schedule_type {
        "cron" => {
            // Validate cron expression
            let cron_expr = settings.get("cron_expression")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing or invalid cron_expression"))?;
            
            // Validate cron expression format
            if let Err(e) = cron::Schedule::from_str(cron_expr) {
                return Err(anyhow!("Invalid cron expression: {}", e));
            }
            
            // Check timezone (optional validation)
            if let Some(timezone) = settings.get("timezone").and_then(|v| v.as_str()) {
                if chrono_tz::Tz::from_str(timezone).is_err() {
                    warn!("Invalid timezone: {}, will use UTC instead", timezone);
                }
            }
        },
        "webhook" => {
            // Validate webhook path
            if let Some(path) = settings.get("webhook_path").and_then(|v| v.as_str()) {
                if !path.starts_with('/') {
                    return Err(anyhow!("webhook_path must start with '/'"));
                }
            } else {
                return Err(anyhow!("Missing or invalid webhook_path"));
            }
        },
        "api" => {
            // No specific validation needed for API triggers
        },
        _ => return Err(anyhow!("Unsupported schedule_type: {}", schedule_type)),
    }
    
    Ok(())
}

/// Validate transformation settings
pub fn validate_transformation_settings(settings: &Value) -> Result<()> {
    // Validate timestamp field
    if settings.get("timestamp_field").is_none() {
        return Err(anyhow!("Missing required field: timestamp_field"));
    }
    
    // Validate mappings
    let mappings = settings.get("mappings")
        .ok_or_else(|| anyhow!("Missing required field: mappings"))?;
    
    if !mappings.is_object() || mappings.as_object().unwrap().is_empty() {
        return Err(anyhow!("mappings must be a non-empty object"));
    }
    
    Ok(())
}

/// Validate output settings
pub fn validate_output_settings(settings: &Value) -> Result<()> {
    // Check that outputs is an array
    let outputs = settings.get("outputs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing or invalid outputs array"))?;
    
    if outputs.is_empty() {
        return Err(anyhow!("At least one output must be configured"));
    }
    
    // Validate each output
    for (idx, output) in outputs.iter().enumerate() {
        // Check output type
        let output_type = output.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Output #{} missing or invalid type", idx + 1))?;
        
        // Check config object
        let config = output.get("config")
            .ok_or_else(|| anyhow!("Output #{} ({}) missing config", idx + 1, output_type))?;
        
        if !config.is_object() {
            return Err(anyhow!("Output #{} ({}) config must be an object", idx + 1, output_type));
        }
        
        // Validate specific output types
        match output_type {
            "file" => validate_file_output(config, idx)?,
            "prometheus" => validate_prometheus_output(config, idx)?,
            "webhook" => validate_webhook_output(config, idx)?,
            _ => return Err(anyhow!("Unsupported output type: {}", output_type)),
        }
    }
    
    Ok(())
}

/// Validate file output settings
fn validate_file_output(config: &Value, idx: usize) -> Result<()> {
    // Check path
    let path = config.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("File output #{} missing or invalid path", idx + 1))?;
    
    if path.is_empty() {
        return Err(anyhow!("File output #{} path cannot be empty", idx + 1));
    }
    
    // Check format
    let format = config.get("format")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("File output #{} missing or invalid format", idx + 1))?;
    
    match format {
        "json" | "csv" | "jsonl" => {}, // Valid formats
        _ => return Err(anyhow!("File output #{} unsupported format: {}", idx + 1, format)),
    }
    
    Ok(())
}

/// Validate Prometheus output settings
fn validate_prometheus_output(config: &Value, idx: usize) -> Result<()> {
    // Check metrics endpoint
    let endpoint = config.get("metrics_endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Prometheus output #{} missing or invalid metrics_endpoint", idx + 1))?;
    
    if !endpoint.starts_with('/') {
        return Err(anyhow!("Prometheus output #{} metrics_endpoint must start with '/'", idx + 1));
    }
    
    Ok(())
}

/// Validate webhook output settings
fn validate_webhook_output(config: &Value, idx: usize) -> Result<()> {
    // Check URL
    let url = config.get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Webhook output #{} missing or invalid url", idx + 1))?;
    
    // Validate URL format
    if let Err(e) = url::Url::parse(url) {
        return Err(anyhow!("Webhook output #{} invalid url: {}", idx + 1, e));
    }
    
    // Check method
    let method = config.get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Webhook output #{} missing or invalid method", idx + 1))?;
    
    match method {
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" => {}, // Valid methods
        _ => return Err(anyhow!("Webhook output #{} unsupported method: {}", idx + 1, method)),
    }
    
    Ok(())
}

use std::str::FromStr;
