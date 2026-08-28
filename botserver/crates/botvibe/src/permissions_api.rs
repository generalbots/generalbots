use crate::permissions::{PermissionEngineRef, PermissionMode};
use axum::{Extension, Json, Router};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct PermissionResponse {
    success: bool,
    mode: String,
    destructive_tools: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetPermissionRequest {
    pub mode: String,
}

#[derive(Debug, Serialize)]
struct SetPermissionResponse {
    success: bool,
    mode: String,
    error: Option<String>,
}

pub fn permissions_router(engine: PermissionEngineRef) -> Router {
    Router::new()
        .route("/api/vibe/permissions", axum::routing::get(get_permissions))
        .route("/api/vibe/permissions", axum::routing::put(set_permissions))
        .layer(Extension(engine))
}

async fn get_permissions(
    Extension(engine): Extension<PermissionEngineRef>,
) -> Json<PermissionResponse> {
    let mode = engine.mode().await;
    Json(PermissionResponse {
        success: true,
        mode: mode.to_string(),
        destructive_tools: DESTRUCTIVE_LIST.iter().map(|s| s.to_string()).collect(),
    })
}

const DESTRUCTIVE_LIST: &[&str] = &[
    "file/delete",
    "file/write",
    "file/replace",
    "file/set-title",
    "shell/run",
    "git/commit",
    "git/init",
    "git/snapshot-previous",
    "git/push",
    "git/pr",
    "git/checkout",
    "publish/project",
    "deploy_app",
    "domain/bind",
    "domain/unbind",
    "domain/security",
    "domain/tls",
    "ops/restart",
    "ops/rollback",
    "backup/restore",
    "backup/snapshot",
    "test/run",
    "canvas/update",
    "canvas/delete",
    "issue/update",
    "issue/close",
    "skill/delete",
    "browser/close",
    "browser/eval",
];

async fn set_permissions(
    Extension(engine): Extension<PermissionEngineRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<SetPermissionRequest>,
) -> Json<SetPermissionResponse> {
    // #919 — only an authenticated administrator may change the global
    // permission mode; a normal user or prompt-injected caller cannot flip
    // the whole deployment to `bypass`.
    if !user.is_admin() {
        log::warn!(
            "Vibe permissions API: non-admin user {} attempted to set mode",
            user.user_id
        );
        return Json(SetPermissionResponse {
            success: false,
            mode: engine.mode().await.to_string(),
            error: Some("forbidden: only administrators can change the permission mode".into()),
        });
    }
    let mode = match req.mode.as_str() {
        "manual" => PermissionMode::Manual,
        "auto" => PermissionMode::Auto,
        "bypass" => PermissionMode::Bypass,
        _ => {
            return Json(SetPermissionResponse {
                success: false,
                mode: engine.mode().await.to_string(),
                error: Some("mode must be one of: manual, auto, bypass".into()),
            });
        }
    };
    engine.set_mode(mode).await;
    Json(SetPermissionResponse {
        success: true,
        mode: mode.to_string(),
        error: None,
    })
}
