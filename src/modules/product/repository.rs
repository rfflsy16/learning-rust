use crate::{
    core::{db::DbPool, error::ApiError},
    modules::product::model::{CreateProduct, Product, ProductFilter, UpdateProduct},
    utils::offset_to_chrono,
};
use bigdecimal::BigDecimal;
use sqlx::{FromRow, QueryBuilder, Row, postgres::PgRow, query, query_as};
use std::str::FromStr;
use tracing::{info, warn};
use uuid::Uuid;

// Konstanta SQL untuk menghindari duplikasi
const SELECT_PRODUCT_FIELDS: &str =
    "id, name, description, price, stock, category, is_active, created_at, updated_at";
const SELECT_PRODUCT_BASE: &str = "SELECT id, name, description, price, stock, category, is_active, created_at, updated_at FROM products";

/// Repository untuk operasi database produk
pub struct ProductRepository {
    pool: DbPool,
}

// Implementasi FromRow untuk Product untuk menangani konversi tipe khusus
impl<'r> FromRow<'r, PgRow> for Product {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.try_get("id")?;
        let name: String = row.try_get("name")?;

        // Mendapatkan deskripsi dengan penanganan error tambahan
        let description: Option<String> = row.try_get("description").unwrap_or_else(|e| {
            warn!("Error mendapatkan field deskripsi: {}", e);
            None
        });

        // Konversi BigDecimal ke f64 lebih efisien
        let price_decimal: BigDecimal = row.try_get("price")?;
        let price = f64::from_str(&price_decimal.to_string()).unwrap_or_else(|e| {
            warn!("Error konversi harga: {}", e);
            0.0
        });

        // Mendapatkan field lainnya
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

impl ProductRepository {
    /// Membuat repository produk baru
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Membuat produk baru di database
    pub async fn create(&self, product: &CreateProduct) -> Result<Product, ApiError> {
        let query_str = format!(
            "INSERT INTO products (name, description, price, stock, category) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING {SELECT_PRODUCT_FIELDS}"
        );

        let result = query_as::<_, Product>(&query_str)
            .bind(&product.name)
            .bind(product.description.as_deref())
            .bind(product.price)
            .bind(product.stock.unwrap_or(0))
            .bind(product.category.as_deref())
            .fetch_one(&self.pool)
            .await
            .map_err(ApiError::Database)?;

        info!("Produk baru dibuat dengan ID: {}", result.id);
        Ok(result)
    }

    /// Mendapatkan produk berdasarkan ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Product, ApiError> {
        let query_str = format!("{SELECT_PRODUCT_BASE} WHERE id = $1");

        query_as::<_, Product>(&query_str)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("Produk dengan ID {} tidak ditemukan", id)))
    }

    /// Daftar produk dengan filter opsional
    pub async fn list(&self, filter: &ProductFilter) -> Result<Vec<Product>, ApiError> {
        // Mulai membangun query SQL dinamis
        let mut query_builder = QueryBuilder::new(SELECT_PRODUCT_BASE);
        query_builder.push(" WHERE 1=1");

        // Terapkan filter
        Self::apply_filters(&mut query_builder, filter);

        // Terapkan pengurutan, limit dan offset
        query_builder.push(" ORDER BY name ASC");

        if let Some(limit) = filter.limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }

        if let Some(offset) = filter.offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
        }

        // Eksekusi query dan konversi hasil langsung ke struct Product
        query_builder
            .build_query_as::<Product>()
            .fetch_all(&self.pool)
            .await
            .map_err(ApiError::Database)
    }

    /// Memperbarui produk yang ada
    pub async fn update(&self, id: Uuid, update: &UpdateProduct) -> Result<Product, ApiError> {
        let mut tx = self.pool.begin().await.map_err(ApiError::Database)?;

        // Periksa apakah produk ada dan dapatkan nilai saat ini
        let current = Self::get_current_product(&mut tx, id).await?;

        // Siapkan nilai update menggunakan pendekatan yang lebih ringkas
        let updated = Self::apply_updates(&mut tx, id, &current, update).await?;

        tx.commit().await.map_err(ApiError::Database)?;
        info!("Produk dengan ID: {} berhasil diperbarui", id);

        Ok(updated)
    }

    /// Menghapus produk berdasarkan ID
    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let result = query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(ApiError::Database)?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!(
                "Produk dengan ID {} tidak ditemukan",
                id
            )));
        }

        info!("Produk dengan ID: {} berhasil dihapus", id);
        Ok(())
    }

    // HELPER METHODS

    /// Mendapatkan produk saat ini dari database dengan kunci FOR UPDATE
    async fn get_current_product(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
    ) -> Result<Product, ApiError> {
        let query_str = format!("{SELECT_PRODUCT_BASE} WHERE id = $1 FOR UPDATE");

        query_as::<_, Product>(&query_str)
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(ApiError::Database)?
            .ok_or_else(|| ApiError::NotFound(format!("Produk dengan ID {} tidak ditemukan", id)))
    }

    /// Menerapkan pembaruan ke produk
    async fn apply_updates(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
        current: &Product,
        update: &UpdateProduct,
    ) -> Result<Product, ApiError> {
        // Siapkan nilai update
        let name = update.name.as_ref().unwrap_or(&current.name);
        let description_str = update
            .description
            .as_deref()
            .or(current.description.as_deref());
        let price = update.price.unwrap_or(current.price);
        let stock = update.stock.unwrap_or(current.stock);
        let category_str = update.category.as_deref().or(current.category.as_deref());
        let is_active = update.is_active.unwrap_or(current.is_active);

        // Eksekusi update
        let query_str = format!(
            "UPDATE products
            SET name = $1, description = $2, price = $3, stock = $4, category = $5, is_active = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING {SELECT_PRODUCT_FIELDS}"
        );

        query_as::<_, Product>(&query_str)
            .bind(name)
            .bind(description_str)
            .bind(price)
            .bind(stock)
            .bind(category_str)
            .bind(is_active)
            .bind(id)
            .fetch_one(&mut **tx)
            .await
            .map_err(ApiError::Database)
    }

    /// Menerapkan filter ke query builder
    fn apply_filters<'a>(
        query_builder: &mut QueryBuilder<'a, sqlx::Postgres>,
        filter: &'a ProductFilter,
    ) {
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
    }
}
