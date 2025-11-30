//! Secrets Management Module
//!
//! Provides integration with HashiCorp Vault for secure secrets management.
//! Secrets are fetched from Vault at runtime, keeping .env minimal with only
//! VAULT_ADDR and VAULT_TOKEN.
//!
//! With Vault, .env contains ONLY:
//! - VAULT_ADDR - Vault server address
//! - VAULT_TOKEN - Vault authentication token
//!
//! Everything else is stored in Vault:
//!
//! Vault paths:
//! - gbo/directory - Zitadel connection (url, project_id, client_id, client_secret)
//! - gbo/tables - PostgreSQL credentials (host, port, database, username, password)
//! - gbo/drive - MinIO/S3 credentials (endpoint, accesskey, secret)
//! - gbo/cache - Redis credentials (host, port, password)
//! - gbo/email - Stalwart credentials (host, username, password)
//! - gbo/llm - LLM API keys (openai_key, anthropic_key, groq_key, deepseek_key)
//! - gbo/encryption - Encryption keys (master_key, data_key)
//! - gbo/meet - LiveKit credentials (url, api_key, api_secret)
//! - gbo/alm - Forgejo credentials (url, admin_password, runner_token)
//! - gbo/vectordb - Qdrant credentials (url, api_key)
//! - gbo/observability - InfluxDB credentials (url, org, token)

use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Secret paths in Vault
pub struct SecretPaths;

impl SecretPaths {
    /// Directory service (Zitadel) - url, project_id, client_id, client_secret
    pub const DIRECTORY: &'static str = "gbo/directory";
    /// Database (PostgreSQL) - host, port, database, username, password
    pub const TABLES: &'static str = "gbo/tables";
    /// Object storage (MinIO) - endpoint, accesskey, secret
    pub const DRIVE: &'static str = "gbo/drive";
    /// Cache (Redis) - host, port, password
    pub const CACHE: &'static str = "gbo/cache";
    /// Email (Stalwart) - host, username, password
    pub const EMAIL: &'static str = "gbo/email";
    /// LLM providers - openai_key, anthropic_key, groq_key, deepseek_key, mistral_key
    pub const LLM: &'static str = "gbo/llm";
    /// Encryption - master_key, data_key
    pub const ENCRYPTION: &'static str = "gbo/encryption";
    /// Video meetings (LiveKit) - url, api_key, api_secret
    pub const MEET: &'static str = "gbo/meet";
    /// ALM (Forgejo) - url, admin_password, runner_token
    pub const ALM: &'static str = "gbo/alm";
    /// Vector database (Qdrant) - url, api_key
    pub const VECTORDB: &'static str = "gbo/vectordb";
    /// Observability (InfluxDB) - url, org, bucket, token
    pub const OBSERVABILITY: &'static str = "gbo/observability";
}

/// Vault configuration
///
/// .env should contain ONLY these two variables:
/// - VAULT_ADDR=https://localhost:8200
/// - VAULT_TOKEN=hvs.xxxxxxxxxxxxx
///
/// All other configuration is fetched from Vault.
#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Vault server address (e.g., https://localhost:8200)
    pub addr: String,
    /// Vault authentication token
    pub token: String,
    /// Skip TLS verification (for self-signed certs)
    pub skip_verify: bool,
    /// Cache TTL in seconds (0 = no caching)
    pub cache_ttl: u64,
    /// Namespace (for Vault Enterprise)
    pub namespace: Option<String>,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            addr: env::var("VAULT_ADDR").unwrap_or_else(|_| "https://localhost:8200".to_string()),
            token: env::var("VAULT_TOKEN").unwrap_or_default(),
            skip_verify: env::var("VAULT_SKIP_VERIFY")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            cache_ttl: env::var("VAULT_CACHE_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            namespace: env::var("VAULT_NAMESPACE").ok(),
        }
    }
}

/// Cached secret with expiry
#[derive(Debug, Clone)]
struct CachedSecret {
    data: HashMap<String, String>,
    expires_at: std::time::Instant,
}

/// Vault response structures
#[derive(Debug, Deserialize)]
struct VaultResponse {
    data: VaultData,
}

#[derive(Debug, Deserialize)]
struct VaultData {
    data: HashMap<String, serde_json::Value>,
}

/// Secrets manager service
#[derive(Clone)]
pub struct SecretsManager {
    config: VaultConfig,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CachedSecret>>>,
    enabled: bool,
}

