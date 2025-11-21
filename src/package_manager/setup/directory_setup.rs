use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

/// Directory (Zitadel) auto-setup manager
pub struct DirectorySetup {
    base_url: String,
    client: Client,
    admin_token: Option<String>,
    config_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultOrganization {
    pub id: String,
    pub name: String,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub base_url: String,
    pub default_org: DefaultOrganization,
    pub default_user: DefaultUser,
    pub admin_token: String,
    pub project_id: String,
    pub client_id: String,
    pub client_secret: String,
}

impl DirectorySetup {
    pub fn new(base_url: String, config_path: PathBuf) -> Self {
        Self {
            base_url,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            admin_token: None,
            config_path,
        }
    }

    /// Wait for directory service to be ready
    pub async fn wait_for_ready(&self, max_attempts: u32) -> Result<()> {
        log::info!("Waiting for Directory service to be ready...");

        for attempt in 1..=max_attempts {
            match self
                .client
                .get(format!("{}/debug/ready", self.base_url))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    log::info!("Directory service is ready!");
                    return Ok(());
                }
                _ => {
                    log::debug!(
                        "Directory not ready yet (attempt {}/{})",
                        attempt,
                        max_attempts
                    );
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }

        anyhow::bail!("Directory service did not become ready in time")
    }

    /// Initialize directory with default configuration
    pub async fn initialize(&mut self) -> Result<DirectoryConfig> {
        log::info!("🔧 Initializing Directory (Zitadel) with defaults...");

        // Check if already initialized
        if let Ok(existing_config) = self.load_existing_config().await {
            log::info!("Directory already initialized, using existing config");
            return Ok(existing_config);
        }

        // Wait for service to be ready
        self.wait_for_ready(30).await?;

        // Get initial admin token (from Zitadel setup)
        self.get_initial_admin_token().await?;

        // Create default organization
        let org = self.create_default_organization().await?;
        log::info!("✅ Created default organization: {}", org.name);

        // Create default user
        let user = self.create_default_user(&org.id).await?;
        log::info!("✅ Created default user: {}", user.username);

        // Create OAuth2 application for BotServer
        let (project_id, client_id, client_secret) = self.create_oauth_application(&org.id).await?;
        log::info!("✅ Created OAuth2 application");

        // Grant user admin permissions
        self.grant_user_permissions(&org.id, &user.id).await?;
        log::info!("✅ Granted admin permissions to default user");

        let config = DirectoryConfig {
            base_url: self.base_url.clone(),
            default_org: org,
            default_user: user,
            admin_token: self.admin_token.clone().unwrap_or_default(),
            project_id,
            client_id,
            client_secret,
        };

        // Save configuration
        self.save_config(&config).await?;
        log::info!("✅ Saved Directory configuration");

        log::info!("🎉 Directory initialization complete!");
        log::info!(
            "📧 Default user: {} / {}",
            config.default_user.email,
            config.default_user.password
        );
        log::info!("🌐 Login at: {}", self.base_url);

        Ok(config)
    }

    /// Get initial admin token from Zitadel
    async fn get_initial_admin_token(&mut self) -> Result<()> {
        // In Zitadel, the initial setup creates a service account
        // For now, use environment variable or default token
        let token = std::env::var("DIRECTORY_ADMIN_TOKEN")
            .unwrap_or_else(|_| "zitadel-admin-sa".to_string());

        self.admin_token = Some(token);
        Ok(())
    }

