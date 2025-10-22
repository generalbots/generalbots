use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use anyhow::Result;
use diesel::connection::SimpleConnection;
use diesel::Connection;
use dotenvy::dotenv;
use log::{info, trace};
use rand::distr::Alphanumeric;
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct BootstrapManager {
    pub install_mode: InstallMode,
    pub tenant: Option<String>,
}

impl BootstrapManager {
    pub fn new(install_mode: InstallMode, tenant: Option<String>) -> Self {
        trace!(
            "Initializing BootstrapManager with mode {:?} and tenant {:?}",
            install_mode,
            tenant
        );
        Self {
            install_mode,
            tenant,
        }
    }

    pub fn start_all(&mut self) -> Result<()> {
        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;
        let components = vec![
            "tables",
            "cache",
            "drive",
            "llm",
            "email",
            "proxy",
            "directory",
            "alm",
            "alm_ci",
            "dns",
            "webmail",
            "meeting",
            "table_editor",
            "doc_editor",
            "desktop",
            "devtools",
            "bot",
            "system",
            "vector_db",
            "host",
        ];

        for component in components {
            if pm.is_installed(component) {
                trace!("Starting component: {}", component);
                pm.start(component)?;
            } else {
                trace!("Component {} not installed, skipping start", component);
            }
        }
        Ok(())
    }

    pub fn bootstrap(&mut self) -> Result<AppConfig> {
        // Check for legacy mode - if TABLES_SERVER is present, skip bootstrap
        if let Ok(tables_server) = std::env::var("TABLES_SERVER") {
            if !tables_server.is_empty() {
                trace!(
                    "Legacy mode detected (TABLES_SERVER present), skipping bootstrap installation"
                );
                info!("Running in legacy mode with existing database configuration");

                // Try to connect to the database and load config
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
                        info!("Successfully connected to legacy database, loading configuration");

                        // Apply migrations
                        if let Err(e) = self.apply_migrations(&mut conn) {
                            log::warn!("Failed to apply migrations: {}", e);
                        }

                        return Ok(AppConfig::from_database(&mut conn));
                    }
                    Err(e) => {
                        log::warn!("Failed to connect to legacy database: {}", e);
                        info!("Using environment variables as fallback");
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
                    trace!("Generated database credentials and wrote to .env file");
                }

                trace!("Installing required component: {}", component);
                futures::executor::block_on(pm.install(component))?;

                if component == "tables" {
                    trace!("Component {} installed successfully", component);

                    let database_url = std::env::var("DATABASE_URL").unwrap();
                    let mut conn = diesel::PgConnection::establish(&database_url)
                        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

                    let migration_dir = include_dir::include_dir!("./migrations");
                    let mut migration_files: Vec<_> = migration_dir
                        .files()
                        .filter_map(|file| {
                            let path = file.path();
                            trace!("Found file: {:?}", path);
                            if path.extension()? == "sql" {
                                trace!("  -> SQL file included");
                                Some(file)
                            } else {
                                trace!("  -> Not a SQL file, skipping");
                                None
                            }
                        })
                        .collect();

                    trace!("Total migration files found: {}", migration_files.len());
                    migration_files.sort_by_key(|f| f.path());

                    for migration_file in migration_files {
                        let migration = migration_file
                            .contents_utf8()
                            .ok_or_else(|| anyhow::anyhow!("Migration file is not valid UTF-8"))?;
                        trace!("Executing migration: {}", migration_file.path().display());

                        // Use batch_execute to handle multiple statements including those with dollar-quoted strings
                        if let Err(e) = conn.batch_execute(migration) {
                            log::error!(
                                "Failed to execute migration {}: {}",
                                migration_file.path().display(),
                                e
                            );
                            return Err(e.into());
                        }
                        trace!(
                            "Successfully executed migration: {}",
                            migration_file.path().display()
                        );
                    }

                    config = AppConfig::from_database(&mut conn);
                    info!("Database migrations completed and configuration loaded");
                }
            }
        }

