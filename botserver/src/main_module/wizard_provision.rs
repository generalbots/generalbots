//! `botserver --provision-stack`: the component sequence the first-run wizard
//! drives (issues #1586-#1591).
//!
//! Each component is installed (offline, from the bundled installers), started
//! and then awaited in dependency order. Progress is emitted as
//! `GBWIZARD:{json}` lines on stdout so the desktop shell can show a truthful
//! per-component state without parsing logs. A downstream component never
//! starts against a half-initialised dependency.

use crate::core::package_manager::{InstallMode, PackageManager};
use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize)]
struct WizardEvent {
    id: String,
    state: String,
    message: String,
    elapsed_ms: u64,
}

struct Component {
    id: &'static str,
    label: &'static str,
    timeout_secs: u64,
    optional: bool,
    /// When false the wizard only materialises the files and leaves the
    /// service to start on first use (heavy dependencies, no blocking).
    start: bool,
    /// Optional components that cannot exist on this host (no published
    /// pythonbox for Linux aarch64 / Windows arm64). The wizard reports the
    /// reason and never attempts a download that would fail or hang.
    unavailable: Option<String>,
}

fn emit(event: WizardEvent) {
    match serde_json::to_string(&event) {
        Ok(payload) => println!("GBWIZARD:{payload}"),
        Err(e) => eprintln!("provision: could not serialise progress: {e}"),
    }
}

fn progress(id: &str, state: &str, message: &str, started: Instant) {
    emit(WizardEvent {
        id: id.to_string(),
        state: state.to_string(),
        message: message.to_string(),
        elapsed_ms: started.elapsed().as_millis() as u64,
    });
}

mod component_plan;
use component_plan::component_plan;

