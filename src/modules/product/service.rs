use crate::core::error::ApiError;
use crate::modules::product::model::{CreateProduct, ProductFilter, UpdateProduct};
use crate::modules::product::repository::ProductRepository;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;

/// Product HTTP request handlers
pub struct ProductService {
    repository: ProductRepository,   // Product repository for database operations
}

impl ProductService {
    /// Create a new product handler
    pub fn new(repository: ProductRepository) -> Self {
        Self { repository }
    }
    
    /// Handler to create a new product
    pub async fn create_product(
        &self,
        product: CreateProduct,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Validate price
        self.validate_price(product.price)?;
        
        // Create product in database and return with 201 status
        let created_product = self.repository.create(&product).await?;
        Ok((StatusCode::CREATED, Json(created_product)))
    }
    
    /// Handler to get a product by ID
    pub async fn get_product(
        &self,
        id: Uuid,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        let product = self.repository.find_by_id(id).await?;
        Ok(Json(product))
    }
    
    /// Handler to list products with optional filtering
    pub async fn list_products(
        &self,
        filter: ProductFilter,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        let products = self.repository.list(&filter).await?;
        Ok(Json(products))
    }
    
    /// Handler to update a product
    pub async fn update_product(
        &self,
        id: Uuid,
        update: UpdateProduct,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Validate price if provided
        if let Some(price) = update.price {
            self.validate_price(price)?;
        }
        
        let updated_product = self.repository.update(id, &update).await?;
        Ok(Json(updated_product))
    }
    
    /// Handler to delete a product
    pub async fn delete_product(
        &self,
        id: Uuid,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        self.repository.delete(id).await?;
        
        Ok((
            StatusCode::OK, 
            Json(json!({
                "success": true,
                "message": format!("Product with ID {} successfully deleted", id)
            }))
        ))
    }
    
    // Helper method to validate price
    fn validate_price(&self, price: f64) -> Result<(), ApiError> {
        if price < 0.0 {
            return Err(ApiError::BadRequest("Price cannot be negative".to_string()));
        }
        Ok(())
    }
}