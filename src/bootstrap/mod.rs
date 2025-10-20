use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use anyhow::Result;
use log::{trace, warn};
use rand::distr::Alphanumeric;
use rand::rngs::ThreadRng;
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
        let required_components = vec!["tables"]; // , "cache", "drive", "llm"];

        for component in required_components {
            if !pm.is_installed(component) {
                trace!("Installing required component: {}", component);
                futures::executor::block_on(pm.install(component))?;
                trace!("Starting component after install: {}", component);
                pm.start(component)?;
            } else {
                trace!("Required component {} already installed", component);
            }
        }

        let config = match diesel::Connection::establish(
            "postgres://botserver:botserver@localhost:5432/botserver",
        ) {
            Ok(mut conn) => {
                self.setup_secure_credentials(&mut conn)?;
                AppConfig::from_database(&mut conn)
            }
            Err(e) => {
                warn!("Failed to connect to database for config: {}", e);
                AppConfig::from_env()
            }
        };

        Ok(config)
    }

    fn setup_secure_credentials(&self, conn: &mut diesel::PgConnection) -> Result<()> {
        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;
        use uuid::Uuid;

        let farm_password =
            std::env::var("FARM_PASSWORD").unwrap_or_else(|_| self.generate_secure_password(32));
        let db_password = self.generate_secure_password(16);

        let encrypted_db_password = self.encrypt_password(&db_password, &farm_password);

        let env_contents = format!(
            "FARM_PASSWORD={}\nDATABASE_URL=postgres://gbuser:{}@localhost:5432/botserver",
            farm_password, db_password
        );

        std::fs::write(".env", env_contents)
            .map_err(|e| anyhow::anyhow!("Failed to write .env file: {}", e))?;

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
        let rng: ThreadRng = rand::rng();
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
