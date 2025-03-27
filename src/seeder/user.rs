use sqlx::PgPool;
use std::path::Path;
use tokio::fs;

use crate::modules::user::model::CreateUser;
use crate::modules::user::repository::UserRepository;

/// Seed user data from JSON file
pub async fn seed(pool: &PgPool) -> anyhow::Result<()> {
    let repo = UserRepository::new(pool.clone());
    
    // Check if we need to seed by looking for existing users
    let filter = crate::modules::user::model::UserFilter {
        username: None,
        email: None,
        limit: Some(1),
        offset: None,
    };
    
    let existing = repo.list(&filter).await?;
    if !existing.is_empty() {
        tracing::info!("Users already exist, skipping seed");
        return Ok(());
    }
    
    tracing::info!("Seeding users from JSON data...");
    
    // Path to JSON file
    let json_path = Path::new("data/users.json");
    
    // Read and parse JSON file
    let json_content = fs::read_to_string(json_path).await?;
    let users: Vec<CreateUser> = serde_json::from_str(&json_content)?;
    
    // Insert all users
    let users_count = users.len();
    for user in users {
        repo.create(&user).await?;
    }
    
    tracing::info!("User seeding completed successfully. Added {} users", users_count);
    Ok(())
}