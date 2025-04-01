//! Router configuration model
//!
//! Contains settings for configuring output destinations like Prometheus,
//! webhooks, files, and databases.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Global router settings
    #[serde(default)]
    pub global: GlobalRouterConfig,
    
    /// Prometheus output configuration
    #[serde(default)]
    pub prometheus: PrometheusConfig,
    
    /// Webhook output configuration
    #[serde(default)]
    pub webhook: WebhookOutputConfig,
    
    /// File output configuration
    #[serde(default)]
    pub file: FileOutputConfig,
    
    /// S3 output configuration
    #[serde(default)]
    pub s3: S3OutputConfig,
    
    /// Database output configuration
    #[serde(default)]
    pub database: DatabaseOutputConfig,
    
    /// Slack notification configuration
    #[serde(default)]
    pub slack: SlackConfig,
    
    /// Custom outputs
    #[serde(default)]
    pub custom: HashMap<String, CustomOutputConfig>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            global: GlobalRouterConfig::default(),
            prometheus: PrometheusConfig::default(),
            webhook: WebhookOutputConfig::default(),
            file: FileOutputConfig::default(),
            s3: S3OutputConfig::default(),
            database: DatabaseOutputConfig::default(),
            slack: SlackConfig::default(),
            custom: HashMap::new(),
        }
    }
}

/// Global router settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRouterConfig {
    /// Enable routing system
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Buffer size for output queues
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    
    /// Batch size for sending data
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    
    /// Flush interval in seconds
    #[serde(default = "default_flush_interval")]
    pub flush_interval: u64,
    
    /// Number of retries for failed deliveries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// Enable routing transformation
    #[serde(default = "default_enable_transformation")]
    pub enable_transformation: bool,
}

impl Default for GlobalRouterConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            buffer_size: default_buffer_size(),
            batch_size: default_batch_size(),
            flush_interval: default_flush_interval(),
            max_retries: default_max_retries(),
            enable_transformation: default_enable_transformation(),
        }
    }
}

/// Prometheus metrics output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    /// Enable Prometheus output
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Metrics path
    #[serde(default = "default_metrics_path")]
    pub metrics_path: String,
    
    /// Include labels in metrics
    #[serde(default = "default_include_labels")]
    pub include_labels: bool,
    
    /// Default metric type (counter, gauge, histogram)
    #[serde(default = "default_metric_type")]
    pub default_metric_type: String,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            metrics_path: default_metrics_path(),
            include_labels: default_include_labels(),
            default_metric_type: default_metric_type(),
        }
    }
}

/// Webhook output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookOutputConfig {
    /// Enable webhook output
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Webhook URLs
    #[serde(default)]
    pub endpoints: Vec<WebhookEndpoint>,
    
    /// Maximum concurrency for webhook deliveries
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: usize,
    
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

impl Default for WebhookOutputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoints: Vec::new(),
            max_concurrency: default_max_concurrency(),
            timeout: default_timeout(),
        }
    }
}

/// Webhook endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    /// Endpoint URL
    pub url: String,
    
    /// Secret for signing requests
    #[serde(default)]
    pub secret: Option<String>,
    
    /// HTTP method (POST, PUT)
    #[serde(default = "default_http_method")]
    pub method: String,
    
    /// Authorization header
    #[serde(default)]
    pub authorization: Option<String>,
    
    /// Event types to send to this endpoint
    #[serde(default)]
    pub event_types: Vec<String>,
}

/// File output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOutputConfig {
    /// Enable file output
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Output directory
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    
    /// File format (json, csv, parquet)
    #[serde(default = "default_file_format")]
    pub format: String,
    
    /// Append to existing files
    #[serde(default = "default_append")]
    pub append: bool,
    
    /// File rotation period (hourly, daily, weekly, monthly)
    #[serde(default = "default_rotation")]
    pub rotation: String,
    
    /// Maximum file size in MB before rotation
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,
}

impl Default for FileOutputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            output_dir: default_output_dir(),
            format: default_file_format(),
            append: default_append(),
            rotation: default_rotation(),
            max_file_size: default_max_file_size(),
        }
    }
}

