use axum::http::StatusCode;
use axum::response::IntoResponse;

#[derive(Debug, Clone)]
pub enum ContactsError {
    DatabaseConnection,
    NotFound,
    CreateFailed,
    UpdateFailed,
    DeleteFailed,
    ImportFailed(String),
    ExportFailed(String),
    InvalidInput(String),
}

impl std::fmt::Display for ContactsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseConnection => write!(f, "Database connection failed"),
            Self::NotFound => write!(f, "Contact not found"),
            Self::CreateFailed => write!(f, "Failed to create contact"),
            Self::UpdateFailed => write!(f, "Failed to update contact"),
            Self::DeleteFailed => write!(f, "Failed to delete contact"),
            Self::ImportFailed(msg) => write!(f, "Import failed: {msg}"),
            Self::ExportFailed(msg) => write!(f, "Export failed: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl std::error::Error for ContactsError {}

impl IntoResponse for ContactsError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
