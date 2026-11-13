use std::fmt;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
pub type DbVerbsResult<T> = Result<T, DbVerbsError>;

#[derive(Debug)]
pub enum DbVerbsError {
    UnsupportedObject(String),
}

impl fmt::Display for DbVerbsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbVerbsError::UnsupportedObject(v) => write!(f, "expected object, got {v}"),
        }
    }
}

impl std::error::Error for DbVerbsError {}

impl IntoResponse for DbVerbsError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "expected object, got unsupported value",
        )
            .into_response()
    }
}
