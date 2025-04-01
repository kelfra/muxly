# Connectors User Guide

This guide provides information about the available connectors in Muxly and how to configure them.

## Available Connectors

Muxly supports the following data source connectors:

1. **BigQuery**: Connect to Google BigQuery to extract data via SQL queries
2. **Google Analytics 4**: Connect to GA4 to retrieve analytics metrics and dimensions
3. **HubSpot**: Connect to HubSpot CRM to access contacts, companies, deals, and more
4. **Custom Plugins**: Extend Muxly with your own custom connectors

## General Configuration

All connectors share a common configuration structure:

```yaml
connectors:
  - id: "my-connector"
    name: "My Connector"
    connector_type: "bigquery"  # One of: bigquery, ga4, hubspot, plugin
    enabled: true
    auth:
      auth_type: "service_account"  # Depends on connector type
      params:
        # Authentication parameters
    connection:
      # Connection-specific settings
    rate_limit:
      max_requests: 100
      period_seconds: 60
      auto_adjust: true
    retry:
      max_attempts: 3
      initial_backoff_ms: 1000
      max_backoff_ms: 30000
      backoff_multiplier: 2.0
      retryable_errors: ["RATE_LIMIT", "SERVER_ERROR"]
```

## BigQuery Connector

The BigQuery connector allows you to extract data from Google BigQuery using SQL queries.

### Authentication

BigQuery supports two authentication methods:

1. **Service Account**:
```yaml
auth:
  auth_type: "service_account"
  params:
    service_account_json: {
      "type": "service_account",
      "project_id": "your-project-id",
      "private_key_id": "key-id",
      "private_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n",
      "client_email": "your-service-account@your-project-id.iam.gserviceaccount.com",
      "client_id": "client-id",
      "auth_uri": "https://accounts.google.com/o/oauth2/auth",
      "token_uri": "https://oauth2.googleapis.com/token",
      "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
      "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/your-service-account%40your-project-id.iam.gserviceaccount.com"
    }
```

2. **Application Default Credentials**:
```yaml
auth:
  auth_type: "application_default"
  params: {}
```

### Connection Settings

```yaml
connection:
  project_id: "your-project-id"
  dataset_id: "your_dataset"  # Optional
  location: "US"  # Optional
  max_results: 1000  # Optional
```

### Usage Example

To fetch data from BigQuery, use the connector's `fetch_data` method with a SQL query:

```json
{
  "query": "SELECT * FROM `your_dataset.your_table` WHERE date = @date",
  "parameters": [
    {
      "name": "date",
      "type": "STRING",
      "value": "2023-01-01"
    }
  ]
}
```

## Google Analytics 4 (GA4) Connector

The GA4 connector allows you to fetch analytics data from Google Analytics 4 properties.

### Authentication

GA4 supports two authentication methods:

1. **OAuth**:
```yaml
auth:
  auth_type: "oauth"
  params:
    client_id: "your-client-id.apps.googleusercontent.com"
    client_secret: "your-client-secret"
    auth_code: "4/P7q7W91a-oMsCeLvIaQm6bTrgtp7"  # Initial auth only
    refresh_token: "1//xEoDL4iW3cxlI7yDbSRFYNG01kVKM2C-259HOF2aQbI"  # If already authenticated
```

2. **Service Account** (simplified):
```yaml
auth:
  auth_type: "service_account"
  params:
    service_account_json: { ... }  # Service account JSON
```

### Connection Settings

```yaml
connection:
  property_id: "123456789"  # GA4 property ID
  default_date_range_days: 30  # Optional
  sampling_level: "DEFAULT"  # Optional, one of: DEFAULT, SMALL, LARGE
  currency: "USD"  # Optional
```

### Usage Example

To fetch data from GA4, use the connector's `fetch_data` method with dimensions and metrics:

```json
{
  "dateRanges": [
    {
      "startDate": "30daysAgo",
      "endDate": "yesterday"
    }
  ],
  "dimensions": [
    {"name": "date"},
    {"name": "country"}
  ],
  "metrics": [
    {"name": "activeUsers"},
    {"name": "sessions"},
    {"name": "conversions"}
  ],
  "limit": 10000
}
```

## HubSpot Connector

The HubSpot connector allows you to access CRM data from HubSpot.

### Authentication

HubSpot supports two authentication methods:

1. **OAuth**:
```yaml
auth:
  auth_type: "oauth"
  params:
    client_id: "your-client-id"
    client_secret: "your-client-secret"
    auth_code: "authorization-code"  # Initial auth only
    refresh_token: "refresh-token"  # If already authenticated
```

2. **API Key**:
```yaml
auth:
  auth_type: "api_key"
  params:
    api_key: "your-api-key"
```

### Connection Settings

```yaml
connection:
  api_version: "v3"  # Optional
  batch_size: 100  # Optional
  include_archived: false  # Optional
  properties:  # Optional
    contacts:
      - "firstname"
      - "lastname"
      - "email"
    companies:
      - "name"
      - "domain"
      - "industry"
```

### Usage Example

To fetch data from HubSpot, use the connector's `fetch_data` method with object type:

```json
{
  "objectType": "contacts",
  "properties": ["firstname", "lastname", "email", "hs_lead_status"],
  "limit": 100,
  "filter": {
    "propertyName": "createdate",
    "operator": "GTE",
    "value": "2023-01-01T00:00:00Z"
  }
}
```

## Custom Plugin Connectors

You can extend Muxly with custom connector plugins. To use a plugin connector:

```yaml
connectors:
  - id: "my-custom-connector"
    name: "My Custom Connector"
    connector_type: "plugin"
    enabled: true
    auth:
      auth_type: "custom"
      params:
        # Custom authentication parameters
    connection:
      plugin_path: "/path/to/your/plugin.so"  # or .dll, .dylib
      config:
        # Plugin-specific configuration
```

See the Developer Guide for information on creating custom connector plugins. 