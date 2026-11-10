//! Stack provisioning driven by the wizard.
//!
//! The wizard does not know how to start PostgreSQL, Vault or MinIO. It spawns
//! `botserver --provision-stack`, which owns the component order and the
//! offline/checksum rules, and forwards each `GBWIZARD:` progress line to the
//! window. When the components are ready the sidecar starts and the wizard waits
//! for `/api/product` before handing over.

mod components;
mod draft;
mod terminate;

use super::{
    progress_path, FinishRequest, ProgressMarker, ProvisionRequest, ProvisionStatus, WizardState,
};
use crate::desktop::i18n;
use crate::desktop::sidecar::Sidecar;
use components::{progress, run_components, start_assistant};
use draft::{apply_integrations, save_draft};
use log::error;
use std::sync::atomic::Ordering;

pub async fn provision(
    app: tauri::AppHandle,
    state: tauri::State<'_, WizardState>,
    sidecar: tauri::State<'_, Sidecar>,
    request: ProvisionRequest,
) -> Result<(), String> {
    if state.provisioning.swap(true, Ordering::SeqCst) {
        return Err(i18n::t("setup-already-running"));
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
    let _ = super::write_progress(&ProgressMarker {
        completed: Vec::new(),
        step: 6, // network mode step — resume here if provisioning is interrupted
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

pub fn status(state: &WizardState) -> ProvisionStatus {
    let components = match state.progress.lock() {
        Ok(progress) => progress.clone(),
        Err(_) => Vec::new(),
    };
    let ready = components.iter().any(|event| event.id == "__done__");
    let failed = match state.failed.lock() {
        Ok(failed) => failed.clone(),
        Err(_) => None,
    };
    ProvisionStatus {
        running: state.provisioning.load(Ordering::SeqCst),
        ready,
        failed,
        components,
    }
}

pub fn cancel(state: &WizardState) -> Result<(), String> {
    state.cancelled.store(true, Ordering::SeqCst);
    let pid = *state.pid.lock().map_err(|_| "state lock poisoned")?;
    if let Some(pid) = pid {
        terminate::terminate(pid);
    }
    Ok(())
}
