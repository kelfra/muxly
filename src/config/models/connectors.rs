//! Connectors configuration model
//!
//! Contains settings for configuring connector integrations such as
//! BigQuery, Google Analytics 4, HubSpot, and custom connectors.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Connectors configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorsConfig {
    /// Global connector settings
    #[serde(default)]
    pub global: GlobalConnectorConfig,
    
    /// BigQuery connector settings
    #[serde(default)]
    pub bigquery: BigQueryConfig,
    
    /// Google Analytics 4 connector settings
    #[serde(default)]
    pub ga4: GA4Config,
    
    /// HubSpot connector settings
    #[serde(default)]
    pub hubspot: HubSpotConfig,
    
    /// Custom connectors
    #[serde(default)]
    pub custom: HashMap<String, CustomConnectorConfig>,
}

impl Default for ConnectorsConfig {
    fn default() -> Self {
        Self {
            global: GlobalConnectorConfig::default(),
            bigquery: BigQueryConfig::default(),
            ga4: GA4Config::default(),
            hubspot: HubSpotConfig::default(),
            custom: HashMap::new(),
        }
    }
}

/// Global connector settings that apply to all connectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConnectorConfig {
    /// Default connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// Default maximum retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// Default backoff strategy for retries (linear, exponential)
    #[serde(default = "default_backoff_strategy")]
    pub backoff_strategy: String,
    
    /// Default concurrent sync limit
    #[serde(default = "default_concurrent_syncs")]
    pub concurrent_syncs: usize,
    
    /// Default connection check interval in seconds
    #[serde(default = "default_check_interval")]
    pub check_interval: u64,
}

impl Default for GlobalConnectorConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            backoff_strategy: default_backoff_strategy(),
            concurrent_syncs: default_concurrent_syncs(),
            check_interval: default_check_interval(),
        }
    }
}

/// BigQuery connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryConfig {
    /// Enable the BigQuery connector
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Project ID
    #[serde(default)]
    pub project_id: Option<String>,
    
    /// Service account JSON key file path
    #[serde(default)]
    pub service_account_key: Option<String>,
    
    /// Maximum rows per batch
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    
    /// Default location/region
    #[serde(default = "default_location")]
    pub location: String,
}

impl Default for BigQueryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            project_id: None,
            service_account_key: None,
            max_rows: default_max_rows(),
            location: default_location(),
        }
    }
}

/// Google Analytics 4 connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GA4Config {
    /// Enable the GA4 connector
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Property ID
    #[serde(default)]
    pub property_id: Option<String>,
    
    /// Service account JSON key file path
    #[serde(default)]
    pub service_account_key: Option<String>,
    
    /// OAuth client ID
    #[serde(default)]
    pub oauth_client_id: Option<String>,
    
    /// OAuth client secret
    #[serde(default)]
    pub oauth_client_secret: Option<String>,
    
    /// OAuth refresh token
    #[serde(default)]
    pub oauth_refresh_token: Option<String>,
    
    /// Default date range in days
    #[serde(default = "default_date_range")]
    pub default_date_range: u32,
}

impl Default for GA4Config {
    fn default() -> Self {
        Self {
            enabled: false,
            property_id: None,
            service_account_key: None,
            oauth_client_id: None,
            oauth_client_secret: None,
            oauth_refresh_token: None,
            default_date_range: default_date_range(),
        }
    }
}

/// HubSpot connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubSpotConfig {
    /// Enable the HubSpot connector
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// API key (legacy)
    #[serde(default)]
    pub api_key: Option<String>,
    
    /// OAuth access token
    #[serde(default)]
    pub access_token: Option<String>,
    
    /// OAuth refresh token
    #[serde(default)]
    pub refresh_token: Option<String>,
    
    /// OAuth client ID
    #[serde(default)]
    pub client_id: Option<String>,
    
    /// OAuth client secret
    #[serde(default)]
    pub client_secret: Option<String>,
    
    /// Default sync batch size
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    
    /// Enable webhook registration
    #[serde(default = "default_enable_webhooks")]
    pub enable_webhooks: bool,
}

impl Default for HubSpotConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            access_token: None,
            refresh_token: None,
            client_id: None,
            client_secret: None,
            batch_size: default_batch_size(),
            enable_webhooks: default_enable_webhooks(),
        }
    }
}

/// Custom connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConnectorConfig {
    /// Enable the custom connector
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Connector type (database, file)
    pub connector_type: String,
    
    /// Plugin file path
    #[serde(default)]
    pub plugin_path: Option<String>,
    
    /// Authentication type (none, basic, bearer, apikey, oauth)
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    
    /// Authentication credentials as a map
    #[serde(default)]
    pub auth_credentials: HashMap<String, String>,
    
    /// Custom configuration parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_backoff_strategy() -> String {
    "exponential".to_string()
}

fn default_concurrent_syncs() -> usize {
    5
}

fn default_check_interval() -> u64 {
    300 // 5 minutes
}

fn default_enabled() -> bool {
    false
}

fn default_max_rows() -> usize {
    10000
}

fn default_location() -> String {
    "US".to_string()
}

fn default_date_range() -> u32 {
    30
}

fn default_batch_size() -> usize {
    100
}

fn default_enable_webhooks() -> bool {
    true
}

fn default_auth_type() -> String {
    "none".to_string()
} 