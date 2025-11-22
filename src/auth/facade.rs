use crate::auth::zitadel::{TokenResponse, ZitadelClient};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub members: Vec<String>,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user: User,
    pub session: Session,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub send_invitation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authentication facade trait
#[async_trait]
pub trait AuthFacade: Send + Sync {
    // User operations
    async fn create_user(&self, request: CreateUserRequest) -> Result<User>;
    async fn get_user(&self, user_id: &str) -> Result<User>;
    async fn get_user_by_email(&self, email: &str) -> Result<User>;
    async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> Result<User>;
    async fn delete_user(&self, user_id: &str) -> Result<()>;
    async fn list_users(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<User>>;
    async fn search_users(&self, query: &str) -> Result<Vec<User>>;

    // Group operations
    async fn create_group(&self, request: CreateGroupRequest) -> Result<Group>;
    async fn get_group(&self, group_id: &str) -> Result<Group>;
    async fn update_group(
        &self,
        group_id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Group>;
    async fn delete_group(&self, group_id: &str) -> Result<()>;
    async fn list_groups(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Group>>;

    // Membership operations
    async fn add_user_to_group(&self, user_id: &str, group_id: &str) -> Result<()>;
    async fn remove_user_from_group(&self, user_id: &str, group_id: &str) -> Result<()>;
    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<Group>>;
    async fn get_group_members(&self, group_id: &str) -> Result<Vec<User>>;

    // Authentication operations
    async fn authenticate(&self, email: &str, password: &str) -> Result<AuthResult>;
    async fn authenticate_with_token(&self, token: &str) -> Result<AuthResult>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResult>;
    async fn logout(&self, session_id: &str) -> Result<()>;
    async fn validate_session(&self, session_id: &str) -> Result<Session>;

    // Permission operations
    async fn grant_permission(&self, subject_id: &str, permission: &str) -> Result<()>;
    async fn revoke_permission(&self, subject_id: &str, permission: &str) -> Result<()>;
    async fn check_permission(
        &self,
        subject_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool>;
    async fn list_permissions(&self, subject_id: &str) -> Result<Vec<Permission>>;
}

/// Zitadel-based authentication facade implementation
#[derive(Debug, Clone)]
pub struct ZitadelAuthFacade {
    pub client: ZitadelClient,
    pub cache: Option<String>,
}

impl ZitadelAuthFacade {
    /// Create a new Zitadel auth facade
    pub fn new(client: ZitadelClient) -> Self {
        Self {
            client,
            cache: None,
        }
    }

    /// Create with Redis cache support
    pub fn with_cache(client: ZitadelClient, redis_url: String) -> Self {
        Self {
            client,
            cache: Some(redis_url),
        }
    }

    /// Convert Zitadel user response to internal user representation
    fn map_zitadel_user(&self, zitadel_user: &serde_json::Value) -> Result<User> {
        let user_id = zitadel_user["userId"]
            .as_str()
            .or_else(|| zitadel_user["id"].as_str())
            .unwrap_or_default()
            .to_string();

        let email = zitadel_user["email"]
            .as_str()
            .or_else(|| zitadel_user["human"]["email"]["email"].as_str())
            .unwrap_or_default()
            .to_string();

        let username = zitadel_user["userName"]
            .as_str()
            .or_else(|| zitadel_user["preferredLoginName"].as_str())
            .map(String::from);

        let first_name = zitadel_user["human"]["profile"]["firstName"]
            .as_str()
            .or_else(|| zitadel_user["profile"]["firstName"].as_str())
            .map(String::from);

        let last_name = zitadel_user["human"]["profile"]["lastName"]
            .as_str()
            .or_else(|| zitadel_user["profile"]["lastName"].as_str())
            .map(String::from);

        let display_name = zitadel_user["human"]["profile"]["displayName"]
            .as_str()
            .or_else(|| zitadel_user["profile"]["displayName"].as_str())
            .or_else(|| zitadel_user["displayName"].as_str())
            .unwrap_or_default()
            .to_string();

        let is_active = zitadel_user["state"]
            .as_str()
            .map(|s| s.contains("ACTIVE"))
            .unwrap_or(true);

        let is_verified = zitadel_user["human"]["email"]["isEmailVerified"]
            .as_bool()
            .or_else(|| zitadel_user["emailVerified"].as_bool())
            .unwrap_or(false);

        Ok(User {
            id: user_id,
            email,
            username,
            first_name,
            last_name,
            display_name,
            avatar_url: None,
            groups: vec![],
            roles: vec![],
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            is_active,
            is_verified,
        })
    }

    /// Convert Zitadel organization to internal group representation
    fn map_zitadel_org(&self, org: &serde_json::Value) -> Result<Group> {
        Ok(Group {
            id: org["id"].as_str().unwrap_or_default().to_string(),
            name: org["name"].as_str().unwrap_or_default().to_string(),
            description: org["description"].as_str().map(String::from),
            parent_id: org["parentId"].as_str().map(String::from),
            members: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Create session from token response
    fn create_session(&self, user_id: String, token_response: &TokenResponse) -> Session {
        let expires_at = Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64);

        Session {
            id: Uuid::new_v4().to_string(),
            user_id,
            token: token_response.access_token.clone(),
            refresh_token: token_response.refresh_token.clone(),
            expires_at,
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        }
    }
}

#[async_trait]
impl AuthFacade for ZitadelAuthFacade {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        let first_name = request.first_name.as_deref().unwrap_or("");
        let last_name = request.last_name.as_deref().unwrap_or("");
        let password = request.password.as_deref();

        let response = self
            .client
            .create_user(&request.email, first_name, last_name, password)
            .await?;

        let mut user = self.map_zitadel_user(&response)?;

        // Add user to groups if specified
        for group_id in &request.groups {
            let _ = self.client.add_org_member(group_id, &user.id, vec![]).await;
        }

        // Grant roles if specified
        for role in &request.roles {
            let _ = self.client.grant_role(&user.id, role).await;
        }

        user.groups = request.groups.clone();
        user.roles = request.roles.clone();

        Ok(user)
    }

    async fn get_user(&self, user_id: &str) -> Result<User> {
        let response = self.client.get_user(user_id).await?;
        let mut user = self.map_zitadel_user(&response)?;

        // Get user's groups (memberships)
        let memberships_response = self.client.get_user_memberships(user_id, 0, 100).await?;
        if let Some(result) = memberships_response["result"].as_array() {
            user.groups = result
                .iter()
                .filter_map(|m| m["orgId"].as_str().map(String::from))
                .collect();
        }

        // Get user's roles (grants)
        let grants_response = self.client.get_user_grants(user_id, 0, 100).await?;
        if let Some(result) = grants_response["result"].as_array() {
            user.roles = result
                .iter()
                .filter_map(|g| g["roleKeys"].as_array())
                .flat_map(|keys| keys.iter())
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
        }

        Ok(user)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let response = self.client.search_users(email).await?;

        let users = response["result"]
            .as_array()
            .ok_or_else(|| anyhow!("No users found"))?;

        if users.is_empty() {
            return Err(anyhow!("User not found"));
        }

        let user_data = &users[0];
        let user_id = user_data["userId"]
            .as_str()
            .or_else(|| user_data["id"].as_str())
            .ok_or_else(|| anyhow!("User ID not found"))?;

        self.get_user(user_id).await
    }

    async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> Result<User> {
        self.client
            .update_user_profile(
                user_id,
                request.first_name.as_deref(),
                request.last_name.as_deref(),
                request.display_name.as_deref(),
            )
            .await?;

        self.get_user(user_id).await
    }

    async fn delete_user(&self, user_id: &str) -> Result<()> {
        self.client.deactivate_user(user_id).await?;
        Ok(())
    }

    async fn list_users(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<User>> {
        let offset = offset.unwrap_or(0) as u32;
        let limit = limit.unwrap_or(100) as u32;

        let response = self.client.list_users(offset, limit).await?;

        let mut users = Vec::new();
        if let Some(result) = response["result"].as_array() {
            for user_data in result {
                if let Ok(user) = self.map_zitadel_user(user_data) {
                    users.push(user);
                }
            }
        }

        Ok(users)
    }

    async fn search_users(&self, query: &str) -> Result<Vec<User>> {
        let response = self.client.search_users(query).await?;

        let mut users = Vec::new();
        if let Some(result) = response["result"].as_array() {
            for user_data in result {
                if let Ok(user) = self.map_zitadel_user(user_data) {
                    users.push(user);
                }
            }
        }

        Ok(users)
    }

    async fn create_group(&self, request: CreateGroupRequest) -> Result<Group> {
        let response = self.client.create_organization(&request.name).await?;

        let org_id = response["organizationId"]
            .as_str()
            .or_else(|| response["id"].as_str())
            .ok_or_else(|| anyhow!("Organization ID not found"))?
            .to_string();

        Ok(Group {
            id: org_id,
            name: request.name,
            description: request.description,
            parent_id: request.parent_id,
            members: vec![],
            permissions: request.permissions,
            metadata: request.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_group(&self, group_id: &str) -> Result<Group> {
        let response = self.client.get_organization(group_id).await?;
        self.map_zitadel_org(&response)
    }

    async fn update_group(
        &self,
        group_id: &str,
        name: Option<String>,
        _description: Option<String>,
    ) -> Result<Group> {
        if let Some(name) = name {
            self.client.update_organization(group_id, &name).await?;
        }

        self.get_group(group_id).await
    }

    async fn delete_group(&self, group_id: &str) -> Result<()> {
        self.client.deactivate_organization(group_id).await?;
        Ok(())
    }

    async fn list_groups(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Group>> {
        let offset = offset.unwrap_or(0) as u32;
        let limit = limit.unwrap_or(100) as u32;

        let response = self.client.list_organizations(offset, limit).await?;

        let mut groups = Vec::new();
        if let Some(result) = response["result"].as_array() {
            for org_data in result {
                if let Ok(group) = self.map_zitadel_org(org_data) {
                    groups.push(group);
                }
            }
        }

        Ok(groups)
    }

    async fn add_user_to_group(&self, user_id: &str, group_id: &str) -> Result<()> {
        self.client
            .add_org_member(group_id, user_id, vec![])
            .await?;
        Ok(())
    }

    async fn remove_user_from_group(&self, user_id: &str, group_id: &str) -> Result<()> {
        self.client.remove_org_member(group_id, user_id).await?;
        Ok(())
    }

    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<Group>> {
        let response = self.client.get_user_memberships(user_id, 0, 100).await?;

        let mut groups = Vec::new();
        if let Some(result) = response["result"].as_array() {
            for membership in result {
                if let Some(org_id) = membership["orgId"].as_str() {
                    if let Ok(group) = self.get_group(org_id).await {
                        groups.push(group);
                    }
                }
            }
        }

        Ok(groups)
    }

    async fn get_group_members(&self, group_id: &str) -> Result<Vec<User>> {
        let response = self.client.get_org_members(group_id, 0, 100).await?;

        let mut members = Vec::new();
        if let Some(result) = response["result"].as_array() {
            for member_data in result {
                if let Some(user_id) = member_data["userId"].as_str() {
                    if let Ok(user) = self.get_user(user_id).await {
                        members.push(user);
                    }
                }
            }
        }

        Ok(members)
    }

    async fn authenticate(&self, email: &str, password: &str) -> Result<AuthResult> {
        let auth_response = self.client.authenticate(email, password).await?;

        let access_token = auth_response["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("No access token in response"))?
            .to_string();

        let refresh_token = auth_response["refresh_token"].as_str().map(String::from);

        let expires_in = auth_response["expires_in"].as_i64().unwrap_or(3600);

        // Get user info
        let user = self.get_user_by_email(email).await?;

        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            token: access_token.clone(),
            refresh_token: refresh_token.clone(),
            expires_at: Utc::now() + chrono::Duration::seconds(expires_in),
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        };

        Ok(AuthResult {
            user,
            session,
            access_token,
            refresh_token,
            expires_in,
        })
    }

    async fn authenticate_with_token(&self, token: &str) -> Result<AuthResult> {
        let intro = self.client.introspect_token(token).await?;

        if !intro.active {
            return Err(anyhow!("Token is not active"));
        }

        let user_id = intro.sub.ok_or_else(|| anyhow!("No user ID in token"))?;
        let user = self.get_user(&user_id).await?;

        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            token: token.to_string(),
            refresh_token: None,
            expires_at: intro
                .exp
                .map(|exp| {
                    DateTime::<Utc>::from_timestamp(exp as i64, 0)
                        .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1))
                })
                .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1)),
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        };

        Ok(AuthResult {
            user,
            session,
            access_token: token.to_string(),
            refresh_token: None,
            expires_in: 3600,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResult> {
        let token_response = self.client.refresh_token(refresh_token).await?;

        // Extract user ID from token
        let intro = self
            .client
            .introspect_token(&token_response.access_token)
            .await?;

        let user_id = intro.sub.ok_or_else(|| anyhow!("No user ID in token"))?;
        let user = self.get_user(&user_id).await?;

        let session = self.create_session(user.id.clone(), &token_response);

        Ok(AuthResult {
            user,
            session: session.clone(),
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_in: token_response.expires_in as i64,
        })
    }

    async fn logout(&self, _session_id: &str) -> Result<()> {
        // Zitadel doesn't have a direct logout endpoint
        // Tokens need to expire or be revoked
        Ok(())
    }

    async fn validate_session(&self, session_id: &str) -> Result<Session> {
        // In a real implementation, you would look up the session in a database
        // For now, we'll treat the session_id as a token
        let intro = self.client.introspect_token(session_id).await?;

        if !intro.active {
            return Err(anyhow!("Session is not active"));
        }

        let user_id = intro.sub.ok_or_else(|| anyhow!("No user ID in session"))?;

        Ok(Session {
            id: Uuid::new_v4().to_string(),
            user_id,
            token: session_id.to_string(),
            refresh_token: None,
            expires_at: intro
                .exp
                .map(|exp| {
                    DateTime::<Utc>::from_timestamp(exp as i64, 0)
                        .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1))
                })
                .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1)),
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        })
    }

    async fn grant_permission(&self, subject_id: &str, permission: &str) -> Result<()> {
        self.client.grant_role(subject_id, permission).await?;
        Ok(())
    }

    async fn revoke_permission(&self, subject_id: &str, grant_id: &str) -> Result<()> {
        self.client.revoke_role(subject_id, grant_id).await?;
        Ok(())
    }

    async fn check_permission(
        &self,
        subject_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool> {
        let permission = format!("{}:{}", resource, action);
        self.client.check_permission(subject_id, &permission).await
    }

    async fn list_permissions(&self, subject_id: &str) -> Result<Vec<Permission>> {
        let response = self.client.get_user_grants(subject_id, 0, 100).await?;

        let mut permissions = Vec::new();
        if let Some(result) = response["result"].as_array() {
            for grant in result {
                if let Some(role_keys) = grant["roleKeys"].as_array() {
                    for role_key in role_keys {
                        if let Some(role_str) = role_key.as_str() {
                            let parts: Vec<&str> = role_str.split(':').collect();
                            let resource = parts.get(0).map(|s| s.to_string()).unwrap_or_default();
                            let action = parts.get(1).map(|s| s.to_string()).unwrap_or_default();

                            permissions.push(Permission {
                                id: Uuid::new_v4().to_string(),
                                name: role_str.to_string(),
                                resource,
                                action,
                                description: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(permissions)
    }
}
