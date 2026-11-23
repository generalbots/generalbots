//! #2833 — fail-closed plugin security primitives.
//!
//! A manifest is trusted only when it carries a valid HMAC-SHA256 signature
//! over its canonical JSON, its `base_url` host is allowlisted and its manifest
//! path stays inside the bot work directory. Credentials are resolved from
//! Vault at call time through `auth_vault_path`; no manifest field holds a
//! secret value. The registry is initialized empty, so a caller cannot reach a
//! plugin until all of these gates have passed.

use base64::Engine as _;
use botcoresecrets::SecretsManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[path = "plugin_signature.rs"]
mod plugin_signature;

pub use plugin_signature::{
    decode_signing_key, is_path_within, sign_manifest_json, sign_manifest_json_with_hex_key,
    verify_manifest_signature, verify_manifest_signature_with_hex_key,
};

/// Permission a manifest must grant before `PLUGIN_CALL` will execute it.
pub const PLUGIN_CALL_PERMISSION: &str = "plugin:call";

/// Vault KV v2 field names. The manifest carries only the Vault *path*, never
/// the secret value (#2833).
pub const VAULT_FIELD_API_KEY: &str = "key";
pub const VAULT_FIELD_TOKEN: &str = "token";
pub const VAULT_FIELD_USERNAME: &str = "username";
pub const VAULT_FIELD_PASSWORD: &str = "password";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    None,
    /// Only the header *name* is stored. The value comes from Vault.
    ApiKey { header_name: String },
    BearerToken,
    BasicAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposedFunction {
    pub name: String,
    pub description: Option<String>,
    pub method: String,
    pub path: String,
    pub request_body: Option<serde_json::Value>,
    pub parameters: Vec<FunctionParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub base_url: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub authentication: AuthType,
    pub functions: Vec<ExposedFunction>,
    pub bot_id: Option<String>,
    /// Vault KV v2 path (without the `secret/` mount prefix) holding the auth
    /// fields. Never a credential itself (#2833).
    #[serde(default)]
    pub auth_vault_path: Option<String>,
    /// Relative path of the manifest inside the bot work directory. Used for
    /// the containment gate at registration and call time (#2833).
    #[serde(default)]
    pub source_path: Option<String>,
    /// Hex HMAC-SHA256 over the canonical JSON of this manifest with this
    /// field removed. Required by `register()` and `PLUGIN_CALL` (#2833).
    #[serde(default)]
    pub signature: Option<String>,
}

impl PluginManifest {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Canonical signature payload: every field serialized in declaration
    /// order with `signature` removed. Signing this form binds the signature
    /// to `base_url`, `permissions`, `bot_id` and `source_path`, so tampering
    /// with any of them invalidates it.
    pub fn canonical_json(&self) -> Result<String, PluginError> {
        let mut value = serde_json::to_value(self)
            .map_err(|e| PluginError::ParseError(format!("manifest serialization: {e}")))?;
        if let Some(object) = value.as_object_mut() {
            object.remove("signature");
        }
        serde_json::to_string(&value)
            .map_err(|e| PluginError::ParseError(format!("manifest canonicalization: {e}")))
    }

    /// Fail-closed signature check using `Mac::verify_slice`, which compares
    /// in constant time. Missing, malformed and wrong signatures are rejected.
    pub fn verify_signature(&self, key: &[u8]) -> Result<(), PluginError> {
        let signature = self.signature.as_deref().ok_or_else(|| {
            PluginError::AuthError("plugin manifest is unsigned".to_string())
        })?;
        let canonical = self.canonical_json()?;
        verify_manifest_signature(&canonical, signature, key)
    }

    /// HTTPS is mandatory and the host must be in the configured allowlist.
    pub fn is_host_allowed(&self, allowed_hosts: &[String]) -> Result<(), PluginError> {
        let url = reqwest::Url::parse(&self.base_url)
            .map_err(|_| PluginError::AuthError("plugin base_url is not a valid URL".to_string()))?;
        if url.scheme() != "https" {
            return Err(PluginError::AuthError("plugin base_url must use HTTPS".to_string()));
        }
        if url.password().is_some() || !url.username().is_empty() {
            return Err(PluginError::AuthError(
                "plugin base_url must not embed credentials".to_string(),
            ));
        }
        let host = url.host_str()
            .ok_or_else(|| PluginError::AuthError("plugin base_url has no host".to_string()))?
            .to_ascii_lowercase();
        if allowed_hosts.iter().any(|h| h.eq_ignore_ascii_case(&host)) {
            Ok(())
        } else {
            Err(PluginError::AuthError(format!(
                "plugin host '{host}' is not allowlisted"
            )))
        }
    }

