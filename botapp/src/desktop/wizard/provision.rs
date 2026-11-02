//! Stack provisioning driven by the wizard.
//!
//! The wizard does not know how to start PostgreSQL, Vault or MinIO. It spawns
//! `botserver --provision-stack`, which owns the component order and the
//! offline/checksum rules, and forwards each `GBWIZARD:` progress line to the
//! window. When the components are ready the sidecar starts and the wizard waits
//! for `/api/product` before handing over.

use super::{
    installers_path, log_path, progress_path, stack_path, workspace_path, FinishRequest,
    ProgressEvent, ProvisionRequest, WizardState,
};
use super::write_progress;
use log::{error, info, warn};
use std::io::Write;
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};

use super::super::sidecar::Sidecar;

const API_PRODUCT: &str = "http://127.0.0.1:8080/api/product";
const API_BOTS: &str = "http://127.0.0.1:8080/api/bots/list";

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

fn progress(
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

fn save_draft(request: &ProvisionRequest) -> std::io::Result<()> {
    let services: std::collections::HashMap<&str, serde_json::Value> = request
        .services
        .iter()
        .filter(|(_, service)| !service.client_id.trim().is_empty())
        .map(|(provider, service)| {
            (
                provider.as_str(),
                serde_json::json!({ "client_id": service.client_id, "client_secret": "" }),
            )
        })
        .collect();
    let draft = serde_json::json!({
        "model": {
            "mode": request.model.mode,
            "provider": request.model.provider,
            "endpoint": request.model.endpoint,
            "model": request.model.model,
        },
        "workspace": request.workspace,
        "telemetry": request.telemetry,
        "display_name": request.display_name,
        "services": services,
    });
    let raw = serde_json::to_string_pretty(&draft)
        .map_err(|e| std::io::Error::other(format!("serialize draft: {e}")))?;
    let stack = stack_path();
    std::fs::create_dir_all(&stack)?;
    std::fs::write(super::draft_path(), raw)
}

/// Runs the provisioning child and forwards its progress. Returns the failure
/// reason, if any, without leaking environment or credentials.
async fn run_components(
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
    command
        .arg("--provision-stack")
        .current_dir(&stack)
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
        .env("GB_WIZARD_TELEMETRY", if request.telemetry { "1" } else { "0" })
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
                let _ = write_progress(&super::ProgressMarker {
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
                details.lines().last().unwrap_or("A setup component failed.").to_string()
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

async fn start_assistant(
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
    if Sidecar::wait_for_api(API_PRODUCT, 120).await {
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
    log::warn!("assistant did not answer in 120s; restarting it once");
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

/// Stores the OAuth client ids/secrets the user entered, through the running
/// API, once. Secrets never touch a config file and are never logged.
async fn apply_integrations(request: &ProvisionRequest) {
    if request.services.is_empty() {
        return;
    }
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            warn!("wizard: could not create the integration client: {e}");
            return;
        }
    };
    let bot_id = match client.get(API_BOTS).send().await {
        Ok(response) if response.status().is_success() => {
            let bots: serde_json::Value = response.json().await.unwrap_or_default();
            bots.as_array()
                .and_then(|list| list.first())
                .and_then(|bot| bot.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| id.to_string())
        }
        Ok(response) => {
            warn!(
                "wizard: could not list bots ({}) to apply integrations",
                response.status()
            );
            None
        }
        Err(e) => {
            warn!("wizard: could not list bots to apply integrations: {e}");
            None
        }
    };
    let Some(bot_id) = bot_id else {
        return;
    };
    for (provider, service) in &request.services {
        if service.client_id.trim().is_empty() {
            continue;
        }
        let url = format!(
            "http://127.0.0.1:8080/api/bots/{bot_id}/integrations/oauth/{provider}/client"
        );
        let body = serde_json::json!({
            "client_id": service.client_id,
            "client_secret": if service.client_secret.is_empty() {
                None
            } else {
                Some(&service.client_secret)
            },
        });
        match client.post(&url).json(&body).send().await {
            Ok(response) if response.status().is_success() => {
                info!("wizard: stored the OAuth client for {provider}");
            }
            Ok(response) => {
                warn!(
                    "wizard: the API rejected the OAuth client for {provider} ({})",
                    response.status()
                );
            }
            Err(e) => {
                warn!("wizard: could not store the OAuth client for {provider}: {e}");
            }
        }
    }
}

pub async fn provision(
    app: tauri::AppHandle,
    state: tauri::State<'_, WizardState>,
    sidecar: tauri::State<'_, Sidecar>,
    request: ProvisionRequest,
) -> Result<(), String> {
    if state.provisioning.swap(true, Ordering::SeqCst) {
        return Err("Setup is already running.".to_string());
    }
    state.cancelled.store(false, Ordering::SeqCst);
    if let Ok(mut progress) = state.progress.lock() {
        progress.clear();
    }
    if let Ok(mut failed) = state.failed.lock() {
        *failed = None;
    }
    let result = provision_inner(&app, &state, &sidecar, request).await;
    state.provisioning.store(false, Ordering::SeqCst);
    state.cancelled.store(false, Ordering::SeqCst);
    if let Err(ref e) = result {
        error!("wizard: setup failed: {e}");
    }
    result
}

async fn provision_inner(
    app: &tauri::AppHandle,
    state: &WizardState,
    sidecar: &Sidecar,
    request: ProvisionRequest,
) -> Result<(), String> {
    save_draft(&request).map_err(|e| format!("could not save the setup choices: {e}"))?;
    let _ = write_progress(&super::ProgressMarker {
        completed: Vec::new(),
        step: 5,
    });

    run_components(app, state, &request).await?;
    start_assistant(app, state, sidecar).await?;
    apply_integrations(&request).await;

    progress(app, state, "__done__", "ready", "Stack is ready", 0);
    let _ = std::fs::remove_file(progress_path());
    Ok(())
}

pub async fn finish(app: tauri::AppHandle, request: FinishRequest) -> Result<(), String> {
    super::handoff(&app, &request)
}

pub fn status(state: &WizardState) -> super::ProvisionStatus {
    let components = match state.progress.lock() {
        Ok(progress) => progress.clone(),
        Err(_) => Vec::new(),
    };
    let ready = components.iter().any(|event| event.id == "__done__");
    let failed = match state.failed.lock() {
        Ok(failed) => failed.clone(),
        Err(_) => None,
    };
    super::ProvisionStatus {
        running: state.provisioning.load(Ordering::SeqCst),
        ready,
        failed,
        components,
    }
}

pub fn cancel(state: &WizardState) -> Result<(), String> {    state.cancelled.store(true, Ordering::SeqCst);
    let pid = *state.pid.lock().map_err(|_| "state lock poisoned")?;
    if let Some(pid) = pid {
        terminate(pid);
    }
    Ok(())
}

#[cfg(unix)]
fn terminate(pid: u32) {
    // SAFETY: sending SIGTERM to a known child pid; failure is reported by the
    // caller through the process exit status.
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
}

#[cfg(windows)]
fn terminate(pid: u32) {
    let _ = botlib::security::command_guard::SafeCommand::new("taskkill")
        .and_then(|command| command.args(&["/PID", &pid.to_string(), "/T", "/F"]))
        .and_then(|command| command.execute());
}
