use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// A single field-level validation failure.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<FieldError>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{resource} not found")]
    NotFound { resource: &'static str },

    #[error("validation failed")]
    Validation(Vec<FieldError>),

    #[error("{0}")]
    Conflict(String),

    #[error("{message}")]
    Unprocessable { code: &'static str, message: String },

    #[error("{0}")]
    PreconditionRequired(String),

    #[error("{0}")]
    PayloadTooLarge(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn not_found(resource: &'static str) -> Self {
        AppError::NotFound { resource }
    }

    pub fn conflict(reason: impl Into<String>) -> Self {
        AppError::Conflict(reason.into())
    }

    pub fn unprocessable(message: impl Into<String>) -> Self {
        AppError::Unprocessable {
            code: "UNPROCESSABLE_ENTITY",
            message: message.into(),
        }
    }

    pub fn invalid_cursor() -> Self {
        AppError::Unprocessable {
            code: "INVALID_CURSOR",
            message: "invalid or expired pagination cursor".to_string(),
        }
    }

    pub fn unsupported_format(value: &str) -> Self {
        AppError::Unprocessable {
            code: "UNSUPPORTED_FORMAT",
            message: format!("unsupported format `{value}`"),
        }
    }

    pub fn precondition_required(message: impl Into<String>) -> Self {
        AppError::PreconditionRequired(message.into())
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        AppError::PayloadTooLarge(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details): (StatusCode, &'static str, String, Vec<FieldError>) =
            match self {
                AppError::NotFound { resource } => (
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    format!("{resource} not found"),
                    Vec::new(),
                ),
                AppError::Validation(fields) => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "VALIDATION_ERROR",
                    "request validation failed".to_string(),
                    fields,
                ),
                AppError::Conflict(reason) => {
                    (StatusCode::CONFLICT, "CONFLICT", reason, Vec::new())
                }
                AppError::Unprocessable { code, message } => {
                    (StatusCode::UNPROCESSABLE_ENTITY, code, message, Vec::new())
                }
                AppError::PreconditionRequired(message) => (
                    StatusCode::PRECONDITION_REQUIRED,
                    "PRECONDITION_REQUIRED",
                    message,
                    Vec::new(),
                ),
                AppError::PayloadTooLarge(message) => (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "PAYLOAD_TOO_LARGE",
                    message,
                    Vec::new(),
                ),
                AppError::Internal(err) => {
                    tracing::error!(error = ?err, "internal server error");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        "internal server error".to_string(),
                        Vec::new(),
                    )
                }
            };

        let body = ErrorBody {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                details,
            },
        };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound {
                resource: "resource",
            },
            other => AppError::Internal(anyhow::Error::new(other)),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut fields = Vec::new();
        for (field, errs) in errors.field_errors() {
            for err in errs {
                fields.push(FieldError {
                    field: field.to_string(),
                    code: err.code.to_string(),
                    message: err
                        .message
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| err.code.to_string()),
                });
            }
        }
        AppError::Validation(fields)
    }
}

pub type AppResult<T> = Result<T, AppError>;
