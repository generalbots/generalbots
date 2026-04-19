use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbPermissions {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub default_access: AccessLevel,
    #[serde(default)]
    pub folders: HashMap<String, FolderPermission>,
    #[serde(default = "default_true")]
    pub inheritance: bool,
}

fn default_version() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    All,
    #[default]
    Authenticated,
    RoleBased,
    GroupBased,
    UserBased,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderPermission {
    #[serde(default)]
    pub access: AccessLevel,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub users: Vec<String>,
    #[serde(default)]
    pub index_visibility: Option<AccessLevel>,
    #[serde(default)]
    pub inherit_parent: Option<bool>,
}

impl Default for FolderPermission {
    fn default() -> Self {
        Self {
            access: AccessLevel::Authenticated,
            roles: Vec::new(),
            groups: Vec::new(),
            users: Vec::new(),
            index_visibility: None,
            inherit_parent: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub is_authenticated: bool,
    pub organization_id: Option<Uuid>,
}

impl UserContext {
    pub fn anonymous() -> Self {
        Self {
            user_id: Uuid::nil(),
            email: None,
            roles: Vec::new(),
            groups: Vec::new(),
            is_authenticated: false,
            organization_id: None,
        }
    }

    pub fn authenticated(user_id: Uuid, email: Option<String>, org_id: Option<Uuid>) -> Self {
        Self {
            user_id,
            email,
            roles: Vec::new(),
            groups: Vec::new(),
            is_authenticated: true,
            organization_id: org_id,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = groups;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: String,
    pub matched_rule: Option<String>,
    pub index_visible: bool,
}

pub struct KbPermissionParser {
    permissions: KbPermissions,
    folder_cache: HashMap<String, ResolvedPermission>,
}

#[derive(Debug, Clone)]
struct ResolvedPermission {
    access: AccessLevel,
    roles: Vec<String>,
    groups: Vec<String>,
    users: Vec<String>,
    index_visibility: AccessLevel,
}

impl KbPermissionParser {
    pub fn new(permissions: KbPermissions) -> Self {
        Self {
            permissions,
            folder_cache: HashMap::new(),
        }
    }

    pub fn from_yaml(yaml_content: &str) -> Result<Self, KbPermissionError> {
        let permissions: KbPermissions = serde_json::from_str(yaml_content)
            .map_err(|e| KbPermissionError::ParseError(e.to_string()))?;
        Ok(Self::new(permissions))
    }

    pub async fn from_file(path: &Path) -> Result<Self, KbPermissionError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| KbPermissionError::IoError(e.to_string()))?;
        Self::from_yaml(&content)
    }

    pub fn check_access(&mut self, folder_path: &str, user: &UserContext) -> PermissionCheckResult {
        let resolved = self.resolve_permission(folder_path);

        let (allowed, reason, matched_rule) = match resolved.access {
            AccessLevel::All => (true, "Public access".to_string(), Some("access: all".to_string())),
            AccessLevel::None => (false, "Access denied".to_string(), Some("access: none".to_string())),
            AccessLevel::Authenticated => {
                if user.is_authenticated {
                    (true, "Authenticated user".to_string(), Some("access: authenticated".to_string()))
                } else {
                    (false, "Authentication required".to_string(), Some("access: authenticated".to_string()))
                }
            }
            AccessLevel::RoleBased => {
                if !user.is_authenticated {
                    (false, "Authentication required".to_string(), Some("access: role_based".to_string()))
                } else if resolved.roles.is_empty() {
                    (true, "No roles required".to_string(), Some("access: role_based (empty)".to_string()))
                } else {
                    let has_role = user.roles.iter().any(|r| resolved.roles.contains(r));
                    if has_role {
                        let matched = user.roles.iter().find(|r| resolved.roles.contains(r));
                        (true, format!("Role matched: {:?}", matched), Some(format!("roles: {:?}", resolved.roles)))
                    } else {
                        (false, format!("Required roles: {:?}", resolved.roles), Some(format!("roles: {:?}", resolved.roles)))
                    }
                }
            }
            AccessLevel::GroupBased => {
                if !user.is_authenticated {
                    (false, "Authentication required".to_string(), Some("access: group_based".to_string()))
                } else if resolved.groups.is_empty() {
                    (true, "No groups required".to_string(), Some("access: group_based (empty)".to_string()))
                } else {
                    let has_group = user.groups.iter().any(|g| resolved.groups.contains(g));
                    if has_group {
                        let matched = user.groups.iter().find(|g| resolved.groups.contains(g));
                        (true, format!("Group matched: {:?}", matched), Some(format!("groups: {:?}", resolved.groups)))
                    } else {
                        (false, format!("Required groups: {:?}", resolved.groups), Some(format!("groups: {:?}", resolved.groups)))
                    }
                }
            }
            AccessLevel::UserBased => {
                if !user.is_authenticated {
                    (false, "Authentication required".to_string(), Some("access: user_based".to_string()))
                } else if resolved.users.is_empty() {
                    (false, "No users allowed".to_string(), Some("access: user_based (empty)".to_string()))
                } else {
                    let user_id_str = user.user_id.to_string();
                    let email_match = user.email.as_ref().map(|e| resolved.users.contains(e)).unwrap_or(false);
                    let id_match = resolved.users.contains(&user_id_str);

                    if email_match || id_match {
                        (true, "User authorized".to_string(), Some(format!("users: {:?}", resolved.users)))
                    } else {
                        (false, "User not in allowed list".to_string(), Some(format!("users: {:?}", resolved.users)))
                    }
                }
            }
        };

        let index_visible = self.check_index_visibility(&resolved.index_visibility, user);

        PermissionCheckResult {
            allowed,
            reason,
            matched_rule,
            index_visible,
        }
    }

    fn check_index_visibility(&self, visibility: &AccessLevel, user: &UserContext) -> bool {
        match visibility {
            AccessLevel::All => true,
            AccessLevel::None => false,
            AccessLevel::Authenticated => user.is_authenticated,
            _ => user.is_authenticated,
        }
    }

    fn resolve_permission(&mut self, folder_path: &str) -> ResolvedPermission {
        if let Some(cached) = self.folder_cache.get(folder_path) {
            return cached.clone();
        }

        let normalized_path = folder_path.trim_matches('/').to_lowercase();

        if let Some(folder_perm) = self.permissions.folders.get(&normalized_path) {
            let resolved = ResolvedPermission {
                access: folder_perm.access.clone(),
                roles: folder_perm.roles.clone(),
                groups: folder_perm.groups.clone(),
                users: folder_perm.users.clone(),
                index_visibility: folder_perm.index_visibility.clone().unwrap_or(folder_perm.access.clone()),
            };
            self.folder_cache.insert(folder_path.to_string(), resolved.clone());
            return resolved;
        }

        if self.permissions.inheritance {
            let parts: Vec<&str> = normalized_path.split('/').collect();
            for i in (0..parts.len()).rev() {
                let parent_path = parts[..i].join("/");
                if let Some(parent_perm) = self.permissions.folders.get(&parent_path) {
                    if parent_perm.inherit_parent.unwrap_or(true) {
                        let resolved = ResolvedPermission {
                            access: parent_perm.access.clone(),
                            roles: parent_perm.roles.clone(),
                            groups: parent_perm.groups.clone(),
                            users: parent_perm.users.clone(),
                            index_visibility: parent_perm.index_visibility.clone().unwrap_or(parent_perm.access.clone()),
                        };
                        self.folder_cache.insert(folder_path.to_string(), resolved.clone());
                        return resolved;
                    }
                }
            }
        }

        let default = ResolvedPermission {
            access: self.permissions.default_access.clone(),
            roles: Vec::new(),
            groups: Vec::new(),
            users: Vec::new(),
            index_visibility: self.permissions.default_access.clone(),
        };
        self.folder_cache.insert(folder_path.to_string(), default.clone());
        default
    }

    pub fn get_qdrant_filter_metadata(&mut self, folder_path: &str) -> QdrantPermissionMetadata {
        let resolved = self.resolve_permission(folder_path);

        QdrantPermissionMetadata {
            access_level: format!("{:?}", resolved.access).to_lowercase(),
            allowed_roles: resolved.roles,
            allowed_groups: resolved.groups,
            allowed_users: resolved.users,
            is_public: matches!(resolved.access, AccessLevel::All),
            requires_auth: !matches!(resolved.access, AccessLevel::All | AccessLevel::None),
        }
    }

    pub fn clear_cache(&mut self) {
        self.folder_cache.clear();
    }

    pub fn permissions(&self) -> &KbPermissions {
        &self.permissions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPermissionMetadata {
    pub access_level: String,
    pub allowed_roles: Vec<String>,
    pub allowed_groups: Vec<String>,
    pub allowed_users: Vec<String>,
    pub is_public: bool,
    pub requires_auth: bool,
}

impl QdrantPermissionMetadata {
    pub fn to_payload(&self) -> HashMap<String, serde_json::Value> {
        let mut payload = HashMap::new();
        payload.insert("access_level".to_string(), serde_json::json!(self.access_level));
        payload.insert("allowed_roles".to_string(), serde_json::json!(self.allowed_roles));
        payload.insert("allowed_groups".to_string(), serde_json::json!(self.allowed_groups));
        payload.insert("allowed_users".to_string(), serde_json::json!(self.allowed_users));
        payload.insert("is_public".to_string(), serde_json::json!(self.is_public));
        payload.insert("requires_auth".to_string(), serde_json::json!(self.requires_auth));
        payload
    }

    pub fn public() -> Self {
        Self {
            access_level: "all".to_string(),
            allowed_roles: Vec::new(),
            allowed_groups: Vec::new(),
            allowed_users: Vec::new(),
            is_public: true,
            requires_auth: false,
        }
    }

    pub fn authenticated_only() -> Self {
        Self {
            access_level: "authenticated".to_string(),
            allowed_roles: Vec::new(),
            allowed_groups: Vec::new(),
            allowed_users: Vec::new(),
            is_public: false,
            requires_auth: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum KbPermissionError {
    ParseError(String),
    IoError(String),
    InvalidPath(String),
}

impl std::fmt::Display for KbPermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "Permission parse error: {e}"),
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::InvalidPath(e) => write!(f, "Invalid path: {e}"),
        }
    }
}

impl std::error::Error for KbPermissionError {}

impl Default for KbPermissions {
    fn default() -> Self {
        Self {
            version: 1,
            default_access: AccessLevel::Authenticated,
            folders: HashMap::new(),
            inheritance: true,
        }
    }
}

pub fn create_default_permissions() -> KbPermissions {
    let mut folders = HashMap::new();

    folders.insert("public".to_string(), FolderPermission {
        access: AccessLevel::All,
        roles: Vec::new(),
        groups: Vec::new(),
        users: Vec::new(),
        index_visibility: Some(AccessLevel::All),
        inherit_parent: Some(true),
    });

    folders.insert("internal".to_string(), FolderPermission {
        access: AccessLevel::Authenticated,
        roles: Vec::new(),
        groups: Vec::new(),
        users: Vec::new(),
        index_visibility: Some(AccessLevel::Authenticated),
        inherit_parent: Some(true),
    });

    KbPermissions {
        version: 1,
        default_access: AccessLevel::Authenticated,
        folders,
        inheritance: true,
    }
}

pub fn generate_permissions_yaml(permissions: &KbPermissions) -> Result<String, KbPermissionError> {
    serde_json::to_string_pretty(permissions)
        .map_err(|e| KbPermissionError::ParseError(e.to_string()))
}

pub fn build_qdrant_permission_filter(user: &UserContext) -> serde_json::Value {
    if !user.is_authenticated {
        return serde_json::json!({
            "must": [
                { "key": "is_public", "match": { "value": true } }
            ]
        });
    }

    let user_id = user.user_id.to_string();
    let email = user.email.clone().unwrap_or_default();

    let mut should_conditions = vec![
        serde_json::json!({ "key": "is_public", "match": { "value": true } }),
        serde_json::json!({ "key": "access_level", "match": { "value": "authenticated" } }),
    ];

    if !user.roles.is_empty() {
        for role in &user.roles {
            should_conditions.push(serde_json::json!({
                "key": "allowed_roles",
                "match": { "any": [role] }
            }));
        }
    }

    if !user.groups.is_empty() {
        for group in &user.groups {
            should_conditions.push(serde_json::json!({
                "key": "allowed_groups",
                "match": { "any": [group] }
            }));
        }
    }

    should_conditions.push(serde_json::json!({
        "key": "allowed_users",
        "match": { "any": [user_id, email] }
    }));

    serde_json::json!({
        "should": should_conditions,
        "min_should": { "conditions": should_conditions.clone(), "min_count": 1 }
    })
}
