use anyhow::Result;
use std::sync::Arc;

use crate::router::{Router, RouterData, DestinationFactory};

/// Factory for creating router instances from configuration
pub struct RouterFactory;

impl RouterFactory {
    /// Create a new router from the provided configuration
    pub fn create_router(config: &RouterData) -> Result<Router> {
        // Create destinations from settings
        let destinations = DestinationFactory::create_destinations(&config.destinations)?;
        
        // Create the router with the destinations
        Ok(Router::new(destinations))
    }
    
    /// Create multiple routers from a list of configurations
    pub fn create_routers(configs: &[RouterData]) -> Result<Vec<Router>> {
        let mut routers = Vec::new();
        
        for config in configs {
            // Skip disabled routers
            if !config.enabled {
                continue;
            }
            
            routers.push(Self::create_router(config)?);
        }
        
        Ok(routers)
    }
} 