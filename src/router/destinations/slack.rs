use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use reqwest::Client;
use std::collections::HashMap;

use crate::router::Destination;

/// Configuration for the Slack destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackDestinationConfig {
    /// Webhook URL
    pub webhook_url: String,
    /// Channel to send to (overrides webhook default)
    pub channel: Option<String>,
    /// Username to use (overrides webhook default)
    pub username: Option<String>,
    /// Icon emoji or URL (overrides webhook default)
    pub icon: Option<String>,
    /// Message template
    pub message_template: Option<String>,
    /// Template variables (field name to JSON path)
    pub template_variables: HashMap<String, String>,
    /// Whether to include full data as attachment
    pub include_data: bool,
    /// Color for attachment
    pub color: Option<String>,
}

/// Destination that sends notifications to Slack
pub struct SlackDestination {
    /// Unique identifier
    pub id: String,
    /// Configuration for the Slack destination
    pub config: SlackDestinationConfig,
    /// HTTP client
    client: Client,
}

impl SlackDestination {
    /// Create a new Slack destination
    pub fn new(id: String, config: SlackDestinationConfig) -> Self {
        Self { 
            id, 
            config,
            client: Client::new(),
        }
    }
    
    /// Format a message using the template and data
    fn format_message(&self, data: &Value) -> String {
        // Use the template or default
        let template = self.config.message_template.as_deref()
            .unwrap_or("New data received from {{connector_id}}");
        
        let mut message = template.to_string();
        
        // Replace connector_id if present
        if let Some(connector_id) = data.get("connector_id").and_then(|v| v.as_str()) {
            message = message.replace("{{connector_id}}", connector_id);
        }
        
        // Replace custom variables
        for (var_name, json_path) in &self.config.template_variables {
            let var_placeholder = format!("{{{{{}}}}}", var_name);
            
            // Simple JSON path implementation
            let mut current = data;
            let mut value_found = false;
            
            for part in json_path.split('.') {
                if let Some(obj) = current.as_object() {
                    if let Some(val) = obj.get(part) {
                        current = val;
                        value_found = true;
                    } else {
                        value_found = false;
                        break;
                    }
                } else {
                    value_found = false;
                    break;
                }
            }
            
            if value_found {
                // Convert the value to string
                let value_str = match current {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => format!("{:?}", current),
                };
                
                message = message.replace(&var_placeholder, &value_str);
            }
        }
        
        message
    }
    
    /// Create attachments from data
    fn create_attachments(&self, data: &Value) -> Vec<Value> {
        if !self.config.include_data {
            return Vec::new();
        }
        
        let mut fields = Vec::new();
        
        // Only include first level fields to avoid huge messages
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                // Skip very large or complex values
                let value_str = match value {
                    Value::String(s) => {
                        if s.len() > 100 {
                            format!("{}...", &s[..97])
                        } else {
                            s.clone()
                        }
                    },
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Array(_) => "[...]".to_string(),
                    Value::Object(_) => "{...}".to_string(),
                };
                
                fields.push(json!({
                    "title": key,
                    "value": value_str,
                    "short": value_str.len() <= 20
                }));
            }
        }
        
        // Create the attachment
        vec![json!({
            "fallback": "Data details",
            "color": self.config.color.as_deref().unwrap_or("#36a64f"),
            "fields": fields,
            "footer": "Muxly Router",
            "ts": chrono::Utc::now().timestamp()
        })]
    }
}

#[async_trait]
impl Destination for SlackDestination {
    fn get_type(&self) -> &str {
        "slack"
    }
    
    fn get_id(&self) -> &str {
        &self.id
    }
    
    async fn send(&self, data: Value) -> Result<()> {
        // Format the message
        let text = self.format_message(&data);
        
        // Create payload
        let mut payload = json!({
            "text": text,
            "attachments": self.create_attachments(&data),
        });
        
        // Add optional fields
        if let Some(channel) = &self.config.channel {
            payload["channel"] = json!(channel);
        }
        
        if let Some(username) = &self.config.username {
            payload["username"] = json!(username);
        }
        
        if let Some(icon) = &self.config.icon {
            if icon.starts_with(':') && icon.ends_with(':') {
                payload["icon_emoji"] = json!(icon);
            } else {
                payload["icon_url"] = json!(icon);
            }
        }
        
        // Send to Slack
        let response = self.client.post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Slack API error: {}", error_text));
        }
        
        Ok(())
    }
    
    async fn send_batch(&self, data: Vec<Value>) -> Result<()> {
        // For Slack, we'll just send a summary message
        if data.is_empty() {
            return Ok(());
        }
        
        // Use the first item for the message template
        let template_data = &data[0];
        
        // Add count information
        let mut enhanced_data = template_data.clone();
        if let Some(obj) = enhanced_data.as_object_mut() {
            obj.insert("batch_count".to_string(), json!(data.len()));
        }
        
        // Send the summary
        self.send(enhanced_data).await
    }
    
    async fn check_availability(&self) -> Result<bool> {
        // Try a basic API test
        // Slack doesn't have a test endpoint for webhooks, so we'll just check if the URL is valid
        if !self.config.webhook_url.starts_with("https://hooks.slack.com/") {
            return Ok(false);
        }
        
        Ok(true)
    }
} 