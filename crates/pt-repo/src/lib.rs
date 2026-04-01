//! Repository layer for Plantastic.
//!
//! Provides typed Rust functions for CRUD operations against PostgreSQL + PostGIS,
//! mapping between domain types (pt-project, pt-materials) and database rows via sqlx.

// All public functions return Result<_, RepoError> whose variants are self-documenting.
#![allow(clippy::missing_errors_doc)]

pub mod convert;
pub mod error;
pub mod material;
pub mod pool;
pub mod project;
pub mod tenant;
pub mod tier_assignment;
pub mod zone;

pub use error::RepoError;
pub use pool::{create_pool, create_pool_with_config, PoolConfig};
