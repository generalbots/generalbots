use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use anyhow::Result;
use diesel::{Connection, RunQueryDsl};
use log::trace;
use rand::distr::Alphanumeric;
use rand::Rng;
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
        let required_components = vec!["tables"];
        let mut config = AppConfig::from_env();

        for component in required_components {
            if !pm.is_installed(component) {
                trace!("Installing required component: {}", component);
                futures::executor::block_on(pm.install(component))?;
                trace!("Starting component after install: {}", component);
                pm.start(component)?;

                if component == "tables" {
                    let db_password = self.generate_secure_password(16);
                    let farm_password = self.generate_secure_password(32);
                    let encrypted_db_password = self.encrypt_password(&db_password, &farm_password);

                    let env_contents = format!(
                        "FARM_PASSWORD={}\nDATABASE_URL=postgres://gbuser:{}@localhost:5432/botserver",
                        farm_password, db_password
                    );

                    std::fs::write(".env", env_contents)
                        .map_err(|e| anyhow::anyhow!("Failed to write .env file: {}", e))?;

                    trace!("Waiting 5 seconds for database to start...");
                    std::thread::sleep(std::time::Duration::from_secs(5));

                    let migration_dir = include_dir::include_dir!("./migrations");
                    let mut migration_files: Vec<_> = migration_dir
                        .files()
                        .filter_map(|file| {
                            let path = file.path();
                            if path.extension()? == "sql" {
                                Some(path.to_path_buf())
                            } else {
                                None
                            }
                        })
                        .collect();

                    migration_files.sort();
                    let mut conn = diesel::PgConnection::establish(&format!(
                        "postgres://gbuser:{}@localhost:5432/botserver",
                        db_password
                    ))?;

                    for migration_file in migration_files {
                        let migration = std::fs::read_to_string(&migration_file)?;
                        trace!("Executing migration: {}", migration_file.display());
                        diesel::sql_query(&migration).execute(&mut conn)?;
                    }

                    self.setup_secure_credentials(&mut conn, &encrypted_db_password)?;
                    config = AppConfig::from_database(&mut conn);
                }
            } else {
                trace!("Required component {} already installed", component);
            }
        }

        Ok(config)
    }

    fn setup_secure_credentials(
        &self,
        conn: &mut diesel::PgConnection,
        encrypted_db_password: &str,
    ) -> Result<()> {
        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;
        use uuid::Uuid;

        let system_bot_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")?;
        diesel::update(bots)
            .filter(bot_id.eq(system_bot_id))
            .set(config.eq(serde_json::json!({
                "encrypted_db_password": encrypted_db_password,
            })))
            .execute(conn)?;

        Ok(())
    }

    fn generate_secure_password(&self, length: usize) -> String {
        let rng = rand::rng();
        rng.sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }

    fn encrypt_password(&self, password: &str, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
