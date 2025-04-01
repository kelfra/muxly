use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod collector;
mod connectors;
mod config;
mod diagnostics;
mod router;
mod scheduler;
mod storage;
mod transform;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "muxly=info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Muxly service");

    // Load configuration
    // config::load_config()?;

    // Initialize database
    // storage::init_database().await?;

    // Create application router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check));
        // Add more routes as they're implemented

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// Simple health check handler
async fn health_check() -> &'static str {
    "Muxly Service is running"
}
