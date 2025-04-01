// Connectors Module - Export public interfaces

mod base;
mod bigquery;
mod ga4;
mod hubspot;
mod plugin;

// Re-export the base module components
pub use base::{
    Connector, 
    ConnectorData,
    ConnectorSettings,
    ConnectionStatus,
    AuthSettings,
    RateLimitSettings,
    RetrySettings,
    create_connector,
};

// Re-export specific connectors
pub use bigquery::BigQueryConnector;
pub use ga4::GA4Connector;
pub use hubspot::HubSpotConnector;
pub use plugin::PluginConnector; 