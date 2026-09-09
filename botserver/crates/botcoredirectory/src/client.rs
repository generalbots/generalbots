use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

// Fields default + alias legacy key names so a config file written by an
// older stack (e.g. `base_url` instead of `api_url`, `service_token` instead
// of `service_account_key`, or a partial JSON with only 5 keys) still
// deserializes. Without the aliases, resolve_directory_config() rejects the
// on-disk conf/system/directory_config.json and falls back to Vault, which
// may hold a raw container IP — then Zitadel 404s the token request because
// the Host header no longer matches the registered instance domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitadelConfig {
    #[serde(default)]
    pub issuer_url: String,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    #[serde(default)]
    pub redirect_uri: String,
    #[serde(default = "default_project_id")]
    pub project_id: String,
    #[serde(default, alias = "base_url")]
    pub api_url: String,
    #[serde(default, alias = "service_token")]
    pub service_account_key: Option<String>,
    /// Operator opt-in: allow plain http for the configured api_url even when
    /// the host is a public DNS name (e.g. an internal reverse proxy fronting
    /// Zitadel on a trusted network). Never enable on public internet deployments.
    #[serde(default)]
    pub allow_insecure_http: bool,
}

fn default_project_id() -> String {
    "default".to_string()
}

#[derive(Debug, Clone)]
pub struct ZitadelClient {
    config: ZitadelConfig,
    /// API base rebuilt from parsed URL components (scheme://host[:port]) so
    /// bearer tokens can never be redirected to a tampered host (SSRF guard).
    api_base: String,
    http_client: reqwest::Client,
    access_token: Arc<RwLock<Option<String>>>,
    token_expires_at: Arc<RwLock<Option<Instant>>>,
    pat_token: Option<String>,
    /// Username and password for password grant OAuth flow
    password_credentials: Option<(String, String)>,
}

impl ZitadelClient {
    /// Rebuilds the API base from validated URL components, dropping any
    /// userinfo, path or query that could retarget the request (SSRF guard).
    fn sanitize_api_base(raw: &str, allow_insecure_http: bool) -> Result<String> {
        let parsed = url::Url::parse(raw).map_err(|e| anyhow!("invalid Zitadel api_url: {e}"))?;
        let host = parsed.host_str().ok_or_else(|| anyhow!("Zitadel api_url requires a host"))?.to_string();
        // Bearer tokens and user data travel to this host: require https for
        // public destinations. Plain http is allowed when traffic provably
        // stays inside a trusted perimeter (loopback/private networks) or when
        // the operator explicitly opted in via `allow_insecure_http`.
        anyhow::ensure!(
            parsed.scheme() == "https"
                || (parsed.scheme() == "http" && (allow_insecure_http || is_private_host(&host))),
            "Zitadel api_url must be https (or http on loopback/private networks; set allow_insecure_http to override)"
        );
        let mut base = format!("{}://{}", parsed.scheme(), host);
        if let Some(port) = parsed.port() {
            base.push_str(&format!(":{port}"));
        }
        Ok(base)
    }