    /// Builds a request, resolving auth secrets from Vault via
    /// `auth_vault_path`. Async because Vault access is async; no secret is
    /// ever read from the manifest itself.
    pub async fn build_request(&self, fn_name: &str) -> Result<PluginRequest, PluginError> {
        let func = self.functions.iter().find(|f| f.name == fn_name)
            .ok_or_else(|| PluginError::FunctionNotFound(fn_name.to_string()))?;
        if !func.path.starts_with('/') || func.path.contains("..") {
            return Err(PluginError::AuthError("plugin function path is invalid".to_string()));
        }
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), func.path);
        let parsed = reqwest::Url::parse(&url)
            .map_err(|_| PluginError::AuthError("plugin request URL is invalid".to_string()))?;
        if parsed.scheme() != "https" || parsed.host_str().is_none() {
            return Err(PluginError::AuthError(
                "plugin request URL must be HTTPS with a host".to_string(),
            ));
        }

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        match &self.authentication {
            AuthType::None => {}
            AuthType::ApiKey { header_name } => {
                if header_name.is_empty() || header_name.chars().any(|c| matches!(c, '\r' | '\n' | ':')) {
                    return Err(PluginError::AuthError("invalid API key header name".to_string()));
                }
                let value = self.vault_secret(VAULT_FIELD_API_KEY).await?;
                headers.insert(header_name.clone(), value);
            }
            AuthType::BearerToken => {
                let value = self.vault_secret(VAULT_FIELD_TOKEN).await?;
                headers.insert("Authorization".to_string(), format!("Bearer {value}"));
            }
            AuthType::BasicAuth => {
                let user = self.vault_secret(VAULT_FIELD_USERNAME).await?;
                let password = self.vault_secret(VAULT_FIELD_PASSWORD).await?;
                let credentials = base64::engine::general_purpose::STANDARD
                    .encode(format!("{user}:{password}"));
                headers.insert("Authorization".to_string(), format!("Basic {credentials}"));
            }
        }

        Ok(PluginRequest {
            url,
            method: func.method.clone(),
            headers,
            body: func.request_body.clone(),
            function_name: fn_name.to_string(),
        })
    }

    /// Strict Vault read: no env fallback for plugin credentials, fail closed.
    async fn vault_secret(&self, field: &str) -> Result<String, PluginError> {
        let path = self.auth_vault_path.as_deref().ok_or_else(|| {
            PluginError::AuthError("plugin auth_vault_path is required".to_string())
        })?;
        let manager = SecretsManager::get()
            .map_err(|e| PluginError::AuthError(format!("Vault unavailable: {e}")))?;
        let secrets = manager.get_secret_strict(path).await.map_err(|e| {
            PluginError::AuthError(format!("Vault read failed for plugin auth: {e}"))
        })?;
        secrets.get(field).cloned().filter(|v| !v.is_empty()).ok_or_else(|| {
            PluginError::AuthError(format!("plugin auth field '{field}' missing in Vault"))
        })
    }
}

#[derive(Debug, Clone)]
pub struct PluginSecurityPolicy {
    /// Exact hostnames allowed as plugin `base_url` hosts. Empty denies all.
    pub allowed_hosts: Vec<String>,
    /// Hex HMAC-SHA256 signing key. Unset means no plugin can be registered.
    pub signing_key_hex: Option<String>,
    /// Root of the bot work directory; manifest paths must resolve under it.
    pub work_root: PathBuf,
}

impl PluginSecurityPolicy {
    /// Loads the fail-closed policy from server-side configuration.
    pub fn from_env() -> Self {
        let allowed_hosts = std::env::var("GB_PLUGIN_ALLOWED_HOSTS")
            .unwrap_or_else(|_| String::new())
            .split(',')
            .map(str::trim)
            .filter(|host| !host.is_empty())
            .map(|host| host.to_ascii_lowercase())
            .collect();
        let signing_key_hex = std::env::var("GB_PLUGIN_SIGNING_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let work_root = PathBuf::from(
            std::env::var("GB_WORK_PATH")
                .or_else(|_| std::env::var("WORK_PATH"))
                .unwrap_or_else(|_| "/opt/gbo/work".to_string()),
        );
        Self {
            allowed_hosts,
            signing_key_hex,
            work_root,
        }
    }

