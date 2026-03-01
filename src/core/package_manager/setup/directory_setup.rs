use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

#[derive(Debug)]
pub struct DirectorySetup {
    base_url: String,
    client: Client,
    admin_token: Option<String>,
    /// Admin credentials for password grant authentication (used during initial setup)
    admin_credentials: Option<(String, String)>,
    config_path: PathBuf,
}

impl DirectorySetup {
    pub fn set_admin_token(&mut self, token: String) {
        self.admin_token = Some(token);
    }

    /// Set admin credentials for password grant authentication
    pub fn set_admin_credentials(&mut self, username: String, password: String) {
        self.admin_credentials = Some((username, password));
    }

    /// Get an access token using either PAT or password grant
    async fn get_admin_access_token(&self) -> Result<String> {
        // If we have a PAT token, use it directly
        if let Some(ref token) = self.admin_token {
            return Ok(token.clone());
        }

        // If we have admin credentials, use password grant
        if let Some((username, password)) = &self.admin_credentials {
            let token_url = format!("{}/oauth/v2/token", self.base_url);
            let params = [
                ("grant_type", "password".to_string()),
                ("username", username.clone()),
                ("password", password.clone()),
                ("scope", "openid profile email urn:zitadel:iam:org:project:id:zitadel:aud".to_string()),
            ];

            let response = self
                .client
                .post(&token_url)
                .form(&params)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get access token: {}", e))?;

            let token_data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to parse token response: {}", e))?;

            let access_token = token_data
                .get("access_token")
                .and_then(|t| t.as_str())
                .ok_or_else(|| anyhow::anyhow!("No access token in response"))?
                .to_string();

            log::info!("Obtained access token via password grant");
            return Ok(access_token);
        }

        Err(anyhow::anyhow!("No admin token or credentials configured"))
    }

    pub async fn ensure_admin_token(&mut self) -> Result<()> {
        if self.admin_token.is_none() && self.admin_credentials.is_none() {
            return Err(anyhow::anyhow!("Admin token or credentials must be configured"));
        }

        // If we have credentials but no token, authenticate and get the token
        if self.admin_token.is_none() && self.admin_credentials.is_some() {
            let token = self.get_admin_access_token().await?;
            self.admin_token = Some(token);
            log::info!("Obtained admin access token from credentials");
        }

        Ok(())
    }

