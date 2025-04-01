# Connectors Module Implementation

This document provides details about the implementation of the Connectors Module in Muxly.

## Key Components

### Infrastructure

1. `src/connectors/mod.rs`
   - Exports all connector modules and public interfaces
   - Provides a unified entry point for connector functionality

2. `src/connectors/base.rs`
   - Defines the `Connector` trait that all connectors must implement
   - Implements common data structures: `ConnectorSettings`, `AuthSettings`, etc.
   - Provides the `create_connector` factory function to instantiate connectors

### Connector Implementations

3. `src/connectors/bigquery.rs`
   - Implements the BigQuery connector for Google BigQuery integration
   - Handles Google Cloud authentication and token management
   - Provides SQL query execution with parameter support
   - Transforms query results into a standardized format

4. `src/connectors/ga4.rs`
   - Implements the GA4 connector for Google Analytics 4 integration
   - Manages OAuth authentication flow with token refreshing
   - Provides report generation with dimensions and metrics
   - Handles data sampling and pagination

5. `src/connectors/hubspot.rs`
   - Implements the HubSpot connector for CRM data integration
   - Supports both OAuth and API key authentication
   - Provides access to contacts, companies, deals, and other CRM objects
   - Handles object associations and relationship management

### Plugin System

6. `src/connectors/plugin.rs`
   - Implements the plugin system for custom connector extensions
   - Provides dynamic loading of connector libraries at runtime
   - Defines a stable API for third-party developers
   - Includes an example plugin implementation

## Core Interfaces

### Connector Trait

All connectors implement the `Connector` trait, which provides a standard interface:

```rust
#[async_trait]
pub trait Connector {
    fn get_type(&self) -> &str;
    fn get_id(&self) -> &str;
    async fn initialize(&mut self, settings: ConnectorSettings) -> Result<()>;
    async fn test_connection(&self) -> Result<ConnectionStatus>;
    async fn fetch_data(&self, params: Value) -> Result<ConnectorData>;
    async fn get_metadata(&self) -> Result<Value>;
    fn get_configuration_template(&self) -> Value;
}
```

### ConnectorData Structure

Data returned from connectors is wrapped in a standardized `ConnectorData` structure:

```rust
pub struct ConnectorData {
    pub connector_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: Value,
    pub metadata: HashMap<String, Value>,
}
```

### Authentication

The system supports multiple authentication methods through the `AuthSettings` structure:

```rust
pub struct AuthSettings {
    pub auth_type: String,
    pub params: HashMap<String, Value>,
}
```

Supported authentication types include:
- OAuth (for GA4 and HubSpot)
- Service Account (for Google Cloud services)
- API Key (for simpler authentication flows)
- Custom Authentication (via plugin extensions)

## Plugin System

The plugin system enables custom connectors through a stable API:

1. Plugin authors implement the `PluginConnector` trait:
```rust
#[async_trait]
pub trait PluginConnector: Connector + Send + Sync + 'static {
    fn new() -> Self where Self: Sized;
    fn get_version(&self) -> &str;
    fn get_author(&self) -> &str;
}
```

2. The `export_connector!` macro exposes the connector to the host application:
```rust
#[macro_export]
macro_rules! export_connector {
    ($connector_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_connector() -> *mut dyn Connector {
            let connector = Box::new(<$connector_type>::new());
            Box::into_raw(connector)
        }
    };
}
```

3. The `PluginConnector` wrapper loads and manages the plugin lifecycle.

## Extensibility

The system is designed to be extensible:

- New connector types can be added with minimal changes to the core codebase
- The plugin system allows for third-party extensions without modifying the source
- The connector interface is stable and backward-compatible for future additions

## Dependencies

The Connectors Module relies on several dependencies:

- `gcp_auth`: For Google Cloud authentication
- `libloading`: For the plugin system dynamic loading
- `oauth2`: For OAuth authentication flows
- `reqwest`: For making HTTP requests
- `tokio`: For asynchronous processing
- `serde`: For serialization and deserialization
- `chrono`: For datetime handling

## Future Enhancements

Planned enhancements for the Connectors Module:

1. Additional connector types:
   - Salesforce
   - Stripe
   - Postgres/MySQL direct connections
   - REST API generic connector

2. Performance improvements:
   - Connection pooling
   - Caching layer
   - Parallel data fetching

3. Security enhancements:
   - Enhanced credential management
   - Fine-grained access control
   - Auditing and logging 