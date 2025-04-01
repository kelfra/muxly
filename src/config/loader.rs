//! Configuration loading functionality
//!
//! This module provides functions for loading configuration from files
//! and merging configuration from environment variables.

use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use tracing::{debug, warn};

use super::Config;

/// Load configuration from a file
pub fn load_from_file<T, P>(path: P) -> Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    debug!("Loading configuration from {}", path.display());
    
    // Read the file content
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    
    // Parse based on file extension
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON config: {}", path.display()))
        }
        Some("yaml") | Some("yml") => {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML config: {}", path.display()))
        }
        Some("toml") => {
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config: {}", path.display()))
        }
        _ => {
            warn!("Unknown config file extension for {}, attempting YAML parse", path.display());
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse config as YAML: {}", path.display()))
        }
    }
}

/// Merge environment variables into configuration
pub fn merge_environment(config: &mut Config) -> Result<()> {
    // This implementation will be enhanced using the envy crate
    // to auto-map environment variables to struct fields
    
    // For now, let's manually handle a few key environment variables
    if let Ok(port) = std::env::var("MUXLY_PORT") {
        if let Ok(port) = port.parse::<u16>() {
            debug!("Overriding port from environment: {}", port);
            config.app.port = port;
        } else {
            warn!("Invalid port value in MUXLY_PORT environment variable");
        }
    }
    
    if let Ok(log_level) = std::env::var("MUXLY_LOG_LEVEL") {
        debug!("Overriding log level from environment: {}", log_level);
        config.app.log_level = log_level;
    }
    
    if let Ok(database_url) = std::env::var("MUXLY_DATABASE_URL") {
        debug!("Overriding database URL from environment");
        config.app.database_url = database_url;
    }
    
    Ok(())
} 