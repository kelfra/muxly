# Router Guide

The Router module in Muxly allows you to define data routes that extract data from connectors, optionally transform it, and send it to one or more destinations.

## Overview

The Router provides the following capabilities:

- Define data routes with sources, transformations, and destinations
- Configure data transformations to clean, filter, and enrich data
- Send data to multiple destinations simultaneously
- Monitor data flow and processing statistics

## Configuration

Router configuration is defined in the main `config.yml` file under the `router` section:

```yaml
router:
  routes:
    - id: "sales-to-marketing"
      name: "Sales Data to Marketing"
      enabled: true
      source:
        connector_id: "bigquery-main"
        data_spec:
          query: "SELECT id, name, email, purchase_amount FROM sales.customers"
      transformations:
        - type: "rename_field"
          params:
            from: "purchase_amount"
            to: "revenue"
      destinations:
        - destination_type: "webhook"
          config:
            url: "https://api.example.com/webhook"
            headers:
              Authorization: "Bearer ${ENV_SECRET_KEY}"
              Content-Type: "application/json"
        - destination_type: "slack"
          config:
            webhook_url: "https://hooks.slack.com/services/XXX/YYY/ZZZ"
            channel: "#data-notifications"
            message_template: "New sales data received: {{count}} records"
```

## Route Configuration

Each route consists of three main components:

1. **Source**: Where the data comes from
2. **Transformations**: How the data is processed (optional)
3. **Destinations**: Where the data is sent

### Source Configuration

A source specifies which connector to use and what data to retrieve:

```yaml
source:
  connector_id: "bigquery-main"  # Must match a configured connector ID
  data_spec:
    # Data specification (varies by connector type)
    query: "SELECT * FROM sales.transactions LIMIT 100"
```

The `data_spec` field varies depending on the connector type:

#### BigQuery Source

```yaml
data_spec:
  query: "SELECT * FROM dataset.table WHERE date = @date"
  parameters:
    - name: "date"
      type: "STRING"
      value: "2023-01-01"
  max_results: 1000  # Optional
```

#### GA4 Source

```yaml
data_spec:
  dateRanges:
    - startDate: "7daysAgo"
      endDate: "yesterday"
  dimensions:
    - name: "date"
    - name: "country"
  metrics:
    - name: "activeUsers"
    - name: "sessions"
  limit: 10000  # Optional
```

#### HubSpot Source

```yaml
data_spec:
  objectType: "contacts"
  properties:
    - "firstname"
    - "lastname"
    - "email"
  filter:
    propertyName: "createdate"
    operator: "GTE"
    value: "2023-01-01T00:00:00Z"
  limit: 100  # Optional
```

### Transformation Configuration

Transformations allow you to modify the data before sending it to destinations. Multiple transformations can be chained together, and they are applied in the order defined:

```yaml
transformations:
  - type: "filter"
    params:
      field: "revenue"
      operator: ">"
      value: 1000
  - type: "rename_field"
    params:
      from: "purchase_amount"
      to: "revenue"
```

#### Available Transformations

##### Field Rename

Renames a field in the data:

```yaml
type: "rename_field"
params:
  from: "purchase_amount"
  to: "revenue"
```

##### Filter

Filters records based on field values:

```yaml
type: "filter"
params:
  field: "revenue"
  operator: ">"  # >, <, >=, <=, =, !=, contains, not_contains, starts_with, ends_with
  value: 1000
```

##### Formula

Creates a new field using a formula:

```yaml
type: "formula"
params:
  output_field: "discount_amount"
  formula: "price * discount_rate"
```

##### Array Flatten

Flattens array fields:

```yaml
type: "array_flatten"
params:
  array_field: "products"
  flatten_fields:
    - "product_id"
    - "product_name"
  preserve_parent: true
```

##### Join

Joins data with another dataset:

```yaml
type: "join"
params:
  join_connector_id: "mysql-products"
  join_data_spec:
    query: "SELECT product_id, category, price FROM products"
  left_key: "product_id"
  right_key: "product_id"
  join_type: "left"  # left, inner, right
  prefix_fields: true  # Add prefix to joined fields
  prefix: "product_"
```

##### Aggregate

Performs aggregations on the data:

