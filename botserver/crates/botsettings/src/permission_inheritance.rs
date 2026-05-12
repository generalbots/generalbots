use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub resource: String,
    pub action: PermissionAction,
    pub scope: PermissionScope,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PermissionAction {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Admin,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionScope {
    Global,
    Organization,
    Bot,
    App,
    Resource(String),
}

#[derive(Debug, Clone)]
pub struct RoleHierarchy {
    roles: HashMap<String, RoleNode>,
    permission_cache: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
struct RoleNode {
    permissions: HashSet<String>,
    parent_roles: Vec<String>,
    hierarchy_level: i32,
}

impl RoleHierarchy {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            permission_cache: HashMap::new(),
        }
    }

    pub fn add_role(
        &mut self,
        name: &str,
        _display_name: &str,
        permissions: Vec<String>,
        parent_roles: Vec<String>,
        hierarchy_level: i32,
    ) {
        let node = RoleNode {
            permissions: permissions.into_iter().collect(),
            parent_roles,
            hierarchy_level,
        };
        self.roles.insert(name.to_string(), node);
        self.permission_cache.clear();
    }

    pub fn remove_role(&mut self, name: &str) {
        self.roles.remove(name);
        self.permission_cache.clear();
    }

    pub fn get_effective_permissions(&mut self, role_name: &str) -> HashSet<String> {
        if let Some(cached) = self.permission_cache.get(role_name) {
            return cached.clone();
        }

        let mut visited = HashSet::new();
        let permissions = self.resolve_permissions_recursive(role_name, &mut visited);
        self.permission_cache.insert(role_name.to_string(), permissions.clone());
        permissions
    }

    fn resolve_permissions_recursive(
        &self,
        role_name: &str,
        visited: &mut HashSet<String>,
    ) -> HashSet<String> {
        if visited.contains(role_name) {
            return HashSet::new();
        }
        visited.insert(role_name.to_string());

        let Some(role) = self.roles.get(role_name) else {
            return HashSet::new();
        };

        let mut permissions = role.permissions.clone();

        for parent in &role.parent_roles {
            let parent_perms = self.resolve_permissions_recursive(parent, visited);
            permissions.extend(parent_perms);
        }

        permissions
    }

    pub fn can_manage(&self, manager_role: &str, target_role: &str) -> bool {
        let manager_level = self.roles.get(manager_role).map(|r| r.hierarchy_level).unwrap_or(0);
        let target_level = self.roles.get(target_role).map(|r| r.hierarchy_level).unwrap_or(0);
        manager_level > target_level
    }

    pub fn get_manageable_roles(&self, role_name: &str) -> Vec<String> {
        let manager_level = self.roles.get(role_name).map(|r| r.hierarchy_level).unwrap_or(0);
        self.roles
            .iter()
            .filter(|(_, node)| node.hierarchy_level < manager_level)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl Default for RoleHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GroupHierarchy {
    groups: HashMap<String, GroupNode>,
    permission_cache: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
struct GroupNode {
    permissions: HashSet<String>,
    parent_group: Option<String>,
    child_groups: Vec<String>,
}

impl GroupHierarchy {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            permission_cache: HashMap::new(),
        }
    }

    pub fn add_group(
        &mut self,
        name: &str,
        permissions: Vec<String>,
        parent_group: Option<String>,
    ) {
        if let Some(ref parent) = parent_group {
            if let Some(parent_node) = self.groups.get_mut(parent) {
                parent_node.child_groups.push(name.to_string());
            }
        }

        let node = GroupNode {
            permissions: permissions.into_iter().collect(),
            parent_group,
            child_groups: Vec::new(),
        };
        self.groups.insert(name.to_string(), node);
        self.permission_cache.clear();
    }

    pub fn remove_group(&mut self, name: &str) {
        if let Some(node) = self.groups.remove(name) {
            if let Some(parent) = &node.parent_group {
                if let Some(parent_node) = self.groups.get_mut(parent) {
                    parent_node.child_groups.retain(|c| c != name);
                }
            }
        }
        self.permission_cache.clear();
    }

    pub fn get_effective_permissions(&mut self, group_name: &str) -> HashSet<String> {
        if let Some(cached) = self.permission_cache.get(group_name) {
            return cached.clone();
        }

        let mut visited = HashSet::new();
        let permissions = self.resolve_permissions_recursive(group_name, &mut visited);
        self.permission_cache.insert(group_name.to_string(), permissions.clone());
        permissions
    }

    fn resolve_permissions_recursive(
        &self,
        group_name: &str,
        visited: &mut HashSet<String>,
    ) -> HashSet<String> {
        if visited.contains(group_name) {
            return HashSet::new();
        }
        visited.insert(group_name.to_string());

        let Some(group) = self.groups.get(group_name) else {
            return HashSet::new();
        };

        let mut permissions = group.permissions.clone();

        if let Some(ref parent) = group.parent_group {
            let parent_perms = self.resolve_permissions_recursive(parent, visited);
            permissions.extend(parent_perms);
        }

        permissions
    }

    pub fn get_ancestors(&self, group_name: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut current = group_name.to_string();

        while let Some(group) = self.groups.get(&current) {
            if let Some(ref parent) = group.parent_group {
                ancestors.push(parent.clone());
                current = parent.clone();
            } else {
                break;
            }
        }

        ancestors
    }

    pub fn get_descendants(&self, group_name: &str) -> Vec<String> {
        let mut descendants = Vec::new();
        let mut queue = vec![group_name.to_string()];

        while let Some(current) = queue.pop() {
            if let Some(group) = self.groups.get(&current) {
                for child in &group.child_groups {
                    descendants.push(child.clone());
                    queue.push(child.clone());
                }
            }
        }

        descendants
    }
}

impl Default for GroupHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PermissionInheritanceResolver {
    role_hierarchy: RoleHierarchy,
    group_hierarchy: GroupHierarchy,
    user_roles: HashMap<Uuid, HashSet<String>>,
    user_groups: HashMap<Uuid, HashSet<String>>,
    user_direct_permissions: HashMap<Uuid, HashSet<String>>,
}

impl PermissionInheritanceResolver {
    pub fn new() -> Self {
        Self {
            role_hierarchy: RoleHierarchy::new(),
            group_hierarchy: GroupHierarchy::new(),
            user_roles: HashMap::new(),
            user_groups: HashMap::new(),
            user_direct_permissions: HashMap::new(),
        }
    }

    pub fn with_default_roles(mut self) -> Self {
        self.role_hierarchy.add_role(
            "owner",
            "Owner",
            vec!["*".to_string()],
            Vec::new(),
            100,
        );

        self.role_hierarchy.add_role(
            "admin",
            "Administrator",
            vec![
                "org:manage".to_string(),
                "org:billing".to_string(),
                "org:members".to_string(),
                "bot:*".to_string(),
                "kb:*".to_string(),
                "app:*".to_string(),
            ],
            vec!["owner".to_string()],
            90,
        );

        self.role_hierarchy.add_role(
            "manager",
            "Manager",
            vec![
                "org:members:view".to_string(),
                "bot:create".to_string(),
                "bot:edit".to_string(),
                "bot:view".to_string(),
                "kb:read".to_string(),
                "kb:write".to_string(),
                "app:create".to_string(),
                "app:edit".to_string(),
            ],
            vec!["admin".to_string()],
            70,
        );

        self.role_hierarchy.add_role(
            "member",
            "Member",
            vec![
                "bot:view".to_string(),
                "kb:read".to_string(),
                "app:view".to_string(),
            ],
            vec!["manager".to_string()],
            50,
        );

        self.role_hierarchy.add_role(
            "viewer",
            "Viewer",
            vec![
                "bot:view".to_string(),
                "kb:read".to_string(),
            ],
            vec!["member".to_string()],
            30,
        );

        self.role_hierarchy.add_role(
            "guest",
            "Guest",
            vec!["bot:view:public".to_string()],
            Vec::new(),
            10,
        );

        self
    }

    pub fn with_default_groups(mut self) -> Self {
        self.group_hierarchy.add_group(
            "everyone",
            vec!["basic:access".to_string()],
            None,
        );

        self.group_hierarchy.add_group(
            "developers",
            vec![
                "bot:create".to_string(),
                "bot:edit".to_string(),
                "kb:write".to_string(),
            ],
            Some("everyone".to_string()),
        );

        self.group_hierarchy.add_group(
            "content_managers",
            vec![
                "kb:write".to_string(),
                "kb:admin".to_string(),
            ],
            Some("everyone".to_string()),
        );

        self.group_hierarchy.add_group(
            "support",
            vec![
                "bot:view".to_string(),
                "analytics:view".to_string(),
            ],
            Some("everyone".to_string()),
        );

        self
    }

    pub fn role_hierarchy_mut(&mut self) -> &mut RoleHierarchy {
        &mut self.role_hierarchy
    }

    pub fn group_hierarchy_mut(&mut self) -> &mut GroupHierarchy {
        &mut self.group_hierarchy
    }

    pub fn assign_role_to_user(&mut self, user_id: Uuid, role: &str) {
        self.user_roles
            .entry(user_id)
            .or_default()
            .insert(role.to_string());
    }

    pub fn remove_role_from_user(&mut self, user_id: Uuid, role: &str) {
        if let Some(roles) = self.user_roles.get_mut(&user_id) {
            roles.remove(role);
        }
    }

    pub fn add_user_to_group(&mut self, user_id: Uuid, group: &str) {
        self.user_groups
            .entry(user_id)
            .or_default()
            .insert(group.to_string());
    }

    pub fn remove_user_from_group(&mut self, user_id: Uuid, group: &str) {
        if let Some(groups) = self.user_groups.get_mut(&user_id) {
            groups.remove(group);
        }
    }

    pub fn add_direct_permission(&mut self, user_id: Uuid, permission: &str) {
        self.user_direct_permissions
            .entry(user_id)
            .or_default()
            .insert(permission.to_string());
    }

    pub fn remove_direct_permission(&mut self, user_id: Uuid, permission: &str) {
        if let Some(perms) = self.user_direct_permissions.get_mut(&user_id) {
            perms.remove(permission);
        }
    }

    pub fn get_effective_permissions(&mut self, user_id: Uuid) -> EffectivePermissions {
        let mut all_permissions = HashSet::new();
        let mut sources = Vec::new();

        if let Some(direct) = self.user_direct_permissions.get(&user_id) {
            for perm in direct {
                all_permissions.insert(perm.clone());
                sources.push(PermissionSource {
                    permission: perm.clone(),
                    source_type: PermissionSourceType::Direct,
                    source_name: "direct".to_string(),
                });
            }
        }

        if let Some(roles) = self.user_roles.get(&user_id).cloned() {
            for role in roles {
                let role_perms = self.role_hierarchy.get_effective_permissions(&role);
                for perm in role_perms {
                    if all_permissions.insert(perm.clone()) {
                        sources.push(PermissionSource {
                            permission: perm,
                            source_type: PermissionSourceType::Role,
                            source_name: role.clone(),
                        });
                    }
                }
            }
        }

        if let Some(groups) = self.user_groups.get(&user_id).cloned() {
            for group in groups {
                let group_perms = self.group_hierarchy.get_effective_permissions(&group);
                for perm in group_perms {
                    if all_permissions.insert(perm.clone()) {
                        sources.push(PermissionSource {
                            permission: perm,
                            source_type: PermissionSourceType::Group,
                            source_name: group.clone(),
                        });
                    }
                }
            }
        }

        EffectivePermissions {
            user_id,
            permissions: all_permissions,
            sources,
        }
    }

    pub fn check_permission(&mut self, user_id: Uuid, required_permission: &str) -> PermissionCheckResult {
        let effective = self.get_effective_permissions(user_id);

        if effective.permissions.contains("*") {
            return PermissionCheckResult {
                allowed: true,
                permission: required_permission.to_string(),
                source: Some(PermissionSource {
                    permission: "*".to_string(),
                    source_type: PermissionSourceType::Role,
                    source_name: "wildcard".to_string(),
                }),
            };
        }

        if effective.permissions.contains(required_permission) {
            let source = effective
                .sources
                .iter()
                .find(|s| s.permission == required_permission)
                .cloned();
            return PermissionCheckResult {
                allowed: true,
                permission: required_permission.to_string(),
                source,
            };
        }

        let parts: Vec<&str> = required_permission.split(':').collect();
        for i in (1..parts.len()).rev() {
            let wildcard = format!("{}:*", parts[..i].join(":"));
            if effective.permissions.contains(&wildcard) {
                let source = effective
                    .sources
                    .iter()
                    .find(|s| s.permission == wildcard)
                    .cloned();
                return PermissionCheckResult {
                    allowed: true,
                    permission: required_permission.to_string(),
                    source,
                };
            }
        }

        PermissionCheckResult {
            allowed: false,
            permission: required_permission.to_string(),
            source: None,
        }
    }

    pub fn get_user_roles(&self, user_id: Uuid) -> Vec<String> {
        self.user_roles
            .get(&user_id)
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_user_groups(&self, user_id: Uuid) -> Vec<String> {
        self.user_groups
            .get(&user_id)
            .map(|g| g.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for PermissionInheritanceResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EffectivePermissions {
    pub user_id: Uuid,
    pub permissions: HashSet<String>,
    pub sources: Vec<PermissionSource>,
}

#[derive(Debug, Clone)]
pub struct PermissionSource {
    pub permission: String,
    pub source_type: PermissionSourceType,
    pub source_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionSourceType {
    Direct,
    Role,
    Group,
}

#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub permission: String,
    pub source: Option<PermissionSource>,
}

pub fn create_default_resolver() -> PermissionInheritanceResolver {
    PermissionInheritanceResolver::new()
        .with_default_roles()
        .with_default_groups()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub visibility: ResourceVisibility,
    pub allowed_roles: Vec<String>,
    pub allowed_groups: Vec<String>,
    pub allowed_users: Vec<Uuid>,
    pub denied_users: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceVisibility {
    Private,
    Organization,
    Public,
}

impl ResourcePermission {
    pub fn check_access(
        &self,
        user_id: Uuid,
        user_roles: &[String],
        user_groups: &[String],
        is_authenticated: bool,
    ) -> bool {
        if self.denied_users.contains(&user_id) {
            return false;
        }

        if self.owner_id == user_id {
            return true;
        }

        if self.allowed_users.contains(&user_id) {
            return true;
        }

        match self.visibility {
            ResourceVisibility::Public => true,
            ResourceVisibility::Organization => {
                if !is_authenticated {
                    return false;
                }

                if !self.allowed_roles.is_empty()
                    && !user_roles.iter().any(|r| self.allowed_roles.contains(r))
                {
                    return false;
                }

                if !self.allowed_groups.is_empty()
                    && !user_groups.iter().any(|g| self.allowed_groups.contains(g))
                {
                    return false;
                }

                true
            }
            ResourceVisibility::Private => false,
        }
    }
}
