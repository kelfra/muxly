# Router Destinations

The Router module in Muxly provides various destination types that can be used to route data from connectors to external systems. This guide covers the available destinations and their configuration options.

## Available Destinations

Muxly supports the following destinations:

1. **Database Destination**: Sends data to a relational database
2. **Email Destination**: Sends email notifications with data
3. **File Destination**: Writes data to local files
4. **Prometheus Destination**: Publishes metrics to Prometheus
5. **S3 Destination**: Stores data in Amazon S3 buckets
6. **Slack Destination**: Sends notifications to Slack channels
7. **Webhook Destination**: Sends data to HTTP endpoints

## Database Destination

The Database destination allows you to store data in various SQL databases.

### Configuration Options

```json
{
  "destination_type": "database",
  "config": {
    "db_type": "postgres",
    "connection_string": "postgresql://username:password@localhost:5432/database",
    "table": "data_table",
    "schema": "public",
    "upsert_key": "id",
    "batch_size": 100,
    "create_table": true,
    "column_mapping": {
      "field_name": "column_name"
    }
  }
}
```

| Option | Description |
|--------|-------------|
| `db_type` | Database type (postgres, mysql, sqlite) |
| `connection_string` | Database connection string |
| `table` | Target table name |
| `schema` | Database schema (for PostgreSQL) |
| `upsert_key` | Field to use for upsert operations |
| `batch_size` | Number of records to insert in a batch |
| `create_table` | Whether to create the table if it doesn't exist |
| `column_mapping` | Mapping of JSON fields to database columns |

## Email Destination

The Email destination sends notifications via email when data is received.

### Configuration Options

```json
{
  "destination_type": "email",
  "config": {
    "smtp_host": "smtp.example.com",
    "smtp_port": 587,
    "smtp_username": "username",
    "smtp_password": "password",
    "use_tls": true,
    "from_email": "notifications@example.com",
    "from_name": "Muxly Notifications",
    "to_emails": ["recipient@example.com"],
    "cc_emails": ["manager@example.com"],
    "subject_template": "New data from {{connector_id}}",
    "template_variables": {
      "count": "metrics.count"
    }
  }
}
```

| Option | Description |
|--------|-------------|
| `smtp_host` | SMTP server hostname |
| `smtp_port` | SMTP server port |
| `smtp_username` | SMTP username |
| `smtp_password` | SMTP password |
| `use_tls` | Whether to use TLS |
| `from_email` | Sender email address |
| `from_name` | Sender name (optional) |
| `to_emails` | List of recipient email addresses |
| `cc_emails` | List of CC recipients (optional) |
| `bcc_emails` | List of BCC recipients (optional) |
| `subject_template` | Template for email subject |
| `body_template` | Template for email body (optional, HTML) |
| `template_variables` | Mapping of template variables to JSON paths |

## File Destination

The File destination writes data to local files.

### Configuration Options

```json
{
  "destination_type": "file",
  "config": {
    "output_dir": "/path/to/output",
    "file_prefix": "data_",
    "file_format": "json",
    "max_file_size_mb": 10,
    "rotation": "daily"
  }
}
```

| Option | Description |
|--------|-------------|
| `output_dir` | Directory to write files to |
| `file_prefix` | Prefix for output files |
| `file_format` | Output format (json, csv, parquet) |
| `max_file_size_mb` | Maximum file size before rotation |
| `rotation` | Rotation policy (none, daily, hourly, size) |
| `compress` | Whether to compress files |

## Prometheus Destination

The Prometheus destination publishes metrics to Prometheus.

### Configuration Options

```json
{
  "destination_type": "prometheus",
  "config": {
    "metrics": [
      {
        "name": "data_value",
        "type": "gauge",
        "description": "Value from data",
        "value_field": "value",
        "labels": {
          "source": "connector_id"
        }
      }
    ]
  }
}
```

| Option | Description |
|--------|-------------|
| `metrics` | List of metrics to publish |
| `metrics[].name` | Metric name |
| `metrics[].type` | Metric type (gauge, counter, histogram) |
| `metrics[].description` | Metric description |
| `metrics[].value_field` | Field in the data to use as the value |
| `metrics[].labels` | Labels to attach to the metric |

## S3 Destination

The S3 destination stores data in Amazon S3 buckets.

### Configuration Options

```json
{
  "destination_type": "s3",
  "config": {
    "bucket": "my-data-bucket",
    "key_prefix": "data/",
    "region": "us-west-2",
    "output_format": "json",
    "key_template": "{{connector_id}}/{{date}}/data.json",
    "credentials": {
      "access_key_id": "AKIAXXXXXXXX",
      "secret_access_key": "XXXXXXXXXX"
    }
  }
}
```

| Option | Description |
|--------|-------------|
| `bucket` | S3 bucket name |
| `key_prefix` | Prefix for S3 keys |
| `region` | AWS region |
| `output_format` | Output format (json, csv, parquet) |
| `key_template` | Template for S3 keys |
| `credentials` | AWS credentials (optional, uses instance role if not provided) |
| `content_type` | Content type for the uploaded files |

## Slack Destination

The Slack destination sends notifications to Slack channels.

### Configuration Options

```json
{
  "destination_type": "slack",
  "config": {
    "webhook_url": "https://hooks.slack.com/services/XXX/YYY/ZZZ",
    "channel": "#data-notifications",
    "username": "Muxly Bot",
    "icon": ":chart_with_upwards_trend:",
    "message_template": "New data received from {{connector_id}}",
    "include_data": true,
    "color": "#36a64f",
    "template_variables": {
      "count": "metrics.count"
    }
  }
}
```

| Option | Description |
|--------|-------------|
| `webhook_url` | Slack webhook URL |
| `channel` | Channel to send to (overrides webhook default) |
| `username` | Username to use (overrides webhook default) |
| `icon` | Icon emoji or URL (overrides webhook default) |
| `message_template` | Template for messages |
| `include_data` | Whether to include data as attachments |
| `color` | Color for attachment |
| `template_variables` | Mapping of template variables to JSON paths |

## Webhook Destination

The Webhook destination sends data to HTTP endpoints.

### Configuration Options

```json
{
  "destination_type": "webhook",
  "config": {
    "url": "https://api.example.com/webhook",
    "method": "POST",
    "headers": {
      "Content-Type": "application/json",
      "Authorization": "Bearer token"
    },
    "body_template": "{{data}}",
    "timeout_seconds": 30
  }
}
```

| Option | Description |
|--------|-------------|
| `url` | Webhook URL |
| `method` | HTTP method (GET, POST, PUT, PATCH) |
| `headers` | HTTP headers |
| `body_template` | Template for request body |
| `timeout_seconds` | Request timeout in seconds |
| `retry_count` | Number of retries on failure |

## Using Destinations in Router Configuration

To use destinations in your router configuration:

```json
{
  "id": "my-router",
  "name": "My Router",
  "enabled": true,
  "source": {
    "connector_id": "my-connector",
    "data_spec": {
      "query": "SELECT * FROM data"
    }
  },
  "destinations": [
    {
      "destination_type": "slack",
      "config": {
        "webhook_url": "https://hooks.slack.com/services/XXX/YYY/ZZZ",
        "message_template": "New data received from {{connector_id}}"
      }
    },
    {
      "destination_type": "s3",
      "config": {
        "bucket": "my-data-bucket",
        "key_prefix": "data/",
        "region": "us-west-2"
      }
    }
  ]
}
```

This configuration routes data from the connector "my-connector" to both Slack and S3 destinations. 