//! Tauri commands for the agent runtime.

use serde::Deserialize;

use super::state::RuntimeStatus;

#[cfg(all(target_os = "windows", feature = "gbgov"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(all(target_os = "windows", feature = "gbgov"))]
use std::sync::{Arc, Mutex};

pub(crate) struct AgentRuntimeManager {
    #[cfg(all(target_os = "windows", feature = "gbgov"))]
    cancel: Arc<AtomicBool>,
    #[cfg(all(target_os = "windows", feature = "gbgov"))]
    running: Arc<AtomicBool>,
    /// Current progress snapshot for status polling.
    #[cfg(all(target_os = "windows", feature = "gbgov"))]
    progress: Arc<Mutex<ProgressSnapshot>>,
}

#[cfg(all(target_os = "windows", feature = "gbgov"))]
#[derive(Default, Clone)]
struct ProgressSnapshot {
    step: Option<String>,
    percent: Option<u8>,
    done_bytes: u64,
    total_bytes: u64,
}

impl AgentRuntimeManager {
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(all(target_os = "windows", feature = "gbgov"))]
            cancel: Arc::new(AtomicBool::new(false)),
            #[cfg(all(target_os = "windows", feature = "gbgov"))]
            running: Arc::new(AtomicBool::new(false)),
            #[cfg(all(target_os = "windows", feature = "gbgov"))]
            progress: Arc::new(Mutex::new(ProgressSnapshot::default())),
        }
    }
}

impl Default for AgentRuntimeManager {
    fn default() -> Self { Self::new() }
}

fn unsupported_status() -> RuntimeStatus {
    RuntimeStatus {
        state: "unsupported",
        detail: Some("agent runtime not available on this build".into()),
        runtime_version: None,
        available_version: None,
        download_bytes: 0,
        disk_bytes: 0,
        needs_admin: false,
        needs_reboot: false,
        managed: true,
        last_error: None,
        step: None,
        percent: None,
        done_bytes: 0,
        total_bytes: 0,
        paused: false,
    }
}

#[tauri::command]
pub fn agent_runtime_status(
    _mgr: tauri::State<'_, AgentRuntimeManager>,
) -> Result<RuntimeStatus, String> {
    #[cfg(all(target_os = "windows", feature = "gbgov"))]
    {
        let host = super::host::SystemHost;
        let stack = crate::desktop::sidecar::Sidecar::stack_path();
        let detection = super::detect::detect(&host, &stack);
        let mut status = super::detect::status_from_detection(&detection);

        // Load persisted last_error from state.json.
        let state_path = stack.join("agent-runtime").join("state.json");
        if let Ok(data) = host.read_file(&state_path) {
            if let Ok(ps) = super::state::PersistedState::load(&data) {
                status.last_error = ps.last_error;
                // paused: cancelled but resumable state on disk.
                if ps.completed_step.is_some()
                    && ps.completed_step != Some(super::state::ProvisionStep::Done)
                    && !_mgr.running.load(Ordering::Relaxed)
                {
                    status.paused = true;
                }
            }
        }

        if _mgr.running.load(Ordering::Relaxed) {
            status.state = "installing";
            if let Ok(snap) = _mgr.progress.lock() {
                status.step = snap.step.clone();
                status.percent = snap.percent;
                status.done_bytes = snap.done_bytes;
                status.total_bytes = snap.total_bytes;
            }
        }
        use super::host::RuntimeHost;
        Ok(status)
    }
    #[cfg(not(all(target_os = "windows", feature = "gbgov")))]
    {
        Ok(unsupported_status())
    }
}

