use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use futures_util::FutureExt;
use serde_json::json;
use std::panic::{catch_unwind, AssertUnwindSafe};
use tracing::{error, warn};

#[derive(Debug, Clone)]
pub struct PanicHandlerConfig {
    pub log_panics: bool,
    pub include_backtrace: bool,
    pub custom_message: Option<String>,
    pub notify_on_panic: bool,
}

impl Default for PanicHandlerConfig {
    fn default() -> Self {
        Self {
            log_panics: true,
            include_backtrace: cfg!(debug_assertions),
            custom_message: None,
            notify_on_panic: false,
        }
    }
}

impl PanicHandlerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn production() -> Self {
        Self {
            log_panics: true,
            include_backtrace: false,
            custom_message: Some("An unexpected error occurred. Please try again later.".to_string()),
            notify_on_panic: true,
        }
    }

    pub fn development() -> Self {
        Self {
            log_panics: true,
            include_backtrace: true,
            custom_message: None,
            notify_on_panic: false,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.custom_message = Some(message.into());
        self
    }

    pub fn with_backtrace(mut self, include: bool) -> Self {
        self.include_backtrace = include;
        self
    }

    pub fn with_logging(mut self, log: bool) -> Self {
        self.log_panics = log;
        self
    }

    pub fn with_notification(mut self, notify: bool) -> Self {
        self.notify_on_panic = notify;
        self
    }
}

pub async fn panic_handler_middleware(request: Request<Body>, next: Next) -> Response {
    panic_handler_middleware_with_config(request, next, &PanicHandlerConfig::default()).await
}

pub async fn panic_handler_middleware_with_config(
    request: Request<Body>,
    next: Next,
    config: &PanicHandlerConfig,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let result = AssertUnwindSafe(next.run(request)).catch_unwind().await;

    match result {
        Ok(response) => response,
        Err(panic_info) => {
            let panic_message = extract_panic_message(&panic_info);

            if config.log_panics {
                error!(
                    request_id = %request_id,
                    method = %method,
                    uri = %uri,
                    panic_message = %panic_message,
                    "Request handler panicked"
                );

                if config.include_backtrace {
                    let backtrace = std::backtrace::Backtrace::capture();
                    error!(backtrace = %backtrace, "Panic backtrace");
                }
            }

            if config.notify_on_panic {
                notify_panic(&request_id, method.as_ref(), &uri.to_string(), &panic_message);
            }

            create_panic_response(&request_id, config)
        }
    }
}

fn extract_panic_message(panic_info: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    }
}

fn create_panic_response(request_id: &str, config: &PanicHandlerConfig) -> Response {
    let message = config
        .custom_message
        .as_deref()
        .unwrap_or("An internal error occurred");

    let body = json!({
        "error": "internal_server_error",
        "message": message,
        "request_id": request_id
    });

    (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
}

fn notify_panic(request_id: &str, method: &str, uri: &str, message: &str) {
    warn!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        message = %message,
        "PANIC NOTIFICATION: Server panic occurred"
    );
}

pub fn set_global_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        error!(
            location = %location,
            message = %message,
            "Global panic handler caught panic"
        );
    }));
}

pub fn catch_panic<F, R>(f: F) -> Result<R, PanicError>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    catch_unwind(f).map_err(|e| PanicError {
        message: extract_panic_message(&e),
    })
}

pub async fn catch_panic_async<F, Fut, R>(f: F) -> Result<R, PanicError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    match AssertUnwindSafe(f()).catch_unwind().await {
        Ok(result) => Ok(result),
        Err(e) => Err(PanicError {
            message: extract_panic_message(&e),
        }),
    }
}

#[derive(Debug, Clone)]
pub struct PanicError {
    pub message: String,
}

impl std::fmt::Display for PanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Panic: {}", self.message)
    }
}

impl std::error::Error for PanicError {}

impl IntoResponse for PanicError {
    fn into_response(self) -> Response {
        let body = json!({
            "error": "internal_server_error",
            "message": "An internal error occurred"
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

pub struct PanicGuard {
    name: String,
    logged: bool,
}

impl PanicGuard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            logged: false,
        }
    }

    pub fn mark_completed(&mut self) {
        self.logged = true;
    }
}

impl Drop for PanicGuard {
    fn drop(&mut self) {
        if !self.logged && std::thread::panicking() {
            error!(
                guard_name = %self.name,
                "PanicGuard detected panic during drop"
            );
        }
    }
}

#[macro_export]
macro_rules! with_panic_guard {
    ($name:expr, $body:expr) => {{
        let mut guard = $crate::security::panic_handler::PanicGuard::new($name);
        let result = $body;
        guard.mark_completed();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PanicHandlerConfig::default();
        assert!(config.log_panics);
        assert!(!config.notify_on_panic);
    }

    #[test]
    fn test_production_config() {
        let config = PanicHandlerConfig::production();
        assert!(config.log_panics);
        assert!(!config.include_backtrace);
        assert!(config.notify_on_panic);
        assert!(config.custom_message.is_some());
    }

    #[test]
    fn test_development_config() {
        let config = PanicHandlerConfig::development();
        assert!(config.log_panics);
        assert!(config.include_backtrace);
        assert!(!config.notify_on_panic);
    }

    #[test]
    fn test_config_builder() {
        let config = PanicHandlerConfig::new()
            .with_message("Custom error")
            .with_backtrace(true)
            .with_logging(false)
            .with_notification(true);

        assert_eq!(config.custom_message, Some("Custom error".to_string()));
        assert!(config.include_backtrace);
        assert!(!config.log_panics);
        assert!(config.notify_on_panic);
    }

    #[test]
    fn test_extract_panic_message_str() {
        let panic: Box<dyn std::any::Any + Send> = Box::new("test panic");
        let message = extract_panic_message(&panic);
        assert_eq!(message, "test panic");
    }

    #[test]
    fn test_extract_panic_message_string() {
        let panic: Box<dyn std::any::Any + Send> = Box::new("string panic".to_string());
        let message = extract_panic_message(&panic);
        assert_eq!(message, "string panic");
    }

    #[test]
    fn test_extract_panic_message_unknown() {
        let panic: Box<dyn std::any::Any + Send> = Box::new(42i32);
        let message = extract_panic_message(&panic);
        assert_eq!(message, "Unknown panic");
    }

    #[test]
    fn test_catch_panic_success() {
        let result = catch_panic(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_catch_panic_failure() {
        let result = catch_panic(|| {
            panic!("test panic");
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("test panic"));
    }

    #[test]
    fn test_panic_error_display() {
        let error = PanicError {
            message: "test error".to_string(),
        };
        assert_eq!(format!("{}", error), "Panic: test error");
    }

    #[test]
    fn test_panic_guard_normal() {
        let mut guard = PanicGuard::new("test");
        guard.mark_completed();
    }

    #[tokio::test]
    async fn test_catch_panic_async_success() {
        let result = catch_panic_async(|| async { 42 }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_create_panic_response() {
        let config = PanicHandlerConfig::default();
        let response = create_panic_response("test-id", &config);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_create_panic_response_custom_message() {
        let config = PanicHandlerConfig::new().with_message("Custom error message");
        let response = create_panic_response("test-id", &config);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
