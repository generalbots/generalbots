use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, Clone)]
pub enum CanvasError {
    DatabaseConnection,
    NotFound,
    ElementNotFound,
    ElementLocked,
    CreateFailed,
    UpdateFailed,
    DeleteFailed,
    ExportFailed(String),
    InvalidInput(String),
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseConnection => write!(f, "Database connection failed"),
            Self::NotFound => write!(f, "Canvas not found"),
            Self::ElementNotFound => write!(f, "Element not found"),
            Self::ElementLocked => write!(f, "Element is locked"),
            Self::CreateFailed => write!(f, "Failed to create"),
            Self::UpdateFailed => write!(f, "Failed to update"),
            Self::DeleteFailed => write!(f, "Failed to delete"),
            Self::ExportFailed(msg) => write!(f, "Export failed: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl std::error::Error for CanvasError {}

impl IntoResponse for CanvasError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::NotFound | Self::ElementNotFound => StatusCode::NOT_FOUND,
            Self::ElementLocked => StatusCode::FORBIDDEN,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
