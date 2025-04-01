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

## Quick Start

### Using Docker (Recommended)

The fastest way to get started with Muxly is using Docker:

```bash
# Clone the repository
git clone https://github.com/kelfra/muxly.git
cd muxly

# Run the installation script
./scripts/install/install.sh
```

### Manual Installation

If you prefer to install Muxly manually:

```bash
# Clone the repository
git clone https://github.com/kelfra/muxly.git
cd muxly

# Create necessary directories
mkdir -p data/output

# Build and run with Docker Compose
docker-compose build
docker-compose up -d
```

## Configuration

Muxly uses TOML configuration files. A sample configuration is provided at `config/muxly.toml.example`.

### Main Configuration Sections

- **Server**: API server settings
- **Database**: Database configuration (SQLite or PostgreSQL)
- **Connectors**: Data source configurations
- **Outputs**: Data destination configurations

### Example Configuration

```toml
[server]
host = "0.0.0.0"
port = 3000

# SQLite database (default)
[database]
type = "sqlite"

[database.sqlite]
path = "./data/muxly.db"

# Sample connector
[[connectors]]
id = "internal_api"
name = "Internal API Connector"
connector_type = "internal_api"
enabled = true

# Output configuration
[[outputs]]
type = "file"
enabled = true
```

See the [full configuration example](config/muxly.toml.example) for more details.

## Core Connectors

Muxly supports the following connectors:

### BigQuery Connector

Connect to Google BigQuery to fetch data from SQL queries.

```toml
[[connectors]]
id = "bigquery"
name = "BigQuery Connector"
connector_type = "bigquery"
```

### Google Analytics 4 Connector

Fetch metrics and dimensions from Google Analytics 4 properties.

```toml
[[connectors]]
id = "ga4"
name = "GA4 Connector"
connector_type = "ga4"
```

### HubSpot Connector

Sync contacts, companies, and deals from HubSpot.

```toml
[[connectors]]
id = "hubspot"
name = "HubSpot Connector"
connector_type = "hubspot"
```

### Internal API Connector

Connect to your own internal APIs to fetch data.

```toml
[[connectors]]
id = "internal_api"
name = "Internal API Connector"
connector_type = "internal_api"
```

## Output Destinations

Muxly can route data to various destinations:

- **File**: Save data to JSON, CSV, or JSONL files
- **Prometheus**: Expose metrics for scraping by Prometheus
- **Webhook**: Send data to custom HTTP endpoints

## Self-Diagnostics

Muxly includes a comprehensive self-diagnostics system:

- **Health Checks**: API endpoint at `/health` for system status
- **Connection Testing**: Test connectors before setting up full data pipelines
- **Troubleshooting**: Detailed error messages and recovery suggestions

## Development

### Prerequisites

- Rust 1.60 or later
- SQLite or PostgreSQL
- Docker and Docker Compose (for containerized deployment)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/kelfra/muxly.git
cd muxly

# Build the project
cargo build --release

# Run the service
./target/release/muxly
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT License](LICENSE)
