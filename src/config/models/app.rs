//! Application configuration model
//!
//! Contains general application settings like server port, logging, database, etc.

use serde::{Deserialize, Serialize};

/// General application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    #[serde(default = "default_name")]
    pub name: String,
    
    /// Server host
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Database connection URL
    #[serde(default = "default_database_url")]
    pub database_url: String,
    
    /// Maximum connection pool size
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    
    /// Enable CORS
    #[serde(default = "default_cors_enabled")]
    pub cors_enabled: bool,
    
    /// CORS allowed origins
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
    
    /// Enable API authentication
    #[serde(default = "default_auth_enabled")]
    pub auth_enabled: bool,
    
    /// JWT secret key
    #[serde(default)]
    pub jwt_secret: Option<String>,
    
    /// Keycloak URL for authentication
    #[serde(default)]
    pub keycloak_url: Option<String>,
    
    /// Keycloak realm
    #[serde(default)]
    pub keycloak_realm: Option<String>,
    
    /// Keycloak client ID
    #[serde(default)]
    pub keycloak_client_id: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            host: default_host(),
            port: default_port(),
            log_level: default_log_level(),
            database_url: default_database_url(),
            max_connections: default_max_connections(),
            connection_timeout: default_connection_timeout(),
            cors_enabled: default_cors_enabled(),
            cors_origins: default_cors_origins(),
            auth_enabled: default_auth_enabled(),
            jwt_secret: None,
            keycloak_url: None,
            keycloak_realm: None,
            keycloak_client_id: None,
        }
    }
}

fn default_name() -> String {
    "Muxly".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_database_url() -> String {
    "sqlite:muxly.db".to_string()
}

fn default_max_connections() -> u32 {
    10
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_cors_enabled() -> bool {
    false
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}

fn default_auth_enabled() -> bool {
    false
} 