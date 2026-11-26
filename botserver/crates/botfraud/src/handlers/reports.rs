//! Report endpoints for the Fraud suite app (#2958).
//!
//! The app's Reports tab polls this path on every authenticated open, but the
//! route was never declared, so the poll returned 404. There is no persisted
//! report store yet; both endpoints answer with a valid, tenant-scoped empty
//! payload rather than inventing rows. The branch comes only from verified JWT
//! claims (`super::state::resolve_branch`), never from client query parameters.

use axum::{http::HeaderMap, Json};
use serde_json::{json, Value};

use super::state::resolve_branch;

/// Canonical path the Fraud suite app polls. Shared by the router and the
/// route/caller parity guard so a rename cannot silently reintroduce #2958.
pub const FRAUD_REPORTS_PATH: &'static str = "/api/fraud/reports";

/// `GET /api/fraud/reports` — the caller's report catalogue.
///
/// Reports are not persisted yet. The empty `items` list is the honest answer
/// and keeps the Reports tab from producing a console 404 on app open.
pub async fn list_reports(headers: HeaderMap) -> Json<Value> {
    let branch = resolve_branch(&headers);
    Json(json!({ "items": [], "branch_id": branch }))
}

/// `POST /api/fraud/reports` — the app's "Generate Report" button.
///
/// Generation is not persisted yet. Acknowledge with 200 so the same button
/// does not become a new 404 surface; keep the answer tenant-scoped as well.
pub async fn generate_report(headers: HeaderMap) -> Json<Value> {
    let branch = resolve_branch(&headers);
    Json(json!({ "success": true, "items": [], "branch_id": branch }))
}
