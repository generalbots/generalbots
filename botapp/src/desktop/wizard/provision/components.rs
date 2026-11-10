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

use crate::desktop::i18n;
use crate::desktop::sidecar::Sidecar;

const API_PRODUCT: &str = "http://127.0.0.1:8080/api/product";

/// Cap for the forwarded stderr text, and how much of the tail is kept.
const STDERR_MAX_BYTES: usize = 8000;
const STDERR_KEEP_BYTES: usize = 6000;

/// Keeps the tail of `buffer` bounded, never splitting a UTF-8 character.
///
/// The provisioner can emit non-ASCII paths and messages; slicing at a raw
/// byte offset panics on a multi-byte boundary.
fn trim_stderr_tail(buffer: &mut String) {
    if buffer.len() <= STDERR_MAX_BYTES {
        return;
    }
    let mut cut = buffer.len() - STDERR_KEEP_BYTES;
    while cut < buffer.len() && !buffer.is_char_boundary(cut) {
        cut += 1;
    }
    *buffer = buffer[cut..].to_string();
}

/// Parses one line of the provisioner's stdout. Only `GBWIZARD:`-prefixed JSON
/// progress events are structured; everything else is plain log output.
fn parse_progress_line(line: &str) -> Option<ProgressEvent> {
    let payload = line.strip_prefix("GBWIZARD:")?;
    serde_json::from_str(payload).ok()
}

