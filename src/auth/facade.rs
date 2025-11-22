use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use reqwest::Client;
use crate::auth::zitadel::ZitadelClient;

/// User representation in the system
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

/// Group representation in the system
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

/// Permission representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

/// Session information
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

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user: User,
    pub session: Session,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}

/// User creation request
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

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Group creation request
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
    async fn update_group(&self, group_id: &str, name: Option<String>, description: Option<String>) -> Result<Group>;
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
    async fn check_permission(&self, subject_id: &str, resource: &str, action: &str) -> Result<bool>;
    async fn list_permissions(&self, subject_id: &str) -> Result<Vec<Permission>>;
}

/// Zitadel-based authentication facade implementation
pub struct ZitadelAuthFacade {
    client: ZitadelClient,
    cache: Option<redis::Client>,
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
    pub fn with_cache(client: ZitadelClient, redis_url: &str) -> Result<Self> {
        let cache = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            cache: Some(cache),
        })
    }

    /// Convert Zitadel user to internal user representation
    fn map_zitadel_user(&self, zitadel_user: serde_json::Value) -> Result<User> {
        Ok(User {
            id: zitadel_user["id"].as_str().unwrap_or_default().to_string(),
            email: zitadel_user["email"].as_str().unwrap_or_default().to_string(),
            username: zitadel_user["userName"].as_str().map(String::from),
            first_name: zitadel_user["profile"]["firstName"].as_str().map(String::from),
            last_name: zitadel_user["profile"]["lastName"].as_str().map(String::from),
            display_name: zitadel_user["profile"]["displayName"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            avatar_url: zitadel_user["profile"]["avatarUrl"].as_str().map(String::from),
            groups: vec![],  // Will be populated separately
            roles: vec![],   // Will be populated separately
            metadata: HashMap::new(),
            created_at: Utc::now(),  // Parse from Zitadel response
            updated_at: Utc::now(),  // Parse from Zitadel response
            last_login: None,
            is_active: zitadel_user["state"].as_str() == Some("STATE_ACTIVE"),
            is_verified: zitadel_user["emailVerified"].as_bool().unwrap_or(false),
        })
    }

    /// Get or create cache connection
    async fn get_cache_conn(&self) -> Option<redis::aio::Connection> {
        if let Some(cache) = &self.cache {
            cache.get_async_connection().await.ok()
        } else {
            None
        }
    }

    /// Cache user data
    async fn cache_user(&self, user: &User) -> Result<()> {
        if let Some(mut conn) = self.get_cache_conn().await {
            use redis::AsyncCommands;
            let key = format!("user:{}", user.id);
            let value = serde_json::to_string(user)?;
            let _: () = conn.setex(key, value, 300).await?;  // 5 minute cache
        }
        Ok(())
    }

    /// Get cached user
    async fn get_cached_user(&self, user_id: &str) -> Option<User> {
        if let Some(mut conn) = self.get_cache_conn().await {
            use redis::AsyncCommands;
            let key = format!("user:{}", user_id);
            if let Ok(value) = conn.get::<_, String>(key).await {
                serde_json::from_str(&value).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl AuthFacade for ZitadelAuthFacade {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        // Create user in Zitadel
        let zitadel_response = self.client.create_user(
            &request.email,
            request.password.as_deref(),
            request.first_name.as_deref(),
            request.last_name.as_deref(),
        ).await?;

        let mut user = self.map_zitadel_user(zitadel_response)?;

        // Add to groups if specified
        for group_id in &request.groups {
            self.add_user_to_group(&user.id, group_id).await?;
        }
        user.groups = request.groups;

        // Assign roles if specified
        for role in &request.roles {
            self.client.grant_role(&user.id, role).await?;
        }
        user.roles = request.roles;

        // Cache the user
        self.cache_user(&user).await?;

        Ok(user)
    }

    async fn get_user(&self, user_id: &str) -> Result<User> {
        // Check cache first
        if let Some(cached_user) = self.get_cached_user(user_id).await {
            return Ok(cached_user);
        }

        // Fetch from Zitadel
        let zitadel_response = self.client.get_user(user_id).await?;
        let mut user = self.map_zitadel_user(zitadel_response)?;

        // Get user's groups
        user.groups = self.client.get_user_memberships(user_id).await?;

        // Get user's roles
        user.roles = self.client.get_user_grants(user_id).await?;

        // Cache the user
        self.cache_user(&user).await?;

        Ok(user)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let users = self.client.search_users(email).await?;
        if users.is_empty() {
            return Err(anyhow!("User not found"));
        }

        let user_id = users[0]["id"].as_str().ok_or_else(|| anyhow!("Invalid user data"))?;
        self.get_user(user_id).await
    }

    async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> Result<User> {
        // Update in Zitadel
        self.client.update_user_profile(
            user_id,
            request.first_name.as_deref(),
            request.last_name.as_deref(),
            request.display_name.as_deref(),
        ).await?;

        // Invalidate cache
        if let Some(mut conn) = self.get_cache_conn().await {
            use redis::AsyncCommands;
            let key = format!("user:{}", user_id);
            let _: () = conn.del(key).await?;
        }

        // Return updated user
        self.get_user(user_id).await
    }

    async fn delete_user(&self, user_id: &str) -> Result<()> {
        // Delete from Zitadel
        self.client.deactivate_user(user_id).await?;

        // Invalidate cache
        if let Some(mut conn) = self.get_cache_conn().await {
            use redis::AsyncCommands;
            let key = format!("user:{}", user_id);
            let _: () = conn.del(key).await?;
        }

        Ok(())
    }

    async fn list_users(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<User>> {
        let zitadel_users = self.client.list_users(limit, offset).await?;
        let mut users = Vec::new();

        for zitadel_user in zitadel_users {
            if let Ok(user) = self.map_zitadel_user(zitadel_user) {
                users.push(user);
            }
        }

        Ok(users)
    }

    async fn search_users(&self, query: &str) -> Result<Vec<User>> {
        let zitadel_users = self.client.search_users(query).await?;
        let mut users = Vec::new();

        for zitadel_user in zitadel_users {
            if let Ok(user) = self.map_zitadel_user(zitadel_user) {
                users.push(user);
            }
        }

        Ok(users)
    }

    async fn create_group(&self, request: CreateGroupRequest) -> Result<Group> {
        // Note: Zitadel uses organizations/projects for grouping
        // This is a simplified mapping
        let org_id = self.client.create_organization(&request.name, request.description.as_deref()).await?;

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
        // Fetch organization details from Zitadel
        let org = self.client.get_organization(group_id).await?;

        Ok(Group {
            id: group_id.to_string(),
            name: org["name"].as_str().unwrap_or_default().to_string(),
            description: org["description"].as_str().map(String::from),
            parent_id: None,
            members: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn update_group(&self, group_id: &str, name: Option<String>, description: Option<String>) -> Result<Group> {
        if let Some(name) = &name {
            self.client.update_organization(group_id, name, description.as_deref()).await?;
        }

        self.get_group(group_id).await
    }

    async fn delete_group(&self, group_id: &str) -> Result<()> {
        self.client.deactivate_organization(group_id).await
    }

    async fn list_groups(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Group>> {
        let orgs = self.client.list_organizations(limit, offset).await?;
        let mut groups = Vec::new();

        for org in orgs {
            groups.push(Group {
                id: org["id"].as_str().unwrap_or_default().to_string(),
                name: org["name"].as_str().unwrap_or_default().to_string(),
                description: org["description"].as_str().map(String::from),
                parent_id: None,
                members: vec![],
                permissions: vec![],
                metadata: HashMap::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(groups)
    }

    async fn add_user_to_group(&self, user_id: &str, group_id: &str) -> Result<()> {
        self.client.add_org_member(group_id, user_id).await
    }

    async fn remove_user_from_group(&self, user_id: &str, group_id: &str) -> Result<()> {
        self.client.remove_org_member(group_id, user_id).await
    }

    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<Group>> {
        let memberships = self.client.get_user_memberships(user_id).await?;
        let mut groups = Vec::new();

        for membership_id in memberships {
            if let Ok(group) = self.get_group(&membership_id).await {
                groups.push(group);
            }
        }

        Ok(groups)
    }

    async fn get_group_members(&self, group_id: &str) -> Result<Vec<User>> {
        let member_ids = self.client.get_org_members(group_id).await?;
        let mut members = Vec::new();

        for member_id in member_ids {
            if let Ok(user) = self.get_user(&member_id).await {
                members.push(user);
            }
        }

        Ok(members)
    }

    async fn authenticate(&self, email: &str, password: &str) -> Result<AuthResult> {
        // Authenticate with Zitadel
        let token_response = self.client.authenticate(email, password).await?;

        // Get user details
        let user = self.get_user_by_email(email).await?;

        // Create session
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            token: token_response["access_token"].as_str().unwrap_or_default().to_string(),
            refresh_token: token_response["refresh_token"].as_str().map(String::from),
            expires_at: Utc::now() + chrono::Duration::seconds(
                token_response["expires_in"].as_i64().unwrap_or(3600)
            ),
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        };

        // Cache session
        if let Some(mut conn) = self.get_cache_conn().await {
            use redis::AsyncCommands;
            let key = format!("session:{}", session.id);
            let value = serde_json::to_string(&session)?;
            let _: () = conn.setex(key, value, 3600).await?;  // 1 hour cache
        }

        Ok(AuthResult {
            user,
            session: session.clone(),
            access_token: session.token,
            refresh_token: session.refresh_token,
            expires_in: token_response["expires_in"].as_i64().unwrap_or(3600),
        })
    }

    async fn authenticate_with_token(&self, token: &str) -> Result<AuthResult> {
        // Validate token with Zitadel
        let introspection = self.client.introspect_token(token).await?;

        if !introspection["active"].as_bool().unwrap_or(false) {
            return Err(anyhow!("Invalid or expired token"));
        }

        let user_id = introspection["sub"].as_str()
            .ok_or_else(|| anyhow!("No subject in token"))?;

        let user = self.get_user(user_id).await?;

        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            token: token.to_string(),
            refresh_token: None,
            expires_at: Utc::now() + chrono::Duration::seconds(
                introspection["exp"].as_i64().unwrap_or(3600)
            ),
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        };

        Ok(AuthResult {
            user,
            session: session.clone(),
            access_token: session.token,
            refresh_token: None,
            expires_in: introspection["exp"].as_i64().unwrap_or(3600),
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResult> {
        let token_response = self.client.refresh_token(refresh_token).await?;

        // Get user from the new token
        let new_token = token_response["access_token"].as_str()
            .ok_or_else(|| anyhow!("No access token in response"))?;

        self.authenticate_with_token(new_token).await
    }

    async fn logout(&self, session_id: &str) -> Result<()> {
        // Invalidate session in cache
        if let Some(mut conn) = self.get_cache_conn().await {
            use redis::AsyncCommands;
            let key = format!("session:{}", session_id);
            let _: () = conn.del(key).await?;
        }

        // Note: Zitadel token revocation would be called here if available

        Ok(())
    }

    async fn validate_session(&self, session_id: &str) -> Result<Session> {
        // Check cache first
        if let Some(mut conn) = self.get_cache_conn().await {
            use redis::AsyncCommands;
            let key = format!("session:{}", session_id);
            if let Ok(value) = conn.get::<_, String>(key).await {
                if let Ok(session) = serde_json::from_str::<Session>(&value) {
                    if session.expires_at > Utc::now() {
                        return Ok(session);
                    }
                }
            }
        }

        Err(anyhow!("Invalid or expired session"))
    }

    async fn grant_permission(&self, subject_id: &str, permission: &str) -> Result<()> {
        self.client.grant_role(subject_id, permission).await
    }

    async fn revoke_permission(&self, subject_id: &str, permission: &str) -> Result<()> {
        self.client.revoke_role(subject_id, permission).await
    }

    async fn check_permission(&self, subject_id: &str, resource: &str, action: &str) -> Result<bool> {
        // Check with Zitadel's permission system
        let permission_string = format!("{}:{}", resource, action);
        self.client.check_permission(subject_id, &permission_string).await
    }

    async fn list_permissions(&self, subject_id: &str) -> Result<Vec<Permission>> {
        let grants = self.client.get_user_grants(subject_id).await?;
        let mut permissions = Vec::new();

        for grant in grants {
            // Parse grant string into permission
            if let Some((resource, action)) = grant.split_once(':') {
                permissions.push(Permission {
                    id: Uuid::new_v4().to_string(),
                    name: grant.clone(),
                    resource: resource.to_string(),
                    action: action.to_string(),
                    description: None,
                });
            }
        }

        Ok(permissions)
    }
}

/// Simple in-memory auth facade for testing and SMB deployments
pub struct SimpleAuthFacade {
    users: std::sync::Arc<tokio::sync::RwLock<HashMap<String, User>>>,
    groups: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Group>>>,
    sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Session>>>,
}

impl SimpleAuthFacade {
    pub fn new() -> Self {
        Self {
            users: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            groups: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AuthFacade for SimpleAuthFacade {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        let user = User {
            id: Uuid::new_v4().to_string(),
            email: request.email.clone(),
            username: request.username,
            first_name: request.first_name,
            last_name: request.last_name,
            display_name: request.email.clone(),
            avatar_url: None,
            groups: request.groups,
            roles: request.roles,
            metadata: request.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            is_active: true,
            is_verified: false,
        };

        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user.clone());

        Ok(user)
    }

    async fn get_user(&self, user_id: &str) -> Result<User> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
            .ok_or_else(|| anyhow!("User not found"))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let users = self.users.read().await;
        users.values()
            .find(|u| u.email == email)
            .cloned()
            .ok_or_else(|| anyhow!("User not found"))
    }

    async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> Result<User> {
        let mut users = self.users.write().await;
        let user = users.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;

        if let Some(first_name) = request.first_name {
            user.first_name = Some(first_name);
        }
        if let Some(last_name) = request.last_name {
            user.last_name = Some(last_name);
        }
        if let Some(display_name) = request.display_name {
            user.display_name = display_name;
        }
        if let Some(avatar_url) = request.avatar_url {
            user.avatar_url = Some(avatar_url);
        }
        user.updated_at = Utc::now();

        Ok(user.clone())
    }

    async fn delete_user(&self, user_id: &str) -> Result<()> {
        let mut users = self.users.write().await;
        users.remove(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        Ok(())
    }

    async fn list_users(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<User>> {
        let users = self.users.read().await;
        let mut all_users: Vec<User> = users.values().cloned().collect();
        all_users.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);

        Ok(all_users.into_iter().skip(offset).take(limit).collect())
    }

    async fn search_users(&self, query: &str) -> Result<Vec<User>> {
        let users = self.users.read().await;
        let query_lower = query.to_lowercase();

        Ok(users.values()
            .filter(|u| {
                u.email.to_lowercase().contains(&query_lower) ||
                u.display_name.to_lowercase().contains(&query_lower) ||
                u.username.as_ref().map(|un| un.to_lowercase().contains(&query_lower)).unwrap_or(false)
            })
            .cloned()
            .collect())
    }

    async fn create_group(&self, request: CreateGroupRequest) -> Result<Group> {
        let group = Group {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            description: request.description,
            parent_id: request.parent_id,
            members: vec![],
            permissions: request.permissions,
            metadata: request.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut groups = self.groups.write().await;
        groups.insert(group.id.clone(), group.clone());

        Ok(group)
    }

    async fn get_group(&self, group_id: &str) -> Result<Group> {
        let groups = self.groups.read().await;
        groups.get(group_id).cloned()
            .ok_or_else(|| anyhow!("Group not found"))
    }

    async fn update_group(&self, group_id: &str, name: Option<String>, description: Option<String>) -> Result<Group> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| anyhow!("Group not found"))?;

        if let Some(name) = name {
            group.name = name;
        }
        if let Some(description) = description {
            group.description = Some(description);
        }
        group.updated_at = Utc::now();

        Ok(group.clone())
    }

    async fn delete_group(&self, group_id: &str) -> Result<()> {
        let mut groups = self.groups.write().await;
        groups.remove(group_id)
            .ok_or_else(|| anyhow!("Group not found"))?;
        Ok(())
    }

    async fn list_groups(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Group>> {
        let groups = self.groups.read().await;
        let mut all_groups: Vec<Group> = groups.values().cloned().collect();
        all_groups.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);

        Ok(all_groups.into_iter().skip(offset).take(limit).collect())
    }

    async fn add_user_to_group(&self, user_id: &str, group_id: &str) -> Result<()> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| anyhow!("Group not found"))?;

        if !group.members.contains(&user_id.to_string()) {
            group.members.push(user_id.to_string());
        }

        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            if !user.groups.contains(&group_id.to_string()) {
                user.groups.push(group_id.to_string());
            }
        }

        Ok(())
    }

    async fn remove_user_from_group(&self, user_id: &str, group_id: &str) -> Result<()> {
        let mut groups = self.groups.write().await;
        if let Some(group) = groups.get_mut(group_id) {
            group.members.retain(|id| id != user_id);
        }

        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.groups.retain(|id| id != group_id);
        }

        Ok(())
    }

    async fn get_user_groups(&self, user_id: &str) -> Result<Vec<Group>> {
        let users = self.users.read().await;
        let user = users.get(user_id)
            .ok_or_else(|| anyhow!("User not found"))?;

        let groups = self.groups.read().await;
        Ok(user.groups.iter()
            .filter_map(|group_id| groups.get(group_id).cloned())
            .collect())
    }

    async fn get_group_members(&self, group_id: &str) -> Result<Vec<User>> {
        let groups = self.groups.read().await;
        let group = groups.get(group_id)
            .ok_or_else(|| anyhow!("Group not found"))?;

        let users = self.users.read().await;
        Ok(group.members.iter()
            .filter_map(|user_id| users.get(user_id).cloned())
            .collect())
    }

    async fn authenticate(&self, email: &str, password: &str) -> Result<AuthResult> {
        // Simple authentication - in production, verify password hash
        let user = self.get_user_by_email(email).await?;

        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            token: Uuid::new_v4().to_string(),
            refresh_token: Some(Uuid::new_v4().to_string()),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());

        Ok(AuthResult {
            user,
            session: session.clone(),
            access_token: session.token,
            refresh_token: session.refresh_token,
            expires_in: 3600,
        })
    }

    async fn authenticate_with_token(&self, token: &str) -> Result<AuthResult> {
        let sessions = self.sessions.read().await;
        let session = sessions.values()
            .find(|s| s.token == token)
            .ok_or_else(|| anyhow!("Invalid token"))?;

        if session.expires_at < Utc::now() {
            return Err(anyhow!("Token expired"));
        }

        let user = self.get_user(&session.user_id).await?;

        Ok(AuthResult {
            user,
            session: session.clone(),
            access_token: session.token.clone(),
            refresh_token: session.refresh_token.clone(),
            expires_in: (session.expires_at - Utc::now()).num_seconds(),
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResult> {
        let sessions = self.sessions.read().await;
        let old_session = sessions.values()
            .find(|s| s.refresh_token.as_ref() == Some(&refresh_token.to_string()))
            .ok_or_else(|| anyhow!("Invalid refresh token"))?;

        let user = self.get_user(&old_session.user_id).await?;

        let new_session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            token: Uuid::new_v4().to_string(),
            refresh_token: Some(Uuid::new_v4().to_string()),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: Utc::now(),
            ip_address: None,
            user_agent: None,
        };

        drop(sessions);
        let mut sessions = self.sessions.write().await;
        sessions.insert(new_session.id.clone(), new_session.clone());

        Ok(AuthResult {
            user,
            session: new_session.clone(),
            access_token: new_session.token,
            refresh_token: new_session.refresh_token,
            expires_in: 3600,
        })
    }

    async fn logout(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        Ok(())
    }

    async fn validate_session(&self, session_id: &str) -> Result<Session> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;

        if session.expires_at < Utc::now() {
            return Err(anyhow!("Session expired"));
        }

        Ok(session.clone())
    }

    async fn grant_permission(&self, subject_id: &str, permission: &str) -> Result<()> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(subject_id) {
            if !user.roles.contains(&permission.to_string()) {
                user.roles.push(permission.to_string());
            }
            return Ok(());
        }

        let mut groups = self.groups.write().await;
        if let Some(group) = groups.get_mut(subject_id) {
            if !group.permissions.contains(&permission.to_string()) {
                group.permissions.push(permission.to_string());
            }
            return Ok(());
        }

        Err(anyhow!("Subject not found"))
    }

    async fn revoke_permission(&self, subject_id: &str, permission: &str) -> Result<()> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(subject_id) {
            user.roles.retain(|r| r != permission);
            return Ok(());
        }

        let mut groups = self.groups.write().await;
        if let Some(group) = groups.get_mut(subject_id) {
            group.permissions.retain(|p| p != permission);
            return Ok(());
        }

        Err(anyhow!("Subject not found"))
    }

    async fn check_permission(&self, subject_id: &str, resource: &str, action: &str) -> Result<bool> {
        let permission = format!("{}:{}", resource, action);

        // Check user permissions
        let users = self.users.read().await;
        if let Some(user) = users.get(subject_id) {
            if user.roles.contains(&permission) || user.roles.contains(&"admin".to_string()) {
                return Ok(true);
            }

            // Check group permissions
            let groups = self.groups.read().await;
            for group_id in &user.groups {
                if let Some(group) = groups.get(group_id) {
                    if group.permissions.contains(&permission) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn list_permissions(&self, subject_id: &str) -> Result<Vec<Permission>> {
        let mut permissions = Vec::new();

        let users = self.users.read().await;
        if let Some(user) = users.get(subject_id) {
            for role in &user.roles {
                if let Some((resource, action)) = role.split_once(':') {
                    permissions.push(Permission {
                        id: Uuid::new_v4().to_string(),
                        name: role.clone(),
                        resource: resource.to_string(),
                        action: action.to_string(),
                        description: None,
                    });
                }
            }
        }

        Ok(permissions)
    }
}
