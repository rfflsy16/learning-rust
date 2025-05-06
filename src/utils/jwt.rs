use crate::core::error::ApiError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation, encode};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;
use tracing::error;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // Subject (user ID)
    exp: usize,  // Expiration time
    iat: usize,  // Issued at
}

pub fn generate_token(user_id: Uuid) -> Result<String, ApiError> {
    // Get JWT secret from environment or use default
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "".to_string());

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

/// Verifikasi token JWT dan ekstrak user ID
pub fn verify_token(token: &str) -> Result<Uuid, ApiError> {
    // Ambil JWT secret dari environment
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "".to_string());
    
    if jwt_secret.is_empty() {
        error!("JWT_SECRET not set in environment");
        return Err(ApiError::Internal("Server configuration error".to_string()));
    }

    // Decode token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|err| {
        error!("Token validation error: {}", err);
        ApiError::Unauthorized("Invalid token".to_string())
    })?;

    // Parse user ID dari subject token
    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))?;

    Ok(user_id)
}
