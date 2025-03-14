use crate::core::{config::Config, db};
use crate::domain::product;
use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::Level;

/// Run the API server
pub async fn run(config: Config) -> Result<()> {
    // Set up database connection pool
    let pool = db::init_pool(&config.database_url).await?;
    
    // Set up tracing for logs
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    
    // Set up CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(Any)                  // Allow any origin
        .allow_methods(Any)                 // Allow any HTTP method
        .allow_headers(Any);                // Allow any headers
    
    // Set up tracing middleware for request logs
    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO));
    
    // Build application routes
    let app = Router::new()
        .merge(product::routes::product_routes(pool.clone()))  // Add product routes
        .layer(cors)                        // Apply CORS middleware
        .layer(trace);                      // Apply tracing middleware
    
    // Get socket address for server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    
    // Start server
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
        
    Ok(())
}