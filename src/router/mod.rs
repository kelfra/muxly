mod destinations;
mod routing;

use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

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

pub use destinations::*;
pub use routing::*; 