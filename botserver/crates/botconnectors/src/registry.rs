use crate::knowledge::{ChatConnector, DriveConnector, MailConnector};
use botcoresecrets::SecretsManager;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub type CredentialsResolver = Arc<dyn Fn(&str) -> Result<Value, String> + Send + Sync>;

const VAULT_MOUNT_PREFIX: &str = "secret/";

fn build_registry() -> HashMap<&'static str, Arc<dyn crate::knowledge::KnowledgeConnector>> {
    let mut map: HashMap<&'static str, Arc<dyn crate::knowledge::KnowledgeConnector>> =
        HashMap::new();
    for connector in
        [Arc::new(ChatConnector) as Arc<dyn crate::knowledge::KnowledgeConnector>,
         Arc::new(MailConnector) as Arc<dyn crate::knowledge::KnowledgeConnector>,
         Arc::new(DriveConnector) as Arc<dyn crate::knowledge::KnowledgeConnector>]
    {
        map.insert(connector.kind(), connector);
    }
    map
}

fn registry() -> &'static HashMap<&'static str, Arc<dyn crate::knowledge::KnowledgeConnector>> {
    static REGISTRY: OnceLock<HashMap<&'static str, Arc<dyn crate::knowledge::KnowledgeConnector>>> =
        OnceLock::new();
    REGISTRY.get_or_init(build_registry)
}

/// Resolve the connector registered under a kind string ("chat", "mail", "drive").
pub fn connector_for_kind(
    kind: &str,
) -> Option<Arc<dyn crate::knowledge::KnowledgeConnector>> {
    registry().get(kind).cloned()
}

/// List the kinds currently registered.
pub fn registered_kinds() -> Vec<&'static str> {
    let mut kinds: Vec<&'static str> = registry().keys().copied().collect();
    kinds.sort_unstable();
    kinds
}

/// Normalize a logical Vault path: strips an optional leading mount segment so
/// both "secret/gbo/..." and "gbo/..." address the same secret via kv2.
fn normalize_vault_path(vault_token_ref: &str) -> String {
    vault_token_ref
        .trim()
        .trim_start_matches(VAULT_MOUNT_PREFIX)
        .to_string()
}

/// Default credential resolver backed by `botcoresecrets`: reads the KV v2
/// secret at the connection's `vault_token_ref` and returns it as a JSON object.
pub async fn resolve_credentials(vault_token_ref: &str) -> Result<Value, String> {
    let manager =
        SecretsManager::get_clone().map_err(|e| format!("Vault manager unavailable: {e}"))?;
    let data = manager
        .get_secret(&normalize_vault_path(vault_token_ref))
        .await
        .map_err(|e| format!("Vault read failed for connector credentials: {e}"))?;
    Ok(Value::Object(
        data.into_iter().map(|(k, v)| (k, Value::String(v))).collect(),
    ))
}

/// Persist a credentials JSON object into Vault at the given logical path.
/// Non-string values are serialized to their JSON representation.
pub async fn store_credentials(path: &str, credentials: &Value) -> Result<(), String> {
    let object = credentials
        .as_object()
        .ok_or_else(|| "Credentials must be a JSON object".to_string())?;
    let flat: HashMap<String, String> = object
        .iter()
        .map(|(k, v)| match v.as_str() {
            Some(s) => (k.clone(), s.to_string()),
            None => (k.clone(), v.to_string()),
        })
        .collect();
    let manager =
        SecretsManager::get_clone().map_err(|e| format!("Vault manager unavailable: {e}"))?;
    manager
        .put_secret(&normalize_vault_path(path), flat)
        .await
        .map_err(|e| format!("Vault write failed for connector credentials: {e}"))
}

/// Best-effort deletion of the credential secret when a connection is removed.
pub async fn delete_credentials(path: &str) {
    let normalized = normalize_vault_path(path);
    match SecretsManager::get_clone() {
        Ok(manager) => {
            if let Err(e) = manager.delete_secret(&normalized).await {
                tracing::warn!("botconnectors: Vault delete failed for '{path}': {e}");
            }
        }
        Err(e) => tracing::warn!("botconnectors: Vault unavailable for delete of '{path}': {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_kinds_registered() {
        assert_eq!(registered_kinds(), vec!["chat", "drive", "mail"]);
        assert!(connector_for_kind("chat").is_some());
        assert!(connector_for_kind("mail").is_some());
        assert!(connector_for_kind("drive").is_some());
        assert!(connector_for_kind("calendar").is_none());
    }

    #[test]
    fn normalizes_vault_mount_prefix() {
        assert_eq!(normalize_vault_path("secret/gbo/connectors/o/i"), "gbo/connectors/o/i");
        assert_eq!(normalize_vault_path("gbo/connectors/o/i"), "gbo/connectors/o/i");
    }
}