    pub fn new(config: ZitadelConfig) -> Result<Self> {
        let api_base = Self::sanitize_api_base(&config.api_url, config.allow_insecure_http)?;
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            api_base,
            http_client,
            access_token: Arc::new(RwLock::new(None)),
            token_expires_at: Arc::new(RwLock::new(None)),
            pat_token: None,
            password_credentials: None,
        })
    }

    /// Create a client that uses password grant OAuth flow
    /// This is used for initial bootstrap with Zitadel's default admin user
    pub fn with_password_grant(
        config: ZitadelConfig,
        username: String,
        password: String,
    ) -> Result<Self> {
        let api_base = Self::sanitize_api_base(&config.api_url, config.allow_insecure_http)?;
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            api_base,
            http_client,
            access_token: Arc::new(RwLock::new(None)),
            token_expires_at: Arc::new(RwLock::new(None)),
            pat_token: None,
            password_credentials: Some((username, password)),
        })
    }

    pub fn with_pat_token(config: ZitadelConfig, pat_token: String) -> Result<Self> {
        let api_base = Self::sanitize_api_base(&config.api_url, config.allow_insecure_http)?;
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            api_base,
            http_client,
            access_token: Arc::new(RwLock::new(None)),
            token_expires_at: Arc::new(RwLock::new(None)),
            pat_token: Some(pat_token),
            password_credentials: None,
        })
    }

    pub fn set_pat_token(&mut self, token: String) {
        self.pat_token = Some(token);
    }

    pub fn api_url(&self) -> String {
        self.api_base.clone()
    }

    pub fn client_id(&self) -> String {
        self.config.client_id.clone()
    }

    pub fn client_secret(&self) -> String {
        self.config.client_secret.clone()
    }

    pub async fn http_get(&self, url: String) -> Result<reqwest::RequestBuilder> {
        let token = self.get_access_token().await
            .map_err(|e| anyhow!("Token acquisition failed for GET {}: {}", url, e))?;
        Ok(self.http_client.get(url).bearer_auth(token))
    }

    pub async fn http_post(&self, url: String) -> Result<reqwest::RequestBuilder> {
        let token = self.get_access_token().await
            .map_err(|e| anyhow!("Token acquisition failed for POST {}: {}", url, e))?;
        Ok(self.http_client.post(url).bearer_auth(token))
    }

    pub async fn http_put(&self, url: String) -> Result<reqwest::RequestBuilder> {
        let token = self.get_access_token().await
            .map_err(|e| anyhow!("Token acquisition failed for PUT {}: {}", url, e))?;
        Ok(self.http_client.put(url).bearer_auth(token))
    }

    pub async fn http_patch(&self, url: String) -> Result<reqwest::RequestBuilder> {
        let token = self.get_access_token().await
            .map_err(|e| anyhow!("Token acquisition failed for PATCH {}: {}", url, e))?;
        Ok(self.http_client.patch(url).bearer_auth(token))
    }

    pub async fn http_delete(&self, url: String) -> Result<reqwest::RequestBuilder> {
        let token = self.get_access_token().await
            .map_err(|e| anyhow!("Token acquisition failed for DELETE {}: {}", url, e))?;
        Ok(self.http_client.delete(url).bearer_auth(token))
    }

    pub async fn get_access_token(&self) -> Result<String> {
        if let Some(ref pat) = self.pat_token {
            return Ok(pat.clone());
        }

        // Check cached token with expiry
        {
            let token = self.access_token.read().await;
            let expires = self.token_expires_at.read().await;
            if let Some(t) = token.as_ref() {
                let still_valid = expires.map(|e| Instant::now() < e).unwrap_or(true);
                if still_valid {
                    return Ok(t.clone());
                }
                log::info!("Cached access token expired, refreshing...");
            }
        }

        let token_url = format!("{}/oauth/v2/token", self.api_base);
        log::info!("Requesting access token from: {}", token_url);

        // Build params dynamically based on auth method
        let mut params: Vec<(&str, String)> = vec![
            ("client_id", self.config.client_id.clone()),
            ("client_secret", self.config.client_secret.clone()),
        ];

        if let Some((username, password)) = &self.password_credentials {
            // Use password grant flow
            params.push(("grant_type", "password".to_string()));
            params.push(("username", username.clone()));
            params.push(("password", password.clone()));
            params.push(("scope", "openid profile email urn:zitadel:iam:org:project:id:zitadel:aud".to_string()));
        } else {
            // Use client credentials flow
            params.push(("grant_type", "client_credentials".to_string()));
            params.push(("scope", "openid profile email urn:zitadel:iam:org:project:id:zitadel:aud".to_string()));
        }

        let response = self
            .http_client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get access token: {}", e))?;

        let token_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse token response: {}", e))?;

        let access_token = token_data
            .get("access_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("No access token in response"))?
            .to_string();

        // Calculate expiry with 60s safety margin
        let expires_in = token_data
            .get("expires_in")
            .and_then(|t| t.as_i64())
            .unwrap_or(3600);
        let expires_at = Instant::now()
            + std::time::Duration::from_secs(expires_in.max(60) as u64 - 60);

        {
            let mut token = self.access_token.write().await;
            *token = Some(access_token.clone());
            let mut expires = self.token_expires_at.write().await;
            *expires = Some(expires_at);
        }

        Ok(access_token)
    }

    pub async fn create_user(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        username: Option<&str>,
    ) -> Result<String> {
        self.create_user_with_password(email, first_name, last_name, username, None).await
    }

    pub async fn create_user_with_phone(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        username: Option<&str>,
        phone: Option<&str>,
        initial_password: Option<&str>,
    ) -> Result<String> {
        let token = self.get_access_token().await?;
        // Note: This Zitadel build (Jan 2024) does not expose /v2/users/human via HTTP.
        // The management API /management/v1/users/human uses firstName/lastName fields.
        let url = format!("{}/management/v1/users/human", self.api_base);

        let mut body = serde_json::json!({
            "userName": username.unwrap_or(email),
            "profile": {
                "firstName": first_name,
                "lastName": last_name,
                "displayName": format!("{} {}", first_name, last_name)
            },
            "email": {
                "email": email,
                "isVerified": true
            }
        });

        if let Some(phone_number) = phone {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("phone".to_string(), serde_json::json!({
                    "phone": phone_number,
                    "isVerified": true
                }));
            }
        }

        if let Some(password) = initial_password {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("password".to_string(), serde_json::Value::String(password.to_string()));
            }
        }

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to create user: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create user: {}", error_text));
        }

        let user_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse user response: {}", e))?;

        let user_id = user_data
            .get("userId")
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("No userId in response"))?
            .to_string();

        Ok(user_id)
    }

    pub async fn create_user_with_password(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        username: Option<&str>,
        initial_password: Option<&str>,
    ) -> Result<String> {
        self.create_user_with_phone(email, first_name, last_name, username, None, initial_password).await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<serde_json::Value> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/users/{}", self.api_base, user_id);

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get user: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get user: {}", error_text));
        }

        let user_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse user response: {}", e))?;

        Ok(user_data)
    }

    pub async fn list_users(&self, limit: u32, offset: u32) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/management/v1/users/_search?limit={}&offset={}",
            self.api_base, limit, offset
        );

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to list users: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to list users: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse users response: {}", e))?;

        let users = data
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(users)
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!("{}/management/v1/users/_search", self.api_base);

        let body = serde_json::json!({
            "queries": [{
                "userNameQuery": {
                    "userName": query,
                    "method": "TEXT_QUERY_METHOD_CONTAINS_IGNORE_CASE"
                }
            }]
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to search users: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to search users: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search response: {}", e))?;

        let users = data
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(users)
    }

    pub async fn search_users_by_phone(&self, phone: &str) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!("{}/management/v1/users/_search", self.api_base);

        let body = serde_json::json!({
            "queries": [{
                "phoneQuery": {
                    "phone": phone,
                    "method": "TEXT_QUERY_METHOD_EQUALS"
                }
            }]
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to search users by phone: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to search users by phone: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search response: {}", e))?;

        let users = data
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(users)
    }

    pub async fn search_users_by_email(&self, email: &str) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!("{}/management/v1/users/_search", self.api_base);

        let body = serde_json::json!({
            "queries": [{
                "emailQuery": {
                    "email": email,
                    "method": "TEXT_QUERY_METHOD_EQUALS"
                }
            }]
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to search users by email: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to search users by email: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search response: {}", e))?;

        let users = data
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(users)
    }

    pub async fn search_users_by_metadata(&self, key: &str, value: &str) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!("{}/management/v1/users/_search", self.api_base);

        let body = serde_json::json!({
            "queries": [{
                "metadataQuery": {
                    "key": key,
                    "value": value,
                    "method": "TEXT_QUERY_METHOD_EQUALS"
                }
            }]
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to search users by metadata: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to search users by metadata: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search response: {}", e))?;

        let users = data
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(users)
    }

    pub async fn find_or_create_user_by_phone(
        &self,
        phone: &str,
        first_name: &str,
        last_name: &str,
    ) -> Result<String> {
        let existing = self.search_users_by_phone(phone).await?;
        if let Some(user) = existing.first() {
            let user_id = user
                .get("userId")
                .or_else(|| user.get("id"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("No userId in search result"))?
                .to_string();
            return Ok(user_id);
        }

        let username = format!("phone_{}", phone.replace('+', ""));
        let email = format!("{}@whatsapp.local", username);

        self.create_user_with_phone(
            &email,
            first_name,
            last_name,
            Some(&username),
            Some(phone),
            None,
        )
        .await
    }

    pub async fn get_user_memberships(
        &self,
        user_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<serde_json::Value> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v2/users/{}/memberships?limit={}&offset={}",
            self.api_base, user_id, limit, offset
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get memberships: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get memberships: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse memberships response: {}", e))?;

        Ok(data)
    }

    pub async fn add_org_member(
        &self,
        org_id: &str,
        user_id: &str,
        roles: Vec<String>,
    ) -> Result<()> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v2/organizations/{}/members",
            self.api_base, org_id
        );

        let body = serde_json::json!({
            "userId": user_id,
            "roles": roles
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to add org member: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to add org member: {}", error_text));
        }

        Ok(())
    }

    pub async fn remove_org_member(&self, org_id: &str, user_id: &str) -> Result<()> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v2/organizations/{}/members/{}",
            self.api_base, org_id, user_id
        );

        let response = self
            .http_client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to remove org member: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to remove org member: {}", error_text));
        }

        Ok(())
    }

    pub async fn get_org_members(&self, org_id: &str) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v2/organizations/{}/members",
            self.api_base, org_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get org members: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get org members: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse org members response: {}", e))?;

        let members = data
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(members)
    }

    pub async fn get_organization(&self, org_id: &str) -> Result<serde_json::Value> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/organizations/{}", self.api_base, org_id);

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get organization: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get organization: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse organization response: {}", e))?;

        Ok(data)
    }

    pub async fn introspect_token(&self, token: &str) -> Result<serde_json::Value> {
        let url = format!("{}/oauth/v2/introspect", self.api_base);

        let params = [
            ("token", token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .http_client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to introspect token: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to introspect token: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse introspection response: {}", e))?;

        Ok(data)
    }

    pub async fn check_permission(
        &self,
        user_id: &str,
        permission: &str,
        resource: &str,
    ) -> Result<bool> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/permissions/check", self.api_base);

        let check_payload = serde_json::json!({
            "userId": user_id,
            "permission": permission,
            "resource": resource,
            "namespace": self.config.project_id.clone()
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&check_payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to check permissions: {}", e))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let data: serde_json::Value = match response.json().await {
            Ok(d) => d,
            Err(e) => {
                log::warn!("check_permission: failed to parse response body: {}", e);
                return Ok(false);
            }
        };

        Ok(data.get("result").and_then(|r| r.as_bool()).unwrap_or(false))
    }

    pub async fn set_user_password(&self, user_id: &str, password: &str, change_required: bool) -> Result<()> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/users/{}/password", self.api_base, user_id);

        let body = serde_json::json!({
            "newPassword": {
                "password": password,
                "changeRequired": change_required
            }
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to set password: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to set password: {}", error_text));
        }

        Ok(())
    }

    pub async fn list_organizations(&self, limit: u32, offset: u32) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/management/v1/orgs/_search?limit={}&offset={}",
            self.api_base, limit, offset
        );

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to list organizations: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to list organizations: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse organizations response: {}", e))?;

        let orgs = data
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(orgs)
    }

    pub async fn create_organization(&self, name: &str) -> Result<String> {
        let token = self.get_access_token().await?;
        // Note: Use management API (/management/v1/orgs) because this Zitadel build
        // does not expose /v2/organizations via HTTP.
        let url = format!("{}/management/v1/orgs", self.api_base);

        let body = serde_json::json!({ "name": name });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to create organization: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create organization: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse organization response: {}", e))?;

        let org_id = data
            .get("organizationId")
            .or_else(|| data.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No organization ID in response"))?
            .to_string();

        Ok(org_id)
    }

    pub async fn update_organization_metadata(&self, org_id: &str, metadata: serde_json::Value) -> Result<()> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/organizations/{}", self.api_base, org_id);

        let body = serde_json::json!({
            "metadata": metadata
        });

        let response = self
            .http_client
            .patch(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to update organization: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to update organization: {}", error_text));
        }

        Ok(())
    }

    pub async fn create_pat(&self, user_id: &str, display_name: &str, expiration_date: Option<&str>) -> Result<String> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/users/{}/pat", self.api_base, user_id);

        let body = if let Some(expiry) = expiration_date {
            serde_json::json!({
                "displayName": display_name,
                "expirationDate": expiry
            })
        } else {
            serde_json::json!({
                "displayName": display_name
            })
        };

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to create PAT: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create PAT: {}", error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse PAT response: {}", e))?;

        let pat_token = data
            .get("token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("No token in PAT response"))?
            .to_string();

        Ok(pat_token)
    }
}

