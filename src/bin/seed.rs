use learning_rust::core::config::Config;
use learning_rust::core::db::init_db;
use learning_rust::seeder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    println!("Starting seeder...");
    
    // Load config
    let config = Config::from_env();
    
    // Initialize database
    println!("Connecting to database...");
    let db_pool = init_db(&config.database_url).await?;
    println!("Database connection established");
    
    // Run all seeders
    println!("Running product seeder...");
    seeder::run_all_seeders(&db_pool).await?;
    
    println!("Seeding completed successfully");
    Ok(())
}