use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{error, warn};

#[derive(Debug, Clone, Serialize)]
pub struct SafeErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
}

impl SafeErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            code: None,
            request_id: None,
            details: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    pub fn internal_error() -> Self {
        Self::new("internal_error", "An internal error occurred")
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("bad_request", message)
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new("not_found", format!("{} not found", resource.into()))
    }

    pub fn unauthorized() -> Self {
        Self::new("unauthorized", "Authentication required")
    }

    pub fn forbidden() -> Self {
        Self::new("forbidden", "You don't have permission to access this resource")
    }

    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        let mut response = Self::new("rate_limited", "Too many requests, please try again later");
        if let Some(secs) = retry_after {
            response = response.with_detail("retry_after_seconds", secs.to_string());
        }
        response
    }

    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new("validation_error", message).with_detail("field", field)
    }

    pub fn service_unavailable() -> Self {
        Self::new("service_unavailable", "Service temporarily unavailable")
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("conflict", message)
    }

    pub fn gone(message: impl Into<String>) -> Self {
        Self::new("gone", message)
    }

    pub fn payload_too_large(max_size: Option<u64>) -> Self {
        let mut response = Self::new("payload_too_large", "Request payload is too large");
        if let Some(size) = max_size {
            response = response.with_detail("max_size_bytes", size.to_string());
        }
        response
    }

    pub fn unsupported_media_type(supported: &[&str]) -> Self {
        Self::new(
            "unsupported_media_type",
            "The media type is not supported",
        )
        .with_detail("supported_types", supported.join(", "))
    }

    pub fn method_not_allowed(allowed: &[&str]) -> Self {
        Self::new("method_not_allowed", "HTTP method not allowed for this endpoint")
            .with_detail("allowed_methods", allowed.join(", "))
    }

    pub fn timeout() -> Self {
        Self::new("timeout", "The request timed out")
    }
}

impl IntoResponse for SafeErrorResponse {
    fn into_response(self) -> Response {
        let status = match self.error.as_str() {
            "bad_request" | "validation_error" => StatusCode::BAD_REQUEST,
            "unauthorized" => StatusCode::UNAUTHORIZED,
            "forbidden" => StatusCode::FORBIDDEN,
            "not_found" => StatusCode::NOT_FOUND,
            "method_not_allowed" => StatusCode::METHOD_NOT_ALLOWED,
            "conflict" => StatusCode::CONFLICT,
            "gone" => StatusCode::GONE,
            "payload_too_large" => StatusCode::PAYLOAD_TOO_LARGE,
            "unsupported_media_type" => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "rate_limited" => StatusCode::TOO_MANY_REQUESTS,
            "timeout" => StatusCode::GATEWAY_TIMEOUT,
            "service_unavailable" => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct ErrorSanitizer {
    hide_internal_errors: bool,
    log_internal_errors: bool,
    include_request_id: bool,
    sensitive_patterns: Vec<String>,
}

impl Default for ErrorSanitizer {
    fn default() -> Self {
        Self {
            hide_internal_errors: true,
            log_internal_errors: true,
            include_request_id: true,
            sensitive_patterns: vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "api_key".to_string(),
                "apikey".to_string(),
                "authorization".to_string(),
                "credential".to_string(),
                "private".to_string(),
                "key".to_string(),
                "database".to_string(),
                "connection".to_string(),
                "dsn".to_string(),
                "postgres".to_string(),
                "mysql".to_string(),
                "redis".to_string(),
                "mongodb".to_string(),
                "aws".to_string(),
                "azure".to_string(),
                "gcp".to_string(),
                "/home/".to_string(),
                "/root/".to_string(),
                "/etc/".to_string(),
                "/var/".to_string(),
                "c:\\".to_string(),
                "d:\\".to_string(),
            ],
        }
    }
}

impl ErrorSanitizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn production() -> Self {
        Self {
            hide_internal_errors: true,
            log_internal_errors: true,
            include_request_id: true,
            sensitive_patterns: Self::default().sensitive_patterns,
        }
    }

    pub fn development() -> Self {
        Self {
            hide_internal_errors: false,
            log_internal_errors: true,
            include_request_id: true,
            sensitive_patterns: Self::default().sensitive_patterns,
        }
    }

    pub fn with_hide_internal(mut self, hide: bool) -> Self {
        self.hide_internal_errors = hide;
        self
    }

    pub fn with_logging(mut self, log: bool) -> Self {
        self.log_internal_errors = log;
        self
    }

    pub fn with_request_id(mut self, include: bool) -> Self {
        self.include_request_id = include;
        self
    }

    pub fn add_sensitive_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.sensitive_patterns.push(pattern.into());
        self
    }

