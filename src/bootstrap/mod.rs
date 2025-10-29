use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use anyhow::Result;
use diesel::connection::SimpleConnection;
use diesel::RunQueryDsl;
use diesel::{Connection, QueryableByName};
use dotenvy::dotenv;
use log::{debug, error, info, trace};
use opendal::Operator;
use rand::distr::Alphanumeric;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[derive(QueryableByName)]
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
    pub s3_operator: Operator,
}

impl BootstrapManager {
    pub fn new(install_mode: InstallMode, tenant: Option<String>) -> Self {
        info!(
            "Initializing BootstrapManager with mode {:?} and tenant {:?}",
            install_mode, tenant
        );
        let config = AppConfig::from_env();
        let s3_operator = Self::create_s3_operator(&config);
        Self {
            install_mode,
            tenant,
            s3_operator,
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
                let database_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://gbuser:@localhost:5432/botserver".to_string());
                let mut conn = diesel::pg::PgConnection::establish(&database_url)
                    .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
                let default_bot_id: uuid::Uuid = diesel::sql_query("SELECT id FROM bots LIMIT 1")
                    .get_result::<BotIdRow>(&mut conn)
                    .map(|row| row.id)
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

    pub fn bootstrap(&mut self) -> Result<AppConfig> {
        if let Ok(tables_server) = std::env::var("TABLES_SERVER") {
            if !tables_server.is_empty() {
                info!(
                    "Legacy mode detected (TABLES_SERVER present), skipping bootstrap installation"
                );
                let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    let username =
                        std::env::var("TABLES_USERNAME").unwrap_or_else(|_| "postgres".to_string());
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

                match diesel::PgConnection::establish(&database_url) {
                    Ok(mut conn) => {
                        if let Err(e) = self.apply_migrations(&mut conn) {
                            log::warn!("Failed to apply migrations: {}", e);
                        }
                        return Ok(AppConfig::from_database(&mut conn));
                    }
                    Err(e) => {
                        log::warn!("Failed to connect to legacy database: {}", e);
                        return Ok(AppConfig::from_env());
                    }
                }
            }
        }

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
        let required_components = vec!["tables", "drive", "cache"];
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
                    let database_url = std::env::var("DATABASE_URL").unwrap();
                    let mut conn = diesel::PgConnection::establish(&database_url)
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

        self.s3_operator = Self::create_s3_operator(&config);
        let default_bucket_path = Path::new("templates/default.gbai/default.gbot/config.csv");
        if default_bucket_path.exists() {
            info!("Found initial config.csv, uploading to default.gbai/default.gbot");
            let operator = &self.s3_operator;
            futures::executor::block_on(async {
                let content = std::fs::read(default_bucket_path).expect("Failed to read config.csv");
                operator.write("default.gbai/default.gbot/config.csv", content).await
            })?;
            debug!("Initial config.csv uploaded successfully");
        }
        Ok(config)
    }
    fn create_s3_operator(config: &AppConfig) -> Operator {
        use opendal::Scheme;
        use std::collections::HashMap;

        let mut endpoint = config.drive.server.clone();
        if !endpoint.ends_with('/') {
            endpoint.push('/');
        }

        let mut map = HashMap::new();
        map.insert("endpoint".to_string(), endpoint);
        map.insert("access_key_id".to_string(), config.drive.access_key.clone());
        map.insert(
            "secret_access_key".to_string(),
            config.drive.secret_key.clone(),
        );
        map.insert(
            "bucket".to_string(),
            format!("default.gbai"),
        );
        map.insert("region".to_string(), "auto".to_string());
        map.insert("force_path_style".to_string(), "true".to_string());

        trace!("Creating S3 operator with endpoint {}", config.drive.server);

        Operator::via_iter(Scheme::S3, map).expect("Failed to initialize S3 operator")
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
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://gbuser:@localhost:5432/botserver".to_string());
        let mut conn = diesel::pg::PgConnection::establish(&database_url)?;

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

    pub async fn upload_templates_to_drive(&self, config: &AppConfig) -> Result<()> {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| config.database_url());
        let mut conn = diesel::PgConnection::establish(&database_url)?;
        self.create_bots_from_templates(&mut conn)?;
        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }
        let operator = &self.s3_operator;
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
if operator.stat(&bucket).await.is_err() {
    info!("Bucket {} not found, creating it", bucket);
    let bucket_path = if bucket.ends_with('/') { bucket.clone() } else { format!("{}/", bucket) };
match operator.create_dir(&bucket_path).await {
        Ok(_) => {
            debug!("Bucket {} created successfully", bucket);
        }
        Err(e) => {
            let err_msg = format!("{}", e);
            if err_msg.contains("BucketAlreadyOwnedByYou") {
                log::warn!("Bucket {} already exists, reusing default.gbai", bucket);
                self.upload_directory_recursive(&operator, &Path::new("templates/default.gbai"), "default.gbai").await?;
                continue;
            } else {
                return Err(e.into());
            }
        }
    }
}
self.upload_directory_recursive(&operator, &path, &bucket).await?;
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
                let formatted_name = bot_name
                    .split('_')
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                let existing: Option<String> = bots::table
                    .filter(bots::name.eq(&formatted_name))
                    .select(bots::name)
                    .first(conn)
                    .optional()?;

                if existing.is_none() {
                    diesel::sql_query(
                        "INSERT INTO bots (id, name, description, llm_provider, llm_config, context_provider, context_config, is_active) \
                         VALUES (gen_random_uuid(), $1, $2, 'openai', '{\"model\": \"gpt-4\", \"temperature\": 0.7}', 'database', '{}', true)"
                    )
                    .bind::<diesel::sql_types::Text, _>(&formatted_name)
                    .bind::<diesel::sql_types::Text, _>(format!("Bot for {} template", bot_name))
                    .execute(conn)?;
                } else {
                    log::trace!("Bot {} already exists", formatted_name);
                }
            }
        }

        Ok(())
    }

    fn upload_directory_recursive<'a>(
        &'a self,
        client: &'a Operator,
        local_path: &'a Path,
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
                let key = if prefix.is_empty() {
                    file_name.clone()
                } else {
                    format!("{}/{}", prefix.trim_end_matches('/'), file_name)
                };

                if path.is_file() {
                    info!("Uploading file: {} with key: {}", path.display(), key);
                    let content = std::fs::read(&path)?;
                    trace!("Writing file {} with key {}", path.display(), key);
                    client.write(&key, content).await?;
                    trace!("Successfully wrote file {}", path.display());
                } else if path.is_dir() {
                    self.upload_directory_recursive(client, &path, &key).await?;
                }
            }
            Ok(())
        })
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
