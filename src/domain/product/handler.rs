use crate::core::error::ApiError;
use crate::domain::product::model::{CreateProduct, ProductFilter, UpdateProduct};
use crate::domain::product::repository::ProductRepository;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

/// Product HTTP request handlers
pub struct ProductHandler {
    repository: ProductRepository,   // Product repository for database operations
}

impl ProductHandler {
    /// Create a new product handler
    pub fn new(repository: ProductRepository) -> Self {
        Self { repository }
    }
    
    /// Handler to create a new product
    pub async fn create_product(
        &self,
        product: CreateProduct,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Validate request data
        if product.price < 0.0 {
            return Err(ApiError::BadRequest("Price cannot be negative".to_string()));
        }
        
        // Create product in database
        let created_product = self.repository.create(&product).await?;
        
        // Return created product with 201 status
        Ok((StatusCode::CREATED, Json(created_product)))
    }
    
    /// Handler to get a product by ID
    pub async fn get_product(
        &self,
        id: Uuid,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Get product from database
        let product = self.repository.find_by_id(id).await?;
        
        // Return product
        Ok(Json(product))
    }
    
    /// Handler to list products with optional filtering
    pub async fn list_products(
        &self,
        filter: ProductFilter,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Get filtered products from database
        let products = self.repository.list(&filter).await?;
        
        // Return product list
        Ok(Json(products))
    }
    
    /// Handler to update a product
    pub async fn update_product(
        &self,
        id: Uuid,
        update: UpdateProduct,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Validate request data
        if let Some(price) = update.price {
            if price < 0.0 {
                return Err(ApiError::BadRequest("Price cannot be negative".to_string()));
            }
        }
        
        // Update product in database
        let updated_product = self.repository.update(id, &update).await?;
        
        // Return updated product
        Ok(Json(updated_product))
    }
    
    /// Handler to delete a product
    pub async fn delete_product(
        &self,
        id: Uuid,
    ) -> Result<impl IntoResponse + '_, ApiError> {
        // Delete product from database
        self.repository.delete(id).await?;
        
        // Return 204 No Content on success
        Ok(StatusCode::NO_CONTENT)
    }
}