//! Localized error payloads returned by the HTTP layer.

use super::{t, t_with_args, Locale, MessageArgs};

#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalizedError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl LocalizedError {
    pub fn new(locale: &Locale, code: &str) -> Self {
        Self {
            code: code.to_string(),
            message: t(locale, code),
            details: None,
        }
    }

    pub fn with_args(locale: &Locale, code: &str, args: &MessageArgs) -> Self {
        Self {
            code: code.to_string(),
            message: t_with_args(locale, code, Some(args)),
            details: None,
        }
    }

    pub fn not_found(locale: &Locale, entity: &str) -> Self {
        let mut args = MessageArgs::new();
        args.insert("entity".to_string(), entity.to_string());
        Self::with_args(locale, "error-http-404", &args)
    }

    pub fn validation(locale: &Locale, field: &str, error_key: &str) -> Self {
        let mut args = MessageArgs::new();
        args.insert("field".to_string(), field.to_string());
        Self::with_args(locale, error_key, &args)
    }

    pub fn internal(locale: &Locale) -> Self {
        Self::new(locale, "error-http-500")
    }

    pub fn unauthorized(locale: &Locale) -> Self {
        Self::new(locale, "error-http-401")
    }

    pub fn forbidden(locale: &Locale) -> Self {
        Self::new(locale, "error-http-403")
    }

    pub fn rate_limited(locale: &Locale, seconds: u64) -> Self {
        let mut args = MessageArgs::new();
        args.insert("seconds".to_string(), seconds.to_string());
        Self::with_args(locale, "error-http-429", &args)
    }

    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Backward-compatible JSON body (#2094).
    ///
    /// `code` is stable and machine-readable, `message` is localized, and the
    /// legacy `error` string carries the same localized text so clients that
    /// still read `error` keep working during the transition.
    #[must_use]
    pub fn body(&self) -> serde_json::Value {
        let mut body = serde_json::json!({
            "code": self.code,
            "message": self.message,
            "error": self.message,
        });

        if let (Some(details), Some(map)) = (&self.details, body.as_object_mut()) {
            map.insert("details".to_string(), details.clone());
        }

        body
    }
}

/// JSON body for a localized error without building a [`LocalizedError`].
#[must_use]
pub fn error_body(locale: &Locale, code: &str) -> serde_json::Value {
    LocalizedError::new(locale, code).body()
}

/// JSON body for a localized error that takes Fluent arguments.
#[must_use]
pub fn error_body_with_args(locale: &Locale, code: &str, args: &MessageArgs) -> serde_json::Value {
    LocalizedError::with_args(locale, code, args).body()
}