impl SecretsManager {
    /// Create a new secrets manager
    pub fn new(config: VaultConfig) -> Result<Self> {
        let enabled = !config.token.is_empty() && !config.addr.is_empty();

        if !enabled {
            warn!("Vault not configured (VAULT_ADDR or VAULT_TOKEN missing). Using environment variables directly.");
        }

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.skip_verify)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            config,
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        })
    }

    /// Create with default configuration from environment
    pub fn from_env() -> Result<Self> {
        Self::new(VaultConfig::default())
    }

    /// Check if Vault is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get a secret from Vault
    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        if !self.enabled {
            return self.get_from_env(path);
        }

        // Check cache first
        if let Some(cached) = self.get_cached(path).await {
            trace!("Secret '{}' found in cache", path);
            return Ok(cached);
        }

        // Fetch from Vault
        let secret = self.fetch_from_vault(path).await?;

        // Cache the result
        if self.config.cache_ttl > 0 {
            self.cache_secret(path, secret.clone()).await;
        }

        Ok(secret)
    }

    /// Get a single value from a secret path
    pub async fn get_value(&self, path: &str, key: &str) -> Result<String> {
        let secret = self.get_secret(path).await?;
        secret
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Key '{}' not found in secret '{}'", key, path))
    }

    /// Get drive credentials
    pub async fn get_drive_credentials(&self) -> Result<(String, String)> {
        let secret = self.get_secret(SecretPaths::DRIVE).await?;
        Ok((
            secret.get("accesskey").cloned().unwrap_or_default(),
            secret.get("secret").cloned().unwrap_or_default(),
        ))
    }

    /// Get database credentials
    pub async fn get_database_credentials(&self) -> Result<(String, String)> {
        let secret = self.get_secret(SecretPaths::TABLES).await?;
        Ok((
            secret
                .get("username")
                .cloned()
                .unwrap_or_else(|| "gbuser".to_string()),
            secret.get("password").cloned().unwrap_or_default(),
        ))
    }

    /// Get cache (Redis) password
    pub async fn get_cache_password(&self) -> Result<Option<String>> {
        let secret = self.get_secret(SecretPaths::CACHE).await?;
        Ok(secret.get("password").cloned())
    }

    /// Get directory (Zitadel) full configuration
    /// Returns (url, project_id, client_id, client_secret)
    pub async fn get_directory_config(&self) -> Result<(String, String, String, String)> {
        let secret = self.get_secret(SecretPaths::DIRECTORY).await?;
        Ok((
            secret
                .get("url")
                .cloned()
                .unwrap_or_else(|| "https://localhost:8080".to_string()),
            secret.get("project_id").cloned().unwrap_or_default(),
            secret.get("client_id").cloned().unwrap_or_default(),
            secret.get("client_secret").cloned().unwrap_or_default(),
        ))
    }

    /// Get directory (Zitadel) credentials only
    pub async fn get_directory_credentials(&self) -> Result<(String, String)> {
        let secret = self.get_secret(SecretPaths::DIRECTORY).await?;
        Ok((
            secret.get("client_id").cloned().unwrap_or_default(),
            secret.get("client_secret").cloned().unwrap_or_default(),
        ))
    }

    /// Get database full configuration
    /// Returns (host, port, database, username, password)
    pub async fn get_database_config(&self) -> Result<(String, u16, String, String, String)> {
        let secret = self.get_secret(SecretPaths::TABLES).await?;
        Ok((
            secret
                .get("host")
                .cloned()
                .unwrap_or_else(|| "localhost".to_string()),
            secret
                .get("port")
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            secret
                .get("database")
                .cloned()
                .unwrap_or_else(|| "botserver".to_string()),
            secret
                .get("username")
                .cloned()
                .unwrap_or_else(|| "gbuser".to_string()),
            secret.get("password").cloned().unwrap_or_default(),
        ))
    }

    /// Get database connection URL
    pub async fn get_database_url(&self) -> Result<String> {
        let (host, port, database, username, password) = self.get_database_config().await?;
        Ok(format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, host, port, database
        ))
    }

    /// Get vector database (Qdrant) configuration
    pub async fn get_vectordb_config(&self) -> Result<(String, Option<String>)> {
        let secret = self.get_secret(SecretPaths::VECTORDB).await?;
        Ok((
            secret
                .get("url")
                .cloned()
                .unwrap_or_else(|| "https://localhost:6334".to_string()),
            secret.get("api_key").cloned(),
        ))
    }

    /// Get observability (InfluxDB) configuration
    pub async fn get_observability_config(&self) -> Result<(String, String, String, String)> {
        let secret = self.get_secret(SecretPaths::OBSERVABILITY).await?;
        Ok((
            secret
                .get("url")
                .cloned()
                .unwrap_or_else(|| "http://localhost:8086".to_string()),
            secret
                .get("org")
                .cloned()
                .unwrap_or_else(|| "pragmatismo".to_string()),
            secret
                .get("bucket")
                .cloned()
                .unwrap_or_else(|| "metrics".to_string()),
            secret.get("token").cloned().unwrap_or_default(),
        ))
    }

    /// Get LLM API keys
    pub async fn get_llm_api_key(&self, provider: &str) -> Result<Option<String>> {
        let secret = self.get_secret(SecretPaths::LLM).await?;
        let key = format!("{}_key", provider.to_lowercase());
        Ok(secret.get(&key).cloned())
    }

    /// Get encryption key
    pub async fn get_encryption_key(&self) -> Result<String> {
        let secret = self.get_secret(SecretPaths::ENCRYPTION).await?;
        secret
            .get("master_key")
            .cloned()
            .ok_or_else(|| anyhow!("Encryption master key not found"))
    }

    /// Store a secret in Vault
    pub async fn put_secret(&self, path: &str, data: HashMap<String, String>) -> Result<()> {
        if !self.enabled {
            warn!("Vault not enabled, cannot store secret at '{}'", path);
            return Ok(());
        }

        let url = format!("{}/v1/secret/data/{}", self.config.addr, path);

        let body = serde_json::json!({
            "data": data
        });

        let response = self
            .client
            .post(&url)
            .header("X-Vault-Token", &self.config.token)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Vault")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Vault write failed ({}): {}", status, error_text));
        }

        // Invalidate cache
        self.invalidate_cache(path).await;

        info!("Secret stored at '{}'", path);
        Ok(())
    }

    /// Delete a secret from Vault
    pub async fn delete_secret(&self, path: &str) -> Result<()> {
        if !self.enabled {
            warn!("Vault not enabled, cannot delete secret at '{}'", path);
            return Ok(());
        }

        let url = format!("{}/v1/secret/data/{}", self.config.addr, path);

        let response = self
            .client
            .delete(&url)
            .header("X-Vault-Token", &self.config.token)
            .send()
            .await
            .context("Failed to connect to Vault")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Vault delete failed ({}): {}", status, error_text));
        }

        // Invalidate cache
        self.invalidate_cache(path).await;

        info!("Secret deleted at '{}'", path);
        Ok(())
    }

    /// Check Vault health
    pub async fn health_check(&self) -> Result<bool> {
        if !self.enabled {
            return Ok(false);
        }

        let url = format!("{}/v1/sys/health", self.config.addr);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Vault")?;

        // Vault returns 200 for initialized, unsealed, active
        // 429 for unsealed, standby
        // 472 for disaster recovery replication secondary
        // 473 for performance standby
        // 501 for not initialized
        // 503 for sealed
        Ok(response.status().as_u16() == 200 || response.status().as_u16() == 429)
    }

    /// Fetch secret from Vault API
    async fn fetch_from_vault(&self, path: &str) -> Result<HashMap<String, String>> {
        let url = format!("{}/v1/secret/data/{}", self.config.addr, path);

        debug!("Fetching secret from Vault: {}", path);

        let mut request = self
            .client
            .get(&url)
            .header("X-Vault-Token", &self.config.token);

        if let Some(ref namespace) = self.config.namespace {
            request = request.header("X-Vault-Namespace", namespace);
        }

        let response = request.send().await.context("Failed to connect to Vault")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            debug!("Secret not found in Vault: {}", path);
            return Ok(HashMap::new());
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Vault read failed ({}): {}", status, error_text));
        }

        let vault_response: VaultResponse = response
            .json()
            .await
            .context("Failed to parse Vault response")?;

        // Convert JSON values to strings
        let data: HashMap<String, String> = vault_response
            .data
            .data
            .into_iter()
            .map(|(k, v)| {
                let value = match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string().trim_matches('"').to_string(),
                };
                (k, value)
            })
            .collect();

        debug!("Secret '{}' fetched from Vault ({} keys)", path, data.len());
        Ok(data)
    }

    /// Get cached secret if not expired
    async fn get_cached(&self, path: &str) -> Option<HashMap<String, String>> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(path) {
            if cached.expires_at > std::time::Instant::now() {
                return Some(cached.data.clone());
            }
        }
        None
    }

    /// Cache a secret
    async fn cache_secret(&self, path: &str, data: HashMap<String, String>) {
        let mut cache = self.cache.write().await;
        cache.insert(
            path.to_string(),
            CachedSecret {
                data,
                expires_at: std::time::Instant::now()
                    + std::time::Duration::from_secs(self.config.cache_ttl),
            },
        );
    }

    /// Invalidate cached secret
    async fn invalidate_cache(&self, path: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(path);
    }

    /// Clear all cached secrets
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Fallback: get secrets from environment variables
    fn get_from_env(&self, path: &str) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        match path {
            SecretPaths::DRIVE => {
                if let Ok(v) = env::var("DRIVE_ACCESSKEY") {
                    data.insert("accesskey".to_string(), v);
                }
                if let Ok(v) = env::var("DRIVE_SECRET") {
                    data.insert("secret".to_string(), v);
                }
            }
            SecretPaths::TABLES => {
                if let Ok(v) = env::var("DB_USER") {
                    data.insert("username".to_string(), v);
                }
                if let Ok(v) = env::var("DB_PASSWORD") {
                    data.insert("password".to_string(), v);
                }
            }
            SecretPaths::CACHE => {
                if let Ok(v) = env::var("REDIS_PASSWORD") {
                    data.insert("password".to_string(), v);
                }
            }
            SecretPaths::DIRECTORY => {
                if let Ok(v) = env::var("DIRECTORY_URL") {
                    data.insert("url".to_string(), v);
                }
                if let Ok(v) = env::var("DIRECTORY_PROJECT_ID") {
                    data.insert("project_id".to_string(), v);
                }
                if let Ok(v) = env::var("ZITADEL_CLIENT_ID") {
                    data.insert("client_id".to_string(), v);
                }
                if let Ok(v) = env::var("ZITADEL_CLIENT_SECRET") {
                    data.insert("client_secret".to_string(), v);
                }
            }
            SecretPaths::TABLES => {
                if let Ok(v) = env::var("DB_HOST") {
                    data.insert("host".to_string(), v);
                }
                if let Ok(v) = env::var("DB_PORT") {
                    data.insert("port".to_string(), v);
                }
                if let Ok(v) = env::var("DB_NAME") {
                    data.insert("database".to_string(), v);
                }
                if let Ok(v) = env::var("DB_USER") {
                    data.insert("username".to_string(), v);
                }
                if let Ok(v) = env::var("DB_PASSWORD") {
                    data.insert("password".to_string(), v);
                }
                // Also support DATABASE_URL for backwards compatibility
                if let Ok(url) = env::var("DATABASE_URL") {
                    // Parse postgres://user:pass@host:port/db
                    if let Some(parsed) = parse_database_url(&url) {
                        data.extend(parsed);
                    }
                }
            }
            SecretPaths::VECTORDB => {
                if let Ok(v) = env::var("QDRANT_URL") {
                    data.insert("url".to_string(), v);
                }
                if let Ok(v) = env::var("QDRANT_API_KEY") {
                    data.insert("api_key".to_string(), v);
                }
            }
            SecretPaths::OBSERVABILITY => {
                if let Ok(v) = env::var("INFLUXDB_URL") {
                    data.insert("url".to_string(), v);
                }
                if let Ok(v) = env::var("INFLUXDB_ORG") {
                    data.insert("org".to_string(), v);
                }
                if let Ok(v) = env::var("INFLUXDB_BUCKET") {
                    data.insert("bucket".to_string(), v);
                }
                if let Ok(v) = env::var("INFLUXDB_TOKEN") {
                    data.insert("token".to_string(), v);
                }
            }
            SecretPaths::EMAIL => {
                if let Ok(v) = env::var("EMAIL_USER") {
                    data.insert("username".to_string(), v);
                }
                if let Ok(v) = env::var("EMAIL_PASSWORD") {
                    data.insert("password".to_string(), v);
                }
            }
            SecretPaths::LLM => {
                if let Ok(v) = env::var("OPENAI_API_KEY") {
                    data.insert("openai_key".to_string(), v);
                }
                if let Ok(v) = env::var("ANTHROPIC_API_KEY") {
                    data.insert("anthropic_key".to_string(), v);
                }
                if let Ok(v) = env::var("GROQ_API_KEY") {
                    data.insert("groq_key".to_string(), v);
                }
            }
            SecretPaths::ENCRYPTION => {
                if let Ok(v) = env::var("ENCRYPTION_KEY") {
                    data.insert("master_key".to_string(), v);
                }
            }
            SecretPaths::MEET => {
                if let Ok(v) = env::var("LIVEKIT_API_KEY") {
                    data.insert("api_key".to_string(), v);
                }
                if let Ok(v) = env::var("LIVEKIT_API_SECRET") {
                    data.insert("api_secret".to_string(), v);
                }
            }
            SecretPaths::ALM => {
                if let Ok(v) = env::var("ALM_URL") {
                    data.insert("url".to_string(), v);
                }
                if let Ok(v) = env::var("ALM_ADMIN_PASSWORD") {
                    data.insert("admin_password".to_string(), v);
                }
                if let Ok(v) = env::var("ALM_RUNNER_TOKEN") {
                    data.insert("runner_token".to_string(), v);
                }
            }
            _ => {
                warn!("Unknown secret path: {}", path);
            }
        }

        Ok(data)
    }
}

