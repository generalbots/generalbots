//! Provisioning child process: spawn, progress forwarding and assistant start.

use super::super::{
    installers_path, log_path, stack_path, workspace_path, ProgressEvent, ProvisionRequest,
    WizardState,
};
use super::super::{write_progress, ProgressMarker};
use log::{info, warn};
use std::io::Write;
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::desktop::sidecar::Sidecar;

const API_PRODUCT: &str = "http://127.0.0.1:8080/api/product";

fn emit(app: &tauri::AppHandle, state: &WizardState, event: ProgressEvent) {
    if let Ok(mut progress) = state.progress.lock() {
        if let Some(existing) = progress.iter_mut().find(|entry| entry.id == event.id) {
            *existing = event.clone();
        } else {
            progress.push(event.clone());
        }
    }
    if event.state == "failed" {
        if let Ok(mut failed) = state.failed.lock() {
            *failed = Some(event.message.clone());
        }
    }
    if let Err(e) = app.emit("wizard://progress", &event) {
        warn!("could not emit wizard progress: {e}");
    }
}

pub(super) fn progress(
    app: &tauri::AppHandle,
    state: &WizardState,
    id: &str,
    step: &str,
    message: &str,
    elapsed_ms: u64,
) {
    emit(
        app,
        state,
        ProgressEvent {
            id: id.to_string(),
            state: step.to_string(),
            message: message.to_string(),
            elapsed_ms,
        },
    );
}

