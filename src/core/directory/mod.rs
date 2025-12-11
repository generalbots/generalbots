pub mod api;
pub mod provisioning;

use anyhow::Result;
use aws_sdk_s3::Client as S3Client;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use provisioning::{BotAccess, UserAccount, UserProvisioningService, UserRole};

/// Directory service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub url: String,
    pub admin_token: String,
    pub project_id: String,
    pub oauth_enabled: bool,
}

impl Default for DirectoryConfig {
    fn default() -> Self {
        Self {
            url: "https://localhost:8300".to_string(),
            admin_token: String::new(),
            project_id: "default".to_string(),
            oauth_enabled: true,
        }
    }
}

/// Main directory service interface
pub struct DirectoryService {
    config: DirectoryConfig,
    provisioning: Arc<UserProvisioningService>,
}

impl std::fmt::Debug for DirectoryService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectoryService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl DirectoryService {
    pub fn new(
        config: DirectoryConfig,
        db_pool: Pool<ConnectionManager<PgConnection>>,
        s3_client: Arc<S3Client>,
    ) -> Result<Self> {
        let provisioning = Arc::new(UserProvisioningService::new(
            db_pool,
            Some(s3_client),
            config.url.clone(),
        ));

        Ok(Self {
            config,
            provisioning,
        })
    }

    pub async fn create_user(&self, account: UserAccount) -> Result<()> {
        self.provisioning.provision_user(&account).await
    }

    pub async fn delete_user(&self, username: &str) -> Result<()> {
        self.provisioning.deprovision_user(username).await
    }

    pub fn get_provisioning_service(&self) -> Arc<UserProvisioningService> {
        Arc::clone(&self.provisioning)
    }

    /// Get the directory service URL
    pub fn get_url(&self) -> &str {
        &self.config.url
    }

    /// Check if OAuth is enabled
    pub fn is_oauth_enabled(&self) -> bool {
        self.config.oauth_enabled
    }

    /// Get the project ID
    pub fn get_project_id(&self) -> &str {
        &self.config.project_id
    }

    /// Get the full configuration (for admin purposes)
    pub fn get_config(&self) -> &DirectoryConfig {
        &self.config
    }
}
