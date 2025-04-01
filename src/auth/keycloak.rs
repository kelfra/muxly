use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};
use axum::{
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
    extract::FromRef,
};
use std::sync::Arc;
use std::collections::HashSet;
use futures::future::BoxFuture;
use keycloak::{KeycloakError, KeycloakAdmin};
use std::future::{ready, Ready};
use tower_http::auth::AsyncAuthorizeRequest;
use crate::error::{MuxlyError, Result};
use std::future::Future;
use std::pin::Pin;

use crate::auth::tokens::AuthToken;

/// Keycloak configuration
#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakConfig {
    /// URL of the Keycloak server
    pub server_url: String,
    /// Realm to use
    pub realm: String,
    /// Client ID
    pub client_id: String,
    /// Client secret (optional, for confidential clients)
    pub client_secret: String,
    pub required_role: Option<String>,
}

/// User information extracted from JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub roles: Vec<String>,
}

/// Keycloak auth service
#[derive(Debug, Clone)]
pub struct KeycloakAuth {
    /// HTTP client
    client: Arc<KeycloakAdmin>,
    /// Configuration for the Keycloak client
    config: KeycloakConfig,
    required_roles: HashSet<String>,
}

impl KeycloakAuth {
    /// Create a new Keycloak auth service
    pub async fn new(config: KeycloakConfig) -> Result<Self> {
        let client = KeycloakAdmin::new(&config.server_url, &config.realm)
            .auth_client(&config.client_id, &config.client_secret)
            .build()
            .map_err(|e| MuxlyError::Authentication(format!("Failed to create Keycloak client: {}", e)))?;
        
        // Parse required roles if any
        let required_roles = if let Some(role) = &config.required_role {
            let mut roles = HashSet::new();
            roles.insert(role.clone());
            roles
        } else {
            HashSet::new()
        };
        
        Ok(KeycloakAuth {
            client: Arc::new(client),
            config,
            required_roles,
        })
    }
    
    /// Verify a token and extract user info
    pub async fn verify_token(&self, token: &str) -> Result<UserInfo> {
        // Decode and validate the token
        let decoded = self.client.decode_token(token)
            .map_err(|e| MuxlyError::Authentication(format!("Token validation failed: {}", e)))?;
            
        // Get user ID
        let sub = decoded.subject()
            .ok_or_else(|| MuxlyError::Authentication("Subject not found in token".to_string()))?
            .to_string();
            
        // Extract roles
        let mut roles = Vec::new();
        if let Some(realm_access) = decoded.realm_access() {
            roles = realm_access.roles.clone();
        }
        
        // Verify required roles if any
        if !self.required_roles.is_empty() {
            let has_required_role = roles.iter().any(|role| self.required_roles.contains(role));
            if !has_required_role {
                return Err(MuxlyError::Authentication("Insufficient permissions".to_string()));
            }
        }
        
        // Extract name and email
        let name = decoded.preferred_username().map(String::from);
        let email = decoded.email().map(String::from);
        
        // Create user info
        let user_info = UserInfo {
            sub,
            name,
            email,
            roles,
        };
        
        Ok(user_info)
    }
    
    /// Extract token from authorization header
    pub fn extract_token(headers: &HeaderMap) -> Result<String> {
        let auth_header = headers.get("Authorization")
            .ok_or_else(|| MuxlyError::Authentication("Missing Authorization header".to_string()))?
            .to_str()
            .map_err(|_| MuxlyError::Authentication("Invalid Authorization header".to_string()))?;
            
        if !auth_header.starts_with("Bearer ") {
            return Err(MuxlyError::Authentication("Invalid Authorization header format. Expected Bearer token".to_string()));
        }
        
        Ok(auth_header[7..].to_string())
    }
}

/// Middleware for authentication
pub async fn keycloak_auth_middleware<B>(
    auth: KeycloakAuth,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Extract token
    let token = match KeycloakAuth::extract_token(req.headers()) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Verify token
    match auth.verify_token(&token).await {
        Ok(user_info) => {
            // Store user info in extensions for later access
            req.extensions_mut().insert(user_info);
            Ok(next.run(req).await)
        },
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Extract current user from request
pub fn get_current_user<B>(req: &Request<B>) -> Option<UserInfo> {
    req.extensions().get::<UserInfo>().cloned()
}

/// Check if user has a specific role
pub fn has_role<B>(req: &Request<B>, role: &str) -> bool {
    if let Some(user) = req.extensions().get::<UserInfo>() {
        user.roles.iter().any(|r| r == role)
    } else {
        false
    }
}

/// Role-based access middleware
pub fn require_role(role: String) -> impl Fn(Request<B>) -> Pin<Box<dyn Future<Output = Result<Request<B>, StatusCode>> + Send>> + Clone
where
    B: Send + 'static,
{
    move |req: Request<B>| {
        let role_clone = role.clone();
        Box::pin(async move {
            if has_role(&req, &role_clone) {
                Ok(req)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        })
    }
} 