```yaml
type: "aggregate"
params:
  group_by:
    - "country"
    - "category"
  aggregations:
    - function: "sum"
      field: "revenue"
      as: "total_revenue"
    - function: "count"
      field: "*"
      as: "transaction_count"
    - function: "avg"
      field: "order_value"
      as: "avg_order_value"
```

### Destination Configuration

Destinations define where the processed data should be sent. Muxly supports multiple destination types to handle different use cases.

> **Note**: For detailed configuration options for each destination type, see the [Destinations Guide](destinations.md).

#### Available Destination Types

##### Database Destination

Stores data in relational databases:

```yaml
destination_type: "database"
config:
  db_type: "postgres"  # postgres, mysql, sqlite
  connection_string: "${DATABASE_URL}"
  table: "marketing_data"
  schema: "public"  # For PostgreSQL
  upsert_key: "id"
  batch_size: 100
  create_table: true  # Create table if it doesn't exist
```

##### Email Destination

Sends email notifications:

```yaml
destination_type: "email"
config:
  smtp_host: "smtp.example.com"
  smtp_port: 587
  smtp_username: "${SMTP_USERNAME}"
  smtp_password: "${SMTP_PASSWORD}"
  use_tls: true
  from_email: "notifications@example.com"
  to_emails:
    - "recipient@example.com"
  subject_template: "New data from {{connector_id}}"
```

##### File Destination

Writes data to local files:

```yaml
destination_type: "file"
config:
  output_dir: "/path/to/output"
  file_prefix: "data_"
  file_format: "json"  # json, csv, parquet
  max_file_size_mb: 10
  rotation: "daily"  # none, daily, hourly, size
```

##### Prometheus Destination

Publishes metrics to Prometheus:

```yaml
destination_type: "prometheus"
config:
  metrics:
    - name: "data_value"
      type: "gauge"
      description: "Value from data"
      value_field: "value"
      labels:
        source: "connector_id"
```

##### S3 Destination

Stores data in Amazon S3 buckets:

```yaml
destination_type: "s3"
config:
  bucket: "my-data-bucket"
  key_prefix: "data/"
  region: "us-west-2"
  output_format: "json"  # json, csv, parquet
  key_template: "{{connector_id}}/{{date}}/data.json"
  credentials:
    access_key_id: "${AWS_ACCESS_KEY_ID}"
    secret_access_key: "${AWS_SECRET_ACCESS_KEY}"
```

##### Slack Destination

Sends notifications to Slack channels:

```yaml
destination_type: "slack"
config:
  webhook_url: "https://hooks.slack.com/services/XXX/YYY/ZZZ"
  channel: "#data-notifications"
  username: "Muxly Bot"
  icon: ":chart_with_upwards_trend:"
  message_template: "New data received from {{connector_id}}"
  include_data: true
```

##### Webhook Destination

Sends data to HTTP endpoints:

```yaml
destination_type: "webhook"
config:
  url: "https://api.example.com/webhook"
  method: "POST"  # Default: POST
  headers:
    Authorization: "Bearer ${TOKEN}"
    Content-Type: "application/json"
  batch_size: 100  # How many records to send in one request
  retry:
    max_attempts: 3
```

## Advanced Features

### Conditional Routing

You can add conditions to routes to determine when data should be sent:

```yaml
routes:
  - id: "high-value-customers"
    name: "High Value Customers Route"
    enabled: true
    source:
      connector_id: "bigquery-sales"
      data_spec:
        query: "SELECT * FROM customers"
    condition: "total_purchases > 1000 AND last_purchase_date > '2023-01-01'"
    destinations:
      - destination_type: "slack"
        config:
          webhook_url: "https://hooks.slack.com/services/XXX/YYY/ZZZ"
          message_template: "High value customer alert: {{customer_name}}"
```

### Error Handling

You can specify error handling behavior for routes:

```yaml
routes:
  - id: "critical-data-sync"
    name: "Critical Data Sync"
    enabled: true
    source:
      connector_id: "mysql-main"
      data_spec:
        query: "SELECT * FROM critical_data"
    destinations:
      - destination_type: "database"
        config:
          db_type: "postgres"
          connection_string: "${DATABASE_URL}"
          table: "critical_data_backup"
    error_handling:
      on_error: "fail"  # Options: continue, fail
      error_destination:  # Optional, where to send error data
        destination_type: "slack"
        config:
          webhook_url: "https://hooks.slack.com/services/XXX/YYY/ZZZ"
          channel: "#data-alerts"
          message_template: "Error in critical data sync: {{error_message}}"
```

