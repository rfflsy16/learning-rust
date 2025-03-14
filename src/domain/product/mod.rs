pub mod model;
pub mod repository;
pub mod handler;
pub mod routes;

// Re-export important types to make them available through the product module
pub use model::{Product, CreateProduct, UpdateProduct, ProductFilter};
pub use repository::ProductRepository;