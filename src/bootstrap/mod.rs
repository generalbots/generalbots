use crate::config::AppConfig;
use crate::package_manager::{InstallMode, PackageManager};
use anyhow::Result;
use log::{debug, info, trace, warn};

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

    pub fn bootstrap(&mut self) -> Result<AppConfig> {
        info!("Starting bootstrap process");

        let pm = PackageManager::new(self.install_mode.clone(), self.tenant.clone())?;

        let required_components = vec!["drive", "cache", "tables", "llm"];

        for component in required_components {
            if !pm.is_installed(component) {
                info!("Installing required component: {}", component);
                futures::executor::block_on(pm.install(component))?;
                trace!("Successfully installed component: {}", component);
            } else {
                debug!("Component {} already installed", component);
            }
        }

        info!("Bootstrap completed successfully");

        let config = match diesel::Connection::establish(
            "postgres://botserver:botserver@localhost:5432/botserver",
        ) {
            Ok(mut conn) => {
                trace!("Connected to database for config loading");
                AppConfig::from_database(&mut conn)
            }
            Err(e) => {
                warn!("Failed to connect to database for config: {}", e);
                trace!("Falling back to environment configuration");
                AppConfig::from_env()
            }
        };

        Ok(config)
    }
}
