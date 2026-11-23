//! #2624 — admin view of DLP violation counts. Only counts are exposed; the
//! stored audit record is a content hash, never the secret itself.

use axum::{extract::State, http::StatusCode, Json};
use botcore::shared::state::AppState;
use std::sync::Arc;

pub(super) async fn handle_dlp_violations(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let count = botsecurity_core::dlp::violation_count().await;
    Ok(Json(serde_json::json!({
        "violations": count,
        "details": "counts only — matched content is never stored"
    })))
}
