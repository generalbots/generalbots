//! First-run wizard for the desktop edition (issues #1586-#1591).
//!
//! The wizard is a separate static webview that runs *before* the local stack
//! exists. It never depends on botserver, botui, Vault, PostgreSQL, MinIO or the
//! network: detection is a marker file, and provisioning is delegated to
//! `botserver --provision-stack` so the component order and offline rules stay
//! in one place.

mod commands;
mod provision;
mod system;

pub use commands::*;

use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use super::sidecar::Sidecar;

/// Components provisioned by the wizard, in dependency order.
pub const COMPONENTS: &[(&str, &str, bool)] = &[
    ("vault", "Local vault", false),
    ("tables", "Database", false),
    ("cache", "Cache", false),
    ("drive", "Object store", false),
    ("vector_db", "Vector store", false),
    ("directory", "Identity provider", false),
    ("llm", "Local model runtime", true),
    ("botserver", "Assistant service", false),
];

#[derive(Default)]
pub struct WizardState {
    /// Pid of the `botserver --provision-stack` child, used only to cancel it.
    pub pid: Mutex<Option<u32>>,
    pub cancelled: AtomicBool,
    pub provisioning: AtomicBool,
    /// Last known state of every component, for clients that poll instead of
    /// listening to `wizard://progress` (events can be missed if the webview
    /// subscribes late).
    pub progress: Mutex<Vec<ProgressEvent>>,
    pub failed: Mutex<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionStatus {
    pub running: bool,
    pub ready: bool,
    pub failed: Option<String>,
    pub components: Vec<ProgressEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub id: String,
    pub label: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardContext {
    pub desktop_mode: bool,
    pub first_run: bool,
    pub desktop_local: bool,
    pub stack_path: String,
    pub workspace_path: String,
    pub log_path: String,
    pub platform: String,
    pub api_base: String,
    pub redirect_uri: String,
    pub resume_step: Option<u8>,
    pub completed_components: Vec<String>,
    pub components: Vec<ComponentSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub id: String,
    pub state: String,
    pub message: String,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceClient {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelDraft {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceDraft {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionRequest {
    #[serde(default)]
    pub model: ModelDraft,
    #[serde(default)]
    pub workspace: WorkspaceDraft,
    #[serde(default)]
    pub telemetry: bool,
    #[serde(default)]
    pub services: std::collections::HashMap<String, ServiceClient>,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminDraft {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishRequest {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub telemetry: bool,
    #[serde(default)]
    pub model: ModelDraft,
    #[serde(default)]
    pub workspace: WorkspaceDraft,
    #[serde(default)]
    pub services: std::collections::HashMap<String, ServiceClient>,
    #[serde(default)]
    pub admin: AdminDraft,
}

pub(crate) fn stack_path() -> PathBuf {
    Sidecar::stack_path()
}

pub(crate) fn installers_path(resource_dir: Option<&Path>) -> PathBuf {
    // An explicit path wins (development runs point at the checkout's bundled
    // installers); packaged builds fall back to the resource directory.
    if let Ok(value) = std::env::var("BOTSERVER_INSTALLERS_PATH") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    resource_dir
        .map(|dir| dir.join("installers"))
        .unwrap_or_else(|| stack_path().join("installers"))
}

pub(crate) fn marker_path() -> PathBuf {
    stack_path().join(".initialized")
}

pub(crate) fn progress_path() -> PathBuf {
    stack_path().join(".wizard-progress.json")
}

pub(crate) fn draft_path() -> PathBuf {
    stack_path().join(".wizard-draft.json")
}

pub(crate) fn log_path() -> PathBuf {
    stack_path().join("logs")
}

pub(crate) fn workspace_path() -> PathBuf {
    if let Ok(value) = std::env::var("GB_WIZARD_WORKSPACE") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    stack_path().join("workspace")
}

/// Desktop mode is explicit: the desktop-local feature or the desktop product
/// file. No heuristics based on a slow API.
pub fn is_desktop_mode() -> bool {
    cfg!(feature = "desktop-local")
        || std::env::var("GB_DESKTOP_MODE").map(|v| v == "1").unwrap_or(false)
        || stack_path().join(".product.desktop-local").exists()
}

/// First run is the absence of the persisted marker. An existing folder without
/// the marker is still a first run (it may be a corrupt or partial stack).
pub fn is_first_run() -> bool {
    is_desktop_mode() && !marker_path().exists()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ProgressMarker {
    #[serde(default)]
    completed: Vec<String>,
    #[serde(default)]
    step: u8,
}

fn read_progress() -> ProgressMarker {
    let raw = match std::fs::read_to_string(progress_path()) {
        Ok(raw) => raw,
        Err(_) => return ProgressMarker::default(),
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub(crate) fn write_progress(marker: &ProgressMarker) -> std::io::Result<()> {
    let stack = stack_path();
    std::fs::create_dir_all(&stack)?;
    let raw = serde_json::to_string_pretty(marker)
        .map_err(|e| std::io::Error::other(format!("serialize progress: {e}")))?;
    std::fs::write(progress_path(), raw)
}

fn components() -> Vec<ComponentSpec> {
    COMPONENTS
        .iter()
        .map(|(id, label, optional)| ComponentSpec {
            id: (*id).to_string(),
            label: (*label).to_string(),
            optional: *optional,
        })
        .collect()
}

pub(crate) fn context() -> WizardContext {
    let progress = read_progress();
    let resume_step = if progress.step > 0 && !marker_path().exists() {
        Some(progress.step)
    } else {
        None
    };
    WizardContext {
        desktop_mode: is_desktop_mode(),
        first_run: is_first_run(),
        desktop_local: cfg!(feature = "desktop-local"),
        stack_path: stack_path().to_string_lossy().to_string(),
        workspace_path: workspace_path().to_string_lossy().to_string(),
        log_path: log_path().to_string_lossy().to_string(),
        platform: std::env::consts::OS.to_string(),
        api_base: "http://127.0.0.1:8080".to_string(),
        redirect_uri: "http://127.0.0.1:8080/api/integrations/oauth/{provider}/callback".to_string(),
        resume_step,
        completed_components: progress.completed,
        components: components(),
    }
}

/// Creates the wizard window and keeps the main window hidden.
pub fn open_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("wizard") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    let mut width = 1040.0_f64;
    let mut height = 700.0_f64;
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let scale = if monitor.scale_factor() > 0.0 {
            monitor.scale_factor()
        } else {
            1.0
        };
        let available_width = f64::from(monitor.size().width) / scale;
        let available_height = f64::from(monitor.size().height) / scale;
        width = width.min(available_width - 80.0);
        height = height.min(available_height - 80.0);
    }

    // In development the wizard is served by botui (devUrl). WebKit may cache
    // the previous page between runs, so force a fresh document each start.
    let wizard_url = if cfg!(debug_assertions) {
        match &app.config().build.dev_url {
            Some(dev) => {
                let mut url = dev.clone();
                url.set_path("/wizard/index.html");
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                url.set_query(Some(&format!("run={stamp}")));
                WebviewUrl::External(url)
            }
            None => WebviewUrl::App("wizard/index.html".into()),
        }
    } else {
        WebviewUrl::App("wizard/index.html".into())
    };

    info!("wizard: opening {wizard_url:?}");
    let builder = WebviewWindowBuilder::new(app, "wizard", wizard_url)
    .title("Set up General Bots")
    .inner_size(width, height)
    .min_inner_size(900.0, 560.0)
    .resizable(true)
    .center()
    .visible(true);

    builder
        .build()
        .map(|_| ())
        .map_err(|e| format!("could not open the setup window: {e}"))
}

/// Re-navigates the hidden suite window to its entry point and reveals it.
///
/// The window is created at startup, before the API answers, so it may have
/// been redirected to the login page by the auth gate. Navigating again (rather
/// than reloading) guarantees the suite boots with the final product config.
pub fn reveal_suite(app: &tauri::AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        // Desktop installs point the window at the sidecar that serves the
        // suite, so `/api` and `/ws` share the page origin. The window is
        // created on that URL at startup; once the API answers we only need
        // to reload it (judging by the page, not by a cross-origin navigate
        // that tears the webview down).
        let explicit = std::env::var("GB_DESKTOP_SUITE_URL")
            .ok()
            .and_then(|value| tauri::Url::parse(value.trim()).ok());
        if let Some(url) = explicit {
            // Desktop-local runs without an identity provider account: hand the
            // suite an internal token so its client-side gate never sends the
            // user to the login screen (the sidecar accepts it in no-security
            // mode). It is a local marker, not a credential.
            let _ = main.eval(
                "try{localStorage.setItem('gb-access-token','desktop-local');\
                 localStorage.setItem('gb-desktop-local','1');}catch(e){}",
            );
            let current = main.url().ok();
            let same_origin = current
                .as_ref()
                .map(|c| {
                    c.scheme() == url.scheme()
                        && c.host_str() == url.host_str()
                        && c.port_or_known_default() == url.port_or_known_default()
                })
                .unwrap_or(false);
            if same_origin {
                if let Err(e) = main.reload() {
                    warn!("could not reload the suite window: {e}");
                }
            } else {
                match serde_json::to_string(url.as_str()) {
                    Ok(literal) => {
                        let script = format!("window.location.replace({literal});");
                        if let Err(e) = main.eval(&script) {
                            warn!("could not navigate the suite window: {e}");
                        }
                    }
                    Err(e) => warn!("could not serialise the suite URL: {e}"),
                }
            }
        } else if let Some(dev) = app.config().build.dev_url.clone() {
            let mut url = dev;
            url.set_path("/suite/desktop.html");
            url.set_query(None);
            if let Err(e) = main.navigate(url) {
                warn!("could not navigate the suite window: {e}");
            }
        }
        let _ = main.show();
        let _ = main.set_focus();
    } else {
        warn!("main window is not available during wizard handoff");
    }
}

/// Writes the administrator account the wizard showed on the finish screen.
/// The login fallback reads this file from the stack directory; the password is
/// never logged and the file is restricted to the current user.
fn write_admin_credentials(request: &FinishRequest) -> Result<(), String> {
    let admin = &request.admin;
    if admin.email.trim().is_empty() || admin.password.is_empty() {
        return Ok(());
    }
    let dir = stack_path().join("conf").join("directory");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create directory config: {e}"))?;
    let user_id = if admin.user_id.trim().is_empty() {
        format!("local-{}", admin.email.trim())
    } else {
        admin.user_id.clone()
    };
    let payload = serde_json::json!({
        "email": admin.email.trim(),
        "password": admin.password,
        "user_id": user_id,
    });
    let raw = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("serialize administrator credentials: {e}"))?;
    let path = dir.join("admin-credentials.json");
    std::fs::write(&path, raw).map_err(|e| format!("write administrator credentials: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
    info!("wizard: administrator credentials written to {}", path.display());
    Ok(())
}

/// Opens the suite once the stack answers, and closes the wizard.
pub fn handoff(app: &tauri::AppHandle, request: &FinishRequest) -> Result<(), String> {
    let stack = stack_path();
    std::fs::create_dir_all(&stack).map_err(|e| format!("create stack folder: {e}"))?;
    write_admin_credentials(request)?;
    let marker = serde_json::json!({
        "completed_at": chrono::Utc::now().to_rfc3339(),
        "desktop_local": cfg!(feature = "desktop-local"),
        "workspace": workspace_path().to_string_lossy(),
    });
    let raw = serde_json::to_string_pretty(&marker)
        .map_err(|e| format!("serialize marker: {e}"))?;
    std::fs::write(marker_path(), raw).map_err(|e| format!("write marker: {e}"))?;

    reveal_suite(app);

    if let Some(wizard) = app.get_webview_window("wizard") {
        if let Err(e) = wizard.close() {
            warn!("could not close the wizard window: {e}");
        }
    }
    info!("first-run wizard completed; suite handed off");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_order_is_dependency_first() {
        let ids: Vec<&str> = COMPONENTS.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(
            ids,
            vec!["vault", "tables", "cache", "drive", "vector_db", "directory", "llm", "botserver"]
        );
    }

    #[test]
    fn progress_marker_round_trips() {
        let marker = ProgressMarker {
            completed: vec!["vault".to_string(), "tables".to_string()],
            step: 5,
        };
        let raw = serde_json::to_string(&marker).unwrap_or_default();
        let back: ProgressMarker = serde_json::from_str(&raw).unwrap_or_default();
        assert_eq!(back.completed.len(), 2);
        assert_eq!(back.step, 5);
    }
}
