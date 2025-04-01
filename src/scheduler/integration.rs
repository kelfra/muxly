use anyhow::Result;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{ApiScheduler, ApiSchedulerConfig, CronConfig, CronScheduler, WebhookConfig, WebhookScheduler};

/// Configuration for the unified scheduler integration
/// 
/// This struct holds the configuration for all three scheduler types:
/// - API scheduler for RESTful job management
/// - Cron scheduler for time-based job execution
/// - Webhook scheduler for event-triggered job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// API scheduler configuration
    pub api: ApiSchedulerConfig,
    /// Cron scheduler configuration
    pub cron: CronConfig,
    /// Webhook scheduler configuration
    pub webhook: WebhookConfig,
}

/// Main integration point for all scheduler types
/// 
/// The SchedulerIntegration provides a unified interface to manage and use
/// different scheduling mechanisms within a single application. It handles
/// initialization, startup, and shutdown of all scheduler types and exposes
/// their functionality through a common API.
pub struct SchedulerIntegration {
    /// API scheduler for RESTful job management
    pub api_scheduler: Arc<ApiScheduler>,
    /// Cron scheduler for time-based job execution
    pub cron_scheduler: Arc<CronScheduler>,
    /// Webhook scheduler for event-triggered job execution
    pub webhook_scheduler: Arc<WebhookScheduler>,
}

impl SchedulerIntegration {
    /// Create a new scheduler integration with the provided configuration
    /// 
    /// # Arguments
    /// * `config` - Configuration for all scheduler types
    /// 
    /// # Returns
    /// A new instance of SchedulerIntegration with initialized schedulers
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
    /// 
    /// This method initializes and starts all scheduler components.
    /// Currently, only the cron scheduler requires an explicit start.
    /// 
    /// # Returns
    /// Result indicating success or failure
    pub async fn start(&self) -> Result<()> {
        self.cron_scheduler.start().await;
        Ok(())
    }

    /// Stop all schedulers
    /// 
    /// This method gracefully shuts down all scheduler components.
    /// Currently, only the cron scheduler requires an explicit stop.
    /// 
    /// # Returns
    /// Result indicating success or failure
    pub async fn stop(&self) -> Result<()> {
        self.cron_scheduler.stop().await;
        Ok(())
    }

    /// Create API routes for all schedulers
    /// 
    /// This method combines the routes from all schedulers into a single Router.
    /// The webhook scheduler routes are nested under the "/webhooks" path.
    /// 
    /// # Returns
    /// An axum Router with all scheduler routes
    pub fn routes(&self) -> Router {
        Router::new()
            .merge(self.api_scheduler.clone().routes())
            .merge(Router::new().nest(
                "/webhooks",
                self.webhook_scheduler.clone().routes(),
            ))
    }
} 