//! Keeps the directory (identity provider) password in step with the local
//! administrator record, and changes the password of a directory account.

use super::*;

/// Field of the administrator record that holds the `password_hash` the
/// directory administrator was last set to, so the update runs once per change.
pub(super) const DIRECTORY_SYNC_MARKER: &str = "directory_synced_hash";

/// Whether the directory administrator still has to receive the password the
/// record holds: the digest changed since the last successful update.
pub(super) fn needs_directory_sync(record: &serde_json::Value) -> bool {
    let hash = record.get("password_hash").and_then(|v| v.as_str()).unwrap_or("");
    let synced = record
        .get(DIRECTORY_SYNC_MARKER)
        .and_then(|v| v.as_str())
        .unwrap_or("");
    !hash.is_empty() && hash != synced
}

/// The directory account the server created on its first start.
fn bootstrap_admin_id() -> Option<String> {
    let path = std::path::PathBuf::from(get_stack_path()).join("conf/directory/bootstrap-state.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let id = json.get("user_id")?.as_str()?.trim();
    (!id.is_empty()).then(|| id.to_string())
}

fn http_client() -> Option<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()
}

/// The directory API base and an administrator token, when both are available.
async fn directory_admin(state: &Arc<AppState>, http: &reqwest::Client) -> Option<(String, String)> {
    let auth = state.auth_service.as_ref()?;
    let service = auth.lock().await;
    let token = resolve_admin_token(http, &*service).await?;
    Some((service.api_url(), token))
}

/// Sets the password of the directory account `user_id` with an administrator
/// token. Returns the HTTP status the directory answered, or `None` when it
/// could not be asked.
async fn set_directory_password(
    state: &Arc<AppState>,
    user_id: &str,
    password: &str,
) -> Option<reqwest::StatusCode> {
    let http = http_client()?;
    let (api_url, token) = directory_admin(state, &http).await?;
    let body = serde_json::json!({
        "newPassword": { "password": password, "changeRequired": false }
    });
    match http
        .post(format!("{}/v2/users/{}/password", api_url, user_id))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
    {
        Ok(response) => Some(response.status()),
        Err(e) => {
            warn!("Directory password update could not be sent: {e}");
            None
        }
    }
}

/// Gives the directory administrator the password the local record at `path`
/// now holds, then records that in the file. Best effort: when the directory
/// is down or its policy refuses the password, the local record still signs
/// the person in and the update is tried again at the next sign-in.
pub(super) async fn sync_directory_admin_password(
    state: Arc<AppState>,
    password: String,
    path: std::path::PathBuf,
) -> bool {
    let Some(user_id) = bootstrap_admin_id() else {
        debug!("No directory administrator to update (bootstrap state missing)");
        return false;
    };
    let expected_hash = std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|json| json.get("password_hash").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| { log::debug!("no stored directory password hash"); Default::default() });
    match set_directory_password(&state, &user_id, &password).await {
        Some(status) if status.is_success() => {
            mark_synced(&path, &expected_hash);
            info!("Directory administrator password now matches the local administrator");
            true
        }
        Some(status) => {
            warn!("Directory refused the administrator password update ({status})");
            false
        }
        None => false,
    }
}

/// Stores the synced digest, unless the password changed again meanwhile.
fn mark_synced(path: &std::path::Path, expected_hash: &str) {
    let Some(mut record) = std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
    else {
        return;
    };
    let current = record.get("password_hash").and_then(|v| v.as_str()).unwrap_or("");
    if expected_hash.is_empty() || current != expected_hash {
        return;
    }
    if let Some(obj) = record.as_object_mut() {
        obj.insert(
            DIRECTORY_SYNC_MARKER.to_string(),
            serde_json::Value::String(expected_hash.to_string()),
        );
    }
    if let Err(e) = write_admin_record(path, &record) {
        warn!("Local admin: could not record the directory update: {e}");
    }
}

/// Why a directory account's password could not be changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DirectoryChangeError {
    CurrentPasswordWrong,
    Unavailable,
    Refused,
}

/// Changes the password of a directory account after the directory itself
/// confirmed the current one (a password session check).
pub(super) async fn change_directory_user_password(
    state: &Arc<AppState>,
    login_name: &str,
    user_id: &str,
    current: &str,
    new_password: &str,
) -> Result<(), DirectoryChangeError> {
    let http = http_client().ok_or(DirectoryChangeError::Unavailable)?;
    let (api_url, token) = directory_admin(state, &http)
        .await
        .ok_or(DirectoryChangeError::Unavailable)?;
    let check = serde_json::json!({
        "checks": {
            "user": { "loginName": login_name },
            "password": { "password": current }
        }
    });
    let verified = http
        .post(format!("{}/v2/sessions", api_url))
        .bearer_auth(&token)
        .json(&check)
        .send()
        .await
        .map_err(|_| DirectoryChangeError::Unavailable)?;
    if !verified.status().is_success() {
        return Err(DirectoryChangeError::CurrentPasswordWrong);
    }
    match set_directory_password(state, user_id, new_password).await {
        Some(status) if status.is_success() => Ok(()),
        Some(_) => Err(DirectoryChangeError::Refused),
        None => Err(DirectoryChangeError::Unavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_directory_is_updated_once_per_password() {
        let fresh = serde_json::json!({ "password_hash": "abc" });
        assert!(needs_directory_sync(&fresh));
        let synced = serde_json::json!({ "password_hash": "abc", DIRECTORY_SYNC_MARKER: "abc" });
        assert!(!needs_directory_sync(&synced));
        let changed = serde_json::json!({ "password_hash": "def", DIRECTORY_SYNC_MARKER: "abc" });
        assert!(needs_directory_sync(&changed));
        let empty = serde_json::json!({ "email": "admin@localhost" });
        assert!(!needs_directory_sync(&empty));
    }
}
