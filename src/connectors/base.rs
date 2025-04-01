use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use anyhow::Result;

/// Common settings that apply to all connectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorSettings {
    /// Unique identifier for the connector
    pub id: String,
    /// Human-readable name for the connector
    pub name: String,
    /// Type of connector (bigquery, ga4, hubspot, etc.)
    pub connector_type: String,
    /// Whether this connector is enabled
    pub enabled: bool,
    /// Authentication settings
    pub auth: AuthSettings,
    /// Connection settings specific to this connector type
    pub connection: HashMap<String, Value>,
    /// Optional rate limit settings
    pub rate_limit: Option<RateLimitSettings>,
    /// Optional retry settings
    pub retry: Option<RetrySettings>,
}

/// Authentication settings for connectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    /// Type of authentication (oauth, api_key, basic, etc.)
    pub auth_type: String,
    /// Authentication parameters
    pub params: HashMap<String, Value>,
}

/// Rate limit settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    /// Maximum number of requests per time period
    pub max_requests: u32,
    /// Time period in seconds
    pub period_seconds: u32,
    /// Whether to automatically adjust based on service responses
    pub auto_adjust: bool,
}

/// Retry settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySettings {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff time in milliseconds
    pub initial_backoff_ms: u32,
    /// Maximum backoff time in milliseconds
    pub max_backoff_ms: u32,
    /// Backoff multiplier
    pub backoff_multiplier: f32,
    /// Errors that should trigger a retry
    pub retryable_errors: Vec<String>,
}

/// Data response from a connector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorData {
    /// Source connector ID
    pub connector_id: String,
    /// Timestamp when data was fetched
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The actual data payload
    pub data: Value,
    /// Metadata about the data fetch
    pub metadata: HashMap<String, Value>,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
    CredentialsInvalid,
    RateLimited,
    Unknown,
}

/// Defines the interface that all connectors must implement
#[async_trait]
pub trait Connector {
    /// Returns the connector type
    fn get_type(&self) -> &str;
    
    /// Returns the connector ID
    fn get_id(&self) -> &str;
    
    /// Initialize the connector with settings
    async fn initialize(&mut self, settings: ConnectorSettings) -> Result<()>;
    
    /// Test the connection to the data source
    async fn test_connection(&self) -> Result<ConnectionStatus>;
    
    /// Fetch data from the data source
    async fn fetch_data(&self, params: Value) -> Result<ConnectorData>;
    
    /// Get metadata information about the connector's data structure
    async fn get_metadata(&self) -> Result<Value>;
    
    /// Get the connector's configuration template
    fn get_configuration_template(&self) -> Value;
}

/// Helper function to create a new connector instance based on connector type
pub fn create_connector(connector_type: &str) -> Result<Box<dyn Connector>> {
    match connector_type {
        "bigquery" => {
            let connector = crate::connectors::bigquery::BigQueryConnector::new();
            Ok(Box::new(connector))
        },
        "ga4" => {
            let connector = crate::connectors::ga4::GA4Connector::new();
            Ok(Box::new(connector))
        },
        "hubspot" => {
            let connector = crate::connectors::hubspot::HubSpotConnector::new();
            Ok(Box::new(connector))
        },
        "plugin" => {
            let connector = crate::connectors::plugin::PluginConnector::new();
            Ok(Box::new(connector))
        },
        _ => Err(anyhow::anyhow!("Unsupported connector type: {}", connector_type)),
    }
}
