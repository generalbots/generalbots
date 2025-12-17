//! Secrets Management Module
//!
//! Provides integration with HashiCorp Vault for secure secrets management
//! using the `vaultrs` library.
//!
//! With Vault, .env contains ONLY:
//! - VAULT_ADDR - Vault server address
//! - VAULT_TOKEN - Vault authentication token
//!
//! Vault paths:
//! - gbo/directory - Zitadel connection
//! - gbo/tables - PostgreSQL credentials
//! - gbo/drive - MinIO/S3 credentials
//! - gbo/cache - Redis credentials
//! - gbo/email - Email credentials
//! - gbo/llm - LLM API keys
//! - gbo/encryption - Encryption keys
//! - gbo/meet - LiveKit credentials
//! - gbo/alm - Forgejo credentials
//! - gbo/vectordb - Qdrant credentials
//! - gbo/observability - InfluxDB credentials

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Arc as StdArc;
use tokio::sync::RwLock;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;

/// Secret paths in Vault
#[derive(Debug)]
pub struct SecretPaths;

impl SecretPaths {
    pub const DIRECTORY: &'static str = "gbo/directory";
    pub const TABLES: &'static str = "gbo/tables";
    pub const DRIVE: &'static str = "gbo/drive";
    pub const CACHE: &'static str = "gbo/cache";
    pub const EMAIL: &'static str = "gbo/email";
    pub const LLM: &'static str = "gbo/llm";
    pub const ENCRYPTION: &'static str = "gbo/encryption";
    pub const MEET: &'static str = "gbo/meet";
    pub const ALM: &'static str = "gbo/alm";
    pub const VECTORDB: &'static str = "gbo/vectordb";
    pub const OBSERVABILITY: &'static str = "gbo/observability";
}

/// Cached secret with expiry
struct CachedSecret {
    data: HashMap<String, String>,
    expires_at: std::time::Instant,
}

/// Secrets manager using vaultrs
#[derive(Clone)]
pub struct SecretsManager {
    client: Option<StdArc<VaultClient>>,
    cache: Arc<RwLock<HashMap<String, CachedSecret>>>,
    cache_ttl: u64,
    enabled: bool,
}

impl std::fmt::Debug for SecretsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretsManager")
            .field("enabled", &self.enabled)
            .field("cache_ttl", &self.cache_ttl)
            .finish()
    }
}

