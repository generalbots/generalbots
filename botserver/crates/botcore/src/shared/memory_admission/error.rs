//! Typed admission refusals with stable HTTP/i18n contracts (#2530, Phase 1).
//!
//! The JSON body carries a machine code, an i18n catalog key and an English
//! fallback message. It never carries document names, bytes or tenant
//! content — those belong only in the structured log event emitted by the
//! gate, and even there only lengths are recorded.

use super::policy::WorkClass;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    /// Upload exceeds the endpoint's fixed cap. HTTP 413.
    PayloadTooLarge {
        class: WorkClass,
        bytes: u64,
        limit_bytes: u64,
    },
    /// Estimated peak heap exceeds the cgroup-derived request budget. HTTP 507.
    BudgetExceeded {
        class: WorkClass,
        estimated_peak_bytes: u64,
        budget_bytes: u64,
    },
    /// All permits for this class are in use. HTTP 429.
    Busy { class: WorkClass, queued: usize },
}

impl AdmissionError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::PayloadTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            Self::BudgetExceeded { .. } => StatusCode::INSUFFICIENT_STORAGE,
            Self::Busy { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::PayloadTooLarge { .. } => "office_payload_too_large",
            Self::BudgetExceeded { .. } => "office_memory_budget_exceeded",
            Self::Busy { .. } => "office_busy",
        }
    }

    /// Catalog key for the suite to translate. The catalogs themselves are
    /// owned by the i18n workstream, not by this module.
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::PayloadTooLarge { .. } => "office.admission.payload-too-large",
            Self::BudgetExceeded { .. } => "office.admission.memory-budget-exceeded",
            Self::Busy { .. } => "office.admission.busy",
        }
    }

    pub fn class(&self) -> WorkClass {
        match self {
            Self::PayloadTooLarge { class, .. }
            | Self::BudgetExceeded { class, .. }
            | Self::Busy { class, .. } => *class,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::PayloadTooLarge {
                bytes, limit_bytes, ..
            } => format!(
                "This document is too large for this operation ({bytes} bytes; limit {limit_bytes} bytes). Try a smaller file."
            ),
            Self::BudgetExceeded {
                estimated_peak_bytes,
                budget_bytes,
                ..
            } => format!(
                "This document needs an estimated {estimated_peak_bytes} bytes to open, above this machine's {budget_bytes}-byte request budget. Try a smaller file or retry later."
            ),
            Self::Busy { queued, .. } => format!(
                "Another document operation is still running ({queued} waiting). Please try again shortly."
            ),
        }
    }

    pub fn json_body(&self) -> serde_json::Value {
        let mut body = serde_json::json!({
            "error": self.code(),
            "i18n_key": self.i18n_key(),
            "message": self.message(),
            "class": self.class().as_str(),
        });
        if let Some(object) = body.as_object_mut() {
            match self {
                Self::PayloadTooLarge {
                    bytes, limit_bytes, ..
                } => {
                    object.insert("bytes".to_string(), serde_json::json!(bytes));
                    object.insert("limit_bytes".to_string(), serde_json::json!(limit_bytes));
                }
                Self::BudgetExceeded {
                    estimated_peak_bytes,
                    budget_bytes,
                    ..
                } => {
                    object.insert(
                        "estimated_peak_bytes".to_string(),
                        serde_json::json!(estimated_peak_bytes),
                    );
                    object.insert("budget_bytes".to_string(), serde_json::json!(budget_bytes));
                }
                Self::Busy { queued, .. } => {
                    object.insert("queued".to_string(), serde_json::json!(queued));
                }
            }
        }
        body
    }

    pub fn into_response(&self) -> Response {
        let body = axum::Json(self.json_body());
        match self {
            Self::Busy { .. } => (self.status_code(), [("Retry-After", "1")], body).into_response(),
            _ => (self.status_code(), body).into_response(),
        }
    }
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
}

impl std::error::Error for AdmissionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;

    const SECRET_NAME: &str = "confidential-salary-sheet.xlsx";

    #[test]
    fn statuses_are_stable_per_reason() {
        let too_large = AdmissionError::PayloadTooLarge {
            class: WorkClass::SheetsImport,
            bytes: 26 * 1024 * 1024,
            limit_bytes: 25 * 1024 * 1024,
        };
        let out_of_budget = AdmissionError::BudgetExceeded {
            class: WorkClass::SlidesImport,
            estimated_peak_bytes: 900,
            budget_bytes: 400,
        };
        let busy = AdmissionError::Busy {
            class: WorkClass::DocsImport,
            queued: 2,
        };
        assert_eq!(too_large.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            out_of_budget.status_code(),
            StatusCode::INSUFFICIENT_STORAGE
        );
        assert_eq!(busy.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(too_large.status_code().as_u16(), 413);
        assert_eq!(out_of_budget.status_code().as_u16(), 507);
        assert_eq!(busy.status_code().as_u16(), 429);
    }

    #[test]
    fn codes_and_i18n_keys_do_not_drift() {
        let errors = [
            AdmissionError::PayloadTooLarge {
                class: WorkClass::DocsImport,
                bytes: 1,
                limit_bytes: 0,
            },
            AdmissionError::BudgetExceeded {
                class: WorkClass::DocsImport,
                estimated_peak_bytes: 2,
                budget_bytes: 1,
            },
            AdmissionError::Busy {
                class: WorkClass::DocsImport,
                queued: 0,
            },
        ];
        for error in errors {
            assert!(error.code().starts_with("office_"));
            assert!(error.i18n_key().starts_with("office.admission."));
            assert_eq!(error.class(), WorkClass::DocsImport);
        }
    }

    #[test]
    fn response_carries_retry_after_only_when_busy() {
        let busy = AdmissionError::Busy {
            class: WorkClass::DocsImport,
            queued: 1,
        };
        let response = busy.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let retry_after = response
            .headers()
            .get(header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok());
        assert_eq!(retry_after, Some("1"));
        let too_large = AdmissionError::PayloadTooLarge {
            class: WorkClass::DocsImport,
            bytes: 1,
            limit_bytes: 0,
        };
        let response = too_large.into_response();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert!(response.headers().get(header::RETRY_AFTER).is_none());
    }

    #[test]
    fn body_and_display_never_leak_document_content() {
        let error = AdmissionError::PayloadTooLarge {
            class: WorkClass::SheetsImport,
            bytes: 26 * 1024 * 1024,
            limit_bytes: 25 * 1024 * 1024,
        };
        let serialized = error.json_body().to_string();
        assert!(serialized.contains("office_payload_too_large"));
        assert!(serialized.contains("sheets_import"));
        assert!(!serialized.contains(SECRET_NAME));
        assert!(!error.to_string().contains(SECRET_NAME));
    }
}
