//! Application-level error type that maps to HTTP responses.

use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// API error type. Each variant maps to an HTTP status code and JSON body.
#[derive(Debug)]
pub enum AppError {
    /// 404 Not Found
    NotFound,
    /// 400 Bad Request
    BadRequest(String),
    /// 409 Conflict
    Conflict(String),
    /// 500 Internal Server Error
    Internal(String),
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::BadRequest(rejection.body_text())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
            Self::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            }
        };

        let body = json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}

impl From<pt_satellite::SatelliteError> for AppError {
    fn from(e: pt_satellite::SatelliteError) -> Self {
        match e {
            pt_satellite::SatelliteError::AddressNotFound(msg) => {
                Self::BadRequest(format!("address not found: {msg}"))
            }
            pt_satellite::SatelliteError::NoParcelData { lat, lng } => {
                Self::BadRequest(format!("no parcel data for ({lat}, {lng})"))
            }
            pt_satellite::SatelliteError::CanopyUnavailable => {
                Self::Internal("canopy data unavailable".to_string())
            }
        }
    }
}

impl From<pt_scan::ScanError> for AppError {
    fn from(e: pt_scan::ScanError) -> Self {
        match e {
            pt_scan::ScanError::InvalidPly(msg) => {
                Self::BadRequest(format!("invalid PLY file: {msg}"))
            }
            pt_scan::ScanError::InsufficientPoints { found, needed } => Self::BadRequest(format!(
                "insufficient points: found {found}, need at least {needed}"
            )),
            other => Self::Internal(format!("scan processing error: {other}")),
        }
    }
}

impl From<pt_proposal::ProposalError> for AppError {
    fn from(e: pt_proposal::ProposalError) -> Self {
        match e {
            pt_proposal::ProposalError::InvalidInput(msg) => Self::BadRequest(msg),
            pt_proposal::ProposalError::Generation(msg) => {
                Self::Internal(format!("proposal generation failed: {msg}"))
            }
        }
    }
}

impl From<crate::s3::S3Error> for AppError {
    fn from(e: crate::s3::S3Error) -> Self {
        Self::Internal(format!("{e}"))
    }
}

impl From<pt_repo::RepoError> for AppError {
    fn from(e: pt_repo::RepoError) -> Self {
        match e {
            pt_repo::RepoError::NotFound => Self::NotFound,
            pt_repo::RepoError::Conflict(msg) => Self::Conflict(msg),
            pt_repo::RepoError::Database(db_err) => {
                Self::Internal(format!("database error: {db_err}"))
            }
            pt_repo::RepoError::Conversion(msg) => Self::Internal(format!("conversion: {msg}")),
        }
    }
}
