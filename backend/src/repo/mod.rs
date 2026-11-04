pub mod collections;
pub mod entries;
pub mod search;
pub mod tags;

use crate::error::AppError;

/// Map a SQLx error, translating unique-constraint violations into 409s.
pub fn map_unique(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &err {
        if db.code().as_deref() == Some("23505") {
            return AppError::conflict("a resource with that value already exists");
        }
    }
    AppError::from(err)
}
