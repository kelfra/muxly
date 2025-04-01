use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

/// Database model for a connector
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Connector {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Type of connector (bigquery, ga4, hubspot, etc.)
    pub connector_type: String,
    /// Whether the connector is enabled
    pub enabled: bool,
    /// Authentication settings as JSON
    pub auth_settings: String,
    /// Connection settings as JSON
    pub connection_settings: String,
    /// When the connector was created
    pub created_at: DateTime<Utc>,
    /// When the connector was last updated
    pub updated_at: DateTime<Utc>,
}

impl Connector {
    /// Create a new connector
    pub fn new(
        name: String,
        connector_type: String,
        enabled: bool,
        auth_settings: Value,
        connection_settings: Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            connector_type,
            enabled,
            auth_settings: auth_settings.to_string(),
            connection_settings: connection_settings.to_string(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get authentication settings as JSON
    pub fn auth_settings_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.auth_settings)
    }
    
    /// Get connection settings as JSON
    pub fn connection_settings_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.connection_settings)
    }
} 