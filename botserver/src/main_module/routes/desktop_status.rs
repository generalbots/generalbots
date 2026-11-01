//! Desktop shell status endpoints (#1620).

use axum::Json;
use serde_json::{json, Value};

/// GET /api/desktop/activity
///
/// Reports how many replies the sidecar is generating. The desktop shell uses
/// it as a secondary signal to the heartbeat file when deciding whether it may
/// terminate while the last window is gone.
pub async fn activity() -> Json<Value> {
    Json(json!({
        "active_turns": crate::core::desktop_activity::active_turns(),
    }))
}
