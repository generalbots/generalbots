/* =============================================================================
 * ui_fragments/mod.rs — HTMX fragment handlers for brazil/timeclock/minutes.
 * Server-side Rust rendering; JS in botui is reduced to thin HTMX consumers.
 *
 * Routes mounted under /suite/{app}/fragments/... and /api/{app}/forms/...
 * Each handler returns axum::response::Html<String> with rendered markup.
 *
 * No panic/unwrap/todo per AGENTS.md security directives — every error
 * path returns Err((StatusCode, String)) and is mapped to a sanitized
 * fragment with the error inlined.
 * =============================================================================*/
pub use axum::{
    extract::{Form, Path, Query},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
pub use chrono::Utc;
pub use diesel::RunQueryDsl;
pub use rust_decimal::Decimal;
pub use serde::Deserialize;
pub use std::collections::HashMap;
pub use uuid::Uuid;

pub mod brazil;
pub mod brazil_queries;
pub mod minutes_app;
pub mod minutes_app_forms;
pub mod timeclock;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::<S>::new()
        .merge(brazil::configure())
        .merge(timeclock::configure())
        .merge(minutes_app::configure())
}

pub(super) fn htmx_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(super) fn fmt_decimal(s: &str) -> String {
    if let Ok(d) = Decimal::from_str_exact(s) {
        let normalized = d.normalize();
        if normalized.fract().is_zero() {
            normalized.to_string()
        } else {
            let s = normalized.to_string();
            if s.contains('.') {
                s
            } else {
                format!("{s}.00")
            }
        }
    } else {
        s.to_string()
    }
}

pub(super) fn err_fragment(msg: &str) -> String {
    format!(
        r##"<div class="gb-fragment-error" role="alert" style="padding:12px 16px;background:rgba(239,68,68,.1);border:1px solid #ef4444;border-radius:6px;color:#fca5a5;font-size:13px">⚠ {msg}</div>"##,
        msg = htmx_escape(msg),
    )
}
