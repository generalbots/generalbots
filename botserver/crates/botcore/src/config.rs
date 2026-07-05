use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::shared::utils::DbPool;
use diesel::prelude::*;
use botsecurity_crypto::encryption::{encrypt_field, decrypt_field, derive_scope_key, load_master_encryption_key};
use botsecurity_crypto::secrets::is_sensitive_key;

#[derive(Debug, Clone, QueryableByName)]
struct ConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_value: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_encrypted: bool,
}

#[derive(Debug, Clone, QueryableByName)]
struct ExistsRow {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    exists: bool,
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
            drive: DriveConfig::default(),
            email: EmailConfig::default(),
            site_path: std::env::var("SITE_PATH")
                .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
            data_dir: std::env::var("DATA_DIR")
                .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
        }
    }
}

impl AppConfig {
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "server.host" => Some(self.server.host.clone()),
            "server.port" => Some(self.server.port.to_string()),
            "server.base_url" => Some(self.server.base_url.clone()),
            "database.url" => Some(self.database.url.clone()),
            "drive.bucket" => Some(self.drive.bucket.clone()),
            "site_path" => Some(self.site_path.clone()),
            "data_dir" => Some(self.data_dir.clone()),
            _ => None,
        }
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }

    pub fn from_database(
        _pool: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }

pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
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
        drive: DriveConfig::default(),
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
    master_key: Vec<u8>,
    has_branch_id: bool,
}

impl ConfigManager {
    pub fn new(pool: DbPool) -> Self {
        let master_key = load_master_encryption_key();
        let has_branch_id = Self::check_has_branch_id(&pool);
        Self { pool: Arc::new(pool), master_key, has_branch_id }
    }

    fn check_has_branch_id(pool: &DbPool) -> bool {
        if let Ok(mut conn) = pool.get() {
            diesel::sql_query(
                "SELECT EXISTS (SELECT 1 FROM information_schema.columns \
                 WHERE table_name='bot_configuration' AND column_name='branch_id') AS exists"
            )
            .get_result::<ExistsRow>(&mut conn)
            .ok()
            .map(|r| r.exists)
            .unwrap_or(false)
        } else {
            false
        }
    }

    fn config_key(&self, bot_id: &uuid::Uuid) -> Vec<u8> {
        derive_scope_key(&self.master_key, "config", bot_id)
    }

