use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Product model representing a product in the database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: Uuid,                    // Unique identifier
    pub name: String,                // Product name
    pub description: Option<String>, // Optional product description
    pub price: f64,                  // Product price
    pub stock: i32,                  // Available inventory
    pub category: Option<String>,    // Optional product category
    pub is_active: bool,             // Whether product is active
    pub created_at: DateTime<Utc>,   // Creation timestamp
    pub updated_at: DateTime<Utc>,   // Last update timestamp
}

/// DTO for creating a new product
#[derive(Debug, Deserialize)]
pub struct CreateProduct {
    pub name: String,                // Required product name
    pub description: Option<String>, // Optional product description
    pub price: f64,                  // Required product price
    pub stock: Option<i32>,          // Optional stock amount (defaults to 0)
    pub category: Option<String>,    // Optional product category
}

/// DTO for updating an existing product
#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,        // Optional product name update
    pub description: Option<String>, // Optional product description update
    pub price: Option<f64>,          // Optional product price update
    pub stock: Option<i32>,          // Optional stock amount update
    pub category: Option<String>,    // Optional product category update
    pub is_active: Option<bool>,     // Optional active status update
}

/// Filter criteria for querying products
#[derive(Debug, Deserialize)]
pub struct ProductFilter {
    pub name: Option<String>,     // Filter by name (partial match)
    pub category: Option<String>, // Filter by category (exact match)
    pub min_price: Option<f64>,   // Filter by minimum price
    pub max_price: Option<f64>,   // Filter by maximum price
    pub is_active: Option<bool>,  // Filter by active status
    pub limit: Option<i64>,       // Maximum number of results
    pub offset: Option<i64>,      // Number of results to skip
}