async fn provision_component(
    pm: &PackageManager,
    component: &Component,
    started: Instant,
) -> Result<(), String> {
    progress(
        component.id,
        "pending",
        &format!("Preparing {}", component.label.to_lowercase()),
        started,
    );

    if let Some(reason) = &component.unavailable {
        // Optional component absent from this platform: never download, fail or
        // wait for it. The wizard shows the exact media gap instead.
        progress(component.id, "ready", reason, started);
        return Ok(());
    }

    let already_installed = pm.is_installed(component.id);
    if !already_installed {
        progress(
            component.id,
            "starting",
            &format!(
                "Installing {} from the bundled files",
                component.label.to_lowercase()
            ),
            started,
        );
        if let Err(e) = pm.install(component.id).await {
            let reason = format!("{} could not be installed: {e}", component.label);
            // An optional component must never block the wizard: the desktop
            // edition offers external providers, so a missing local runtime is
            // a degraded feature, not a broken install.
            if component.optional {
                progress(
                    component.id,
                    "ready",
                    "Skipped (optional) — install failed",
                    started,
                );
                return Ok(());
            }
            progress(component.id, "failed", &reason, started);
            return Err(reason);
        }
    }

    // A stack reset removes conf/ and data/ but leaves bin/ intact, so
    // is_installed() returns true while the config files are gone.  Re-run the
    // pre/post-install commands for every already-installed component to
    // regenerate directories and configuration that the binary alone doesn't
    // carry.
    #[cfg(windows)]
    if already_installed && !pm.is_running(component.id) {
        progress(component.id, "starting", &format!("Preparing {} configuration", component.label.to_lowercase()), started);
        if let Err(e) = pm.prepare_installed_windows_component(component.id) {
            log::warn!("Pre-install commands for {} failed: {e}", component.id);
        }
    }

    if let Some(program) = pm.executable_path(component.id) {
        if !program.exists() {
            let message = format!(
                "{} is missing its program file ({}).",
                component.label,
                program.display()
            );
            if component.optional {
                progress(
                    component.id,
                    "ready",
                    "Skipped (optional) — program file missing",
                    started,
                );
                return Ok(());
            }
            progress(component.id, "failed", &message, started);
            return Err(message);
        }
    }

    if !component.start {
        progress(
            component.id,
            "ready",
            "Bundled with offline wheels — installs on first use",
            started,
        );
        return Ok(());
    }

    let mut spawned_child = None;
    if !pm.is_running(component.id) {
        if std::env::var("GB_WIZARD_DEBUG").is_ok() {
            eprintln!(
                "provision: {} reports not running (stack: {}, vault_addr: {:?})",
                component.id,
                std::env::var("BOTSERVER_STACK_PATH").unwrap_or_default(),
                std::env::var("VAULT_ADDR").ok()
            );
        }
        progress(
            component.id,
            "starting",
            &format!("Starting {}", component.label.to_lowercase()),
            started,
        );
        match pm.start(component.id) {
          Ok(child) => spawned_child = Some(child),
          Err(e) => {
            let message = e.to_string();
            let already_running = message.contains("Already running")
                || message.contains("already running")
                || message.contains("be running");
            if !already_running {
                let reason = format!("{} could not be started: {e}", component.label);
                progress(component.id, "failed", &reason, started);
                return Err(reason);
            }
          }
        }
    }

    let deadline = Instant::now() + Duration::from_secs(component.timeout_secs);
    while Instant::now() < deadline {
        if pm.is_running(component.id) {
            progress(component.id, "ready", "Ready", started);
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    if component.id == "directory" {
        if let Some(mut child) = spawned_child {
            if matches!(child.try_wait(), Ok(None)) {
                match child.kill() {
                    Ok(()) => { let _ = child.wait(); }
                    Err(e) => log::warn!("could not stop timed-out identity provider: {e}"),
                }
            }
        }
    }

    let reason = format!(
        "{} did not become ready within {} seconds. Open the log folder for details.",
        component.label, component.timeout_secs
    );
    if component.optional {
        // An optional component never blocks the rest of the wizard.
        progress(component.id, "ready", "Skipped (optional)", started);
        return Ok(());
    }
    progress(component.id, "failed", &reason, started);
    Err(reason)
}

/// Saves the model the user chose on step 2 so the assistant actually uses it.
/// Local mode is seeded by the installer; free/own-key/hosted write the global
/// LLM secret (a desktop install is single-user).
async fn apply_llm_choice() {
    let mode = std::env::var("GB_WIZARD_MODEL_MODE").unwrap_or_default();
    let remote_model = !std::env::var("GB_WIZARD_LLM_URL")
        .unwrap_or_default()
        .trim()
        .is_empty()
        && !std::env::var("GB_WIZARD_LLM_MODEL")
            .unwrap_or_default()
            .trim()
            .is_empty();
    let wants_local = if mode.trim().is_empty() {
        !remote_model
    } else {
        mode.eq_ignore_ascii_case("local")
    };
    if wants_local {
        return;
    }
    let url = std::env::var("GB_WIZARD_LLM_URL").unwrap_or_default();
    let model = std::env::var("GB_WIZARD_LLM_MODEL").unwrap_or_default();
    if url.trim().is_empty() || model.trim().is_empty() {
        return;
    }
    if let Ok(stack) = std::env::var("BOTSERVER_STACK_PATH") {
        dotenvy::from_path(std::path::PathBuf::from(stack).join(".env")).ok();
    } else {
        dotenvy::dotenv().ok();
    }

    // On a first run the stack `.env` (and therefore VAULT_TOKEN) is written by
    // the assistant's first boot, which happens after provisioning. The Vault
    // root token is already on disk from the install, so use it here instead of
    // silently dropping the model choice.
    let stack = std::path::PathBuf::from(std::env::var("BOTSERVER_STACK_PATH").unwrap_or_default());
    if std::env::var("VAULT_TOKEN")
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        let init = stack.join("conf/vault/init.json");
        if let Ok(raw) = std::fs::read_to_string(&init) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(token) = json.get("root_token").and_then(|t| t.as_str()) {
                    std::env::set_var("VAULT_TOKEN", token);
                }
            }
        }
    }
    if std::env::var("VAULT_ADDR")
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        std::env::set_var("VAULT_ADDR", if cfg!(target_os = "windows") {
            "http://127.0.0.1:8200"
        } else {
            "https://127.0.0.1:8200"
        });
    }
    if std::env::var("VAULT_CACERT").is_err() {
        let ca = stack.join("conf/system/certificates/ca/ca.crt");
        if ca.exists() {
            std::env::set_var("VAULT_CACERT", ca);
        }
    }
    // Build the manager from the environment we just loaded instead of the
    // process-wide singleton: provisioning calls the secret store before the
    // stack `.env` exists, so the singleton may have been cached as disabled.
    let manager = match botcoresecrets::SecretsManager::from_env() {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("provision: could not initialise secrets for the model choice: {e}");
            return;
        }
    };
    if !manager.is_enabled() {
        // Vault is not reachable from the provisioner on a first run (its
        // credentials land in the stack .env during the assistant's first
        // boot). Persist the choice on disk; the boot applies it before the
        // LLM provider is built, so the very first message already uses it.
        let stack =
            std::path::PathBuf::from(std::env::var("BOTSERVER_STACK_PATH").unwrap_or_default());
        let path = stack.join("conf/system/model-choice.json");
        let payload = serde_json::json!({ "url": url, "model": model, "api_key": std::env::var("GB_WIZARD_LLM_KEY").unwrap_or_default() });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&path, payload.to_string()) {
            Ok(()) => eprintln!("provision: model choice staged at {}", path.display()),
            Err(e) => eprintln!("provision: could not stage the model choice: {e}"),
        }
        return;
    }
    let mut data = std::collections::HashMap::new();
    data.insert("url".to_string(), url);
    data.insert("model".to_string(), model);
    if let Ok(key) = std::env::var("GB_WIZARD_LLM_KEY") {
        if !key.trim().is_empty() {
            data.insert("openai_key".to_string(), key);
        }
    }
    match manager.put_secret("gbo/llm", data).await {
        Ok(()) => println!(
            "GBWIZARD:{}",
            serde_json::json!({
                "id": "llm-config",
                "state": "ready",
                "message": "Model choice saved",
                "elapsed_ms": 0,
            })
        ),
        Err(e) => eprintln!("provision: could not save the model choice: {e}"),
    }
}

