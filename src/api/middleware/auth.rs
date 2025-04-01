use anyhow::Result;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::error;

/// Middleware for authenticating API requests
pub async fn auth_middleware<B>(request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    // Extract the authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());
    
    // In a real implementation, this would validate the token with Keycloak or another auth provider
    // For now, just check if the header exists and has a valid format
    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            // Continue to the next middleware or handler
            Ok(next.run(request).await)
        },
        Some(_) => {
            error!("Invalid authorization header format");
            Err(StatusCode::UNAUTHORIZED)
        },
        None => {
            // If no header, check if this is a public endpoint
            // For now, just let it through (would be refined in a real implementation)
            Ok(next.run(request).await)
        }
    }
} 