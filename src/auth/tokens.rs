use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The access token
    pub access_token: String,
    /// The refresh token
    pub refresh_token: String,
    /// The token type (e.g. "Bearer")
    pub token_type: String,
    /// When the token expires (Unix timestamp in seconds)
    pub expires_at: u64,
}

impl AuthToken {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        now >= self.expires_at
    }
    
    /// Get the token with the token type (e.g. "Bearer <token>")
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
    
    /// Get seconds until expiration
    pub fn expires_in(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        self.expires_at as i64 - now as i64
    }
    
    /// Create a new token from parts
    pub fn new(access_token: String, refresh_token: String, token_type: String, expires_in: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        let expires_at = now + expires_in;
        
        Self {
            access_token,
            refresh_token,
            token_type,
            expires_at,
        }
    }
} 