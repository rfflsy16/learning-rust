use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

/// Database connection pool type
pub type DbPool = Pool<Postgres>;

/// Initialize database connection pool
pub async fn init_pool(database_url: &str) -> Result<DbPool> {
    // Create connection pool with specified options
    let pool = PgPoolOptions::new()
        .max_connections(10)                  // Maximum number of connections in the pool
        .connect(database_url)                // Connect to database using URL
        .await?;                             // Await connection and propagate any errors
        
    // Run migrations to ensure database schema is up to date
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
        
    Ok(pool)
}