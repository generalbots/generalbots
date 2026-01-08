//! Organization Management Module
//!
//! Provides organization creation, role management, group management,
//! and access control for multi-tenant deployments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Organization Types
// ============================================================================

/// Organization entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub website: Option<String>,
    pub plan_id: String,
    pub owner_id: Uuid,
    pub settings: OrganizationSettings,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Organization {
    pub fn new(name: String, owner_id: Uuid) -> Self {
        let slug = slugify(&name);
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            name,
            slug,
            description: None,
            logo_url: None,
            website: None,
            plan_id: "free".to_string(),
            owner_id,
            settings: OrganizationSettings::default(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn with_plan(mut self, plan_id: String) -> Self {
        self.plan_id = plan_id;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Organization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub allow_public_bots: bool,
    pub require_2fa: bool,
    pub allowed_email_domains: Vec<String>,
    pub default_user_role: String,
    pub max_members: Option<u32>,
    pub sso_enabled: bool,
    pub sso_provider: Option<String>,
    pub audit_log_retention_days: u32,
    pub ip_whitelist: Vec<String>,
    pub custom_branding: Option<CustomBranding>,
}

impl Default for OrganizationSettings {
    fn default() -> Self {
        Self {
            allow_public_bots: false,
            require_2fa: false,
            allowed_email_domains: Vec::new(),
            default_user_role: "member".to_string(),
            max_members: None,
            sso_enabled: false,
            sso_provider: None,
            audit_log_retention_days: 90,
            ip_whitelist: Vec::new(),
            custom_branding: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomBranding {
    pub primary_color: String,
    pub secondary_color: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub custom_css: Option<String>,
}

// ============================================================================
// Organization Member
// ============================================================================

/// Organization member entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub status: MemberStatus,
    pub invited_by: Option<Uuid>,
    pub invited_at: Option<DateTime<Utc>>,
    pub joined_at: Option<DateTime<Utc>>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrganizationMember {
    pub fn new(organization_id: Uuid, user_id: Uuid, role: &str) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            organization_id,
            user_id,
            role: role.to_string(),
            status: MemberStatus::Active,
            invited_by: None,
            invited_at: None,
            joined_at: Some(now),
            last_active_at: Some(now),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn as_invited(mut self, invited_by: Uuid) -> Self {
        self.status = MemberStatus::Invited;
        self.invited_by = Some(invited_by);
        self.invited_at = Some(Utc::now());
        self.joined_at = None;
        self
    }

    pub fn accept_invitation(&mut self) {
        self.status = MemberStatus::Active;
        self.joined_at = Some(Utc::now());
        self.last_active_at = Some(Utc::now());
    }

    pub fn is_active(&self) -> bool {
        self.status == MemberStatus::Active
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    Active,
    Invited,
    Suspended,
    Deactivated,
}

// ============================================================================
// Roles
// ============================================================================

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub organization_id: Option<Uuid>, // None for system roles
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub is_system: bool,
    pub is_default: bool,
    pub hierarchy_level: u32, // Lower = more powerful
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    pub fn new(name: &str, display_name: &str, permissions: Vec<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            organization_id: None,
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: None,
            permissions,
            is_system: false,
            is_default: false,
            hierarchy_level: 100,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn system_role(name: &str, display_name: &str, permissions: Vec<String>, level: u32) -> Self {
        Self {
            is_system: true,
            hierarchy_level: level,
            ..Self::new(name, display_name, permissions)
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
            || self.permissions.contains(&"*".to_string())
    }

    pub fn can_manage(&self, other: &Role) -> bool {
        self.hierarchy_level < other.hierarchy_level
    }
}

/// Default system roles
pub fn default_roles() -> Vec<Role> {
    vec![
        Role::system_role(
            "owner",
            "Owner",
            vec!["*".to_string()],
            0,
        ),
        Role::system_role(
            "admin",
            "Administrator",
            vec![
                "org:manage".to_string(),
                "org:members".to_string(),
                "org:settings".to_string(),
                "bot:*".to_string(),
                "kb:*".to_string(),
                "app:*".to_string(),
                "analytics:*".to_string(),
            ],
            10,
        ),
        Role::system_role(
            "manager",
            "Manager",
            vec![
                "org:members:view".to_string(),
                "bot:create".to_string(),
                "bot:edit".to_string(),
                "bot:delete".to_string(),
                "bot:publish".to_string(),
                "kb:read".to_string(),
                "kb:write".to_string(),
                "app:create".to_string(),
                "app:edit".to_string(),
                "app:delete".to_string(),
                "analytics:view".to_string(),
                "analytics:export".to_string(),
            ],
            20,
        ),
        Role::system_role(
            "member",
            "Member",
            vec![
                "bot:create".to_string(),
                "bot:edit".to_string(),
                "kb:read".to_string(),
                "kb:write".to_string(),
                "app:create".to_string(),
                "app:edit".to_string(),
                "analytics:view".to_string(),
            ],
            50,
        ),
        Role::system_role(
            "viewer",
            "Viewer",
            vec![
                "bot:view".to_string(),
                "kb:read".to_string(),
                "app:view".to_string(),
                "analytics:view".to_string(),
            ],
            80,
        ),
        Role::system_role(
            "guest",
            "Guest",
            vec![
                "kb:read".to_string(),
            ],
            90,
        ),
    ]
}

// ============================================================================
// Groups
// ============================================================================

/// Group definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub parent_group_id: Option<Uuid>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Group {
    pub fn new(organization_id: Uuid, name: &str, display_name: &str) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            organization_id,
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: None,
            permissions: Vec::new(),
            parent_group_id: None,
            is_system: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn as_system(mut self) -> Self {
        self.is_system = true;
        self
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
}

/// Group member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub added_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl GroupMember {
    pub fn new(group_id: Uuid, user_id: Uuid, added_by: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            group_id,
            user_id,
            added_by,
            created_at: Utc::now(),
        }
    }
}

/// Default groups for an organization
pub fn default_groups(organization_id: Uuid) -> Vec<Group> {
    vec![
        Group::new(organization_id, "everyone", "Everyone")
            .with_description("All members of the organization".to_string())
            .with_permissions(vec!["kb:read:public".to_string()])
            .as_system(),
        Group::new(organization_id, "developers", "Developers")
            .with_description("Development team members".to_string())
            .with_permissions(vec![
                "bot:create".to_string(),
                "bot:edit".to_string(),
                "kb:write".to_string(),
                "app:create".to_string(),
            ]),
        Group::new(organization_id, "content_managers", "Content Managers")
            .with_description("Content management team".to_string())
            .with_permissions(vec![
                "kb:read".to_string(),
                "kb:write".to_string(),
            ]),
        Group::new(organization_id, "support", "Support Team")
            .with_description("Customer support team".to_string())
            .with_permissions(vec![
                "bot:view".to_string(),
                "kb:read".to_string(),
                "analytics:view".to_string(),
            ]),
    ]
}

// ============================================================================
// User Role Assignment
// ============================================================================

/// User role within an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl UserRole {
    pub fn new(
        user_id: Uuid,
        organization_id: Uuid,
        role_id: Uuid,
        role_name: &str,
        assigned_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            organization_id,
            role_id,
            role_name: role_name.to_string(),
            assigned_by,
            assigned_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| exp < Utc::now())
            .unwrap_or(false)
    }
}

// ============================================================================
// Bot Access Control
// ============================================================================

/// Bot access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotAccessConfig {
    pub bot_id: Uuid,
    pub organization_id: Uuid,
    pub visibility: BotVisibility,
    pub allowed_roles: Vec<String>,
    pub allowed_groups: Vec<String>,
    pub allowed_users: Vec<Uuid>,
    pub denied_users: Vec<Uuid>,
    pub requires_authentication: bool,
    pub ip_restrictions: Vec<String>,
    pub rate_limit_per_user: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BotAccessConfig {
    pub fn new(bot_id: Uuid, organization_id: Uuid) -> Self {
        let now = Utc::now();

        Self {
            bot_id,
            organization_id,
            visibility: BotVisibility::Private,
            allowed_roles: Vec::new(),
            allowed_groups: Vec::new(),
            allowed_users: Vec::new(),
            denied_users: Vec::new(),
            requires_authentication: true,
            ip_restrictions: Vec::new(),
            rate_limit_per_user: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn public() -> Self {
        Self {
            visibility: BotVisibility::Public,
            requires_authentication: false,
            ..Self::new(Uuid::nil(), Uuid::nil())
        }
    }

    pub fn check_access(&self, user: &UserAccessContext) -> AccessCheckResult {
        // Check if user is explicitly denied
        if self.denied_users.contains(&user.user_id) {
            return AccessCheckResult::Denied("User explicitly denied".to_string());
        }

        // Public bots
        if self.visibility == BotVisibility::Public {
            if self.requires_authentication && !user.is_authenticated {
                return AccessCheckResult::Denied("Authentication required".to_string());
            }
            return AccessCheckResult::Allowed;
        }

        // Must be authenticated for non-public bots
        if !user.is_authenticated {
            return AccessCheckResult::Denied("Authentication required".to_string());
        }

        // Check if user is explicitly allowed
        if self.allowed_users.contains(&user.user_id) {
            return AccessCheckResult::Allowed;
        }

        // Check role access
        if !self.allowed_roles.is_empty() {
            for role in &user.roles {
                if self.allowed_roles.contains(role) {
                    return AccessCheckResult::Allowed;
                }
            }
        }

        // Check group access
        if !self.allowed_groups.is_empty() {
            for group in &user.groups {
                if self.allowed_groups.contains(group) {
                    return AccessCheckResult::Allowed;
                }
            }
        }

        // Organization-wide access
        if self.visibility == BotVisibility::Organization {
            if user.organization_id == Some(self.organization_id) {
                return AccessCheckResult::Allowed;
            }
        }

        AccessCheckResult::Denied("Access not granted".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotVisibility {
    Private,      // Only explicit users/roles/groups
    Organization, // All org members
    Public,       // Anyone (optionally with auth)
}

// ============================================================================
// App Access Control
// ============================================================================

/// App access configuration (Forms, Sites, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAccessConfig {
    pub app_id: Uuid,
    pub app_type: AppType,
    pub organization_id: Uuid,
    pub visibility: AppVisibility,
    pub allowed_roles: Vec<String>,
    pub allowed_groups: Vec<String>,
    pub allowed_users: Vec<Uuid>,
    pub requires_authentication: bool,
    pub submission_requires_auth: bool, // For forms
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AppAccessConfig {
    pub fn new(app_id: Uuid, app_type: AppType, organization_id: Uuid) -> Self {
        let now = Utc::now();

        Self {
            app_id,
            app_type,
            organization_id,
            visibility: AppVisibility::Private,
            allowed_roles: Vec::new(),
            allowed_groups: Vec::new(),
            allowed_users: Vec::new(),
            requires_authentication: true,
            submission_requires_auth: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn check_access(&self, user: &UserAccessContext, action: AppAction) -> AccessCheckResult {
        match action {
            AppAction::View => self.check_view_access(user),
            AppAction::Edit => self.check_edit_access(user),
            AppAction::Submit => self.check_submit_access(user),
            AppAction::Admin => self.check_admin_access(user),
        }
    }

    fn check_view_access(&self, user: &UserAccessContext) -> AccessCheckResult {
        if self.visibility == AppVisibility::Public {
            if self.requires_authentication && !user.is_authenticated {
                return AccessCheckResult::Denied("Authentication required".to_string());
            }
            return AccessCheckResult::Allowed;
        }

        if !user.is_authenticated {
            return AccessCheckResult::Denied("Authentication required".to_string());
        }

        self.check_membership(user)
    }

    fn check_edit_access(&self, user: &UserAccessContext) -> AccessCheckResult {
        if !user.is_authenticated {
            return AccessCheckResult::Denied("Authentication required".to_string());
        }

        // Check for edit permission in roles
        if user.has_permission("app:edit") {
            return self.check_membership(user);
        }

        AccessCheckResult::Denied("Edit permission required".to_string())
    }

    fn check_submit_access(&self, user: &UserAccessContext) -> AccessCheckResult {
        if self.submission_requires_auth && !user.is_authenticated {
            return AccessCheckResult::Denied("Authentication required for submission".to_string());
        }

        // Public apps allow submissions
        if self.visibility == AppVisibility::Public {
            return AccessCheckResult::Allowed;
        }

        self.check_membership(user)
    }

    fn check_admin_access(&self, user: &UserAccessContext) -> AccessCheckResult {
        if !user.is_authenticated {
            return AccessCheckResult::Denied("Authentication required".to_string());
        }

        if user.has_permission("app:admin") || user.has_permission("*") {
            return AccessCheckResult::Allowed;
        }

        AccessCheckResult::Denied("Admin permission required".to_string())
    }

    fn check_membership(&self, user: &UserAccessContext) -> AccessCheckResult {
        // Explicit user access
        if self.allowed_users.contains(&user.user_id) {
            return AccessCheckResult::Allowed;
        }

        // Role access
        if !self.allowed_roles.is_empty() {
            for role in &user.roles {
                if self.allowed_roles.contains(role) {
                    return AccessCheckResult::Allowed;
                }
            }
        }

        // Group access
        if !self.allowed_groups.is_empty() {
            for group in &user.groups {
                if self.allowed_groups.contains(group) {
                    return AccessCheckResult::Allowed;
                }
            }
        }

        // Organization-wide
        if self.visibility == AppVisibility::Organization {
            if user.organization_id == Some(self.organization_id) {
                return AccessCheckResult::Allowed;
            }
        }

        AccessCheckResult::Denied("Access not granted".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppType {
    Form,
    Site,
    Dashboard,
    Report,
    Workflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppVisibility {
    Private,
    Organization,
    Public,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    View,
    Edit,
    Submit,
    Admin,
}

// ============================================================================
// Access Check Types
// ============================================================================

/// User context for access checks
#[derive(Debug, Clone)]
pub struct UserAccessContext {
    pub user_id: Uuid,
    pub is_authenticated: bool,
    pub organization_id: Option<Uuid>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub permissions: Vec<String>,
}

impl UserAccessContext {
    pub fn anonymous() -> Self {
        Self {
            user_id: Uuid::nil(),
            is_authenticated: false,
            organization_id: None,
            roles: Vec::new(),
            groups: Vec::new(),
            permissions: Vec::new(),
        }
    }

    pub fn authenticated(user_id: Uuid, org_id: Option<Uuid>) -> Self {
        Self {
            user_id,
            is_authenticated: true,
            organization_id: org_id,
            roles: Vec::new(),
            groups: Vec::new(),
            permissions: Vec::new(),
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

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
            || self.permissions.contains(&"*".to_string())
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_group(&self, group: &str) -> bool {
        self.groups.contains(&group.to_string())
    }
}

/// Access check result
#[derive(Debug, Clone)]
pub enum AccessCheckResult {
    Allowed,
    Denied(String),
    RequiresElevation(String),
}

impl AccessCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Allowed => None,
            Self::Denied(r) => Some(r),
            Self::RequiresElevation(r) => Some(r),
        }
    }
}

// ============================================================================
// Organization Service
// ============================================================================

/// Organization management service
pub struct OrganizationService {
    // In production, this would have database pool
}

impl OrganizationService {
    pub fn new() -> Self {
        Self {}
    }

    /// Create a new organization with default roles and groups
    pub async fn create_organization(
        &self,
        name: String,
        owner_id: Uuid,
        plan_id: Option<String>,
    ) -> Result<OrganizationCreationResult, OrganizationError> {
        // Create organization
        let mut org = Organization::new(name, owner_id);
        if let Some(plan) = plan_id {
            org = org.with_plan(plan);
        }

        // Create default roles for org (custom roles in addition to system roles)
        let roles = default_roles();

        // Create default groups
        let groups = default_groups(org.id);

        // Create owner membership
        let owner_member = OrganizationMember::new(org.id, owner_id, "owner");

        // Assign owner role
        let owner_role = roles.iter().find(|r| r.name == "owner").unwrap();
        let owner_role_assignment = UserRole::new(
            owner_id,
            org.id,
            owner_role.id,
            &owner_role.name,
            None,
        );

        Ok(OrganizationCreationResult {
            organization: org,
            roles,
            groups,
            owner_member,
            owner_role: owner_role_assignment,
        })
    }

    /// Invite a user to an organization
    pub fn create_invitation(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        role: &str,
        invited_by: Uuid,
    ) -> OrganizationMember {
        OrganizationMember::new(organization_id, user_id, role)
            .as_invited(invited_by)
    }

    /// Check if user has permission in organization
    pub fn check_permission(
        &self,
        user: &UserAccessContext,
        permission: &str,
    ) -> bool {
        user.has_permission(permission)
    }

    /// Get effective permissions for a user (from roles + groups)
    pub fn get_effective_permissions(
        &self,
        roles: &[Role],
        groups: &[Group],
    ) -> Vec<String> {
        let mut permissions = Vec::new();

        for role in roles {
            for perm in &role.permissions {
                if !permissions.contains(perm) {
                    permissions.push(perm.clone());
                }
            }
        }

        for group in groups {
            for perm in &group.permissions {
                if !permissions.contains(perm) {
                    permissions.push(perm.clone());
                }
            }
        }

        permissions
    }
}

impl Default for OrganizationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of organization creation
#[derive(Debug)]
pub struct OrganizationCreationResult {
    pub organization: Organization,
    pub roles: Vec<Role>,
    pub groups: Vec<Group>,
    pub owner_member: OrganizationMember,
    pub owner_role: UserRole,
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone)]
pub enum OrganizationError {
    NotFound,
    AlreadyExists,
    InvalidName(String),
    PermissionDenied(String),
    MemberLimitReached,
    InvalidRole(String),
    InvalidGroup(String),
    DatabaseError(String),
}

impl std::fmt::Display for OrganizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Organization not found"),
            Self::AlreadyExists => write!(f, "Organization already exists"),
            Self::InvalidName(n) => write!(f, "Invalid organization name: {}", n),
            Self::PermissionDenied(p) => write!(f, "Permission denied: {}", p),
            Self::MemberLimitReached => write!(f, "Member limit reached"),
            Self::InvalidRole(r) => write!(f, "Invalid role: {}", r),
            Self::InvalidGroup(g) => write!(f, "Invalid group: {}", g),
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for OrganizationError {}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a URL-safe slug from a name
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ============================================================================
// Permission Constants
// ============================================================================

pub mod permissions {
    pub const ORG_MANAGE: &str = "org:manage";
    pub const ORG_BILLING: &str = "org:billing";
    pub const ORG_MEMBERS: &str = "org:members";
    pub const ORG_MEMBERS_VIEW: &str = "org:members:view";
    pub const ORG_SETTINGS: &str = "org:settings";
    pub const ORG_DELETE: &str = "org:delete";

    pub const BOT_CREATE: &str = "bot:create";
    pub const BOT_VIEW: &str = "bot:view";
    pub const BOT_EDIT: &str = "bot:edit";
    pub const BOT_DELETE: &str = "bot:delete";
    pub const BOT_PUBLISH: &str = "bot:publish";
    pub const BOT_ALL: &str = "bot:*";

    pub const KB_READ: &str = "kb:read";
    pub const KB_WRITE: &str = "kb:write";
    pub const KB_ADMIN: &str = "kb:admin";
    pub const KB_ALL: &str = "kb:*";

    pub const APP_CREATE: &str = "app:create";
    pub const APP_VIEW: &str = "app:view";
    pub const APP_EDIT: &str = "app:edit";
    pub const APP_DELETE: &str = "app:delete";
    pub const APP_ADMIN: &str = "app:admin";
    pub const APP_ALL: &str = "app:*";

    pub const ANALYTICS_VIEW: &str = "analytics:view";
    pub const ANALYTICS_EXPORT: &str = "analytics:export";
    pub const ANALYTICS_ALL: &str = "analytics:*";

    pub const WILDCARD: &str = "*";
}
