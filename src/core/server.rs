use axum::Router;

use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::core::config::Config;
use crate::domain::product::routes::product_routes;

/// Start the HTTP server
pub async fn run_server(config: Config, pool: PgPool) -> anyhow::Result<()> {
    let app = create_router(pool);
    
    tracing::info!("Starting server on {}:{}", config.server_host, config.server_port);
    
    // Use the new axum::serve API
    let listener = tokio::net::TcpListener::bind(config.server_addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create API router with all routes
fn create_router(pool: PgPool) -> Router {
    // Configure CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build router with all routes
    Router::new()
        .merge(product_routes(pool))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}