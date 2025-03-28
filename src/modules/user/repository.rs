use crate::core::db::DbPool;
use crate::core::error::ApiError;
use crate::modules::user::model::{CreateUser, User, UserFilter, UpdateUser};
use crate::utils::offset_to_chrono;
use sqlx::{query, query_as, FromRow, postgres::PgRow, Row};
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

/// Repository for user database operations
pub struct UserRepository {
    pool: DbPool,
}

// Implement FromRow for User to handle custom type conversions
impl<'r> FromRow<'r, PgRow> for User {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            password: row.try_get("password")?,
            created_at: offset_to_chrono(row.try_get("created_at")?),
            updated_at: offset_to_chrono(row.try_get("updated_at")?),
        })
    }
}

// SQL query constants - using static str to avoid allocations
static SELECT_USER_FIELDS: &str = "id, username, email, password, created_at, updated_at";
static SELECT_USER_BASE: &str = "SELECT id, username, email, password, created_at, updated_at FROM users";

impl UserRepository {
    /// Create a new user repository
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new user in the database (registration)
    pub async fn create(&self, user: &CreateUser) -> Result<User, ApiError> {
        // Hash the password
        let password_hash = self.hash_password(&user.password)?;
        
        let query_str = format!(
            "INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING {SELECT_USER_FIELDS}"
        );
        
        query(&query_str)
            .bind(&user.username)
            .bind(&user.email)
            .bind(&password_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| self.handle_db_error(e))
            .and_then(|row| User::from_row(&row).map_err(ApiError::Database))
    }
    
    /// Find a user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<User, ApiError> {
        let query_str = format!("{SELECT_USER_BASE} WHERE id = $1");
        
        query_as::<_, User>(&query_str)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("User with ID {} not found", id)))
    }
    
    /// Find a user by email (for login)
    pub async fn find_by_email(&self, email: &str) -> Result<User, ApiError> {
        let query_str = format!("{SELECT_USER_BASE} WHERE email = $1");
        
        query_as::<_, User>(&query_str)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound("Invalid email or password".to_string()))
    }
    
    /// List users with optional filtering
    pub async fn list(&self, filter: &UserFilter) -> Result<Vec<User>, ApiError> {
        // Start building the dynamic SQL query with capacity hint
        let mut query_builder = sqlx::QueryBuilder::new(
            format!("{SELECT_USER_BASE} WHERE 1=1")
        );
        
        // Apply filters
        if let Some(username) = &filter.username {
            query_builder.push(" AND username ILIKE ");
            query_builder.push_bind(format!("%{}%", username));
        }
        
        if let Some(email) = &filter.email {
            query_builder.push(" AND email ILIKE ");
            query_builder.push_bind(format!("%{}%", email));
        }
        
        // Apply order, limit and offset
        query_builder.push(" ORDER BY username ASC");
        
        if let Some(limit) = filter.limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }
        
        if let Some(offset) = filter.offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
        }
        
        // Execute query and convert results
        query_builder
            .build_query_as::<User>()
            .fetch_all(&self.pool)
            .await
            .map_err(ApiError::Database)
    }
    
    /// Update an existing user
    pub async fn update(&self, id: Uuid, update: &UpdateUser) -> Result<User, ApiError> {
        let mut tx = self.pool.begin().await.map_err(ApiError::Database)?;
        
        // Check if user exists and get current values
        let query_str = format!("{SELECT_USER_BASE} WHERE id = $1 FOR UPDATE");
        
        let current = query_as::<_, User>(&query_str)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("User with ID {} not found", id)))?;
        
        // Prepare update values
        let username = update.username.as_ref().unwrap_or(&current.username);
        let email = update.email.as_ref().unwrap_or(&current.email);
        
        // Hash password if provided
        let password = match &update.password {
            Some(new_password) => self.hash_password(new_password)?,
            None => current.password.clone(),
        };
        
        // Execute update
        let query_str = format!(
            "UPDATE users SET username = $1, email = $2, password = $3, updated_at = NOW() 
            WHERE id = $4 RETURNING {SELECT_USER_FIELDS}"
        );
        
        let updated = query_as::<_, User>(&query_str)
            .bind(username)
            .bind(email)
            .bind(&password)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| self.handle_db_error(e))?;
        
        tx.commit().await.map_err(ApiError::Database)?;
        
        Ok(updated)
    }
    
    /// Delete a user by ID
    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let result = query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(ApiError::Database)?;
        
        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("User with ID {} not found", id)));
        }
        
        Ok(())
    }
    
    /// Hash a password using Argon2
    fn hash_password(&self, password: &str) -> Result<String, ApiError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2.hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| ApiError::Internal(format!("Password hashing error: {}", e)))
    }
    
    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ApiError> {
        PasswordHash::new(hash)
            .map_err(|e| ApiError::Internal(format!("Password parsing error: {}", e)))
            .map(|parsed_hash| Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    
    /// Handle database errors with common patterns
    fn handle_db_error(&self, error: sqlx::Error) -> ApiError {
        let error_string = error.to_string();
        if error_string.contains("duplicate key") {
            if error_string.contains("idx_users_email") {
                ApiError::BadRequest("Email already in use".to_string())
            } else if error_string.contains("idx_users_username") {
                ApiError::BadRequest("Username already in use".to_string())
            } else {
                ApiError::Database(error)
            }
        } else {
            ApiError::Database(error)
        }
    }
}