    pub fn sanitize_error<E: std::error::Error>(
        &self,
        error: &E,
        request_id: Option<&str>,
    ) -> SafeErrorResponse {
        let error_string = error.to_string();

        if self.log_internal_errors {
            error!(
                request_id = ?request_id,
                error = %error_string,
                "Internal error occurred"
            );
        }

        if self.hide_internal_errors || self.contains_sensitive(&error_string) {
            let mut response = SafeErrorResponse::internal_error();
            if self.include_request_id {
                if let Some(rid) = request_id {
                    response = response.with_request_id(rid);
                }
            }
            response
        } else {
            let sanitized = self.sanitize_message(&error_string);
            let mut response = SafeErrorResponse::new("error", sanitized);
            if self.include_request_id {
                if let Some(rid) = request_id {
                    response = response.with_request_id(rid);
                }
            }
            response
        }
    }

    pub fn sanitize_message(&self, message: &str) -> String {
        let mut result = message.to_string();

        for pattern in &self.sensitive_patterns {
            if result.to_lowercase().contains(&pattern.to_lowercase()) {
                result = redact_around_pattern(&result, pattern);
            }
        }

        result = redact_stack_traces(&result);
        result = redact_file_paths(&result);
        result = redact_ip_addresses(&result);
        result = redact_connection_strings(&result);

        result
    }

    pub fn contains_sensitive(&self, message: &str) -> bool {
        let lower = message.to_lowercase();
        for pattern in &self.sensitive_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                return true;
            }
        }

        if looks_like_stack_trace(message) {
            return true;
        }

        if looks_like_connection_string(message) {
            return true;
        }

        false
    }

    pub fn safe_response_for_status(
        &self,
        status: StatusCode,
        request_id: Option<&str>,
    ) -> SafeErrorResponse {
        let mut response = match status {
            StatusCode::BAD_REQUEST => SafeErrorResponse::bad_request("Invalid request"),
            StatusCode::UNAUTHORIZED => SafeErrorResponse::unauthorized(),
            StatusCode::FORBIDDEN => SafeErrorResponse::forbidden(),
            StatusCode::NOT_FOUND => SafeErrorResponse::not_found("Resource"),
            StatusCode::METHOD_NOT_ALLOWED => SafeErrorResponse::method_not_allowed(&[]),
            StatusCode::CONFLICT => SafeErrorResponse::conflict("Resource conflict"),
            StatusCode::GONE => SafeErrorResponse::gone("Resource no longer available"),
            StatusCode::PAYLOAD_TOO_LARGE => SafeErrorResponse::payload_too_large(None),
            StatusCode::UNSUPPORTED_MEDIA_TYPE => SafeErrorResponse::unsupported_media_type(&[]),
            StatusCode::TOO_MANY_REQUESTS => SafeErrorResponse::rate_limited(None),
            StatusCode::INTERNAL_SERVER_ERROR => SafeErrorResponse::internal_error(),
            StatusCode::SERVICE_UNAVAILABLE => SafeErrorResponse::service_unavailable(),
            StatusCode::GATEWAY_TIMEOUT => SafeErrorResponse::timeout(),
            _ => SafeErrorResponse::new(
                format!("error_{}", status.as_u16()),
                status.canonical_reason().unwrap_or("An error occurred"),
            ),
        };

        if self.include_request_id {
            if let Some(rid) = request_id {
                response = response.with_request_id(rid);
            }
        }

        response
    }
}

fn redact_around_pattern(text: &str, pattern: &str) -> String {
    let lower_text = text.to_lowercase();
    let lower_pattern = pattern.to_lowercase();

    if let Some(pos) = lower_text.find(&lower_pattern) {
        let start = pos;
        let mut end = pos + pattern.len();

        let chars: Vec<char> = text.chars().collect();
        while end < chars.len() && !chars[end].is_whitespace() && chars[end] != ',' && chars[end] != ';' {
            end += 1;
        }

        let before = &text[..start];
        let after = if end < text.len() { &text[end..] } else { "" };
        format!("{}[REDACTED]{}", before, after)
    } else {
        text.to_string()
    }
}