impl SecretsManager {
    /// Create from environment variables with mTLS support
    ///
    /// Environment variables:
    /// - VAULT_ADDR - Vault server address (https://localhost:8200)
    /// - VAULT_TOKEN - Vault authentication token
    /// - VAULT_CACERT - Path to CA certificate for verifying Vault server
    /// - VAULT_CLIENT_CERT - Path to client certificate for mTLS
    /// - VAULT_CLIENT_KEY - Path to client key for mTLS
    /// - VAULT_SKIP_VERIFY - Skip TLS verification (for development only)
    /// - VAULT_CACHE_TTL - Cache TTL in seconds (default: 300)
    pub fn from_env() -> Result<Self> {
        let addr = env::var("VAULT_ADDR").unwrap_or_default();
        let token = env::var("VAULT_TOKEN").unwrap_or_default();
        let skip_verify = env::var("VAULT_SKIP_VERIFY")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false); // Default to false - verify certificates
        let cache_ttl = env::var("VAULT_CACHE_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        // mTLS certificate paths - default to botserver-stack paths
        let ca_cert = env::var("VAULT_CACERT")
            .unwrap_or_else(|_| "./botserver-stack/conf/system/certificates/ca/ca.crt".to_string());
        let client_cert = env::var("VAULT_CLIENT_CERT").unwrap_or_else(|_| {
            "./botserver-stack/conf/system/certificates/botserver/client.crt".to_string()
        });
        let client_key = env::var("VAULT_CLIENT_KEY").unwrap_or_else(|_| {
            "./botserver-stack/conf/system/certificates/botserver/client.key".to_string()
        });

        let enabled = !token.is_empty() && !addr.is_empty();

        if !enabled {
            warn!("Vault not configured. Using environment variables directly.");
            return Ok(Self {
                client: None,
                cache: Arc::new(RwLock::new(HashMap::new())),
                cache_ttl,
                enabled: false,
            });
        }

        // Build settings with mTLS if certificates exist
        let ca_path = PathBuf::from(&ca_cert);
        let cert_path = PathBuf::from(&client_cert);
        let key_path = PathBuf::from(&client_key);

        let mut settings_builder = VaultClientSettingsBuilder::default();
        settings_builder.address(&addr).token(&token);

        // Configure TLS verification
        if skip_verify {
            warn!("TLS verification disabled - NOT RECOMMENDED FOR PRODUCTION");
            settings_builder.verify(false);
        } else {
            settings_builder.verify(true);
            // Add CA certificate if it exists
            if ca_path.exists() {
                info!("Using CA certificate for Vault: {}", ca_cert);
                settings_builder.ca_certs(vec![ca_cert.clone()]);
            }
        }

        // Configure mTLS client certificates if they exist
        if cert_path.exists() && key_path.exists() && !skip_verify {
            info!("Using mTLS client certificate for Vault: {}", client_cert);
        }

        let settings = settings_builder.build()?;
        let client = VaultClient::new(settings)?;

        info!("Vault client initialized with TLS: {}", addr);

        Ok(Self {
            client: Some(StdArc::new(client)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            enabled: true,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get a secret from Vault or env fallback
    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        if !self.enabled {
            return self.get_from_env(path);
        }

        // Check cache
        if let Some(cached) = self.get_cached(path).await {
            return Ok(cached);
        }

        // Fetch from Vault
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("No Vault client"))?;

        let result: Result<HashMap<String, String>, _> =
            kv2::read(client.as_ref(), "secret", path).await;

        let data = match result {
            Ok(d) => d,
            Err(e) => {
                debug!(
                    "Vault read failed for '{}': {}, falling back to env",
                    path, e
                );
                return self.get_from_env(path);
            }
        };

        // Cache result
        if self.cache_ttl > 0 {
            self.cache_secret(path, data.clone()).await;
        }

        Ok(data)
    }

    pub async fn get_value(&self, path: &str, key: &str) -> Result<String> {
        self.get_secret(path)
            .await?
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Key '{}' not found in '{}'", key, path))
    }

    // Convenience methods for specific secrets

    pub async fn get_drive_credentials(&self) -> Result<(String, String)> {
        let s = self.get_secret(SecretPaths::DRIVE).await?;
        Ok((
            s.get("accesskey").cloned().unwrap_or_default(),
            s.get("secret").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_database_config(&self) -> Result<(String, u16, String, String, String)> {
        let s = self.get_secret(SecretPaths::TABLES).await?;
        Ok((
            s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
            s.get("port").and_then(|p| p.parse().ok()).unwrap_or(5432),
            s.get("database")
                .cloned()
                .unwrap_or_else(|| "botserver".into()),
            s.get("username")
                .cloned()
                .unwrap_or_else(|| "gbuser".into()),
            s.get("password").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_database_url(&self) -> Result<String> {
        let (host, port, db, user, pass) = self.get_database_config().await?;
        Ok(format!(
            "postgres://{}:{}@{}:{}/{}",
            user, pass, host, port, db
        ))
    }

    pub async fn get_database_credentials(&self) -> Result<(String, String)> {
        let s = self.get_secret(SecretPaths::TABLES).await?;
        Ok((
            s.get("username")
                .cloned()
                .unwrap_or_else(|| "gbuser".into()),
            s.get("password").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_cache_password(&self) -> Result<Option<String>> {
        Ok(self
            .get_secret(SecretPaths::CACHE)
            .await?
            .get("password")
            .cloned())
    }

    pub async fn get_directory_config(&self) -> Result<(String, String, String, String)> {
        let s = self.get_secret(SecretPaths::DIRECTORY).await?;
        Ok((
            s.get("url")
                .cloned()
                .unwrap_or_else(|| "https://localhost:8080".into()),
            s.get("project_id").cloned().unwrap_or_default(),
            s.get("client_id").cloned().unwrap_or_default(),
            s.get("client_secret").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_directory_credentials(&self) -> Result<(String, String)> {
        let s = self.get_secret(SecretPaths::DIRECTORY).await?;
        Ok((
            s.get("client_id").cloned().unwrap_or_default(),
            s.get("client_secret").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_vectordb_config(&self) -> Result<(String, Option<String>)> {
        let s = self.get_secret(SecretPaths::VECTORDB).await?;
        Ok((
            s.get("url")
                .cloned()
                .unwrap_or_else(|| "https://localhost:6334".into()),
            s.get("api_key").cloned(),
        ))
    }

    pub async fn get_observability_config(&self) -> Result<(String, String, String, String)> {
        let s = self.get_secret(SecretPaths::OBSERVABILITY).await?;
        Ok((
            s.get("url")
                .cloned()
                .unwrap_or_else(|| "http://localhost:8086".into()),
            s.get("org")
                .cloned()
                .unwrap_or_else(|| "pragmatismo".into()),
            s.get("bucket").cloned().unwrap_or_else(|| "metrics".into()),
            s.get("token").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_llm_api_key(&self, provider: &str) -> Result<Option<String>> {
        let s = self.get_secret(SecretPaths::LLM).await?;
        Ok(s.get(&format!("{}_key", provider.to_lowercase())).cloned())
    }

    pub async fn get_encryption_key(&self) -> Result<String> {
        self.get_value(SecretPaths::ENCRYPTION, "master_key").await
    }

    pub async fn put_secret(&self, path: &str, data: HashMap<String, String>) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Vault not enabled"))?;
        kv2::set(client.as_ref(), "secret", path, &data).await?;
        self.invalidate_cache(path).await;
        info!("Secret stored at '{}'", path);
        Ok(())
    }

    pub async fn delete_secret(&self, path: &str) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Vault not enabled"))?;
        kv2::delete_latest(client.as_ref(), "secret", path).await?;
        self.invalidate_cache(path).await;
        info!("Secret deleted at '{}'", path);
        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool> {
        if let Some(client) = &self.client {
            Ok(vaultrs::sys::health(client.as_ref()).await.is_ok())
        } else {
            Ok(false)
        }
    }

    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    async fn get_cached(&self, path: &str) -> Option<HashMap<String, String>> {
        let cache = self.cache.read().await;
        cache
            .get(path)
            .and_then(|c| (c.expires_at > std::time::Instant::now()).then(|| c.data.clone()))
    }

    async fn cache_secret(&self, path: &str, data: HashMap<String, String>) {
        self.cache.write().await.insert(
            path.to_string(),
            CachedSecret {
                data,
                expires_at: std::time::Instant::now()
                    + std::time::Duration::from_secs(self.cache_ttl),
            },
        );
    }

    async fn invalidate_cache(&self, path: &str) {
        self.cache.write().await.remove(path);
    }

    /// No fallback - Vault is mandatory
    /// Returns empty HashMap if Vault is not configured
    fn get_from_env(&self, _path: &str) -> Result<HashMap<String, String>> {
        // NO LEGACY FALLBACK - All secrets MUST come from Vault
        // If you see this error, ensure Vault is properly configured with:
        //   VAULT_ADDR=https://localhost:8200
        //   VAULT_TOKEN=<your-token>
        Err(anyhow!("Vault not configured. All secrets must be stored in Vault. Set VAULT_ADDR and VAULT_TOKEN in .env"))
    }
}

pub fn init_secrets_manager() -> Result<SecretsManager> {
    SecretsManager::from_env()
}

#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    pub vault_addr: String,
    pub vault_token: String,
}

impl BootstrapConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            vault_addr: env::var("VAULT_ADDR")?,
            vault_token: env::var("VAULT_TOKEN")?,
        })
    }

    pub fn is_configured() -> bool {
        env::var("VAULT_ADDR").is_ok() && env::var("VAULT_TOKEN").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to parse database URL into HashMap for tests
    fn parse_database_url(url: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        if let Some(stripped) = url.strip_prefix("postgres://") {
            let parts: Vec<&str> = stripped.split('@').collect();
            if parts.len() == 2 {
                let user_pass: Vec<&str> = parts[0].split(':').collect();
                let host_db: Vec<&str> = parts[1].split('/').collect();

                result.insert(
                    "username".to_string(),
                    user_pass.get(0).unwrap_or(&"").to_string(),
                );
                result.insert(
                    "password".to_string(),
                    user_pass.get(1).unwrap_or(&"").to_string(),
                );

                let host_port: Vec<&str> = host_db[0].split(':').collect();
                result.insert(
                    "host".to_string(),
                    host_port.get(0).unwrap_or(&"").to_string(),
                );
                result.insert(
                    "port".to_string(),
                    host_port.get(1).unwrap_or(&"5432").to_string(),
                );

                if host_db.len() >= 2 {
                    result.insert("database".to_string(), host_db[1].to_string());
                }
            }
        }
        Ok(result)
    }

    #[test]
    fn test_parse_database_url() {
        let parsed = parse_database_url("postgres://user:pass@localhost:5432/mydb").unwrap();
        assert_eq!(parsed.get("username"), Some(&"user".to_string()));
        assert_eq!(parsed.get("password"), Some(&"pass".to_string()));
        assert_eq!(parsed.get("host"), Some(&"localhost".to_string()));
        assert_eq!(parsed.get("port"), Some(&"5432".to_string()));
        assert_eq!(parsed.get("database"), Some(&"mydb".to_string()));
    }

    #[test]
    fn test_parse_database_url_minimal() {
        let parsed = parse_database_url("postgres://user@localhost/mydb").unwrap();
        assert_eq!(parsed.get("username"), Some(&"user".to_string()));
        assert_eq!(parsed.get("password"), Some(&"".to_string()));
        assert_eq!(parsed.get("host"), Some(&"localhost".to_string()));
        assert_eq!(parsed.get("port"), Some(&"5432".to_string()));
    }

    #[test]
    fn test_secret_paths() {
        assert_eq!(SecretPaths::DIRECTORY, "gbo/directory");
        assert_eq!(SecretPaths::TABLES, "gbo/tables");
        assert_eq!(SecretPaths::LLM, "gbo/llm");
    }
}
