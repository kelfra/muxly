# Muxly

A lightweight, cross-platform service written in Rust that enables SaaS companies to collect, unify, and route product metrics and data from disparate sources. Muxly connects internal APIs with third-party services like BigQuery, GA4, and HubSpot, transforming all data into a consistent JSON format before routing it to desired destinations.

## Project Status

✅ **Completed Modules**:
- **Configuration Module**: Robust configuration system with multiple formats, validation, and smart defaults
- **Scheduler Module**: Unified scheduling system with cron, webhook, and API-based triggers
- **Connectors Module**: Implementations for BigQuery, GA4, HubSpot, plus a plugin system
- **Router Module Destinations**: Multiple destination types including Database, Email, File, Prometheus, S3, Slack, and Webhook

🚧 **In Progress**:
- **Router Module Rules**: Conditional routing and transformation pipelines (Coming in next release)

## Features

### Core Capabilities
- **Collect** internal metrics via a RESTful API
- **Connect** to third-party services (BigQuery, GA4, HubSpot)
- **Transform** all data into consistent JSON format
- **Schedule** data syncs using cron, webhooks, or manual API triggers
- **Route** data to various destinations (Databases, Email, Files, Prometheus, S3, Slack, Webhooks)

### Design Principles
- **Zero-Friction Integration**: Simple setup with minimal configuration requirements
- **Smart Defaults**: Sensible defaults for all settings based on best practices
- **Self-Diagnosing**: Built-in health checks and connection testers
- **Template-First Design**: Pre-built configurations for all supported integrations
- **Simplified Deployment**: One-command installation with Docker

## Quick Start

### Docker Installation (Recommended)

The fastest way to get started with Muxly is using Docker:

```bash
# Pull the Docker image
docker pull muxly/muxly:latest

# Run with a mounted config directory
docker run -p 3000:3000 -v $(pwd)/config:/var/lib/muxly/config muxly/muxly:latest
```

Or using Docker Compose:

```bash
# Create a docker-compose.yml file
cat > docker-compose.yml << EOF
version: '3.8'

services:
  muxly:
    image: muxly/muxly:latest
    ports:
      - "3000:3000"
    volumes:
      - ./config:/var/lib/muxly/config
      - ./data:/var/lib/muxly/data
    environment:
      - RUST_LOG=info
EOF

# Start the container
docker-compose up -d
```

### Building from Source

If you prefer to build from source:

```bash
# Clone the repository
git clone https://github.com/kelfra/muxly.git
cd muxly

# Build the project
cargo build --release

# Run the server
./target/release/muxly
```

## Configuration Examples

### Minimal Configuration

```yaml
# config/config.yaml
app:
  name: Muxly
  host: 0.0.0.0
  port: 3000
  log_level: info

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

## Architecture Overview

Muxly is built around four main components:

1. **Configuration System**: Handles settings from files and environment variables with validation and smart defaults
2. **Connector System**: Manages connections to various data sources with proper authentication and data retrieval
3. **Scheduler System**: Orchestrates when and how data is collected using cron, webhooks, or API triggers
4. **Router System**: Determines where data gets sent after collection and transformation (coming soon)

The modular architecture allows you to pick and choose which components you need, while the plugin system enables custom extensions for specific use cases.

## Implemented Modules

### Scheduler Module

Muxly includes a robust scheduler system with three different scheduling mechanisms:

#### Cron Scheduler

The cron scheduler allows you to schedule tasks to run at specified intervals using cron expressions. Features include:

- Define jobs using standard cron expressions
- Configure jobs with customizable timezone support
- Enable/disable jobs dynamically
- Track job execution history
- Configurable catch-up behavior for missed jobs

#### Webhook Scheduler

The webhook scheduler enables triggering actions via HTTP webhooks. Features include:

- Register custom webhook handlers for different endpoints
- Secure webhooks with HMAC signature validation
- Enable/disable webhook endpoints dynamically
- JSON payload support

#### API Scheduler

The API scheduler provides a REST API for managing and triggering jobs. Features include:

- Create, view, enable/disable, and trigger jobs via REST API
- Job execution history and status tracking
- Parameterized job execution
- Asynchronous job execution with status polling

### Configuration Module

Muxly includes a robust configuration system that provides flexible management of application settings:

#### Features

- **Multiple Formats**: Support for YAML, JSON, and TOML configuration files
- **Environment Variables**: Override configuration via environment variables
- **Validation**: Comprehensive validation with detailed error reporting
- **Schema Support**: JSON Schema for documentation and validation
- **Defaults**: Sensible default values for all settings
- **Strongly Typed**: Type-safe configuration with Rust structs

### Connectors Module

Muxly provides integration with several data sources:

#### Supported Connectors

- **BigQuery**: Connect to Google BigQuery to extract data via SQL queries
- **Google Analytics 4**: Connect to GA4 to retrieve analytics metrics and dimensions
- **HubSpot**: Connect to HubSpot CRM to access contacts, companies, deals, and more
- **Custom API**: Connect to any RESTful API endpoint to fetch custom metrics
- **Custom Plugins**: Extend Muxly with your own custom connectors

#### Features

- Multiple authentication methods (OAuth, API keys, service accounts)
- Robust error handling and rate limiting
- Data transformation to consistent JSON format
- Customizable data fetching options

## Documentation

### User Documentation

For users of Muxly, the following guides are available:

- [**Getting Started**](docs/user-guide/README.md) - Overview of the user documentation
- [**Configuration Guide**](docs/user-guide/configuration.md) - How to configure Muxly
- [**Connectors Guide**](docs/user-guide/connectors.md) - Working with data source connectors
- [**Scheduler Guide**](docs/user-guide/scheduler.md) - Automating data tasks
- [**Router Guide**](docs/user-guide/router.md) - Defining data routes

### Developer Documentation

For developers working on Muxly, the following documentation is available:

- [**Project Status**](docs/project_status.md) - Current project status and implementation progress
- [**Implementation Plan**](docs/development/implementation_plan.md) - Detailed implementation plan
- [**Connectors Implementation**](docs/development/connectors.md) - Details of connectors module implementation
- [**Known Issues**](docs/development/known_issues.md) - Current known issues and limitations

## API Reference

Muxly provides a RESTful API for managing connectors, schedulers, and data routes:

- `GET /health` - Health check endpoint
- `GET /v1/connectors` - List all connectors
- `POST /v1/connectors` - Create a new connector
- `GET /v1/jobs` - List all scheduled jobs
- `POST /v1/jobs/{id}/run` - Trigger a job manually

For a complete API reference, see the [API documentation](docs/user-guide/api-reference.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
