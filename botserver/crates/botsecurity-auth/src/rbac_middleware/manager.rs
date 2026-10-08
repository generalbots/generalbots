use super::*;

impl RbacManager {
    pub fn new(config: RbacConfig) -> Self {
        Self {
            config,
            route_permissions: Arc::new(RwLock::new(Vec::new())),
            resource_acls: Arc::new(RwLock::new(HashMap::new())),
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            user_groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {

        Self::new(RbacConfig::default())
    }

    pub async fn register_route(&self, permission: RoutePermission) {
        let mut routes = self.route_permissions.write().await;
        routes.push(permission);
    }

    pub async fn register_routes(&self, permissions: Vec<RoutePermission>) {
        let mut routes = self.route_permissions.write().await;
        routes.extend(permissions);
    }

    pub async fn check_route_access(
        &self,
        path: &str,
        method: &str,
        user: &AuthenticatedUser,
    ) -> AccessDecisionResult {
        let cache_key = format!("route:{}:{}:{}", path, method, user.user_id);

        if self.config.enable_permission_cache {
            let cache = self.permission_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    return entry.value.clone().with_cache_hit();
                }
            }
        }

        let routes = self.route_permissions.read().await;
        let method_upper = method.to_uppercase();

        for route in routes.iter() {
            if route.method.to_uppercase() != method_upper && route.method != "*" {
                continue;
            }

            if !route.matches_path(path) {
                continue;
            }

            // Check allow_anonymous FIRST before authentication check
            if route.allow_anonymous {
                let result = AccessDecisionResult::allow("Anonymous access allowed")
                    .with_rule(route.path_pattern.clone());
                self.cache_result(&cache_key, &result).await;
                return result;
            }

            // Only check authentication after confirming route is not anonymous
            if !user.is_authenticated() {
                let result = AccessDecisionResult::deny("Authentication required");
                return result;
            }

            if !route.required_roles.is_empty() {
                let has_role = route.required_roles.iter().any(|r| {
                    let role = r.parse::<Role>().unwrap_or(Role::Anonymous);
                    user.has_role(&role)
                });

                if !has_role {
                    let result = AccessDecisionResult::deny("Insufficient role")
                        .with_rule(route.path_pattern.clone());
                    return result;
                }
            }

            if !route.required_permission.is_empty() {
                let has_permission = self
                    .check_permission_string(user, &route.required_permission)
                    .await;

                if !has_permission {
                    let result = AccessDecisionResult::deny("Missing required permission")
                        .with_rule(route.path_pattern.clone());
                    return result;
                }
            }

            let result = AccessDecisionResult::allow("Access granted")
                .with_rule(route.path_pattern.clone());
            self.cache_result(&cache_key, &result).await;
            return result;
        }

        if self.config.default_deny {
            let path_matches: Vec<String> = routes
                .iter()
                .filter(|r| r.matches_path(path))
                .map(|r| format!("{}/{}", r.method, r.path_pattern))
                .collect();
            log::info!(
                "RBAC deny trace {} {}: {} routes, path-matches(method-agnostic): {:?}",
                method,
                path,
                routes.len(),
                path_matches
            );
            AccessDecisionResult::deny("No matching route permission found")
        } else {
            AccessDecisionResult::allow("Default allow - no matching rule")
        }
    }

