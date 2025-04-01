use std::time::Instant;

use anyhow::Result;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{debug, info};

/// Middleware for logging API requests
pub async fn logging_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let start = Instant::now();
    
    debug!("Request: {} {}", method, path);
    
    // Process the request
    let response = next.run(req).await;
    
    // Log the response time
    let duration = start.elapsed();
    info!("Response: {} {} - {:?}", method, path, duration);
    
    Ok(response)
} 