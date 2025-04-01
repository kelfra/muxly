use anyhow::{Result, anyhow};
use serde_json::Value;
use std::sync::Arc;

use crate::router::{
    Destination, DestinationSettings,
    DatabaseDestination, EmailDestination, FileDestination, 
    PrometheusDestination, SlackDestination, S3Destination, WebhookDestination
};

/// Factory for creating destination instances from configuration
pub struct DestinationFactory;

impl DestinationFactory {
    /// Create a new destination based on the provided settings
    pub fn create_destination(settings: &DestinationSettings) -> Result<Arc<dyn Destination>> {
        match settings.destination_type.as_str() {
            "database" => {
                let config: crate::router::destinations::database::DatabaseDestinationConfig = 
                    serde_json::from_value(settings.config.clone())?;
                Ok(Arc::new(DatabaseDestination::new(
                    format!("db_{}", uuid::Uuid::new_v4()),
                    config,
                )))
            },
            "email" => {
                let config: crate::router::destinations::email::EmailDestinationConfig = 
                    serde_json::from_value(settings.config.clone())?;
                Ok(Arc::new(EmailDestination::new(
                    format!("email_{}", uuid::Uuid::new_v4()),
                    config,
                )))
            },
            "file" => {
                let config: crate::router::destinations::file::FileDestinationConfig = 
                    serde_json::from_value(settings.config.clone())?;
                Ok(Arc::new(FileDestination::new(
                    format!("file_{}", uuid::Uuid::new_v4()),
                    config,
                )))
            },
            "prometheus" => {
                let config: crate::router::destinations::prometheus::PrometheusDestinationConfig = 
                    serde_json::from_value(settings.config.clone())?;
                Ok(Arc::new(PrometheusDestination::new(
                    format!("prometheus_{}", uuid::Uuid::new_v4()),
                    config,
                )))
            },
            "slack" => {
                let config: crate::router::destinations::slack::SlackDestinationConfig = 
                    serde_json::from_value(settings.config.clone())?;
                Ok(Arc::new(SlackDestination::new(
                    format!("slack_{}", uuid::Uuid::new_v4()),
                    config,
                )))
            },
            "s3" => {
                let config: crate::router::destinations::storage::S3DestinationConfig = 
                    serde_json::from_value(settings.config.clone())?;
                Ok(Arc::new(S3Destination::new(
                    format!("s3_{}", uuid::Uuid::new_v4()),
                    config,
                )))
            },
            "webhook" => {
                let config: crate::router::destinations::webhook::WebhookDestinationConfig = 
                    serde_json::from_value(settings.config.clone())?;
                Ok(Arc::new(WebhookDestination::new(
                    format!("webhook_{}", uuid::Uuid::new_v4()),
                    config,
                )))
            },
            _ => Err(anyhow!("Unsupported destination type: {}", settings.destination_type)),
        }
    }
    
    /// Create multiple destinations from a list of settings
    pub fn create_destinations(settings: &[DestinationSettings]) -> Result<Vec<Arc<dyn Destination>>> {
        let mut destinations = Vec::new();
        
        for setting in settings {
            destinations.push(Self::create_destination(setting)?);
        }
        
        Ok(destinations)
    }
} 