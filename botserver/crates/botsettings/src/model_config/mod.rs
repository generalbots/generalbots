//! Settings > Model — read, switch and test the LLM configuration (#2561).
//!
//! The wizard picks the initial model mode; this module lets the administrator
//! change it afterwards without re-running the wizard.  Every write goes
//! through the same Vault path (`secret/gbo/llm`) that the provisioner and the
//! on-device AI page use, and `ConfigManager::get_config` reads Vault live, so
//! the next conversation turn picks up the change with no restart.
//!
//! # Security
//!
//! - Admin-only (same gate as the on-device AI page).
//! - The API key is accepted on write and stored in Vault; it is NEVER echoed
//!   back on read — the response says whether one is stored, not what it is.
//! - `test` sends one tiny request with a short timeout and reports reachability
//!   and latency; the prompt is a fixed benign string, not user-controlled.
//! - No endpoint is contacted at start-up or as a side-effect of `GET /config`:
//!   outbound connections happen only on explicit `POST .../test`.

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use botsecurity::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use botcoresecrets::SecretsManager;

use super::on_device_ai::catalog as local_catalog;
use super::on_device_ai::hardware as local_hardware;

// ── types ────────────────────────────────────────────────────────────────

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn api_error(status: StatusCode, code: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": code })))
}

/// The modes the Settings UI can switch between.  `claude-code` and
/// `chatgpt-plan` are excluded: they need the agent workspace, which this
/// edition does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelMode {
    Local,
    OwnKey,
    Free,
}

impl ModelMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::OwnKey => "own_key",
            Self::Free => "free",
        }
    }
}

/// Infer the current mode from the stored endpoint URL.
fn infer_mode(url: &str, model: &str) -> ModelMode {
    if url.trim().is_empty() && model.trim().is_empty() {
        return ModelMode::Free;
    }
    if is_loopback(url) {
        return ModelMode::Local;
    }
    if is_free_endpoint(url) {
        return ModelMode::Free;
    }
    ModelMode::OwnKey
}

fn is_loopback(url: &str) -> bool {
    let low = url.to_ascii_lowercase();
    low.contains("localhost") || low.contains("127.0.0.1") || low.contains("[::1]")
}

/// The keyless free-tier endpoint shipped in the wizard defaults.
const FREE_ENDPOINT: &str = "https://api.kilo.ai/api/gateway/chat/completions";
const FREE_MODEL: &str = "dots-studio/dots-3-note-preview:free";
const FREE_PROVIDER: &str = "kilo";

fn is_free_endpoint(url: &str) -> bool {
    url.trim()
        .to_ascii_lowercase()
        .starts_with("https://api.kilo.ai/")
}

// ── auth ─────────────────────────────────────────────────────────────────

/// Same gate as the on-device AI page: admin, super-admin, or the single
/// desktop-local owner.
fn require_admin(user: &AuthenticatedUser) -> Result<(), (StatusCode, Json<Value>)> {
    let local_owner = user.metadata.contains_key("desktop_local");
    if user.is_admin() || user.is_super_admin() || local_owner {
        return Ok(());
    }
    Err(api_error(StatusCode::FORBIDDEN, "admin_required"))
}

// ── Vault path constant ──────────────────────────────────────────────────

const CENTRAL_LLM_PATH: &str = "gbo/llm";

// ── GET /api/llm/config — current model configuration ────────────────────