    fn get_query(&self) -> &'static str {
        if self.has_branch_id {
            "SELECT config_value, is_encrypted FROM bot_configuration \
             WHERE branch_id = $1 AND bot_id = $2 AND config_key = $3 LIMIT 1"
        } else {
            "SELECT config_value, is_encrypted FROM bot_configuration \
             WHERE bot_id = $1 AND config_key = $2 LIMIT 1"
        }
    }

    pub fn get_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        default: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let key_bytes = self.config_key(bot_id);

        if let Ok(mut conn) = self.pool.get() {
            let bot_val = if self.has_branch_id {
                diesel::sql_query(self.get_query())
                    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
                    .bind::<diesel::sql_types::Uuid, _>(bot_id)
                    .bind::<diesel::sql_types::Text, _>(key)
                    .get_result::<ConfigRow>(&mut conn).ok()
            } else {
                diesel::sql_query(self.get_query())
                    .bind::<diesel::sql_types::Uuid, _>(bot_id)
                    .bind::<diesel::sql_types::Text, _>(key)
                    .get_result::<ConfigRow>(&mut conn).ok()
            };

            if let Some(r) = bot_val {
                if !is_placeholder_value(&r.config_value) && !is_local_file_path(&r.config_value) {
                    let val = if r.is_encrypted {
                        decrypt_field(&r.config_value, &key_bytes).unwrap_or_else(|_| r.config_value.clone())
                    } else {
                        r.config_value
                    };
                    return Ok(val);
                }
            }

            let default_val = if self.has_branch_id {
                diesel::sql_query(self.get_query())
                    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
                    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
                    .bind::<diesel::sql_types::Text, _>(key)
                    .get_result::<ConfigRow>(&mut conn).ok()
            } else {
                diesel::sql_query(self.get_query())
                    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
                    .bind::<diesel::sql_types::Text, _>(key)
                    .get_result::<ConfigRow>(&mut conn).ok()
            };

            if let Some(r) = default_val {
                if !is_placeholder_value(&r.config_value) {
                    let val = if r.is_encrypted {
                        decrypt_field(&r.config_value, &key_bytes).unwrap_or_else(|_| r.config_value.clone())
                    } else {
                        r.config_value
                    };
                    return Ok(val);
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
        let key_bytes = self.config_key(bot_id);

        if let Ok(mut conn) = self.pool.get() {
            let row = if self.has_branch_id {
                diesel::sql_query(
                    "SELECT config_value, is_encrypted FROM bot_configuration \
                     WHERE branch_id = $1 AND bot_id = $2 AND config_key = $3 LIMIT 1"
                )
                .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(key)
                .get_result::<ConfigRow>(&mut conn).ok()
            } else {
                diesel::sql_query(
                    "SELECT config_value, is_encrypted FROM bot_configuration \
                     WHERE bot_id = $1 AND config_key = $2 LIMIT 1"
                )
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(key)
                .get_result::<ConfigRow>(&mut conn).ok()
            };

            if let Some(r) = row {
                let val = if r.is_encrypted {
                    decrypt_field(&r.config_value, &key_bytes).unwrap_or_else(|_| r.config_value.clone())
                } else {
                    r.config_value
                };
                return Ok(val);
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
        self.set_config_with_branch(bot_id, key, value, None)
    }

    pub fn set_config_with_branch(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        value: &str,
        branch_id: Option<uuid::Uuid>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let is_sensitive = is_sensitive_key(key);
        let key_bytes = self.config_key(bot_id);

        let final_value = if is_sensitive {
            encrypt_field(value, &key_bytes).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        };

        if let Ok(mut conn) = self.pool.get() {
            if self.has_branch_id {
                let bid = branch_id.unwrap_or(uuid::Uuid::nil());
                diesel::sql_query(
                    "INSERT INTO bot_configuration (id, branch_id, bot_id, config_key, config_value, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) \
                     ON CONFLICT (branch_id, bot_id, config_key) DO UPDATE SET config_value = $5, updated_at = NOW()"
                )
                .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
                .bind::<diesel::sql_types::Uuid, _>(bid)
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(key)
                .bind::<diesel::sql_types::Text, _>(final_value)
                .execute(&mut conn)?;
            } else {
                diesel::sql_query(
                    "INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type, is_encrypted, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, 'string', $5, NOW(), NOW()) \
                     ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = $4, is_encrypted = $5, updated_at = NOW()"
                )
                .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(key)
                .bind::<diesel::sql_types::Text, _>(final_value)
                .bind::<diesel::sql_types::Bool, _>(is_sensitive)
                .execute(&mut conn)?;
            }
        }
        Ok(())
    }
}

pub use AppConfig as Config;

impl Default for DriveConfig {
    fn default() -> Self {
        if let Ok(vault_addr) = std::env::var("VAULT_ADDR") {
            if let Ok(vault_token) = std::env::var("VAULT_TOKEN") {
                let ca_cert = std::env::var("VAULT_CACERT").unwrap_or_default();
                let url = format!("{}/v1/secret/data/gbo/drive", vault_addr);

                if let Ok(output) = std::process::Command::new("curl")
                    .args(["-sf", "--cacert", &ca_cert, "-H", &format!("X-Vault-Token: {}", &vault_token), &url])
                    .output()
                {
                    if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        if let Some(secret_data) = data.get("data").and_then(|d| d.get("data")) {
                            let host = secret_data.get("host").and_then(|v| v.as_str()).unwrap_or("localhost");
                            let port = secret_data.get("port").and_then(|v| v.as_str()).unwrap_or("9100");
                            let accesskey = secret_data.get("accesskey").and_then(|v| v.as_str()).unwrap_or("");
                            let secret = secret_data.get("secret").and_then(|v| v.as_str()).unwrap_or("");
                            let bucket = secret_data.get("bucket").and_then(|v| v.as_str()).unwrap_or("default.gbai");
                            let server = format!("{}:{}", host, port);

                            return Self {
                                endpoint: format!("http://{}:{}", host, port),
                                bucket: bucket.to_string(),
                                region: "auto".to_string(),
                                access_key: accesskey.to_string(),
                                secret_key: secret.to_string(),
                                server,
                            };
                        }
                    }
                }
            }
        }

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
