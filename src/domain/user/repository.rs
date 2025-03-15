// use crate::core::db::DbPool;
// // use crate::core::error::ApiError;
// use crate::domain::user::model::{CreateUser, User, UserFilter, UpdateUser};
// // use crate::utils::offset_to_chrono;
// use sqlx::{postgres::{self, PgRow}, query, FromRow, ::PgRow, Row};
// use uuid::Uuid;

// pub struct UserRepository{
//     pool: DbPool
// }

// // Implement FromRow for User to handle custon type conversions

// impl<'r> FromRow<'r, PgRow> for User {
//     fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {

//     }
// }