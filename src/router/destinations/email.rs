use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use lettre::{
    AsyncTransport, Message, message::{header, MultiPart, SinglePart}, 
    AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials
};
use std::collections::HashMap;
use chrono::Utc;

use crate::router::Destination;

/// Configuration for the Email destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDestinationConfig {
    /// SMTP server hostname
    pub smtp_host: String,
    /// SMTP server port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_username: String,
    /// SMTP password
    pub smtp_password: String,
    /// Use TLS for SMTP connection
    pub use_tls: bool,
    /// Sender email address
    pub from_email: String,
    /// Sender name
    pub from_name: Option<String>,
    /// Recipient email addresses
    pub to_emails: Vec<String>,
    /// CC email addresses
    pub cc_emails: Option<Vec<String>>,
    /// BCC email addresses
    pub bcc_emails: Option<Vec<String>>,
    /// Email subject template
    pub subject_template: String,
    /// Email body template (HTML)
    pub body_template: Option<String>,
    /// Template variables (field name to JSON path)
    pub template_variables: HashMap<String, String>,
}

/// Destination that sends notifications via email
pub struct EmailDestination {
    /// Unique identifier
    pub id: String,
    /// Configuration for the Email destination
    pub config: EmailDestinationConfig,
    /// SMTP transport
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl EmailDestination {
    /// Create a new Email destination
    pub fn new(id: String, config: EmailDestinationConfig) -> Self {
        Self { 
            id, 
            config,
            transport: None,
        }
    }
    
    /// Initialize the SMTP transport
    async fn init_transport(&mut self) -> Result<()> {
        let credentials = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );
        
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)?
            .port(self.config.smtp_port)
            .credentials(credentials);
            
        if !self.config.use_tls {
            builder = builder.tls(lettre::transport::smtp::client::Tls::None);
        }
        
        self.transport = Some(builder.build());
        
        Ok(())
    }
    
    /// Format a string using the template and data
    fn format_template(&self, template: &str, data: &Value) -> String {
        let mut result = template.to_string();
        
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
                
                result = result.replace(&var_placeholder, &value_str);
            }
        }
        
        result
    }
    
    /// Create HTML table from data
    fn create_data_table(&self, data: &Value) -> String {
        let mut table = String::from("<table border='1' cellpadding='4' style='border-collapse: collapse;'>");
        table.push_str("<tr><th>Field</th><th>Value</th></tr>");
        
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                // Format the value
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
                    Value::Array(_) => "[array]".to_string(),
                    Value::Object(_) => "{object}".to_string(),
                };
                
                table.push_str(&format!(
                    "<tr><td><strong>{}</strong></td><td>{}</td></tr>",
                    key, value_str
                ));
            }
        }
        
        table.push_str("</table>");
        table
    }
    
    /// Create a default HTML email body
    fn create_default_body(&self, data: &Value) -> String {
        let mut body = String::from(r#"
        <html>
        <head>
            <style>
                body { font-family: Arial, sans-serif; line-height: 1.5; }
                .container { padding: 20px; }
                h2 { color: #333366; }
                .footer { margin-top: 30px; font-size: 12px; color: #666; }
            </style>
        </head>
        <body>
            <div class="container">
                <h2>Muxly Data Notification</h2>
                <p>Data was received from a connector. Details are below:</p>
        "#);
        
        body.push_str(&self.create_data_table(data));
        
        body.push_str(r#"
                <div class="footer">
                    <p>This is an automated message from Muxly Router.</p>
                    <p>Sent at: "#);
        
        // Add timestamp
        body.push_str(&Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
        
        body.push_str(r#"</p>
                </div>
            </div>
        </body>
        </html>
        "#);
        
        body
    }
}

#[async_trait]
impl Destination for EmailDestination {
    fn get_type(&self) -> &str {
        "email"
    }
    
    fn get_id(&self) -> &str {
        &self.id
    }
    
    async fn send(&mut self, data: Value) -> Result<()> {
        // Initialize transport if not already initialized
        if self.transport.is_none() {
            self.init_transport().await?;
        }
        
        // Format the subject
        let subject = self.format_template(&self.config.subject_template, &data);
        
        // Format the body
        let body = if let Some(template) = &self.config.body_template {
            self.format_template(template, &data)
        } else {
            self.create_default_body(&data)
        };
        
        // Create the message
        let mut message_builder = Message::builder()
            .from(match &self.config.from_name {
                Some(name) => format!("{} <{}>", name, self.config.from_email).parse()?,
                None => self.config.from_email.parse()?,
            });
        
        // Add recipients
        for email in &self.config.to_emails {
            message_builder = message_builder.to(email.parse()?);
        }
        
        // Add CC recipients
        if let Some(cc_emails) = &self.config.cc_emails {
            for email in cc_emails {
                message_builder = message_builder.cc(email.parse()?);
            }
        }
        
        // Add BCC recipients
        if let Some(bcc_emails) = &self.config.bcc_emails {
            for email in bcc_emails {
                message_builder = message_builder.bcc(email.parse()?);
            }
        }
        
        // Build the message with HTML and optional plain text alternative
        let message = message_builder
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(html2text::from_read(body.as_bytes(), 80))
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(body)
                    )
            )?;
        
        // Send the email
        if let Some(transport) = &self.transport {
            transport.send(message).await?;
        } else {
            return Err(anyhow::anyhow!("SMTP transport not initialized"));
        }
        
        Ok(())
    }
    
    async fn send_batch(&mut self, data: Vec<Value>) -> Result<()> {
        // For email, we'll just send a summary message
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
    
    async fn check_availability(&mut self) -> Result<bool> {
        // Initialize transport if not already initialized
        if self.transport.is_none() {
            if let Err(e) = self.init_transport().await {
                log::error!("Failed to initialize SMTP transport: {}", e);
                return Ok(false);
            }
        }
        
        // Just verify we have a valid transport
        Ok(self.transport.is_some())
    }
} 