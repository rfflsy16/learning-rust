use crate::core::error::ApiError;
use crate::modules::user::model::{AuthResponse, CreateUser, LoginUser, UpdateUser, UserFilter, UserResponse};
use crate::modules::user::repository::UserRepository;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;
use regex::Regex;
use once_cell::sync::Lazy;
use crate::utils::generate_token;
// Email validation regex using Lazy static
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

/// User HTTP request handlers
pub struct UserService {
    repository: UserRepository,   // User repository for database operations
}

impl UserService {
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
        self.validate_password(&user.password)?;
        
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
        
        // Generate JWT token and create response
        let token = generate_token(user.id)?;
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
                self.check_field_exists(None, Some(email)).await?;
            }
        }
        
        // Validate username if provided
        if let Some(ref username) = update.username {
            // Only validate uniqueness if username is changing
            if username != &current_user.username {
                self.check_field_exists(Some(username), None).await?;
            }
        }
        
        // Validate password if provided
        if let Some(ref password) = update.password {
            self.validate_password(password)?;
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
        // Check username and email existence
        self.check_field_exists(Some(username), None).await?;
        self.check_field_exists(None, Some(email)).await?;
        
        Ok(())
    }
    
    /// Helper method to check if a field exists
    async fn check_field_exists(&self, username: Option<&str>, email: Option<&str>) -> Result<(), ApiError> {
        let filter = UserFilter {
            username: username.map(ToString::to_string),
            email: email.map(ToString::to_string),
            limit: Some(1),
            offset: None,
        };
        
        let exists = self.repository.list(&filter).await?;
        if !exists.is_empty() {
            let field_name = if username.is_some() { "Username" } else { "Email" };
            return Err(ApiError::BadRequest(format!("{} already in use", field_name)));
        }
        
        Ok(())
    }
    
    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<(), ApiError> {
        if password.len() < 8 {
            return Err(ApiError::BadRequest("Password must be at least 8 characters".to_string()));
        }
        Ok(())
    }
    
    /// Helper method to validate email format
    fn is_valid_email(&self, email: &str) -> bool {
        EMAIL_REGEX.is_match(email)
    }
}