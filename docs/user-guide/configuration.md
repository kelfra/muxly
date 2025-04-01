# Configuration Guide

This guide explains how to configure Muxly for your specific needs.

## Configuration File

Muxly uses a YAML configuration file to define its behavior. By default, it looks for a file named `config.yml` in the current directory, but you can specify a different path using the `--config` flag:

```bash
muxly --config /path/to/your/config.yml
```

## Configuration Structure

The configuration file has the following structure:

```yaml
# General settings
general:
  log_level: "info"  # debug, info, warn, error
  api:
    port: 8080
    host: "127.0.0.1"
    enable_cors: true
    cors_allowed_origins:
      - "http://localhost:3000"

# Connectors configuration
connectors:
  - id: "bigquery-main"
    name: "BigQuery Main"
    connector_type: "bigquery"
    enabled: true
    auth:
      # Authentication settings (see Connectors Guide)
    connection:
      # Connection settings (see Connectors Guide)

# Scheduler settings
scheduler:
  enabled: true
  job_storage: "memory"  # memory, database
  concurrent_jobs: 5
  poll_interval_seconds: 30
  max_retries: 3
  jobs:
    - id: "daily-sync"
      name: "Daily Data Sync"
      schedule: "0 0 * * *"  # Cron expression (midnight daily)
      enabled: true
      connector_id: "bigquery-main"
      action:
        type: "query"
        params:
          # Action-specific parameters

# Router configuration
router:
  routes:
    - id: "sales-to-marketing"
      name: "Sales Data to Marketing"
      enabled: true
      source:
        connector_id: "bigquery-main"
        data_spec:
          # Data specification
      transformations:
        # Optional transformations
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

# Security settings
security:
  api_keys:
    enabled: true
    keys:
      - name: "admin"
        key: "${API_KEY}"
        roles: ["admin"]
      - name: "read-only"
        key: "${READ_ONLY_KEY}"
        roles: ["reader"]
  
  # JWT settings if using JWT authentication
  jwt:
    enabled: false
    secret: "${JWT_SECRET}"
    expiration_hours: 24
```

## Environment Variables

You can use environment variables in your configuration file by using the `${VARIABLE_NAME}` syntax. This is useful for sensitive information like API keys and secrets.

Example:
```yaml
auth:
  auth_type: "api_key"
  params:
    api_key: "${HUBSPOT_API_KEY}"
```

You can set these environment variables in your environment or use a `.env` file in the same directory as your configuration file.

## Configuration Sections

### General Settings

```yaml
general:
  log_level: "info"  # debug, info, warn, error
  api:
    port: 8080
    host: "127.0.0.1"
    enable_cors: true
    cors_allowed_origins:
      - "http://localhost:3000"
```

- `log_level`: Controls the verbosity of logs
- `api`: Configuration for the API server
  - `port`: Port to listen on
  - `host`: Host address to bind to
  - `enable_cors`: Whether to enable CORS
  - `cors_allowed_origins`: List of allowed origins for CORS

### Connectors

See the [Connectors Guide](./connectors.md) for detailed information on configuring connectors.

### Scheduler

```yaml
scheduler:
  enabled: true
  job_storage: "memory"  # memory, database
  concurrent_jobs: 5
  poll_interval_seconds: 30
  max_retries: 3
  jobs:
    - id: "daily-sync"
      name: "Daily Data Sync"
      schedule: "0 0 * * *"  # Cron expression (midnight daily)
      enabled: true
      connector_id: "bigquery-main"
      action:
        type: "query"
        params:
          query: "SELECT * FROM sales.transactions WHERE date = CURRENT_DATE() - 1"
```

- `enabled`: Whether the scheduler is enabled
- `job_storage`: Where to store job information (memory or database)
- `concurrent_jobs`: Maximum number of jobs to run simultaneously
- `poll_interval_seconds`: How often to check for jobs that need to run
- `max_retries`: Maximum number of retries for failed jobs
- `jobs`: List of scheduled jobs
  - `id`: Unique identifier for the job
  - `name`: Human-readable name
  - `schedule`: Cron expression for when to run the job
  - `enabled`: Whether this specific job is enabled
  - `connector_id`: ID of the connector to use
  - `action`: Action to perform
    - `type`: Type of action (query, export, etc.)
    - `params`: Parameters specific to the action type

### Router

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
        - type: "filter"
          params:
            field: "revenue"
            operator: ">"
            value: 1000
      destinations:
        - destination_type: "webhook"
          config:
            url: "https://api.example.com/webhook"
            headers:
              Authorization: "Bearer ${ENV_SECRET_KEY}"
              Content-Type: "application/json"
```

- `routes`: List of data routing configurations
  - `id`: Unique identifier for the route
  - `name`: Human-readable name
  - `enabled`: Whether this route is enabled
  - `source`: Data source configuration
    - `connector_id`: ID of the connector to use
    - `data_spec`: Specification for what data to retrieve
  - `transformations`: Optional list of transformations to apply to the data
  - `destinations`: List of destinations to send the data to
    - `destination_type`: Type of destination (webhook, database, file, etc.)
    - `config`: Configuration specific to the destination type

### Security

```yaml
security:
  api_keys:
    enabled: true
    keys:
      - name: "admin"
        key: "${API_KEY}"
        roles: ["admin"]
      - name: "read-only"
        key: "${READ_ONLY_KEY}"
        roles: ["reader"]
  
  jwt:
    enabled: false
    secret: "${JWT_SECRET}"
    expiration_hours: 24
```

- `api_keys`: Configuration for API key authentication
  - `enabled`: Whether API key authentication is enabled
  - `keys`: List of API keys
    - `name`: Name of the key
    - `key`: The actual key (should use environment variable)
    - `roles`: List of roles for this key
- `jwt`: Configuration for JWT authentication
  - `enabled`: Whether JWT authentication is enabled
  - `secret`: Secret used to sign JWTs (should use environment variable)
  - `expiration_hours`: How long JWTs are valid for

## Validation

Muxly validates your configuration file on startup and will report any errors. Common validation errors include:

- Missing required fields
- Invalid values for fields
- References to non-existent connectors
- Invalid authentication configurations

To validate your configuration without starting the application, use:

```bash
muxly validate --config /path/to/your/config.yml
```

## Example Configurations

### Minimal Configuration

```yaml
general:
  log_level: "info"
  api:
    port: 8080

connectors:
  - id: "sample-ga4"
    name: "GA4 Sample"
    connector_type: "ga4"
    enabled: true
    auth:
      auth_type: "service_account"
      params:
        service_account_json: "${GA4_SERVICE_ACCOUNT}"
    connection:
      property_id: "123456789"

router:
  routes:
    - id: "ga4-to-file"
      name: "GA4 to File Export"
      enabled: true
      source:
        connector_id: "sample-ga4"
        data_spec:
          dateRanges:
            - startDate: "7daysAgo"
              endDate: "yesterday"
          dimensions:
            - name: "date"
            - name: "country"
          metrics:
            - name: "activeUsers"
      destinations:
        - destination_type: "file"
          config:
            path: "./exports/ga4_export.csv"
            format: "csv"
```

### Complete Example

See the [examples directory](../examples) for complete configuration examples.

## Next Steps

- Learn about [Connectors](./connectors.md)
- Explore [Scheduler](./scheduler.md) capabilities 
- Understand [Router](./router.md) configuration 