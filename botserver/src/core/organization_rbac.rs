use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
    Execute,
    Share,
    Export,
    Import,
    Manage,
    Configure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Organization,
    Bot,
    KnowledgeBase,
    Form,
    Site,
    Document,
    Conversation,
    User,
    Role,
    Group,
    Channel,
    Workflow,
    Project,
    Meeting,
    Report,
    ApiKey,
    Webhook,
    Integration,
    Billing,
    Audit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub resource_id: Uuid,
    pub organization_id: Uuid,
    pub parent_resource: Option<Box<Resource>>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub id: Uuid,
    pub permission: Permission,
    pub resource_type: ResourceType,
    pub resource_id: Option<Uuid>,
    pub conditions: Vec<PermissionCondition>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCondition {
    pub condition_type: ConditionType,
    pub operator: ConditionOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    TimeOfDay,
    DayOfWeek,
    IpAddress,
    Location,
    DeviceType,
    MfaVerified,
    SessionAge,
    ResourceOwner,
    ResourceTag,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    In,
    NotIn,
    Matches,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationRole {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system_role: bool,
    pub hierarchy_level: u32,
    pub permissions: Vec<PermissionGrant>,
    pub inherits_from: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationGroup {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system_group: bool,
    pub roles: Vec<Uuid>,
    pub members: Vec<Uuid>,
    pub permissions: Vec<PermissionGrant>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionContext {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub roles: Vec<Uuid>,
    pub groups: Vec<Uuid>,
    pub direct_permissions: Vec<PermissionGrant>,
    pub is_organization_owner: bool,
    pub is_organization_admin: bool,
    pub mfa_verified: bool,
    pub session_created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCheckRequest {
    pub user_context: UserPermissionContext,
    pub permission: Permission,
    pub resource: Resource,
    pub action_context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCheckResult {
    pub allowed: bool,
    pub reason: AccessReason,
    pub matching_grants: Vec<PermissionGrant>,
    pub evaluated_policies: Vec<PolicyEvaluation>,
    pub audit_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessReason {
    Allowed,
    OrganizationOwner,
    OrganizationAdmin,
    RoleGrant,
    GroupGrant,
    DirectGrant,
    ResourceOwner,
    InheritedPermission,
    DeniedNoPermission,
    DeniedConditionFailed,
    DeniedExpiredGrant,
    DeniedResourceNotFound,
    DeniedOrganizationMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub result: PolicyResult,
    pub conditions_evaluated: Vec<ConditionEvaluation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyResult {
    Allow,
    Deny,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionEvaluation {
    pub condition: PermissionCondition,
    pub passed: bool,
    pub actual_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: Uuid,
    pub resource_type: ResourceType,
    pub resource_pattern: Option<String>,
    pub effect: PolicyEffect,
    pub principals: PolicyPrincipals,
    pub permissions: Vec<Permission>,
    pub conditions: Vec<PermissionCondition>,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyPrincipals {
    pub users: Vec<Uuid>,
    pub roles: Vec<Uuid>,
    pub groups: Vec<Uuid>,
    pub all_authenticated: bool,
    pub resource_owner: bool,
}

type UserRolesMap = HashMap<(Uuid, Uuid), Vec<Uuid>>;

pub struct OrganizationRbacService {
    roles: Arc<RwLock<HashMap<Uuid, OrganizationRole>>>,
    groups: Arc<RwLock<HashMap<Uuid, OrganizationGroup>>>,
    policies: Arc<RwLock<HashMap<Uuid, ResourcePolicy>>>,
    user_roles: Arc<RwLock<UserRolesMap>>,
    audit_log: Arc<RwLock<Vec<AccessAuditEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub permission: Permission,
    pub resource_type: ResourceType,
    pub resource_id: Option<Uuid>,
    pub result: AccessReason,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Default for OrganizationRbacService {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganizationRbacService {
    pub fn new() -> Self {
        Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn check_access(&self, request: AccessCheckRequest) -> AccessCheckResult {
        let audit_id = Uuid::new_v4();
        let mut matching_grants = Vec::new();
        let mut evaluated_policies = Vec::new();

        if request.resource.organization_id != request.user_context.organization_id {
            return self
                .create_denied_result(
                    audit_id,
                    AccessReason::DeniedOrganizationMismatch,
                    &request,
                )
                .await;
        }

        if request.user_context.is_organization_owner {
            return self
                .create_allowed_result(
                    audit_id,
                    AccessReason::OrganizationOwner,
                    matching_grants,
                    evaluated_policies,
                    &request,
                )
                .await;
        }

        if request.user_context.is_organization_admin
            && self.is_admin_allowed_permission(&request.permission)
        {
            return self
                .create_allowed_result(
                    audit_id,
                    AccessReason::OrganizationAdmin,
                    matching_grants,
                    evaluated_policies,
                    &request,
                )
                .await;
        }

        if let Some(owner_id) = &request.resource.owner_id {
            if *owner_id == request.user_context.user_id
                && self.is_owner_implied_permission(&request.permission)
            {
                return self
                    .create_allowed_result(
                        audit_id,
                        AccessReason::ResourceOwner,
                        matching_grants,
                        evaluated_policies,
                        &request,
                    )
                    .await;
            }
        }

        let policy_result = self.evaluate_policies(&request, &mut evaluated_policies).await;
        if let Some(result) = policy_result {
            if result == PolicyEffect::Deny {
                return self
                    .create_denied_result(audit_id, AccessReason::DeniedConditionFailed, &request)
                    .await;
            }
        }

        if let Some(grant) =
            self.check_direct_permissions(&request, &mut matching_grants).await
        {
            if self.evaluate_conditions(&grant.conditions, &request).await {
                return self
                    .create_allowed_result(
                        audit_id,
                        AccessReason::DirectGrant,
                        matching_grants,
                        evaluated_policies,
                        &request,
                    )
                    .await;
            }
        }

        if let Some(grant) = self.check_role_permissions(&request, &mut matching_grants).await {
            if self.evaluate_conditions(&grant.conditions, &request).await {
                return self
                    .create_allowed_result(
                        audit_id,
                        AccessReason::RoleGrant,
                        matching_grants,
                        evaluated_policies,
                        &request,
                    )
                    .await;
            }
        }

        if let Some(grant) = self.check_group_permissions(&request, &mut matching_grants).await {
            if self.evaluate_conditions(&grant.conditions, &request).await {
                return self
                    .create_allowed_result(
                        audit_id,
                        AccessReason::GroupGrant,
                        matching_grants,
                        evaluated_policies,
                        &request,
                    )
                    .await;
            }
        }

        if let Some(grant) = self
            .check_inherited_permissions(&request, &mut matching_grants)
            .await
        {
            if self.evaluate_conditions(&grant.conditions, &request).await {
                return self
                    .create_allowed_result(
                        audit_id,
                        AccessReason::InheritedPermission,
                        matching_grants,
                        evaluated_policies,
                        &request,
                    )
                    .await;
            }
        }

        self.create_denied_result(audit_id, AccessReason::DeniedNoPermission, &request)
            .await
    }

    async fn evaluate_policies(
        &self,
        request: &AccessCheckRequest,
        evaluated: &mut Vec<PolicyEvaluation>,
    ) -> Option<PolicyEffect> {
        let policies = self.policies.read().await;
        let mut applicable_policies: Vec<&ResourcePolicy> = policies
            .values()
            .filter(|p| {
                p.enabled
                    && p.organization_id == request.user_context.organization_id
                    && p.resource_type == request.resource.resource_type
                    && p.permissions.contains(&request.permission)
            })
            .collect();

        applicable_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in applicable_policies {
            let principal_matches = self.check_principal_match(policy, request);
            if !principal_matches {
                evaluated.push(PolicyEvaluation {
                    policy_id: policy.id,
                    policy_name: policy.name.clone(),
                    result: PolicyResult::NotApplicable,
                    conditions_evaluated: Vec::new(),
                });
                continue;
            }

            let mut condition_evals = Vec::new();
            let conditions_pass = self
                .evaluate_policy_conditions(&policy.conditions, request, &mut condition_evals)
                .await;

            let result = if conditions_pass {
                match policy.effect {
                    PolicyEffect::Allow => PolicyResult::Allow,
                    PolicyEffect::Deny => PolicyResult::Deny,
                }
            } else {
                PolicyResult::NotApplicable
            };

            evaluated.push(PolicyEvaluation {
                policy_id: policy.id,
                policy_name: policy.name.clone(),
                result: result.clone(),
                conditions_evaluated: condition_evals,
            });

            if result == PolicyResult::Deny {
                return Some(PolicyEffect::Deny);
            }
            if result == PolicyResult::Allow {
                return Some(PolicyEffect::Allow);
            }
        }

        None
    }

    fn check_principal_match(&self, policy: &ResourcePolicy, request: &AccessCheckRequest) -> bool {
        if policy.principals.all_authenticated {
            return true;
        }

        if policy.principals.resource_owner {
            if let Some(owner_id) = &request.resource.owner_id {
                if *owner_id == request.user_context.user_id {
                    return true;
                }
            }
        }

        if policy.principals.users.contains(&request.user_context.user_id) {
            return true;
        }

        for role_id in &request.user_context.roles {
            if policy.principals.roles.contains(role_id) {
                return true;
            }
        }

        for group_id in &request.user_context.groups {
            if policy.principals.groups.contains(group_id) {
                return true;
            }
        }

        false
    }

    async fn evaluate_policy_conditions(
        &self,
        conditions: &[PermissionCondition],
        request: &AccessCheckRequest,
        evals: &mut Vec<ConditionEvaluation>,
    ) -> bool {
        for condition in conditions {
            let (passed, actual_value) = self.evaluate_single_condition(condition, request);
            evals.push(ConditionEvaluation {
                condition: condition.clone(),
                passed,
                actual_value,
            });
            if !passed {
                return false;
            }
        }
        true
    }

    fn evaluate_single_condition(
        &self,
        condition: &PermissionCondition,
        request: &AccessCheckRequest,
    ) -> (bool, Option<String>) {
        match condition.condition_type {
            ConditionType::MfaVerified => {
                let actual = request.user_context.mfa_verified;
                let expected = condition.value == "true";
                (actual == expected, Some(actual.to_string()))
            }
            ConditionType::IpAddress => {
                if let Some(ip) = &request.user_context.ip_address {
                    let passed = self.match_condition_value(&condition.operator, ip, &condition.value);
                    (passed, Some(ip.clone()))
                } else {
                    (false, None)
                }
            }
            ConditionType::DeviceType => {
                if let Some(device) = &request.user_context.device_type {
                    let passed = self.match_condition_value(&condition.operator, device, &condition.value);
                    (passed, Some(device.clone()))
                } else {
                    (false, None)
                }
            }
            ConditionType::SessionAge => {
                let age_seconds = (Utc::now() - request.user_context.session_created_at).num_seconds();
                let max_age: i64 = condition.value.parse().unwrap_or(3600);
                let passed = match condition.operator {
                    ConditionOperator::LessThan => age_seconds < max_age,
                    ConditionOperator::GreaterThan => age_seconds > max_age,
                    _ => age_seconds <= max_age,
                };
                (passed, Some(age_seconds.to_string()))
            }
            ConditionType::ResourceOwner => {
                if let Some(owner_id) = &request.resource.owner_id {
                    let is_owner = *owner_id == request.user_context.user_id;
                    let expected = condition.value == "true";
                    (is_owner == expected, Some(is_owner.to_string()))
                } else {
                    (condition.value == "false", None)
                }
            }
            ConditionType::TimeOfDay | ConditionType::DayOfWeek | ConditionType::Location => {
                (true, None)
            }
            ConditionType::ResourceTag => {
                if let Some(tag_value) = request.action_context.get(&condition.value) {
                    (true, Some(tag_value.clone()))
                } else {
                    (false, None)
                }
            }
            ConditionType::Custom => {
                if let Some(custom_value) = request.action_context.get(&condition.value) {
                    (true, Some(custom_value.clone()))
                } else {
                    (false, None)
                }
            }
        }
    }

    fn match_condition_value(&self, operator: &ConditionOperator, actual: &str, expected: &str) -> bool {
        match operator {
            ConditionOperator::Equals => actual == expected,
            ConditionOperator::NotEquals => actual != expected,
            ConditionOperator::Contains => actual.contains(expected),
            ConditionOperator::StartsWith => actual.starts_with(expected),
            ConditionOperator::EndsWith => actual.ends_with(expected),
            ConditionOperator::In => {
                expected.split(',').any(|v| v.trim() == actual)
            }
            ConditionOperator::NotIn => {
                !expected.split(',').any(|v| v.trim() == actual)
            }
            ConditionOperator::Matches => {
                regex::Regex::new(expected)
                    .map(|re| re.is_match(actual))
                    .unwrap_or(false)
            }
            ConditionOperator::GreaterThan | ConditionOperator::LessThan => {
                if let (Ok(a), Ok(e)) = (actual.parse::<i64>(), expected.parse::<i64>()) {
                    match operator {
                        ConditionOperator::GreaterThan => a > e,
                        ConditionOperator::LessThan => a < e,
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }

    async fn check_direct_permissions(
        &self,
        request: &AccessCheckRequest,
        matching: &mut Vec<PermissionGrant>,
    ) -> Option<PermissionGrant> {
        for grant in &request.user_context.direct_permissions {
            if self.grant_matches(grant, request) {
                matching.push(grant.clone());
                return Some(grant.clone());
            }
        }
        None
    }

    async fn check_role_permissions(
        &self,
        request: &AccessCheckRequest,
        matching: &mut Vec<PermissionGrant>,
    ) -> Option<PermissionGrant> {
        let roles = self.roles.read().await;

        for role_id in &request.user_context.roles {
            if let Some(role) = roles.get(role_id) {
                for grant in &role.permissions {
                    if self.grant_matches(grant, request) {
                        matching.push(grant.clone());
                        return Some(grant.clone());
                    }
                }
            }
        }
        None
    }

    async fn check_group_permissions(
        &self,
        request: &AccessCheckRequest,
        matching: &mut Vec<PermissionGrant>,
    ) -> Option<PermissionGrant> {
        let groups = self.groups.read().await;

        for group_id in &request.user_context.groups {
            if let Some(group) = groups.get(group_id) {
                for grant in &group.permissions {
                    if self.grant_matches(grant, request) {
                        matching.push(grant.clone());
                        return Some(grant.clone());
                    }
                }

                let roles_guard = self.roles.read().await;
                for role_id in &group.roles {
                    if let Some(role) = roles_guard.get(role_id) {
                        for grant in &role.permissions {
                            if self.grant_matches(grant, request) {
                                matching.push(grant.clone());
                                return Some(grant.clone());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    async fn check_inherited_permissions(
        &self,
        request: &AccessCheckRequest,
        matching: &mut Vec<PermissionGrant>,
    ) -> Option<PermissionGrant> {
        let roles = self.roles.read().await;

        let mut visited_roles = HashSet::new();
        let mut roles_to_check: Vec<Uuid> = request.user_context.roles.clone();

        while let Some(role_id) = roles_to_check.pop() {
            if visited_roles.contains(&role_id) {
                continue;
            }
            visited_roles.insert(role_id);

            if let Some(role) = roles.get(&role_id) {
                for parent_role_id in &role.inherits_from {
                    if let Some(parent_role) = roles.get(parent_role_id) {
                        for grant in &parent_role.permissions {
                            if self.grant_matches(grant, request) {
                                matching.push(grant.clone());
                                return Some(grant.clone());
                            }
                        }
                        roles_to_check.push(*parent_role_id);
                    }
                }
            }
        }
        None
    }

    fn grant_matches(&self, grant: &PermissionGrant, request: &AccessCheckRequest) -> bool {
        if grant.permission != request.permission {
            return false;
        }

        if grant.resource_type != request.resource.resource_type {
            return false;
        }

        if let Some(resource_id) = grant.resource_id {
            if resource_id != request.resource.resource_id {
                return false;
            }
        }

        if let Some(expires_at) = grant.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }

        true
    }

    async fn evaluate_conditions(
        &self,
        conditions: &[PermissionCondition],
        request: &AccessCheckRequest,
    ) -> bool {
        for condition in conditions {
            let (passed, _) = self.evaluate_single_condition(condition, request);
            if !passed {
                return false;
            }
        }
        true
    }

    fn is_admin_allowed_permission(&self, permission: &Permission) -> bool {
        matches!(
            permission,
            Permission::Read
                | Permission::Write
                | Permission::Delete
                | Permission::Execute
                | Permission::Share
                | Permission::Export
                | Permission::Import
                | Permission::Manage
                | Permission::Configure
        )
    }

    fn is_owner_implied_permission(&self, permission: &Permission) -> bool {
        matches!(
            permission,
            Permission::Read
                | Permission::Write
                | Permission::Delete
                | Permission::Share
                | Permission::Export
        )
    }

    async fn create_allowed_result(
        &self,
        audit_id: Uuid,
        reason: AccessReason,
        matching_grants: Vec<PermissionGrant>,
        evaluated_policies: Vec<PolicyEvaluation>,
        request: &AccessCheckRequest,
    ) -> AccessCheckResult {
        self.log_access(audit_id, request, &reason).await;
        AccessCheckResult {
            allowed: true,
            reason,
            matching_grants,
            evaluated_policies,
            audit_id,
        }
    }

    async fn create_denied_result(
        &self,
        audit_id: Uuid,
        reason: AccessReason,
        request: &AccessCheckRequest,
    ) -> AccessCheckResult {
        self.log_access(audit_id, request, &reason).await;
        AccessCheckResult {
            allowed: false,
            reason,
            matching_grants: Vec::new(),
            evaluated_policies: Vec::new(),
            audit_id,
        }
    }

    async fn log_access(&self, audit_id: Uuid, request: &AccessCheckRequest, reason: &AccessReason) {
        let entry = AccessAuditEntry {
            id: audit_id,
            timestamp: Utc::now(),
            user_id: request.user_context.user_id,
            organization_id: request.user_context.organization_id,
            permission: request.permission.clone(),
            resource_type: request.resource.resource_type.clone(),
            resource_id: Some(request.resource.resource_id),
            result: reason.clone(),
            ip_address: request.user_context.ip_address.clone(),
            user_agent: request.action_context.get("user_agent").cloned(),
        };

        let mut log = self.audit_log.write().await;
        log.push(entry);

        if log.len() > 10000 {
            log.drain(0..1000);
        }
    }

    pub async fn create_role(&self, role: OrganizationRole) -> Result<OrganizationRole, String> {
        let mut roles = self.roles.write().await;
        if roles.values().any(|r| {
            r.organization_id == role.organization_id && r.name == role.name && r.id != role.id
        }) {
            return Err("Role with this name already exists".to_string());
        }
        roles.insert(role.id, role.clone());
        Ok(role)
    }

    pub async fn update_role(&self, role: OrganizationRole) -> Result<OrganizationRole, String> {
        let mut roles = self.roles.write().await;
        if !roles.contains_key(&role.id) {
            return Err("Role not found".to_string());
        }
        roles.insert(role.id, role.clone());
        Ok(role)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<(), String> {
        let mut roles = self.roles.write().await;
        if let Some(role) = roles.get(&role_id) {
            if role.is_system_role {
                return Err("Cannot delete system role".to_string());
            }
        }
        roles.remove(&role_id);
        Ok(())
    }

    pub async fn get_role(&self, role_id: Uuid) -> Option<OrganizationRole> {
        let roles = self.roles.read().await;
        roles.get(&role_id).cloned()
    }

    pub async fn get_organization_roles(&self, organization_id: Uuid) -> Vec<OrganizationRole> {
        let roles = self.roles.read().await;
        roles
            .values()
            .filter(|r| r.organization_id == organization_id)
            .cloned()
            .collect()
    }

    pub async fn create_group(&self, group: OrganizationGroup) -> Result<OrganizationGroup, String> {
        let mut groups = self.groups.write().await;
        if groups.values().any(|g| {
            g.organization_id == group.organization_id && g.name == group.name && g.id != group.id
        }) {
            return Err("Group with this name already exists".to_string());
        }
        groups.insert(group.id, group.clone());
        Ok(group)
    }

    pub async fn update_group(&self, group: OrganizationGroup) -> Result<OrganizationGroup, String> {
        let mut groups = self.groups.write().await;
        if !groups.contains_key(&group.id) {
            return Err("Group not found".to_string());
        }
        groups.insert(group.id, group.clone());
        Ok(group)
    }

    pub async fn delete_group(&self, group_id: Uuid) -> Result<(), String> {
        let mut groups = self.groups.write().await;
        if let Some(group) = groups.get(&group_id) {
            if group.is_system_group {
                return Err("Cannot delete system group".to_string());
            }
        }
        groups.remove(&group_id);
        Ok(())
    }

    pub async fn add_user_to_role(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), String> {
        let roles = self.roles.read().await;
        if !roles.contains_key(&role_id) {
            return Err("Role not found".to_string());
        }
        drop(roles);

        let mut user_roles = self.user_roles.write().await;
        let entry = user_roles
            .entry((organization_id, user_id))
            .or_default();

        if !entry.contains(&role_id) {
            entry.push(role_id);
        }
        Ok(())
    }

    pub async fn remove_user_from_role(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), String> {
        let mut user_roles = self.user_roles.write().await;
        if let Some(roles) = user_roles.get_mut(&(organization_id, user_id)) {
            roles.retain(|&r| r != role_id);
        }
        Ok(())
    }

    pub async fn get_user_roles(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Vec<OrganizationRole> {
        let user_roles = self.user_roles.read().await;
        let roles = self.roles.read().await;

        user_roles
            .get(&(organization_id, user_id))
            .map(|role_ids| {
                role_ids
                    .iter()
                    .filter_map(|id| roles.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
}
