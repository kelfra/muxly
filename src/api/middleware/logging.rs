use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, error};

/// Middleware for logging API requests
pub async fn logging_middleware<B>(request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_owned();
    let method = request.method().clone();
    
    // Record the start time
    let start = Instant::now();
    
    // Log the incoming request
    info!("Request: {} {}", method, path);
    
    // Process the request
    let response = next.run(request).await;
    
    // Calculate the elapsed time
    let elapsed = start.elapsed();
    
    // Log the response
    let status = response.status();
    info!("Response: {} {} - {} - {:?}", method, path, status.as_u16(), elapsed);
    
    Ok(response)
} 