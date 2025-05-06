use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::utils::verify_token;
use crate::core::error::ApiError;

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
                return ApiError::Unauthorized("Invalid authorization format".to_string()).into_response();
            }
        }
        None => return ApiError::Unauthorized("Missing authorization header".to_string()).into_response(),
    };

    // Verifikasi token dgn fungsi dari utils
    match verify_token(token) {
        Ok(user_id) => {
            info!("Authorized user: {}", user_id);
            // Kita bisa nambah user_id ke extensions kalo perlu akses di handler
            let mut modified_req = req;
            modified_req.extensions_mut().insert(user_id);
            next.run(modified_req).await
        }
        Err(error) => ApiError::Unauthorized(error.to_string()).into_response(),
    }
}
