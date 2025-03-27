use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// API error types
#[derive(Error, Debug)]
#[allow(dead_code)]  // Suppress warnings about unused variants
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Database(ref err) => {
                tracing::error!("Database error: {:?}", err);
                
                // Check for unique constraint violations
                let err_string = err.to_string();
                if err_string.contains("duplicate key") {
                    if err_string.contains("idx_users_email") || err_string.contains("users_email_key") {
                        return (StatusCode::CONFLICT, "Email already in use".to_string()).into_response();
                    } else if err_string.contains("idx_users_username") || err_string.contains("users_username_key") {
                        return (StatusCode::CONFLICT, "Username already in use".to_string()).into_response();
                    }
                }
                
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
            Self::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}