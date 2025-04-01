use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::connectors::base::{
    AuthSettings, Connector, ConnectorData, ConnectorSettings, ConnectionStatus,
};

/// Type for connector creation function in dynamic libraries
pub type CreateConnectorFn = unsafe fn() -> *mut dyn Connector;

/// Plugin-specific connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConnectionSettings {
    /// Path to the plugin library
    pub plugin_path: String,
    /// Plugin-specific configuration
    pub config: HashMap<String, Value>,
}

/// A wrapper for dynamically loaded connector plugins
pub struct PluginConnector {
    /// Connector identifier
    id: String,
    /// Human-readable name
    name: String,
    /// Whether this connector is enabled
    enabled: bool,
    /// Authentication settings
    auth: AuthSettings,
    /// Connection settings specific to this plugin
    connection: PluginConnectionSettings,
    /// Underlying library
    _library: Option<Arc<Mutex<Library>>>,
    /// The actual connector implementation
    inner: Option<Box<dyn Connector>>,
}

impl PluginConnector {
    /// Create a new plugin connector
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            enabled: false,
            auth: AuthSettings {
                auth_type: String::new(),
                params: HashMap::new(),
            },
            connection: PluginConnectionSettings {
                plugin_path: String::new(),
                config: HashMap::new(),
            },
            _library: None,
            inner: None,
        }
    }
    
    /// Load a connector from a dynamic library
    async fn load_plugin(&mut self) -> Result<()> {
        let plugin_path = &self.connection.plugin_path;
        if plugin_path.is_empty() {
            return Err(anyhow!("Plugin path not specified"));
        }
        
        let path = Path::new(plugin_path);
        if !path.exists() {
            return Err(anyhow!("Plugin file not found: {}", plugin_path));
        }
        
        // Load the dynamic library
        // Using unsafe here because dynamic loading is inherently unsafe
        unsafe {
            // Attempt to load the library
            let library = Library::new(path)
                .map_err(|e| anyhow!("Failed to load plugin library: {}", e))?;
            
            // Look for the create_connector symbol
            let creator: Symbol<CreateConnectorFn> = library
                .get(b"create_connector")
                .map_err(|e| anyhow!("Failed to find create_connector symbol: {}", e))?;
            
            // Create the connector instance
            let connector_ptr = creator();
            if connector_ptr.is_null() {
                return Err(anyhow!("Plugin returned a null connector"));
            }
            
            // Convert the raw pointer to a Box
            let inner = Box::from_raw(connector_ptr);
            
            // Store both the library (to keep it loaded) and the connector
            self._library = Some(Arc::new(Mutex::new(library)));
            self.inner = Some(inner);
            
            Ok(())
        }
    }
}

#[async_trait]
impl Connector for PluginConnector {
    fn get_type(&self) -> &str {
        "plugin"
    }
    
    fn get_id(&self) -> &str {
        &self.id
    }
    
    async fn initialize(&mut self, settings: ConnectorSettings) -> Result<()> {
        // Extract settings
        self.id = settings.id;
        self.name = settings.name;
        self.enabled = settings.enabled;
        self.auth = settings.auth;
        
        // Parse connection settings
        let connection_value = serde_json::to_value(&settings.connection)?;
        self.connection = serde_json::from_value(connection_value)?;
        
        // Load the plugin
        self.load_plugin().await?;
        
        // Initialize the inner connector with the same settings
        if let Some(inner) = &mut self.inner {
            inner.initialize(settings.clone()).await?;
        }
        
        Ok(())
    }
    
    async fn test_connection(&self) -> Result<ConnectionStatus> {
        if !self.enabled {
            return Ok(ConnectionStatus::Disconnected);
        }
        
        // Delegate to the inner connector
        if let Some(inner) = &self.inner {
            inner.test_connection().await
        } else {
            Ok(ConnectionStatus::Error("Plugin not loaded".to_string()))
        }
    }
    