/// Parse a DATABASE_URL into individual components
fn parse_database_url(url: &str) -> Option<HashMap<String, String>> {
    // postgres://user:pass@host:port/database
    let url = url.strip_prefix("postgres://")?;
    let mut data = HashMap::new();

    // Split user:pass@host:port/database
    let (auth, rest) = url.split_once('@')?;
    let (user, pass) = auth.split_once(':').unwrap_or((auth, ""));

    data.insert("username".to_string(), user.to_string());
    data.insert("password".to_string(), pass.to_string());

    // Split host:port/database
    let (host_port, database) = rest.split_once('/').unwrap_or((rest, "botserver"));
    let (host, port) = host_port.split_once(':').unwrap_or((host_port, "5432"));

    data.insert("host".to_string(), host.to_string());
    data.insert("port".to_string(), port.to_string());
    data.insert("database".to_string(), database.to_string());

    Some(data)
}

/// Initialize secrets manager from environment
///
/// .env should contain ONLY:
/// ```
/// VAULT_ADDR=https://localhost:8200
/// VAULT_TOKEN=hvs.xxxxxxxxxxxxx
/// ```
///
/// All other configuration is fetched from Vault at runtime.
pub fn init_secrets_manager() -> Result<SecretsManager> {
    SecretsManager::from_env()
}

