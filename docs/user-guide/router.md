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

Destinations define where the processed data should be sent:

```yaml
destinations:
  - destination_type: "webhook"
    config:
      url: "https://api.example.com/webhook"
      headers:
        Authorization: "Bearer ${ENV_SECRET_KEY}"
        Content-Type: "application/json"
  - destination_type: "database"
    config:
      type: "postgres"
      connection_string: "${DATABASE_URL}"
      table: "marketing_data"
      upsert_key: "id"
```

#### Available Destination Types

##### Webhook

Sends data to an HTTP endpoint:

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
    initial_backoff_ms: 1000
```

##### Database

Writes data to a database:

```yaml
destination_type: "database"
config:
  type: "postgres"  # postgres, mysql, sqlite
  connection_string: "${DATABASE_URL}"
  table: "marketing_data"
  schema: "public"  # Optional
  upsert_key: "id"  # Optional, for upsert operations
  batch_size: 1000  # How many records to insert in one operation
```

##### File

Writes data to a file:

```yaml
destination_type: "file"
config:
  path: "/exports/data_${TIMESTAMP}.csv"
  format: "csv"  # csv, json, parquet
  options:
    delimiter: ","  # For CSV
    include_header: true  # For CSV
    pretty_print: true  # For JSON
```

##### S3

Writes data to an AWS S3 bucket:

```yaml
destination_type: "s3"
config:
  bucket: "my-data-bucket"
  key: "exports/data_${TIMESTAMP}.json"
  region: "us-west-2"
  format: "json"  # csv, json, parquet
  credentials:
    access_key_id: "${AWS_ACCESS_KEY_ID}"
    secret_access_key: "${AWS_SECRET_ACCESS_KEY}"
```

##### BigQuery

Writes data to Google BigQuery:

```yaml
destination_type: "bigquery"
config:
  project_id: "your-project-id"
  dataset_id: "your_dataset"
  table_id: "your_table"
  auth:
    auth_type: "service_account"
    params:
      service_account_json: "${SERVICE_ACCOUNT_JSON}"
  write_mode: "append"  # append, overwrite, upsert
  upsert_key: "id"  # Required for upsert mode
```

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

## Advanced Features

### Conditional Processing

You can add conditions to determine whether data should be processed:

```yaml
routes:
  - id: "conditional-route"
    # ... other configuration ...
    condition:
      type: "expression"
      expression: "record.revenue > 1000 && record.country == 'US'"
```

### Error Handling

Configure how errors should be handled:

```yaml
routes:
  - id: "sales-route"
    # ... other configuration ...
    error_handling:
      on_error: "continue"  # continue, fail
      error_destination:
        destination_type: "file"
        config:
          path: "/exports/errors_${TIMESTAMP}.json"
          format: "json"
```

### Data Validation

Add validation rules to ensure data quality:

```yaml
routes:
  - id: "validated-route"
    # ... other configuration ...
    validation:
      - field: "email"
        rule: "email"
        required: true
      - field: "revenue"
        rule: "number"
        min: 0
        required: true
      on_validation_error: "filter"  # filter, fail
```

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