    fn signing_key(&self) -> Result<Vec<u8>, PluginError> {
        let hex_key = self.signing_key_hex.as_deref().ok_or_else(|| {
            PluginError::AuthError("plugin signing key is not configured".to_string())
        })?;
        decode_signing_key(hex_key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub function_name: String,
}

#[derive(Debug, Clone)]
pub enum PluginError {
    FunctionNotFound(String),
    HttpError(String),
    ParseError(String),
    AuthError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::FunctionNotFound(name) => write!(f, "Plugin function not found: {name}"),
            PluginError::HttpError(msg) => write!(f, "Plugin HTTP error: {msg}"),
            PluginError::ParseError(msg) => write!(f, "Plugin parse error: {msg}"),
            PluginError::AuthError(msg) => write!(f, "Plugin auth error: {msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, PluginManifest>>,
    policy: PluginSecurityPolicy,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::with_policy(PluginSecurityPolicy::from_env())
    }

    pub fn with_policy(policy: PluginSecurityPolicy) -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            policy,
        }
    }

    pub fn policy(&self) -> &PluginSecurityPolicy {
        &self.policy
    }

    /// Registers a manifest only after signature, host and work-dir gates pass.
    pub async fn register(&self, manifest: PluginManifest) -> Result<(), PluginError> {
        let key = self.policy.signing_key()?;
        manifest.verify_signature(&key)?;
        manifest.is_host_allowed(&self.policy.allowed_hosts)?;
        let source = manifest.source_path.as_deref().ok_or_else(|| {
            PluginError::AuthError("plugin source_path is required".to_string())
        })?;
        if !is_path_within(&self.policy.work_root, source) {
            return Err(PluginError::AuthError(
                "plugin manifest path escapes the work directory".to_string(),
            ));
        }
        let mut plugins = self.plugins.write().await;
        plugins.insert(manifest.name.clone(), manifest);
        Ok(())
    }

    pub async fn unregister(&self, name: &str) {
        let mut plugins = self.plugins.write().await;
        plugins.remove(name);
    }

    pub async fn get(&self, name: &str) -> Option<PluginManifest> {
        let plugins = self.plugins.read().await;
        plugins.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<PluginManifest> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    pub async fn len(&self) -> usize {
        let plugins = self.plugins.read().await;
        plugins.len()
    }

    /// Returns `true` when no plugin manifest is registered.
    pub async fn is_empty(&self) -> bool {
        let plugins = self.plugins.read().await;
        plugins.is_empty()
    }

    /// Every gate a caller (`PLUGIN_CALL`) must pass before using a registered
    /// plugin: signature, bot scope, work-dir containment, permission and host
    /// allowlist. Returns the authorized manifest or a fail-closed error.
    pub async fn authorize(&self, name: &str, bot_id: &str) -> Result<PluginManifest, PluginError> {
        let manifest = self.get(name).await.ok_or_else(|| {
            PluginError::AuthError(format!("plugin '{name}' is not installed"))
        })?;
        let key = self.policy.signing_key()?;
        manifest.verify_signature(&key)?;
        manifest.is_host_allowed(&self.policy.allowed_hosts)?;
        if !manifest.has_permission(PLUGIN_CALL_PERMISSION) {
            return Err(PluginError::AuthError(format!(
                "plugin '{name}' lacks the {PLUGIN_CALL_PERMISSION} permission"
            )));
        }
        match manifest.bot_id.as_deref() {
            Some(id) if id == bot_id => {}
            _ => {
                return Err(PluginError::AuthError(format!(
                    "plugin '{name}' is not installed for this bot"
                )))
            }
        }
        let source = manifest.source_path.as_deref().ok_or_else(|| {
            PluginError::AuthError("plugin source_path is missing".to_string())
        })?;
        if !is_path_within(&self.policy.work_root, source) {
            return Err(PluginError::AuthError(
                "plugin manifest path escapes the work directory".to_string(),
            ));
        }
        Ok(manifest)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedPluginRegistry = Arc<PluginRegistry>;

use std::sync::OnceLock;

static GLOBAL_PLUGIN_REGISTRY: OnceLock<SharedPluginRegistry> = OnceLock::new();

pub fn init_global_registry() -> SharedPluginRegistry {
    init_global_registry_with_policy(PluginSecurityPolicy::from_env())
}

/// Boot entry point: installs one registry with an explicit, fail-closed policy.
pub fn init_global_registry_with_policy(policy: PluginSecurityPolicy) -> SharedPluginRegistry {
    let registry = Arc::new(PluginRegistry::with_policy(policy));
    if GLOBAL_PLUGIN_REGISTRY.set(registry.clone()).is_err() {
        log::warn!("plugin registry already initialized; ignoring duplicate boot call");
    }
    registry
}

pub fn global_registry() -> Option<&'static SharedPluginRegistry> {
    GLOBAL_PLUGIN_REGISTRY.get()
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod tests;