pub async fn run() -> std::io::Result<()> {
    let started = Instant::now();
    // The vault check command reads `${VAULT_ADDR}`. On a first run the wizard
    // has not written `.env` yet, so point the local component checks at the
    // stack's own vault before probing it.
    if std::env::var("VAULT_ADDR")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        std::env::set_var(
            "VAULT_ADDR",
            if cfg!(target_os = "windows") {
                "http://127.0.0.1:8200"
            } else {
                "https://127.0.0.1:8200"
            },
        );
    }
    let tenant = std::env::var("GB_TENANT")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let pm = PackageManager::new(InstallMode::Local, tenant)
        .map_err(|e| std::io::Error::other(format!("package manager: {e}")))?;

    for component in component_plan() {
        if let Err(reason) = provision_component(&pm, &component, started).await {
            return Err(std::io::Error::other(reason));
        }
    }

    // Vault finishes initialization a few seconds after its process reports
    // ready; wait for the init file so the model choice can actually be saved.
    for _ in 0..120 {
        let stack =
            std::path::PathBuf::from(std::env::var("BOTSERVER_STACK_PATH").unwrap_or_default());
        if stack.join("conf/vault/init.json").exists() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    apply_llm_choice().await;

    println!(
        "GBWIZARD:{}",
        serde_json::json!({
            "id": "components",
            "state": "ready",
            "message": "All components are ready",
            "elapsed_ms": started.elapsed().as_millis() as u64,
        })
    );
    Ok(())
}
