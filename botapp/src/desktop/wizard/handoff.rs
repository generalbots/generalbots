//! Administrator credentials and the final wizard handoff.

use log::{info, warn};
use tauri::Manager;

use super::paths::{marker_path, stack_path, workspace_path};
use super::types::{AdminDraft, FinishRequest};
use super::window::reveal_suite;

/// Writes the administrator account the wizard showed on the finish screen.
/// The login fallback reads this file from the stack directory; the password is
/// never logged and the file is restricted to the current user.
fn write_admin_credentials(request: &FinishRequest) -> Result<(), String> {
    write_admin_credentials_draft(&request.admin)
}

/// Writes the administrator account from the wizard draft. The accounts step
/// runs before Finish, so it saves the same file early and signs in with it.
pub(crate) fn write_admin_credentials_draft(admin: &AdminDraft) -> Result<(), String> {
    if admin.email.trim().is_empty() || admin.password.is_empty() {
        return Ok(());
    }
    let dir = stack_path().join("conf").join("directory");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create directory config: {e}"))?;
    let user_id = if admin.user_id.trim().is_empty() {
        format!("local-{}", admin.email.trim())
    } else {
        admin.user_id.clone()
    };
    let payload = serde_json::json!({
        "email": admin.email.trim(),
        "password": admin.password,
        "user_id": user_id,
    });
    let raw = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("serialize administrator credentials: {e}"))?;
    let path = dir.join("admin-credentials.json");
    std::fs::write(&path, raw).map_err(|e| format!("write administrator credentials: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
    info!(
        "wizard: administrator credentials written to {}",
        path.display()
    );
    Ok(())
}

/// Writes `GB_NETWORK_MODE=<mode>` to `stack/conf/.env`, creating or updating
/// the existing file. The botserver reads this on startup to gate peer features.
fn write_network_mode(mode: &str) -> Result<(), String> {
    let dir = stack_path().join("conf");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create conf dir for .env: {e}"))?;
    let env_path = dir.join(".env");
    let key = "GB_NETWORK_MODE";
    let new_line = format!("{key}={mode}");
    let existing = std::fs::read_to_string(&env_path).unwrap_or_default();
    let updated = if existing.lines().any(|l| l.starts_with(&format!("{key}="))) {
        let lines: Vec<String> = existing
            .lines()
            .map(|l| {
                if l.starts_with(&format!("{key}=")) {
                    new_line.clone()
                } else {
                    l.to_string()
                }
            })
            .collect();
        lines.join("\n") + "\n"
    } else if existing.is_empty() {
        format!("{new_line}\n")
    } else {
        format!("{existing}{new_line}\n")
    };
    std::fs::write(&env_path, &updated)
        .map_err(|e| format!("write network mode to .env: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&env_path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&env_path, permissions);
        }
    }
    info!("wizard: GB_NETWORK_MODE={mode} written to {}", env_path.display());
    Ok(())
}

/// Opens the suite once the stack answers, and closes the wizard.
pub fn handoff(app: &tauri::AppHandle, request: &FinishRequest) -> Result<(), String> {
    let stack = stack_path();
    std::fs::create_dir_all(&stack).map_err(|e| format!("create stack folder: {e}"))?;
    write_admin_credentials(request)?;

    // Persist the network mode chosen on the wizard Network step.
    let network_mode = if request.network.mode == "p2p" { "p2p" } else { "offline" };
    if let Err(e) = write_network_mode(network_mode) {
        warn!("wizard: could not persist network mode: {e}");
    }

    let marker = serde_json::json!({
        "completed_at": chrono::Utc::now().to_rfc3339(),
        "desktop_local": cfg!(feature = "desktop-local"),
        "workspace": workspace_path().to_string_lossy(),
        "network_mode": network_mode,
    });
    let raw =
        serde_json::to_string_pretty(&marker).map_err(|e| format!("serialize marker: {e}"))?;
    std::fs::write(marker_path(), raw).map_err(|e| format!("write marker: {e}"))?;

    reveal_suite(app);

    if let Some(wizard) = app.get_webview_window("wizard") {
        if let Err(e) = wizard.close() {
            warn!("could not close the wizard window: {e}");
        }
    }
    info!("first-run wizard completed; suite handed off");
    Ok(())
}
