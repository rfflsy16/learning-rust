use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User model representing a user in the database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,         // Unique identifier
    pub username: String, // Username
    pub email: String,    // Email address
    #[serde(skip_serializing)] // Don't include password in JSON responses
    pub password: String, // Hashed password
    pub created_at: DateTime<Utc>, // Creation timestamp
    pub updated_at: DateTime<Utc>, // Last update timestamp
}

/// DTO for creating a new user (registration)
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String, // Required username
    pub email: String,    // Required email
    pub password: String, // Required password (will be hashed)
}

/// DTO for user login
#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,    // Email for login
    pub password: String, // Password for login
}

/// DTO for updating an existing user
#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>, // Optional username update
    pub email: Option<String>,    // Optional email update
    pub password: Option<String>, // Optional password update
}

/// Filter criteria for querying users
#[derive(Debug, Deserialize)]
pub struct UserFilter {
    pub username: Option<String>, // Filter by username (partial match)
    pub email: Option<String>,    // Filter by email (partial match)
    pub limit: Option<i64>,       // Maximum number of results
    pub offset: Option<i64>,      // Number of results to skip
}

/// Response for successful login
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse, // User information
    pub token: String,      // JWT token
}

/// User response without sensitive information
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,                  // User ID
    pub username: String,          // Username
    pub email: String,             // Email
    pub created_at: DateTime<Utc>, // Creation timestamp
    pub updated_at: DateTime<Utc>, // Last update timestamp
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
