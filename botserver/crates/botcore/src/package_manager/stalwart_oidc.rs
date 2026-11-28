//! Stalwart ↔ Zitadel OIDC provisioning. Called by `DirectorySetup::initialize`
//! once the PAT and BotServer credentials exist. HTTP helpers take an explicit
//! client; the config writer is a pure text transform plus a 0600 file write.
//! Client secrets are never logged — only the config path and client id.

use anyhow::{anyhow, Context, Result};
use log::{info, warn};
use reqwest::Client;
use std::path::Path;

const STALWART_APP_NAME: &str = "Stalwart";
const STORAGE_DIRECTORY_LINE: &str = "storage.directory = \"zitadel\"";

/// Zitadel v2 introspection endpoint for the given issuer base URL.
/// Conservative: strips trailing slashes, never adds a scheme or guesses a port.
pub fn introspection_url(base_url: &str) -> String {
    format!("{}/oauth/v2/introspect", base_url.trim_end_matches('/'))
}

/// Request body used to register the Stalwart Web OIDC application.
fn oidc_app_body() -> serde_json::Value {
    serde_json::json!({
        "name": STALWART_APP_NAME,
        "redirectUris": ["/admin/oauth/callback", "/account/oauth/callback"],
        "responseTypes": ["OIDC_RESPONSE_TYPE_CODE"],
        "grantTypes": [
            "OIDC_GRANT_TYPE_AUTHORIZATION_CODE",
            "OIDC_GRANT_TYPE_REFRESH_TOKEN"
        ],
        "appType": "OIDC_APP_TYPE_WEB",
        "authMethodType": "OIDC_AUTH_METHOD_TYPE_POST",
        "devMode": true
    })
}

/// Read an error body without ever echoing it into the log: Zitadel error
/// payloads may quote request fields (including secrets) back to the caller.
async fn discard_error_body(response: reqwest::Response, operation: &str) {
    if let Err(e) = response.text().await {
        warn!("Could not read {operation} error body: {e}");
    }
}

/// Create or reuse the `Stalwart` OIDC app and return (client id, secret).
pub async fn provision_oidc_app(
    client: &Client,
    base_url: &str,
    auth_header: &str,
    project_id: &str,
) -> Result<(String, String)> {
    let base = base_url.trim_end_matches('/');
    match find_app(client, base, auth_header, project_id).await? {
        Some((app_id, known_client_id)) => {
            info!("Reusing {STALWART_APP_NAME} OIDC app ({app_id})");
            let client_id = match known_client_id {
                Some(id) if !id.is_empty() => id,
                _ => fetch_client_id(client, base, auth_header, project_id, &app_id).await?,
            };
            let secret = regenerate_secret(client, base, auth_header, project_id, &app_id).await?;
            Ok((client_id, secret))
        }
        None => create_app(client, base, auth_header, project_id).await,
    }
}

/// Look up an existing app by name through management v1 app search.
async fn find_app(
    client: &Client,
    base: &str,
    auth_header: &str,
    project_id: &str,
) -> Result<Option<(String, Option<String>)>> {
    let url = format!("{base}/management/v1/projects/{project_id}/apps/_search");
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .json(&serde_json::json!({}))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        discard_error_body(response, "app search").await;
        return Err(anyhow!("Failed to search Zitadel apps ({status})"));
    }

    let data: serde_json::Value = response.json().await?;
    let Some(results) = data.get("result").and_then(|v| v.as_array()) else {
        return Ok(None);
    };

    for app in results {
        if app.get("name").and_then(|v| v.as_str()) != Some(STALWART_APP_NAME) {
            continue;
        }
        let Some(id) = app.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let client_id = app
            .get("oidcConfig")
            .and_then(|c| c.get("clientId"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        return Ok(Some((id.to_string(), client_id)));
    }

    Ok(None)
}

/// Read the client id of an existing app (the search response does not always
/// carry `oidcConfig`).
async fn fetch_client_id(
    client: &Client,
    base: &str,
    auth_header: &str,
    project_id: &str,
    app_id: &str,
) -> Result<String> {
    let url = format!("{base}/management/v1/projects/{project_id}/apps/{app_id}");
    let response = client
        .get(url)
        .header("Authorization", auth_header)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        discard_error_body(response, "app read").await;
        return Err(anyhow!("Failed to read Zitadel app ({status})"));
    }

    let data: serde_json::Value = response.json().await?;
    let client_id = data
        .get("oidcConfig")
        .and_then(|c| c.get("clientId"))
        .and_then(|v| v.as_str())
        .or_else(|| data.get("clientId").and_then(|v| v.as_str()))
        .ok_or_else(|| anyhow!("No clientId in Zitadel app response"))?;

    Ok(client_id.to_string())
}

