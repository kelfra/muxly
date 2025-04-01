use regex::Regex;
use serde_json::Value;
use tracing::{debug, warn};

use crate::collector::ValidationRule;

/// Validate data against validation rules
pub fn validate_data(data: &Value, rules: &[ValidationRule]) -> bool {
    // If there are no rules, validation passes
    if rules.is_empty() {
        return true;
    }
    
    // Apply all rules
    for rule in rules {
        if !validate_rule(data, rule) {
            // Log the validation failure
            if let Some(message) = &rule.error_message {
                warn!("Validation failed: {}", message);
            } else {
                warn!(
                    "Validation failed for field '{}' with rule '{}'",
                    rule.field, rule.rule_type
                );
            }
            
            return false;
        }
    }
    
    true
}

/// Validate a single rule against data
fn validate_rule(data: &Value, rule: &ValidationRule) -> bool {
    // Get the field value using JSON pointer syntax
    let field_value = get_field_value(data, &rule.field);
    
    // Apply the rule based on its type
    match rule.rule_type.as_str() {
        "required" => validate_required(&field_value),
        "min" => validate_min(&field_value, &rule.value),
        "max" => validate_max(&field_value, &rule.value),
        "regex" => validate_regex(&field_value, &rule.value),
        "type" => validate_type(&field_value, &rule.value),
        "enum" => validate_enum(&field_value, &rule.value),
        "length" => validate_length(&field_value, &rule.value),
        _ => {
            // Unknown rule type, log a warning and pass
            warn!("Unknown validation rule type: {}", rule.rule_type);
            true
        }
    }
}

/// Get field value from data using JSON pointer syntax
fn get_field_value<'a>(data: &'a Value, field: &str) -> Option<&'a Value> {
    // Check if it's a JSON pointer
    if field.starts_with('/') {
        data.pointer(field)
    } else {
        // Otherwise treat as a simple field name
        match data {
            Value::Object(obj) => obj.get(field),
            _ => None,
        }
    }
}

/// Validate that a field is present and not null
fn validate_required(field_value: &Option<&Value>) -> bool {
    if let Some(value) = field_value {
        !value.is_null()
    } else {
        false
    }
}

/// Validate that a numeric field is at least a minimum value
fn validate_min(field_value: &Option<&Value>, min_value: &Option<Value>) -> bool {
    // If no value or no min specified, validation passes
    if field_value.is_none() || min_value.is_none() {
        return true;
    }
    
    let value = match field_value.unwrap() {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    };
    
    let min = match min_value.as_ref().unwrap() {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    };
    
    match (value, min) {
        (Some(v), Some(m)) => v >= m,
        _ => false, // If either couldn't be parsed as a number
    }
}

/// Validate that a numeric field is at most a maximum value
fn validate_max(field_value: &Option<&Value>, max_value: &Option<Value>) -> bool {
    // If no value or no max specified, validation passes
    if field_value.is_none() || max_value.is_none() {
        return true;
    }
    
    let value = match field_value.unwrap() {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    };
    
    let max = match max_value.as_ref().unwrap() {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    };
    
    match (value, max) {
        (Some(v), Some(m)) => v <= m,
        _ => false, // If either couldn't be parsed as a number
    }
}

/// Validate that a string field matches a regex pattern
fn validate_regex(field_value: &Option<&Value>, pattern: &Option<Value>) -> bool {
    // If no value or no pattern specified, validation passes
    if field_value.is_none() || pattern.is_none() {
        return true;
    }
    
    let value = match field_value.unwrap() {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    };
    
    let pattern_str = match pattern.as_ref().unwrap() {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    };
    
    match (value, pattern_str) {
        (Some(v), Some(p)) => {
            // Compile the regex pattern
            match Regex::new(p) {
                Ok(regex) => regex.is_match(v),
                Err(e) => {
                    warn!("Invalid regex pattern: {}", e);
                    false
                }
            }
        }
        _ => false, // If either isn't a string
    }
}

/// Validate that a field is of a specific JSON type
fn validate_type(field_value: &Option<&Value>, expected_type: &Option<Value>) -> bool {
    // If no value or no type specified, validation passes
    if field_value.is_none() || expected_type.is_none() {
        return true;
    }
    
    let value = field_value.unwrap();
    
    let type_str = match expected_type.as_ref().unwrap() {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    };
    
    if let Some(expected) = type_str {
        match expected {
            "null" => value.is_null(),
            "boolean" => value.is_boolean(),
            "number" => value.is_number(),
            "string" => value.is_string(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            _ => {
                warn!("Unknown type for validation: {}", expected);
                false
            }
        }
    } else {
        false
    }
}

/// Validate that a field's value is one of a set of allowed values
fn validate_enum(field_value: &Option<&Value>, allowed_values: &Option<Value>) -> bool {
    // If no value or no allowed values specified, validation passes
    if field_value.is_none() || allowed_values.is_none() {
        return true;
    }
    
    let value = field_value.unwrap();
    
    let allowed = match allowed_values.as_ref().unwrap() {
        Value::Array(a) => Some(a),
        _ => None,
    };
    
    if let Some(allowed_array) = allowed {
        allowed_array.contains(value)
    } else {
        false
    }
}

/// Validate the length of a string or array
fn validate_length(field_value: &Option<&Value>, length_constraints: &Option<Value>) -> bool {
    // If no value or no length constraints specified, validation passes
    if field_value.is_none() || length_constraints.is_none() {
        return true;
    }
    
    let value = field_value.unwrap();
    
    // Get the length of the value
    let length = match value {
        Value::String(s) => s.len(),
        Value::Array(a) => a.len(),
        _ => return false, // Not a string or array
    };
    
    let constraints = match length_constraints.as_ref().unwrap() {
        Value::Object(obj) => Some(obj),
        _ => None,
    };
    
    if let Some(obj) = constraints {
        // Check minimum length
        if let Some(min) = obj.get("min") {
            if let Some(min_len) = min.as_u64() {
                if length < min_len as usize {
                    return false;
                }
            }
        }
        
        // Check maximum length
        if let Some(max) = obj.get("max") {
            if let Some(max_len) = max.as_u64() {
                if length > max_len as usize {
                    return false;
                }
            }
        }
        
        // Check exact length
        if let Some(exact) = obj.get("exact") {
            if let Some(exact_len) = exact.as_u64() {
                if length != exact_len as usize {
                    return false;
                }
            }
        }
        
        true
    } else {
        false
    }
} 