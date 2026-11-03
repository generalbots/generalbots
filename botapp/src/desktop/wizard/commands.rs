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

const API_BASE: &str = "http://127.0.0.1:8080";

async fn api_login(admin: &super::AdminDraft) -> Result<String, String> {
    if admin.email.trim().is_empty() || admin.password.is_empty() {
        return Err("The administrator account is not ready yet.".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;
    let response = client
        .post(format!("{API_BASE}/api/auth/login"))
        .json(&serde_json::json!({
            "email": admin.email.trim(),
            "password": admin.password,
        }))
        .send()
        .await
        .map_err(|e| format!("Could not reach the assistant: {e}"))?;
    let status = response.status();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("The assistant returned an unreadable answer: {e}"))?;
    if !status.is_success() {
        let message = body
            .get("error")
            .or_else(|| body.get("detail"))
            .and_then(|value| value.as_str())
            .unwrap_or("The administrator sign-in was refused.");
        return Err(message.to_string());
    }
    body.get("access_token")
        .or_else(|| body.get("token"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "The assistant returned no session token.".to_string())
}

async fn api_request(
    method: reqwest::Method,
    path: &str,
    admin: &super::AdminDraft,
    body: Option<serde_json::Value>,
    timeout_secs: u64,
) -> Result<serde_json::Value, String> {
    let token = api_login(admin).await?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;
    let mut request = client
        .request(method, format!("{API_BASE}{path}"))
        .bearer_auth(token);
    if let Some(payload) = body {
        request = request.json(&payload);
    }
    let response = request
        .send()
        .await
        .map_err(|e| format!("Could not reach the assistant: {e}"))?;
    let status = response.status();
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("The assistant returned an unreadable answer: {e}"))?;
    if !status.is_success() {
        let message = payload
            .get("error")
            .and_then(|value| value.as_str())
            .unwrap_or("The assistant refused the request.");
        return Err(message.to_string());
    }
    Ok(payload)
}

#[tauri::command]
pub fn wizard_save_admin(admin: super::AdminDraft) -> Result<(), String> {
    super::write_admin_credentials_draft(&admin)
}

#[tauri::command]
pub async fn wizard_ai_accounts(
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    api_request(reqwest::Method::GET, "/api/ai-accounts", &admin, None, 15).await
}

#[tauri::command]
pub async fn wizard_ai_connect(
    provider: String,
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    // Provisioning the VM and installing the Claude CLI can take a minute.
    api_request(
        reqwest::Method::POST,
        &format!("/api/ai-accounts/{provider}/connect"),
        &admin,
        Some(serde_json::json!({})),
        180,
    )
    .await
}

#[tauri::command]
pub async fn wizard_ai_status(
    provider: String,
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    api_request(
        reqwest::Method::POST,
        &format!("/api/ai-accounts/{provider}/status"),
        &admin,
        Some(serde_json::json!({})),
        60,
    )
    .await
}

#[tauri::command]
pub async fn wizard_ai_disconnect(
    provider: String,
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    api_request(
        reqwest::Method::POST,
        &format!("/api/ai-accounts/{provider}/disconnect"),
        &admin,
        Some(serde_json::json!({})),
        30,
    )
    .await
}

#[tauri::command]
pub async fn wizard_ai_code(
    code: String,
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    api_request(
        reqwest::Method::POST,
        "/api/ai-accounts/claude-code/code",
        &admin,
        Some(serde_json::json!({ "code": code })),
        90,
    )
    .await
}
