use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use aws_sdk_s3::{Client, config::Region};
use aws_types::Credentials;
use bytes::Bytes;
use chrono::Utc;

use crate::router::Destination;

/// Configuration for the S3 storage destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3DestinationConfig {
    /// S3 bucket name
    pub bucket: String,
    /// Key prefix (folder) in the bucket
    pub key_prefix: String,
    /// AWS region
    pub region: String,
    /// Format of the output files (json, csv, parquet)
    pub format: String,
    /// Template for the key (filename)
    pub key_template: String,
    /// AWS credentials
    pub credentials: S3Credentials,
    /// Content type to use
    pub content_type: Option<String>,
}

/// S3 authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Credentials {
    /// AWS access key ID
    pub access_key_id: String,
    /// AWS secret access key
    pub secret_access_key: String,
    /// Optional session token
    pub session_token: Option<String>,
}

/// Destination that writes data to AWS S3
pub struct S3Destination {
    /// Unique identifier
    pub id: String,
    /// Configuration for the S3 destination
    pub config: S3DestinationConfig,
    /// S3 client
    client: Option<Client>,
}

impl S3Destination {
    /// Create a new S3 destination
    pub fn new(id: String, config: S3DestinationConfig) -> Self {
        Self { 
            id, 
            config,
            client: None,
        }
    }
    
    /// Initialize the S3 client
    pub async fn initialize(&mut self) -> Result<()> {
        let region = Region::new(self.config.region.clone());
        
        let credentials = Credentials::new(
            &self.config.credentials.access_key_id,
            &self.config.credentials.secret_access_key,
            self.config.credentials.session_token.as_deref(),
            None,
            "S3Destination",
        );
        
        let config = aws_sdk_s3::Config::builder()
            .region(region)
            .credentials_provider(credentials)
            .build();
        
        let client = Client::from_conf(config);
        self.client = Some(client);
        
        Ok(())
    }
    
    /// Generate a key (filename) based on the template
    fn generate_key(&self, data: &Value) -> String {
        let mut key = self.config.key_template.clone();
        
        // Replace {{date}} with the current date
        let now = Utc::now();
        key = key.replace("{{date}}", &now.format("%Y-%m-%d").to_string());
        key = key.replace("{{timestamp}}", &now.format("%Y%m%d_%H%M%S").to_string());
        
        // Replace {{connector_id}} with the connector ID if present
        if let Some(connector_id) = data.get("connector_id").and_then(|v| v.as_str()) {
            key = key.replace("{{connector_id}}", connector_id);
        }
        
        // Add prefix if not included in template
        if !key.starts_with(&self.config.key_prefix) {
            if !self.config.key_prefix.ends_with('/') && !key.starts_with('/') {
                key = format!("{}/{}", self.config.key_prefix, key);
            } else {
                key = format!("{}{}", self.config.key_prefix, key);
            }
        }
        
        // Add extension if not present
        if !key.ends_with(&format!(".{}", self.config.format)) {
            key = format!("{}.{}", key, self.config.format);
        }
        
        key
    }
    
    /// Determine content type based on format
    fn get_content_type(&self) -> String {
        self.config.content_type.clone().unwrap_or_else(|| {
            match self.config.format.as_str() {
                "json" => "application/json".to_string(),
                "csv" => "text/csv".to_string(),
                "parquet" => "application/octet-stream".to_string(),
                _ => "application/octet-stream".to_string(),
            }
        })
    }
    
    /// Convert data to bytes based on format
    fn serialize_data(&self, data: &Value) -> Result<Bytes> {
        match self.config.format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(data)?;
                Ok(Bytes::from(json))
            },
            "csv" => {
                // In a real implementation, this would convert the JSON to CSV
                // For now, just convert to string
                let json = serde_json::to_string(data)?;
                Ok(Bytes::from(json))
            },
            "parquet" => {
                // In a real implementation, this would convert the JSON to Parquet
                // For now, just convert to string
                let json = serde_json::to_string(data)?;
                Ok(Bytes::from(json))
            },
            _ => Err(anyhow::anyhow!("Unsupported format: {}", self.config.format)),
        }
    }
    
    /// Serialize batch data
    fn serialize_batch(&self, data: &[Value]) -> Result<Bytes> {
        match self.config.format.as_str() {
            "json" => {
                // Create a JSON array with all items
                let json = serde_json::to_string_pretty(data)?;
                Ok(Bytes::from(json))
            },
            "csv" => {
                // In a real implementation, this would convert the JSON to CSV
                // For now, just convert to JSONL (JSON Lines)
                let mut result = Vec::new();
                for item in data {
                    let line = serde_json::to_string(item)?;
                    result.push(line);
                }
                Ok(Bytes::from(result.join("\n")))
            },
            "parquet" => {
                // In a real implementation, this would convert the JSON to Parquet
                // For now, just convert to string
                let json = serde_json::to_string(data)?;
                Ok(Bytes::from(json))
            },
            _ => Err(anyhow::anyhow!("Unsupported format: {}", self.config.format)),
        }
    }
}

#[async_trait]
impl Destination for S3Destination {
    fn get_type(&self) -> &str {
        "s3"
    }
    
    fn get_id(&self) -> &str {
        &self.id
    }
    
    async fn send(&self, data: Value) -> Result<()> {
        if let Some(client) = &self.client {
            // Generate key
            let key = self.generate_key(&data);
            
            // Serialize data
            let body = self.serialize_data(&data)?;
            
            // Upload to S3
            client.put_object()
                .bucket(&self.config.bucket)
                .key(&key)
                .body(body.into())
                .content_type(self.get_content_type())
                .send()
                .await?;
        } else {
            return Err(anyhow::anyhow!("S3 client not initialized"));
        }
        
        Ok(())
    }
    
    async fn send_batch(&self, data: Vec<Value>) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        
        if let Some(client) = &self.client {
            // Use the first item to generate a key with a batch indicator
            let mut key = self.generate_key(&data[0]);
            key = key.replace(".", "_batch.");
            
            // Serialize batch data
            let body = self.serialize_batch(&data)?;
            
            // Upload to S3
            client.put_object()
                .bucket(&self.config.bucket)
                .key(&key)
                .body(body.into())
                .content_type(self.get_content_type())
                .send()
                .await?;
        } else {
            return Err(anyhow::anyhow!("S3 client not initialized"));
        }
        
        Ok(())
    }
    
    async fn check_availability(&self) -> Result<bool> {
        if let Some(client) = &self.client {
            // Check if we can list the bucket
            match client.list_objects_v2()
                .bucket(&self.config.bucket)
                .prefix(&self.config.key_prefix)
                .max_keys(1)
                .send()
                .await
            {
                Ok(_) => Ok(true),
                Err(e) => {
                    tracing::error!("S3 connection check failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }
} 