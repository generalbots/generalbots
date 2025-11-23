use crate::config::AppConfig;
use crate::package_manager::setup::{DirectorySetup, EmailSetup};
use crate::package_manager::{InstallMode, PackageManager};
use crate::shared::utils::establish_pg_connection;
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use dotenvy::dotenv;
use log::{error, info, trace};
use rand::distr::Alphanumeric;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
#[derive(Debug)]
pub struct ComponentInfo {
    pub name: &'static str,
}
#[derive(Debug)]
pub struct BootstrapManager {
    pub install_mode: InstallMode,
    pub tenant: Option<String>,
}
impl BootstrapManager {
    pub async fn new(install_mode: InstallMode, tenant: Option<String>) -> Self {
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
            ComponentInfo { name: "tables" },
            ComponentInfo { name: "cache" },
            ComponentInfo { name: "drive" },
            ComponentInfo { name: "llm" },
            ComponentInfo { name: "email" },
            ComponentInfo { name: "proxy" },
            ComponentInfo { name: "directory" },
            ComponentInfo { name: "alm" },
            ComponentInfo { name: "alm_ci" },
            ComponentInfo { name: "dns" },
            ComponentInfo { name: "webmail" },
            ComponentInfo { name: "meeting" },
            ComponentInfo {
                name: "table_editor",
            },
            ComponentInfo { name: "doc_editor" },
            ComponentInfo { name: "desktop" },
            ComponentInfo { name: "devtools" },
            ComponentInfo { name: "bot" },
            ComponentInfo { name: "system" },
            ComponentInfo { name: "vector_db" },
            ComponentInfo { name: "host" },
        ];
        for component in components {
            if pm.is_installed(component.name) {
                pm.start(component.name)?;
            }
        }
        Ok(())
    }

    fn generate_secure_password(&self, length: usize) -> String {
        let mut rng = rand::rng();
        (0..length)
            .map(|_| {
                let byte = rand::Rng::sample(&mut rng, Alphanumeric);
                char::from(byte)
            })
            .collect()
    }

    pub async fn bootstrap(&mut self) -> Result<()> {
        let env_path = std::env::current_dir().unwrap().join(".env");
        let db_password = self.generate_secure_password(32);
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            format!("postgres://gbuser:{}@localhost:5432/botserver", db_password)
        });

        let drive_password = self.generate_secure_password(16);
        let drive_user = "gbdriveuser".to_string();
        let drive_env = format!(
            "\nDRIVE_SERVER=http://localhost:9000\nDRIVE_ACCESSKEY={}\nDRIVE_SECRET={}\n",
            drive_user, drive_password
        );
        let contents_env = format!("DATABASE_URL={}\n{}", database_url, drive_env);
        let _ = std::fs::write(&env_path, contents_env);
        dotenv().ok();

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone()).unwrap();
        let required_components = vec!["tables", "drive", "cache", "llm"];
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
                _ = pm.install(component).await;
                if component == "tables" {
                    let mut conn = establish_pg_connection().unwrap();
                    self.apply_migrations(&mut conn)?;
                }

                // Auto-configure Directory after installation
                if component == "directory" {
                    info!("🔧 Auto-configuring Directory (Zitadel)...");
                    if let Err(e) = self.setup_directory().await {
                        error!("Failed to setup Directory: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Setup Directory (Zitadel) with default organization and user
    async fn setup_directory(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/directory_config.json");

        // Ensure config directory exists
        tokio::fs::create_dir_all("./config").await?;

        let mut setup = DirectorySetup::new("http://localhost:8080".to_string(), config_path);

        // Create default organization
        let org_name = "default";
        let org_id = setup
            .create_organization(org_name, "Default Organization")
            .await?;
        info!("✅ Created default organization: {}", org_name);

        // Create admin@default account for bot administration
        let admin_user = setup
            .create_user(
                &org_id,
                "admin",
                "admin@default",
                "Admin123!",
                "Admin",
                "Default",
                true, // is_admin
            )
            .await?;
        info!("✅ Created admin user: admin@default");

        // Create user@default account for regular bot usage
        let regular_user = setup
            .create_user(
                &org_id,
                "user",
                "user@default",
                "User123!",
                "User",
                "Default",
                false, // is_admin
            )
            .await?;
        info!("✅ Created regular user: user@default");
        info!("   Regular user ID: {}", regular_user.id);

        // Create OAuth2 application for BotServer
        let (project_id, client_id, client_secret) =
            setup.create_oauth_application(&org_id).await?;
        info!("✅ Created OAuth2 application in project: {}", project_id);

        // Save configuration
        let config = setup
            .save_config(
                org_id.clone(),
                org_name.to_string(),
                admin_user,
                client_id.clone(),
                client_secret,
            )
            .await?;

        info!("✅ Directory initialized successfully!");
        info!("   Organization: default");
        info!("   Admin User: admin@default / Admin123!");
        info!("   Regular User: user@default / User123!");
        info!("   Client ID: {}", client_id);
        info!("   Login URL: {}", config.base_url);

        Ok(())
    }

    /// Setup Email (Stalwart) with Directory integration
    async fn setup_email(&self) -> Result<()> {
        let config_path = PathBuf::from("./config/email_config.json");
        let directory_config_path = PathBuf::from("./config/directory_config.json");

        let mut setup = EmailSetup::new("http://localhost:8080".to_string(), config_path);

        // Try to integrate with Directory if it exists
        let directory_config = if directory_config_path.exists() {
            Some(directory_config_path)
        } else {
            None
        };

        let config = setup.initialize(directory_config).await?;

        info!("✅ Email server initialized successfully!");
        info!("   SMTP: {}:{}", config.smtp_host, config.smtp_port);
        info!("   IMAP: {}:{}", config.imap_host, config.imap_port);
        info!("   Admin: {} / {}", config.admin_user, config.admin_pass);
        if config.directory_integration {
            info!("   🔗 Integrated with Directory for authentication");
        }

        Ok(())
    }

    async fn get_drive_client(config: &AppConfig) -> Client {
        let endpoint = if !config.drive.server.ends_with('/') {
            format!("{}/", config.drive.server)
        } else {
            config.drive.server.clone()
        };
        let base_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region("auto")
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                config.drive.access_key.clone(),
                config.drive.secret_key.clone(),
                None,
                None,
                "static",
            ))
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&base_config)
            .force_path_style(true)
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    }

    pub async fn upload_templates_to_drive(&self, _config: &AppConfig) -> Result<()> {
        let mut conn = establish_pg_connection()?;
        self.create_bots_from_templates(&mut conn)?;
        let templates_dir = Path::new("templates");
        if !templates_dir.exists() {
            return Ok(());
        }
        let client = Self::get_drive_client(_config).await;
        let mut read_dir = tokio::fs::read_dir(templates_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
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
                if client.head_bucket().bucket(&bucket).send().await.is_err() {
                    match client.create_bucket().bucket(&bucket).send().await {
                        Ok(_) => {
                            self.upload_directory_recursive(&client, &path, &bucket, "/")
                                .await?;
                        }
                        Err(e) => {
                            error!("Failed to create bucket {}: {:?}", bucket, e);
                            return Err(anyhow::anyhow!("Failed to create bucket {}: {}. Check S3 credentials and endpoint configuration", bucket, e));
                        }
                    }
                } else {
                    trace!("Bucket {} already exists", bucket);
                }
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
                    diesel::sql_query("INSERT INTO bots (id, name, description, llm_provider, llm_config, context_provider, context_config, is_active) VALUES (gen_random_uuid(), $1, $2, 'openai', '{\"model\": \"gpt-4\", \"temperature\": 0.7}', 'database', '{}', true)").bind::<diesel::sql_types::Text, _>(&bot_name).bind::<diesel::sql_types::Text, _>(format!("Bot for {} template", bot_name)).execute(conn)?;
                } else {
                    trace!("Bot {} already exists", bot_name);
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
            let _normalized_path = if !local_path.to_string_lossy().ends_with('/') {
                format!("{}/", local_path.to_string_lossy())
            } else {
                local_path.to_string_lossy().to_string()
            };
            let mut read_dir = tokio::fs::read_dir(local_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let mut key = prefix.trim_matches('/').to_string();
                if !key.is_empty() {
                    key.push('/');
                }
                key.push_str(&file_name);
                if path.is_file() {
                    trace!(
                        "Uploading file {} to bucket {} with key {}",
                        path.display(),
                        bucket,
                        key
                    );
                    let content = tokio::fs::read(&path).await?;
                    client
                        .put_object()
                        .bucket(bucket)
                        .key(&key)
                        .body(content.into())
                        .send()
                        .await?;
                } else if path.is_dir() {
                    self.upload_directory_recursive(client, &path, bucket, &key)
                        .await?;
                }
            }
            Ok(())
        })
    }
    pub fn apply_migrations(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use diesel_migrations::HarnessWithOutput;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

        let mut harness = HarnessWithOutput::write_to_stdout(conn);
        if let Err(e) = harness.run_pending_migrations(MIGRATIONS) {
            error!("Failed to apply migrations: {}", e);
            return Err(anyhow::anyhow!("Migration error: {}", e));
        }

        Ok(())
    }
}
