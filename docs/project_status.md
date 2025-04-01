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
- **Authenticate** and authorize users with role-based access control

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
- **Custom API Connector**: Flexible connector for custom API integrations
- **Plugin System**: Support for custom connectors through a dynamic loading mechanism

Each connector includes:
- Comprehensive authentication methods (OAuth, API keys, service accounts)
- Robust error handling and rate-limiting support
- Data transformation to convert from source format to Muxly's consistent JSON format
- Configuration templates for easy setup
- Connection testing capabilities

#### 4. Router Module
We have successfully implemented the Router module with destinations, rules, and routing logic:

- **Destination Base**: Common traits, factories, and utilities for all destinations
- **Database Destination**: For storing data in SQL databases with configurable mappings
- **Email Destination**: For sending email notifications with templated content
- **File Destination**: For writing data to local files in various formats
- **Prometheus Destination**: For exposing metrics to Prometheus monitoring
- **S3 Destination**: For storing data in Amazon S3 and compatible cloud storage
- **Slack Destination**: For sending notifications to Slack channels
- **Webhook Destination**: For sending data to HTTP endpoints

- **Routing Rules and Conditions**: Dynamic routing based on data content
- **Transformations**: Data modification before routing
- **Error Handling**: Configurable error handling strategies

#### 5. Authentication System
We have implemented a comprehensive authentication and authorization system:

- **Local Authentication**: Username/password authentication with secure password hashing
- **Keycloak Integration**: SSO with Keycloak identity provider
- **Role-Based Access Control**: Configurable roles and permissions
- **Token Management**: JWT token generation, validation, and refreshing
- **Secure Credential Storage**: Encrypted storage for sensitive credentials

#### 6. Database Migrations
We have implemented database schema migrations for all components:

- **Core Tables**: Settings, connectors, transformations
- **Router Tables**: Routes, destinations, rules, transformations
- **Scheduler Tables**: Jobs, executions, webhooks
- **Authentication Tables**: Users, roles, permissions, sessions

#### 7. Documentation
We have created comprehensive documentation for both developers and users:

- **Developer Documentation**:
  - Implementation details for each module
  - Known issues and future improvements
  - Implementation plan for upcoming features
  - Database schema documentation

- **User Documentation**:
  - Configuration guide with examples
  - Authentication guide
  - Connectors usage and authentication
  - Scheduler configuration and job setup
  - Router configuration and destinations
  - Router rules and conditions
  - API reference
  - Troubleshooting guides

### In Progress Components

#### 1. API Documentation Completion
We are finalizing the API documentation for all endpoints:

- Ensuring all endpoints are documented in the OpenAPI specification
- Adding examples and use cases for each endpoint
- Validating request and response schemas

#### 2. User Guide Enhancement
We are enhancing the user guides with:

- More detailed examples
- Common use case walkthroughs
- Best practices for each module
- Troubleshooting guidance

## Known Issues

- There are some edge cases in connector error handling that need improvement
- The main application has references to components that will be enhanced in future PRs
- Some dependencies in Cargo.toml may need updates to maintain compatibility with the latest Rust version

## Implementation Timeline

### Phase 1: Core Infrastructure (2-3 weeks) - COMPLETED
- ✅ Configuration module
- ✅ Scheduler module implementation
- ✅ Default configurations

### Phase 2: Connectors (3-4 weeks) - COMPLETED
- ✅ Connector trait and base
- ✅ BigQuery integration
- ✅ GA4 integration
- ✅ HubSpot integration
- ✅ Custom API connector
- ✅ Plugin system for custom connectors
- ✅ Documentation for developers and users

### Phase 3: Router (2-3 weeks) - COMPLETED
- ✅ Router trait and base
- ✅ Output destinations
  - ✅ Database destination
  - ✅ Email destination
  - ✅ File destination
  - ✅ Prometheus destination
  - ✅ S3 storage destination
  - ✅ Slack notification destination
  - ✅ Webhook destination
- ✅ Routing rules and conditions

### Phase 4: Authentication & Documentation (1-2 weeks) - COMPLETED
- ✅ Authentication system implementation
- ✅ Keycloak integration
- ✅ Database migrations setup
- ✅ API documentation (OpenAPI/Swagger)
- ✅ Deployment scripts
- ✅ CI/CD integration
- ✅ User guide for all features

## Next Steps

With the core implementation complete, the next steps are:

1. Increase test coverage across all components
2. Optimize performance for large data volumes
3. Add more connectors for additional data sources
4. Enhance monitoring and observability features
5. Consider building a simple web UI for management 