pub mod handlers;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    None,
    ApiKey,
    Bearer,
    Basic,
}

impl AuthType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ApiKey => "api_key",
            Self::Bearer => "bearer",
            Self::Basic => "basic",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s {
            "api_key" | "apikey" => Self::ApiKey,
            "bearer" => Self::Bearer,
            "basic" => Self::Basic,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposedFunction {
    pub name: String,
    pub description: String,
    pub params: Vec<FunctionParam>,
    pub return_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub base_url: String,
    pub permissions: Vec<String>,
    pub auth_type: AuthType,
    pub auth_vault_path: Option<String>,
    pub functions: Vec<ExposedFunction>,
    pub enabled: bool,
    /// Hex HMAC-SHA256 over the canonical JSON of this manifest with this
    /// field removed. Required for the manifest to load (#2833).
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStateEntry {
    pub manifest: PluginManifest,
    pub source_bucket: String,
    pub last_loaded: String,
}

impl PluginManifest {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    pub fn get_function(&self, name: &str) -> Option<&ExposedFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Canonical signature payload. `enabled` is covered by the same signature,
    /// so flipping it in Drive without re-signing cannot enable a plugin (#2833).
    pub fn canonical_json_without_signature(&self) -> Result<String, String> {
        let mut value = serde_json::to_value(self)
            .map_err(|e| format!("manifest serialization error: {e}"))?;
        if let Some(object) = value.as_object_mut() {
            object.remove("signature");
        }
        serde_json::to_string(&value)
            .map_err(|e| format!("manifest canonicalization error: {e}"))
    }

    /// Authenticated plugins must name a Vault path; no inline credential is
    /// part of this schema (#2833).
    pub fn has_vault_auth_source(&self) -> bool {
        match self.auth_type {
            AuthType::None => true,
            _ => self
                .auth_vault_path
                .as_deref()
                .map_or(false, |path| !path.trim().is_empty()),
        }
    }
}

pub const DEFAULT_PLUGINS_PREFIX: &str = "plugins";

/// Loads signed manifests using the server-side signing key from
/// `GB_PLUGIN_SIGNING_KEY`. An unset key rejects every manifest (fail closed).
pub fn load_manifests_from_drive(
    bucket_name: &str,
    load_from_drive: &crate::LoadFromDriveFn,
) -> Result<Vec<PluginStateEntry>, String> {
    let signing_key_hex =
        std::env::var("GB_PLUGIN_SIGNING_KEY").unwrap_or_else(|_| String::new());
    load_manifests_from_drive_with_key(bucket_name, load_from_drive, &signing_key_hex)
}

/// Same loader with an explicit key, so the signature gate can be tested
/// without mutating process-wide environment variables.
pub fn load_manifests_from_drive_with_key(
    bucket_name: &str,
    load_from_drive: &crate::LoadFromDriveFn,
    signing_key_hex: &str,
) -> Result<Vec<PluginStateEntry>, String> {
    let prefix = format!("{}/", DEFAULT_PLUGINS_PREFIX);
    let mut plugins = Vec::new();

    let entries_result = load_from_drive(bucket_name, &prefix);
    let entries = match entries_result {
        Ok(e) => e,
        Err(e) => {
            log::warn!("No plugin directory found in bucket {}: {}", bucket_name, e);
            return Ok(Vec::new());
        }
    };

    for line in entries.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.ends_with(".json") {
            continue;
        }

        let manifest_path = format!("{}{}", prefix, trimmed);
        match load_from_drive(bucket_name, &manifest_path) {
            Ok(json_str) => {
                match parse_manifest(&json_str) {
                    Ok(manifest) => {
                        // #2833: verify before load. The signature covers the
                        // whole manifest (including `enabled` and `base_url`),
                        // so an unsigned or tampered file never enters state.
                        let canonical = match manifest.canonical_json_without_signature() {
                            Ok(value) => value,
                            Err(e) => {
                                log::error!(
                                    "Cannot canonicalize plugin manifest {}: {}",
                                    manifest_path,
                                    e
                                );
                                continue;
                            }
                        };
                        let signature = manifest.signature.as_deref().unwrap_or("");
                        if let Err(e) = botcorepkg::plugin::verify_manifest_signature_with_hex_key(
                            &canonical,
                            signature,
                            signing_key_hex,
                        ) {
                            log::error!(
                                "Rejected unsigned or tampered plugin manifest {}: {}",
                                manifest_path,
                                e
                            );
                            continue;
                        }
                        if !manifest.has_vault_auth_source() {
                            log::error!(
                                "Rejected plugin manifest {}: authenticated plugins require auth_vault_path",
                                manifest_path
                            );
                            continue;
                        }
                        log::info!("Loaded signed plugin manifest: {} v{}", manifest.name, manifest.version);
                        plugins.push(PluginStateEntry {
                            manifest,
                            source_bucket: bucket_name.to_string(),
                            last_loaded: chrono::Utc::now().to_rfc3339(),
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to parse plugin manifest {}: {}", manifest_path, e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to load manifest file {}: {}", manifest_path, e);
            }
        }
    }

    Ok(plugins)
}

pub fn parse_manifest(json_str: &str) -> Result<PluginManifest, String> {
    let mut manifest: PluginManifest =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    if manifest.name.is_empty() {
        return Err("Plugin name is required".to_string());
    }
    if manifest.version.is_empty() {
        return Err("Plugin version is required".to_string());
    }

    if manifest.id.is_nil() {
        manifest.id = Uuid::new_v4();
    }

    Ok(manifest)
}

pub fn get_active_plugins(plugins: &[PluginStateEntry]) -> Vec<&PluginStateEntry> {
    plugins.iter().filter(|p| p.manifest.enabled).collect()
}

pub fn get_plugins_by_permission<'a>(
    plugins: &'a [PluginStateEntry],
    permission: &str,
) -> Vec<&'a PluginStateEntry> {
    plugins
        .iter()
        .filter(|p| p.manifest.enabled && p.manifest.has_permission(permission))
        .collect()
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub use handlers::configure_plugin_routes;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    const KEY_HEX: &str = "746573742d7369676e696e672d6b6579";

    fn manifest(signature: Option<String>, auth_type: AuthType) -> PluginManifest {
        PluginManifest {
            id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            name: "sample".to_string(),
            version: "1.0.0".to_string(),
            description: "sample plugin".to_string(),
            base_url: "https://api.example.com".to_string(),
            permissions: vec!["plugin:call".to_string()],
            auth_type,
            auth_vault_path: None,
            functions: Vec::new(),
            enabled: false,
            signature,
        }
    }

    fn signed_json(auth_type: AuthType) -> String {
        let unsigned = manifest(None, auth_type.clone());
        let canonical = unsigned.canonical_json_without_signature().unwrap();
        let signature =
            botcorepkg::plugin::sign_manifest_json_with_hex_key(&canonical, KEY_HEX).unwrap();
        serde_json::to_string(&manifest(Some(signature), auth_type)).unwrap()
    }

    fn loader_for(json: String) -> crate::LoadFromDriveFn {
        Arc::new(move |_bucket: &str, path: &str| -> Result<String, String> {
            match path {
                "plugins/" => Ok("sample.json\n".to_string()),
                "plugins/sample.json" => Ok(json.clone()),
                _ => Err("not found".to_string()),
            }
        })
    }

    #[test]
    fn signed_manifest_loads() {
        let loader = loader_for(signed_json(AuthType::None));
        let loaded = load_manifests_from_drive_with_key("bot", &loader, KEY_HEX).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].manifest.name, "sample");
    }

    #[test]
    fn unsigned_and_tampered_manifests_are_rejected() {
        let unsigned = serde_json::to_string(&manifest(None, AuthType::None)).unwrap();
        let loader = loader_for(unsigned);
        assert!(load_manifests_from_drive_with_key("bot", &loader, KEY_HEX)
            .unwrap()
            .is_empty(), "unsigned manifest must be rejected");

        // Flip `enabled` after signing: the signature no longer covers the
        // stored bytes, so the manifest must not load.
        let signed = signed_json(AuthType::None);
        let mut value: serde_json::Value = serde_json::from_str(&signed).unwrap();
        value["enabled"] = serde_json::json!(true);
        let loader = loader_for(value.to_string());
        assert!(load_manifests_from_drive_with_key("bot", &loader, KEY_HEX)
            .unwrap()
            .is_empty(), "tampered enabled flag must be rejected");

        // A wrong signing key must also fail closed.
        let loader = loader_for(signed_json(AuthType::None));
        assert!(load_manifests_from_drive_with_key("bot", &loader, "77616f6e67")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn authenticated_manifest_requires_vault_path() {
        let loader = loader_for(signed_json(AuthType::ApiKey));
        assert!(load_manifests_from_drive_with_key("bot", &loader, KEY_HEX)
            .unwrap()
            .is_empty(), "auth without auth_vault_path must be rejected");
    }
}
