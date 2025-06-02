use anyhow::Result;
use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method},
    middleware,
};
use sqlx::PgPool;
use std::{env, str::FromStr};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    core::{authentication::auth_middleware, config::Config},
    modules::{product::routes::product_routes, user::routes::user_routes},
};

const ALLOWED_HEADERS: [&str; 2] = ["content-type", "authorization"];

/// Start the HTTP server
pub async fn run_server(config: Config, pool: PgPool) -> Result<()> {
    let app = create_router(pool);

    tracing::info!(
        "Starting server on {}:{}",
        config.server_host,
        config.server_port
    );

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
        .layer(middleware::from_fn(auth_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Create CORS layer with configured restrictions
fn create_cors_layer() -> CorsLayer {
    let allowed_origins_str = env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "".to_string());

    let origins: Vec<&str> = allowed_origins_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let allowed_origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| match origin.parse::<HeaderValue>() {
            Ok(header_value) => {
                tracing::info!("Allowing CORS origin: {}", origin);
                Some(header_value)
            }
            Err(e) => {
                tracing::warn!("Failed to parse CORS origin '{}': {}", origin, e);
                None
            }
        })
        .collect();

    let allowed_headers: Vec<HeaderName> = ALLOWED_HEADERS
        .iter()
        .filter_map(|header| HeaderName::from_str(header).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(allowed_headers)
        .allow_credentials(true)
}