fn redact_stack_traces(text: &str) -> String {
    let patterns = [
        r"at .+:\d+:\d+",
        r"File .+, line \d+",
        r"\s+at .+\(.+\)",
        r"\.rs:\d+",
        r"\.go:\d+",
        r"\.py:\d+",
        r"\.java:\d+",
        r"\.js:\d+",
        r"\.ts:\d+",
    ];

    let mut result = text.to_string();
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, "[STACK_TRACE_REDACTED]").to_string();
        }
    }
    result
}

fn redact_file_paths(text: &str) -> String {
    let patterns = [
        r"/[a-zA-Z0-9_\-./]+\.(rs|go|py|java|js|ts|rb|php|c|cpp|h)",
        r"[A-Z]:\\[a-zA-Z0-9_\-\\]+\.\w+",
        r"/home/[a-zA-Z0-9_\-/]+",
        r"/root/[a-zA-Z0-9_\-/]+",
        r"/var/[a-zA-Z0-9_\-/]+",
        r"/etc/[a-zA-Z0-9_\-/]+",
    ];

    let mut result = text.to_string();
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, "[PATH_REDACTED]").to_string();
        }
    }
    result
}

fn redact_ip_addresses(text: &str) -> String {
    let ip_pattern = r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b";
    if let Ok(re) = regex::Regex::new(ip_pattern) {
        re.replace_all(text, "[IP_REDACTED]").to_string()
    } else {
        text.to_string()
    }
}

fn redact_connection_strings(text: &str) -> String {
    let patterns = [
        r"postgres://[^\s]+",
        r"postgresql://[^\s]+",
        r"mysql://[^\s]+",
        r"mongodb://[^\s]+",
        r"mongodb\+srv://[^\s]+",
        r"redis://[^\s]+",
        r"amqp://[^\s]+",
        r"jdbc:[^\s]+",
    ];

    let mut result = text.to_string();
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, "[CONNECTION_STRING_REDACTED]").to_string();
        }
    }
    result
}

fn looks_like_stack_trace(text: &str) -> bool {
    let indicators = [
        "at line",
        "stack trace",
        "backtrace",
        "panic:",
        "Traceback",
        "Exception in thread",
        "    at ",
        "Caused by:",
    ];

    let lower = text.to_lowercase();
    for indicator in &indicators {
        if lower.contains(&indicator.to_lowercase()) {
            return true;
        }
    }

    false
}

fn looks_like_connection_string(text: &str) -> bool {
    let indicators = [
        "://",
        "host=",
        "dbname=",
        "user=",
        "password=",
        "port=",
        "sslmode=",
    ];

    let lower = text.to_lowercase();
    let count = indicators.iter().filter(|i| lower.contains(*i)).count();
    count >= 2
}

pub fn sanitize_for_log(message: &str) -> String {
    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_message(message)
}

pub fn safe_error<E: std::error::Error>(error: E) -> SafeErrorResponse {
    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_error(&error, None)
}

pub fn safe_error_with_request_id<E: std::error::Error>(
    error: E,
    request_id: &str,
) -> SafeErrorResponse {
    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_error(&error, Some(request_id))
}

