use std::collections::HashMap;

use botcoresecrets::manager::SecretsManager;
use serde_json::Value;
use uuid::Uuid;

use crate::error::IntegrationError;
use crate::scope::ConnectionScope;

/// Wrapper over the platform secrets manager dedicated to integration
/// connection credentials (#939).
///
/// Every operation fails closed: any Vault failure becomes
/// [`IntegrationError::VaultUnavailable`] and never falls back to environment
/// defaults or empty data.
#[derive(Clone)]
pub struct ConnectionVault {
    manager: SecretsManager,
}

/// Builds the canonical Vault path for a connection credential envelope:
/// `gbo/{org}/{branch}/{bot}/integrations/{owner}/{connection}`.
pub fn build_connection_path(scope: &ConnectionScope, connection_id: Uuid) -> String {
    format!(
        "gbo/{}/{}/{}/integrations/{}/{}",
        scope.org_id,
        scope.branch_id,
        scope.bot_id,
        scope.owner_user_id(),
        connection_id
    )
}

/// Candidate Vault paths for a provider's OAuth client, most specific first:
/// per bot, per branch, then the platform-wide client.
///
/// The platform entry is the one an operator registers once
/// (`integrations/oauth/{provider}`); a bot or branch entry overrides it for a
/// tenant that runs its own OAuth app. Users never handle client ids.
pub fn oauth_client_paths(
    org_id: Uuid,
    branch_id: Uuid,
    bot_id: Uuid,
    provider: &str,
) -> [String; 3] {
    [
        format!("gbo/{org_id}/{branch_id}/{bot_id}/integrations/oauth/{provider}"),
        format!("gbo/{org_id}/{branch_id}/integrations/oauth/{provider}"),
        format!("integrations/oauth/{provider}"),
    ]
}

fn entries_to_value(entries: std::collections::HashMap<String, String>) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key, Value::String(value)))
            .collect(),
    )
}

impl ConnectionVault {
    pub fn new(manager: SecretsManager) -> Self {
        Self { manager }
    }

    /// Persists the secret envelope for a connection at its canonical path
    /// and returns the path that was written.
    pub async fn store(
        &self,
        scope: &ConnectionScope,
        connection_id: Uuid,
        secrets: &Value,
    ) -> Result<String, IntegrationError> {
        let path = build_connection_path(scope, connection_id);
        let entries = flatten_secrets(secrets)?;
        self.manager.put_secret(&path, entries).await.map_err(|e| {
            log::error!("vault store failed for connection {connection_id}: {e}");
            IntegrationError::VaultUnavailable
        })?;
        Ok(path)
    }

    /// Reads the secret envelope strictly from Vault - no environment or
    /// built-in fallbacks are ever consulted.
    pub async fn load_strict(&self, path: &str) -> Result<Value, IntegrationError> {
        let entries = self.manager.get_secret_strict(path).await.map_err(|e| {
            log::error!("vault strict load failed for {path}: {e}");
            IntegrationError::VaultUnavailable
        })?;
        let map: serde_json::Map<String, Value> = entries
            .into_iter()
            .map(|(key, value)| (key, Value::String(value)))
            .collect();
        Ok(Value::Object(map))
    }

    /// Resolves a provider's OAuth client configuration, most specific first
    /// (bot → branch → platform). Returns the value and the path that
    /// answered, so callers can log which scope the client came from.
    pub async fn load_oauth_client(
        &self,
        org_id: Uuid,
        branch_id: Uuid,
        bot_id: Uuid,
        provider: &str,
    ) -> Result<(Value, String), IntegrationError> {
        for path in oauth_client_paths(org_id, branch_id, bot_id, provider) {
            if let Ok(entries) = self.manager.get_secret(&path).await {
                if !entries.is_empty() {
                    return Ok((entries_to_value(entries), path));
                }
            }
        }
        Err(IntegrationError::NotFound)
    }

    /// Stores a provider's OAuth client for a branch (the "configure once"
    /// step for a self-hosted install). Returns the path written.
    pub async fn put_provider_client(
        &self,
        org_id: Uuid,
        branch_id: Uuid,
        provider: &str,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<String, IntegrationError> {
        let path = format!("gbo/{org_id}/{branch_id}/integrations/oauth/{provider}");
        let mut entries: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        entries.insert("client_id".to_string(), client_id.trim().to_string());
        if let Some(secret) = client_secret.map(str::trim).filter(|s| !s.is_empty()) {
            entries.insert("client_secret".to_string(), secret.to_string());
        }
        self.manager.put_secret(&path, entries).await.map_err(|e| {
            log::error!("vault store failed for oauth client {path}: {e}");
            IntegrationError::VaultUnavailable
        })?;
        Ok(path)
    }

    /// Removes the secret envelope. Failures are fail closed.
    pub async fn delete(&self, path: &str) -> Result<(), IntegrationError> {
        self.manager.delete_secret(path).await.map_err(|e| {
            log::error!("vault delete failed for {path}: {e}");
            IntegrationError::VaultUnavailable
        })
    }
}

/// Converts a JSON object into the flat key/value map Vault expects.
fn flatten_secrets(secrets: &Value) -> Result<HashMap<String, String>, IntegrationError> {
    let object = match secrets {
        Value::Object(map) => map,
        _ => {
            return Err(IntegrationError::Validation(
                "secret payload must be a JSON object".to_string(),
            ))
        }
    };
    let mut entries = HashMap::new();
    for (key, value) in object {
        if key.trim().is_empty() {
            return Err(IntegrationError::Validation(
                "secret keys must not be empty".to_string(),
            ));
        }
        let rendered = match value {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        };
        entries.insert(key.clone(), rendered);
    }
    Ok(entries)
}

#[cfg(test)]
mod oauth_client_path_tests {
    use super::oauth_client_paths;
    use uuid::Uuid;

    #[test]
    fn platform_client_is_the_last_resort_and_the_first_match_wins() {
        let org = Uuid::from_u128(1);
        let branch = Uuid::from_u128(2);
        let bot = Uuid::from_u128(3);
        let paths = oauth_client_paths(org, branch, bot, "gmail");
        assert!(paths[0].contains(&format!("{bot}/integrations/oauth/gmail")));
        assert!(paths[0].starts_with(&format!("gbo/{org}/{branch}/{bot}/")));
        assert!(paths[1].starts_with(&format!("gbo/{org}/{branch}/")));
        assert!(!paths[1].contains(&bot.to_string()), "branch path skips the bot id");
        assert_eq!(paths[2], "integrations/oauth/gmail");
    }
}
