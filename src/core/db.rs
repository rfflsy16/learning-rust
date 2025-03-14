use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Type alias for database connection pool
pub type DbPool = PgPool;

/// Initialize database connection pool
pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}