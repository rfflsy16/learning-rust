use std::env;
use std::net::SocketAddr;

/// Application configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub server_addr: SocketAddr,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        // Load .env file if present
        dotenv::dotenv().ok();

        // Get database URL or panic if not set
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        // Get server host or use default
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        // Get server port or use default
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid port number");

        // Combine host and port into socket address
        let server_addr = format!("{}:{}", server_host, server_port)
            .parse()
            .expect("Failed to parse server address");

        Self {
            database_url,
            server_host,
            server_port,
            server_addr,
        }
    }
}
