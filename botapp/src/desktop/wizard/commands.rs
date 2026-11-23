//! Tauri commands exposed to the static wizard page.

use super::provision;
use super::system::{self, ModelTestResult, WorkspaceCheck};
use super::{FinishRequest, ModelRequest, ProvisionRequest, WizardContext, WizardState};
use crate::desktop::drive::confirmation;
use log::warn;
use std::path::Path;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

/// #2689 — the reset confirmation must be the opaque receipt minted by the
/// native dialog, never a guessable folder name such as `stack`. Pure so the
/// rejection of a plain string is testable without a dialog or a grant store.
pub(crate) fn is_reset_token(value: &str) -> bool {
    value.len() == 24 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod reset_confirmation_tests {
    use super::is_reset_token;

    #[test]
    fn a_plain_folder_name_is_not_a_confirmation() {
        for bad in [
            "",
            "stack",
            " Stack ",
            "0123456789abcdef0123456z",
            "0123456789abcdef01234567a",
        ] {
            assert!(!is_reset_token(bad), "accepted plain confirmation {bad:?}");
        }
    }

    #[test]
    fn only_minted_receipts_have_the_confirmation_shape() {
        assert!(is_reset_token("0123456789abcdef01234567"));
        assert!(is_reset_token("ffffffffffffffffffffffff"));
    }
}

#[tauri::command]
pub fn wizard_context() -> WizardContext {
    super::context()
}

#[tauri::command]
pub fn wizard_save_draft(draft: serde_json::Value) -> Result<(), String> {
    let stack = super::stack_path();
    std::fs::create_dir_all(&stack).map_err(|e| format!("create stack folder: {e}"))?;
    let raw = serde_json::to_string_pretty(&draft).map_err(|e| format!("serialize draft: {e}"))?;
    let path = super::draft_path();
    std::fs::write(&path, raw).map_err(|e| format!("write draft: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
    Ok(())
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
pub async fn wizard_host_check(app: tauri::AppHandle) -> super::HostCheckResult {
    super::hostcheck::check(&app).await
}

/// Media capabilities for the finish screen. The BotModels service is the
/// source of truth when it is running; otherwise this probes the bundled
/// wheelhouse and the host libraries so missing features are named up front.
#[tauri::command]
pub async fn wizard_media_capabilities() -> serde_json::Value {
    super::media::capabilities().await
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

/// #2689 — the reset used to accept the guessable string `stack` from any
/// script on the sidecar origin. The user must now approve the exact target in
/// a native dialog first; the typed name alone is not a capability.
#[tauri::command]
pub fn request_reset_stack_confirmation(
    app: tauri::AppHandle,
    window: tauri::Window,
    state: tauri::State<'_, WizardState>,
) -> Result<String, String> {
    // #2692 — only the trusted local wizard window may mint a reset grant.
    let label = confirmation::trusted_label(&window)?;
    if state.provisioning.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(crate::desktop::i18n::t("setup-cancel-before-reset"));
    }
    let stack = super::stack_path();
    if stack.file_name().is_none() || stack.parent().is_none() {
        return Err("Refusing to reset an unsafe stack path.".to_string());
    }
    let target = stack.to_string_lossy().to_string();
    let approved = app
        .dialog()
        .message(format!("Permanently delete all local state in {target}?"))
        .title("Confirm stack reset")
        .buttons(MessageDialogButtons::YesNo)
        .blocking_show();
    if !approved {
        return Err("Confirmation was cancelled".to_string());
    }
    confirmation::mint(confirmation::Operation::ResetStack, &target, &label)
}

#[tauri::command]
pub fn wizard_reset_stack(
    window: tauri::Window,
    confirmation: String,
    state: tauri::State<'_, WizardState>,
) -> Result<(), String> {
    // #2692 — verify the caller again before consuming; the sidecar window is
    // denied even if a capability grants this command.
    let label = confirmation::trusted_label(&window)?;
    if state.provisioning.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(crate::desktop::i18n::t("setup-cancel-before-reset"));
    }
    // #2689 — a plain folder name such as `stack` is not a capability; only
    // the opaque receipt minted after the native dialog is accepted here.
    if !is_reset_token(&confirmation) {
        return Err("Reset requires the confirmation granted in the native dialog.".to_string());
    }
    let stack = super::stack_path();
    let target = stack.to_string_lossy().to_string();
    // #2689 — the confirmed target must be the exact configured stack once
    // symlinks are resolved; a guessable name never stands in for the path.
    if !system::reset_target_is_exact(&stack, Path::new(&target)) {
        return Err("Refusing to reset an unverified stack path.".to_string());
    }
    confirmation::consume_for(confirmation::Operation::ResetStack, &target, &label)?;
    system::reset_stack()
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
pub async fn wizard_finish(app: tauri::AppHandle, request: FinishRequest) -> Result<(), String> {
    provision::finish(app, request).await
}

fn api_base() -> String {
    let port = crate::desktop::sidecar::probe::sidecar_port();
    format!("http://127.0.0.1:{port}")
}

async fn api_login(admin: &super::AdminDraft) -> Result<String, String> {
    if admin.email.trim().is_empty() || admin.password.is_empty() {
        return Err("The administrator account is not ready yet.".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;
    let response = client
        .post(format!("{}/api/auth/login", api_base()))
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
        .request(method, format!("{}{path}", api_base()))
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
pub async fn wizard_ai_accounts(admin: super::AdminDraft) -> Result<serde_json::Value, String> {
    api_request(reqwest::Method::GET, "/api/ai-accounts", &admin, None, 15).await
}

#[tauri::command]
pub async fn wizard_ai_connect(
    provider: String,
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    // Provisioning the VM and installing the Claude CLI can take a few
    // minutes on a cold cache; the API installs on demand before replying.
    api_request(
        reqwest::Method::POST,
        &format!("/api/ai-accounts/{provider}/connect"),
        &admin,
        Some(serde_json::json!({})),
        600,
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
    provider: String,
    code: String,
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    api_request(
        reqwest::Method::POST,
        &format!("/api/ai-accounts/{provider}/code"),
        &admin,
        Some(serde_json::json!({ "code": code })),
        90,
    )
    .await
}

#[tauri::command]
pub async fn wizard_ai_use_chat(
    provider: String,
    admin: super::AdminDraft,
) -> Result<serde_json::Value, String> {
    api_request(
        reqwest::Method::POST,
        &format!("/api/ai-accounts/{provider}/use-for-chat"),
        &admin,
        Some(serde_json::json!({})),
        30,
    )
    .await
}