## Monitoring Routes

You can monitor your routes through the Muxly API:

```bash
# Get all routes
curl http://localhost:3000/v1/router/routes

# Get a specific route
curl http://localhost:3000/v1/router/routes/sales-to-marketing

# Get route execution history
curl http://localhost:3000/v1/router/routes/sales-to-marketing/history
```

For more information on the available API endpoints, see the [API Reference](api-reference.md).

## Using Dynamic Variables

You can use dynamic variables in your route configurations. These variables are replaced at runtime:

| Variable | Description | Example |
|----------|-------------|---------|
| `${DATE}` | Current date (YYYY-MM-DD) | `2023-04-15` |
| `${YESTERDAY}` | Yesterday's date (YYYY-MM-DD) | `2023-04-14` |
| `${TIMESTAMP}` | Current timestamp (YYYYMMDD_HHMMSS) | `20230415_120000` |
| `${YEAR}` | Current year (YYYY) | `2023` |
| `${MONTH}` | Current month (MM) | `04` |
| `${DAY}` | Current day (DD) | `15` |
| `${ENV_VARIABLE}` | Environment variable | Value of the environment variable |

Example usage:
```yaml
data_spec:
  query: "SELECT * FROM sales.transactions WHERE date = '${YESTERDAY}'"
```

## Running Routes

Routes can be executed in different ways:

### API Execution

You can trigger routes via the API:

```bash
curl -X POST http://localhost:8080/api/v1/router/routes/sales-to-marketing/run \
  -H "Authorization: Bearer ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"parameters": {"start_date": "2023-01-01"}}'
```

### Scheduled Execution

You can schedule routes using the Scheduler module:

```yaml
scheduler:
  jobs:
    - id: "daily-sales-sync"
      name: "Daily Sales Sync"
      schedule: "0 0 * * *"
      enabled: true
      action:
        type: "pipeline"
        params:
          pipeline_id: "sales-to-marketing"
          run_params:
            date: "${DATE}"
```

See the [Scheduler Guide](./scheduler.md) for more details.

## Monitoring

You can monitor route execution through the API or web interface:

### API Endpoints

- `GET /api/v1/router/routes` - List all configured routes
- `GET /api/v1/router/routes/{route_id}` - Get details of a specific route
- `GET /api/v1/router/routes/{route_id}/runs` - Get execution history of a route
- `POST /api/v1/router/routes/{route_id}/run` - Manually trigger a route
- `PUT /api/v1/router/routes/{route_id}/enable` - Enable a route
- `PUT /api/v1/router/routes/{route_id}/disable` - Disable a route

### Web Interface

The Muxly web interface provides a dashboard for monitoring routes, including:

- Route status
- Execution history with success/failure status
- Data processing statistics
- Manual run capability
- Enable/disable functionality

## Best Practices

1. **Use descriptive route IDs and names** to easily identify routes.
2. **Keep transformations simple** and chain multiple small transformations instead of complex ones.
3. **Use validation** to ensure data quality before sending to destinations.
4. **Monitor performance** and optimize transformations for large datasets.
5. **Use environment variables** for sensitive information like API keys and connection strings.
6. **Start with small batches** and gradually increase as you confirm stability.
7. **Implement error handling** to capture and address data processing issues.
8. **Document your routes** with clear descriptions of their purpose and data flow.

## Troubleshooting

### Common Issues

#### Data Not Appearing at Destination

- Verify the source connector is working correctly
- Check transformation logic for filters that might exclude all data
- Validate destination configuration, especially authentication
- Check network connectivity to destination services

#### Slow Performance

- Optimize source queries to retrieve only necessary data
- Simplify complex transformations
- Increase batch sizes for better throughput
- Consider adding indexes to database tables

#### Out of Memory Errors

- Reduce batch sizes to process less data at once
- Optimize transformations to use less memory
- For large datasets, consider using incremental processing 