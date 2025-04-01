use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;

use crate::router::{
    Destination, Router, RouterData, RoutingRule, DestinationFactory,
    evaluate_rule, apply_transformations, RoutingResult
};

/// A route defines how data is processed and where it goes
pub struct Route {
    /// The router configuration
    pub config: RouterData,
    /// The router instance
    pub router: Router,
    /// The routing rules
    pub rules: Vec<RoutingRule>,
    /// The destination map (id to destination)
    destination_map: HashMap<String, Arc<dyn Destination>>,
}

impl Route {
    /// Create a new route from configuration
    pub fn new(config: RouterData, rules: Vec<RoutingRule>) -> Result<Self> {
        // Create destinations from config
        let destinations = DestinationFactory::create_destinations(&config.destinations)?;
        
        // Create a map of destination ids to destinations
        let mut destination_map = HashMap::new();
        for (i, dest) in destinations.iter().enumerate() {
            let id = if i < config.destinations.len() {
                match &config.destinations[i].destination_type {
                    dest_type => format!("{}_{}", dest_type, dest.get_id()),
                }
            } else {
                dest.get_id().to_string()
            };
            
            destination_map.insert(id, Arc::clone(dest));
        }
        
        // Create the router
        let router = Router::new(destinations);
        
        Ok(Self {
            config,
            router,
            rules,
            destination_map,
        })
    }
    
    /// Process data through the route
    pub async fn process(&self, data: Value) -> Result<Vec<RoutingResult>> {
        let mut results = Vec::new();
        
        // Apply transformations if any
        let mut processed_data = data;
        
        if !self.config.transformations.is_empty() {
            let transformations = self.config.transformations.iter()
                .map(|t| crate::router::TransformationStep {
                    transformation_type: t.transformation_type.clone(),
                    params: t.params.clone(),
                })
                .collect::<Vec<_>>();
            
            processed_data = apply_transformations(&transformations, processed_data)?;
        }
        
        // Apply rules or use default routing if no rules
        if self.rules.is_empty() {
            // Default routing: send to all destinations
            self.router.route(processed_data.clone()).await?;
            
            // Record the result
            results.push(RoutingResult {
                rule_id: "default".to_string(),
                matched: true,
                routed_to: self.destination_map.keys().cloned().collect(),
                error: None,
            });
        } else {
            // Apply each rule in priority order
            let mut sorted_rules = self.rules.clone();
            sorted_rules.sort_by_key(|r| r.priority);
            
            for rule in sorted_rules {
                // Skip disabled rules
                if !rule.enabled {
                    continue;
                }
                
                // Check if the rule matches
                let matched = match evaluate_rule(&rule, &processed_data) {
                    Ok(m) => m,
                    Err(e) => {
                        results.push(RoutingResult {
                            rule_id: rule.id.clone(),
                            matched: false,
                            routed_to: Vec::new(),
                            error: Some(e.to_string()),
                        });
                        continue;
                    }
                };
                
                if matched {
                    // Apply rule-specific transformations
                    let mut rule_data = processed_data.clone();
                    
                    if !rule.transformations.is_empty() {
                        rule_data = apply_transformations(&rule.transformations, rule_data)?;
                    }
                    
                    // Route to specified destinations
                    let mut routed_to = Vec::new();
                    let mut error = None;
                    
                    for dest_id in &rule.destination_ids {
                        if let Some(dest) = self.destination_map.get(dest_id) {
                            if let Err(e) = dest.send(rule_data.clone()).await {
                                error = Some(e.to_string());
                            } else {
                                routed_to.push(dest_id.clone());
                            }
                        }
                    }
                    
                    // Record the result
                    results.push(RoutingResult {
                        rule_id: rule.id.clone(),
                        matched: true,
                        routed_to,
                        error,
                    });
                }
            }
        }
        
        Ok(results)
    }
} 