    pub async fn check_resource_access(
        &self,
        user: &AuthenticatedUser,
        resource_type: &str,
        resource_id: &str,
        permission: &str,
    ) -> AccessDecisionResult {
        let cache_key = format!(
            "resource:{}:{}:{}:{}",
            resource_type, resource_id, permission, user.user_id
        );

        if self.config.enable_permission_cache {
            let cache = self.permission_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    return entry.value.clone().with_cache_hit();
                }
            }
        }

        if user.is_admin() {
            let result = AccessDecisionResult::allow("Admin access");
            self.cache_result(&cache_key, &result).await;
            return result;
        }

        let acl_key = format!("{}:{}", resource_type, resource_id);
        let acls = self.resource_acls.read().await;

        if let Some(acl) = acls.get(&acl_key) {
            let user_groups = self.get_user_groups(user.user_id).await;
            let user_id = if user.is_authenticated() {
                Some(user.user_id)
            } else {
                None
            };

            if acl.check_access(user_id, &user_groups, permission) {
                let result = AccessDecisionResult::allow("ACL permission granted");
                self.cache_result(&cache_key, &result).await;
                return result;
            }

            let result = AccessDecisionResult::deny("ACL permission denied");
            return result;
        }

        if self.config.default_deny {
            AccessDecisionResult::deny("No ACL found for resource")
        } else {
            AccessDecisionResult::allow("Default allow - no ACL defined")
        }
    }

    pub async fn set_resource_acl(&self, acl: ResourceAcl) {
        let key = format!("{}:{}", acl.resource_type, acl.resource_id);
        let mut acls = self.resource_acls.write().await;
        acls.insert(key, acl);

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn get_resource_acl(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Option<ResourceAcl> {
        let key = format!("{resource_type}:{resource_id}");
        let acls = self.resource_acls.read().await;
        acls.get(&key).cloned()
    }

    pub async fn delete_resource_acl(&self, resource_type: &str, resource_id: &str) {
        let key = format!("{resource_type}:{resource_id}");
        let mut acls = self.resource_acls.write().await;
        acls.remove(&key);

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn set_user_groups(&self, user_id: Uuid, groups: Vec<String>) {
        let mut user_groups = self.user_groups.write().await;
        user_groups.insert(user_id, groups);

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn add_user_to_group(&self, user_id: Uuid, group: &str) {
        let mut user_groups = self.user_groups.write().await;
        user_groups
            .entry(user_id)
            .or_default()
            .push(group.to_string());

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn remove_user_from_group(&self, user_id: Uuid, group: &str) {
        let mut user_groups = self.user_groups.write().await;
        if let Some(groups) = user_groups.get_mut(&user_id) {
            groups.retain(|g| g != group);
        }

        self.invalidate_cache_prefix("resource:").await;
    }

    pub async fn get_user_groups(&self, user_id: Uuid) -> Vec<String> {
        let user_groups = self.user_groups.read().await;
        user_groups.get(&user_id).cloned().unwrap_or_default()
    }

    pub async fn invalidate_user_cache(&self, user_id: Uuid) {
        let suffix = format!(":{user_id}");
        let infix = format!(":{user_id}:");
        let mut cache = self.permission_cache.write().await;
        cache.retain(|k, _| !k.ends_with(&suffix) && !k.contains(&infix));
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.permission_cache.write().await;
        cache.clear();
    }

    async fn cache_result(&self, key: &str, result: &AccessDecisionResult) {
        if !self.config.enable_permission_cache {
            return;
        }

        let mut cache = self.permission_cache.write().await;
        cache.insert(
            key.to_string(),
            CacheEntry::new(result.clone(), self.config.cache_ttl_seconds),
        );
    }

    async fn invalidate_cache_prefix(&self, prefix: &str) {
        let mut cache = self.permission_cache.write().await;
        cache.retain(|k, _| !k.starts_with(prefix));
    }

    pub async fn check_permission_string(&self, user: &AuthenticatedUser, permission_str: &str) -> bool {
        let cache_key = format!("user_perm:{}:{}", user.user_id, permission_str);

        if self.config.enable_permission_cache {
            let cache = self.permission_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    return entry.value.is_allowed();
                }
            }
        }

        if user.is_admin() || user.is_super_admin() {
            return true;
        }

        // Helper to match wildcards (e.g. bot:* matches bot:create)
        // Try parsing the requested permission
        let req_permission = match Permission::from_alias(permission_str) {
            Some(p) => p,
            None => {
                // If it can't be parsed directly to a variant, fall back to string-based wildcard matching
                for role in &user.roles {
                    for user_perm in role.permissions() {
                        if match_wildcard(user_perm.as_alias(), permission_str) {
                            let decision = AccessDecisionResult::allow("Wildcard match");
                            self.cache_result(&cache_key, &decision).await;
                            return true;
                        }
                    }
                }
                let decision = AccessDecisionResult::deny("No match found");
                self.cache_result(&cache_key, &decision).await;
                return false;
            }
        };

        // If it resolved to a Permission enum, check direct user permissions with wildcard matching support
        for role in &user.roles {
            for user_perm in role.permissions() {
                if user_perm == Permission::Admin {
                    let decision = AccessDecisionResult::allow("Admin override");
                    self.cache_result(&cache_key, &decision).await;
                    return true;
                }
                if match_wildcard(user_perm.as_alias(), req_permission.as_alias()) {
                    let decision = AccessDecisionResult::allow("Resolved wildcard match");
                    self.cache_result(&cache_key, &decision).await;
                    return true;
                }
            }
        }

        let decision = AccessDecisionResult::deny("No matching permission found");
        self.cache_result(&cache_key, &decision).await;
        false
    }


 pub fn config(&self) -> &RbacConfig {
 &self.config
 }
}

impl botlib::traits::RbacService for RbacManager {
 fn check_permission(
 &self,
 _user_id: uuid::Uuid,
 _resource: &str,
 _action: &str,
 ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>> {
 let default_deny = self.config.default_deny;
 Box::pin(async move { Ok(!default_deny) })
 }

 fn register_routes(
 &self,
 _default_permissions: serde_json::Value,
 ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
 Box::pin(async { Ok(()) })
 }
}