async fn handle_get_config(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> ApiResult {
    require_admin(&user)?;

    let cfg = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();

    let url = cfg.get_config(&nil, "llm-url", Some("")).unwrap_or_default();
    let model = cfg.get_config(&nil, "llm-model", Some("")).unwrap_or_default();
    let key = cfg.get_config(&nil, "llm-key", Some("")).unwrap_or_default();
    let provider = cfg.get_config(&nil, "llm-provider", Some("")).unwrap_or_default();

    let mode = infer_mode(&url, &model);
    let (host, scope) = crate::model_config::destination::describe(&url);
    let runtime = local_runtime::runtime_status().await;

    let active_local_model = if mode == ModelMode::Local {
        let bare = super::on_device_ai::vault_store::bare_name(&model);
        if bare.is_empty() { None } else { Some(bare.to_string()) }
    } else {
        None
    };

    Ok(Json(json!({
        "mode": mode.as_str(),
        "provider": provider,
        "endpoint_host": host,
        "endpoint_scope": scope.as_str(),
        "model": model,
        "has_key": !key.trim().is_empty(),
        "local_runtime": {
            "installed": runtime.installed,
            "runtime_reachable": runtime.runtime_reachable,
            "active_model": active_local_model,
            "installing": runtime.installing,
            "install_job": runtime.install_job,
            "install_info": runtime.install_info,
        },
    })))
}

// ── PUT /api/llm/config — switch the model configuration ─────────────────

#[derive(Deserialize)]
struct SwitchRequest {
    mode: ModelMode,
    /// Required for `own_key`; ignored for `local` and `free`.
    endpoint: Option<String>,
    /// Required for `own_key`; optional for `free` (has a default).
    model: Option<String>,
    /// The API key. Required for `own_key` when the endpoint needs one.
    /// Never echoed back.
    api_key: Option<String>,
    /// Provider hint (e.g. "openai"); auto-detected from the URL when absent.
    provider: Option<String>,
    /// For `local` mode: which downloaded model file to activate.
    local_model_id: Option<String>,
    /// For `local` mode: optional performance profile.
    local_profile: Option<String>,
}

async fn handle_put_config(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<SwitchRequest>,
) -> ApiResult {
    require_admin(&user)?;

    let secrets = SecretsManager::get_clone()
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "vault_unavailable"))?;
    if !secrets.is_enabled() {
        return Err(api_error(StatusCode::SERVICE_UNAVAILABLE, "vault_not_enabled"));
    }

    let mut central: HashMap<String, String> = secrets
        .get_secret(CENTRAL_LLM_PATH)
        .await
        .unwrap_or_default();

    match req.mode {
        ModelMode::Local => {
            // Guard: the inference runtime must be installed.
            let runtime = local_runtime::runtime_status().await;
            if !runtime.installed {
                return Err(api_error(StatusCode::CONFLICT, "runtime_not_installed"));
            }
            // Activate the specified local model.  The on_device_ai vault_store
            // handles the detailed write (model file + profile + per-bot path).
            let model_id = req.local_model_id.as_deref().unwrap_or("");
            if model_id.is_empty() {
                return Err(api_error(StatusCode::BAD_REQUEST, "local_model_id_required"));
            }
            let spec = local_catalog::find(model_id)
                .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "model_not_found"))?;
            let dir = local_hardware::models_dir();
            if !spec.server_only && !dir.join(spec.file).exists() {
                return Err(api_error(StatusCode::CONFLICT, "model_not_downloaded"));
            }
            let profile = match req.local_profile.as_deref() {
                Some(id) => Some(
                    local_catalog::find_profile(id)
                        .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "unknown_profile"))?,
                ),
                None => None,
            };
            super::on_device_ai::vault_store::save_choice(&state, spec, profile)
                .await
                .map_err(|reason| {
                    log::error!("[model_config] cannot persist local choice: {reason}");
                    api_error(StatusCode::INTERNAL_SERVER_ERROR, "vault_write_failed")
                })?;
        }

        ModelMode::OwnKey => {
            let endpoint = req.endpoint.as_deref().unwrap_or("").trim();
            let model = req.model.as_deref().unwrap_or("").trim();
            if endpoint.is_empty() {
                return Err(api_error(StatusCode::BAD_REQUEST, "endpoint_required"));
            }
            if model.is_empty() {
                return Err(api_error(StatusCode::BAD_REQUEST, "model_required"));
            }
            // Validate URL shape.
            if reqwest::Url::parse(endpoint).is_err() {
                return Err(api_error(StatusCode::BAD_REQUEST, "invalid_endpoint_url"));
            }
            central.insert("url".to_string(), endpoint.to_string());
            central.insert("model".to_string(), model.to_string());
            if let Some(key) = &req.api_key {
                if !key.trim().is_empty() {
                    central.insert("openai_key".to_string(), key.trim().to_string());
                }
            }
            if let Some(prov) = &req.provider {
                if !prov.trim().is_empty() {
                    central.insert("provider".to_string(), prov.trim().to_string());
                } else {
                    central.remove("provider");
                }
            } else {
                central.remove("provider");
            }
            secrets
                .put_secret(CENTRAL_LLM_PATH, central)
                .await
                .map_err(|e| {
                    log::error!("[model_config] vault write failed: {e}");
                    api_error(StatusCode::INTERNAL_SERVER_ERROR, "vault_write_failed")
                })?;
            // Mirror to the per-bot nil path so `get_config` resolves on both
            // lookup chains.
            let nil = uuid::Uuid::nil();
            let bot_path = format!("gbo/{nil}/{nil}/{nil}");
            let mut per_bot = secrets.get_secret(&bot_path).await.unwrap_or_default();
            per_bot.insert("llm-url".to_string(), endpoint.to_string());
            per_bot.insert("llm-model".to_string(), model.to_string());
            if let Some(key) = &req.api_key {
                if !key.trim().is_empty() {
                    per_bot.insert("llm-key".to_string(), key.trim().to_string());
                }
            }
            let _ = secrets.put_secret(&bot_path, per_bot).await;
        }

        ModelMode::Free => {
            central.insert("url".to_string(), FREE_ENDPOINT.to_string());
            central.insert("model".to_string(), FREE_MODEL.to_string());
            central.insert("provider".to_string(), FREE_PROVIDER.to_string());
            // The free tier is keyless: clear any stored key so it does not
            // leak from a previous own-key configuration.
            central.remove("openai_key");
            secrets
                .put_secret(CENTRAL_LLM_PATH, central)
                .await
                .map_err(|e| {
                    log::error!("[model_config] vault write failed: {e}");
                    api_error(StatusCode::INTERNAL_SERVER_ERROR, "vault_write_failed")
                })?;
        }
    }

    let restart_required = req.mode == ModelMode::Local;
    Ok(Json(json!({
        "ok": true,
        "mode": req.mode.as_str(),
        "effective_on_next_turn": !restart_required,
        "restart_required": restart_required,
    })))
}