        Ok(config)
    }

    fn generate_secure_password(&self, length: usize) -> String {
        // Ensure the Rng trait is in scope for `sample`
        use rand::Rng;
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

    pub async fn upload_templates_to_minio(&self, config: &AppConfig) -> Result<()> {
        use aws_sdk_s3::config::Credentials;
        use aws_sdk_s3::config::Region;

        info!("Uploading template bots to MinIO and creating bot entries...");

        // First, create bot entries in database for each template
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| config.database_url());
        let mut conn = diesel::PgConnection::establish(&database_url)?;
        self.create_bots_from_templates(&mut conn)?;

        let creds = Credentials::new(
            &config.minio.access_key,
            &config.minio.secret_key,
            None,
            None,
            "minio",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .credentials_provider(creds)
            .endpoint_url(&config.minio.server)
            .region(Region::new("us-east-1"))
            .force_path_style(true)
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        // Upload templates from templates/ directory
        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            trace!("Templates directory not found, skipping upload");
            return Ok(());
        }

        // Walk through each .gbai folder in templates/
        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.extension().map(|e| e == "gbai").unwrap_or(false) {
                let bot_name = path.file_name().unwrap().to_string_lossy().to_string();
                let bucket_name = format!("{}{}", config.minio.org_prefix, bot_name);

                trace!("Creating bucket: {}", bucket_name);

                // Create bucket if it doesn't exist
                match client.create_bucket().bucket(&bucket_name).send().await {
                    Ok(_) => info!("Created bucket: {}", bucket_name),
                    Err(e) => {
                        let err_str = e.to_string();
                        if err_str.contains("BucketAlreadyOwnedByYou")
                            || err_str.contains("BucketAlreadyExists")
                        {
                            trace!("Bucket {} already exists", bucket_name);
                        } else {
                            log::warn!("Failed to create bucket {}: {}", bucket_name, e);
                        }
                    }
                }

                // Upload all files recursively
                self.upload_directory_recursive(&client, &path, &bucket_name, "")
                    .await?;
                info!("Uploaded template bot: {}", bot_name);
            }
        }

        info!("Template bots uploaded successfully");
        Ok(())
    }

    fn create_bots_from_templates(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use crate::shared::models::schema::bots;
        use diesel::prelude::*;

        info!("Creating bot entries from template folders...");

        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            trace!("Templates directory not found, skipping bot creation");
            return Ok(());
        }

        // Walk through each .gbai folder in templates/
        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.extension().map(|e| e == "gbai").unwrap_or(false) {
                let bot_folder = path.file_name().unwrap().to_string_lossy().to_string();
                // Remove .gbai extension to get bot name
                let bot_name = bot_folder.trim_end_matches(".gbai");

                // Format the name nicely (capitalize first letter of each word)
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

                // Check if bot already exists
                let existing: Option<String> = bots::table
                    .filter(bots::name.eq(&formatted_name))
                    .select(bots::name)
                    .first(conn)
                    .optional()?;

                if existing.is_none() {
                    // Insert new bot
                    diesel::sql_query(
                        "INSERT INTO bots (id, name, description, llm_provider, llm_config, context_provider, context_config, is_active) \
                         VALUES (gen_random_uuid(), $1, $2, 'openai', '{\"model\": \"gpt-4\", \"temperature\": 0.7}', 'database', '{}', true)"
                    )
                    .bind::<diesel::sql_types::Text, _>(&formatted_name)
                    .bind::<diesel::sql_types::Text, _>(format!("Bot for {} template", bot_name))
                    .execute(conn)?;

                    info!("Created bot entry: {}", formatted_name);
                } else {
                    trace!("Bot already exists: {}", formatted_name);
                }
            }
        }

        info!("Bot creation from templates completed");
        Ok(())
    }

    fn upload_directory_recursive<'a>(
        &'a self,
        client: &'a aws_sdk_s3::Client,
        local_path: &'a Path,
        bucket: &'a str,
        prefix: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            use aws_sdk_s3::primitives::ByteStream;

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
                    trace!(
                        "Uploading file: {} to bucket: {} with key: {}",
                        path.display(),
                        bucket,
                        key
                    );

                    let body = ByteStream::from_path(&path).await?;

                    client
                        .put_object()
                        .bucket(bucket)
                        .key(&key)
                        .body(body)
                        .send()
                        .await?;

                    trace!("Uploaded: {}", key);
                } else if path.is_dir() {
                    self.upload_directory_recursive(client, &path, bucket, &key)
                        .await?;
                }
            }

            Ok(())
        })
    }

    fn apply_migrations(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        info!("Applying database migrations...");

        let migrations_dir = std::path::Path::new("migrations");
        if !migrations_dir.exists() {
            trace!("No migrations directory found, skipping");
            return Ok(());
        }

        // Get all .sql files sorted
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

            trace!("Reading migration: {}", filename);
            match std::fs::read_to_string(&path) {
                Ok(sql) => {
                    trace!("Applying migration: {}", filename);
                    match conn.batch_execute(&sql) {
                        Ok(_) => info!("Applied migration: {}", filename),
                        Err(e) => {
                            // Ignore errors for already applied migrations
                            trace!("Migration {} result: {}", filename, e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read migration {}: {}", filename, e);
                }
            }
        }

        info!("Migrations check completed");
        Ok(())
    }
}
