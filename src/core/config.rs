use anyhow::Result;
use dotenv::dotenv;
use std::env;

/// Application configuration
pub struct Config {
    pub database_url: String,    // Database connection string
    pub server_port: u16,        // Server port number
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        // Load .env file if present
        dotenv().ok();
        
        // Read environment variables
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
            
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid port number");
            
        Ok(Config {
            database_url,
            server_port,
        })
    }
}