    async fn fetch_data(&self, params: Value) -> Result<ConnectorData> {
        // Delegate to the inner connector
        if let Some(inner) = &self.inner {
            inner.fetch_data(params).await
        } else {
            Err(anyhow!("Plugin not loaded"))
        }
    }
    
    async fn get_metadata(&self) -> Result<Value> {
        // Delegate to the inner connector
        if let Some(inner) = &self.inner {
            inner.get_metadata().await
        } else {
            Err(anyhow!("Plugin not loaded"))
        }
    }
    
    fn get_configuration_template(&self) -> Value {
        // If we have an inner connector, delegate to it for additional config fields
        let inner_template = if let Some(inner) = &self.inner {
            inner.get_configuration_template()
        } else {
            json!({})
        };
        
        // Merge with our plugin-specific config
        let plugin_template = json!({
            "connection": {
                "plugin_path": {
                    "type": "string",
                    "required": true,
                    "description": "Path to the connector plugin library (.so, .dll, .dylib)"
                },
                "config": {
                    "type": "object",
                    "required": false,
                    "description": "Plugin-specific configuration values"
                }
            }
        });
        
        // Merge templates, prioritizing plugin-specific fields
        serde_json::from_value(
            merge_json_objects(plugin_template, inner_template)
        ).unwrap_or(plugin_template)
    }
}

/// Plugin API that custom connectors must implement
pub mod api {
    use super::*;
    
    /// This macro helps plugin authors expose their connector properly
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
    
    /// This trait must be implemented by any plugin connector
    #[async_trait]
    pub trait PluginConnector: Connector + Send + Sync + 'static {
        /// Create a new instance of this connector
        fn new() -> Self where Self: Sized;
        
        /// Get the plugin version 
        fn get_version(&self) -> &str;
        
        /// Get the plugin's author
        fn get_author(&self) -> &str;
    }
    
    /// Example implementation for plugin authors
    pub struct ExamplePlugin {
        id: String,
        name: String,
        enabled: bool,
    }
    
    impl ExamplePlugin {
        pub fn new() -> Self {
            Self {
                id: String::new(),
                name: String::new(),
                enabled: false,
            }
        }
    }
    
    #[async_trait]
    impl PluginConnector for ExamplePlugin {
        fn new() -> Self {
            Self::new()
        }
        
        fn get_version(&self) -> &str {
            "1.0.0"
        }
        
        fn get_author(&self) -> &str {
            "Muxly Team"
        }
    }
    
    #[async_trait]
    impl Connector for ExamplePlugin {
        fn get_type(&self) -> &str {
            "example_plugin"
        }
        
        fn get_id(&self) -> &str {
            &self.id
        }
        
        async fn initialize(&mut self, settings: ConnectorSettings) -> Result<()> {
            self.id = settings.id;
            self.name = settings.name;
            self.enabled = settings.enabled;
            Ok(())
        }
        
        async fn test_connection(&self) -> Result<ConnectionStatus> {
            Ok(ConnectionStatus::Connected)
        }
        
        async fn fetch_data(&self, _params: Value) -> Result<ConnectorData> {
            Ok(ConnectorData {
                connector_id: self.id.clone(),
                timestamp: Utc::now(),
                data: json!({"message": "Hello from example plugin!"}),
                metadata: HashMap::new(),
            })
        }
        
        async fn get_metadata(&self) -> Result<Value> {
            Ok(json!({
                "type": "example_plugin",
                "version": self.get_version(),
                "author": self.get_author(),
            }))
        }
        
        fn get_configuration_template(&self) -> Value {
            json!({
                "connection": {
                    "example_setting": {
                        "type": "string",
                        "required": false,
                        "description": "An example setting for the plugin"
                    }
                }
            })
        }
    }
}

/// Helper function to merge two JSON objects
fn merge_json_objects(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Object(mut base_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                match base_map.get_mut(&key) {
                    Some(base_value) => {
                        *base_value = merge_json_objects(base_value.clone(), value);
                    }
                    None => {
                        base_map.insert(key, value);
                    }
                }
            }
            Value::Object(base_map)
        }
        (_, overlay) => overlay,
    }
}