#[tauri::command]
pub fn agent_runtime_install(
    _app: tauri::AppHandle,
    _mgr: tauri::State<'_, AgentRuntimeManager>,
) -> Result<(), String> {
    #[cfg(all(target_os = "windows", feature = "gbgov"))]
    {
        if _mgr.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        _mgr.cancel.store(false, Ordering::SeqCst);

        let cancel = Arc::clone(&_mgr.cancel);
        let running = Arc::clone(&_mgr.running);
        let progress = Arc::clone(&_mgr.progress);

        std::thread::spawn(move || {
            let host = super::host::SystemHost;
            let stack = crate::desktop::sidecar::Sidecar::stack_path();
            let events = TauriEvents {
                app: _app,
                progress,
            };
            let result = super::provision::run(&host, &stack, &events, &cancel);
            running.store(false, Ordering::SeqCst);
            if let Err(code) = result {
                log::info!("agent_runtime: provisioning ended: {code}");
            }
        });
        Ok(())
    }
    #[cfg(not(all(target_os = "windows", feature = "gbgov")))]
    {
        let _ = _app;
        Err("unsupported".into())
    }
}

#[tauri::command]
pub fn agent_runtime_cancel(
    _mgr: tauri::State<'_, AgentRuntimeManager>,
) -> Result<(), String> {
    #[cfg(all(target_os = "windows", feature = "gbgov"))]
    {
        _mgr.cancel.store(true, Ordering::SeqCst);
        Ok(())
    }
    #[cfg(not(all(target_os = "windows", feature = "gbgov")))]
    {
        Err("unsupported".into())
    }
}

#[derive(Deserialize)]
pub struct RemoveRequest {
    #[serde(default)]
    pub keep_export: bool,
}

#[tauri::command]
pub fn agent_runtime_remove(
    request: RemoveRequest,
    _mgr: tauri::State<'_, AgentRuntimeManager>,
) -> Result<(), String> {
    #[cfg(all(target_os = "windows", feature = "gbgov"))]
    {
        if _mgr.running.load(Ordering::Relaxed) {
            return Err("provisioning in progress".into());
        }
        let host = super::host::SystemHost;
        let stack = crate::desktop::sidecar::Sidecar::stack_path();
        match super::removal::remove(&host, &stack, request.keep_export) {
            super::removal::RemovalResult::Ok
            | super::removal::RemovalResult::NotRegistered => Ok(()),
            super::removal::RemovalResult::ForeignDistro { base_path } => {
                log::warn!(
                    "agent_runtime: refusing to remove distro at {}",
                    base_path.display()
                );
                Err("the distro is not managed by this application".into())
            }
            super::removal::RemovalResult::CommandFailed(msg) => {
                log::warn!("agent_runtime: removal failed: {msg}");
                Err("removal failed".into())
            }
            super::removal::RemovalResult::IoError(msg) => {
                log::warn!("agent_runtime: removal I/O error: {msg}");
                Err("removal failed".into())
            }
        }
    }
    #[cfg(not(all(target_os = "windows", feature = "gbgov")))]
    {
        let _ = request;
        Err("unsupported".into())
    }
}

#[tauri::command]
pub fn agent_runtime_check_update(
    _mgr: tauri::State<'_, AgentRuntimeManager>,
) -> Result<RuntimeStatus, String> {
    agent_runtime_status(_mgr)
}

// ── Tauri event emitter that also updates the progress snapshot ──────────

#[cfg(all(target_os = "windows", feature = "gbgov"))]
struct TauriEvents {
    app: tauri::AppHandle,
    progress: Arc<Mutex<ProgressSnapshot>>,
}

#[cfg(all(target_os = "windows", feature = "gbgov"))]
impl super::provision::ProvisionEvents for TauriEvents {
    fn progress(&self, event: super::state::ProgressEvent) {
        if let Ok(mut snap) = self.progress.lock() {
            snap.step = Some(event.step.to_string());
            snap.percent = Some(event.percent);
            snap.done_bytes = event.done_bytes;
            snap.total_bytes = event.total_bytes;
        }
        use tauri::Emitter;
        let _ = self.app.emit("agent-runtime://progress", &event);
    }

    fn error(&self, event: super::state::ErrorEvent) {
        use tauri::Emitter;
        let _ = self.app.emit("agent-runtime://error", &event);
    }
}
