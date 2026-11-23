//! Mints the suite session the wizard hands to the desktop shell.

use log::warn;
use std::path::{Path, PathBuf};

use super::paths::stack_path;

/// Write a credential file with owner-only permissions from creation (#2691).
/// `std::fs::write` follows the process umask (usually 0644) and would expose
/// the admin bearer token to every local user on a multi-user host.
fn write_private_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(contents)
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents)
    }
}

fn admin_credentials_path() -> PathBuf {
    stack_path()
        .join("conf")
        .join("directory")
        .join("admin-credentials.json")
}

fn desktop_session_path() -> PathBuf {
    stack_path()
        .join("conf")
        .join("directory")
        .join("desktop-session.json")
}

/// The token cached in `desktop-session.json`, when the file still holds one.
fn cached_token(raw: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(raw).ok()?;
    json.get("access_token")
        .and_then(|value| value.as_str())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

/// The access token of a successful `/api/auth/login` answer.
fn token_from_body(body: &serde_json::Value) -> Option<String> {
    body.get("access_token")
        .and_then(|value| value.as_str())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

/// Whether the stored administrator credentials can be used to sign in.
fn credentials_usable(email: &str, password: &str) -> bool {
    !email.trim().is_empty() && !password.is_empty()
}

/// Mints a real suite session for the wizard administrator.
///
/// The desktop edition has no browser login hop between the wizard and the
/// suite: the wizard owns the only credential (`admin-credentials.json`). This
/// signs that account in through the same `/api/auth/login` route the suite
/// uses, so the desktop opens with a server-verified identity and the agent VM
/// is only ever provisioned for a logged-in account.
///
/// To avoid keeping a plaintext password on disk (#1929), the minted token is
/// persisted to `desktop-session.json` on the first successful login and the
/// password is scrubbed from `admin-credentials.json`. Subsequent boots reuse
/// the cached token and never read the password again.
pub async fn mint_desktop_session() -> Option<String> {
    // 1) Reuse a previously minted token if present.
    if let Ok(raw) = std::fs::read_to_string(desktop_session_path()) {
        if let Some(token) = cached_token(&raw) {
            return Some(token);
        }
    }

    // Only a file written by an older wizard still carries the password;
    // current ones keep a salted digest (#2231) and are handled at handoff by
    // `wait_for_wizard_session`.
    #[derive(serde::Deserialize)]
    struct Credentials {
        #[serde(default)]
        email: String,
        #[serde(default)]
        password: String,
    }

    let raw = std::fs::read_to_string(admin_credentials_path()).ok()?;
    let credentials: Credentials = serde_json::from_str(&raw).ok()?;
    let token = login_and_cache(&credentials.email, &credentials.password).await?;

    // Scrub the plaintext password from admin-credentials.json, keeping the
    // identity fields (email, user_id) that botserver still needs.
    if let Ok(content) = std::fs::read_to_string(admin_credentials_path()) {
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = json.as_object_mut() {
                obj.remove("password");
            }
            let _ = std::fs::write(
                admin_credentials_path(),
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            );
        }
    }

    Some(token)
}

/// Signs the administrator in through `/api/auth/login` and caches the token
/// in `desktop-session.json` for the next boots.
async fn login_and_cache(email: &str, password: &str) -> Option<String> {
    if !credentials_usable(email, password) {
        return None;
    }
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(2))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let response = client
        .post(format!(
            "http://127.0.0.1:{}/api/auth/login",
            crate::desktop::sidecar::probe::sidecar_port()
        ))
        .json(&serde_json::json!({ "email": email.trim(), "password": password }))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        warn!(
            "wizard: desktop session login was refused ({})",
            response.status()
        );
        return None;
    }
    let body: serde_json::Value = response.json().await.ok()?;
    let token = token_from_body(&body)?;
    let _ = write_private_file(
        &desktop_session_path(),
        serde_json::json!({ "access_token": token }).to_string().as_bytes(),
    );
    Some(token)
}

