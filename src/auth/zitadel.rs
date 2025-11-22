use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ZitadelConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZitadelUser {
    pub sub: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub preferred_username: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub id_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    pub sub: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub exp: Option<u64>,
}

pub struct ZitadelAuth {
    config: ZitadelConfig,
    client: Client,
    work_root: PathBuf,
}

impl ZitadelAuth {
    pub fn new(config: ZitadelConfig, work_root: PathBuf) -> Self {
        Self {
            config,
            client: Client::new(),
            work_root,
        }
    }

    /// Generate authorization URL for OAuth2 flow
    pub fn get_authorization_url(&self, state: &str) -> String {
        format!(
            "{}/oauth/v2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email&state={}",
            self.config.issuer_url,
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            state
        )
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let token_url = format!("{}/oauth/v2/token", self.config.issuer_url);

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .client
            .post(&token_url)
            .form(&params)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(response)
    }

    /// Verify and decode JWT token
    pub async fn verify_token(&self, token: &str) -> Result<ZitadelUser> {
        let introspect_url = format!("{}/oauth/v2/introspect", self.config.issuer_url);

        let params = [
            ("token", token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let introspection: IntrospectionResponse = self
            .client
            .post(&introspect_url)
            .form(&params)
            .send()
            .await?
            .json()
            .await?;

        if !introspection.active {
            anyhow::bail!("Token is not active");
        }

        // Fetch user info
        self.get_user_info(token).await
    }

    /// Get user information from userinfo endpoint
    pub async fn get_user_info(&self, access_token: &str) -> Result<ZitadelUser> {
        let userinfo_url = format!("{}/oidc/v1/userinfo", self.config.issuer_url);

        let response = self
            .client
            .get(&userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<ZitadelUser>()
            .await?;

        Ok(response)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let token_url = format!("{}/oauth/v2/token", self.config.issuer_url);

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .client
            .post(&token_url)
            .form(&params)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(response)
    }

    /// Initialize user workspace directories
    pub async fn initialize_user_workspace(
        &self,
        bot_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<UserWorkspace> {
        let workspace = UserWorkspace::new(self.work_root.clone(), bot_id, user_id);
        workspace.create_directories().await?;
        Ok(workspace)
    }

    /// Get or create user workspace
    pub async fn get_user_workspace(&self, bot_id: &Uuid, user_id: &Uuid) -> Result<UserWorkspace> {
        let workspace = UserWorkspace::new(self.work_root.clone(), bot_id, user_id);

        // Create if doesn't exist
        if !workspace.root().exists() {
            workspace.create_directories().await?;
        }

        Ok(workspace)
    }
}

/// User workspace structure for per-user data isolation
#[derive(Debug, Clone)]
pub struct UserWorkspace {
    root: PathBuf,
    bot_id: Uuid,
    user_id: Uuid,
}

impl UserWorkspace {
    pub fn new(work_root: PathBuf, bot_id: &Uuid, user_id: &Uuid) -> Self {
        Self {
            root: work_root.join(bot_id.to_string()).join(user_id.to_string()),
            bot_id: *bot_id,
            user_id: *user_id,
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn vectordb_root(&self) -> PathBuf {
        self.root.join("vectordb")
    }

    pub fn email_vectordb(&self) -> PathBuf {
        self.vectordb_root().join("emails")
    }

    pub fn drive_vectordb(&self) -> PathBuf {
        self.vectordb_root().join("drive")
    }

    pub fn cache_root(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn email_cache(&self) -> PathBuf {
        self.cache_root().join("email_metadata.db")
    }

    pub fn drive_cache(&self) -> PathBuf {
        self.cache_root().join("drive_metadata.db")
    }

    pub fn preferences_root(&self) -> PathBuf {
        self.root.join("preferences")
    }

    pub fn email_settings(&self) -> PathBuf {
        self.preferences_root().join("email_settings.json")
    }

    pub fn drive_settings(&self) -> PathBuf {
        self.preferences_root().join("drive_sync.json")
    }

    pub fn temp_root(&self) -> PathBuf {
        self.root.join("temp")
    }

    /// Create all necessary directories for user workspace
    pub async fn create_directories(&self) -> Result<()> {
        let directories = vec![
            self.root.clone(),
            self.vectordb_root(),
            self.email_vectordb(),
            self.drive_vectordb(),
            self.cache_root(),
            self.preferences_root(),
            self.temp_root(),
        ];

        for dir in directories {
            if !dir.exists() {
                fs::create_dir_all(&dir).await?;
                log::info!("Created directory: {:?}", dir);
            }
        }

        Ok(())
    }

    /// Clean up temporary files
    pub async fn clean_temp(&self) -> Result<()> {
        let temp_dir = self.temp_root();
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).await?;
            fs::create_dir(&temp_dir).await?;
        }
        Ok(())
    }

    /// Get workspace size in bytes
    pub async fn get_size(&self) -> Result<u64> {
        let mut total_size = 0u64;

        let mut stack = vec![self.root.clone()];

        while let Some(path) = stack.pop() {
            let mut entries = fs::read_dir(&path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let metadata = entry.metadata().await?;
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    stack.push(entry.path());
                }
            }
        }

        Ok(total_size)
    }

    /// Remove entire workspace (use with caution!)
    pub async fn delete_workspace(&self) -> Result<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).await?;
            log::warn!("Deleted workspace: {:?}", self.root);
        }
        Ok(())
    }
}

/// Helper to extract user ID from JWT token
pub fn extract_user_id_from_token(token: &str) -> Result<String> {
    // Decode JWT without verification (just to extract sub)
    // In production, use proper JWT validation
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid JWT format");
    }

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let payload = URL_SAFE_NO_PAD.decode(parts[1])?;
    let json: serde_json::Value = serde_json::from_slice(&payload)?;

    json.get("sub")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No sub claim in token"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_paths() {
        let workspace = UserWorkspace::new(PathBuf::from("/tmp/work"), &Uuid::nil(), &Uuid::nil());

        assert_eq!(
            workspace.email_vectordb(),
            PathBuf::from("/tmp/work/00000000-0000-0000-0000-000000000000/00000000-0000-0000-0000-000000000000/vectordb/emails")
        );

        assert_eq!(
            workspace.drive_vectordb(),
            PathBuf::from("/tmp/work/00000000-0000-0000-0000-000000000000/00000000-0000-0000-0000-000000000000/vectordb/drive")
        );
    }

    #[tokio::test]
    async fn test_workspace_creation() {
        let temp_dir = std::env::temp_dir().join("botserver_test");
        let workspace = UserWorkspace::new(temp_dir.clone(), &Uuid::new_v4(), &Uuid::new_v4());

        workspace.create_directories().await.unwrap();

        assert!(workspace.root().exists());
        assert!(workspace.email_vectordb().exists());
        assert!(workspace.drive_vectordb().exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
