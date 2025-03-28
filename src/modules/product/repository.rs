use crate::core::db::DbPool;
use crate::core::error::ApiError;
use crate::modules::product::model::{CreateProduct, Product, ProductFilter, UpdateProduct};
use crate::utils::offset_to_chrono;
use sqlx::{query, query_as, FromRow, postgres::PgRow, Row};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use std::str::FromStr;

/// Repository for product database operations
pub struct ProductRepository {
    pool: DbPool,
}

// Implement FromRow for Product to handle custom type conversions
impl<'r> FromRow<'r, PgRow> for Product {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        
        // Get description with additional error handling
        let description: Option<String> = match row.try_get("description") {
            Ok(desc) => desc,
            Err(e) => {
                tracing::warn!("Error getting description field: {}", e);
                None // Fallback to None if there's an error
            }
        };
        
        // Convert BigDecimal to f64 more efficiently
        let price_decimal: BigDecimal = row.try_get("price")?;
        let price = f64::from_str(&price_decimal.to_string()).unwrap_or(0.0);
        
        // Get remaining fields
        Ok(Product {
            id,
            name,
            description,
            price,
            stock: row.try_get("stock")?,
            category: row.try_get("category")?,
            is_active: row.try_get("is_active")?,
            created_at: offset_to_chrono(row.try_get("created_at")?),
            updated_at: offset_to_chrono(row.try_get("updated_at")?),
        })
    }
}

// SQL query constants
static SELECT_PRODUCT_FIELDS: &str = "id, name, description, price, stock, category, is_active, created_at, updated_at";
static SELECT_PRODUCT_BASE: &str = "SELECT id, name, description, price, stock, category, is_active, created_at, updated_at FROM products";

impl ProductRepository {
    /// Create a new product repository
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new product in the database
    pub async fn create(&self, product: &CreateProduct) -> Result<Product, ApiError> {
        let query_str = format!(
            "INSERT INTO products (name, description, price, stock, category) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING {SELECT_PRODUCT_FIELDS}"
        );
        
        query(&query_str)
            .bind(&product.name)
            .bind(product.description.as_deref())  
            .bind(product.price)
            .bind(product.stock.unwrap_or(0))
            .bind(product.category.as_deref())  
            .fetch_one(&self.pool)
            .await
            .map_err(ApiError::Database)
            .and_then(|row| Product::from_row(&row).map_err(ApiError::Database))
    }
    
    /// Get a product by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Product, ApiError> {
        let query_str = format!("{SELECT_PRODUCT_BASE} WHERE id = $1");
        
        query_as::<_, Product>(&query_str)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))
    }
    
    /// List products with optional filtering
    pub async fn list(&self, filter: &ProductFilter) -> Result<Vec<Product>, ApiError> {
        // Start building the dynamic SQL query
        let mut query_builder = sqlx::QueryBuilder::new(
            format!("{SELECT_PRODUCT_BASE} WHERE 1=1")
        );
        
        // Apply filters
        if let Some(name) = &filter.name {
            query_builder.push(" AND name ILIKE ");
            query_builder.push_bind(format!("%{}%", name));
        }
        
        if let Some(category) = &filter.category {
            query_builder.push(" AND category = ");
            query_builder.push_bind(category);
        }
        
        if let Some(min_price) = filter.min_price {
            query_builder.push(" AND price >= ");
            query_builder.push_bind(min_price);
        }
        
        if let Some(max_price) = filter.max_price {
            query_builder.push(" AND price <= ");
            query_builder.push_bind(max_price);
        }
        
        if let Some(is_active) = filter.is_active {
            query_builder.push(" AND is_active = ");
            query_builder.push_bind(is_active);
        }
        
        // Apply order, limit and offset
        query_builder.push(" ORDER BY name ASC");
        
        if let Some(limit) = filter.limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }
        
        if let Some(offset) = filter.offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
        }
        
        // Execute query and convert results directly to Product structs
        query_builder
            .build_query_as::<Product>()
            .fetch_all(&self.pool)
            .await
            .map_err(ApiError::Database)
    }
    
    /// Update an existing product
    pub async fn update(&self, id: Uuid, update: &UpdateProduct) -> Result<Product, ApiError> {
        let mut tx = self.pool.begin().await.map_err(ApiError::Database)?;
        
        // Check if product exists and get current values
        let query_str = format!("{SELECT_PRODUCT_BASE} WHERE id = $1 FOR UPDATE");
        
        let current = query_as::<_, Product>(&query_str)
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))?;
        
        // Prepare update values using more concise approach
        let name = update.name.as_ref().unwrap_or(&current.name);
        let description_str = update.description.as_deref().or_else(|| current.description.as_deref());
        let price = update.price.unwrap_or(current.price);
        let stock = update.stock.unwrap_or(current.stock);
        let category_str = update.category.as_deref().or_else(|| current.category.as_deref());
        let is_active = update.is_active.unwrap_or(current.is_active);
        
        // Execute update
        let query_str = format!(
            "UPDATE products
            SET name = $1, description = $2, price = $3, stock = $4, category = $5, is_active = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING {SELECT_PRODUCT_FIELDS}"
        );
        
        let updated = query_as::<_, Product>(&query_str)
            .bind(name)
            .bind(description_str)
            .bind(price)
            .bind(stock)
            .bind(category_str)
            .bind(is_active)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(ApiError::Database)?;
        
        tx.commit().await.map_err(ApiError::Database)?;
        
        Ok(updated)
    }
    
    /// Delete a product by ID
    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let result = query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(ApiError::Database)?;
        
        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("Product with ID {} not found", id)));
        }
        
        Ok(())
    }
}