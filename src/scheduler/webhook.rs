use anyhow::{Result, anyhow};
use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook path
    pub path: String,
    /// Secret for webhook authentication
    pub secret: Option<String>,
    /// Whether the webhook is enabled
    pub enabled: bool,
}

/// Webhook handler function
pub type WebhookHandler = Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>;

/// A registered webhook in the scheduler
pub struct RegisteredWebhook {
    /// Webhook ID
    id: String,
    /// Webhook path
    path: String,
    /// Webhook handler
    handler: WebhookHandler,
    /// Webhook secret for authentication
    secret: Option<String>,
    /// Whether the webhook is enabled
    enabled: bool,
}

/// Response from webhook execution
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    /// Response status
    pub status: String,
    /// Response message
    pub message: String,
    /// Response data
    pub data: Option<serde_json::Value>,
}

/// Request to validate a webhook call
#[derive(Debug, Deserialize)]
pub struct WebhookRequest {
    /// Webhook payload
    pub payload: serde_json::Value,
    /// Webhook signature for authentication
    pub signature: Option<String>,
}

/// Webhook scheduler
pub struct WebhookScheduler {
    /// Registered webhooks
    webhooks: RwLock<HashMap<String, RegisteredWebhook>>,
}

impl WebhookScheduler {
    /// Create a new webhook scheduler with default configuration
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            webhooks: RwLock::new(HashMap::new()),
            signatures: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new webhook
    pub async fn register_webhook(
        &self,
        path: &str,
        handler: WebhookHandler,
        secret: Option<String>,
        enabled: bool,
    ) -> Result<String> {
        // Generate a webhook ID
        let id = Uuid::new_v4().to_string();
        
        // Create a registered webhook
        let webhook = RegisteredWebhook {
            id: id.clone(),
            path: path.to_string(),
            handler,
            secret,
            enabled,
        };
        
        // Add to webhooks
        let mut webhooks = self.webhooks.write().await;
        
        // Check if path is already registered
        if webhooks.values().any(|w| w.path == path) {
            return Err(anyhow!("Webhook with path '{}' already registered", path));
        }
        
        webhooks.insert(id.clone(), webhook);
        
        Ok(id)
    }
    
    /// Unregister a webhook
    pub async fn unregister_webhook(&self, id: &str) -> Result<()> {
        let mut webhooks = self.webhooks.write().await;
        
        if webhooks.remove(id).is_none() {
            Err(anyhow!("Webhook with ID '{}' not found", id))
        } else {
            Ok(())
        }
    }
    
    /// Get all registered webhook paths
    pub async fn get_webhook_paths(&self) -> Vec<String> {
        let webhooks = self.webhooks.read().await;
        webhooks.values().filter(|w| w.enabled).map(|w| w.path.clone()).collect()
    }
    
    /// Create webhook routes
    pub fn routes(self: Arc<Self>) -> Router {
        Router::new()
            .route("/webhook/:path", post(handle_webhook))
            .with_state(self)
    }
    
    /// Handle a webhook call
    pub async fn handle_webhook_call(
        &self,
        path: &str,
        request: WebhookRequest,
    ) -> Result<WebhookResponse> {
        // Find webhook by path
        let webhooks = self.webhooks.read().await;
        let webhook = webhooks
            .values()
            .find(|w| w.path == path && w.enabled)
            .ok_or_else(|| anyhow!("Webhook for path '{}' not found or disabled", path))?;
        
        // Validate signature if required
        if let Some(secret) = &webhook.secret {
            if let Some(signature) = &request.signature {
                if !validate_signature(signature, secret, &request.payload) {
                    return Err(anyhow!("Invalid webhook signature"));
                }
            } else {
                return Err(anyhow!("Webhook signature required but not provided"));
            }
        }
        
        // Call the webhook handler
        info!("Executing webhook handler for path: {}", path);
        match (webhook.handler)(request.payload) {
            Ok(result) => Ok(WebhookResponse {
                status: "success".to_string(),
                message: "Webhook executed successfully".to_string(),
                data: Some(result),
            }),
            Err(e) => {
                error!("Webhook handler failed: {}", e);
                Err(anyhow!("Webhook execution failed: {}", e))
            }
        }
    }
}

/// Handler for webhook HTTP requests
async fn handle_webhook(
    State(scheduler): State<Arc<WebhookScheduler>>,
    Path(path): Path<String>,
    Json(request): Json<WebhookRequest>,
) -> Json<WebhookResponse> {
    match scheduler.handle_webhook_call(&path, request).await {
        Ok(response) => Json(response),
        Err(e) => Json(WebhookResponse {
            status: "error".to_string(),
            message: format!("Webhook error: {}", e),
            data: None,
        }),
    }
}

/// Validate webhook signature
fn validate_signature(signature: &str, secret: &str, payload: &serde_json::Value) -> bool {
    // In a real implementation, this would do proper HMAC validation
    // For now, this is a simplified implementation
    
    // Convert payload to string
    let payload_str = payload.to_string();
    
    // Generate HMAC signature
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    
    // Create HMAC instance
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    
    // Add payload to HMAC
    mac.update(payload_str.as_bytes());
    
    // Get HMAC result
    let result = mac.finalize();
    let hex_result = hex::encode(result.into_bytes());
    
    // Compare signature
    signature == hex_result
}
