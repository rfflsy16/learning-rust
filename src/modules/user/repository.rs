use crate::{
    core::{
        db::DbPool,
        error::ApiError,
    },
    modules::user::model::{CreateUser, User, UserFilter, UpdateUser},
    utils::offset_to_chrono,
};
use sqlx::{query_as, FromRow, postgres::PgRow, Row, query, QueryBuilder};
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use tracing::info;

// Konstanta SQL - menggunakan static str untuk menghindari alokasi
// const SELECT_USER_FIELDS: &str = "id, username, email, password, created_at, updated_at";
const SELECT_USER_BASE: &str = "SELECT id, username, email, password, created_at, updated_at FROM users";
const QUERY_FIND_BY_ID: &str = "SELECT id, username, email, password, created_at, updated_at FROM users WHERE id = $1";
const QUERY_FIND_BY_EMAIL: &str = "SELECT id, username, email, password, created_at, updated_at FROM users WHERE email = $1";
const QUERY_DELETE: &str = "DELETE FROM users WHERE id = $1";
const QUERY_CREATE: &str = "INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id, username, email, password, created_at, updated_at";
const QUERY_FOR_UPDATE: &str = "SELECT id, username, email, password, created_at, updated_at FROM users WHERE id = $1 FOR UPDATE";
const QUERY_UPDATE: &str = "UPDATE users SET username = $1, email = $2, password = $3, updated_at = NOW() WHERE id = $4 RETURNING id, username, email, password, created_at, updated_at";

/// Repository untuk operasi database pengguna
pub struct UserRepository {
    pool: DbPool,
}

// Implementasi FromRow untuk User untuk menangani konversi tipe khusus
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

impl UserRepository {
    /// Membuat repository pengguna baru
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Membuat pengguna baru di database (registrasi)
    pub async fn create(&self, user: &CreateUser) -> Result<User, ApiError> {
        // Hash password
        let password_hash = self.hash_password(&user.password)?;
        
        let created_user = query_as::<_, User>(QUERY_CREATE)
            .bind(&user.username)
            .bind(&user.email)
            .bind(&password_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(ApiError::Database)?;
        
        info!("Pengguna baru dibuat dengan ID: {}", created_user.id);
        Ok(created_user)
    }
    
    /// Mencari pengguna berdasarkan ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<User, ApiError> {
        query_as::<_, User>(QUERY_FIND_BY_ID)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("User with ID {} not found", id)))
    }
    
    /// Mencari pengguna berdasarkan email (untuk login)
    pub async fn find_by_email(&self, email: &str) -> Result<User, ApiError> {
        query_as::<_, User>(QUERY_FIND_BY_EMAIL)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound("Invalid email or password".to_string()))
    }
    
    /// Daftar pengguna dengan filter opsional
    pub async fn list(&self, filter: &UserFilter) -> Result<Vec<User>, ApiError> {
        // Mulai membangun query SQL dinamis
        let mut query_builder = QueryBuilder::new(SELECT_USER_BASE);
        query_builder.push(" WHERE 1=1");
        
        // Terapkan filter
        self.apply_filters(&mut query_builder, filter);
        
        // Eksekusi query dan konversi hasil
        query_builder
            .build_query_as::<User>()
            .fetch_all(&self.pool)
            .await
            .map_err(ApiError::Database)
    }
    
    /// Memperbarui pengguna yang ada
    pub async fn update(&self, id: Uuid, update: &UpdateUser) -> Result<User, ApiError> {
        let mut tx = self.pool.begin().await.map_err(ApiError::Database)?;
        
        // Periksa apakah pengguna ada dan dapatkan nilai saat ini
        let current = query_as::<_, User>(QUERY_FOR_UPDATE)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("User with ID {} not found", id)))?;
        
        // Siapkan nilai pembaruan
        let username = update.username.as_ref().unwrap_or(&current.username);
        let email = update.email.as_ref().unwrap_or(&current.email);
        
        // Hash password jika disediakan
        let password = match &update.password {
            Some(new_password) => self.hash_password(new_password)?,
            None => current.password.clone(),
        };
        
        // Eksekusi pembaruan
        let updated = query_as::<_, User>(QUERY_UPDATE)
            .bind(username)
            .bind(email)
            .bind(&password)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(ApiError::Database)?;
        
        tx.commit().await.map_err(ApiError::Database)?;
        info!("User with ID: {} successfully updated", id);
        
        Ok(updated)
    }
    
    /// Menghapus pengguna berdasarkan ID
    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let result = query(QUERY_DELETE)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(ApiError::Database)?;
        
        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("User with ID {} not found", id)));
        }
        
        info!("User with ID: {} successfully deleted", id);
        Ok(())
    }
    
    /// Hash password menggunakan Argon2
    fn hash_password(&self, password: &str) -> Result<String, ApiError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2.hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| ApiError::Internal(format!("Error hashing password: {}", e)))
    }
    
    /// Verifikasi password dengan hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ApiError> {
        PasswordHash::new(hash)
            .map_err(|e| ApiError::Internal(format!("Error parsing password: {}", e)))
            .map(|parsed_hash| Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    
    /// Menerapkan filter ke query builder
    fn apply_filters<'a>(&self, query_builder: &mut QueryBuilder<'a, sqlx::Postgres>, filter: &'a UserFilter) {
        if let Some(username) = &filter.username {
            query_builder.push(" AND username ILIKE ");
            query_builder.push_bind(format!("%{}%", username));
        }
        
        if let Some(email) = &filter.email {
            query_builder.push(" AND email ILIKE ");
            query_builder.push_bind(format!("%{}%", email));
        }
        
        // Terapkan pengurutan, limit dan offset
        query_builder.push(" ORDER BY username ASC");
        
        if let Some(limit) = filter.limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }
        
        if let Some(offset) = filter.offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
        }
    }
}