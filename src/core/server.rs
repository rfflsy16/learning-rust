use axum::{
    http::{Method, HeaderName, HeaderValue},
    Router,
};
use std::str::FromStr;
use sqlx::PgPool;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use anyhow::Result;

use crate::{
    core::config::Config,
    modules::{
        product::routes::product_routes,
        user::routes::user_routes,
    },
};

const ALLOWED_ORIGINS: [&str; 1] = ["http://localhost:8080"];
const ALLOWED_HEADERS: [&str; 2] = ["content-type", "authorization"];

/// Start the HTTP server
pub async fn run_server(config: Config, pool: PgPool) -> Result<()> {
    let app = create_router(pool);
    
    tracing::info!("Starting server on {}:{}", config.server_host, config.server_port);
    
    let listener = tokio::net::TcpListener::bind(config.server_addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create API router with all routes
fn create_router(pool: PgPool) -> Router {
    let cors = create_cors_layer();
    
    Router::new()
        .merge(product_routes(pool.clone()))
        .merge(user_routes(pool))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Create CORS layer with configured restrictions
fn create_cors_layer() -> CorsLayer {
    let allowed_origins: Vec<HeaderValue> = ALLOWED_ORIGINS
        .iter()
        .filter_map(|origin| {
            let header_value = origin.parse::<HeaderValue>().ok()?;
            tracing::info!("Allowing CORS origin: {}", origin);
            Some(header_value)
        })
        .collect();

    let allowed_headers: Vec<HeaderName> = ALLOWED_HEADERS
        .iter()
        .filter_map(|header| HeaderName::from_str(header).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers(allowed_headers)
        .allow_credentials(true)
}