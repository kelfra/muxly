mod keycloak;
mod credentials;
mod tokens;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Type of authentication (keycloak, oauth, api_key, etc.)
    pub auth_type: String,
    /// Authentication parameters
    pub params: serde_json::Value,
}

/// Credentials manager
pub struct CredentialsManager {
    /// Configuration for the credentials manager
    config: AuthConfig,
    /// Keycloak client for authentication
    keycloak_client: Option<Arc<keycloak::KeycloakClient>>,
}

impl CredentialsManager {
    /// Create a new credentials manager
    pub fn new(config: AuthConfig) -> Self {
        // Create the appropriate client based on the auth type
        let keycloak_client = if config.auth_type == "keycloak" {
            Some(Arc::new(keycloak::KeycloakClient::new(&config.params)))
        } else {
            None
        };
        
        Self {
            config,
            keycloak_client,
        }
    }
    
    /// Get the auth type
    pub fn auth_type(&self) -> &str {
        &self.config.auth_type
    }
    
    /// Get the authentication parameters
    pub fn params(&self) -> &serde_json::Value {
        &self.config.params
    }
    
    /// Get a credential by key
    pub fn get_credential(&self, key: &str) -> Option<String> {
        self.config.params.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
    
    /// Authenticate with Keycloak
    pub async fn authenticate_keycloak(&self, username: &str, password: &str) -> Result<tokens::AuthToken> {
        if let Some(client) = &self.keycloak_client {
            client.authenticate(username, password).await
        } else {
            Err(anyhow::anyhow!("Keycloak client not initialized"))
        }
    }
    
    /// Validate a token with Keycloak
    pub async fn validate_token(&self, token: &str) -> Result<bool> {
        if let Some(client) = &self.keycloak_client {
            client.validate_token(token).await
        } else {
            Err(anyhow::anyhow!("Keycloak client not initialized"))
        }
    }
    
    /// Refresh a token with Keycloak
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<tokens::AuthToken> {
        if let Some(client) = &self.keycloak_client {
            client.refresh_token(refresh_token).await
        } else {
            Err(anyhow::anyhow!("Keycloak client not initialized"))
        }
    }
    
    /// Get user information from token
    pub async fn get_user_info(&self, token: &str) -> Result<serde_json::Value> {
        if let Some(client) = &self.keycloak_client {
            client.get_user_info(token).await
        } else {
            Err(anyhow::anyhow!("Keycloak client not initialized"))
        }
    }
}

pub use credentials::Credentials;
pub use keycloak::KeycloakClient;
pub use tokens::AuthToken; 