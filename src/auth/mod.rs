pub mod keycloak;
mod credentials;
mod tokens;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use crate::error::MuxlyError;

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

// Re-export auth types
pub use keycloak::{KeycloakAuth, KeycloakConfig, UserInfo};

// State for authentication
#[derive(Clone)]
pub struct AuthState {
    pub keycloak: Arc<KeycloakAuth>,
}

// Extractor for authenticated users
pub struct AuthUser(pub UserInfo);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AuthState: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                let error = MuxlyError::Authentication("Missing or invalid authorization header".to_string());
                error.into_response()
            })?;

        // Extract auth state
        let auth_state = AuthState::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                let error = MuxlyError::Internal("Auth state not configured".to_string());
                error.into_response()
            })?;

        // Verify the token
        let user = auth_state
            .keycloak
            .verify_token(bearer.token())
            .await
            .map_err(|err| err.into_response())?;

        Ok(AuthUser(user))
    }
}

// Helper function to check if user has a role
pub fn has_role(user: &UserInfo, role: &str) -> bool {
    user.roles.iter().any(|r| r == role)
}

// Helper function to require a role
pub fn require_role(user: &UserInfo, role: &str) -> Result<(), MuxlyError> {
    if has_role(user, role) {
        Ok(())
    } else {
        Err(MuxlyError::Authentication(format!("Missing required role: {}", role)))
    }
} 