/// Runs the provisioning child and forwards its progress. Returns the failure
/// reason, if any, without leaking environment or credentials.
pub(super) async fn run_components(
    app: &tauri::AppHandle,
    state: &WizardState,
    request: &ProvisionRequest,
) -> Result<(), String> {
    let binary = Sidecar::botserver_path();
    if !binary.exists() {
        return Err(format!(
            "The assistant binary is missing at {}. Reinstall the desktop app.",
            binary.display()
        ));
    }

    let resource_dir = app.path().resource_dir().ok();
    let installers = installers_path(resource_dir.as_deref());
    let stack = stack_path();
    let workspace = if request.workspace.path.trim().is_empty() {
        workspace_path()
    } else {
        std::path::PathBuf::from(request.workspace.path.trim())
    };
    std::fs::create_dir_all(&workspace)
        .map_err(|e| format!("could not create the workspace folder: {e}"))?;
    std::fs::create_dir_all(log_path()).map_err(|e| format!("could not create logs: {e}"))?;

    let mut command = tokio::process::Command::new(&binary);
    #[cfg(target_os = "windows")]
    command.env("PATH", Sidecar::prepare_windows_runtime(&installers)
        .map_err(|e| format!("could not prepare Windows runtime: {e}"))?);
    command
        .arg("--provision-stack")
        .current_dir(&stack)
        .env_remove("DATABASE_URL")
        .env_remove("PG_URL")
        .env("BOTSERVER_STACK_PATH", &stack)
        .env("BOTSERVER_INSTALLERS_PATH", &installers)
        .env("GB_WIZARD_WORKSPACE", &workspace)
        .env("GB_WIZARD_MODEL_MODE", &request.model.mode)
        // The provisioner only saves a non-local model when it knows the mode;
        // without this the free/hosted choice was silently dropped and the
        // assistant kept answering with the bundled local model.
        .env("GB_WIZARD_MODEL_MODE", &request.model.mode)
        .env("GB_WIZARD_LLM_URL", &request.model.endpoint)
        .env("GB_WIZARD_LLM_MODEL", &request.model.model)
        .env("GB_WIZARD_LLM_KEY", &request.model.api_key)
        .env("GB_WIZARD_LLM_PROVIDER", &request.model.provider)
        // Mirror start.sh: the bundled installers cover the small services;
        // the wizard may fetch the rest once unless GB_DESKTOP_OFFLINE=1.
        // This keeps a Windows first run from stopping at the first component
        // that is not in the payload.
        .env(
            "GB_DESKTOP_ALLOW_DOWNLOAD",
            if std::env::var("GB_DESKTOP_OFFLINE").as_deref() == Ok("1") {
                "0"
            } else {
                "1"
            },
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    info!(
        "wizard: provisioning stack at {} with {}",
        stack.display(),
        binary.display()
    );

    let mut child = command
        .spawn()
        .map_err(|e| format!("could not start the setup process: {e}"))?;

    if let Some(pid) = child.id() {
        *state.pid.lock().map_err(|_| "state lock poisoned")? = Some(pid);
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stderr_buffer = Arc::new(std::sync::Mutex::new(String::new()));
    let stderr_clone = Arc::clone(&stderr_buffer);
    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(mut buffer) = stderr_clone.lock() {
                    buffer.push_str(&line);
                    buffer.push('\n');
                    if buffer.len() > 8000 {
                        let keep = buffer.len() - 6000;
                        *buffer = buffer[keep..].to_string();
                    }
                }
            }
        });
    }

    let started = std::time::Instant::now();
    let mut completed: Vec<String> = Vec::new();
    let mut failure: Option<String> = None;

    if let Some(stdout) = stdout {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if state.cancelled.load(Ordering::SeqCst) {
                break;
            }
            let Some(payload) = line.strip_prefix("GBWIZARD:") else {
                if !line.trim().is_empty() {
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(log_path().join("wizard-provision.log"))
                    {
                        let _ = writeln!(file, "{line}");
                    }
                }
                continue;
            };
            let Ok(event) = serde_json::from_str::<ProgressEvent>(payload) else {
                continue;
            };
            if event.state == "ready" {
                completed.push(event.id.clone());
                let _ = write_progress(&ProgressMarker {
                    completed: completed.clone(),
                    step: 5,
                });
            }
            if event.state == "failed" {
                failure = Some(event.message.clone());
            }
            emit(app, state, event);
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("could not wait for the setup process: {e}"))?;
    if let Ok(mut guard) = state.pid.lock() {
        *guard = None;
    }

    if state.cancelled.load(Ordering::SeqCst) {
        return Err("Setup was cancelled. Partial data will be reused next time.".to_string());
    }
    if !status.success() {
        let details = stderr_buffer
            .lock()
            .map(|buffer| buffer.trim().to_string())
            .unwrap_or_default();
        let reason = failure.unwrap_or_else(|| {
            if details.is_empty() {
                "A setup component failed. Open the log folder for details.".to_string()
            } else {
                details
                    .lines()
                    .last()
                    .unwrap_or("A setup component failed.")
                    .to_string()
            }
        });
        return Err(reason);
    }
    if let Some(reason) = failure {
        return Err(reason);
    }

    info!(
        "wizard: components ready in {:.1}s",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

pub(super) async fn start_assistant(
    app: &tauri::AppHandle,
    state: &WizardState,
    sidecar: &Sidecar,
) -> Result<(), String> {
    progress(
        app,
        state,
        "botserver",
        "starting",
        "Starting the assistant service…",
        0,
    );
    let resource_dir = app.path().resource_dir().ok();
    if !sidecar.is_running() {
        sidecar
            .start(resource_dir.clone())
            .map_err(|e| format!("could not start the assistant: {e}"))?;
    }
    // A cold first boot runs migrations, seeds the vault and starts every
    // service; 120 s was too tight and the wizard declared the assistant dead
    // while it was still starting.
    if Sidecar::wait_for_api(API_PRODUCT, 300).await {
        // The identity provider's OAuth client is created by the assistant
        // during its first boot. Until that file exists the auth service cannot
        // initialise and the suite's sign-in answers 500. Wait for it, then
        // restart the sidecar once so the auth service starts with it.
        let directory_config = stack_path().join("conf/system/directory_config.json");
        let mut appeared = directory_config.exists();
        for _ in 0..150 {
            if appeared {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            appeared = directory_config.exists();
        }
        if appeared {
            log::info!("directory config is ready; restarting the assistant once");
            sidecar.stop();
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            if let Err(e) = sidecar.start(resource_dir.clone()) {
                log::warn!("could not restart the assistant: {e}");
            } else {
                let _ = Sidecar::wait_for_api(API_PRODUCT, 180).await;
            }
        }
        progress(
            app,
            state,
            "botserver",
            "ready",
            "Assistant service is ready",
            0,
        );
        return Ok(());
    }

    // A clean install can lose the first boot to a component that is still
    // finishing (the identity provider writes its config a few seconds after
    // being reported ready). One deliberate retry with a fresh process beats
    // failing the whole first run.
    log::warn!("assistant did not answer in 300s; restarting it once");
    progress(
        app,
        state,
        "botserver",
        "starting",
        "Starting the assistant service again…",
        0,
    );
    sidecar.stop();
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    if let Err(e) = sidecar.start(resource_dir) {
        let reason = format!("could not start the assistant: {e}");
        progress(app, state, "botserver", "failed", &reason, 0);
        return Err(reason);
    }
    if Sidecar::wait_for_api(API_PRODUCT, 180).await {
        progress(
            app,
            state,
            "botserver",
            "ready",
            "Assistant service is ready",
            0,
        );
        Ok(())
    } else {
        let reason = "The assistant did not answer in time. Open the log folder for details.";
        progress(app, state, "botserver", "failed", reason, 0);
        Err(reason.to_string())
    }
}
