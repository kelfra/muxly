//! Configuration models module
//!
//! This module contains all configuration model definitions for 
//! different components of the application.

pub mod app;
pub mod connectors;
pub mod router;
pub mod scheduler;

// Re-export all models for easier access
pub use app::AppConfig;
pub use connectors::{
    ConnectorsConfig, GlobalConnectorConfig, BigQueryConfig, 
    GA4Config, HubSpotConfig, CustomConnectorConfig,
};
pub use router::{
    RouterConfig, GlobalRouterConfig, PrometheusConfig, 
    WebhookOutputConfig, FileOutputConfig, S3OutputConfig,
    DatabaseOutputConfig, SlackConfig, CustomOutputConfig,
};
pub use scheduler::{
    SchedulerConfig, ApiSchedulerConfig, CronConfig, WebhookConfig,
}; 