    fn generate_secure_password() -> String {
        use rand::distr::Alphanumeric;
        use rand::Rng;
        let mut rng = rand::rng();
        (0..16)
            .map(|_| {
                let byte = rng.sample(Alphanumeric);
                char::from(byte)
            })
            .collect()
    }
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

pub struct CreateUserParams<'a> {
    pub org_id: &'a str,
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub is_admin: bool,
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
                .unwrap_or_else(|e| {
                    log::warn!("Failed to create HTTP client with timeout: {}, using default", e);
                    Client::new()
                }),
            admin_token: None,
            admin_credentials: None,
            config_path,
        }
    }

    /// Create a DirectorySetup with initial admin credentials for password grant
    pub fn with_admin_credentials(base_url: String, config_path: PathBuf, username: String, password: String) -> Self {
        Self {
            base_url,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|e| {
                    log::warn!("Failed to create HTTP client with timeout: {}, using default", e);
                    Client::new()
                }),
            admin_token: None,
            admin_credentials: Some((username, password)),
            config_path,
        }
    }

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

    pub async fn initialize(&mut self) -> Result<DirectoryConfig> {
        log::info!(" Initializing Directory (Zitadel) with defaults...");

        if let Ok(existing_config) = self.load_existing_config().await {
            log::info!("Directory already initialized, using existing config");
            return Ok(existing_config);
        }

        self.wait_for_ready(30).await?;

        // Wait additional time for Zitadel API to be fully ready
        log::info!("Waiting for Zitadel API to be fully initialized...");
        sleep(Duration::from_secs(10)).await;

        self.ensure_admin_token().await?;

        let org = self.create_default_organization().await?;
        log::info!(" Created default organization: {}", org.name);

        let user = self.create_default_user(&org.id).await?;
        log::info!(" Created default user: {}", user.username);

        // Retry OAuth client creation up to 3 times with delays
        let (project_id, client_id, client_secret) = {
            let mut last_error = None;
            let mut result = None;

            for attempt in 1..=3 {
                match self.create_oauth_application(&org.id).await {
                    Ok(credentials) => {
                        result = Some(credentials);
                        break;
                    }
                    Err(e) => {
                        log::warn!(
                            "OAuth client creation attempt {}/3 failed: {}",
                            attempt,
                            e
                        );
                        last_error = Some(e);
                        if attempt < 3 {
                            log::info!("Retrying in 5 seconds...");
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }

            result.ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to create OAuth client after 3 attempts: {}",
                    last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error"))
                )
            })?
        };
        log::info!(" Created OAuth2 application");

        self.grant_user_permissions(&org.id, &user.id).await?;
        log::info!(" Granted admin permissions to default user");

        let config = DirectoryConfig {
            base_url: self.base_url.clone(),
            default_org: org,
            default_user: user,
            admin_token: self.admin_token.clone().unwrap_or_default(),
            project_id,
            client_id,
            client_secret,
        };

        self.save_config_internal(&config).await?;
        log::info!(" Saved Directory configuration");

        log::info!(" Directory initialization complete!");
        log::info!("");
        log::info!("╔══════════════════════════════════════════════════════════════╗");
        log::info!("║                    DEFAULT CREDENTIALS                       ║");
        log::info!("╠══════════════════════════════════════════════════════════════╣");
        log::info!("║  Email:    {:<50}║", config.default_user.email);
        log::info!("║  Password: {:<50}║", config.default_user.password);
        log::info!("╠══════════════════════════════════════════════════════════════╣");
        log::info!("║  Login at: {:<50}║", self.base_url);
        log::info!("╚══════════════════════════════════════════════════════════════╝");
        log::info!("");
        log::info!(">>> COPY THESE CREDENTIALS NOW - Press ENTER to continue <<<");

        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);

        Ok(config)
    }

    pub async fn create_organization(&mut self, name: &str, description: &str) -> Result<String> {
        self.ensure_admin_token().await?;

        let response = self
            .client
            .post(format!("{}/management/v1/orgs", self.base_url))
            .bearer_auth(self.admin_token.as_ref().unwrap_or(&String::new()))
            .json(&json!({
                "name": name,
                "description": description,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to create organization: {}", error_text);
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result["id"].as_str().unwrap_or("").to_string())
    }

    async fn create_default_organization(&self) -> Result<DefaultOrganization> {
        let org_name = "BotServer".to_string();

        let response = self
            .client
            .post(format!("{}/management/v1/orgs", self.base_url))
            .bearer_auth(self.admin_token.as_ref().unwrap_or(&String::new()))
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

    pub async fn create_user(
        &mut self,
        params: CreateUserParams<'_>,
    ) -> Result<DefaultUser> {
        self.ensure_admin_token().await?;

        let response = self
            .client
            .post(format!("{}/management/v1/users/human", self.base_url))
            .bearer_auth(self.admin_token.as_ref().unwrap_or(&String::new()))
            .json(&json!({
                "userName": params.username,
                "profile": {
                    "firstName": params.first_name,
                    "lastName": params.last_name,
                    "displayName": format!("{} {}", params.first_name, params.last_name)
                },
                "email": {
                    "email": params.email,
                    "isEmailVerified": true
                },
                "password": params.password,
                "organisation": {
                    "orgId": params.org_id
                }
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to create user: {}", error_text);
        }

        let result: serde_json::Value = response.json().await?;

        let user = DefaultUser {
            id: result["userId"].as_str().unwrap_or("").to_string(),
            username: params.username.to_string(),
            email: params.email.to_string(),
            password: params.password.to_string(),
            first_name: params.first_name.to_string(),
            last_name: params.last_name.to_string(),
        };

        if params.is_admin {
            self.grant_user_permissions(params.org_id, &user.id).await?;
        }

        Ok(user)
    }

    async fn create_default_user(&self, org_id: &str) -> Result<DefaultUser> {
        let username = format!(
            "admin_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );
        let email = format!("{}@botserver.local", username);
        let password = Self::generate_secure_password();

        let response = self
            .client
            .post(format!("{}/management/v1/users/human", self.base_url))
            .bearer_auth(self.admin_token.as_ref().unwrap_or(&String::new()))
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
                "organisation": {
                    "orgId": org_id
                }
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

    pub async fn create_oauth_application(
        &self,
        _org_id: &str,
    ) -> Result<(String, String, String)> {
        let app_name = "BotServer";
        let redirect_uri = "http://localhost:8080/auth/callback".to_string();

        // Get access token using either PAT or password grant
        let access_token = self.get_admin_access_token().await
            .map_err(|e| anyhow::anyhow!("Failed to get admin access token: {}", e))?;

        let project_response = self
            .client
            .post(format!("{}/management/v1/projects", self.base_url))
            .bearer_auth(&access_token)
            .json(&json!({
                "name": app_name,
            }))
            .send()
            .await?;

        if !project_response.status().is_success() {
            let error_text = project_response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to create project: {}", error_text));
        }

        let project_result: serde_json::Value = project_response.json().await?;
        let project_id = project_result["id"].as_str().unwrap_or("").to_string();

        if project_id.is_empty() {
            return Err(anyhow::anyhow!("Project ID is empty in response"));
        }

        let app_response = self.client
            .post(format!("{}/management/v1/projects/{}/apps/oidc", self.base_url, project_id))
            .bearer_auth(&access_token)
            .json(&json!({
                "name": app_name,
                "redirectUris": [redirect_uri, "http://localhost:3000/auth/callback", "http://localhost:8080/auth/callback", "http://localhost:9000/auth/callback"],
                "responseTypes": ["OIDC_RESPONSE_TYPE_CODE"],
                "grantTypes": ["OIDC_GRANT_TYPE_AUTHORIZATION_CODE", "OIDC_GRANT_TYPE_REFRESH_TOKEN", "OIDC_GRANT_TYPE_PASSWORD"],
                "appType": "OIDC_APP_TYPE_WEB",
                "authMethodType": "OIDC_AUTH_METHOD_TYPE_POST",
                "postLogoutRedirectUris": ["http://localhost:8080", "http://localhost:3000", "http://localhost:9000"],
                "accessTokenType": "OIDC_TOKEN_TYPE_BEARER",
                "devMode": true,
            }))
            .send()
            .await?;

        if !app_response.status().is_success() {
            let error_text = app_response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to create OAuth application: {}", error_text));
        }

        let app_result: serde_json::Value = app_response.json().await?;
        let client_id = app_result["clientId"].as_str().unwrap_or("").to_string();
        let client_secret = app_result["clientSecret"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if client_id.is_empty() {
            return Err(anyhow::anyhow!("Client ID is empty in response"));
        }

        log::info!("Created OAuth application with client_id: {}", client_id);
        Ok((project_id, client_id, client_secret))
    }

    pub async fn grant_user_permissions(&self, org_id: &str, user_id: &str) -> Result<()> {
        let _response = self
            .client
            .post(format!(
                "{}/management/v1/orgs/{}/members",
                self.base_url, org_id
            ))
            .bearer_auth(self.admin_token.as_ref().unwrap_or(&String::new()))
            .json(&json!({
                "userId": user_id,
                "roles": ["ORG_OWNER"]
            }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn save_config(
        &mut self,
        org_id: String,
        org_name: String,
        admin_user: DefaultUser,
        client_id: String,
        client_secret: String,
    ) -> Result<DirectoryConfig> {
        self.ensure_admin_token().await?;

        let config = DirectoryConfig {
            base_url: self.base_url.clone(),
            default_org: DefaultOrganization {
                id: org_id,
                name: org_name.clone(),
                domain: format!("{}.localhost", org_name.to_lowercase()),
            },
            default_user: admin_user,
            admin_token: self.admin_token.clone().unwrap_or_default(),
            project_id: String::new(),
            client_id,
            client_secret,
        };

        let json = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, json).await?;

        log::info!(
            "Saved Directory configuration to {}",
            self.config_path.display()
        );
        Ok(config)
    }

    async fn save_config_internal(&self, config: &DirectoryConfig) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    anyhow::anyhow!("Failed to create config directory {}: {}", parent.display(), e)
                })?;
                log::info!("Created config directory: {}", parent.display());
            }
        }

        let json = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, json).await.map_err(|e| {
            anyhow::anyhow!("Failed to write config to {}: {}", self.config_path.display(), e)
        })?;
        log::info!("Saved Directory configuration to {}", self.config_path.display());
        Ok(())
    }

    async fn load_existing_config(&self) -> Result<DirectoryConfig> {
        let content = fs::read_to_string(&self.config_path).await?;
        let config: DirectoryConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub async fn get_config(&self) -> Result<DirectoryConfig> {
        self.load_existing_config().await
    }
}

pub async fn generate_directory_config(config_path: PathBuf, _db_path: PathBuf) -> Result<()> {
    let yaml_config = r"
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

Port: 9000
ExternalDomain: localhost
ExternalPort: 9000
ExternalSecure: false

TLS:
  Enabled: false
"
    .to_string();

    fs::write(config_path, yaml_config).await?;
    Ok(())
}
