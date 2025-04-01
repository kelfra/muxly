//! Scheduler configuration model
//!
//! Contains settings for the scheduler module components: API, cron, and webhook schedulers.

use serde::{Deserialize, Serialize};

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// API scheduler configuration
    #[serde(default)]
    pub api: ApiSchedulerConfig,
    
    /// Cron scheduler configuration
    #[serde(default)]
    pub cron: CronConfig,
    
    /// Webhook scheduler configuration
    #[serde(default)]
    pub webhook: WebhookConfig,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            api: ApiSchedulerConfig::default(),
            cron: CronConfig::default(),
            webhook: WebhookConfig::default(),
        }
    }
}

/// API scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSchedulerConfig {
    /// Enable API scheduler
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Maximum concurrent jobs
    #[serde(default = "default_max_concurrent_jobs")]
    pub max_concurrent_jobs: usize,
    
    /// Job execution timeout in seconds
    #[serde(default = "default_job_timeout")]
    pub job_timeout: u64,
    
    /// Maximum execution history size per job
    #[serde(default = "default_max_history_size")]
    pub max_history_size: usize,
}

impl Default for ApiSchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            max_concurrent_jobs: default_max_concurrent_jobs(),
            job_timeout: default_job_timeout(),
            max_history_size: default_max_history_size(),
        }
    }
}

/// Cron scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronConfig {
    /// Enable cron scheduler
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Default catch-up behavior for jobs
    #[serde(default = "default_catch_up")]
    pub catch_up: bool,
    
    /// Default cron expression (optional)
    #[serde(default)]
    pub cron_expression: Option<String>,
    
    /// Default timezone for cron jobs
    #[serde(default)]
    pub timezone: Option<String>,
}

impl Default for CronConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            catch_up: default_catch_up(),
            cron_expression: None,
            timezone: None,
        }
    }
}

/// Webhook scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Enable webhook scheduler
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Secret for validating webhook requests
    #[serde(default)]
    pub secret: Option<String>,
    
    /// Maximum payload size in bytes
    #[serde(default = "default_max_payload_size")]
    pub max_payload_size: usize,
    
    /// Whether to validate webhook signatures
    #[serde(default = "default_validate_signatures")]
    pub validate_signatures: bool,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            secret: None,
            max_payload_size: default_max_payload_size(),
            validate_signatures: default_validate_signatures(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_max_concurrent_jobs() -> usize {
    10
}

fn default_job_timeout() -> u64 {
    300 // 5 minutes
}

fn default_max_history_size() -> usize {
    100
}

fn default_catch_up() -> bool {
    false
}

fn default_max_payload_size() -> usize {
    1024 * 1024 // 1 MB
}

fn default_validate_signatures() -> bool {
    true
} 