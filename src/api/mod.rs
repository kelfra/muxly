mod routes;
mod handlers;
mod middleware;

use axum::{
    Router,
    routing::{get, post, put, delete},
};

pub fn api_router() -> Router {
    Router::new()
        .merge(health_routes())
        .merge(connector_routes())
        .merge(output_routes())
}

fn health_routes() -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/detailed", get(handlers::health::detailed_health_check))
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