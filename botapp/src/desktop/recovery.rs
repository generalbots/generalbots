//! "Forgot password" from the sign-in page, inside the desktop app only.
//!
//! The server writes a one-use secret and the port of its loopback recovery
//! listener to `conf/directory/local-recovery.json` in the stack, a folder in
//! this Windows account's own profile. Reading that file is the proof that the
//! request comes from this computer and this account; the reset itself runs in
//! the server, which also signs out every older session.

use log::{info, warn};

#[derive(serde::Deserialize)]
struct RecoveryFile {
    port: u16,
    secret: String,
}

fn recovery_file() -> Result<RecoveryFile, String> {
    let path = super::wizard::stack_path().join("conf/directory/local-recovery.json");
    let raw = std::fs::read_to_string(path).map_err(|_| "recovery_unavailable".to_string())?;
    parse_recovery_file(&raw)
}

fn parse_recovery_file(raw: &str) -> Result<RecoveryFile, String> {
    let file: RecoveryFile =
        serde_json::from_str(raw).map_err(|_| "recovery_unavailable".to_string())?;
    if file.port == 0 || file.secret.trim().is_empty() {
        return Err("recovery_unavailable".to_string());
    }
    Ok(file)
}

/// The stable code of a refused reset, so the page can say it in its language.
fn refusal_code(body: &serde_json::Value) -> String {
    body.get("code")
        .and_then(|v| v.as_str())
        .filter(|code| !code.is_empty())
        .unwrap_or("recovery_failed")
        .to_string()
}

/// A native confirmation a script in the page cannot click; the test rig,
/// driven through DevTools, skips it.
fn person_confirms(app: &tauri::AppHandle) -> bool {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    if super::splash::is_test_rig() {
        return true;
    }
    app.dialog()
        .message(super::i18n::t("recovery-confirm-message"))
        .title(super::i18n::t("recovery-confirm-title"))
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancel)
        .blocking_show()
}

/// Sets a new administrator password and returns the new session token.
/// Errors are stable codes (`password_too_short`, `recovery_unavailable`, ...).
#[tauri::command]
pub async fn local_password_reset(
    app: tauri::AppHandle,
    new_password: String,
) -> Result<serde_json::Value, String> {
    let file = recovery_file()?;
    if !person_confirms(&app) {
        return Err("recovery_cancelled".to_string());
    }
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(std::time::Duration::from_secs(2))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|_| "recovery_unavailable".to_string())?;
    let response = client
        .post(format!("http://127.0.0.1:{}/local-recovery/reset", file.port))
        .header("x-gb-recovery-secret", file.secret.trim())
        .json(&serde_json::json!({ "new_password": new_password }))
        .send()
        .await
        .map_err(|_| "recovery_unavailable".to_string())?;
    let status = response.status();
    let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        warn!("recovery: the assistant refused the password reset ({status})");
        return Err(refusal_code(&body));
    }
    let token = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "recovery_failed".to_string())?;
    info!("recovery: administrator password reset from the sign-in page");
    Ok(serde_json::json!({
        "access_token": token,
        "email": body.get("email").and_then(|v| v.as_str()).unwrap_or(""),
    }))
}

#[cfg(test)]
mod tests {
    use super::{parse_recovery_file, refusal_code};

    #[test]
    fn the_recovery_file_needs_a_port_and_a_secret() {
        assert!(parse_recovery_file(r#"{"port":51234,"secret":"ab"}"#).is_ok());
        assert!(parse_recovery_file(r#"{"port":0,"secret":"ab"}"#).is_err());
        assert!(parse_recovery_file(r#"{"port":51234,"secret":" "}"#).is_err());
        assert!(parse_recovery_file("not json").is_err());
    }

    #[test]
    fn a_refusal_keeps_the_server_code() {
        let weak = serde_json::json!({ "code": "password_needs_symbol" });
        assert_eq!(refusal_code(&weak), "password_needs_symbol");
        assert_eq!(refusal_code(&serde_json::Value::Null), "recovery_failed");
    }
}
