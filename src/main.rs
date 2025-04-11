mod core;
mod modules;
mod seeder;
mod utils;

use crate::core::config::Config;
use crate::core::db::init_db;
use crate::core::server::run_server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = Config::from_env();

    // Initialize database
    tracing::info!("Connecting to database...");
    let db_pool = init_db(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Optional: Run seeders in development
    #[cfg(debug_assertions)]
    {
        tracing::info!("Running seeders in development mode");
        seeder::run_all_seeders(&db_pool).await?;
    }

    // Run server
    run_server(config, db_pool).await?;

    Ok(())
}
