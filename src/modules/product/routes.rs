use crate::core::db::DbPool;
use crate::modules::product::service::ProductHandler;
use crate::modules::product::model::{CreateProduct, ProductFilter, UpdateProduct};
use crate::modules::product::repository::ProductRepository;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing:: get,
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

/// Type alias for the shared state
type SharedHandler = Arc<ProductHandler>;

/// Create product routes
pub fn product_routes(pool: DbPool) -> Router {
    // Create repository and handler
    let repository = ProductRepository::new(pool);
    let handler = Arc::new(ProductHandler::new(repository));
    
    // Define routes with shared state
    Router::new()
        .route("/api/products", 
            get(_products)
            .post(create_product)
        )
        .route("/api/products/{id}",
            get(get_product)
            .put(update_product)
            .delete(delete_product)
        )
        .with_state(handler)
}

/// Handler functions that take the shared state
async fn _products(
    State(handler): State<SharedHandler>,
    Query(filter): Query<ProductFilter>,
) -> impl IntoResponse {
    handler.list_products(filter).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn create_product(
    State(handler): State<SharedHandler>,
    Json(product): Json<CreateProduct>,
) -> impl IntoResponse {
    handler.create_product(product).await
        .map_or_else(|err | err.into_response(), |response| response.into_response())
}

async fn get_product(
    State(handler): State<SharedHandler>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    handler.get_product(id).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn update_product(
    State(handler): State<SharedHandler>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateProduct>,
) -> impl IntoResponse {
    handler.update_product(id, update).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn delete_product(
    State(handler): State<SharedHandler>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    handler.delete_product(id).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}