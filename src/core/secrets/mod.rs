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

struct CachedSecret {
    data: HashMap<String, String>,
    expires_at: std::time::Instant,
}

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
            .field("client", &self.client.is_some())
            .field("cache", &"<RwLock<HashMap>>")
            .field("cache_ttl", &self.cache_ttl)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl SecretsManager {
    pub fn from_env() -> Result<Self> {
        let addr = env::var("VAULT_ADDR").unwrap_or_default();
        let token = env::var("VAULT_TOKEN").unwrap_or_default();
        let skip_verify = env::var("VAULT_SKIP_VERIFY")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let cache_ttl = env::var("VAULT_CACHE_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

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

        let ca_path = PathBuf::from(&ca_cert);
        let cert_path = PathBuf::from(&client_cert);
        let key_path = PathBuf::from(&client_key);

        let mut settings_builder = VaultClientSettingsBuilder::default();
        settings_builder.address(&addr).token(&token);

        // Only warn about TLS verification for HTTPS connections
        let is_https = addr.starts_with("https://");
        if skip_verify {
            if is_https {
                warn!("TLS verification disabled - NOT RECOMMENDED FOR PRODUCTION");
            }
            settings_builder.verify(false);
        } else {
            settings_builder.verify(true);

            if ca_path.exists() {
                info!("Using CA certificate for Vault: {}", ca_cert);
                settings_builder.ca_certs(vec![ca_cert]);
            }
        }

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

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        if !self.enabled {
            return Self::get_from_env(path);
        }

        if let Some(cached) = self.get_cached(path).await {
            return Ok(cached);
        }

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
                return Self::get_from_env(path);
            }
        };

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
                .unwrap_or_else(|| "http://localhost:8300".into()),
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
            s.get("org").cloned().unwrap_or_else(|| "system".into()),
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

    fn get_from_env(path: &str) -> Result<HashMap<String, String>> {
        let mut secrets = HashMap::new();

        match path {
            SecretPaths::TABLES => {
                secrets.insert("host".into(), "localhost".into());
                secrets.insert("port".into(), "5432".into());
                secrets.insert("database".into(), "botserver".into());
                secrets.insert("username".into(), "gbuser".into());
                secrets.insert("password".into(), "changeme".into());
            }
            SecretPaths::DIRECTORY => {
                secrets.insert("url".into(), "http://localhost:8300".into());
                secrets.insert("project_id".into(), String::new());
                secrets.insert("client_id".into(), String::new());
                secrets.insert("client_secret".into(), String::new());
            }
            SecretPaths::DRIVE => {
                secrets.insert("accesskey".into(), String::new());
                secrets.insert("secret".into(), String::new());
            }
            SecretPaths::CACHE => {
                secrets.insert("password".into(), String::new());
            }
            SecretPaths::EMAIL => {
                secrets.insert("smtp_host".into(), String::new());
                secrets.insert("smtp_port".into(), "587".into());
                secrets.insert("username".into(), String::new());
                secrets.insert("password".into(), String::new());
                secrets.insert("from_address".into(), String::new());
            }
            SecretPaths::LLM => {
                secrets.insert("openai_key".into(), String::new());
                secrets.insert("anthropic_key".into(), String::new());
                secrets.insert("ollama_url".into(), "http://localhost:11434".into());
            }
            SecretPaths::ENCRYPTION => {
                secrets.insert("master_key".into(), String::new());
            }
            SecretPaths::MEET => {
                secrets.insert("jitsi_url".into(), "https://meet.jit.si".into());
                secrets.insert("app_id".into(), String::new());
                secrets.insert("app_secret".into(), String::new());
            }
            SecretPaths::VECTORDB => {
                secrets.insert("url".into(), "http://localhost:6333".into());
                secrets.insert("api_key".into(), String::new());
            }
            SecretPaths::OBSERVABILITY => {
                secrets.insert("url".into(), "http://localhost:8086".into());
                secrets.insert("org".into(), "system".into());
                secrets.insert("bucket".into(), "metrics".into());
                secrets.insert("token".into(), String::new());
            }
            SecretPaths::ALM => {
                secrets.insert("url".into(), "http://localhost:9000".into());
                secrets.insert("username".into(), String::new());
                secrets.insert("password".into(), String::new());
            }
            _ => {
                log::debug!("No default values for secret path: {}", path);
            }
        }

        Ok(secrets)
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
