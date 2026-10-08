use super::*;

/// Match a wildcard pattern (e.g. `bot:*` matches `bot:create:messages`)
/// against a value string. Supports `*` and `**` as wildcards.
/// Splits on both `:` and `.` delimiters.
pub fn match_wildcard(pattern: &str, value: &str) -> bool {
    let p_lower = pattern.to_lowercase();
    let v_lower = value.to_lowercase();

    if p_lower == "*" || p_lower == "**" {
        return true;
    }

    let pattern_parts: Vec<&str> = p_lower.split([':', '.']).collect();
    let value_parts: Vec<&str> = v_lower.split([':', '.']).collect();

    for (i, part) in pattern_parts.iter().enumerate() {
        if *part == "*" || *part == "**" {
            return true;
        }
        if i >= value_parts.len() || *part != value_parts[i] {
            return false;
        }
    }

    pattern_parts.len() == value_parts.len()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    pub cache_ttl_seconds: u64,
    pub enable_permission_cache: bool,
    pub enable_group_inheritance: bool,
    pub default_deny: bool,
    pub audit_all_decisions: bool,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 300,
            enable_permission_cache: true,
            enable_group_inheritance: true,
            default_deny: true,
            audit_all_decisions: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub resource_type: String,
    pub resource_id: String,
    pub permission: String,
}

