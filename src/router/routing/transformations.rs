use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
use jsonpath_lib as jsonpath;
use std::collections::HashMap;

/// Transformation step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationStep {
    /// Type of transformation
    pub transformation_type: String,
    /// Parameters for the transformation
    pub params: Value,
}

/// Apply a transformation to data
pub fn apply_transformation(step: &TransformationStep, data: Value) -> Result<Value> {
    match step.transformation_type.as_str() {
        "rename_field" => rename_field(step, data),
        "filter" => filter(step, data),
        "formula" => formula(step, data),
        "array_flatten" => array_flatten(step, data),
        "join" => join(step, data),
        "aggregate" => aggregate(step, data),
        "extract" => extract(step, data),
        "set_field" => set_field(step, data),
        "remove_field" => remove_field(step, data),
        "format_string" => format_string(step, data),
        _ => Err(anyhow!("Unknown transformation type: {}", step.transformation_type)),
    }
}

/// Rename a field in the data
fn rename_field(step: &TransformationStep, mut data: Value) -> Result<Value> {
    let from = step.params.get("from")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'from' parameter"))?;
    
    let to = step.params.get("to")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'to' parameter"))?;
    
    if let Value::Object(obj) = &mut data {
        if let Some(value) = obj.remove(from) {
            obj.insert(to.to_string(), value);
        }
    }
    
    Ok(data)
}

/// Filter data based on a condition
fn filter(step: &TransformationStep, data: Value) -> Result<Value> {
    let field = step.params.get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'field' parameter"))?;
    
    let operator = step.params.get("operator")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'operator' parameter"))?;
    
    let value = step.params.get("value")
        .ok_or_else(|| anyhow!("Missing 'value' parameter"))?;
    
    // Create a condition string in the format expected by the conditions module
    let condition = match operator {
        ">" => format!("{} > {}", field, value),
        "<" => format!("{} < {}", field, value),
        ">=" => format!("{} >= {}", field, value),
        "<=" => format!("{} <= {}", field, value),
        "=" => format!("{} == {}", field, value),
        "!=" => format!("{} != {}", field, value),
        "contains" => format!("{} CONTAINS {}", field, value),
        "not_contains" => format!("{} NOT CONTAINS {}", field, value),
        "starts_with" => format!("{} STARTS WITH {}", field, value),
        "ends_with" => format!("{} ENDS WITH {}", field, value),
        _ => return Err(anyhow!("Unknown operator: {}", operator)),
    };
    
    // Evaluate the condition
    let result = crate::router::routing::conditions::evaluate_condition(&condition, &data)?;
    
    if result {
        Ok(data)
    } else {
        // Return an empty object if the condition is not met
        Ok(json!({}))
    }
}

/// Create a new field using a formula
fn formula(step: &TransformationStep, mut data: Value) -> Result<Value> {
    let output_field = step.params.get("output_field")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'output_field' parameter"))?;
    
    let formula = step.params.get("formula")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'formula' parameter"))?;
    
    // Very basic formula evaluation that only supports field references and basic arithmetic
    // In a real implementation, we would use a proper expression evaluator
    
    // Extract field names from the formula
    let mut field_values = HashMap::new();
    let mut current_field = String::new();
    let mut in_field = false;
    
    for c in formula.chars() {
        if c.is_alphabetic() || c == '_' || (in_field && c.is_numeric()) {
            if !in_field {
                in_field = true;
                current_field.clear();
            }
            current_field.push(c);
        } else {
            if in_field && !current_field.is_empty() {
                // Extract the field value from the data
                if let Value::Object(obj) = &data {
                    if let Some(value) = obj.get(&current_field) {
                        if let Some(num) = value.as_f64() {
                            field_values.insert(current_field.clone(), num);
                        }
                    }
                }
                in_field = false;
            }
        }
    }
    
    // Check for any remaining field
    if in_field && !current_field.is_empty() {
        if let Value::Object(obj) = &data {
            if let Some(value) = obj.get(&current_field) {
                if let Some(num) = value.as_f64() {
                    field_values.insert(current_field.clone(), num);
                }
            }
        }
    }
    
    // Simplified formula evaluation by substituting field values
    // This is a very basic implementation that only works for simple cases
    let mut formula_with_values = formula.to_string();
    for (field, value) in field_values {
        formula_with_values = formula_with_values.replace(&field, &value.to_string());
    }
    
    // Evaluate the formula (very basic implementation)
    // In a real implementation, we would use a proper expression evaluator
    let result = evaluate_simple_expression(&formula_with_values)?;
    
    // Set the output field in the data
    if let Value::Object(obj) = &mut data {
        obj.insert(output_field.to_string(), json!(result));
    }
    
    Ok(data)
}

