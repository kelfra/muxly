# Muxly

A lightweight, cross-platform service written in Rust that enables SaaS companies to collect, unify, and route product metrics and data from disparate sources. Muxly connects internal APIs with third-party services like BigQuery, GA4, and HubSpot, transforming all data into a consistent JSON format before routing it to desired destinations.

## Features

- **Zero-Friction Integration**: Simple setup with minimal configuration requirements
- **Smart Defaults**: Sensible defaults for all settings based on best practices
- **Self-Diagnosing**: Built-in health checks and connection testers
- **Template-First Design**: Pre-built configurations for all supported integrations
- **Simplified Deployment**: One-command installation with Docker

### Core Capabilities

- **Collect** internal metrics via a RESTful API
- **Connect** to third-party services (BigQuery, GA4, HubSpot)
- **Transform** all data into consistent JSON format
- **Route** data to various destinations (Prometheus, files, webhooks)
- **Schedule** data syncs using cron, webhooks, or manual API triggers

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

## Implemented Modules

### Scheduler Module

Muxly includes a robust scheduler system with three different scheduling mechanisms:

### Cron Scheduler

The cron scheduler allows you to schedule tasks to run at specified intervals using cron expressions. Features include:

- Define jobs using standard cron expressions
- Configure jobs with customizable timezone support
- Enable/disable jobs dynamically
- Track job execution history
- Configurable catch-up behavior for missed jobs

Example:

```rust
// Schedule a job to run every minute
cron_scheduler.add_job(
    "minute-job",
    "0 * * * * *", 
    Box::new(|| {
        // Your job handler
        Box::pin(async { Ok(()) })
    }),
).await?;
```

### Webhook Scheduler

The webhook scheduler enables triggering actions via HTTP webhooks. Features include:

- Register custom webhook handlers for different endpoints
- Secure webhooks with HMAC signature validation
- Enable/disable webhook endpoints dynamically
- JSON payload support

Example request to trigger a webhook:

```
POST /webhooks/my-custom-webhook
X-Webhook-Signature: 123abc...
Content-Type: application/json

{
  "event": "user.created",
  "data": { ... }
}
```

### API Scheduler

The API scheduler provides a REST API for managing and triggering jobs. Features include:

- Create, view, enable/disable, and trigger jobs via REST API
- Job execution history and status tracking
- Parameterized job execution
- Asynchronous job execution with status polling

API endpoints:

- `GET /jobs` - List all jobs
- `GET /jobs/:id` - Get job details
- `POST /jobs/:id/run` - Run a job
- `POST /jobs/:id/enable` - Enable a job
- `POST /jobs/:id/disable` - Disable a job
- `GET /executions/:id` - Get job execution status

### Configuration Module

Muxly includes a robust configuration system that provides flexible management of application settings:

### Features

- **Multiple Formats**: Support for YAML, JSON, and TOML configuration files
- **Environment Variables**: Override configuration via environment variables
- **Validation**: Comprehensive validation with detailed error reporting
- **Schema Support**: JSON Schema for documentation and validation
- **Defaults**: Sensible default values for all settings
- **Strongly Typed**: Type-safe configuration with Rust structs

### Configuration Structure

The configuration is organized into these main sections:

```yaml
# Main sections
app:       # General application settings
connectors: # Data source configurations
router:    # Data destination configurations
scheduler: # Job scheduling settings
```

### Loading Configuration

Configuration is loaded in this order of precedence:

1. Default values
2. Configuration files
3. Environment variables

Example usage:

```rust
// Load configuration
let config_path = Some("config/development.yaml");
let config = config::init_config(config_path).await?;

// Access configuration values
let port = config.app.port;
let database_url = &config.app.database_url;
```

### Environment Variables

Environment variables can override any configuration value using the format:

```
MUXLY_SECTION_KEY=value
```

For example:
- `MUXLY_PORT=8080` overrides the app.port setting
- `MUXLY_LOG_LEVEL=debug` sets the logging level
- `MUXLY_DATABASE_URL=postgres://...` overrides the database connection

### Connectors Module

Muxly provides integration with several data sources:

#### Supported Connectors

- **BigQuery**: Connect to Google BigQuery to extract data via SQL queries
- **Google Analytics 4**: Connect to GA4 to retrieve analytics metrics and dimensions
- **HubSpot**: Connect to HubSpot CRM to access contacts, companies, deals, and more
- **Custom Plugins**: Extend Muxly with your own custom connectors

#### Features

- Multiple authentication methods (OAuth, API keys, service accounts)
- Robust error handling and rate limiting
- Data transformation to consistent JSON format
- Customizable data fetching options

## Quick Start

### Using Docker (Recommended)

The fastest way to get started with Muxly is using Docker:

```