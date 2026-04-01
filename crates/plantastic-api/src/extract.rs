//! Custom Axum extractors.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

/// Tenant ID extracted from the `X-Tenant-Id` header.
///
/// V1 placeholder for auth — will be replaced by JWT claims later.
/// Returns 400 if the header is missing or not a valid UUID.
#[derive(Debug, Clone, Copy)]
pub struct TenantId(pub Uuid);

impl FromRequestParts<AppState> for TenantId {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("X-Tenant-Id")
            .ok_or_else(|| AppError::BadRequest("missing X-Tenant-Id header".to_string()))?;

        let value = header.to_str().map_err(|_| {
            AppError::BadRequest("X-Tenant-Id header is not valid UTF-8".to_string())
        })?;

        let id = Uuid::parse_str(value)
            .map_err(|_| AppError::BadRequest("X-Tenant-Id is not a valid UUID".to_string()))?;

        Ok(TenantId(id))
    }
}
