use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use crate::shared::utils::establish_pg_connection;
use anyhow::Result;
use diesel::{connection::SimpleConnection, RunQueryDsl, QueryableByName};
use dotenvy::dotenv;
use log::{debug, error, info, trace};
use aws_sdk_s3::Client;
use aws_config::BehaviorVersion;
use rand::distr::Alphanumeric;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

use diesel::Queryable;

#[derive(QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Queryable)]
#[diesel(table_name = crate::shared::models::schema::bots)]
struct BotIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
}

pub struct ComponentInfo {
    pub name: &'static str,
    pub termination_command: &'static str,
}

pub struct BootstrapManager {
    pub install_mode: InstallMode,
    pub tenant: Option<String>,
    pub s3_client: Client,
}

impl BootstrapManager {
    pub async fn new(install_mode: InstallMode, tenant: Option<String>) -> Self {
        info!(
            "Initializing BootstrapManager with mode {:?} and tenant {:?}",
            install_mode, tenant
        );
        let config = AppConfig::from_env();
        let s3_client = futures::executor::block_on(Self::create_s3_operator(&config));
        Self {
            install_mode,
            tenant,
            s3_client,
        }
    }

    pub fn start_all(&mut self) -> Result<()> {
        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
        let components = vec![
            ComponentInfo {
                name: "tables",
                termination_command: "pg_ctl",
            },
            ComponentInfo {
                name: "cache",
                termination_command: "valkey-server",
            },
            ComponentInfo {
                name: "drive",
                termination_command: "minio",
            },
            ComponentInfo {
                name: "llm",
                termination_command: "llama-server",
            },
            ComponentInfo {
                name: "email",
                termination_command: "stalwart",
            },
            ComponentInfo {
                name: "proxy",
                termination_command: "caddy",
            },
            ComponentInfo {
                name: "directory",
                termination_command: "zitadel",
            },
            ComponentInfo {
                name: "alm",
                termination_command: "forgejo",
            },
            ComponentInfo {
                name: "alm_ci",
                termination_command: "forgejo-runner",
            },
            ComponentInfo {
                name: "dns",
                termination_command: "coredns",
            },
            ComponentInfo {
                name: "webmail",
                termination_command: "php",
            },
            ComponentInfo {
                name: "meeting",
                termination_command: "livekit-server",
            },
            ComponentInfo {
                name: "table_editor",
                termination_command: "nocodb",
            },
            ComponentInfo {
                name: "doc_editor",
                termination_command: "coolwsd",
            },
            ComponentInfo {
                name: "desktop",
                termination_command: "xrdp",
            },
            ComponentInfo {
                name: "devtools",
                termination_command: "",
            },
            ComponentInfo {
                name: "bot",
                termination_command: "",
            },
            ComponentInfo {
                name: "system",
                termination_command: "",
            },
            ComponentInfo {
                name: "vector_db",
                termination_command: "qdrant",
            },
            ComponentInfo {
                name: "host",
                termination_command: "",
            },
        ];

        for component in components {
            if pm.is_installed(component.name) {
                pm.start(component.name)?;
            } else {
                let mut conn = establish_pg_connection()?;
                let default_bot_id: uuid::Uuid = diesel::sql_query("SELECT id FROM bots LIMIT 1")
                    .load::<BotIdRow>(&mut conn)
                    .map(|rows| rows.first().map(|r| r.id).unwrap_or_else(|| uuid::Uuid::new_v4()))
                    .unwrap_or_else(|_| uuid::Uuid::new_v4());

                if let Err(e) = self.update_bot_config(&default_bot_id, component.name) {
                    error!(
                        "Failed to update bot config after installing {}: {}",
                        component.name, e
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn bootstrap(&mut self) -> Result<AppConfig> {
        // First check for legacy mode
        if let Ok(tables_server) = std::env::var("TABLES_SERVER") {
            if !tables_server.is_empty() {
                info!(
                    "Legacy mode detected (TABLES_SERVER present), skipping bootstrap installation"
                );
                let _database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    let username =
                        std::env::var("TABLES_USERNAME").unwrap_or_else(|_| "gbuser".to_string());
                    let password =
                        std::env::var("TABLES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
                    let server =
                        std::env::var("TABLES_SERVER").unwrap_or_else(|_| "localhost".to_string());
                    let port = std::env::var("TABLES_PORT").unwrap_or_else(|_| "5432".to_string());
                    let database =
                        std::env::var("TABLES_DATABASE").unwrap_or_else(|_| "gbserver".to_string());
                    format!(
                        "postgres://{}:{}@{}:{}/{}",
                        username, password, server, port, database
                    )
                });

                // In legacy mode, still try to load config.csv if available
                if let Ok(config) = self.load_config_from_csv().await {
                    return Ok(config);
                }

                match establish_pg_connection() {
                    Ok(mut conn) => {
                        if let Err(e) = self.apply_migrations(&mut conn) {
                            log::warn!("Failed to apply migrations: {}", e);
                        }
                        return Ok(AppConfig::from_database(&mut conn));
                    }
                    Err(e) => {
                        log::warn!("Failed to connect to database: {}", e);
                        return Ok(AppConfig::from_env());
                    }
                }
            }
        }

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
        let required_components = vec!["tables", "drive", "cache", "llm"];
        let mut config = AppConfig::from_env();

        for component in required_components {
            if !pm.is_installed(component) {
                let termination_cmd = pm
                    .components
                    .get(component)
                    .and_then(|cfg| cfg.binary_name.clone())
                    .unwrap_or_else(|| component.to_string());

                if !termination_cmd.is_empty() {
                    let check = Command::new("pgrep")
                        .arg("-f")
                        .arg(&termination_cmd)
                        .output();
                    if let Ok(output) = check {
                        if !output.stdout.is_empty() {
                            println!("Component '{}' appears to be already running from a previous install.", component);
                            println!("Do you want to terminate it? (y/n)");
                            let mut input = String::new();
                            io::stdout().flush().unwrap();
                            io::stdin().read_line(&mut input).unwrap();
                            if input.trim().eq_ignore_ascii_case("y") {
                                let _ = Command::new("pkill")
                                    .arg("-f")
                                    .arg(&termination_cmd)
                                    .status();
                                println!("Terminated existing '{}' process.", component);
                            } else {
                                println!(
                                    "Skipping start of '{}' as it is already running.",
                                    component
                                );
                                continue;
                            }
                        }
                    }
                }

                if component == "tables" {
                    let db_password = self.generate_secure_password(16);
                    let farm_password = self.generate_secure_password(32);
                    let env_contents = format!(
                        "FARM_PASSWORD={}\nDATABASE_URL=postgres://gbuser:{}@localhost:5432/botserver",
                        farm_password, db_password
                    );
                    std::fs::write(".env", &env_contents)
                        .map_err(|e| anyhow::anyhow!("Failed to write .env file: {}", e))?;
                    dotenv().ok();
                }

                futures::executor::block_on(pm.install(component))?;

                if component == "tables" {
                    let mut conn = establish_pg_connection()
                        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

                    let migration_dir = include_dir::include_dir!("./migrations");
                    let mut migration_files: Vec<_> = migration_dir
                        .files()
                        .filter_map(|file| {
                            let path = file.path();
                            if path.extension()? == "sql" {
                                Some(file)
                            } else {
                                None
                            }
                        })
                        .collect();

                    migration_files.sort_by_key(|f| f.path());

                    for migration_file in migration_files {
                        let migration = migration_file
                            .contents_utf8()
                            .ok_or_else(|| anyhow::anyhow!("Migration file is not valid UTF-8"))?;

                        if let Err(e) = conn.batch_execute(migration) {
                            log::error!(
                                "Failed to execute migration {}: {}",
                                migration_file.path().display(),
                                e
                            );
                            return Err(e.into());
                        }
                        info!(
                            "Successfully executed migration: {}",
                            migration_file.path().display()
                        );
                    }

                    config = AppConfig::from_database(&mut conn);
                }
            }
        }

        self.s3_client = futures::executor::block_on(Self::create_s3_operator(&config));
        
        // Load config from CSV if available
        if let Ok(csv_config) = self.load_config_from_csv().await {
            Ok(csv_config)
        } else {
            Ok(config)
        }
    }



    async fn create_s3_operator(config: &AppConfig) -> Client {
        let endpoint = if !config.drive.server.ends_with('/') {
            format!("{}/", config.drive.server)
        } else {
            config.drive.server.clone()
        };

        let base_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region("auto")
            .credentials_provider(
                aws_sdk_s3::config::Credentials::new(
                    config.drive.access_key.clone(),
                    config.drive.secret_key.clone(),
                    None,
                    None,
                    "static",
                )
            )
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&base_config)
            .force_path_style(true)
            .build();

        aws_sdk_s3::Client::from_conf(s3_config)
    }




    fn generate_secure_password(&self, length: usize) -> String {
        let mut rng = rand::rng();
        std::iter::repeat_with(|| rng.sample(Alphanumeric) as char)
            .take(length)
            .collect()
    }

    fn encrypt_password(&self, password: &str, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn update_bot_config(&self, bot_id: &uuid::Uuid, component: &str) -> Result<()> {
        use diesel::sql_types::{Text, Uuid as SqlUuid};
        let mut conn = establish_pg_connection()?;

        // Ensure globally unique keys and update values atomically
        let config_key = format!("{}_{}", bot_id, component);
        let config_value = "true".to_string();
        let new_id = uuid::Uuid::new_v4();

        diesel::sql_query(
            "INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type)
             VALUES ($1, $2, $3, $4, 'string')
             ON CONFLICT (config_key)
             DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()",
        )
        .bind::<SqlUuid, _>(new_id)
        .bind::<SqlUuid, _>(bot_id)
        .bind::<Text, _>(&config_key)
        .bind::<Text, _>(&config_value)
        .execute(&mut conn)?;

        Ok(())
    }

    pub async fn upload_templates_to_drive(&self, _config: &AppConfig) -> Result<()> {
                let mut conn = establish_pg_connection()?;
        self.create_bots_from_templates(&mut conn)?;
        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }
        let client = &self.s3_client;
        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".gbai")
            {
                let bot_name = path.file_name().unwrap().to_string_lossy().to_string();
                let bucket = bot_name.trim_start_matches('/').to_string();
                info!("Uploading template {} to Drive bucket {}", bot_name, bucket);
                
                // Check if bucket exists
                if client.head_bucket().bucket(&bucket).send().await.is_err() {
                    info!("Bucket {} not found, creating it", bucket);
                    match client.create_bucket()
                        .bucket(&bucket)
                        .send()
                        .await {
                        Ok(_) => {
                            debug!("Bucket {} created successfully", bucket);
                        }
                        Err(e) => {
                            error!("Failed to create bucket {}: {:?}", bucket, e);
                            return Err(anyhow::anyhow!(
                                "Failed to create bucket {}: {}. Check S3 credentials and endpoint configuration",
                                bucket, e
                            ));
                        }
                    }
                }
                
                self.upload_directory_recursive(client, &path, &bucket, "/")
                    .await?;
                info!("Uploaded template {} to Drive bucket {}", bot_name, bucket);
            }
        }
        Ok(())
    }

    fn create_bots_from_templates(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use crate::shared::models::schema::bots;
        use diesel::prelude::*;

        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.extension().map(|e| e == "gbai").unwrap_or(false) {
                let bot_folder = path.file_name().unwrap().to_string_lossy().to_string();
                let bot_name = bot_folder.trim_end_matches(".gbai");

                let existing: Option<String> = bots::table
                    .filter(bots::name.eq(&bot_name))
                    .select(bots::name)
                    .first(conn)
                    .optional()?;

                if existing.is_none() {
                    diesel::sql_query(
                        "INSERT INTO bots (id, name, description, llm_provider, llm_config, context_provider, context_config, is_active) \
                         VALUES (gen_random_uuid(), $1, $2, 'openai', '{\"model\": \"gpt-4\", \"temperature\": 0.7}', 'database', '{}', true)"
                    )
                    .bind::<diesel::sql_types::Text, _>(&bot_name)
                    .bind::<diesel::sql_types::Text, _>(format!("Bot for {} template", bot_name))
                    .execute(conn)?;
                } else {
                    log::trace!("Bot {} already exists", bot_name);
                }
            }
        }

        Ok(())
    }

