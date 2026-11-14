//! Administrator credentials and the final wizard handoff.

use log::{info, warn};
use tauri::Manager;

use super::paths::{marker_path, stack_path, workspace_path};
use super::types::{AdminDraft, FinishRequest};
use super::window::reveal_suite_signed_in;

/// Writes the administrator account the wizard showed on the finish screen.
/// The login fallback reads this file from the stack directory; the password is
/// never logged and the file is restricted to the current user.
fn write_admin_credentials(request: &FinishRequest) -> Result<(), String> {
    write_admin_credentials_draft(&request.admin)
}

/// Defaults the administrator user id to `local-{email}` when the wizard did
/// not assign one.
fn resolve_user_id(user_id: &str, email: &str) -> String {
    let user_id = user_id.trim();
    if user_id.is_empty() {
        format!("local-{}", email.trim())
    } else {
        user_id.to_string()
    }
}

/// Derives the non-reversible credentials record persisted for the local
/// administrator. The plaintext password is never written to disk (#2231):
/// only a per-file salt and the SHA-256 digest of `salt || password` are.
fn password_record(password: &str, email: &str) -> (String, String) {
    use sha2::{Digest, Sha256};
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut salt_hasher = Sha256::new();
    salt_hasher.update(email.trim().as_bytes());
    salt_hasher.update(nanos.to_le_bytes());
    salt_hasher.update(std::process::id().to_le_bytes());
    let salt = format!("{:x}", salt_hasher.finalize());
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    (salt, hash)
}

/// The sign-in page only accepts an email address, so an administrator saved
/// under any other identifier (`admin`) could never sign in again.
fn is_email_like(value: &str) -> bool {
    match value.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && !domain.is_empty()
                && !domain.contains('@')
                && !value.chars().any(char::is_whitespace)
        }
        None => false,
    }
}

