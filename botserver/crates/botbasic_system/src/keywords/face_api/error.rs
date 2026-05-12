//! Face API Error Types
//!
//! This module contains error types for Face API operations.

#[derive(Debug, Clone)]
pub enum FaceApiError {
    ConfigError(String),
    NetworkError(String),
    ApiError(String),
    ParseError(String),
    InvalidInput(String),
    NoFaceFound,
    NotImplemented(String),
    RateLimited,
    Unauthorized,
}

impl std::fmt::Display for FaceApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ApiError(msg) => write!(f, "API error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::NoFaceFound => write!(f, "No face found in image"),
            Self::NotImplemented(provider) => write!(f, "{} provider not implemented", provider),
            Self::RateLimited => write!(f, "Rate limit exceeded"),
            Self::Unauthorized => write!(f, "Unauthorized - check API credentials"),
        }
    }
}

impl std::error::Error for FaceApiError {}
