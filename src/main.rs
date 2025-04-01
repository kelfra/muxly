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
use hyper::server::Server;

use scheduler::{SchedulerConfig, SchedulerIntegration, ApiSchedulerConfig, CronConfig, WebhookConfig};

async fn hello_world() -> &'static str {
    "Hello, Muxly!"
}

async fn health_check() -> &'static str {
    "OK"
}

// Graceful shutdown handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
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
            max_concurrent_jobs: Some(10),
            job_timeout_seconds: Some(60),
            max_history_size: Some(100),
        },
        cron: CronConfig {
            enabled: true,
            max_concurrent_jobs: Some(5),
            job_timeout_seconds: Some(300),
            catch_up: Some(false),
            cron_expression: "*/5 * * * * *".to_string(), // Every 5 seconds
            timezone: Some("UTC".to_string()),
        },
        webhook: WebhookConfig {
            enabled: true,
            secret: Some("test-secret".to_string()),
            path: "/webhooks".to_string(),
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
        Ok(()) as Result<(), anyhow::Error>
    });
    
    cron_scheduler.add_job(
        "*/5 * * * * *", // Every 5 seconds
        cron_handler,
        true,  // enabled
        false, // catch up
    ).await?;
    
    info!("Registered cron job");

    // Build application with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .merge(scheduler_integration.routes());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on http://{}", addr);

    let server = Server::bind(&addr)
        .serve(app.into_make_service());

    info!("Server started successfully!");
    
    // Handle graceful shutdown
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    
    // Start the server
    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
    }

    // Stop the scheduler before exiting
    scheduler_integration.stop().await?;
    
    Ok(())
}