/// Writes the administrator account from the wizard draft. The accounts step
/// runs before Finish, so it saves the same file early and signs in with it.
pub(crate) fn write_admin_credentials_draft(admin: &AdminDraft) -> Result<(), String> {
    if admin.email.trim().is_empty() || admin.password.is_empty() {
        return Ok(());
    }
    if !is_email_like(admin.email.trim()) {
        return Err(
            "The administrator email must be an email address, for example admin@localhost."
                .to_string(),
        );
    }
    let dir = stack_path().join("conf").join("directory");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create directory config: {e}"))?;
    let user_id = resolve_user_id(&admin.user_id, &admin.email);
    let (salt, hash) = password_record(&admin.password, &admin.email);
    let payload = serde_json::json!({
        "email": admin.email.trim(),
        "password_salt": salt,
        "password_hash": hash,
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

/// The network mode persisted to the stack `.env`: only `p2p` is opt-in,
/// anything else falls back to the offline default.
fn network_mode(requested: &str) -> &'static str {
    if requested == "p2p" {
        "p2p"
    } else {
        "offline"
    }
}

/// Sets `key=value` in a `.env` body, replacing an existing assignment in
/// place and appending when the key is new.
///
/// The appended line is always preceded by a newline: concatenating it onto a
/// file whose last line has no terminator would glue two assignments together.
fn merge_env_line(existing: &str, key: &str, value: &str) -> String {
    let new_line = format!("{key}={value}");
    let prefix = format!("{key}=");
    if existing.lines().any(|line| line.starts_with(&prefix)) {
        let lines: Vec<&str> = existing
            .lines()
            .map(|line| {
                if line.starts_with(&prefix) {
                    new_line.as_str()
                } else {
                    line
                }
            })
            .collect();
        format!("{}\n", lines.join("\n"))
    } else if existing.is_empty() {
        format!("{new_line}\n")
    } else if existing.ends_with('\n') {
        format!("{existing}{new_line}\n")
    } else {
        format!("{existing}\n{new_line}\n")
    }
}

/// Writes `GB_NETWORK_MODE=<mode>` to the stack `.env`, creating or updating
/// the existing file. #1865 — this is the file botserver loads at boot
/// (`{BOTSERVER_STACK_PATH}/.env`); the previous `conf/.env` path was never
/// read, so the first-run answer had no effect.
fn write_network_mode(mode: &str) -> Result<(), String> {
    let dir = stack_path();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create stack dir for .env: {e}"))?;
    let env_path = dir.join(".env");
    let existing = std::fs::read_to_string(&env_path).unwrap_or_default();
    let updated = merge_env_line(&existing, "GB_NETWORK_MODE", mode);
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
    let network_mode = network_mode(&request.network.mode);
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

    // The credentials file keeps only a salted digest, so this is the last
    // moment the password is available for the automatic sign-in.
    reveal_suite_signed_in(
        app,
        request.admin.email.trim().to_string(),
        request.admin.password.clone(),
    );

    if let Some(wizard) = app.get_webview_window("wizard") {
        if let Err(e) = wizard.close() {
            warn!("could not close the wizard window: {e}");
        }
    }
    info!("first-run wizard completed; suite handed off");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_email_like, merge_env_line, network_mode, resolve_user_id};

    #[test]
    fn the_administrator_identifier_must_be_an_email() {
        assert!(is_email_like("admin@localhost"));
        assert!(is_email_like("ana.silva@example.org"));
        assert!(!is_email_like("admin"));
        assert!(!is_email_like("@localhost"));
        assert!(!is_email_like("admin@"));
        assert!(!is_email_like("a@b@c"));
        assert!(!is_email_like("ad min@localhost"));
    }

    #[test]
    fn only_p2p_is_an_opt_in_network_mode() {
        assert_eq!(network_mode("p2p"), "p2p");
        assert_eq!(network_mode("offline"), "offline");
        // Anything unrecognised (empty, older drafts, typos) stays offline.
        assert_eq!(network_mode(""), "offline");
        assert_eq!(network_mode("P2P"), "offline");
        assert_eq!(network_mode("online"), "offline");
    }

    #[test]
    fn a_missing_env_file_gets_the_assignment() {
        assert_eq!(
            merge_env_line("", "GB_NETWORK_MODE", "p2p"),
            "GB_NETWORK_MODE=p2p\n"
        );
    }

    #[test]
    fn an_existing_assignment_is_replaced_in_place() {
        let updated = merge_env_line("A=1\nGB_NETWORK_MODE=offline\nB=2\n", "GB_NETWORK_MODE", "p2p");
        assert_eq!(updated, "A=1\nGB_NETWORK_MODE=p2p\nB=2\n");
    }

    #[test]
    fn a_new_key_is_appended_on_its_own_line() {
        // The last line has no terminator: gluing the assignment onto it would
        // produce a single unreadable `B=2GB_NETWORK_MODE=p2p` line.
        let updated = merge_env_line("A=1\nB=2", "GB_NETWORK_MODE", "offline");
        assert_eq!(updated, "A=1\nB=2\nGB_NETWORK_MODE=offline\n");
        assert!(updated.lines().all(|line| line.contains('=')));
        assert_eq!(updated.lines().count(), 3);
    }

    #[test]
    fn a_terminated_file_is_appended_without_a_blank_line() {
        let updated = merge_env_line("A=1\n", "GB_NETWORK_MODE", "p2p");
        assert_eq!(updated, "A=1\nGB_NETWORK_MODE=p2p\n");
    }

    #[test]
    fn a_prefixed_key_is_not_mistaken_for_the_same_key() {
        let updated = merge_env_line("GB_NETWORK_MODE_OLD=1\n", "GB_NETWORK_MODE", "p2p");
        assert_eq!(updated, "GB_NETWORK_MODE_OLD=1\nGB_NETWORK_MODE=p2p\n");
    }

    #[test]
    fn the_administrator_id_defaults_to_the_email() {
        assert_eq!(resolve_user_id("", " admin@example.com "), "local-admin@example.com");
        assert_eq!(resolve_user_id("   ", "a@b.c"), "local-a@b.c");
        assert_eq!(resolve_user_id(" 42 ", "a@b.c"), "42");
        assert_eq!(resolve_user_id("local-x@y.z", "a@b.c"), "local-x@y.z");
    }
}
