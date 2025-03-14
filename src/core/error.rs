use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Custom API error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),         // Database-related errors
    
    #[error("Not found: {0}")]
    NotFound(String),                     // Resource not found
    
    #[error("Bad request: {0}")]
    BadRequest(String),                   // Invalid client request
    
    #[error("Internal server error: {0}")]
    Internal(String),                     // Internal server errors
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),                 // Authentication errors
}

/// Convert API errors into HTTP responses
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            ApiError::NotFound(ref message) => (StatusCode::NOT_FOUND, message.clone()),
            ApiError::BadRequest(ref message) => (StatusCode::BAD_REQUEST, message.clone()),
            ApiError::Internal(ref message) => {
                tracing::error!("Internal error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            ApiError::Unauthorized(ref message) => (StatusCode::UNAUTHORIZED, message.clone()),
        };

        // Create a JSON response with the error details
        let body = Json(json!({
            "error": {
                "message": error_message,
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}