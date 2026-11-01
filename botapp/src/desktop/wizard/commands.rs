//! Tauri commands exposed to the static wizard page.

use super::provision;
use super::system::{self, ModelTestResult, WorkspaceCheck};
use super::{FinishRequest, ModelRequest, ProvisionRequest, WizardContext, WizardState};
use log::warn;

#[tauri::command]
pub fn wizard_context() -> WizardContext {
    super::context()
}

#[tauri::command]
pub fn wizard_save_draft(draft: serde_json::Value) -> Result<(), String> {
    let stack = super::stack_path();
    std::fs::create_dir_all(&stack).map_err(|e| format!("create stack folder: {e}"))?;
    let raw = serde_json::to_string_pretty(&draft)
        .map_err(|e| format!("serialize draft: {e}"))?;
    std::fs::write(super::draft_path(), raw).map_err(|e| format!("write draft: {e}"))
}

#[tauri::command]
pub fn wizard_load_draft() -> Result<Option<serde_json::Value>, String> {
    let raw = match std::fs::read_to_string(super::draft_path()) {
        Ok(raw) => raw,
        Err(_) => return Ok(None),
    };
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(value) => Ok(Some(value)),
        Err(e) => {
            warn!("wizard: ignoring an unreadable draft: {e}");
            Ok(None)
        }
    }
}

#[tauri::command]
pub fn wizard_check_workspace(path: String) -> WorkspaceCheck {
    system::check_workspace(&path)
}

#[tauri::command]
pub fn wizard_pick_folder(app: tauri::AppHandle) -> Option<String> {
    system::pick_folder(&app)
}

#[tauri::command]
pub async fn wizard_test_model(request: ModelRequest) -> ModelTestResult {
    system::test_model(request).await
}

#[tauri::command]
pub async fn wizard_provision(
    app: tauri::AppHandle,
    state: tauri::State<'_, WizardState>,
    sidecar: tauri::State<'_, super::super::sidecar::Sidecar>,
    request: ProvisionRequest,
) -> Result<(), String> {
    provision::provision(app, state, sidecar, request).await
}

#[tauri::command]
pub fn wizard_cancel_provision(state: tauri::State<'_, WizardState>) -> Result<(), String> {
    provision::cancel(&state)
}

#[tauri::command]
pub fn wizard_provision_status(state: tauri::State<'_, WizardState>) -> super::ProvisionStatus {
    provision::status(&state)
}

#[tauri::command]
pub fn wizard_reset_stack(confirmation: String) -> Result<(), String> {
    system::reset_stack(&confirmation)
}

#[tauri::command]
pub fn wizard_open_folder(app: tauri::AppHandle, kind: String) -> Result<(), String> {
    system::open_folder(&app, &kind)
}

#[tauri::command]
pub fn wizard_open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    system::open_url(&app, &url)
}

#[tauri::command]
pub async fn wizard_finish(
    app: tauri::AppHandle,
    request: FinishRequest,
) -> Result<(), String> {
    provision::finish(app, request).await
}
