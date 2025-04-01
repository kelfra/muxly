use anyhow::Result;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{ApiScheduler, ApiSchedulerConfig, CronConfig, CronScheduler, WebhookConfig, WebhookScheduler};

/// Scheduler integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// API scheduler configuration
    pub api: ApiSchedulerConfig,
    /// Cron scheduler configuration
    pub cron: CronConfig,
    /// Webhook scheduler configuration
    pub webhook: WebhookConfig,
}

/// Scheduler integration
pub struct SchedulerIntegration {
    /// API scheduler
    pub api_scheduler: Arc<ApiScheduler>,
    /// Cron scheduler
    pub cron_scheduler: Arc<CronScheduler>,
    /// Webhook scheduler
    pub webhook_scheduler: Arc<WebhookScheduler>,
}

impl SchedulerIntegration {
    /// Create a new scheduler integration
    pub fn new(config: SchedulerConfig) -> Self {
        let api_scheduler = Arc::new(ApiScheduler::new(config.api));
        let cron_scheduler = Arc::new(CronScheduler::new());
        let webhook_scheduler = Arc::new(WebhookScheduler::new(config.webhook));

        Self {
            api_scheduler,
            cron_scheduler,
            webhook_scheduler,
        }
    }

    /// Start all schedulers
    pub async fn start(&self) -> Result<()> {
        self.cron_scheduler.start().await;
        Ok(())
    }

    /// Stop all schedulers
    pub async fn stop(&self) -> Result<()> {
        self.cron_scheduler.stop().await;
        Ok(())
    }

    /// Create API routes for all schedulers
    pub fn routes(&self) -> Router {
        Router::new()
            .merge(self.api_scheduler.clone().routes())
            .merge(Router::new().nest(
                "/webhooks",
                self.webhook_scheduler.clone().routes(),
            ))
    }
} 