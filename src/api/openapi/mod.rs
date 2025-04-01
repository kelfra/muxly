use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use axum::Router;

// Import API components
use crate::api::health;
use crate::api::connectors;
use crate::api::router;
use crate::api::scheduler;

// Generate the OpenAPI specification
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health endpoints
        health::health_check,
        
        // Connector endpoints
        connectors::list_connectors,
        connectors::get_connector,
        connectors::create_connector,
        connectors::update_connector,
        connectors::delete_connector,
        connectors::test_connection,
        
        // Router endpoints
        router::list_routes,
        router::get_route,
        router::create_route,
        router::update_route,
        router::delete_route,
        
        // Scheduler endpoints
        scheduler::list_jobs,
        scheduler::get_job,
        scheduler::create_job,
        scheduler::update_job,
        scheduler::delete_job,
        scheduler::run_job,
        scheduler::get_job_executions,
    ),
    components(
        schemas(
            // Health schemas
            health::HealthResponse,
            
            // Connector schemas
            connectors::ConnectorRequest,
            connectors::ConnectorResponse,
            connectors::ConnectorListResponse,
            connectors::ConnectionTestResponse,
            
            // Router schemas
            router::RouteRequest,
            router::RouteResponse,
            router::RouteListResponse,
            
            // Scheduler schemas
            scheduler::JobRequest,
            scheduler::JobResponse,
            scheduler::JobListResponse,
            scheduler::JobExecutionResponse,
            scheduler::JobExecutionListResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "connectors", description = "Connector management endpoints"),
        (name = "router", description = "Router management endpoints"),
        (name = "scheduler", description = "Scheduler management endpoints"),
    ),
    info(
        title = "Muxly API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Muxly API for collecting, unifying, and routing product metrics",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        ),
        contact(
            name = "Muxly Team",
            url = "https://github.com/kelfra/muxly"
        ),
    )
)]
pub struct ApiDoc;

// Mount the Swagger UI
pub fn mount_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}

// Add documentation routes to the API
pub fn add_documentation_routes(router: Router) -> Router {
    router.merge(mount_swagger_ui())
} 