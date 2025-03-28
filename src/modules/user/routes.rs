use crate::core::db::DbPool;
use crate::modules::user::service::UserHandler;
use crate::modules::user::model::{CreateUser, LoginUser, UserFilter, UpdateUser};
use crate::modules::user::repository::UserRepository;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

/// Type alias for the shared state
type SharedHandler = Arc<UserHandler>;

/// Create user routes
pub fn user_routes(pool: DbPool) -> Router {
    // Create repository and handler
    let repository = UserRepository::new(pool);
    let handler = Arc::new(UserHandler::new(repository));
    
    // Define routes with shared state
    Router::new()
        .route("/api/users", 
            get(list_users)
            .post(register_user)
        )
        .route("/api/users/{id}",
            get(get_user)
            .put(update_user)
            .delete(delete_user)
        )
        .route("/api/auth/login",
            post(login_user)
        )
        .with_state(handler)
}

/// Handler functions that take the shared state
async fn list_users(
    State(handler): State<SharedHandler>,
    Query(filter): Query<UserFilter>,
) -> impl IntoResponse {
    handler.list_users(filter).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn register_user(
    State(handler): State<SharedHandler>,
    Json(user): Json<CreateUser>,
) -> impl IntoResponse {
    handler.register(user).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn login_user(
    State(handler): State<SharedHandler>,
    Json(login): Json<LoginUser>,
) -> impl IntoResponse {
    handler.login(login).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn get_user(
    State(handler): State<SharedHandler>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    handler.get_user(id).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn update_user(
    State(handler): State<SharedHandler>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateUser>,
) -> impl IntoResponse {
    handler.update_user(id, update).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}

async fn delete_user(
    State(handler): State<SharedHandler>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    handler.delete_user(id).await
        .map_or_else(|err| err.into_response(), |response| response.into_response())
}