pub fn log_and_sanitize<E: std::error::Error>(
    error: &E,
    context: &str,
    request_id: Option<&str>,
) -> SafeErrorResponse {
    warn!(
        context = %context,
        request_id = ?request_id,
        error = %error,
        "Error occurred"
    );

    let sanitizer = ErrorSanitizer::production();
    sanitizer.sanitize_error(error, request_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    #[test]
    fn test_safe_error_response_new() {
        let response = SafeErrorResponse::new("test_error", "Test message");
        assert_eq!(response.error, "test_error");
        assert_eq!(response.message, "Test message");
        assert!(response.code.is_none());
        assert!(response.request_id.is_none());
    }

    #[test]
    fn test_safe_error_response_builder() {
        let response = SafeErrorResponse::new("error", "message")
            .with_code("E001")
            .with_request_id("req-123")
            .with_detail("field", "value");

        assert_eq!(response.code, Some("E001".to_string()));
        assert_eq!(response.request_id, Some("req-123".to_string()));
        assert_eq!(
            response.details.as_ref().and_then(|d| d.get("field")),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_factory_methods() {
        let internal = SafeErrorResponse::internal_error();
        assert_eq!(internal.error, "internal_error");

        let not_found = SafeErrorResponse::not_found("User");
        assert_eq!(not_found.error, "not_found");
        assert!(not_found.message.contains("User"));

        let rate_limited = SafeErrorResponse::rate_limited(Some(30));
        assert_eq!(rate_limited.error, "rate_limited");
        assert!(rate_limited.details.is_some());
    }

    #[test]
    fn test_error_sanitizer_hides_sensitive() {
        let sanitizer = ErrorSanitizer::production();
        let error = TestError("Connection failed: password=secret123".to_string());
        let response = sanitizer.sanitize_error(&error, Some("req-123"));

        assert_eq!(response.error, "internal_error");
        assert!(!response.message.contains("secret123"));
    }

    #[test]
    fn test_error_sanitizer_development() {
        let sanitizer = ErrorSanitizer::development();
        let error = TestError("Simple error message".to_string());
        let response = sanitizer.sanitize_error(&error, None);

        assert_eq!(response.message, "Simple error message");
    }

    #[test]
    fn test_contains_sensitive() {
        let sanitizer = ErrorSanitizer::default();

        assert!(sanitizer.contains_sensitive("Failed with password=abc"));
        assert!(sanitizer.contains_sensitive("API_KEY is invalid"));
        assert!(sanitizer.contains_sensitive("at /home/user/app.rs:42"));
        assert!(!sanitizer.contains_sensitive("Simple error"));
    }

    #[test]
    fn test_sanitize_message() {
        let sanitizer = ErrorSanitizer::default();

        let result = sanitizer.sanitize_message("Error at /home/user/file.rs:42");
        assert!(!result.contains("/home/user"));

        let result = sanitizer.sanitize_message("postgres://user:pass@host/db");
        assert!(!result.contains("user:pass"));
    }

    #[test]
    fn test_redact_ip_addresses() {
        let result = redact_ip_addresses("Connection from 192.168.1.100 failed");
        assert!(!result.contains("192.168.1.100"));
        assert!(result.contains("[IP_REDACTED]"));
    }

    #[test]
    fn test_redact_connection_strings() {
        let result = redact_connection_strings("Using postgres://admin:secret@localhost/mydb");
        assert!(!result.contains("admin:secret"));
        assert!(result.contains("[CONNECTION_STRING_REDACTED]"));
    }

    #[test]
    fn test_looks_like_stack_trace() {
        assert!(looks_like_stack_trace("panic: something went wrong"));
        assert!(looks_like_stack_trace("Traceback (most recent call last):"));
        assert!(looks_like_stack_trace("    at com.example.Main.run"));
        assert!(!looks_like_stack_trace("Simple error message"));
    }

    #[test]
    fn test_looks_like_connection_string() {
        assert!(looks_like_connection_string("host=localhost dbname=test user=admin"));
        assert!(looks_like_connection_string("postgres://localhost/db"));
        assert!(!looks_like_connection_string("Simple message"));
    }

    #[test]
    fn test_safe_response_for_status() {
        let sanitizer = ErrorSanitizer::production();

        let response = sanitizer.safe_response_for_status(StatusCode::NOT_FOUND, Some("req-123"));
        assert_eq!(response.error, "not_found");
        assert_eq!(response.request_id, Some("req-123".to_string()));

        let response = sanitizer.safe_response_for_status(StatusCode::INTERNAL_SERVER_ERROR, None);
        assert_eq!(response.error, "internal_error");
    }

    #[test]
    fn test_sanitize_for_log() {
        let result = sanitize_for_log("password=secret123 at /home/user/app.rs");
        assert!(!result.contains("secret123"));
        assert!(!result.contains("/home/user"));
    }

    #[test]
    fn test_config_builder() {
        let sanitizer = ErrorSanitizer::new()
            .with_hide_internal(false)
            .with_logging(false)
            .with_request_id(false)
            .add_sensitive_pattern("custom_secret");

        assert!(!sanitizer.hide_internal_errors);
        assert!(!sanitizer.log_internal_errors);
        assert!(!sanitizer.include_request_id);
        assert!(sanitizer.sensitive_patterns.contains(&"custom_secret".to_string()));
    }
}
