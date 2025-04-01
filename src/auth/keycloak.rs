use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};

use crate::auth::tokens::AuthToken;

/// Keycloak client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakConfig {
    /// URL of the Keycloak server
    pub server_url: String,
    /// Realm to use
    pub realm: String,
    /// Client ID
    pub client_id: String,
    /// Client secret (optional, for confidential clients)
    pub client_secret: Option<String>,
}

/// Keycloak client for authentication and authorization
pub struct KeycloakClient {
    /// HTTP client
    client: Client,
    /// Configuration for the Keycloak client
    config: KeycloakConfig,
}

impl KeycloakClient {
    /// Create a new Keycloak client
    pub fn new(params: &Value) -> Self {
        // Extract Keycloak configuration from parameters
        let config = KeycloakConfig {
            server_url: params.get("server_url")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost:8080")
                .to_string(),
            realm: params.get("realm")
                .and_then(|v| v.as_str())
                .unwrap_or("master")
                .to_string(),
            client_id: params.get("client_id")
                .and_then(|v| v.as_str())
                .unwrap_or("muxly")
                .to_string(),
            client_secret: params.get("client_secret")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };
        
        // Create HTTP client with reasonable timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        
        Self { client, config }
    }
    
    /// Get the token endpoint URL
    fn token_endpoint(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.config.server_url,
            self.config.realm
        )
    }
    
    /// Get the userinfo endpoint URL
    fn userinfo_endpoint(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/userinfo",
            self.config.server_url,
            self.config.realm
        )
    }
    
    /// Authenticate with username and password
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        debug!("Authenticating user {} with Keycloak", username);
        
        let mut form = vec![
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
            ("client_id", &self.config.client_id),
        ];
        
        // Add client secret if available
        if let Some(secret) = &self.config.client_secret {
            form.push(("client_secret", secret));
        }
        
        let response = self.client
            .post(&self.token_endpoint())
            .form(&form)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("Keycloak authentication failed: {} - {}", status, body);
            return Err(anyhow!("Authentication failed: {}", status));
        }
        
        let token_response: serde_json::Value = response.json().await?;
        
        let access_token = token_response.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing access_token in response"))?
            .to_string();
        
        let refresh_token = token_response.get("refresh_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing refresh_token in response"))?
            .to_string();
        
        let expires_in = token_response.get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(300);
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let expires_at = now + expires_in;
        
        Ok(AuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_at,
        })
    }
    
    /// Validate a token
    pub async fn validate_token(&self, token: &str) -> Result<bool> {
        debug!("Validating token with Keycloak");
        
        // We'll use the userinfo endpoint to validate the token
        // If the token is valid, this will return user information
        // If not, it will return a 401 Unauthorized
        let response = self.client
            .get(&self.userinfo_endpoint())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    /// Refresh a token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken> {
        debug!("Refreshing token with Keycloak");
        
        let mut form = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
        ];
        
        // Add client secret if available
        if let Some(secret) = &self.config.client_secret {
            form.push(("client_secret", secret));
        }
        
        let response = self.client
            .post(&self.token_endpoint())
            .form(&form)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("Keycloak token refresh failed: {} - {}", status, body);
            return Err(anyhow!("Token refresh failed: {}", status));
        }
        
        let token_response: serde_json::Value = response.json().await?;
        
        let access_token = token_response.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing access_token in response"))?
            .to_string();
        
        let refresh_token = token_response.get("refresh_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing refresh_token in response"))?
            .to_string();
        
        let expires_in = token_response.get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(300);
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let expires_at = now + expires_in;
        
        Ok(AuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_at,
        })
    }
    
    /// Get user information from token
    pub async fn get_user_info(&self, token: &str) -> Result<Value> {
        debug!("Getting user info from Keycloak");
        
        let response = self.client
            .get(&self.userinfo_endpoint())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("Failed to get user info: {} - {}", status, body);
            return Err(anyhow!("Failed to get user info: {}", status));
        }
        
        let user_info = response.json().await?;
        Ok(user_info)
    }
} 