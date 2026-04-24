use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::shared::utils::DbPool;
use diesel::prelude::*;

#[derive(Debug, Clone, QueryableByName)]
struct ConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_value: String,
}

fn is_placeholder_value(val: &str) -> bool {
    let lower = val.trim().to_lowercase();
    lower.is_empty() || lower == "none" || lower == "null" || lower == "n/a"
}

fn is_local_file_path(val: &str) -> bool {
    let lower = val.to_lowercase();
    val.starts_with("../")
        || val.starts_with("./")
        || val.starts_with('/')
        || val.starts_with('~')
        || lower.ends_with(".gguf")
        || lower.ends_with(".bin")
        || lower.ends_with(".safetensors")
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub drive: DriveConfig,
    pub email: EmailConfig,
    pub site_path: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from_address: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        // Configuration loading priority: 1) Environment (from Vault via .env), 2) Defaults for development
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: std::env::var("PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(8080),
                base_url: String::new(),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/botserver".to_string()),
                max_connections: 10,
            },
            drive: DriveConfig::from_vault()
                .unwrap_or_else(|_| DriveConfig::default()),
            email: EmailConfig::default(),
            site_path: std::env::var("SITE_PATH")
                .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
            data_dir: std::env::var("DATA_DIR")
                .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }

    pub fn from_database(
        _pool: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load config from database
        // For now, return default
        Ok(Self::default())
    }

pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
    // Configuration loading: Environment vars (from Vault via .env) with fallbacks for development
    Ok(Self {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            base_url: String::new(),
        },
        database: DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/botserver".to_string()),
            max_connections: 10,
        },
        drive: DriveConfig::from_vault()
            .unwrap_or_else(|_| DriveConfig::default()),
        email: EmailConfig::default(),
        site_path: std::env::var("SITE_PATH")
            .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
        data_dir: std::env::var("DATA_DIR")
            .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
    })
}
}

/// Configuration manager for runtime config updates
pub struct ConfigManager {
    pool: Arc<DbPool>,
}

impl ConfigManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool: Arc::new(pool) }
    }

    pub fn get_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        default: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Ok(mut conn) = self.pool.get() {
            let bot_val = diesel::sql_query(
                "SELECT config_value FROM bot_configuration WHERE bot_id = $1 AND config_key = $2 LIMIT 1"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(key)
            .get_result::<ConfigRow>(&mut conn)
            .ok()
            .map(|r| r.config_value);

            if let Some(ref val) = bot_val {
                if !is_placeholder_value(val) && !is_local_file_path(val) {
                    return Ok(val.clone());
                }
            }

            let default_val = diesel::sql_query(
                "SELECT config_value FROM bot_configuration WHERE bot_id = $1 AND config_key = $2 LIMIT 1"
            )
            .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
            .bind::<diesel::sql_types::Text, _>(key)
            .get_result::<ConfigRow>(&mut conn)
            .ok()
            .map(|r| r.config_value);

            if let Some(ref val) = default_val {
                if !is_placeholder_value(val) {
                    return Ok(val.clone());
                }
            }
        }
        Ok(default.unwrap_or("").to_string())
    }

    pub fn get_bot_config_value(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Ok(mut conn) = self.pool.get() {
            let row = diesel::sql_query(
                "SELECT config_value FROM bot_configuration WHERE bot_id = $1 AND config_key = $2 LIMIT 1"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(key)
            .get_result::<ConfigRow>(&mut conn)
            .ok();
            if let Some(r) = row {
                return Ok(r.config_value);
            }
        }
        Err("Config key not found".into())
    }

    pub fn set_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut conn) = self.pool.get() {
            diesel::sql_query(
                "INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type, is_encrypted, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, 'string', false, NOW(), NOW()) \
                 ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = $4, updated_at = NOW()"
            )
            .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(key)
            .bind::<diesel::sql_types::Text, _>(value)
            .execute(&mut conn)?;
        }
        Ok(())
    }
}

// Re-export for convenience
pub use AppConfig as Config;

// Manual implementation to load from Vault
impl Default for DriveConfig {
    fn default() -> Self {
        // Try to load from Vault
        if let Ok(vault_addr) = std::env::var("VAULT_ADDR") {
            if let Ok(vault_token) = std::env::var("VAULT_TOKEN") {
                let ca_cert = std::env::var("VAULT_CACERT").unwrap_or_default();
                let url = format!("{}/v1/secret/data/gbo/drive", vault_addr);
                
                if let Ok(output) = std::process::Command::new("curl")
                    .args(&["-sf", "--cacert", &ca_cert, "-H", &format!("X-Vault-Token: {}", &vault_token), &url])
                    .output()
                {
                    if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        if let Some(secret_data) = data.get("data").and_then(|d| d.get("data")) {
                            let host = secret_data.get("host").and_then(|v| v.as_str()).unwrap_or("localhost");
                            let accesskey = secret_data.get("accesskey").and_then(|v| v.as_str()).unwrap_or("");
                            let secret = secret_data.get("secret").and_then(|v| v.as_str()).unwrap_or("");
                            let bucket = secret_data.get("bucket").and_then(|v| v.as_str()).unwrap_or("default.gbai");
                            
                            return Self {
                                endpoint: format!("http://{}", host),
                                bucket: bucket.to_string(),
                                region: "auto".to_string(),
                                access_key: accesskey.to_string(),
                                secret_key: secret.to_string(),
                                server: host.to_string(),
                            };
                        }
                    }
                }
            }
        }
        
        // Fallback for development: read from environment or use minimal defaults
        Self {
            endpoint: std::env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9100".to_string()),
            bucket: std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "default.gbai".to_string()),
            region: "auto".to_string(),
            access_key: std::env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            server: std::env::var("MINIO_SERVER")
                .unwrap_or_else(|_| "localhost:9100".to_string()),
        }
    }
}
