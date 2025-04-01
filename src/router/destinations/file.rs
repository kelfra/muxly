use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use chrono::Utc;

use crate::router::Destination;

/// Configuration for the file destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDestinationConfig {
    /// Path to save files
    pub path: String,
    /// Format of the output files (json, csv, jsonl)
    pub format: String,
    /// Template for the filename
    pub filename_template: String,
    /// Maximum size of a file in MB
    pub max_file_size_mb: u64,
    /// Whether to rotate files
    pub rotate_files: bool,
}

/// Destination that writes data to files
pub struct FileDestination {
    /// Unique identifier
    pub id: String,
    /// Configuration for the file destination
    pub config: FileDestinationConfig,
}

impl FileDestination {
    /// Create a new file destination
    pub fn new(id: String, config: FileDestinationConfig) -> Self {
        Self { id, config }
    }
    
    /// Generate a filename based on the template
    fn generate_filename(&self, data: &Value) -> String {
        let mut filename = self.config.filename_template.clone();
        
        // Replace {{date}} with the current date
        let now = Utc::now();
        filename = filename.replace("{{date}}", &now.format("%Y-%m-%d").to_string());
        
        // Replace {{connector_id}} with the connector ID if present
        if let Some(connector_id) = data.get("connector_id").and_then(|v| v.as_str()) {
            filename = filename.replace("{{connector_id}}", connector_id);
        }
        
        // Add extension if not present
        if !filename.ends_with(&format!(".{}", self.config.format)) {
            filename = format!("{}.{}", filename, self.config.format);
        }
        
        filename
    }
    
    /// Get the full path for a file
    fn get_file_path(&self, filename: &str) -> PathBuf {
        let base_path = Path::new(&self.config.path);
        base_path.join(filename)
    }
}

#[async_trait]
impl Destination for FileDestination {
    fn get_type(&self) -> &str {
        "file"
    }
    
    fn get_id(&self) -> &str {
        &self.id
    }
    
    async fn send(&self, data: Value) -> Result<()> {
        // Create the output directory if it doesn't exist
        fs::create_dir_all(&self.config.path)?;
        
        // Generate the filename
        let filename = self.generate_filename(&data);
        let file_path = self.get_file_path(&filename);
        
        // Serialize the data based on the format
        let content = match self.config.format.as_str() {
            "json" => serde_json::to_string_pretty(&data)?,
            "jsonl" => {
                let mut content = serde_json::to_string(&data)?;
                content.push('\n');
                content
            },
            "csv" => {
                // In a real implementation, this would convert the JSON to CSV
                // For now, just convert to string
                format!("{}\n", serde_json::to_string(&data)?)
            },
            _ => return Err(anyhow::anyhow!("Unsupported format: {}", self.config.format)),
        };
        
        // Write to the file
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        
        Ok(())
    }
    
    async fn send_batch(&self, data: Vec<Value>) -> Result<()> {
        // Create the output directory if it doesn't exist
        fs::create_dir_all(&self.config.path)?;
        
        // In a real implementation, this would handle batching and file rotation
        // For now, just write each item to a file
        for item in data {
            self.send(item).await?;
        }
        
        Ok(())
    }
    
    async fn check_availability(&self) -> Result<bool> {
        // Check if the directory exists or can be created
        if let Err(e) = fs::create_dir_all(&self.config.path) {
            tracing::error!("Failed to create directory {}: {}", self.config.path, e);
            return Ok(false);
        }
        
        // Check if we can write to the directory
        let test_file = self.get_file_path("test.tmp");
        match File::create(&test_file) {
            Ok(mut file) => {
                // Write some data to ensure we can actually write
                if file.write_all(b"test").is_err() {
                    return Ok(false);
                }
                
                // Clean up
                if let Err(e) = fs::remove_file(&test_file) {
                    tracing::warn!("Failed to remove test file: {}", e);
                }
                
                Ok(true)
            },
            Err(e) => {
                tracing::error!("Failed to create test file: {}", e);
                Ok(false)
            }
        }
    }
} 