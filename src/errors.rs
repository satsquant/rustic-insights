use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("Failed to process metrics: {0}")]
    MetricsProcessingError(String),

    #[error("Failed to register metric: {0}")]
    MetricRegistrationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal server error: {0}")]
    InternalError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ServerError::MetricsProcessingError(_) => StatusCode::BAD_REQUEST,
            ServerError::MetricRegistrationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::ConfigurationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            status: self.status_code().to_string(),
            message: self.to_string(),
        };

        HttpResponse::build(self.status_code()).json(error_response)
    }
}
