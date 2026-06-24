use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, Clone)]
pub enum WebinarError {
    DatabaseConnection,
    NotFound,
    NotAuthorized,
    CreateFailed,
    UpdateFailed,
    JoinFailed,
    InvalidState(String),
    InvalidInput(String),
    FeatureDisabled(String),
    RegistrationNotRequired,
    RegistrationFailed,
    AlreadyRegistered,
    MaxParticipantsReached,
}

impl std::fmt::Display for WebinarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseConnection => write!(f, "Database connection failed"),
            Self::NotFound => write!(f, "Webinar not found"),
            Self::NotAuthorized => write!(f, "Not authorized"),
            Self::CreateFailed => write!(f, "Failed to create"),
            Self::UpdateFailed => write!(f, "Failed to update"),
            Self::JoinFailed => write!(f, "Failed to join"),
            Self::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Self::FeatureDisabled(msg) => write!(f, "Feature disabled: {msg}"),
            Self::RegistrationNotRequired => write!(f, "Registration not required"),
            Self::RegistrationFailed => write!(f, "Registration failed"),
            Self::AlreadyRegistered => write!(f, "Already registered"),
            Self::MaxParticipantsReached => write!(f, "Maximum participants reached"),
        }
    }
}

impl std::error::Error for WebinarError {}

impl IntoResponse for WebinarError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::NotAuthorized => StatusCode::FORBIDDEN,
            Self::AlreadyRegistered => StatusCode::CONFLICT,
            Self::InvalidInput(_) | Self::InvalidState(_) => StatusCode::BAD_REQUEST,
            Self::MaxParticipantsReached => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