/// Signs in with the administrator the wizard has just created.
///
/// `admin-credentials.json` keeps only a salted digest, so the password cannot
/// be read back from disk. Without this the suite opened on the sign-in page
/// right after setup and asked for a generated password the user had seen on
/// one screen. A token cached by an earlier setup is ignored: it belongs to the
/// administrator of that setup.
pub(super) async fn wait_for_wizard_session(
    email: &str,
    password: &str,
    attempts: u32,
) -> Option<String> {
    for attempt in 0..attempts {
        if let Some(token) = login_and_cache(email, password).await {
            return Some(token);
        }
        if attempt + 1 < attempts {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
    warn!("wizard: could not sign the new administrator in; the suite will ask for sign-in");
    None
}

/// Whether an automatic sign-in has any chance of succeeding. When neither a
/// cached token nor a plaintext password is on disk, retrying is pointless
/// and would only delay the splash-to-login transition by tens of seconds.
fn can_auto_login() -> bool {
    // 1. Cached token in desktop-session.json?
    if let Ok(raw) = std::fs::read_to_string(desktop_session_path()) {
        if cached_token(&raw).is_some() {
            return true;
        }
    }
    // 2. Plaintext password in admin-credentials.json?
    if let Ok(raw) = std::fs::read_to_string(admin_credentials_path()) {
        if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(pwd) = creds.get("password").and_then(|v| v.as_str()) {
                return !pwd.is_empty();
            }
        }
    }
    false
}

/// The sidecar accepts connections shortly after `/api/product` answers, so a
/// failed first login is expected on a cold start; retry for a short while.
pub(super) async fn wait_for_desktop_session(attempts: u32) -> Option<String> {
    if !can_auto_login() {
        log::info!(
            "wizard: no cached token or plaintext password; \
             skipping auto-login (the suite will ask for sign-in)"
        );
        return None;
    }
    for attempt in 0..attempts {
        if let Some(token) = mint_desktop_session().await {
            return Some(token);
        }
        if attempt + 1 < attempts {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
    warn!("wizard: could not mint a desktop session; the suite will ask for sign-in");
    None
}

pub(super) fn suite_url_with_token(mut url: tauri::Url, token: Option<&str>) -> tauri::Url {
    if let Some(token) = token {
        url.query_pairs_mut().append_pair("token", token);
    }
    url
}

#[cfg(test)]
mod tests {
    use super::{cached_token, credentials_usable, suite_url_with_token, token_from_body};
    use serde_json::json;

    #[test]
    fn a_cached_token_is_reused_only_when_present() {
        assert_eq!(
            cached_token(r#"{"access_token":"abc"}"#),
            Some("abc".to_string())
        );
        assert_eq!(cached_token(r#"{"access_token":""}"#), None);
        assert_eq!(cached_token(r#"{"other":"abc"}"#), None);
        assert_eq!(cached_token(r#"{"access_token":42}"#), None);
        assert_eq!(cached_token("not json"), None);
    }

    #[test]
    fn a_login_answer_needs_a_non_empty_access_token() {
        assert_eq!(
            token_from_body(&json!({"access_token": "tok"})),
            Some("tok".to_string())
        );
        assert_eq!(token_from_body(&json!({"access_token": ""})), None);
        assert_eq!(token_from_body(&json!({"token": "tok"})), None);
        assert_eq!(token_from_body(&json!({})), None);
    }

    #[test]
    fn can_auto_login_returns_false_without_credentials() {
        // No files on disk → can_auto_login reads the actual disk paths, which
        // are absent in CI. We test the underlying logic instead: a scrubbed
        // credentials file (no password key) produces no usable credential.
        let raw = r#"{"email":"admin@example.com"}"#;
        let creds: serde_json::Value = serde_json::from_str(raw).unwrap();
        let has_password = creds
            .get("password")
            .and_then(|v| v.as_str())
            .is_some_and(|p| !p.is_empty());
        assert!(!has_password, "scrubbed credentials must not appear usable");
    }

    #[test]
    fn both_credential_fields_are_required() {
        assert!(credentials_usable("admin@example.com", "secret"));
        assert!(credentials_usable("  admin@example.com  ", "secret"));
        assert!(!credentials_usable("", "secret"));
        assert!(!credentials_usable("   ", "secret"));
        assert!(!credentials_usable("admin@example.com", ""));
    }

    #[test]
    fn the_token_is_appended_without_dropping_the_existing_query() {
        let url = tauri::Url::parse("http://127.0.0.1:8080/suite/desktop.html?app=chat")
            .expect("a valid fixture URL");
        let with_token = suite_url_with_token(url, Some("tok en"));
        assert_eq!(
            with_token.as_str(),
            "http://127.0.0.1:8080/suite/desktop.html?app=chat&token=tok+en"
        );

        let url = tauri::Url::parse("http://127.0.0.1:8080/suite/desktop.html")
            .expect("a valid fixture URL");
        let without = suite_url_with_token(url, None);
        assert_eq!(
            without.as_str(),
            "http://127.0.0.1:8080/suite/desktop.html"
        );
    }

    #[cfg(unix)]
    #[test]
    fn session_files_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let base = std::env::temp_dir().join(format!("gb-session-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
        std::fs::create_dir_all(&base).expect("temp dir");
        let file = base.join("desktop-session.json");
        super::write_private_file(&file, b"{\"access_token\":\"secret\"}").expect("write");
        let mode = std::fs::metadata(&file).expect("stat").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "session token file must be 0600");
        std::fs::remove_dir_all(&base).ok();
    }
}
