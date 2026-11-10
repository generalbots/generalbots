//! Offline tests for permission inheritance and resource visibility
//! (issue #2075). Every helper here is pure: it works on in-memory maps.

use super::*;

fn set(items: &[&str]) -> HashSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

fn role_hierarchy() -> RoleHierarchy {
    let mut hierarchy = RoleHierarchy::new();
    hierarchy.add_role("owner", "Owner", vec!["*".to_string()], Vec::new(), 100);
    hierarchy.add_role(
        "admin",
        "Admin",
        vec!["bot:*".to_string(), "org:manage".to_string()],
        vec!["owner".to_string()],
        90,
    );
    hierarchy.add_role(
        "member",
        "Member",
        vec!["bot:view".to_string()],
        vec!["admin".to_string()],
        50,
    );
    hierarchy
}

#[test]
fn role_permissions_include_every_ancestor() {
    let mut hierarchy = role_hierarchy();
    let permissions = hierarchy.get_effective_permissions("member");

    assert_eq!(permissions, set(&["bot:view", "bot:*", "org:manage", "*"]));
}

#[test]
fn unknown_role_has_no_permissions() {
    let mut hierarchy = role_hierarchy();
    assert!(hierarchy.get_effective_permissions("ghost").is_empty());
}

#[test]
fn role_cycles_terminate_and_still_collect_both_sides() {
    let mut hierarchy = RoleHierarchy::new();
    hierarchy.add_role("a", "A", vec!["a:perm".to_string()], vec!["b".to_string()], 10);
    hierarchy.add_role("b", "B", vec!["b:perm".to_string()], vec!["a".to_string()], 20);

    assert_eq!(
        hierarchy.get_effective_permissions("a"),
        set(&["a:perm", "b:perm"])
    );
}

#[test]
fn removing_a_role_clears_the_cached_permissions() {
    let mut hierarchy = role_hierarchy();
    assert!(hierarchy
        .get_effective_permissions("member")
        .contains("bot:view"));

    hierarchy.remove_role("member");

    assert!(hierarchy.get_effective_permissions("member").is_empty());
}

#[test]
fn management_follows_the_hierarchy_level() {
    let hierarchy = role_hierarchy();

    assert!(hierarchy.can_manage("owner", "admin"));
    assert!(hierarchy.can_manage("admin", "member"));
    assert!(!hierarchy.can_manage("member", "admin"));
    assert!(!hierarchy.can_manage("admin", "admin"), "equal level is not manageable");
    assert!(!hierarchy.can_manage("ghost", "admin"));
}

#[test]
fn manageable_roles_are_the_strictly_lower_levels() {
    let hierarchy = role_hierarchy();
    let mut manageable = hierarchy.get_manageable_roles("admin");
    manageable.sort();

    assert_eq!(manageable, vec!["member".to_string()]);
}

fn group_hierarchy() -> GroupHierarchy {
    let mut hierarchy = GroupHierarchy::new();
    hierarchy.add_group("everyone", vec!["basic:access".to_string()], None);
    hierarchy.add_group(
        "developers",
        vec!["bot:edit".to_string()],
        Some("everyone".to_string()),
    );
    hierarchy.add_group(
        "backend",
        vec!["db:write".to_string()],
        Some("developers".to_string()),
    );
    hierarchy
}

#[test]
fn group_permissions_include_the_parent_chain() {
    let mut hierarchy = group_hierarchy();
    assert_eq!(
        hierarchy.get_effective_permissions("backend"),
        set(&["db:write", "bot:edit", "basic:access"])
    );
}

#[test]
fn group_ancestors_and_descendants_walk_the_chain() {
    let hierarchy = group_hierarchy();

    assert_eq!(
        hierarchy.get_ancestors("backend"),
        vec!["developers".to_string(), "everyone".to_string()]
    );
    assert_eq!(
        hierarchy.get_ancestors("everyone"),
        Vec::<String>::new()
    );

    let mut descendants = hierarchy.get_descendants("everyone");
    descendants.sort();
    assert_eq!(
        descendants,
        vec!["backend".to_string(), "developers".to_string()]
    );
}

#[test]
fn group_cycles_terminate_instead_of_looping_forever() {
    let mut hierarchy = GroupHierarchy::new();
    hierarchy.add_group("a", vec!["a:perm".to_string()], None);
    hierarchy.add_group("b", vec!["b:perm".to_string()], Some("a".to_string()));
    // Close the cycle: a becomes a child of b.
    hierarchy.add_group("a", vec!["a:perm".to_string()], Some("b".to_string()));

    assert_eq!(hierarchy.get_ancestors("a"), vec!["b".to_string()]);
    assert_eq!(hierarchy.get_descendants("a"), vec!["b".to_string()]);
}

#[test]
fn a_group_declared_before_its_parent_is_still_a_descendant() {
    let mut hierarchy = GroupHierarchy::new();
    hierarchy.add_group("child", vec!["c:perm".to_string()], Some("parent".to_string()));
    hierarchy.add_group("parent", vec!["p:perm".to_string()], None);

    assert_eq!(hierarchy.get_descendants("parent"), vec!["child".to_string()]);
    assert_eq!(
        hierarchy.get_effective_permissions("child"),
        set(&["c:perm", "p:perm"])
    );
}