mod test_endpoint;
mod local_runtime;

// ── GET /api/llm/config/modes — available modes for the edition ──────────

async fn handle_modes(
    Extension(user): Extension<AuthenticatedUser>,
) -> ApiResult {
    require_admin(&user)?;

    let hardware = local_hardware::probe();
    let models_dir = local_hardware::models_dir();
    let has_local_models = models_dir.exists()
        && std::fs::read_dir(&models_dir)
            .map(|mut d| d.any(|e| {
                e.ok()
                    .map(|e| e.path().extension().and_then(|x| x.to_str()) == Some("gguf"))
                    .unwrap_or(false)
            }))
            .unwrap_or(false);
    let runtime = local_runtime::runtime_status().await;

    Ok(Json(json!({
        "modes": [
            {
                "id": "local",
                "available": true,
                "runtime_installed": runtime.installed,
                "runtime_reachable": runtime.runtime_reachable,
                "has_downloaded_models": has_local_models,
                "hardware": {
                    "device": hardware.device,
                    "vram_gb": hardware.vram_gb,
                    "ram_gb": hardware.ram_gb,
                    "backend": hardware.backend,
                },
                "install_info": runtime.install_info,
                "note": if !runtime.installed {
                    "The inference engine is not installed. Install it first."
                } else if hardware.vram_gb == 0 {
                    "No GPU detected; the model will run on CPU (slower)."
                } else {
                    ""
                },
            },
            {
                "id": "own_key",
                "available": true,
                "note": "Connect to any OpenAI-compatible endpoint.",
            },
            {
                "id": "free",
                "available": true,
                "note": "Keyless hosted model; lower rate limits.",
            },
        ],
    })))
}

// ── routes ───────────────────────────────────────────────────────────────

pub fn configure_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/llm/config", get(handle_get_config))
        .route("/api/llm/config", put(handle_put_config))
        .route("/api/llm/config/test", post(test_endpoint::handle_test))
        .route("/api/llm/config/modes", get(handle_modes))
        .route("/api/llm/runtime/status", get(local_runtime::handle_status))
        .route("/api/llm/runtime/install", post(local_runtime::handle_install))
        .route("/api/llm/runtime/jobs/:id", get(local_runtime::handle_job_status))
}

pub(crate) mod destination;

// ── tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_mode_classifies_correctly() {
        assert_eq!(infer_mode("http://localhost:8081/v1/chat/completions", "model.gguf"), ModelMode::Local);
        assert_eq!(infer_mode("http://127.0.0.1:8081/v1", "m"), ModelMode::Local);
        assert_eq!(infer_mode(FREE_ENDPOINT, FREE_MODEL), ModelMode::Free);
        assert_eq!(infer_mode("https://api.kilo.ai/other", "x"), ModelMode::Free);
        assert_eq!(infer_mode("https://api.openai.com/v1/chat/completions", "gpt-4o"), ModelMode::OwnKey);
        assert_eq!(infer_mode("", ""), ModelMode::Free);
    }

    #[test]
    fn free_endpoint_detected() {
        assert!(is_free_endpoint(FREE_ENDPOINT));
        assert!(is_free_endpoint("https://api.kilo.ai/something"));
        assert!(!is_free_endpoint("https://api.openai.com/v1"));
    }
}
