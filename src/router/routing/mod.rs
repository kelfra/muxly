mod conditions;
mod transformations;

pub use conditions::*;
pub use transformations::*;

use anyhow::Result;
use serde_json::Value;
use serde::{Deserialize, Serialize};

/// Routing rule that determines how data should be routed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Condition for when this rule applies (optional)
    pub condition: Option<String>,
    /// Priority of the rule (lower number = higher priority)
    pub priority: i32,
    /// Destination IDs to route to
    pub destination_ids: Vec<String>,
    /// Transformations to apply
    pub transformations: Vec<TransformationStep>,
}

/// Result of evaluating a routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// ID of the rule that was applied
    pub rule_id: String,
    /// Whether the rule matched
    pub matched: bool,
    /// Destination IDs that the data was sent to
    pub routed_to: Vec<String>,
    /// Any error that occurred
    pub error: Option<String>,
}

/// Evaluates a routing rule against data
pub fn evaluate_rule(rule: &RoutingRule, data: &Value) -> Result<bool> {
    // If there's no condition, the rule always matches
    if rule.condition.is_none() {
        return Ok(true);
    }
    
    // Otherwise, evaluate the condition
    conditions::evaluate_condition(&rule.condition.as_ref().unwrap(), data)
}

/// Apply transformations to data
pub fn apply_transformations(transformations: &[TransformationStep], data: Value) -> Result<Value> {
    let mut current_data = data;
    
    for step in transformations {
        current_data = transformations::apply_transformation(step, current_data)?;
    }
    
    Ok(current_data)
} 