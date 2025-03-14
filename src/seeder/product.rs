use sqlx::PgPool;
use std::path::Path;
use tokio::fs;

use crate::domain::product::model::CreateProduct;
use crate::domain::product::repository::ProductRepository;

/// Seed product data from JSON file
pub async fn seed(pool: &PgPool) -> anyhow::Result<()> {
    let repo = ProductRepository::new(pool.clone());
    
    // Check if we need to seed by looking for existing products
    let filter = crate::domain::product::model::ProductFilter {
        name: None,
        category: None,
        min_price: None,
        max_price: None,
        is_active: None,
        limit: Some(1),
        offset: None,
    };
    
    let existing = repo.list(&filter).await?;
    if !existing.is_empty() {
        tracing::info!("Products already exist, skipping seed");
        return Ok(());
    }
    
    tracing::info!("Seeding products from JSON data...");
    
    // Path to JSON file
    let json_path = Path::new("data/products.json");
    
    // Read and parse JSON file
    let json_content = fs::read_to_string(json_path).await?;
    let products: Vec<CreateProduct> = serde_json::from_str(&json_content)?;
    
    // Insert all products
    // In the seed function, change:
    let products_count = products.len();
    for product in products {
        repo.create(&product).await?;
    }
    
    tracing::info!("Product seeding completed successfully. Added {} products", products_count);
    Ok(())
}