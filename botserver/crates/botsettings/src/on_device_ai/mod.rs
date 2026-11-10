//! Settings → On-device AI (#2109).
//!
//! Local-only model management: the catalogue is grouped by VRAM tier, each
//! row can be downloaded, activated or deleted, and the active choice is
//! persisted in Vault (`secret/gbo/llm`). No external provider keys are read
//! or written here — this page manages weights that run on the device.
//!
//! Activation writes the choice and reports `restart_required`: the local
//! engine picks the new weights up on its next start. Restarting the engine
//! in-process is out of scope for this slice.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

mod catalog;
mod hardware;
mod jobs;
mod vault_store;

use catalog::ModelSpec;

type ApiError = (StatusCode, Json<Value>);

fn error(status: StatusCode, code: &str) -> ApiError {
    (status, Json(json!({ "error": code })))
}

fn require_admin(user: &AuthenticatedUser) -> Result<(), ApiError> {
    if user.is_admin() || user.is_super_admin() {
        return Ok(());
    }
    Err(error(StatusCode::FORBIDDEN, "admin_required"))
}

pub fn configure_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/llm/local/hardware", get(hardware_status))
        .route("/api/llm/local/models", get(list_models))
        .route("/api/llm/local/models/:id/download", post(start_download))
        .route("/api/llm/local/models/:id", delete(delete_model))
        .route("/api/llm/local/jobs/:id", get(job_status))
        .route("/api/llm/local/jobs/:id/cancel", post(cancel_job))
        .route("/api/llm/local/activate", post(activate))
        .route("/api/llm/local/profiles", get(list_profiles))
}

async fn hardware_status(
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    let probe = hardware::probe();
    Ok(Json(json!(probe)))
}

async fn list_models(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    let probe = hardware::probe();
    let configured = vault_store::active_model(&state);
    let dir = hardware::models_dir();
    let tiers: Vec<Value> = catalog::tier_groups()
        .iter()
        .map(|group| {
            let rows: Vec<Value> = group
                .models
                .iter()
                .copied()
                .map(|spec| model_row(spec, &probe, &configured, &dir))
                .collect();
            json!({ "tier_gb": group.tier_gb, "label": &group.label, "models": rows })
        })
        .collect();
    let recommended = catalog::recommended(probe.vram_gb, probe.ram_gb);
    Ok(Json(json!({
        "hardware": &probe,
        "active_model": vault_store::bare_name(&configured),
        "recommended": recommended.id,
        "recommended_fits": catalog::fits(recommended, probe.vram_gb, probe.ram_gb),
        "models_dir": dir.to_string_lossy(),
        "free_bytes": hardware::free_bytes_for(&dir),
        "tiers": tiers,
    })))
}

fn model_row(
    spec: &'static ModelSpec,
    probe: &hardware::Hardware,
    configured: &str,
    dir: &std::path::Path,
) -> Value {
    let active = vault_store::is_active(configured, spec);
    let downloaded = dir.join(spec.file).exists();
    let partial = std::fs::metadata(dir.join(format!("{}.part", spec.file)))
        .map(|meta| meta.len())
        .unwrap_or(0);
    let progress = jobs::progress(spec.id);
    let status = catalog::row_status(active, downloaded, progress.is_some());
    json!({
        "id": spec.id,
        "name": spec.name,
        "params": spec.params,
        "quant": spec.quant,
        "size_bytes": spec.size_bytes,
        "min_vram_gb": spec.min_vram_gb,
        "repo": spec.repo,
        "file": spec.file,
        "status": status,
        "progress": progress,
        "partial_bytes": partial,
        "active": active,
        "downloaded": downloaded,
        "fits": catalog::fits(spec, probe.vram_gb, probe.ram_gb),
        "checksum_pinned": !spec.sha256.is_empty(),
        "server_only": spec.server_only,
    })
}

async fn start_download(
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    let spec = catalog::find(&id).ok_or_else(|| error(StatusCode::NOT_FOUND, "model_not_found"))?;
    if spec.repo.is_empty() {
        return Err(error(StatusCode::CONFLICT, "no_remote_source"));
    }
    let dir = hardware::models_dir();
    if let Some(free) = hardware::free_bytes_for(&dir) {
        let needed = spec.size_bytes.saturating_add(spec.size_bytes / 10);
        if free < needed {
            return Err(error(StatusCode::CONFLICT, "insufficient_disk"));
        }
    }
    match jobs::start(spec, &dir) {
        Ok(job_id) => Ok(Json(json!({ "job_id": job_id }))),
        Err(reason) => {
            log::error!("[on_device_ai] cannot start download for {id}: {reason}");
            Err(error(StatusCode::INTERNAL_SERVER_ERROR, "download_unavailable"))
        }
    }
}

async fn job_status(
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    match jobs::snapshot(&id) {
        Some(snapshot) => Ok(Json(json!(snapshot))),
        None => Err(error(StatusCode::NOT_FOUND, "job_not_found")),
    }
}

async fn cancel_job(
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    match jobs::cancel(&id) {
        Ok(()) => Ok(Json(json!({ "ok": true }))),
        Err(reason) => Err(error(StatusCode::NOT_FOUND, &reason)),
    }
}

async fn delete_model(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    let spec = catalog::find(&id).ok_or_else(|| error(StatusCode::NOT_FOUND, "model_not_found"))?;
    let configured = vault_store::active_model(&state);
    if vault_store::is_active(&configured, spec) {
        return Err(error(StatusCode::CONFLICT, "active_model_cannot_be_deleted"));
    }
    let dir = hardware::models_dir();
    for path in [dir.join(spec.file), dir.join(format!("{}.part", spec.file))] {
        if path.exists() {
            if let Err(reason) = std::fs::remove_file(&path) {
                log::error!("[on_device_ai] cannot delete {}: {reason}", path.display());
                return Err(error(StatusCode::INTERNAL_SERVER_ERROR, "delete_failed"));
            }
        }
    }
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct ActivateRequest {
    model_id: String,
    profile: Option<String>,
}

async fn activate(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<ActivateRequest>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    let spec = catalog::find(&request.model_id)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "model_not_found"))?;
    let dir = hardware::models_dir();
    if !spec.server_only && !dir.join(spec.file).exists() {
        return Err(error(StatusCode::CONFLICT, "model_not_downloaded"));
    }
    let profile = match request.profile.as_deref() {
        Some(id) => Some(
            catalog::find_profile(id)
                .ok_or_else(|| error(StatusCode::BAD_REQUEST, "unknown_profile"))?,
        ),
        None => None,
    };
    vault_store::save_choice(&state, spec, profile)
        .await
        .map_err(|reason| {
            log::error!("[on_device_ai] cannot persist choice: {reason}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "vault_write_failed")
        })?;
    Ok(Json(json!({
        "active": spec.file,
        "profile": profile.map(|profile| profile.id),
        "restarted": false,
        "restart_required": true,
    })))
}

async fn list_profiles(
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, ApiError> {
    require_admin(&user)?;
    Ok(Json(json!(catalog::profiles())))
}
