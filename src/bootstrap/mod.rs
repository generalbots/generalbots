use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use anyhow::Result;
use diesel::connection::SimpleConnection;
use diesel::Connection;
use dotenvy::dotenv;
use log::{info, trace};
use rand::distr::Alphanumeric;
use sha2::{Digest, Sha256};

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
}
