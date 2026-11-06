//! Mints the suite session the wizard hands to the desktop shell.

use log::warn;
use std::path::PathBuf;

use super::paths::stack_path;

fn admin_credentials_path() -> PathBuf {
    stack_path()
        .join("conf")
        .join("directory")
        .join("admin-credentials.json")
}

/// Mints a real suite session for the wizard administrator.
///
/// The desktop edition has no browser login hop between the wizard and the
/// suite: the wizard owns the only credential (`admin-credentials.json`). This
/// signs that account in through the same `/api/auth/login` route the suite
/// uses, so the desktop opens with a server-verified identity and the agent VM
/// is only ever provisioned for a logged-in account.
pub async fn mint_desktop_session() -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Credentials {
        email: String,
        password: String,
    }

    let raw = std::fs::read_to_string(admin_credentials_path()).ok()?;
    let credentials: Credentials = serde_json::from_str(&raw).ok()?;
    if credentials.email.trim().is_empty() || credentials.password.is_empty() {
        return None;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let response = client
        .post("http://127.0.0.1:8080/api/auth/login")
        .json(&serde_json::json!({
            "email": credentials.email.trim(),
            "password": credentials.password,
        }))
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
    body.get("access_token")
        .and_then(|value| value.as_str())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

/// The sidecar accepts connections shortly after `/api/product` answers, so a
/// failed first login is expected on a cold start; retry for a short while.
pub(super) async fn wait_for_desktop_session(attempts: u32) -> Option<String> {
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
