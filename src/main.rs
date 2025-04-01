mod api;
mod auth;
mod collector;
mod config;
mod error;
mod scheduler;
mod storage;
mod transform;

use std::sync::Arc;
use std::net::SocketAddr;
use axum::{
    routing::get,
    Router,
    http::Method,
    middleware,
    Extension,
};
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;
use tokio::signal;

use error::{MuxlyError, Result};
use auth::{KeycloakAuth, KeycloakConfig, AuthState};
use scheduler::{SchedulerConfig, SchedulerIntegration, ApiSchedulerConfig, CronConfig, WebhookConfig};
use storage::{DatabaseConfig, init_database, shutdown_database};

async fn hello_world() -> &'static str {
    "Hello, Muxly!"
}

async fn health_check() -> &'static str {
    "OK"
}

// Graceful shutdown handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
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

    // Initialize database
    let db_config = DatabaseConfig {
        url: "sqlite:muxly.db".to_string(),
        ..Default::default()
    };
    
    let db_pool = init_database(&db_config).await?;
    info!("Database initialized successfully");

    // Initialize Keycloak authentication
    let keycloak_config = KeycloakConfig {
        server_url: std::env::var("KEYCLOAK_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
        realm: std::env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "muxly".to_string()),
        client_id: std::env::var("KEYCLOAK_CLIENT_ID").unwrap_or_else(|_| "muxly-api".to_string()),
        client_secret: std::env::var("KEYCLOAK_CLIENT_SECRET").unwrap_or_else(|_| "secret".to_string()),
        required_role: Some("user".to_string()),
    };
    
    let keycloak_auth = match KeycloakAuth::new(keycloak_config).await {
        Ok(auth) => {
            info!("Keycloak authentication initialized successfully");
            Arc::new(auth)
        },
        Err(e) => {
            warn!("Failed to initialize Keycloak authentication: {}", e);
            warn!("Starting without authentication. THIS IS NOT SECURE FOR PRODUCTION!");
            Arc::new(KeycloakAuth::new(KeycloakConfig {
                server_url: "http://localhost:8080".to_string(),
                realm: "muxly".to_string(),
                client_id: "muxly-api".to_string(),
                client_secret: "secret".to_string(),
                required_role: None,
            }).await?)
        }
    };
    
    let auth_state = AuthState {
        keycloak: keycloak_auth.clone(),
    };

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
    scheduler_integration.start().await.map_err(|e| MuxlyError::Scheduler(e.to_string()))?;
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
    ).await.map_err(|e| MuxlyError::Scheduler(e.to_string()))?;
    
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
    ).await.map_err(|e| MuxlyError::Scheduler(e.to_string()))?;
    
    info!("Registered cron job");

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    // Build application with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .merge(api::api_router())
        .merge(scheduler_integration.routes())
        .layer(cors)
        .layer(Extension(db_pool.clone()))
        .layer(Extension(auth_state));

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on http://{}", addr);

    // Build the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server started successfully!");
    
    // Start the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| MuxlyError::Internal(format!("Server error: {}", e)))?;

    // Cleanup resources
    info!("Stopping services...");
    
    // Stop the scheduler
    if let Err(e) = scheduler_integration.stop().await {
        error!("Error stopping scheduler: {}", e);
    }
    
    // Shutdown database
    if let Err(e) = shutdown_database(&db_pool).await {
        error!("Error closing database connections: {}", e);
    }
    
    info!("All services stopped successfully");
    Ok(())
}