/// Bootstrap configuration structure
/// Used when Vault is not yet available (initial setup)
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    pub vault_addr: String,
    pub vault_token: String,
}

impl BootstrapConfig {
    /// Load from .env file
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            vault_addr: env::var("VAULT_ADDR").context("VAULT_ADDR not set in .env")?,
            vault_token: env::var("VAULT_TOKEN").context("VAULT_TOKEN not set in .env")?,
        })
    }

    /// Check if .env is properly configured
    pub fn is_configured() -> bool {
        env::var("VAULT_ADDR").is_ok() && env::var("VAULT_TOKEN").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_config_default() {
        // Temporarily set environment variables
        std::env::set_var("VAULT_ADDR", "https://test:8200");
        std::env::set_var("VAULT_TOKEN", "test-token");

        let config = VaultConfig::default();
        assert_eq!(config.addr, "https://test:8200");
        assert_eq!(config.token, "test-token");
        assert!(config.skip_verify);

        // Clean up
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[test]
    fn test_secrets_manager_disabled_without_token() {
        std::env::remove_var("VAULT_TOKEN");
        std::env::set_var("VAULT_ADDR", "https://localhost:8200");

        let manager = SecretsManager::from_env().unwrap();
        assert!(!manager.is_enabled());

        std::env::remove_var("VAULT_ADDR");
    }

    #[tokio::test]
    async fn test_get_from_env_fallback() {
        std::env::set_var("DRIVE_ACCESSKEY", "test-access");
        std::env::set_var("DRIVE_SECRET", "test-secret");
        std::env::remove_var("VAULT_TOKEN");

        let manager = SecretsManager::from_env().unwrap();
        let secret = manager.get_secret(SecretPaths::DRIVE).await.unwrap();

        assert_eq!(secret.get("accesskey"), Some(&"test-access".to_string()));
        assert_eq!(secret.get("secret"), Some(&"test-secret".to_string()));

        std::env::remove_var("DRIVE_ACCESSKEY");
        std::env::remove_var("DRIVE_SECRET");
    }
}