impl ResourcePermission {
    pub fn new(resource_type: &str, resource_id: &str, permission: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            permission: permission.to_string(),
        }
    }

    pub fn read(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "read")
    }

    pub fn write(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "write")
    }

    pub fn delete(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "delete")
    }

    pub fn admin(resource_type: &str, resource_id: &str) -> Self {
        Self::new(resource_type, resource_id, "admin")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    Allow,
    Deny,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecisionResult {
    pub decision: AccessDecision,
    pub reason: String,
    pub evaluated_at: DateTime<Utc>,
    pub cache_hit: bool,
    pub matched_rule: Option<String>,
}

impl AccessDecisionResult {
    pub fn allow(reason: &str) -> Self {
        Self {
            decision: AccessDecision::Allow,
            reason: reason.to_string(),
            evaluated_at: Utc::now(),
            cache_hit: false,
            matched_rule: None,
        }
    }

    pub fn deny(reason: &str) -> Self {
        Self {
            decision: AccessDecision::Deny,
            reason: reason.to_string(),
            evaluated_at: Utc::now(),
            cache_hit: false,
            matched_rule: None,
        }
    }

    pub fn with_cache_hit(mut self) -> Self {
        self.cache_hit = true;
        self
    }

    pub fn with_rule(mut self, rule: String) -> Self {
        self.matched_rule = Some(rule);
        self
    }

    pub fn is_allowed(&self) -> bool {
        self.decision == AccessDecision::Allow
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePermission {
    pub path_pattern: String,
    pub method: String,
    pub required_permission: String,
    pub required_roles: Vec<String>,
    pub allow_anonymous: bool,
    pub description: Option<String>,
}

impl RoutePermission {
    pub fn new(path_pattern: &str, method: &str, permission: &str) -> Self {
        Self {
            path_pattern: path_pattern.to_string(),
            method: method.to_string(),
            required_permission: permission.to_string(),
            required_roles: Vec::new(),
            allow_anonymous: false,
            description: None,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.required_roles = roles;
        self
    }

    pub fn with_anonymous(mut self, allow: bool) -> Self {
        self.allow_anonymous = allow;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn matches_path(&self, path: &str) -> bool {
        if self.path_pattern.contains('*') {
            let pattern_parts: Vec<&str> = self.path_pattern.split('/').collect();
            let path_parts: Vec<&str> = path.split('/').collect();

            if pattern_parts.len() > path_parts.len() && !self.path_pattern.ends_with("*") {
                return false;
            }

            for (i, pattern_part) in pattern_parts.iter().enumerate() {
                if *pattern_part == "*" || *pattern_part == "**" {
                    if *pattern_part == "**" {
                        return true;
                    }
                    continue;
                }

                if pattern_part.starts_with(':')
                    || (pattern_part.starts_with('{') && pattern_part.ends_with('}'))
                {
                    continue;
                }

                if i >= path_parts.len() || *pattern_part != path_parts[i] {
                    return false;
                }
            }

            pattern_parts.len() <= path_parts.len() || self.path_pattern.contains("**")
        } else if self.path_pattern.contains(':') || (self.path_pattern.contains('{') && self.path_pattern.contains('}')) {
            let pattern_parts: Vec<&str> = self.path_pattern.split('/').collect();
            let path_parts: Vec<&str> = path.split('/').collect();

            if pattern_parts.len() != path_parts.len() {
                return false;
            }

            for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
                if pattern_part.starts_with(':')
                    || (pattern_part.starts_with('{') && pattern_part.ends_with('}'))
                {
                    continue;
                }
                if *pattern_part != *path_part {
                    return false;
                }
            }

            true
        } else {
            self.path_pattern == path
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAcl {
    pub resource_type: String,
    pub resource_id: String,
    pub owner_id: Option<Uuid>,
    pub permissions: HashMap<Uuid, HashSet<String>>,
    pub group_permissions: HashMap<String, HashSet<String>>,
    pub public_permissions: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ResourceAcl {
    pub fn new(resource_type: &str, resource_id: &str) -> Self {
        let now = Utc::now();
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            owner_id: None,
            permissions: HashMap::new(),
            group_permissions: HashMap::new(),
            public_permissions: HashSet::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_owner(mut self, owner_id: Uuid) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    pub fn grant_user(&mut self, user_id: Uuid, permission: &str) {
        self.permissions
            .entry(user_id)
            .or_default()
            .insert(permission.to_string());
        self.updated_at = Utc::now();
    }

    pub fn revoke_user(&mut self, user_id: Uuid, permission: &str) {
        if let Some(perms) = self.permissions.get_mut(&user_id) {
            perms.remove(permission);
            if perms.is_empty() {
                self.permissions.remove(&user_id);
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn grant_group(&mut self, group_name: &str, permission: &str) {
        self.group_permissions
            .entry(group_name.to_string())
            .or_default()
            .insert(permission.to_string());
        self.updated_at = Utc::now();
    }

    pub fn revoke_group(&mut self, group_name: &str, permission: &str) {
        if let Some(perms) = self.group_permissions.get_mut(group_name) {
            perms.remove(permission);
            if perms.is_empty() {
                self.group_permissions.remove(group_name);
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn set_public(&mut self, permission: &str) {
        self.public_permissions.insert(permission.to_string());
        self.updated_at = Utc::now();
    }

    pub fn remove_public(&mut self, permission: &str) {
        self.public_permissions.remove(permission);
        self.updated_at = Utc::now();
    }

    pub fn check_access(&self, user_id: Option<Uuid>, groups: &[String], permission: &str) -> bool {
        if self.public_permissions.contains(permission) {
            return true;
        }

        if let Some(uid) = user_id {
            if self.owner_id == Some(uid) {
                return true;
            }

            if let Some(user_perms) = self.permissions.get(&uid) {
                if user_perms.contains(permission) || user_perms.contains("admin") {
                    return true;
                }
            }
        }

        for group in groups {
            if let Some(group_perms) = self.group_permissions.get(group) {
                if group_perms.contains(permission) || group_perms.contains("admin") {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CacheEntry<T> {
    // Visited from `manager.rs` (#1370 split).
    pub(crate) value: T,
    pub(crate) expires_at: DateTime<Utc>,
}

impl<T: Clone> CacheEntry<T> {
    pub(crate) fn new(value: T, ttl_seconds: u64) -> Self {
        Self {
            value,
            expires_at: Utc::now() + chrono::Duration::seconds(ttl_seconds as i64),
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

pub struct RbacManager {
    // #1370 — the manager's impl lives in `manager.rs`, so its fields are
    // crate-visible rather than module-private.
    pub(crate) config: RbacConfig,
    pub(crate) route_permissions: Arc<RwLock<Vec<RoutePermission>>>,
    pub(crate) resource_acls: Arc<RwLock<HashMap<String, ResourceAcl>>>,
    pub(crate) permission_cache: Arc<RwLock<HashMap<String, CacheEntry<AccessDecisionResult>>>>,
    pub(crate) user_groups: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
}

impl std::fmt::Debug for RbacManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RbacManager")
            .field("config", &self.config)
            .finish()
    }
}
