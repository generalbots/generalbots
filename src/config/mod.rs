use diesel::prelude::*;
use diesel::sql_types::Text;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LLMConfig {
    pub url: String,
    pub key: String,
    pub model: String,
}

#[derive(Clone)]
pub struct AppConfig {
    pub drive: DriveConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub database_custom: DatabaseConfig,
    pub email: EmailConfig,
    pub llm: LLMConfig,
    pub embedding: LLMConfig,
    pub site_path: String,
    pub stack_path: PathBuf,
    pub db_conn: Option<Arc<Mutex<PgConnection>>>,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub server: String,
    pub port: u32,
    pub database: String,
}

#[derive(Clone)]
pub struct DriveConfig {
    pub server: String,
    pub access_key: String,
    pub secret_key: String,
    pub use_ssl: bool,
    pub org_prefix: String,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct EmailConfig {
    pub from: String,
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
pub struct ServerConfigRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub config_key: String,
    #[diesel(sql_type = Text)]
    pub config_value: String,
    #[diesel(sql_type = Text)]
    pub config_type: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_encrypted: bool,
}

impl AppConfig {
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.server,
            self.database.port,
            self.database.database
        )
    }

    pub fn database_custom_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database_custom.username,
            self.database_custom.password,
            self.database_custom.server,
            self.database_custom.port,
            self.database_custom.database
        )
    }

    pub fn component_path(&self, component: &str) -> PathBuf {
        self.stack_path.join(component)
    }

    pub fn bin_path(&self, component: &str) -> PathBuf {
        self.stack_path.join("bin").join(component)
    }

    pub fn data_path(&self, component: &str) -> PathBuf {
        self.stack_path.join("data").join(component)
    }

    pub fn config_path(&self, component: &str) -> PathBuf {
        self.stack_path.join("conf").join(component)
    }

    pub fn log_path(&self, component: &str) -> PathBuf {
        self.stack_path.join("logs").join(component)
    }

    pub fn from_database(conn: &mut PgConnection) -> Self {
        info!("Loading configuration from database");

        let config_map = match Self::load_config_from_db(conn) {
            Ok(map) => {
                info!("Loaded {} config values from database", map.len());
                map
            }
            Err(e) => {
                warn!("Failed to load config from database: {}. Using defaults", e);
                HashMap::new()
            }
        };

        let get_str = |key: &str, default: &str| -> String {
            config_map
                .get(key)
                .map(|v| v.config_value.clone())
                .unwrap_or_else(|| default.to_string())
        };

        let get_u32 = |key: &str, default: u32| -> u32 {
            config_map
                .get(key)
                .and_then(|v| v.config_value.parse().ok())
                .unwrap_or(default)
        };

        let get_u16 = |key: &str, default: u16| -> u16 {
            config_map
                .get(key)
                .and_then(|v| v.config_value.parse().ok())
                .unwrap_or(default)
        };

        let get_bool = |key: &str, default: bool| -> bool {
            config_map
                .get(key)
                .map(|v| v.config_value.to_lowercase() == "true")
                .unwrap_or(default)
        };

        let stack_path = PathBuf::from(get_str("STACK_PATH", "./botserver-stack"));

        // For database credentials, prioritize environment variables over database values
        // because we need the correct credentials to connect to the database in the first place
        let database = DatabaseConfig {
            username: std::env::var("TABLES_USERNAME")
                .unwrap_or_else(|_| get_str("TABLES_USERNAME", "gbuser")),
            password: std::env::var("TABLES_PASSWORD")
                .unwrap_or_else(|_| get_str("TABLES_PASSWORD", "")),
            server: std::env::var("TABLES_SERVER")
                .unwrap_or_else(|_| get_str("TABLES_SERVER", "localhost")),
            port: std::env::var("TABLES_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(|| get_u32("TABLES_PORT", 5432)),
            database: std::env::var("TABLES_DATABASE")
                .unwrap_or_else(|_| get_str("TABLES_DATABASE", "botserver")),
        };

        let database_custom = DatabaseConfig {
            username: std::env::var("CUSTOM_USERNAME")
                .unwrap_or_else(|_| get_str("CUSTOM_USERNAME", "gbuser")),
            password: std::env::var("CUSTOM_PASSWORD")
                .unwrap_or_else(|_| get_str("CUSTOM_PASSWORD", "")),
            server: std::env::var("CUSTOM_SERVER")
                .unwrap_or_else(|_| get_str("CUSTOM_SERVER", "localhost")),
            port: std::env::var("CUSTOM_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(|| get_u32("CUSTOM_PORT", 5432)),
            database: std::env::var("CUSTOM_DATABASE")
                .unwrap_or_else(|_| get_str("CUSTOM_DATABASE", "gbuser")),
        };

        let minio = DriveConfig {
            server: {
                let server = get_str("DRIVE_SERVER", "http://localhost:9000");
                if !server.starts_with("http://") && !server.starts_with("https://") {
                    format!("http://{}", server)
                } else {
                    server
                }
            },
            access_key: get_str("DRIVE_ACCESSKEY", "minioadmin"),
            secret_key: get_str("DRIVE_SECRET", "minioadmin"),
            use_ssl: get_bool("DRIVE_USE_SSL", false),
            org_prefix: get_str("DRIVE_ORG_PREFIX", "pragmatismo-"),
        };

        let email = EmailConfig {
            from: get_str("EMAIL_FROM", "noreply@example.com"),
            server: get_str("EMAIL_SERVER", "smtp.example.com"),
            port: get_u16("EMAIL_PORT", 587),
            username: get_str("EMAIL_USER", "user"),
            password: get_str("EMAIL_PASS", "pass"),
        };

        // Write drive config to .env file
        if let Err(e) = write_drive_config_to_env(&minio) {
            warn!("Failed to write drive config to .env: {}", e);
        }

        AppConfig {
            drive: minio,
            server: ServerConfig {
                host: get_str("SERVER_HOST", "127.0.0.1"),
                port: get_u16("SERVER_PORT", 8080),
            },
            database,
            database_custom,
            email,
            llm: LLMConfig {
                url: get_str("LLM_URL", "http://localhost:8081"),
                key: get_str("LLM_KEY", ""),
                model: get_str("LLM_MODEL", "gpt-4"),
            },
            embedding: LLMConfig {
                url: get_str("EMBEDDING_URL", "http://localhost:8082"),
                key: get_str("EMBEDDING_KEY", ""),
                model: get_str("EMBEDDING_MODEL", "text-embedding-ada-002"),
            },
            site_path: get_str("SITES_ROOT", "./botserver-stack/sites"),
            stack_path,
            db_conn: None,
        }
    }

    pub fn from_env() -> Self {
        info!("Loading configuration from environment variables");

        let stack_path =
            std::env::var("STACK_PATH").unwrap_or_else(|_| "./botserver-stack".to_string());

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://gbuser:@localhost:5432/botserver".to_string());
        let (db_username, db_password, db_server, db_port, db_name) =
            parse_database_url(&database_url);

        let database = DatabaseConfig {
            username: db_username,
            password: db_password,
            server: db_server,
            port: db_port,
            database: db_name,
        };

        let database_custom = DatabaseConfig {
            username: std::env::var("CUSTOM_USERNAME").unwrap_or_else(|_| "gbuser".to_string()),
            password: std::env::var("CUSTOM_PASSWORD").unwrap_or_else(|_| "".to_string()),
            server: std::env::var("CUSTOM_SERVER").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("CUSTOM_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            database: std::env::var("CUSTOM_DATABASE").unwrap_or_else(|_| "botserver".to_string()),
        };

        let minio = DriveConfig {
            server: std::env::var("DRIVE_SERVER")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("DRIVE_ACCESSKEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("DRIVE_SECRET").unwrap_or_else(|_| "minioadmin".to_string()),
            use_ssl: std::env::var("DRIVE_USE_SSL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            org_prefix: std::env::var("DRIVE_ORG_PREFIX")
                .unwrap_or_else(|_| "pragmatismo-".to_string()),
        };

        let email = EmailConfig {
            from: std::env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@example.com".to_string()),
            server: std::env::var("EMAIL_SERVER")
                .unwrap_or_else(|_| "smtp.example.com".to_string()),
            port: std::env::var("EMAIL_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            username: std::env::var("EMAIL_USER").unwrap_or_else(|_| "user".to_string()),
            password: std::env::var("EMAIL_PASS").unwrap_or_else(|_| "pass".to_string()),
        };

        AppConfig {
            drive: minio,
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
            },
            database,
            database_custom,
            email,
            llm: LLMConfig {
                url: std::env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:8081".to_string()),
                key: std::env::var("LLM_KEY").unwrap_or_else(|_| "".to_string()),
                model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
            },
            embedding: LLMConfig {
                url: std::env::var("EMBEDDING_URL").unwrap_or_else(|_| "http://localhost:8082".to_string()),
                key: std::env::var("EMBEDDING_KEY").unwrap_or_else(|_| "".to_string()),
                model: std::env::var("EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-ada-002".to_string()),
            },
            site_path: std::env::var("SITES_ROOT")
                .unwrap_or_else(|_| "./botserver-stack/sites".to_string()),
            stack_path: PathBuf::from(stack_path),
            db_conn: None,
        }
    }

    fn load_config_from_db(
        conn: &mut PgConnection,
    ) -> Result<HashMap<String, ServerConfigRow>, diesel::result::Error> {
        let results = diesel::sql_query("SELECT id, config_key, config_value, config_type, is_encrypted FROM server_configuration").load::<ServerConfigRow>(conn)?;

        let mut map = HashMap::new();
        for row in results {
            map.insert(row.config_key.clone(), row);
        }

        Ok(map)
    }

    pub fn set_config(
        &self,
        conn: &mut PgConnection,
        key: &str,
        value: &str,
    ) -> Result<(), diesel::result::Error> {
        diesel::sql_query("SELECT set_config($1, $2)")
            .bind::<Text, _>(key)
            .bind::<Text, _>(value)
            .execute(conn)?;
        info!("Updated configuration: {} = {}", key, value);
        Ok(())
    }

    pub fn get_config(
        &self,
        conn: &mut PgConnection,
        key: &str,
        fallback: Option<&str>,
    ) -> Result<String, diesel::result::Error> {
        let fallback_str = fallback.unwrap_or("");

        #[derive(Debug, QueryableByName)]
        struct ConfigValue {
            #[diesel(sql_type = Text)]
            value: String,
        }

        let result = diesel::sql_query("SELECT get_config($1, $2) as value")
            .bind::<Text, _>(key)
            .bind::<Text, _>(fallback_str)
            .get_result::<ConfigValue>(conn)
            .map(|row| row.value)?;
        Ok(result)
    }
}

fn write_drive_config_to_env(drive: &DriveConfig) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(".env")?;
    
    writeln!(file,"")?;
    writeln!(file, "DRIVE_SERVER={}", drive.server)?;
    writeln!(file, "DRIVE_ACCESSKEY={}", drive.access_key)?;
    writeln!(file, "DRIVE_SECRET={}", drive.secret_key)?;
    writeln!(file, "DRIVE_USE_SSL={}", drive.use_ssl)?;
    writeln!(file, "DRIVE_ORG_PREFIX={}", drive.org_prefix)?;

    Ok(())
}

fn parse_database_url(url: &str) -> (String, String, String, u32, String) {
    if let Some(stripped) = url.strip_prefix("postgres://") {
        let parts: Vec<&str> = stripped.split('@').collect();
        if parts.len() == 2 {
            let user_pass: Vec<&str> = parts[0].split(':').collect();
            let host_db: Vec<&str> = parts[1].split('/').collect();

            if user_pass.len() >= 2 && host_db.len() >= 2 {
                let username = user_pass[0].to_string();
                let password = user_pass[1].to_string();

                let host_port: Vec<&str> = host_db[0].split(':').collect();
                let server = host_port[0].to_string();
                let port = host_port
                    .get(1)
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(5432);
                let database = host_db[1].to_string();

                return (username, password, server, port, database);
            }
        }
    }

    (
        "gbuser".to_string(),
        "".to_string(),
        "localhost".to_string(),
        5432,
        "botserver".to_string(),
    )
}

pub struct ConfigManager {
    conn: Arc<Mutex<PgConnection>>,
}

impl ConfigManager {
    pub fn new(conn: Arc<Mutex<PgConnection>>) -> Self {
        Self { conn }
    }

    pub fn get_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        fallback: Option<&str>,
    ) -> Result<String, diesel::result::Error> {
        let mut conn = self.conn.lock().unwrap();
        let fallback_str = fallback.unwrap_or("");

        #[derive(Debug, QueryableByName)]
        struct ConfigValue {
            #[diesel(sql_type = Text)]
            value: String,
        }

        let result = diesel::sql_query(
            "SELECT get_bot_config($1, $2, $3) as value"
        )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<Text, _>(key)
            .bind::<Text, _>(fallback_str)
            .get_result::<ConfigValue>(&mut *conn)
            .map(|row| row.value)?;
        Ok(result)
    }

    pub fn sync_gbot_config(
        &self,
        bot_id: &uuid::Uuid,
        config_path: &str,
    ) -> Result<usize, String> {
        use sha2::{Digest, Sha256};
        use std::fs;

        let content = fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let file_hash = format!("{:x}", hasher.finalize());

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        #[derive(QueryableByName)]
        struct SyncHash {
            #[diesel(sql_type = Text)]
            file_hash: String,
        }

        let last_hash: Option<String> =
            diesel::sql_query("SELECT file_hash FROM gbot_config_sync WHERE bot_id = $1")
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .get_result::<SyncHash>(&mut *conn)
                .optional()
                .map_err(|e| format!("Database error: {}", e))?
                .map(|row| row.file_hash);

        if last_hash.as_ref() == Some(&file_hash) {
            info!("Config file unchanged for bot {}", bot_id);
            return Ok(0);
        }

        let mut updated = 0;
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();

let new_id: uuid::Uuid = uuid::Uuid::new_v4();
diesel::sql_query("INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type) VALUES ($1, $2, $3, $4, 'string') ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()")
    .bind::<diesel::sql_types::Uuid, _>(new_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(key)
    .bind::<diesel::sql_types::Text, _>(value)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to update config: {}", e))?;

                updated += 1;
            }
        }

        diesel::sql_query("INSERT INTO gbot_config_sync (id, bot_id, config_file_path, file_hash, sync_count) VALUES (gen_random_uuid()::text, $1, $2, $3, 1) ON CONFLICT (bot_id) DO UPDATE SET last_sync_at = NOW(), file_hash = EXCLUDED.file_hash, sync_count = gbot_config_sync.sync_count + 1")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(config_path)
            .bind::<diesel::sql_types::Text, _>(&file_hash)
            .execute(&mut *conn)
            .map_err(|e| format!("Failed to update sync record: {}", e))?;

        info!(
            "Synced {} config values for bot {} from {}",
            updated, bot_id, config_path
        );
        Ok(updated)
    }
}
