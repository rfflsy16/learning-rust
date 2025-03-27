use crate::core::error::ApiError;
use crate::modules::user::model::{AuthResponse, CreateUser, LoginUser, UpdateUser, UserFilter, UserResponse};
use crate::modules::user::repository::UserRepository;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use std::env;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,         // Subject (user ID)
    exp: usize,          // Expiration time
    iat: usize,          // Issued at
}

/// User HTTP request handlers
pub struct UserHandler {
    repository: UserRepository,   // User repository for database operations
}

impl UserHandler {
    /// Create a new user handler
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }
    
    /// Handler to register a new user
    pub async fn register(
        &self,
        user: CreateUser,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Validate email format
        if !self.is_valid_email(&user.email) {
            return Err(ApiError::BadRequest("Invalid email format".to_string()));
        }
        
        // Validate password strength
        if user.password.len() < 8 {
            return Err(ApiError::BadRequest("Password must be at least 8 characters".to_string()));
        }
        
        // Validate username and email uniqueness
        self.validate_unique_fields(&user.username, &user.email).await?;
        
        // Create user in database
        let created_user = self.repository.create(&user).await?;
        
        // Convert to UserResponse to avoid sending password
        let user_response = UserResponse::from(created_user);
        
        Ok((StatusCode::CREATED, Json(user_response)))
    }
    
    /// Handler for user login
    pub async fn login(
        &self,
        login: LoginUser,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Find user by email
        let user = self.repository.find_by_email(&login.email).await?;
        
        // Verify password
        if !self.repository.verify_password(&login.password, &user.password)? {
            return Err(ApiError::Unauthorized("Invalid email or password".to_string()));
        }
        
        // Generate JWT token
        let token = self.generate_token(user.id)?;
        
        // Create response
        let auth_response = AuthResponse {
            user: UserResponse::from(user),
            token,
        };
        
        Ok(Json(auth_response))
    }
    
    /// Handler to get a user by ID
    pub async fn get_user(
        &self,
        id: Uuid,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        let user = self.repository.find_by_id(id).await?;
        Ok(Json(UserResponse::from(user)))
    }
    
    /// Handler to list users with optional filtering
    pub async fn list_users(
        &self,
        filter: UserFilter,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        let users = self.repository.list(&filter).await?;
        
        // Convert to UserResponse to avoid sending passwords
        let user_responses: Vec<UserResponse> = users.into_iter()
            .map(UserResponse::from)
            .collect();
            
        Ok(Json(user_responses))
    }
    
    /// Handler to update a user
    pub async fn update_user(
        &self,
        id: Uuid,
        update: UpdateUser,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Get current user to compare with updates
        let current_user = self.repository.find_by_id(id).await?;
        
        // Validate email if provided
        if let Some(ref email) = update.email {
            if !self.is_valid_email(email) {
                return Err(ApiError::BadRequest("Invalid email format".to_string()));
            }
            
            // Only validate uniqueness if email is changing
            if email != &current_user.email {
                let email_filter = UserFilter {
                    username: None,
                    email: Some(email.to_string()),
                    limit: Some(1),
                    offset: None,
                };
                
                let email_exists = self.repository.list(&email_filter).await?;
                if !email_exists.is_empty() {
                    return Err(ApiError::BadRequest("Email already in use".to_string()));
                }
            }
        }
        
        // Validate username if provided
        if let Some(ref username) = update.username {
            // Only validate uniqueness if username is changing
            if username != &current_user.username {
                let username_filter = UserFilter {
                    username: Some(username.to_string()),
                    email: None,
                    limit: Some(1),
                    offset: None,
                };
                
                let username_exists = self.repository.list(&username_filter).await?;
                if !username_exists.is_empty() {
                    return Err(ApiError::BadRequest("Username already in use".to_string()));
                }
            }
        }
        
        // Validate password if provided
        if let Some(ref password) = update.password {
            if password.len() < 8 {
                return Err(ApiError::BadRequest("Password must be at least 8 characters".to_string()));
            }
        }
        
        let updated_user = self.repository.update(id, &update).await?;
        Ok(Json(UserResponse::from(updated_user)))
    }
    
    /// Handler to delete a user
    pub async fn delete_user(
        &self,
        id: Uuid,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        self.repository.delete(id).await?;
        
        Ok((
            StatusCode::OK, 
            Json(json!({
                "success": true,
                "message": format!("User with ID {} successfully deleted", id)
            }))
        ))
    }
    
    /// Validate that username and email are unique
    async fn validate_unique_fields(&self, username: &str, email: &str) -> Result<(), ApiError> {
        // Check if username exists
        let username_filter = UserFilter {
            username: Some(username.to_string()),
            email: None,
            limit: Some(1),
            offset: None,
        };
        
        let username_exists = self.repository.list(&username_filter).await?;
        if !username_exists.is_empty() {
            return Err(ApiError::BadRequest("Username already in use".to_string()));
        }
        
        // Check if email exists
        let email_filter = UserFilter {
            username: None,
            email: Some(email.to_string()),
            limit: Some(1),
            offset: None,
        };
        
        let email_exists = self.repository.list(&email_filter).await?;
        if !email_exists.is_empty() {
            return Err(ApiError::BadRequest("Email already in use".to_string()));
        }
        
        Ok(())
    }
    
    // Helper method to validate email format
    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation
        email.contains('@') && email.contains('.')
    }
    
    // Generate JWT token
    fn generate_token(&self, user_id: Uuid) -> Result<String, ApiError> {
        // Get JWT secret from environment or use default
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret".to_string());
        
        // Create claims
        let now = Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now + Duration::hours(24)).timestamp() as usize; // Token valid for 24 hours
        
        let claims = Claims {
            sub: user_id.to_string(),
            iat,
            exp,
        };
        
        // Encode token
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|e| ApiError::Internal(format!("Token generation error: {}", e)))?;
        
        Ok(token)
    }
}