/// Flatten array fields into separate records
fn array_flatten(step: &TransformationStep, data: Value) -> Result<Value> {
    let array_field = step.params.get("array_field")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'array_field' parameter"))?;
    
    let flatten_fields = step.params.get("flatten_fields")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Missing or invalid 'flatten_fields' parameter"))?;
    
    let preserve_parent = step.params.get("preserve_parent")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    
    // Extract the array from the data
    let array = match data.pointer(&format!("/{}", array_field)) {
        Some(Value::Array(arr)) => arr,
        _ => return Err(anyhow!("Field '{}' is not an array or doesn't exist", array_field)),
    };
    
    // Prepare the result
    let mut result = Vec::new();
    
    // Process each item in the array
    for item in array {
        if let Value::Object(item_obj) = item {
            let mut new_record = if preserve_parent {
                if let Value::Object(obj) = &data {
                    // Clone the parent record
                    let mut parent = obj.clone();
                    // Remove the array field to avoid circular references
                    parent.remove(array_field);
                    parent
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };
            
            // Add the flattened fields
            for field in flatten_fields {
                if let Value::String(field_name) = field {
                    if let Some(value) = item_obj.get(field_name) {
                        new_record.insert(field_name.clone(), value.clone());
                    }
                }
            }
            
            result.push(Value::Object(new_record));
        }
    }
    
    Ok(Value::Array(result))
}

/// Join data with another dataset
fn join(_step: &TransformationStep, data: Value) -> Result<Value> {
    // In a real implementation, we would fetch data from the join_connector
    // and perform the join. For now, we just return the original data.
    Ok(data)
}

/// Perform aggregations on the data
fn aggregate(_step: &TransformationStep, data: Value) -> Result<Value> {
    // In a real implementation, we would group the data and apply
    // aggregation functions. For now, we just return the original data.
    Ok(data)
}

/// Extract specific fields from the data
fn extract(step: &TransformationStep, data: Value) -> Result<Value> {
    let fields = step.params.get("fields")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Missing or invalid 'fields' parameter"))?;
    
    let mut result = json!({});
    
    if let Value::Object(result_obj) = &mut result {
        if let Value::Object(data_obj) = &data {
            for field in fields {
                if let Value::String(field_name) = field {
                    if let Some(value) = data_obj.get(field_name) {
                        result_obj.insert(field_name.clone(), value.clone());
                    }
                }
            }
        }
    }
    
    Ok(result)
}

/// Set a field to a specific value
fn set_field(step: &TransformationStep, mut data: Value) -> Result<Value> {
    let field = step.params.get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'field' parameter"))?;
    
    let value = step.params.get("value")
        .ok_or_else(|| anyhow!("Missing 'value' parameter"))?;
    
    if let Value::Object(obj) = &mut data {
        obj.insert(field.to_string(), value.clone());
    }
    
    Ok(data)
}

/// Remove a field from the data
fn remove_field(step: &TransformationStep, mut data: Value) -> Result<Value> {
    let field = step.params.get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'field' parameter"))?;
    
    if let Value::Object(obj) = &mut data {
        obj.remove(field);
    }
    
    Ok(data)
}

/// Format a string using field values
fn format_string(step: &TransformationStep, mut data: Value) -> Result<Value> {
    let template = step.params.get("template")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'template' parameter"))?;
    
    let output_field = step.params.get("output_field")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing or invalid 'output_field' parameter"))?;
    
    // Format the string by replacing {{field}} with the field value
    let mut result = template.to_string();
    
    if let Value::Object(data_obj) = &data {
        for (key, value) in data_obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => format!("{:?}", value),
            };
            
            result = result.replace(&placeholder, &value_str);
        }
    }
    
    // Set the output field
    if let Value::Object(obj) = &mut data {
        obj.insert(output_field.to_string(), Value::String(result));
    }
    
    Ok(data)
}

/// Helper function for basic expression evaluation
fn evaluate_simple_expression(expr: &str) -> Result<f64> {
    // This is a very simplified expression evaluator that only handles basic arithmetic
    // In a real implementation, we would use a proper expression evaluator
    
    // Support very basic expressions like "a + b * c"
    let cleaned = expr.replace(' ', "");
    
    // First try to parse as a simple number
    if let Ok(num) = cleaned.parse::<f64>() {
        return Ok(num);
    }
    
    // Very basic parsing for + and - operations
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut last_char = ' ';
    
    for c in cleaned.chars() {
        if (c == '+' || c == '-') && last_char != 'e' && last_char != 'E' {
            if !current.is_empty() {
                parts.push(current);
                current = String::new();
            }
            current.push(c);
        } else {
            current.push(c);
        }
        last_char = c;
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    // If no parts were found, try to parse the whole expression
    if parts.is_empty() {
        return cleaned.parse::<f64>().map_err(|_| anyhow!("Invalid expression: {}", expr));
    }
    
    // Evaluate each part recursively and combine the results
    let mut result = 0.0;
    let mut op = '+';
    
    for part in parts {
        if part == "+" {
            op = '+';
        } else if part == "-" {
            op = '-';
        } else {
            let value = part.parse::<f64>().map_err(|_| anyhow!("Invalid part in expression: {}", part))?;
            match op {
                '+' => result += value,
                '-' => result -= value,
                _ => return Err(anyhow!("Unsupported operator: {}", op)),
            }
        }
    }
    
    Ok(result)
} 