/// Zitadel never returns an existing client secret, so regenerate it on reuse.
async fn regenerate_secret(
    client: &Client,
    base: &str,
    auth_header: &str,
    project_id: &str,
    app_id: &str,
) -> Result<String> {
    let url = format!(
        "{base}/management/v1/projects/{project_id}/apps/{app_id}/oidc_config/_generate_client_secret"
    );
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .json(&serde_json::json!({}))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        discard_error_body(response, "client secret generation").await;
        return Err(anyhow!("Failed to regenerate Stalwart client secret ({status})"));
    }

    let data: serde_json::Value = response.json().await?;
    let secret = data
        .get("clientSecret")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("No clientSecret in Zitadel secret response"))?;

    Ok(secret.to_string())
}

/// Create a new app; the creation response carries both credentials.
async fn create_app(
    client: &Client,
    base: &str,
    auth_header: &str,
    project_id: &str,
) -> Result<(String, String)> {
    info!("Creating {STALWART_APP_NAME} OIDC app in Zitadel");
    let url = format!("{base}/management/v1/projects/{project_id}/apps/oidc");
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .json(&oidc_app_body())
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        discard_error_body(response, "OIDC app creation").await;
        return Err(anyhow!("Failed to create Stalwart OIDC app ({status})"));
    }

    let data: serde_json::Value = response.json().await?;
    let client_id = data
        .get("clientId")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("No clientId in Stalwart OIDC app response"))?;
    let secret = data
        .get("clientSecret")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("No clientSecret in Stalwart OIDC app response"))?;

    Ok((client_id.to_string(), secret.to_string()))
}

/// Write (or update) Stalwart's directory configuration. Only the status and
/// path are logged; the secret is never included.
pub fn write_stalwart_config(
    config_path: &Path,
    introspection_endpoint: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<()> {
    let existing = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read {}", config_path.display()));
        }
    };

    let updated = merge_stalwart_config(
        &existing,
        introspection_endpoint,
        client_id,
        client_secret,
    );

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(config_path, &updated)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(config_path, std::fs::Permissions::from_mode(0o600))?;
    }

    info!("Wrote Stalwart OIDC directory block to {}", config_path.display());
    Ok(())
}

/// Escape a value for use inside a double-quoted TOML basic string.
fn toml_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04X}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped.push('"');
    escaped
}

/// The exact block inserted into `config.toml`. Keeping this in one function
/// makes the written text directly unit-testable.
fn directory_block(endpoint: &str, client_id: &str, client_secret: &str) -> String {
    format!(
        "[directory.\"zitadel\"]\n\
         type = \"oidc\"\n\
         timeout = \"15s\"\n\
         endpoint.method = \"introspect\"\n\
         endpoint.url = {}\n\
         auth.method = \"client_secret_post\"\n\
         auth.username = {}\n\
         auth.secret = {}\n\
         fields.email = \"email\"\n\
         fields.username = \"preferred_username\"\n\
         fields.full-name = \"name\"\n",
        toml_string(endpoint),
        toml_string(client_id),
        toml_string(client_secret),
    )
}

fn is_zitadel_header(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "[directory.\"zitadel\"]" || trimmed == "[directory.'zitadel']"
}

fn is_table_header(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('[') && trimmed.ends_with(']')
}

fn is_storage_directory_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix("storage.directory") else {
        return false;
    };
    rest.trim_start().starts_with('=')
}

