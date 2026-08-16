//! Connector runtime operations: Vault-backed credential storage, connectivity
//! tests and sync schedule validation.
//!
//! Sensitive connector fields (api keys, OAuth client secrets, passwords) are
//! written to Vault at `secret/gbo/orgs/{org}/bots/{bot}/sources/{connector_id}`
//! and never persisted to the `connectors` table or echoed to the UI. The DB
//! row only keeps the vault path plus non-sensitive configuration.

use botcoresecrets::SecretsManager;
use serde_json::Value;
use std::collections::HashMap;

/// Auth-config keys considered sensitive and therefore vaulted.
const SENSITIVE_KEYS: &[&str] = &[
    "api_key",
    "api_key_header",
    "password",
    "oauth2_client_secret",
    "client_secret",
    "token",
];

/// Strips sensitive fields out of an auth config JSON object, returning the
/// sanitized object and the map of secrets to vault. Non-object configs are
/// returned unchanged with no secrets.
pub fn split_sensitive_auth(config: &Value) -> (Value, HashMap<String, String>) {
    let Some(obj) = config.as_object() else {
        return (config.clone(), HashMap::new());
    };
    let mut sanitized = obj.clone();
    let mut secrets = HashMap::new();
    for key in SENSITIVE_KEYS {
        if let Some(value) = sanitized.remove(*key) {
            if let Some(s) = value.as_str() {
                if !s.is_empty() {
                    secrets.insert((*key).to_string(), s.to_string());
                }
            } else {
                secrets.insert((*key).to_string(), value.to_string());
            }
        }
    }
    (Value::Object(sanitized), secrets)
}

/// Vault KV2 path for a connector's secrets, scoped per org/bot.
pub fn secrets_path(org_id: &str, bot_id: &str, connector_id: &str) -> String {
    format!("gbo/orgs/{org_id}/bots/{bot_id}/sources/{connector_id}")
}

/// Persists the connector secrets in Vault. Logs a warning when Vault is not
/// configured so operators know credentials were not vaulted.
pub fn store_secrets(
    manager: &SecretsManager,
    path: &str,
    secrets: &HashMap<String, String>,
) {
    if secrets.is_empty() {
        return;
    }
    if !manager.is_configured() {
        log::warn!(
            "connector secrets for {path} not vaulted: Vault is not configured"
        );
        return;
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    match rt {
        Ok(runtime) => {
            let result = runtime.block_on(manager.put_secret(path, secrets.clone()));
            if let Err(e) = result {
                log::error!("failed to store connector secrets at {path}: {e}");
            }
        }
        Err(e) => log::error!("failed to build runtime for secret storage: {e}"),
    }
}

/// Reads the vaulted secrets for a connector, if any.
pub fn load_secrets(manager: &SecretsManager, path: &str) -> HashMap<String, String> {
    if !manager.is_configured() || path.is_empty() {
        return HashMap::new();
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    match rt {
        Ok(runtime) => match runtime.block_on(manager.get_secret(path)) {
            Ok(secrets) => secrets,
            Err(e) => {
                log::warn!("failed to read connector secrets at {path}: {e}");
                HashMap::new()
            }
        },
        Err(e) => {
            log::error!("failed to build runtime for secret read: {e}");
            HashMap::new()
        }
    }
}

/// Tests connectivity for a connector based on its type:
/// - HTTP-ish connectors (REST, GraphQL, sheets, sharepoint, SaaS) GET the
///   configured base URL (or the first endpoint URL);
/// - databases (mysql, postgres) attempt a TCP connect to host:port;
/// - unknown types fall back to an HTTP check against the first endpoint URL.
///
/// Returns (ok, latency_ms, detail).
pub async fn test_connection(
    connector_type: &str,
    auth: &Value,
    endpoints: &Value,
) -> (bool, u64, String) {
    let started = std::time::Instant::now();
    let base_url = auth
        .get("base_url")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            endpoints
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("url"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });

    let lower = connector_type.to_lowercase();
    let is_db = lower.contains("mysql") || lower.contains("postgres");

    if is_db {
        let host = auth
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost");
        let port = auth
            .get("port")
            .and_then(|v| v.as_str())
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(if lower.contains("mysql") { 3306 } else { 5432 });
        return match tokio::net::TcpStream::connect((host, port)).await {
            Ok(_) => (
                true,
                started.elapsed().as_millis() as u64,
                format!("TCP connect to {host}:{port} succeeded"),
            ),
            Err(e) => (
                false,
                started.elapsed().as_millis() as u64,
                format!("TCP connect to {host}:{port} failed: {e}"),
            ),
        };
    }

    let Some(url) = base_url else {
        return (
            false,
            started.elapsed().as_millis() as u64,
            "no base_url or endpoint url configured".to_string(),
        );
    };

    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let ok = status.is_success() || status.is_redirection();
            (
                ok,
                started.elapsed().as_millis() as u64,
                format!("HTTP GET {url} -> {status}"),
            )
        }
        Err(e) => (
            false,
            started.elapsed().as_millis() as u64,
            format!("HTTP GET {url} failed: {e}"),
        ),
    }
}

