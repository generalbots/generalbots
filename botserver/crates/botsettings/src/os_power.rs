//! Shell-mode power routes (#2444, #2445), split out of `os` to keep both
//! files inside the 450-line limit. Authorization is shared with the OS panel
//! through [`super::os::authorize`], so these routes carry the same gate:
//! shell mode, administrator and a loopback host.

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;
use serde::Deserialize;
use std::sync::Arc;

use crate::os::authorize;
use crate::os_data::read_power_status;

#[derive(Deserialize)]
pub struct PowerRequest {
    /// One of `signout`, `suspend`, `restart`, `shutdown`. `lock` never leaves
    /// the browser and is not accepted here.
    action: String,
}

/// #2445 — battery/AC status from `/sys/class/power_supply` (gated like OS panel).
pub async fn power_status(
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = authorize(&user, &headers) {
        return response.into_response();
    }
    Json(read_power_status()).into_response()
}

/// #2444 — the only path to a power action on the appliance.
///
/// logind's default policy lets the *active session* power the machine off
/// with no confirmation, and the shell's session bus is reachable from page
/// script (#2502). The image therefore denies logind power actions to the
/// kiosk user and grants exactly one helper (`/usr/local/libexec/gb-power`)
/// with a closed action list; this handler is the audited caller.
///
/// Gated exactly like the rest of the OS panel: shell mode, administrator,
/// loopback host. The action is matched against a fixed table, so nothing from
/// the request reaches the command line besides the chosen verb.
pub async fn power_action(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
    Json(request): Json<PowerRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&user, &headers) {
        return response.into_response();
    }

    let verb = match request.action.as_str() {
        "signout" => "signout",
        "suspend" => "suspend",
        "restart" => "reboot",
        "shutdown" => "poweroff",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "unknown_power_action" })),
            )
                .into_response();
        }
    };

    let result = tokio::task::spawn_blocking(move || {
        botsecurity::command_guard::SafeCommand::new("sudo")
            .and_then(|command| command.arg("/usr/local/libexec/gb-power"))
            .and_then(|command| command.arg(verb))
            .and_then(|command| command.execute())
    })
    .await;

    let success = matches!(&result, Ok(Ok(output)) if output.status.success());

    crate::audit_log::record_audit_event(crate::audit_log::AuditEvent {
        state: &state,
        event_type: "os_power_action",
        actor_id: user.user_id,
        action: "power_action",
        target_type: Some("host"),
        target_id: None,
        success,
        details: Some(verb),
    });

    if success {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "action": request.action })),
        )
            .into_response()
    } else {
        log::error!("power_action {verb} failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "power_action_failed" })),
        )
            .into_response()
    }
}
