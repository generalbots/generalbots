#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitadelConfig {
    pub issuer_url: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub project_id: String,
    pub api_url: String,
    pub service_account_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ZitadelClient {
    config: ZitadelConfig,
    http_client: reqwest::Client,
    access_token: Arc<RwLock<Option<String>>>,
}

impl ZitadelClient {
    pub async fn new(config: ZitadelConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            http_client,
            access_token: Arc::new(RwLock::new(None)),
        })
    }

    /// Get the API base URL
    pub fn api_url(&self) -> &str {
        &self.config.api_url
    }

    /// Make a GET request with authentication
    pub async fn http_get(&self, url: String) -> reqwest::RequestBuilder {
        let token = self.get_access_token().await.unwrap_or_default();
        self.http_client.get(url).bearer_auth(token)
    }

    /// Make a POST request with authentication
    pub async fn http_post(&self, url: String) -> reqwest::RequestBuilder {
        let token = self.get_access_token().await.unwrap_or_default();
        self.http_client.post(url).bearer_auth(token)
    }

    /// Make a PUT request with authentication
    pub async fn http_put(&self, url: String) -> reqwest::RequestBuilder {
        let token = self.get_access_token().await.unwrap_or_default();
        self.http_client.put(url).bearer_auth(token)
    }

    /// Make a PATCH request with authentication
    pub async fn http_patch(&self, url: String) -> reqwest::RequestBuilder {
        let token = self.get_access_token().await.unwrap_or_default();
        self.http_client.patch(url).bearer_auth(token)
    }

    pub async fn get_access_token(&self) -> Result<String> {
        // Check if we have a cached token
        {
            let token = self.access_token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        // Get new token using client credentials
        let token_url = format!("{}/oauth/v2/token", self.config.api_url);

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("scope", "openid profile email"),
        ];

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

        // Cache the token
        {
            let mut token = self.access_token.write().await;
            *token = Some(access_token.clone());
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
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/users/human", self.config.api_url);

        let body = serde_json::json!({
            "userName": username.unwrap_or(email),
            "profile": {
                "givenName": first_name,
                "familyName": last_name,
                "displayName": format!("{} {}", first_name, last_name)
            },
            "email": {
                "email": email,
                "isVerified": false
            }
        });

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

    pub async fn get_user(&self, user_id: &str) -> Result<serde_json::Value> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/users/{}", self.config.api_url, user_id);

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
            "{}/v2/users?limit={}&offset={}",
            self.config.api_url, limit, offset
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
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
            .map(|arr| arr.iter().cloned().collect())
            .unwrap_or_default();

        Ok(users)
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/users/_search", self.config.api_url);

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
            .map(|arr| arr.iter().cloned().collect())
            .unwrap_or_default();

        Ok(users)
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
            self.config.api_url, user_id, limit, offset
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
            self.config.api_url, org_id
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
            self.config.api_url, org_id, user_id
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
            self.config.api_url, org_id
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
            .map(|arr| arr.iter().cloned().collect())
            .unwrap_or_default();

        Ok(members)
    }

    pub async fn get_organization(&self, org_id: &str) -> Result<serde_json::Value> {
        let token = self.get_access_token().await?;
        let url = format!("{}/v2/organizations/{}", self.config.api_url, org_id);

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
        let url = format!("{}/oauth/v2/introspect", self.config.api_url);

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
        // Check if user has specific permission on resource
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v2/users/{}/permissions/check",
            self.config.api_url, user_id
        );

        let check_payload = serde_json::json!({
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

        // Simple check - in production, parse and validate permissions
        Ok(true)
    }
}
