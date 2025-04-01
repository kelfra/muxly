use anyhow::{Result, anyhow};
use serde_json::Value;
use jsonpath_lib as jsonpath;

/// Evaluates a condition expression against data
pub fn evaluate_condition(condition: &str, data: &Value) -> Result<bool> {
    if condition.is_empty() {
        return Ok(true);
    }
    
    // Parse the condition
    let parsed = parse_condition(condition)?;
    
    // Evaluate the parsed condition
    evaluate_parsed_condition(&parsed, data)
}

/// Parsed condition
#[derive(Debug, Clone)]
enum Condition {
    /// Simple comparison (field operator value)
    Comparison {
        field: String,
        operator: ComparisonOperator,
        value: Value,
    },
    /// Logical AND of multiple conditions
    And(Vec<Condition>),
    /// Logical OR of multiple conditions
    Or(Vec<Condition>),
    /// Logical NOT of a condition
    Not(Box<Condition>),
    /// JSON path exists
    Exists(String),
}

/// Comparison operators
#[derive(Debug, Clone)]
enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
}

/// Parses a condition string into a structured condition
fn parse_condition(condition: &str) -> Result<Condition> {
    // This is a simplified parser for demonstration
    // In a real implementation, we would use a proper parser
    
    // Basic check for AND/OR conditions
    if condition.contains(" AND ") {
        let parts: Vec<&str> = condition.split(" AND ").collect();
        let mut conditions = Vec::new();
        
        for part in parts {
            conditions.push(parse_condition(part.trim())?);
        }
        
        return Ok(Condition::And(conditions));
    }
    
    if condition.contains(" OR ") {
        let parts: Vec<&str> = condition.split(" OR ").collect();
        let mut conditions = Vec::new();
        
        for part in parts {
            conditions.push(parse_condition(part.trim())?);
        }
        
        return Ok(Condition::Or(conditions));
    }
    
    // Check for NOT condition
    if condition.starts_with("NOT ") {
        let inner = parse_condition(&condition[4..])?;
        return Ok(Condition::Not(Box::new(inner)));
    }
    
    // Check for EXISTS condition
    if condition.starts_with("EXISTS ") {
        let path = condition[7..].trim().to_string();
        return Ok(Condition::Exists(path));
    }
    
    // Parse comparison operator
    // This is a very basic parser that assumes format: field operator value
    // e.g., "revenue > 1000" or "country == 'US'"
    let operators = [
        " == ", " != ", " > ", " >= ", " < ", " <= ",
        " CONTAINS ", " NOT CONTAINS ", " STARTS WITH ", " ENDS WITH ",
    ];
    
    for op in operators.iter() {
        if condition.contains(op) {
            let parts: Vec<&str> = condition.split(op).collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid condition format: {}", condition));
            }
            
            let field = parts[0].trim().to_string();
            let value_str = parts[1].trim();
            
            // Parse value based on type
            let value = if value_str.starts_with('\'') && value_str.ends_with('\'') {
                // String value
                Value::String(value_str[1..value_str.len()-1].to_string())
            } else if value_str == "true" {
                // Boolean true
                Value::Bool(true)
            } else if value_str == "false" {
                // Boolean false
                Value::Bool(false)
            } else if value_str == "null" {
                // Null value
                Value::Null
            } else if let Ok(num) = value_str.parse::<i64>() {
                // Integer
                Value::Number(num.into())
            } else if let Ok(num) = value_str.parse::<f64>() {
                // Float
                let num_val = serde_json::Number::from_f64(num)
                    .ok_or_else(|| anyhow!("Invalid floating point number: {}", value_str))?;
                Value::Number(num_val)
            } else {
                // Default to string without quotes
                Value::String(value_str.to_string())
            };
            
            let operator = match *op {
                " == " => ComparisonOperator::Equal,
                " != " => ComparisonOperator::NotEqual,
                " > " => ComparisonOperator::GreaterThan,
                " >= " => ComparisonOperator::GreaterThanOrEqual,
                " < " => ComparisonOperator::LessThan,
                " <= " => ComparisonOperator::LessThanOrEqual,
                " CONTAINS " => ComparisonOperator::Contains,
                " NOT CONTAINS " => ComparisonOperator::NotContains,
                " STARTS WITH " => ComparisonOperator::StartsWith,
                " ENDS WITH " => ComparisonOperator::EndsWith,
                _ => return Err(anyhow!("Unknown operator: {}", op)),
            };
            
            return Ok(Condition::Comparison {
                field,
                operator,
                value,
            });
        }
    }
    
    Err(anyhow!("Could not parse condition: {}", condition))
}

