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
