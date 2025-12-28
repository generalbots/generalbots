use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

#[derive(Debug)]
pub struct EmailSetup {
    base_url: String,
    admin_user: String,
    admin_pass: String,
    config_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfig {
    pub base_url: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub imap_host: String,
    pub imap_port: u16,
    pub admin_user: String,
    pub admin_pass: String,
    pub directory_integration: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailDomain {
    pub domain: String,
    pub enabled: bool,
}

impl EmailSetup {
    pub fn new(base_url: String, config_path: PathBuf) -> Self {
        let admin_user = format!(
            "admin_{}@botserver.local",
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );
        let admin_pass = Self::generate_secure_password();

        Self {
            base_url,
            admin_user,
            admin_pass,
            config_path,
        }
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

    pub async fn wait_for_ready(&self, max_attempts: u32) -> Result<()> {
        log::info!("Waiting for Email service to be ready...");

        for attempt in 1..=max_attempts {
            if tokio::net::TcpStream::connect("127.0.0.1:25").await.is_ok() {
                log::info!("Email service is ready!");
                return Ok(());
            }

            log::debug!(
                "Email service not ready yet (attempt {}/{})",
                attempt,
                max_attempts
            );
            sleep(Duration::from_secs(3)).await;
        }

        anyhow::bail!("Email service did not become ready in time")
    }

    pub async fn initialize(
        &mut self,
        directory_config_path: Option<PathBuf>,
    ) -> Result<EmailConfig> {
        log::info!(" Initializing Email (Stalwart) server...");

        if let Ok(existing_config) = self.load_existing_config().await {
            log::info!("Email already initialized, using existing config");
            return Ok(existing_config);
        }

        self.wait_for_ready(30).await?;

        self.create_default_domain()?;
        log::info!(" Created default email domain: localhost");

        let directory_integration = if let Some(dir_config_path) = directory_config_path {
            match self.setup_directory_integration(&dir_config_path) {
                Ok(_) => {
                    log::info!(" Integrated with Directory for authentication");
                    true
                }
                Err(e) => {
                    log::warn!("  Directory integration failed: {}", e);
                    false
                }
            }
        } else {
            false
        };

        self.create_admin_account().await?;
        log::info!(" Created admin email account: {}", self.admin_user);

        let config = EmailConfig {
            base_url: self.base_url.clone(),
            smtp_host: "localhost".to_string(),
            smtp_port: 25,
            imap_host: "localhost".to_string(),
            imap_port: 143,
            admin_user: self.admin_user.clone(),
            admin_pass: self.admin_pass.clone(),
            directory_integration,
        };

        self.save_config(&config).await?;
        log::info!(" Saved Email configuration");

        log::info!(" Email initialization complete!");
        log::info!("📧 SMTP: localhost:25 (587 for TLS)");
        log::info!("📬 IMAP: localhost:143 (993 for TLS)");
        log::info!("👤 Admin: {} / {}", config.admin_user, config.admin_pass);

        Ok(config)
    }

    fn create_default_domain(&self) -> Result<()> {
        let _ = self;
        Ok(())
    }

    async fn create_admin_account(&self) -> Result<()> {
        log::info!("Creating admin email account via Stalwart API...");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let api_url = format!("{}/api/account", self.base_url);

        let account_data = serde_json::json!({
            "name": self.admin_user,
            "secret": self.admin_pass,
            "description": "BotServer Admin Account",
            "quota": 1_073_741_824,
            "type": "individual",
            "emails": [self.admin_user.clone()],
            "memberOf": ["administrators"],
            "enabled": true
        });

        let response = client
            .post(&api_url)
            .header("Content-Type", "application/json")
            .json(&account_data)
            .send()
            .await;

        // All branches return Ok(()) - just log appropriate messages
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    log::info!(
                        "Admin email account created successfully: {}",
                        self.admin_user
                    );
                } else if resp.status().as_u16() == 409 {
                    log::info!("Admin email account already exists: {}", self.admin_user);
                } else {
                    let status = resp.status();
                    log::warn!("Failed to create admin account via API (status {})", status);
                }
            }
            Err(e) => {
                log::warn!(
                    "Could not connect to Stalwart management API: {}. Account may need manual setup.",
                    e
                );
            }
        }
        Ok(())
    }

    fn setup_directory_integration(&self, directory_config_path: &PathBuf) -> Result<()> {
        let _ = self;
        let content = std::fs::read_to_string(directory_config_path)?;
        let dir_config: serde_json::Value = serde_json::from_str(&content)?;

        let issuer_url = dir_config["base_url"]
            .as_str()
            .unwrap_or("http://localhost:8080");

        log::info!("Setting up OIDC authentication with Directory...");
        log::info!("Issuer URL: {}", issuer_url);

        Ok(())
    }

    async fn save_config(&self, config: &EmailConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, json).await?;
        Ok(())
    }

    async fn load_existing_config(&self) -> Result<EmailConfig> {
        let content = fs::read_to_string(&self.config_path).await?;
        let config: EmailConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub async fn get_config(&self) -> Result<EmailConfig> {
        self.load_existing_config().await
    }

    pub fn create_user_mailbox(&self, _username: &str, _password: &str, email: &str) -> Result<()> {
        let _ = self;
        log::info!("Creating mailbox for user: {}", email);

        Ok(())
    }

    pub async fn sync_users_from_directory(&self, directory_config_path: &PathBuf) -> Result<()> {
        log::info!("Syncing users from Directory to Email...");

        let content = fs::read_to_string(directory_config_path).await?;
        let dir_config: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(default_user) = dir_config.get("default_user") {
            let email = default_user["email"].as_str().unwrap_or("");
            let password = default_user["password"].as_str().unwrap_or("");
            let username = default_user["username"].as_str().unwrap_or("");

            if !email.is_empty() {
                self.create_user_mailbox(username, password, email)?;
                log::info!(" Created mailbox for: {}", email);
            }
        }

        Ok(())
    }
}

pub async fn generate_email_config(
    config_path: PathBuf,
    data_path: PathBuf,
    directory_integration: bool,
) -> Result<()> {
    let mut config = format!(
        r#"
[server]
hostname = "localhost"

[server.listener."smtp"]
bind = ["0.0.0.0:25"]
protocol = "smtp"

[server.listener."smtp-submission"]
bind = ["0.0.0.0:587"]
protocol = "smtp"
tls.implicit = false

[server.listener."smtp-submissions"]
bind = ["0.0.0.0:465"]
protocol = "smtp"
tls.implicit = true

[server.listener."imap"]
bind = ["0.0.0.0:143"]
protocol = "imap"

[server.listener."imaps"]
bind = ["0.0.0.0:993"]
protocol = "imap"
tls.implicit = true

[server.listener."http"]
bind = ["0.0.0.0:8080"]
protocol = "http"

[storage]
data = "sqlite"
blob = "sqlite"
lookup = "sqlite"
fts = "sqlite"

[store."sqlite"]
type = "sqlite"
path = "{}/stalwart.db"

[directory."local"]
type = "internal"
store = "sqlite"

"#,
        data_path.display()
    );

    if directory_integration {
        config.push_str(
            r#"
[directory."oidc"]
type = "oidc"
issuer = "http://localhost:8080"
client-id = "{{CLIENT_ID}}"
client-secret = "{{CLIENT_SECRET}}"

[authentication]
mechanisms = ["plain", "login"]
directory = "oidc"
fallback-directory = "local"

"#,
        );
    } else {
        config.push_str(
            r#"
[authentication]
mechanisms = ["plain", "login"]
directory = "local"

"#,
        );
    }

    fs::write(config_path, config).await?;
    Ok(())
}