/// Evaluates a parsed condition against data
fn evaluate_parsed_condition(condition: &Condition, data: &Value) -> Result<bool> {
    match condition {
        Condition::Comparison { field, operator, value } => {
            // Use JSONPath to extract the field value
            let selector = if field.starts_with('$') {
                field.clone()
            } else {
                format!("$.{}", field)
            };
            
            let result = jsonpath::select(data, &selector)?;
            if result.is_empty() {
                return Ok(false); // Field doesn't exist
            }
            
            let field_value = &result[0];
            
            // Compare values based on operator
            match operator {
                ComparisonOperator::Equal => Ok(field_value == value),
                ComparisonOperator::NotEqual => Ok(field_value != value),
                ComparisonOperator::GreaterThan => {
                    compare_numbers(field_value, value, |a, b| a > b)
                },
                ComparisonOperator::GreaterThanOrEqual => {
                    compare_numbers(field_value, value, |a, b| a >= b)
                },
                ComparisonOperator::LessThan => {
                    compare_numbers(field_value, value, |a, b| a < b)
                },
                ComparisonOperator::LessThanOrEqual => {
                    compare_numbers(field_value, value, |a, b| a <= b)
                },
                ComparisonOperator::Contains => {
                    if let (Value::String(field_str), Value::String(value_str)) = (field_value, value) {
                        Ok(field_str.contains(value_str))
                    } else {
                        Ok(false)
                    }
                },
                ComparisonOperator::NotContains => {
                    if let (Value::String(field_str), Value::String(value_str)) = (field_value, value) {
                        Ok(!field_str.contains(value_str))
                    } else {
                        Ok(false)
                    }
                },
                ComparisonOperator::StartsWith => {
                    if let (Value::String(field_str), Value::String(value_str)) = (field_value, value) {
                        Ok(field_str.starts_with(value_str))
                    } else {
                        Ok(false)
                    }
                },
                ComparisonOperator::EndsWith => {
                    if let (Value::String(field_str), Value::String(value_str)) = (field_value, value) {
                        Ok(field_str.ends_with(value_str))
                    } else {
                        Ok(false)
                    }
                },
            }
        },
        Condition::And(conditions) => {
            for condition in conditions {
                if !evaluate_parsed_condition(condition, data)? {
                    return Ok(false);
                }
            }
            Ok(true)
        },
        Condition::Or(conditions) => {
            for condition in conditions {
                if evaluate_parsed_condition(condition, data)? {
                    return Ok(true);
                }
            }
            Ok(false)
        },
        Condition::Not(condition) => {
            Ok(!evaluate_parsed_condition(condition, data)?)
        },
        Condition::Exists(path) => {
            let selector = if path.starts_with('$') {
                path.clone()
            } else {
                format!("$.{}", path)
            };
            
            let result = jsonpath::select(data, &selector)?;
            Ok(!result.is_empty())
        },
    }
}

/// Helper function to compare numeric values
fn compare_numbers<F>(a: &Value, b: &Value, compare: F) -> Result<bool>
where
    F: Fn(f64, f64) -> bool,
{
    let a_num = match a {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                f
            } else {
                return Err(anyhow!("Could not convert number to f64"));
            }
        },
        Value::String(s) => s.parse::<f64>().map_err(|_| anyhow!("Could not parse string as number"))?,
        _ => return Err(anyhow!("Expected number or string, got {:?}", a)),
    };
    
    let b_num = match b {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                f
            } else {
                return Err(anyhow!("Could not convert number to f64"));
            }
        },
        Value::String(s) => s.parse::<f64>().map_err(|_| anyhow!("Could not parse string as number"))?,
        _ => return Err(anyhow!("Expected number or string, got {:?}", b)),
    };
    
    Ok(compare(a_num, b_num))
} 