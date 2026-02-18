use thiserror::Error;

/// Error type for Hub01 Shop API operations.
///
/// - `RequestFailed` — network/transport errors (wraps `reqwest::Error`)
/// - `Authentication` — HTTP 401
/// - `PermissionDenied` — HTTP 403
/// - `NotFound` — HTTP 404
/// - `Validation` — HTTP 422, carries optional field-level errors
/// - `Api` — any other non-2xx status code
#[derive(Debug, Error)]
pub enum HubApiError {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Not found: {message}")]
    NotFound { message: String },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        errors: Option<serde_json::Value>,
    },

    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, HubApiError>;
