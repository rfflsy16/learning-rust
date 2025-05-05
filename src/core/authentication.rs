use std::env;
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Algorithm, Validation};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

/// Struktur klaim JWT untuk dekoding token
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,  // User ID
    exp: usize,   // Waktu kedaluwarsa
    iat: usize,   // Waktu diterbitkan
}

/// Middleware untuk autentikasi token JWT
pub async fn auth_middleware(req: Request, next: Next) -> Response {
    // Cek path request - kalo gak dimulai dengan /api/products, skip auth
    let path = req.uri().path();
    if !path.starts_with("/api/products") {
        return next.run(req).await;
    }

    // Ambil token dari header Authorization
    let auth_header = req.headers().get(header::AUTHORIZATION);
    
    let token = match auth_header {
        Some(value) => {
            let auth_value = value.to_str().unwrap_or_default();
            // Token biasanya dalam format "Bearer {token}"
            if auth_value.starts_with("Bearer ") {
                &auth_value[7..]
            } else {
                return handle_unauthorized("Invalid authorization format".to_string());
            }
        }
        None => return handle_unauthorized("Missing authorization header".to_string()),
    };

    // Verifikasi token
    match verify_token(token) {
        Ok(user_id) => {
            info!("Authorized user: {}", user_id);
            // Kita bisa nambah user_id ke extensions kalo perlu akses di handler
            let mut modified_req = req;
            modified_req.extensions_mut().insert(user_id);
            next.run(modified_req).await
        }
        Err(error_msg) => handle_unauthorized(error_msg),
    }
}

/// Verifikasi token JWT dan ekstrak user ID
fn verify_token(token: &str) -> Result<Uuid, String> {
    // Ambil JWT secret dari environment
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "".to_string());
    
    if jwt_secret.is_empty() {
        error!("JWT_SECRET not set in environment");
        return Err("Server configuration error".to_string());
    }

    // Decode token
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(err) => {
            error!("Token validation error: {}", err);
            return Err("Invalid token".to_string());
        }
    };

    // Parse user ID dari subject token
    let user_id = match Uuid::parse_str(&token_data.claims.sub) {
        Ok(id) => id,
        Err(_) => return Err("Invalid user ID in token".to_string()),
    };

    Ok(user_id)
}

/// Helper untuk response unauthorized yg konsisten
fn handle_unauthorized(message: String) -> Response {
    let body = Json(serde_json::json!({
        "error": message
    }));
    
    (StatusCode::UNAUTHORIZED, body).into_response()
}
