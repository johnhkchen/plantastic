//! Repository error types.

use std::fmt;

/// Errors that can occur in the repository layer.
#[derive(Debug)]
pub enum RepoError {
    /// Requested entity was not found.
    NotFound,
    /// A uniqueness constraint was violated.
    Conflict(String),
    /// Underlying database error.
    Database(sqlx::Error),
    /// Failed to convert between domain types and database rows.
    Conversion(String),
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::Database(e) => write!(f, "database error: {e}"),
            Self::Conversion(msg) => write!(f, "conversion error: {msg}"),
        }
    }
}

impl std::error::Error for RepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for RepoError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(db_err) => {
                // PostgreSQL unique_violation is code 23505
                if db_err.code().as_deref() == Some("23505") {
                    Self::Conflict(db_err.message().to_string())
                } else {
                    Self::Database(e)
                }
            }
            _ => Self::Database(e),
        }
    }
}
