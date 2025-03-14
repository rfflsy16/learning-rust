use sqlx::PgPool;

pub mod product;

/// Run all data seeders
pub async fn run_all_seeders(pool: &PgPool) -> anyhow::Result<()> {
    // Run product seeder
    product::seed(pool).await?;
    
    Ok(())
}