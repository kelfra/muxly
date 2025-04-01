mod routes;
mod handlers;
mod middleware;
pub mod health;
pub mod openapi;

use axum::{
    Router,
    routing::{get, post, put, delete},
};

pub fn api_router() -> Router {
    // Start with the regular routes
    let router = Router::new()
        .merge(health::routes())  // Use our new documented health routes
        .merge(connector_routes())
        .merge(output_routes());
    
    // Add OpenAPI documentation routes
    openapi::add_documentation_routes(router)
}

// These routes will be replaced/updated with OpenAPI documentation in the future
pub mod connectors {
    // Placeholder that will be documented later
    pub async fn list_connectors() {}
    pub async fn get_connector() {}
    pub async fn create_connector() {}
    pub async fn update_connector() {}
    pub async fn delete_connector() {}
    pub async fn test_connection() {}
    
    // Placeholder schema types for OpenAPI
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ConnectorRequest {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ConnectorResponse {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ConnectorListResponse {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct ConnectionTestResponse {}
}

pub mod router {
    // Placeholder that will be documented later
    pub async fn list_routes() {}
    pub async fn get_route() {}
    pub async fn create_route() {}
    pub async fn update_route() {}
    pub async fn delete_route() {}
    
    // Placeholder schema types for OpenAPI
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct RouteRequest {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct RouteResponse {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct RouteListResponse {}
}

pub mod scheduler {
    // Placeholder that will be documented later
    pub async fn list_jobs() {}
    pub async fn get_job() {}
    pub async fn create_job() {}
    pub async fn update_job() {}
    pub async fn delete_job() {}
    pub async fn run_job() {}
    pub async fn get_job_executions() {}
    
    // Placeholder schema types for OpenAPI
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct JobRequest {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct JobResponse {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct JobListResponse {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct JobExecutionResponse {}
    
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct JobExecutionListResponse {}
}

fn connector_routes() -> Router {
    Router::new()
        .route("/connectors", get(handlers::connectors::list_connectors))
        .route("/connectors", post(handlers::connectors::create_connector))
        .route("/connectors/:id", get(handlers::connectors::get_connector))
        .route("/connectors/:id", put(handlers::connectors::update_connector))
        .route("/connectors/:id", delete(handlers::connectors::delete_connector))
        .route("/connectors/:id/test", post(handlers::connectors::test_connector))
        .route("/connectors/:id/sync", post(handlers::connectors::trigger_sync))
}

fn output_routes() -> Router {
    Router::new()
        .route("/outputs", get(handlers::outputs::list_outputs))
        .route("/outputs", post(handlers::outputs::create_output))
        .route("/outputs/:id", get(handlers::outputs::get_output))
        .route("/outputs/:id", put(handlers::outputs::update_output))
        .route("/outputs/:id", delete(handlers::outputs::delete_output))
} 