/// Pure, idempotent merge of the block and `storage.directory` into existing
/// TOML text. Re-running it on its own output leaves the text unchanged.
fn merge_stalwart_config(
    existing: &str,
    endpoint: &str,
    client_id: &str,
    client_secret: &str,
) -> String {
    let mut lines: Vec<String> = existing.split('\n').map(str::to_string).collect();
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }

    if let Some(start) = lines.iter().position(|line| is_zitadel_header(line)) {
        let mut end = start + 1;
        while end < lines.len() && !is_table_header(&lines[end]) {
            end += 1;
        }
        lines.drain(start..end);
        while lines.last().is_some_and(|line| line.trim().is_empty()) {
            lines.pop();
        }
    }

    if let Some(line) = lines.iter_mut().find(|line| is_storage_directory_line(line)) {
        *line = STORAGE_DIRECTORY_LINE.to_string();
    } else {
        let insert_at = lines
            .iter()
            .position(|line| line.trim() == "[storage]")
            .map(|index| index + 1)
            .unwrap_or_else(|| {
                lines
                    .iter()
                    .position(|line| is_table_header(line))
                    .unwrap_or(lines.len())
            });
        lines.insert(insert_at, STORAGE_DIRECTORY_LINE.to_string());
    }

    let mut result = lines.join("\n");
    if !result.is_empty() {
        result.push_str("\n\n");
    }
    result.push_str(directory_block(endpoint, client_id, client_secret).trim_end());
    result.push('\n');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_introspection_url_strips_trailing_slashes() {
        assert_eq!(
            introspection_url("https://zitadel.example/"),
            "https://zitadel.example/oauth/v2/introspect"
        );
        assert_eq!(
            introspection_url("http://localhost:8300//"),
            "http://localhost:8300/oauth/v2/introspect"
        );
    }

    #[test]
    fn test_directory_block_text() {
        let block = directory_block(
            "https://zitadel.example/oauth/v2/introspect",
            "test-client",
            "test-secret",
        );
        let expected = concat!(
            "[directory.\"zitadel\"]\n",
            "type = \"oidc\"\n",
            "timeout = \"15s\"\n",
            "endpoint.method = \"introspect\"\n",
            "endpoint.url = \"https://zitadel.example/oauth/v2/introspect\"\n",
            "auth.method = \"client_secret_post\"\n",
            "auth.username = \"test-client\"\n",
            "auth.secret = \"test-secret\"\n",
            "fields.email = \"email\"\n",
            "fields.username = \"preferred_username\"\n",
            "fields.full-name = \"name\"\n",
        );
        assert_eq!(block, expected);

        let escaped = directory_block("endpoint", "client", "a\"b\\c");
        assert!(escaped.contains("auth.secret = \"a\\\"b\\\\c\"\n"));
    }

    #[test]
    fn test_merge_inserts_block_and_storage_line_without_touching_other_content() {
        let merged = merge_stalwart_config(
            "[server]\nport = 8080\n",
            "https://zitadel.example/oauth/v2/introspect",
            "test-client",
            "test-secret",
        );
        assert!(merged.contains("[server]\nport = 8080"));
        assert_eq!(merged.matches("storage.directory = \"zitadel\"").count(), 1);
        assert_eq!(merged.matches("[directory.\"zitadel\"]").count(), 1);
        assert!(merged.ends_with("fields.full-name = \"name\"\n"));
    }

    #[test]
    fn test_merge_replaces_existing_block_and_line_idempotently() {
        let existing = concat!(
            "storage.directory = \"local\"\n",
            "[directory.\"zitadel\"]\n",
            "type = \"local\"\n",
            "auth.secret = \"stale\"\n",
            "\n",
            "[server]\n",
            "port = 8080\n",
        );
        let first = merge_stalwart_config(
            existing,
            "https://zitadel.example/oauth/v2/introspect",
            "new-client",
            "new-secret",
        );
        let second = merge_stalwart_config(
            &first,
            "https://zitadel.example/oauth/v2/introspect",
            "new-client",
            "new-secret",
        );

        assert_eq!(first, second);
        assert_eq!(first.matches("storage.directory = \"zitadel\"").count(), 1);
        assert_eq!(first.matches("[directory.\"zitadel\"]").count(), 1);
        assert_eq!(first.matches("auth.secret = \"new-secret\"").count(), 1);
        assert!(!first.contains("stale"));
        assert!(!first.contains("\"local\""));
        assert!(first.contains("[server]\nport = 8080"));
    }
}