/// Validates a cron-like schedule string. Accepts `*/n`, numeric, `*` and
/// comma lists across 5 standard fields (min hour dom mon dow).
pub fn validate_schedule(schedule: Option<&str>) -> Result<(), String> {
    let Some(raw) = schedule else {
        return Ok(());
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(());
    }
    let fields: Vec<&str> = raw.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(format!(
            "schedule must have 5 cron fields (min hour dom mon dow), got {}",
            fields.len()
        ));
    }
    for field in fields {
        for part in field.split(',') {
            if part == "*" || part == "?" {
                continue;
            }
            if let Some(step) = part.strip_prefix("*/") {
                if step.parse::<u32>().map(|v| v >= 1).unwrap_or(false) {
                    continue;
                }
            }
            if let Some((lo, hi)) = part.split_once('-') {
                if lo.parse::<u32>().ok().zip(hi.parse::<u32>().ok()).is_some() {
                    continue;
                }
            }
            if part.parse::<u32>().is_err() {
                return Err(format!("invalid cron field: {part}"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sensitive_auth_extracts_secrets() {
        let config = serde_json::json!({
            "auth_type": "bearer",
            "api_key": "sk_live_123",
            "base_url": "https://api.example.com",
            "username": "bob"
        });
        let (sanitized, secrets) = split_sensitive_auth(&config);
        assert_eq!(secrets.get("api_key").map(String::as_str), Some("sk_live_123"));
        assert!(sanitized.get("api_key").is_none());
        assert_eq!(sanitized["username"], "bob");
        assert_eq!(sanitized["base_url"], "https://api.example.com");
    }

    #[test]
    fn test_split_sensitive_auth_non_object() {
        let (sanitized, secrets) = split_sensitive_auth(&serde_json::json!("plain"));
        assert!(secrets.is_empty());
        assert_eq!(sanitized, serde_json::json!("plain"));
    }

    #[test]
    fn test_secrets_path_scoped_per_org_bot() {
        let p = secrets_path("org-1", "bot-1", "conn-1");
        assert_eq!(p, "gbo/orgs/org-1/bots/bot-1/sources/conn-1");
    }

    #[test]
    fn test_validate_schedule() {
        assert!(validate_schedule(None).is_ok());
        assert!(validate_schedule(Some("0 * * * *")).is_ok());
        assert!(validate_schedule(Some("*/15 * * * *")).is_ok());
        assert!(validate_schedule(Some("0 9 * * 1-5")).is_ok());
        assert!(validate_schedule(Some("0 9 * *")).is_err());
        assert!(validate_schedule(Some("bad * * * *")).is_err());
    }
}