/// True when the host provably stays inside a trusted perimeter: loopback,
/// RFC1918/ULA private addresses or link-local. DNS hostnames cannot be
/// verified as private without resolution (SSRF risk) and always need https.
fn is_private_host(host: &str) -> bool {
    let h = host.trim_start_matches('[').trim_end_matches(']');
    if h == "localhost" || h.starts_with("127.") {
        return true;
    }
    if let Ok(addr) = h.parse::<std::net::IpAddr>() {
        return match addr {
            std::net::IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
            std::net::IpAddr::V6(v6) => {
                v6.is_loopback() || (v6.segments()[0] & 0xfe00) == 0xfc00 || v6.is_unicast_link_local()
            }
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_accepts_https_public_host() {
        let base = ZitadelClient::sanitize_api_base("https://auth.example.com", false).unwrap();
        assert_eq!(base, "https://auth.example.com");
    }

    #[test]
    fn test_sanitize_accepts_http_private_ipv4() {
        let base = ZitadelClient::sanitize_api_base("http://10.157.134.9:9000", false).unwrap();
        assert_eq!(base, "http://10.157.134.9:9000");
    }

    #[test]
    fn test_sanitize_accepts_http_172_16_range() {
        let base = ZitadelClient::sanitize_api_base("http://172.16.0.10:8080", false).unwrap();
        assert_eq!(base, "http://172.16.0.10:8080");
    }

    #[test]
    fn test_sanitize_accepts_http_localhost() {
        let base = ZitadelClient::sanitize_api_base("http://localhost:9000", false).unwrap();
        assert_eq!(base, "http://localhost:9000");
    }

    #[test]
    fn test_sanitize_accepts_http_bracketed_v6_loopback() {
        let base = ZitadelClient::sanitize_api_base("http://[::1]:9000", false).unwrap();
        assert_eq!(base, "http://[::1]:9000");
    }

    #[test]
    fn test_sanitize_rejects_http_public_host() {
        assert!(ZitadelClient::sanitize_api_base("http://auth.example.com", false).is_err());
    }

    #[test]
    fn test_sanitize_rejects_http_public_ip() {
        assert!(ZitadelClient::sanitize_api_base("http://203.0.113.5", false).is_err());
    }

    #[test]
    fn test_sanitize_rejects_http_dns_name() {
        assert!(ZitadelClient::sanitize_api_base("http://directory.internal:9000", false).is_err());
    }

    #[test]
    fn test_sanitize_opt_in_allows_http_public_host() {
        let base = ZitadelClient::sanitize_api_base("http://login.example.com:8080", true).unwrap();
        assert_eq!(base, "http://login.example.com:8080");
    }

    #[test]
    fn test_sanitize_drops_path_userinfo_and_query() {
        let base = ZitadelClient::sanitize_api_base("https://user:pass@auth.example.com/path?q=1", false).unwrap();
        assert_eq!(base, "https://auth.example.com");
    }

    #[test]
    fn test_config_deserializes_with_and_without_flag() {
        let without: ZitadelConfig = serde_json::from_str(r#"{"api_url": "https://a.b"}"#).unwrap();
        assert!(!without.allow_insecure_http);
        let with: ZitadelConfig =
            serde_json::from_str(r#"{"base_url": "http://a.b", "allow_insecure_http": true}"#).unwrap();
        assert!(with.allow_insecure_http);
        assert_eq!(with.api_url, "http://a.b");
    }
}