    /// Create default organization
    async fn create_default_organization(&self) -> Result<DefaultOrganization> {
        let org_name =
            std::env::var("DIRECTORY_DEFAULT_ORG").unwrap_or_else(|_| "BotServer".to_string());

        let response = self
            .client
            .post(format!("{}/management/v1/orgs", self.base_url))
            .bearer_auth(self.admin_token.as_ref().unwrap())
            .json(&json!({
                "name": org_name,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to create organization: {}", error_text);
        }

        let result: serde_json::Value = response.json().await?;

        Ok(DefaultOrganization {
            id: result["id"].as_str().unwrap_or("").to_string(),
            name: org_name.clone(),
            domain: format!("{}.localhost", org_name.to_lowercase()),
        })
    }

    /// Create default user in organization
    async fn create_default_user(&self, org_id: &str) -> Result<DefaultUser> {
        let username =
            std::env::var("DIRECTORY_DEFAULT_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let email = std::env::var("DIRECTORY_DEFAULT_EMAIL")
            .unwrap_or_else(|_| "admin@localhost".to_string());
        let password = std::env::var("DIRECTORY_DEFAULT_PASSWORD")
            .unwrap_or_else(|_| "BotServer123!".to_string());

        let response = self
            .client
            .post(format!("{}/management/v1/users/human", self.base_url))
            .bearer_auth(self.admin_token.as_ref().unwrap())
            .json(&json!({
                "userName": username,
                "profile": {
                    "firstName": "Admin",
                    "lastName": "User",
                    "displayName": "Administrator"
                },
                "email": {
                    "email": email,
                    "isEmailVerified": true
                },
                "password": password,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to create user: {}", error_text);
        }

        let result: serde_json::Value = response.json().await?;

        Ok(DefaultUser {
            id: result["userId"].as_str().unwrap_or("").to_string(),
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
            first_name: "Admin".to_string(),
            last_name: "User".to_string(),
        })
    }

    /// Create OAuth2 application for BotServer
    async fn create_oauth_application(&self, org_id: &str) -> Result<(String, String, String)> {
        let app_name = "BotServer";
        let redirect_uri = std::env::var("DIRECTORY_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8080/auth/callback".to_string());

        // Create project
        let project_response = self
            .client
            .post(format!("{}/management/v1/projects", self.base_url))
            .bearer_auth(self.admin_token.as_ref().unwrap())
            .json(&json!({
                "name": app_name,
            }))
            .send()
            .await?;

        let project_result: serde_json::Value = project_response.json().await?;
        let project_id = project_result["id"].as_str().unwrap_or("").to_string();

        // Create OIDC application
        let app_response = self.client
            .post(format!("{}/management/v1/projects/{}/apps/oidc", self.base_url, project_id))
            .bearer_auth(self.admin_token.as_ref().unwrap())
            .json(&json!({
                "name": app_name,
                "redirectUris": [redirect_uri],
                "responseTypes": ["OIDC_RESPONSE_TYPE_CODE"],
                "grantTypes": ["OIDC_GRANT_TYPE_AUTHORIZATION_CODE", "OIDC_GRANT_TYPE_REFRESH_TOKEN"],
                "appType": "OIDC_APP_TYPE_WEB",
                "authMethodType": "OIDC_AUTH_METHOD_TYPE_BASIC",
                "postLogoutRedirectUris": ["http://localhost:8080"],
            }))
            .send()
            .await?;

        let app_result: serde_json::Value = app_response.json().await?;
        let client_id = app_result["clientId"].as_str().unwrap_or("").to_string();
        let client_secret = app_result["clientSecret"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok((project_id, client_id, client_secret))
    }

    /// Grant admin permissions to user
    async fn grant_user_permissions(&self, org_id: &str, user_id: &str) -> Result<()> {
        // Grant ORG_OWNER role
        let _response = self
            .client
            .post(format!(
                "{}/management/v1/orgs/{}/members",
                self.base_url, org_id
            ))
            .bearer_auth(self.admin_token.as_ref().unwrap())
            .json(&json!({
                "userId": user_id,
                "roles": ["ORG_OWNER"]
            }))
            .send()
            .await?;

        Ok(())
    }

    /// Save configuration to file
    async fn save_config(&self, config: &DirectoryConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, json).await?;
        Ok(())
    }

    /// Load existing configuration
    async fn load_existing_config(&self) -> Result<DirectoryConfig> {
        let content = fs::read_to_string(&self.config_path).await?;
        let config: DirectoryConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Get stored configuration
    pub async fn get_config(&self) -> Result<DirectoryConfig> {
        self.load_existing_config().await
    }
}

/// Generate Zitadel configuration file
pub async fn generate_directory_config(config_path: PathBuf, db_path: PathBuf) -> Result<()> {
    let yaml_config = format!(
        r#"
Log:
  Level: info

Database:
  Postgres:
    Host: localhost
    Port: 5432
    Database: zitadel
    User: zitadel
    Password: zitadel
    SSL:
      Mode: disable

Machine:
  Identification:
    Hostname: localhost
    WebhookAddress: http://localhost:8080

ExternalDomain: localhost:8080
ExternalPort: 8080
ExternalSecure: false

TLS:
  Enabled: false
"#
    );

    fs::write(config_path, yaml_config).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_setup_creation() {
        let setup = DirectorySetup::new(
            "http://localhost:8080".to_string(),
            PathBuf::from("/tmp/directory_config.json"),
        );
        assert_eq!(setup.base_url, "http://localhost:8080");
    }
}
