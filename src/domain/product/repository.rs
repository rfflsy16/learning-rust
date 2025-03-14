use crate::core::db::DbPool;
use crate::core::error::ApiError;
use crate::domain::product::model::{CreateProduct, Product, ProductFilter, UpdateProduct};
use sqlx::{postgres::PgQueryResult, query, query_as};
use uuid::Uuid;

/// Repository for product database operations
pub struct ProductRepository {
    pool: DbPool,    // Database connection pool
}

impl ProductRepository {
    /// Create a new product repository
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new product in the database
    pub async fn create(&self, product: &CreateProduct) -> Result<Product, ApiError> {
        // SQL query to insert new product
        let result = query_as!(
            Product,
            r#"
            INSERT INTO products (name, description, price, stock, category)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, price, stock, category, is_active, created_at, updated_at
            "#,
            product.name,
            product.description,
            product.price,
            product.stock.unwrap_or(0),
            product.category
        )
        .fetch_one(&self.pool)
        .await
        .map_err(ApiError::Database)?;
        
        Ok(result)
    }
    
    /// Get a product by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Product, ApiError> {
        // SQL query to find product by ID
        let product = query_as!(
            Product,
            r#"
            SELECT id, name, description, price, stock, category, is_active, created_at, updated_at
            FROM products
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))?;
        
        Ok(product)
    }
    
    /// List products with optional filtering
    pub async fn list(&self, filter: &ProductFilter) -> Result<Vec<Product>, ApiError> {
        // Start building the dynamic SQL query
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT id, name, description, price, stock, category, is_active, created_at, updated_at FROM products WHERE 1=1"
        );
        
        // Apply name filter (case-insensitive partial match)
        if let Some(name) = &filter.name {
            query_builder.push(" AND name ILIKE ");
            query_builder.push_bind(format!("%{}%", name));
        }
        
        // Apply category filter (exact match)
        if let Some(category) = &filter.category {
            query_builder.push(" AND category = ");
            query_builder.push_bind(category);
        }
        
        // Apply price range filters
        if let Some(min_price) = filter.min_price {
            query_builder.push(" AND price >= ");
            query_builder.push_bind(min_price);
        }
        
        if let Some(max_price) = filter.max_price {
            query_builder.push(" AND price <= ");
            query_builder.push_bind(max_price);
        }
        
        // Apply active status filter
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
        
        // Build and execute the query
        let products = query_builder
            .build_query_as::<Product>()
            .fetch_all(&self.pool)
            .await
            .map_err(ApiError::Database)?;
            
        Ok(products)
    }
    
    /// Update an existing product
    pub async fn update(&self, id: Uuid, update: &UpdateProduct) -> Result<Product, ApiError> {
        // Start transaction
        let mut tx = self.pool.begin().await.map_err(ApiError::Database)?;
        
        // Check if product exists
        let current = query_as!(
            Product,
            r#"
            SELECT id, name, description, price, stock, category, is_active, created_at, updated_at
            FROM products
            WHERE id = $1
            FOR UPDATE
            "#,
            id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))?;
        
        // Apply updates only to fields that are provided
        let name = update.name.as_ref().unwrap_or(&current.name);
        let description = update.description.as_ref().or(current.description.as_ref());
        let price = update.price.unwrap_or(current.price);
        let stock = update.stock.unwrap_or(current.stock);
        let category = update.category.as_ref().or(current.category.as_ref());
        let is_active = update.is_active.unwrap_or(current.is_active);
        
        // Execute update query
        let updated = query_as!(
            Product,
            r#"
            UPDATE products
            SET name = $1, description = $2, price = $3, stock = $4, category = $5, is_active = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING id, name, description, price, stock, category, is_active, created_at, updated_at
            "#,
            name,
            description,
            price,
            stock,
            category,
            is_active,
            id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ApiError::Database)?;
        
        // Commit transaction
        tx.commit().await.map_err(ApiError::Database)?;
        
        Ok(updated)
    }
    
    /// Delete a product by ID
    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        // Execute delete query
        let result = query!(
            r#"
            DELETE FROM products
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(ApiError::Database)?;
        
        // Check if any row was affected
        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("Product with ID {} not found", id)));
        }
        
        Ok(())
    }
}