/// S3 output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3OutputConfig {
    /// Enable S3 output
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// AWS region
    #[serde(default)]
    pub region: Option<String>,
    
    /// S3 bucket name
    #[serde(default)]
    pub bucket: Option<String>,
    
    /// S3 path prefix
    #[serde(default = "default_s3_prefix")]
    pub prefix: String,
    
    /// File format (json, csv, parquet)
    #[serde(default = "default_file_format")]
    pub format: String,
    
    /// AWS access key ID
    #[serde(default)]
    pub access_key_id: Option<String>,
    
    /// AWS secret access key
    #[serde(default)]
    pub secret_access_key: Option<String>,
    
    /// Use AWS instance profile
    #[serde(default = "default_use_instance_profile")]
    pub use_instance_profile: bool,
}

impl Default for S3OutputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            region: None,
            bucket: None,
            prefix: default_s3_prefix(),
            format: default_file_format(),
            access_key_id: None,
            secret_access_key: None,
            use_instance_profile: default_use_instance_profile(),
        }
    }
}

/// Database output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOutputConfig {
    /// Enable database output
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Database connection URL
    #[serde(default)]
    pub connection_url: Option<String>,
    
    /// Database type (postgres, mysql, sqlite, sqlserver)
    #[serde(default = "default_db_type")]
    pub db_type: String,
    
    /// Target table name
    #[serde(default)]
    pub table: Option<String>,
    
    /// Schema name
    #[serde(default)]
    pub schema: Option<String>,
    
    /// Create table if not exists
    #[serde(default = "default_create_table")]
    pub create_table: bool,
    
    /// Batch size for inserts
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

impl Default for DatabaseOutputConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            connection_url: None,
            db_type: default_db_type(),
            table: None,
            schema: None,
            create_table: default_create_table(),
            batch_size: default_batch_size(),
        }
    }
}

/// Slack notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Enable Slack notifications
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Webhook URL
    #[serde(default)]
    pub webhook_url: Option<String>,
    
    /// Default channel
    #[serde(default)]
    pub channel: Option<String>,
    
    /// Bot username
    #[serde(default = "default_username")]
    pub username: String,
    
    /// Icon emoji or URL
    #[serde(default = "default_icon")]
    pub icon: String,
    
    /// Event types to notify about
    #[serde(default)]
    pub event_types: Vec<String>,
}

impl Default for SlackConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhook_url: None,
            channel: None,
            username: default_username(),
            icon: default_icon(),
            event_types: Vec::new(),
        }
    }
}

/// Custom output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomOutputConfig {
    /// Enable custom output
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Output type
    pub output_type: String,
    
    /// Plugin file path
    #[serde(default)]
    pub plugin_path: Option<String>,
    
    /// Configuration parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

fn default_enabled() -> bool {
    true
}

fn default_buffer_size() -> usize {
    1000
}

fn default_batch_size() -> usize {
    100
}

fn default_flush_interval() -> u64 {
    5
}

fn default_max_retries() -> u32 {
    3
}

fn default_enable_transformation() -> bool {
    true
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

fn default_include_labels() -> bool {
    true
}

fn default_metric_type() -> String {
    "gauge".to_string()
}

fn default_max_concurrency() -> usize {
    10
}

fn default_timeout() -> u64 {
    30
}

fn default_http_method() -> String {
    "POST".to_string()
}

fn default_output_dir() -> String {
    "./output".to_string()
}

fn default_file_format() -> String {
    "json".to_string()
}

fn default_append() -> bool {
    true
}

fn default_rotation() -> String {
    "daily".to_string()
}

fn default_max_file_size() -> usize {
    100
}

fn default_s3_prefix() -> String {
    "muxly/".to_string()
}

fn default_use_instance_profile() -> bool {
    false
}

fn default_db_type() -> String {
    "postgres".to_string()
}

fn default_create_table() -> bool {
    false
}

fn default_username() -> String {
    "Muxly Bot".to_string()
}

fn default_icon() -> String {
    ":chart_with_upwards_trend:".to_string()
} 