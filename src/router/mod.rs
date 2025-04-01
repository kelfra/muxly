mod destinations;
mod destination_factory;
mod route;
mod router_factory;
mod routing;

use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Router for sending data to destinations
pub struct Router {
    /// List of destinations to route data to
    pub destinations: Vec<Arc<dyn Destination>>,
}

/// Trait for defining a destination
#[async_trait::async_trait]
pub trait Destination: Send + Sync {
    /// Get the destination type
    fn get_type(&self) -> &str;
    
    /// Get the destination ID
    fn get_id(&self) -> &str;
    
    /// Send data to the destination
    async fn send(&self, data: Value) -> Result<()>;
    
    /// Send a batch of data to the destination
    async fn send_batch(&self, data: Vec<Value>) -> Result<()>;
    
    /// Check if the destination is available
    async fn check_availability(&self) -> Result<bool>;
}

/// Router data structure for storing routing settings and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterData {
    /// Unique identifier for the router
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether this router is enabled
    pub enabled: bool,
    /// Source connector configuration
    pub source: SourceSettings,
    /// Transformations to apply
    pub transformations: Vec<TransformationSettings>,
    /// Destinations to send data to
    pub destinations: Vec<DestinationSettings>,
    /// Condition for routing (optional)
    pub condition: Option<String>,
    /// Error handling settings
    pub error_handling: Option<ErrorHandlingSettings>,
}

/// Source settings for a router
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSettings {
    /// ID of the connector to use
    pub connector_id: String,
    /// Data specification for the connector
    pub data_spec: Value,
}

/// Transformation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationSettings {
    /// Type of transformation
    pub transformation_type: String,
    /// Parameters for the transformation
    pub params: Value,
}

/// Destination settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationSettings {
    /// Type of destination
    pub destination_type: String,
    /// Configuration for the destination
    pub config: Value,
}

/// Error handling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingSettings {
    /// What to do on error: continue or fail
    pub on_error: String,
    /// Optional destination for error data
    pub error_destination: Option<DestinationSettings>,
}

/// Status of a routing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStatus {
    /// Successfully routed data
    Success,
    /// Failed to route data
    Failure(String),
    /// Partially successful (some destinations failed)
    PartialSuccess(Vec<String>),
}

impl Router {
    /// Create a new router
    pub fn new(destinations: Vec<Arc<dyn Destination>>) -> Self {
        Self { destinations }
    }
    
    /// Route data to all enabled destinations
    pub async fn route(&self, data: Value) -> Result<()> {
        for destination in &self.destinations {
            if let Err(e) = destination.send(data.clone()).await {
                // Log error but continue routing to other destinations
                tracing::error!(
                    "Failed to send data to destination {}: {}",
                    destination.get_id(),
                    e
                );
            }
        }
        
        Ok(())
    }
    
    /// Route a batch of data to all enabled destinations
    pub async fn route_batch(&self, data: Vec<Value>) -> Result<()> {
        for destination in &self.destinations {
            if let Err(e) = destination.send_batch(data.clone()).await {
                // Log error but continue routing to other destinations
                tracing::error!(
                    "Failed to send batch to destination {}: {}",
                    destination.get_id(),
                    e
                );
            }
        }
        
        Ok(())
    }
}

// Re-export destination types
pub use destinations::{
    DatabaseDestination,
    EmailDestination,
    FileDestination,
    PrometheusDestination,
    SlackDestination,
    S3Destination,
    WebhookDestination,
};

// Re-export destination factory
pub use destination_factory::DestinationFactory;

// Re-export router factory
pub use router_factory::RouterFactory;

// Re-export route functionality
pub use route::Route;

// Re-export routing functionality
pub use routing::*; 