//! Application error types and error handling
//!
//! This module provides comprehensive error handling for the Signal AI backend,
//! including proper HTTP status codes and user-friendly error messages.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application result type alias
pub type AppResult<T> = Result<T, AppError>;

/// Main application error type
///
/// All application errors should use this type for consistent
/// error handling and logging.
#[derive(Error, Debug)]
pub enum AppError {
    /// Database operation errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// LLM service errors
    #[error("LLM service error: {0}")]
    Llm(#[from] anyhow::Error),

    /// Signal messaging errors
    #[error("Signal messaging error: {message}")]
    Signal { message: String },

    /// Input validation errors
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    /// HTTP request errors
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal server errors
    #[error("Internal server error: {message}")]
    Internal { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Resource not found errors
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
}

impl AppError {
    /// Create a new validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a new signal error
    pub fn signal(message: impl Into<String>) -> Self {
        Self::Signal {
            message: message.into(),
        }
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a new config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Llm(_) => StatusCode::BAD_GATEWAY,
            Self::Signal { .. } => StatusCode::BAD_GATEWAY,
            Self::Validation { .. } => StatusCode::BAD_REQUEST,
            Self::Http(_) => StatusCode::BAD_GATEWAY,
            Self::Serialization(_) => StatusCode::BAD_REQUEST,
            Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Config { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
        }
    }

    /// Check if this error should be logged as an error vs warning
    pub fn should_log_error(&self) -> bool {
        matches!(
            self,
            Self::Database(_) | Self::Internal { .. } | Self::Config { .. } | Self::Io(_)
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // Log the error
        if self.should_log_error() {
            tracing::error!("Application error: {}", self);
        } else {
            tracing::warn!("Application warning: {}", self);
        }

        // Create error response
        let error_response = json!({
            "error": {
                "message": self.to_string(),
                "type": error_type_name(&self),
                "status": status.as_u16()
            }
        });

        (status, Json(error_response)).into_response()
    }
}

/// Get a string representation of the error type
fn error_type_name(error: &AppError) -> &'static str {
    match error {
        AppError::Database(_) => "database_error",
        AppError::Llm(_) => "llm_service_error",
        AppError::Signal { .. } => "signal_error",
        AppError::Validation { .. } => "validation_error",
        AppError::Http(_) => "http_error",
        AppError::Serialization(_) => "serialization_error",
        AppError::Io(_) => "io_error",
        AppError::Internal { .. } => "internal_error",
        AppError::Config { .. } => "configuration_error",
        AppError::NotFound { .. } => "not_found",
    }
} 