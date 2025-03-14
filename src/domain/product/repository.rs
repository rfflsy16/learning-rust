use crate::core::db::DbPool;
use crate::core::error::ApiError;
use crate::domain::product::model::{CreateProduct, Product, ProductFilter, UpdateProduct};
use crate::utils::offset_to_chrono;
use sqlx::{query,  FromRow, postgres::PgRow, Row}; // Added Execute trait here
use uuid::Uuid;
use bigdecimal::BigDecimal;

/// Repository for product database operations
pub struct ProductRepository {
    pool: DbPool,    // Database connection pool
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
        
        // Convert BigDecimal to f64
        let price_decimal: BigDecimal = row.try_get("price")?;
        let price = price_decimal.to_string().parse::<f64>().unwrap_or(0.0);
        
        let stock: i32 = row.try_get("stock")?;
        let category: Option<String> = row.try_get("category")?;
        let is_active: bool = row.try_get("is_active")?;
        
        // Convert OffsetDateTime to DateTime<Utc>
        let created_at = offset_to_chrono(row.try_get("created_at")?);
        let updated_at = offset_to_chrono(row.try_get("updated_at")?);
        
        Ok(Product {
            id,
            name,
            description,
            price,
            stock,
            category,
            is_active,
            created_at,
            updated_at,
        })
    }
}

impl ProductRepository {
    /// Create a new product repository
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new product in the database
    pub async fn create(&self, product: &CreateProduct) -> Result<Product, ApiError> {
        // SQL query to insert new product
        let row = query(
            r#"
            INSERT INTO products (name, description, price, stock, category)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, price, stock, category, is_active, created_at, updated_at
            "#,
        )
        .bind(&product.name)
        .bind(product.description.as_deref())  // Convert Option<String> to Option<&str>
        .bind(product.price)
        .bind(product.stock.unwrap_or(0))
        .bind(product.category.as_deref())  // Convert Option<String> to Option<&str>
        .fetch_one(&self.pool)
        .await
        .map_err(ApiError::Database)?;
        
        // Convert row to Product using FromRow implementation
        let result = Product::from_row(&row).map_err(ApiError::Database)?;
        
        Ok(result)
    }
    
    /// Get a product by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Product, ApiError> {
        // SQL query to find product by ID
        let row = query(
            r#"
            SELECT id, name, description, price, stock, category, is_active, created_at, updated_at
            FROM products
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))?;
        
        // Convert row to Product using FromRow implementation
        let product = Product::from_row(&row).map_err(ApiError::Database)?;
        
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
        
        // Build and execute the query directly
        let rows = query_builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(ApiError::Database)?;
            
        // Convert rows to Products
        let mut products = Vec::with_capacity(rows.len());
        for row in rows {
            let product = Product::from_row(&row).map_err(ApiError::Database)?;
            products.push(product);
        }
            
        Ok(products)
    }
    
    /// Update an existing product
    pub async fn update(&self, id: Uuid, update: &UpdateProduct) -> Result<Product, ApiError> {
        // Start transaction
        let mut tx = self.pool.begin().await.map_err(ApiError::Database)?;
        
        // Check if product exists
        let row = query(
            r#"
            SELECT id, name, description, price, stock, category, is_active, created_at, updated_at
            FROM products
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ApiError::Database)?
        .ok_or_else(|| ApiError::NotFound(format!("Product with ID {} not found", id)))?;
        
        // Convert row to Product using FromRow implementation
        let current = Product::from_row(&row).map_err(ApiError::Database)?;
        
        // Apply updates only to fields that are provided
        let name = update.name.as_ref().unwrap_or(&current.name);
        
        // Handle description properly - convert to Option<&str>
        let description_str = match (&update.description, &current.description) {
            (Some(desc), _) => Some(desc.as_str()),
            (None, Some(desc)) => Some(desc.as_str()),
            (None, None) => None,
        };
        
        let price = update.price.unwrap_or(current.price);
        let stock = update.stock.unwrap_or(current.stock);
        
        // Handle category properly - convert to Option<&str>
        let category_str = match (&update.category, &current.category) {
            (Some(cat), _) => Some(cat.as_str()),
            (None, Some(cat)) => Some(cat.as_str()),
            (None, None) => None,
        };
        
        let is_active = update.is_active.unwrap_or(current.is_active);
        
        // Execute update query
        let row = query(
            r#"
            UPDATE products
            SET name = $1, description = $2, price = $3, stock = $4, category = $5, is_active = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING id, name, description, price, stock, category, is_active, created_at, updated_at
            "#,
        )
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
        
        // Convert row to Product using FromRow implementation
        let updated = Product::from_row(&row).map_err(ApiError::Database)?;
        
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