    fn upload_directory_recursive<'a>(
        &'a self,
        client: &'a Client,
        local_path: &'a Path,
        bucket: &'a str,
        prefix: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let normalized_path = if !local_path.to_string_lossy().ends_with('/') {
                format!("{}/", local_path.to_string_lossy())
            } else {
                local_path.to_string_lossy().to_string()
            };
            trace!("Starting upload from local path: {}", normalized_path);
            for entry in std::fs::read_dir(local_path)? {
                let entry = entry?;
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                
                // Construct key path, ensuring no duplicate slashes
                let mut key = prefix.trim_matches('/').to_string();
                if !key.is_empty() {
                    key.push('/');
                }
                key.push_str(&file_name);

                if path.is_file() {
                    info!("Uploading file: {} to bucket {} with key: {}", 
                        path.display(), bucket, key);
                    let content = std::fs::read(&path)?;
                    client.put_object()
                        .bucket(bucket)
                        .key(&key)
                        .body(content.into())
                        .send()
                        .await?;
                } else if path.is_dir() {
                    self.upload_directory_recursive(client, &path, bucket, &key).await?;
                }
            }
            Ok(())
        })
    }

    async fn load_config_from_csv(&self) -> Result<AppConfig> {
        use crate::config::ConfigManager;
        use uuid::Uuid;

        let client = &self.s3_client;
        let bucket = "default.gbai";
        let config_key = "default.gbot/config.csv";
        
        match client.get_object()
            .bucket(bucket)
            .key(config_key)
            .send()
            .await 
        {
            Ok(response) => {
                let bytes = response.body.collect().await?.into_bytes();
                let csv_content = String::from_utf8(bytes.to_vec())?;
                
                // Create new connection for config loading
                let config_conn = establish_pg_connection()?;
                let config_manager = ConfigManager::new(Arc::new(Mutex::new(config_conn)));
                
                // Use default bot ID or create one if needed
                let default_bot_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")?;
                
                // Write CSV to temp file for ConfigManager
                let temp_path = std::env::temp_dir().join("config.csv");
                std::fs::write(&temp_path, csv_content)?;
                
                // First sync the CSV to database
                config_manager.sync_gbot_config(&default_bot_id, temp_path.to_str().unwrap())
                    .map_err(|e| anyhow::anyhow!("Failed to sync gbot config: {}", e))?;
                
                // Create fresh connection for final config load
                let mut final_conn = establish_pg_connection()?;
                let config = AppConfig::from_database(&mut final_conn);
                info!("Successfully loaded config from CSV with LLM settings");
                Ok(config)
            }
            Err(e) => {
                debug!("No config.csv found: {}", e);
                Err(e.into())
            }
        }
    }

    fn apply_migrations(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        let migrations_dir = std::path::Path::new("migrations");
        if !migrations_dir.exists() {
            return Ok(());
        }

        let mut sql_files: Vec<_> = std::fs::read_dir(migrations_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "sql")
                    .unwrap_or(false)
            })
            .collect();

        sql_files.sort_by_key(|entry| entry.path());

        for entry in sql_files {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy();
            match std::fs::read_to_string(&path) {
                Ok(sql) => match conn.batch_execute(&sql) {
                    Err(e) => {
                        log::warn!("Migration {} failed: {}", filename, e);
                    }
                    _ => {}
                },
                Err(e) => {
                    log::warn!("Failed to read migration {}: {}", filename, e);
                }
            }
        }

        Ok(())
    }
}
