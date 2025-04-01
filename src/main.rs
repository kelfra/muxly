mod api;
mod auth;
mod collector;
mod config;
mod scheduler;
mod storage;
mod transform;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use scheduler::{SchedulerConfig, SchedulerIntegration, ApiSchedulerConfig, CronConfig, WebhookConfig};

async fn hello_world() -> &'static str {
    "Hello, Muxly!"
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    info!("Starting Muxly service");

    // Initialize scheduler
    let scheduler_config = SchedulerConfig {
        api: ApiSchedulerConfig {
            enabled: true,
        },
        cron: CronConfig {
            enabled: true,
            catch_up: false,
            cron_expression: None,
            timezone: None,
        },
        webhook: WebhookConfig {
            enabled: true,
            secret: Some("test-webhook-secret".to_string()),
        },
    };

    let scheduler_integration = Arc::new(SchedulerIntegration::new(scheduler_config));
    
    // Start the scheduler
    scheduler_integration.start().await?;
    info!("Schedulers started");

    // Register an example API job
    let api_scheduler = scheduler_integration.api_scheduler.clone();
    let job_id = api_scheduler.register_job(
        "example-job",
        Some("An example job that returns a greeting".to_string()),
        Arc::new(|params| {
            let name = params.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("World");
            
            Ok(serde_json::json!({
                "greeting": format!("Hello, {}!", name)
            }))
        }),
        true,
    ).await?;
    
    info!("Registered API job with ID: {}", job_id);

    // Register an example cron job
    let cron_scheduler = scheduler_integration.cron_scheduler.clone();
    let cron_handler = Arc::new(|| {
        info!("Cron job executed!");
        Box::pin(async { Ok(()) })
    });
    
    cron_scheduler.add_job(
        "example-cron",
        "*/5 * * * * *", // Every 5 seconds
        cron_handler,
        true, // enabled
        false, // catch_up
    ).await?;
    
    info!("Registered cron job");

    // Build application with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .merge(scheduler_integration.routes());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    // Stop the scheduler before exiting
    scheduler_integration.stop().await?;
    
    Ok(())
}
