use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl<T> ApiResponse<T> {
    #[must_use]
    pub const fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
            code: None,
        }
    }

    #[must_use]
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: Some(message.into()),
            code: None,
        }
    }

    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            message: None,
            code: None,
        }
    }

    #[must_use]
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            message: None,
            code: Some(code.into()),
        }
    }

    #[must_use]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ApiResponse<U> {
        ApiResponse {
            success: self.success,
            data: self.data.map(f),
            error: self.error,
            message: self.message,
            code: self.code,
        }
    }

    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        !self.success
    }
}

impl<T: Default> Default for ApiResponse<T> {
    fn default() -> Self {
        Self::success(T::default())
    }
}
