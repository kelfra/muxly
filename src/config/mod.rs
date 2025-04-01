//! Configuration management for the Muxly application
//!
//! This module handles loading, validating, and accessing application configuration
//! from various sources including files, environment variables, and defaults.

pub mod loader;
pub mod schema;
pub mod validation;
pub mod models;

use std::path::Path;
use std::sync::Arc;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use self::models::app::AppConfig;
use self::models::connectors::ConnectorsConfig;
use self::models::router::RouterConfig;
use self::models::scheduler::SchedulerConfig;

/// Main configuration container for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General application settings
    pub app: AppConfig,
    
    /// Connectors configuration
    pub connectors: ConnectorsConfig,
    
    /// Router configuration
    pub router: RouterConfig,
    
    /// Scheduler configuration
    pub scheduler: SchedulerConfig,
}

impl Config {
    /// Creates a new configuration with default values
    pub fn default() -> Self {
        Self {
            app: AppConfig::default(),
            connectors: ConnectorsConfig::default(),
            router: RouterConfig::default(),
            scheduler: SchedulerConfig::default(),
        }
    }
    
    /// Loads configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        loader::load_from_file(path)
            .context("Failed to load configuration from file")
    }
    
    /// Merges environment variables into the configuration
    pub fn merge_environment(&mut self) -> Result<()> {
        loader::merge_environment(self)
            .context("Failed to merge environment variables into configuration")
    }
    
    /// Validates the configuration
    pub fn validate(&self) -> Result<()> {
        validation::validate_config(self)
            .context("Configuration validation failed")
    }
}

/// Shared configuration that can be safely passed between components
pub type SharedConfig = Arc<Config>;

/// Loads and initializes the application configuration
pub async fn init_config<P: AsRef<Path>>(config_path: Option<P>) -> Result<SharedConfig> {
    // Start with default configuration
    let mut config = Config::default();
    
    // Load from file if specified
    if let Some(path) = config_path {
        info!("Loading configuration from file");
        config = Config::from_file(path)?;
    }
    
    // Merge environment variables
    debug!("Merging environment variables");
    config.merge_environment()?;
    
    // Validate the configuration
    debug!("Validating configuration");
    config.validate()?;
    
    info!("Configuration initialized successfully");
    Ok(Arc::new(config))
} 