#[test]
fn resolver_merges_direct_role_and_group_permissions() {
    let user = Uuid::from_u128(42);
    let mut resolver = create_default_resolver();
    resolver.assign_role_to_user(user, "member");
    resolver.add_user_to_group(user, "developers");
    resolver.add_direct_permission(user, "report:export");

    let effective = resolver.get_effective_permissions(user);

    assert!(effective.permissions.contains("report:export"));
    assert!(effective.permissions.contains("bot:view"));
    assert!(effective.permissions.contains("basic:access"));
    assert_eq!(effective.user_id, user);
    assert!(effective
        .sources
        .iter()
        .any(|s| s.source_type == PermissionSourceType::Direct));
    assert!(effective
        .sources
        .iter()
        .any(|s| s.source_type == PermissionSourceType::Role));
    assert!(effective
        .sources
        .iter()
        .any(|s| s.source_type == PermissionSourceType::Group));
}

#[test]
fn check_permission_honours_exact_wildcard_and_prefix_rules() {
    let user = Uuid::from_u128(43);
    let mut resolver = create_default_resolver();
    resolver.assign_role_to_user(user, "member");

    assert!(resolver.check_permission(user, "bot:view").allowed);
    // member inherits manager -> admin -> owner, so "*" is present.
    let wildcard = resolver.check_permission(user, "anything:at:all");
    assert!(wildcard.allowed);
    assert_eq!(
        wildcard.source.as_ref().map(|s| s.source_name.as_str()),
        Some("wildcard")
    );
}

#[test]
fn prefix_wildcards_allow_their_whole_namespace_only() {
    let user = Uuid::from_u128(44);
    let mut resolver = PermissionInheritanceResolver::new();
    resolver.add_direct_permission(user, "bot:*");

    assert!(resolver.check_permission(user, "bot:edit").allowed);
    assert!(resolver.check_permission(user, "bot:kb:edit").allowed);
    assert!(!resolver.check_permission(user, "org:manage").allowed);
}

#[test]
fn removing_assignments_revokes_the_permission() {
    let user = Uuid::from_u128(45);
    let mut resolver = PermissionInheritanceResolver::new();
    resolver.assign_role_to_user(user, "member");
    resolver.add_user_to_group(user, "developers");
    resolver.add_direct_permission(user, "report:export");

    resolver.remove_role_from_user(user, "member");
    resolver.remove_user_from_group(user, "developers");
    resolver.remove_direct_permission(user, "report:export");

    assert!(!resolver.check_permission(user, "bot:view").allowed);
    assert!(!resolver.check_permission(user, "report:export").allowed);
    assert!(resolver.get_user_roles(user).is_empty());
    assert!(resolver.get_user_groups(user).is_empty());
}

fn resource(visibility: ResourceVisibility) -> ResourcePermission {
    ResourcePermission {
        resource_type: "bot".to_string(),
        resource_id: Uuid::from_u128(1),
        organization_id: Uuid::from_u128(2),
        owner_id: Uuid::from_u128(100),
        visibility,
        allowed_roles: Vec::new(),
        allowed_groups: Vec::new(),
        allowed_users: Vec::new(),
        denied_users: Vec::new(),
    }
}

#[test]
fn check_access_grants_the_owner_and_explicitly_allowed_users() {
    let owner = Uuid::from_u128(100);
    let allowed = Uuid::from_u128(101);
    let mut permission = resource(ResourceVisibility::Private);
    permission.allowed_users.push(allowed);

    assert!(permission.check_access(owner, &[], &[], true));
    assert!(permission.check_access(allowed, &[], &[], true));
    assert!(!permission.check_access(Uuid::from_u128(102), &[], &[], true));
}

#[test]
fn check_access_lets_a_denial_override_every_grant() {
    let owner = Uuid::from_u128(100);
    let mut permission = resource(ResourceVisibility::Public);
    permission.denied_users.push(owner);

    assert!(!permission.check_access(owner, &["admin".to_string()], &[], true));
}

#[test]
fn check_access_requires_authentication_for_organization_visibility() {
    let user = Uuid::from_u128(101);
    let permission = resource(ResourceVisibility::Organization);

    assert!(permission.check_access(user, &[], &[], true));
    assert!(!permission.check_access(user, &[], &[], false));
}

#[test]
fn check_access_enforces_role_and_group_restrictions() {
    let user = Uuid::from_u128(101);
    let mut permission = resource(ResourceVisibility::Organization);
    permission.allowed_roles.push("manager".to_string());
    permission.allowed_groups.push("developers".to_string());

    assert!(permission.check_access(
        user,
        &["manager".to_string()],
        &["developers".to_string()],
        true
    ));
    assert!(!permission.check_access(user, &["member".to_string()], &["developers".to_string()], true));
    assert!(!permission.check_access(user, &["manager".to_string()], &["support".to_string()], true));
}

#[test]
fn private_visibility_never_grants_access_implicitly() {
    let permission = resource(ResourceVisibility::Private);
    assert!(!permission.check_access(Uuid::from_u128(101), &["admin".to_string()], &[], true));
}
