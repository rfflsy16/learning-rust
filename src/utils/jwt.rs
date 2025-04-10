use crate::core::error::ApiError;
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;
use chrono::{Utc, Duration};


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,         // Subject (user ID)
    exp: usize,          // Expiration time
    iat: usize,          // Issued at
}

pub fn generate_token(user_id: Uuid) -> Result<String, ApiError> {
    // Get JWT secret from environment or use default
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret".to_string());
    
    // Create claims
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::hours(24)).timestamp() as usize, // Token valid for 24 hours
    };
    
    // Encode token
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(format!("Token generation error: {}", e)))
}