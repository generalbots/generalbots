use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
#[cfg(test)]
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitadelConfig {
    pub issuer_url: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    pub sub: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub exp: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ZitadelAuth {
    pub config: ZitadelConfig,
    pub client: Client,
    pub work_root: PathBuf,
}

/// Zitadel API client for direct API interactions
#[derive(Debug, Clone)]
pub struct ZitadelClient {
    pub config: ZitadelConfig,
    pub client: Client,
    pub base_url: String,
    pub access_token: Option<String>,
}

impl ZitadelClient {
    /// Create a new Zitadel client
    pub fn new(config: ZitadelConfig) -> Self {
        let base_url = config.issuer_url.trim_end_matches('/').to_string();
        Self {
            config,
            client: Client::new(),
            base_url,
            access_token: None,
        }
    }

    /// Authenticate and get access token
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<serde_json::Value> {
        let response = self
            .client
            .post(format!("{}/oauth/v2/token", self.base_url))
            .form(&[
                ("grant_type", "password"),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("username", email),
                ("password", password),
                ("scope", "openid profile email"),
            ])
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        password: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({
            "userName": email,
            "profile": {
                "firstName": first_name,
                "lastName": last_name,
                "displayName": format!("{} {}", first_name, last_name)
            },
            "email": {
                "email": email,
                "isEmailVerified": false
            }
        });

        if let Some(pwd) = password {
            body["password"] = serde_json::json!(pwd);
        }

        let response = self
            .client
            .post(format!(
                "{}/management/v1/users/human/_import",
                self.base_url
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<serde_json::Value> {
        let response = self
            .client
            .get(format!("{}/management/v1/users/{}", self.base_url, user_id))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Search users
    pub async fn search_users(&self, query: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": {
                "offset": 0,
                "limit": 100,
                "asc": true
            },
            "queries": [{"userNameQuery": {"userName": query, "method": "TEXT_QUERY_METHOD_CONTAINS"}}]
        });

        let response = self
            .client
            .post(format!("{}/management/v1/users/_search", self.base_url))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Update user profile
    pub async fn update_user_profile(
        &self,
        user_id: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut body = serde_json::json!({});

        if let Some(fn_val) = first_name {
            body["firstName"] = serde_json::json!(fn_val);
        }
        if let Some(ln_val) = last_name {
            body["lastName"] = serde_json::json!(ln_val);
        }
        if let Some(dn_val) = display_name {
            body["displayName"] = serde_json::json!(dn_val);
        }

        let response = self
            .client
            .put(format!(
                "{}/management/v1/users/{}/profile",
                self.base_url, user_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Deactivate user
    pub async fn deactivate_user(&self, user_id: &str) -> Result<serde_json::Value> {
        let response = self
            .client
            .post(format!(
                "{}/management/v1/users/{}/deactivate",
                self.base_url, user_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// List users with pagination
    pub async fn list_users(&self, offset: u32, limit: u32) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": {
                "offset": offset,
                "limit": limit,
                "asc": true
            }
        });

        let response = self
            .client
            .post(format!("{}/management/v1/users/_search", self.base_url))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Create organization
    pub async fn create_organization(&self, name: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "name": name
        });

        let response = self
            .client
            .post(format!("{}/management/v1/orgs", self.base_url))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Get organization by ID
    pub async fn get_organization(&self, org_id: &str) -> Result<serde_json::Value> {
        let response = self
            .client
            .get(format!("{}/management/v1/orgs/{}", self.base_url, org_id))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Update organization
    pub async fn update_organization(&self, org_id: &str, name: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "name": name
        });

        let response = self
            .client
            .put(format!("{}/management/v1/orgs/{}", self.base_url, org_id))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Deactivate organization
    pub async fn deactivate_organization(&self, org_id: &str) -> Result<serde_json::Value> {
        let response = self
            .client
            .post(format!(
                "{}/management/v1/orgs/{}/deactivate",
                self.base_url, org_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// List organizations
    pub async fn list_organizations(&self, offset: u32, limit: u32) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": {
                "offset": offset,
                "limit": limit,
                "asc": true
            }
        });

        let response = self
            .client
            .post(format!("{}/management/v1/orgs/_search", self.base_url))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Add member to organization
    pub async fn add_org_member(
        &self,
        org_id: &str,
        user_id: &str,
        roles: Vec<String>,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "userId": user_id,
            "roles": roles
        });

        let response = self
            .client
            .post(format!(
                "{}/management/v1/orgs/{}/members",
                self.base_url, org_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Remove member from organization
    pub async fn remove_org_member(
        &self,
        org_id: &str,
        user_id: &str,
    ) -> Result<serde_json::Value> {
        let response = self
            .client
            .delete(format!(
                "{}/management/v1/orgs/{}/members/{}",
                self.base_url, org_id, user_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Get organization members
    pub async fn get_org_members(
        &self,
        org_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": {
                "offset": offset,
                "limit": limit,
                "asc": true
            }
        });

        let response = self
            .client
            .post(format!(
                "{}/management/v1/orgs/{}/members/_search",
                self.base_url, org_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Get user memberships
    pub async fn get_user_memberships(
        &self,
        user_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": {
                "offset": offset,
                "limit": limit,
                "asc": true
            }
        });

        let response = self
            .client
            .post(format!(
                "{}/management/v1/users/{}/memberships/_search",
                self.base_url, user_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Grant role to user
    pub async fn grant_role(&self, user_id: &str, role_key: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "roleKeys": [role_key]
        });

        let response = self
            .client
            .post(format!(
                "{}/management/v1/users/{}/grants",
                self.base_url, user_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Revoke role from user
    pub async fn revoke_role(&self, user_id: &str, grant_id: &str) -> Result<serde_json::Value> {
        let response = self
            .client
            .delete(format!(
                "{}/management/v1/users/{}/grants/{}",
                self.base_url, user_id, grant_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Get user grants
    pub async fn get_user_grants(
        &self,
        user_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": {
                "offset": offset,
                "limit": limit,
                "asc": true
            }
        });

        let response = self
            .client
            .post(format!(
                "{}/management/v1/users/{}/grants/_search",
                self.base_url, user_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    /// Check permission for user
    pub async fn check_permission(&self, user_id: &str, permission: &str) -> Result<bool> {
        let body = serde_json::json!({
            "permission": permission
        });

        let response = self
            .client
            .post(format!(
                "{}/management/v1/users/{}/permissions/_check",
                self.base_url, user_id
            ))
            .bearer_auth(self.access_token.as_ref().unwrap_or(&String::new()))
            .json(&body)
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data
            .get("result")
            .and_then(|r| r.as_bool())
            .unwrap_or(false))
    }

    /// Introspect token
    pub async fn introspect_token(&self, token: &str) -> Result<IntrospectionResponse> {
        let response = self
            .client
            .post(format!("{}/oauth/v2/introspect", self.base_url))
            .form(&[
                ("token", token),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        let intro = response.json::<IntrospectionResponse>().await?;
        Ok(intro)
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let response = self
            .client
            .post(format!("{}/oauth/v2/token", self.base_url))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        let token = response.json::<TokenResponse>().await?;
        Ok(token)
    }
}

impl ZitadelAuth {
    pub fn new(config: ZitadelConfig, work_root: PathBuf) -> Self {
        Self {
            config,
            client: Client::new(),
            work_root,
        }
    }

    /// Get OAuth2 authorization URL
    pub fn get_authorization_url(&self, state: &str) -> String {
        format!(
            "{}/oauth/v2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid profile email&state={}",
            self.config.issuer_url, self.config.client_id, self.config.redirect_uri, state
        )
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let response = self
            .client
            .post(format!("{}/oauth/v2/token", self.config.issuer_url))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.config.redirect_uri),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        let token = response.json::<TokenResponse>().await?;
        Ok(token)
    }

    /// Verify and decode JWT token
    pub async fn verify_token(&self, token: &str) -> Result<ZitadelUser> {
        let response = self
            .client
            .post(format!("{}/oauth/v2/introspect", self.config.issuer_url))
            .form(&[
                ("token", token),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        let intro: IntrospectionResponse = response.json().await?;

        if !intro.active {
            anyhow::bail!("Token is not active");
        }

        Ok(ZitadelUser {
            sub: intro.sub.unwrap_or_default(),
            name: intro.username.clone().unwrap_or_default(),
            email: intro.email.unwrap_or_default(),
            email_verified: true,
            preferred_username: intro.username.unwrap_or_default(),
            given_name: None,
            family_name: None,
            picture: None,
        })
    }

    /// Get user info from userinfo endpoint
    pub async fn get_user_info(&self, access_token: &str) -> Result<ZitadelUser> {
        let response = self
            .client
            .get(format!("{}/oidc/v1/userinfo", self.config.issuer_url))
            .bearer_auth(access_token)
            .send()
            .await?;

        let user = response.json::<ZitadelUser>().await?;
        Ok(user)
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let response = self
            .client
            .post(format!("{}/oauth/v2/token", self.config.issuer_url))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        let token = response.json::<TokenResponse>().await?;
        Ok(token)
    }

    /// Initialize user workspace directories
    pub async fn initialize_user_workspace(&self, user_id: &str) -> Result<UserWorkspace> {
        let workspace = UserWorkspace::new(&self.work_root, user_id);
        workspace.create_directories().await?;
        Ok(workspace)
    }

    /// Get user workspace paths
    pub fn get_user_workspace(&self, user_id: &str) -> UserWorkspace {
        UserWorkspace::new(&self.work_root, user_id)
    }
}

/// User workspace directory structure
#[derive(Debug, Clone)]
pub struct UserWorkspace {
    pub root: PathBuf,
}

impl UserWorkspace {
    pub fn new(work_root: &PathBuf, user_id: &str) -> Self {
        Self {
            root: work_root.join("users").join(user_id),
        }
    }

    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn vectordb_root(&self) -> PathBuf {
        self.root.join("vectordb")
    }

    pub fn email_vectordb(&self) -> PathBuf {
        self.vectordb_root().join("email")
    }

    pub fn drive_vectordb(&self) -> PathBuf {
        self.vectordb_root().join("drive")
    }

    pub fn cache_root(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn email_cache(&self) -> PathBuf {
        self.cache_root().join("email")
    }

    pub fn drive_cache(&self) -> PathBuf {
        self.cache_root().join("drive")
    }

    pub fn preferences_root(&self) -> PathBuf {
        self.root.join("preferences")
    }

    pub fn email_settings(&self) -> PathBuf {
        self.preferences_root().join("email.json")
    }

    pub fn drive_settings(&self) -> PathBuf {
        self.preferences_root().join("drive.json")
    }

    pub fn temp_root(&self) -> PathBuf {
        self.root.join("temp")
    }

    /// Create all workspace directories
    pub async fn create_directories(&self) -> Result<()> {
        let dirs = vec![
            self.vectordb_root(),
            self.email_vectordb(),
            self.drive_vectordb(),
            self.cache_root(),
            self.email_cache(),
            self.drive_cache(),
            self.preferences_root(),
            self.temp_root(),
        ];

        for dir in dirs {
            fs::create_dir_all(&dir).await?;
        }

        Ok(())
    }

    /// Clean temporary files
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

        let mut entries = fs::read_dir(&self.root).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += self.get_dir_size(&entry.path()).await?;
            }
        }

        Ok(total_size)
    }

    fn get_dir_size<'a>(
        &'a self,
        path: &'a PathBuf,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + 'a>> {
        Box::pin(async move {
            let mut total_size = 0u64;

            let mut entries = fs::read_dir(path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let metadata = entry.metadata().await?;
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    total_size += self.get_dir_size(&entry.path()).await?;
                }
            }

            Ok(total_size)
        })
    }

    /// Delete entire workspace
    pub async fn delete_workspace(&self) -> Result<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).await?;
        }
        Ok(())
    }
}

/// Extract user ID from JWT token (without full validation)
pub fn extract_user_id_from_token(token: &str) -> Result<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid JWT token format");
    }

    let payload = URL_SAFE_NO_PAD.decode(parts[1])?;
    let claims: serde_json::Value = serde_json::from_slice(&payload)?;

    claims
        .get("sub")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No 'sub' claim in token"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_paths() {
        let work_root = PathBuf::from("/tmp/work");
        let user_id = "user123";
        let workspace = UserWorkspace::new(&work_root, user_id);

        assert_eq!(workspace.root(), PathBuf::from("/tmp/work/users/user123"));
        assert_eq!(
            workspace.email_vectordb(),
            PathBuf::from("/tmp/work/users/user123/vectordb/email")
        );
        assert_eq!(
            workspace.drive_cache(),
            PathBuf::from("/tmp/work/users/user123/cache/drive")
        );
    }

    #[tokio::test]
    async fn test_workspace_creation() {
        let temp_dir = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let user_id = "test_user";
        let workspace = UserWorkspace::new(&temp_dir, user_id);

        workspace.create_directories().await.unwrap();

        assert!(workspace.root().exists());
        assert!(workspace.email_vectordb().exists());
        assert!(workspace.drive_cache().exists());

        // Cleanup
        workspace.delete_workspace().await.unwrap();
    }
}
