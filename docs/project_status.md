# Muxly Project Status Document

## Project Overview

Muxly is a lightweight, cross-platform service written in Rust that enables SaaS companies to collect, unify, and route product metrics and data from disparate sources. It connects internal APIs with third-party services like BigQuery, GA4, and HubSpot, transforming all data into a consistent JSON format before routing it to desired destinations.

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

## Project Structure

```
muxly/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── api/                    # API layer and controllers
│   ├── auth/                   # Authentication and credential management
│   ├── collector/              # Internal metrics collection
│   ├── connectors/             # Third-party integrations
│   │   ├── base.rs             # Connector trait and common utilities
│   │   ├── bigquery.rs         # BigQuery integration
│   │   ├── ga4.rs              # Google Analytics 4 integration
│   │   ├── hubspot.rs          # HubSpot integration
│   │   └── plugin.rs           # Plugin system for custom connectors
│   ├── config/                 # Configuration management
│   ├── scheduler/              # Job scheduling systems
│   │   ├── cron.rs             # Cron-based scheduling
│   │   ├── webhook.rs          # Webhook triggers
│   │   └── api.rs              # Manual API triggers
│   ├── router/                 # Output destination routing
│   ├── storage/                # Data persistence
│   └── transform/              # JSON transformation pipeline
├── migrations/                 # Database migrations
├── config/                     # Default configurations
├── docs/                       # Documentation
│   ├── development/            # Documentation for developers
│   └── user-guide/             # Documentation for users
└── scripts/                    # Deployment and utility scripts
```

## Implementation Progress

### Completed Components

#### 1. Scheduler Module
We have implemented a robust scheduling system with three complementary mechanisms:

- **Cron Scheduler**: Time-based scheduling using cron expressions with timezone support
- **Webhook Scheduler**: Event-driven scheduling triggered by HTTP webhooks with signature validation
- **API Scheduler**: RESTful API for programmatic job management

The scheduler module includes:
- A unified integration point via `SchedulerIntegration` class
- Configuration system for all scheduler types
- Proper error handling and logging
- Clear API documentation

#### 2. Configuration Module
We've built a comprehensive configuration system that supports:

- Multiple file formats (YAML, JSON, TOML)
- Environment variable overrides
- Validation with detailed error reporting
- JSON Schema for documentation and validation
- Default configuration values
- Strongly typed configuration models

#### 3. Connectors Module
We have successfully implemented the connectors module with support for various third-party data sources:

- **Base Connector Infrastructure**: Common traits, authentication handling, and error types
- **BigQuery Connector**: Integration with Google BigQuery for SQL-based data extraction
- **GA4 Connector**: Integration with Google Analytics 4 for metrics and dimensions
- **HubSpot Connector**: Integration with HubSpot CRM for contacts, companies, and deals
- **Plugin System**: Support for custom connectors through a dynamic loading mechanism

Each connector includes:
- Comprehensive authentication methods (OAuth, API keys, service accounts)
- Robust error handling and rate-limiting support
- Data transformation to convert from source format to Muxly's consistent JSON format
- Configuration templates for easy setup
- Connection testing capabilities

#### 4. Router Module Destinations
We have successfully implemented various destination types for the Router module:

- **Destination Base**: Common traits, factories, and utilities for all destinations
- **Database Destination**: For storing data in SQL databases with configurable mappings
- **Email Destination**: For sending email notifications with templated content
- **File Destination**: For writing data to local files in various formats
- **Prometheus Destination**: For exposing metrics to Prometheus monitoring
- **S3 Destination**: For storing data in Amazon S3 and compatible cloud storage
- **Slack Destination**: For sending notifications to Slack channels
- **Webhook Destination**: For sending data to HTTP endpoints

Each destination includes:
- Configuration options with sensible defaults
- Error handling and connection verification
- Support for both individual and batch data sending
- Templating capabilities for customized output formats

#### 5. Documentation
We have created comprehensive documentation for both developers and users:

- **Developer Documentation**:
  - Implementation details for each module
  - Known issues and future improvements
  - Implementation plan for upcoming features

- **User Documentation**:
  - Configuration guide with examples
  - Connectors usage and authentication
  - Scheduler configuration and job setup
  - Router configuration and destinations
  - Troubleshooting guides

### In Progress Components

#### 1. Router Module Routing Logic
We are currently working on the routing logic for the Router module:

- Conditional routing based on data content
- Dynamic destination selection
- Error handling and retry mechanisms
- Transformation pipelines for data processing

### Planned Components

Based on the implementation plan, our next priorities are:

#### 1. Complete Router Module
The remaining components of the router module will handle:

- Routing rules and conditions
- Error handling and delivery confirmation
- Transformation pipelines
- JSON path filtering

## Known Issues

- There are compile errors in several modules that reference API routes and handlers not yet implemented
- Some middleware implementations need updating to match the latest Axum version
- The main application has references to components that will be implemented in future PRs
- Dependencies in Cargo.toml need to be updated to maintain compatibility with the latest Rust version

## Implementation Timeline

### Phase 1: Core Infrastructure (2-3 weeks) - Completed
- ✅ Configuration module
- ✅ Scheduler module implementation
- ✅ Default configurations

### Phase 2: Connectors (3-4 weeks) - Completed
- ✅ Connector trait and base
- ✅ BigQuery integration
- ✅ GA4 integration
- ✅ HubSpot integration
- ✅ Plugin system for custom connectors
- ✅ Documentation for developers and users

### Phase 3: Router (2-3 weeks) - In Progress
- ✅ Router trait and base
- ✅ Output destinations
  - ✅ Database destination
  - ✅ Email destination
  - ✅ File destination
  - ✅ Prometheus destination
  - ✅ S3 storage destination
  - ✅ Slack notification destination
  - ✅ Webhook destination
- 🔄 Routing rules and conditions (In Progress)

### Phase 4: Documentation & Deployment (1-2 weeks)
- System documentation
- API documentation
- Deployment scripts

## Next Steps

The next phase will focus on implementing the Router Module for data destination handling, with the following key tasks:

1. Define the Router trait in base.rs
2. Implement the Prometheus output for metrics
3. Create the Webhook output for HTTP endpoints
4. Develop the File output for local storage
5. Build the S3 output for cloud storage
6. Implement the Database output for SQL persistence
7. Add Slack notifications for alerting 