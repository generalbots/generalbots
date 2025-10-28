use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use actix_web::http::uri::Builder;
use anyhow::Result;
use diesel::connection::SimpleConnection;
use diesel::Connection;
use diesel::RunQueryDsl;
use dotenvy::dotenv;
use log::{error, info};
use opendal::services::S3;
use opendal::Operator;
use rand::Rng;
use rand::distr::Alphanumeric;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub struct ComponentInfo {
    pub name: &'static str,
    pub termination_command: &'static str,
}

pub struct BootstrapManager {
    pub install_mode: InstallMode,
    pub tenant: Option<String>,
}

impl BootstrapManager {
    pub fn new(install_mode: InstallMode, tenant: Option<String>) -> Self {
        info!(
            "Initializing BootstrapManager with mode {:?} and tenant {:?}",
            install_mode, tenant
        );
        Self {
            install_mode,
            tenant,
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
                if let Err(e) = self.update_bot_config(component.name) {
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

        Ok(config)
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

    fn update_bot_config(&self, component: &str) -> Result<()> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://gbuser:@localhost:5432/botserver".to_string());
        let mut conn = diesel::pg::PgConnection::establish(&database_url)?;

        for (k, v) in vec![(component.to_string(), "true".to_string())] {
            diesel::sql_query(
                "INSERT INTO bot_config (key, value) VALUES ($1, $2) \
                 ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
            )
            .bind::<diesel::sql_types::Text, _>(&k)
            .bind::<diesel::sql_types::Text, _>(&v)
            .execute(&mut conn)?;
        }

        Ok(())
    }

    pub async fn upload_templates_to_minio(&self, config: &AppConfig) -> Result<()> {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| config.database_url());
        let mut conn = diesel::PgConnection::establish(&database_url)?;
        self.create_bots_from_templates(&mut conn)?;

        let client = Operator::new(
            S3::default()
                .root("/")
                .endpoint(&config.minio.server)
                .access_key_id(&config.minio.access_key)
                .secret_access_key(&config.minio.secret_key)
        )?.finish();

        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.extension().map(|e| e == "gbai").unwrap_or(false) {
                let bot_name = path.file_name().unwrap().to_string_lossy().to_string();

                self.upload_directory_recursive(&client, &path, &bot_name, "")
                    .await?;
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
        bucket: &'a str,
        prefix: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            for entry in std::fs::read_dir(local_path)? {
                let entry = entry?;
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let key = if prefix.is_empty() {
                    file_name.clone()
                } else {
                    format!("{}/{}", prefix, file_name)
                };

                if path.is_file() {
                    info!(
                        "Uploading file: {} to bucket: {} with key: {}",
                        path.display(),
                        bucket,
                        key
                    );
                    let content = std::fs::read(&path)?;
                    client.write(&key, content).await?;
                } else if path.is_dir() {
                    self.upload_directory_recursive(client, &path, bucket, &key)
                        .await?;
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
