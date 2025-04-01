# Router Rules and Conditions

This guide explains how to configure and use routing rules and conditions in Muxly. Routing rules allow you to direct data to different destinations based on the content of the data.

## Overview

Routing rules consist of:

1. **Conditions**: Expressions that determine when the rule should be applied
2. **Transformations**: Changes to make to the data before sending it
3. **Destinations**: Where to send the data when the rule matches

## Rule Configuration

Here's an example of a router configuration with rules:

```yaml
router:
  routes:
    - id: "sales-data-route"
      name: "Sales Data Router"
      enabled: true
      source:
        connector_id: "bigquery-sales"
        data_spec:
          query: "SELECT * FROM sales.transactions"
      transformations:
        - type: "rename_field"
          params:
            from: "purchase_amount"
            to: "revenue"
      rules:
        - id: "high-value-orders"
          name: "High Value Orders"
          enabled: true
          condition: "revenue > 1000"
          priority: 1
          destination_ids:
            - "slack_notifications"
            - "database_analytics"
          transformations:
            - type: "set_field"
              params:
                field: "category"
                value: "high-value"
        - id: "regular-orders"
          name: "Regular Orders"
          enabled: true
          condition: "revenue <= 1000"
          priority: 2
          destination_ids:
            - "file_storage"
```

## Condition Syntax

Conditions are expressions that evaluate to true or false. The condition syntax supports:

### Basic Comparisons

```
field operator value
```

Where:
- `field` is the name of a field in the data
- `operator` is one of: `==`, `!=`, `>`, `>=`, `<`, `<=`, `CONTAINS`, `NOT CONTAINS`, `STARTS WITH`, `ENDS WITH`
- `value` is a literal value like a number or string (strings should be quoted with single quotes, e.g., `'US'`)

Examples:
- `revenue > 1000`
- `country == 'US'`
- `email CONTAINS '@example.com'`
- `product_id STARTS WITH 'XYZ'`

### Logical Operators

Conditions can be combined using logical operators:

- `condition1 AND condition2`: Both conditions must be true
- `condition1 OR condition2`: At least one condition must be true
- `NOT condition`: The condition must be false

Examples:
- `revenue > 1000 AND country == 'US'`
- `category == 'electronics' OR category == 'computers'`
- `NOT status == 'cancelled'`

### Existence Checks

You can check if a field exists using the `EXISTS` keyword:

```
EXISTS field_path
```

Example:
- `EXISTS shipping_address`
- `NOT EXISTS error_code`

### JSONPath Support

For accessing nested fields, you can use JSONPath syntax:

```
$.user.profile.country == 'US'
```

This checks if the country in the user's profile is 'US'.

## Transformations

Transformations allow you to modify data before sending it to destinations. Common transformations include:

### Field Rename
```yaml
type: "rename_field"
params:
  from: "purchase_amount"
  to: "revenue"
```

### Filter
```yaml
type: "filter"
params:
  field: "revenue"
  operator: ">"
  value: 1000
```

### Formula
```yaml
type: "formula"
params:
  output_field: "discount_amount"
  formula: "price * discount_rate"
```

### Set Field
```yaml
type: "set_field"
params:
  field: "priority"
  value: "high"
```

### Remove Field
```yaml
type: "remove_field"
params:
  field: "internal_notes"
```

### Format String
```yaml
type: "format_string"
params:
  template: "Order from {{customer_name}} for ${{total_amount}}"
  output_field: "order_summary"
```

## Destination Selection

Routes direct data to destinations based on:

1. **Specific IDs**: Explicitly list destination IDs
2. **Dynamic selection**: Use conditions to determine destinations
3. **Default routing**: Send to all destinations if no rules match

## Priority Order

When multiple rules match the same data, the rule with the lowest priority number (highest priority) is applied first. If multiple rules have the same priority, they are applied in the order they are defined.

## Error Handling

You can configure error handling for rules:

```yaml
rules:
  - id: "critical-data-rule"
    # ... other configuration ...
    error_handling:
      on_error: "fail"  # or "continue"
      retry:
        max_attempts: 3
        backoff_seconds: 5
```

## Testing Rules

You can test rules using the Muxly API:

```bash
# Test a rule with sample data
curl -X POST http://localhost:3000/v1/router/rules/test \
  -H "Content-Type: application/json" \
  -d '{
    "rule_id": "high-value-orders",
    "data": {
      "revenue": 1500,
      "country": "US"
    }
  }'
```

## Monitoring and Troubleshooting

To monitor rule execution and troubleshoot issues:

1. Check rule execution history:
```bash
curl http://localhost:3000/v1/router/rules/high-value-orders/history
```

2. Enable debug logging for detailed information:
```yaml
app:
  log_level: debug
```

3. Use the rule testing API to validate conditions:
```bash
curl -X POST http://localhost:3000/v1/router/conditions/test \
  -H "Content-Type: application/json" \
  -d '{
    "condition": "revenue > 1000 AND country == 'US'",
    "data": {
      "revenue": 1500,
      "country": "US"
    }
  }'
``` 