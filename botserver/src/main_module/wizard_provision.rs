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
}

fn emit(event: WizardEvent) {
    match serde_json::to_string(&event) {
        Ok(payload) => println!("GBWIZARD:{payload}"),
        Err(e) => eprintln!("provision: could not serialise progress: {e}"),
    }
}

fn progress(
    id: &str,
    state: &str,
    message: &str,
    started: Instant,
) {
    emit(WizardEvent {
        id: id.to_string(),
        state: state.to_string(),
        message: message.to_string(),
        elapsed_ms: started.elapsed().as_millis() as u64,
    });
}

fn component_plan() -> Vec<Component> {
    let model_mode = std::env::var("GB_WIZARD_MODEL_MODE").unwrap_or_else(|_| "local".to_string());
    let mut plan = vec![
        Component { id: "vault", label: "Local vault", timeout_secs: 90, optional: false },
        Component { id: "tables", label: "Database", timeout_secs: 90, optional: false },
        Component { id: "cache", label: "Cache", timeout_secs: 45, optional: false },
        Component { id: "drive", label: "Object store", timeout_secs: 60, optional: false },
        Component { id: "vector_db", label: "Vector store", timeout_secs: 60, optional: false },
        Component { id: "directory", label: "Identity provider", timeout_secs: 90, optional: false },
    ];
    if model_mode == "local" {
        plan.push(Component {
            id: "llm",
            label: "Local model runtime",
            timeout_secs: 120,
            optional: true,
        });
    }
    plan
}

/// Returns the first failure reason without including credentials or paths that
/// the wizard cannot display.
async fn provision_component(
    pm: &PackageManager,
    component: &Component,
    started: Instant,
) -> Result<(), String> {
    progress(component.id, "pending", &format!("Preparing {}", component.label.to_lowercase()), started);

    if !pm.is_installed(component.id) {
        progress(
            component.id,
            "starting",
            &format!("Installing {} from the bundled files", component.label.to_lowercase()),
            started,
        );
        if let Err(e) = pm.install(component.id).await {
            let reason = format!("{} could not be installed: {e}", component.label);
            progress(component.id, "failed", &reason, started);
            return Err(reason);
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
        if let Err(e) = pm.start(component.id) {
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

    for _ in 0..component.timeout_secs {
        if pm.is_running(component.id) {
            progress(component.id, "ready", "Ready", started);
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
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
    if mode.is_empty() || mode == "local" {
        return;
    }
    let url = std::env::var("GB_WIZARD_LLM_URL").unwrap_or_default();
    let model = std::env::var("GB_WIZARD_LLM_MODEL").unwrap_or_default();
    if url.trim().is_empty() || model.trim().is_empty() {
        return;
    }
    dotenvy::dotenv().ok();
    let manager = match botcoresecrets::SecretsManager::get() {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("provision: could not initialise secrets for the model choice: {e}");
            return;
        }
    };
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
        std::env::set_var("VAULT_ADDR", "https://127.0.0.1:8200");
    }
    let tenant = std::env::var("GB_TENANT").ok().filter(|value| !value.trim().is_empty());
    let pm = PackageManager::new(InstallMode::Local, tenant)
        .map_err(|e| std::io::Error::other(format!("package manager: {e}")))?;

    for component in component_plan() {
        if let Err(reason) = provision_component(&pm, &component, started).await {
            return Err(std::io::Error::other(reason));
        }
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