/// `components` ready is the provisioner's terminal event; a `failed` event
/// ends the run too.
fn is_terminal(event: &ProgressEvent) -> bool {
    event.state == "failed" || (event.id == "components" && event.state == "ready")
}

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
        // The provisioner only saves a non-local model when it knows the mode;
        // without this the free/hosted choice was silently dropped and the
        // assistant kept answering with the bundled local model.
        .env("GB_WIZARD_MODEL_MODE", &request.model.mode)
        .env("GB_WIZARD_LLM_URL", &request.model.endpoint)
        .env("GB_WIZARD_LLM_MODEL", &request.model.model)
        .env("GB_WIZARD_LLM_KEY", &request.model.api_key)
        .env("GB_WIZARD_LLM_PROVIDER", &request.model.provider)
        .env(
            "GB_WIZARD_SKIP_AGENTIC_WORK",
            if request.skip_all_agentic_work { "1" } else { "0" },
        )
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
                    trim_stderr_tail(&mut buffer);
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
            let Some(event) = parse_progress_line(&line) else {
                if !line.starts_with("GBWIZARD:") && !line.trim().is_empty() {
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
            // On Windows the services the provisioner starts with `start /B`
            // (vault, postgres, minio, qdrant, zitadel) inherit this stdout
            // pipe and keep it open after the provisioner exits, so waiting for
            // EOF blocks forever and the wizard stays at "Assistant service
            // pending" (7 of 8, 88%). The terminal event ends the read instead.
            let terminal = is_terminal(&event);
            emit(app, state, event);
            if terminal {
                break;
            }
        }
    }

    // The direct child exits right after its terminal event, but the inherited
    // pipes of the detached services may keep `wait` from resolving promptly.
    // Bound it: the components already reported their final state.
    let status = match tokio::time::timeout(std::time::Duration::from_secs(30), child.wait()).await {
        Ok(Ok(status)) => Some(status),
        Ok(Err(e)) => return Err(format!("could not wait for the setup process: {e}")),
        Err(_) => {
            warn!("wizard: provisioning child did not exit within 30s; continuing");
            None
        }
    };
    if let Ok(mut guard) = state.pid.lock() {
        *guard = None;
    }

    if state.cancelled.load(Ordering::SeqCst) {
        return Err(i18n::t("setup-cancelled"));
    }
    if let Some(status) = status {
        if !status.success() {
            let details = stderr_buffer
                .lock()
                .map(|buffer| buffer.trim().to_string())
                .unwrap_or_default();
            let reason = failure.unwrap_or_else(|| {
                if details.is_empty() {
                    i18n::t("setup-component-failed-hint")
                } else {
                    let fallback = i18n::t("setup-component-failed");
                    details
                        .lines()
                        .last()
                        .unwrap_or(fallback.as_str())
                        .to_string()
                }
            });
            return Err(reason);
        }
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
        &i18n::t("setup-starting-assistant"),
        0,
    );
    let resource_dir = app.path().resource_dir().ok();
    // #2023 — `Sidecar::start` is idempotent: it reuses a live child, attaches
    // to an assistant that already answers `/api/product`, and refuses to open a
    // second listener when port 8080 is taken. Calling it here can therefore
    // never duplicate the app-startup sidecar (the double boot this issue
    // reports). The only restart the wizard performs is the explicit one below.
    let directory_config = stack_path().join("conf/system/directory_config.json");
    // Capture whether the identity-provider config predates this start: if it
    // does, the running assistant booted with it and must be reused as is.
    let config_before_start = directory_config.exists();

    sidecar
        .start(resource_dir.clone())
        .map_err(|e| format!("could not start the assistant: {e}"))?;
    // A cold first boot runs migrations, seeds the vault and starts every
    // service; 120 s was too tight and the wizard declared the assistant dead
    // while it was still starting.
    if Sidecar::wait_for_api(API_PRODUCT, 300).await {
        // The identity provider's OAuth client is created by the assistant
        // during its first boot. Until that file exists the auth service cannot
        // initialise and the suite's sign-in answers 500. This is the single
        // intentional restart: it happens only on a first boot, only when this
        // run's assistant created the file, and the UI is told about it.
        let mut appeared = config_before_start;
        for _ in 0..150 {
            if appeared {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            appeared = directory_config.exists();
        }
        if appeared && !config_before_start {
            log::info!(
                "first boot created the directory config; restarting the assistant once so the auth service loads it"
            );
            progress(
                app,
                state,
                "botserver",
                "starting",
                &i18n::t("setup-finalizing-sign-in"),
                0,
            );
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
        &i18n::t("setup-restarting-assistant"),
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

#[cfg(test)]
mod tests {
    use super::{is_terminal, parse_progress_line, trim_stderr_tail, STDERR_MAX_BYTES};

    #[test]
    fn only_gbwizard_json_lines_are_progress_events() {
        let line = r#"GBWIZARD:{"id":"vault","state":"ready","message":"ok","elapsed_ms":12}"#;
        let event = parse_progress_line(line).expect("structured progress line");
        assert_eq!(event.id, "vault");
        assert_eq!(event.state, "ready");
        assert_eq!(event.elapsed_ms, 12);

        assert!(parse_progress_line("starting vault...").is_none());
        assert!(parse_progress_line("GBWIZARD:{not json}").is_none());
        assert!(parse_progress_line("GBWIZARD:").is_none());
        assert!(parse_progress_line("").is_none());
    }

    #[test]
    fn only_the_final_event_ends_the_run() {
        let event = |id: &str, state: &str| super::ProgressEvent {
            id: id.to_string(),
            state: state.to_string(),
            message: String::new(),
            elapsed_ms: 0,
        };
        assert!(is_terminal(&event("components", "ready")));
        assert!(is_terminal(&event("vault", "failed")));
        assert!(!is_terminal(&event("vault", "ready")));
        assert!(!is_terminal(&event("components", "starting")));
        assert!(!is_terminal(&event("botserver", "ready")));
    }

    #[test]
    fn the_stderr_tail_never_splits_a_character() {
        let mut short = String::from("ok\n");
        trim_stderr_tail(&mut short);
        assert_eq!(short, "ok\n");

        // A one-byte prefix plus 5000 two-byte characters: the raw byte cut
        // lands in the middle of a character, so it has to move forward.
        let mut buffer = format!("a{}", "é".repeat(5000));
        let before = buffer.len();
        assert!(before > STDERR_MAX_BYTES);
        trim_stderr_tail(&mut buffer);
        assert!(buffer.len() <= STDERR_MAX_BYTES);
        assert!(buffer.chars().all(|c| c == 'é'));
        assert!(!buffer.is_empty());
    }
}
