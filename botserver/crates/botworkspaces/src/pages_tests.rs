//! Page-tree operations: breadcrumbs, depth, ancestor/descendant walks, the
//! move-cycle guard and the permission matrix. These run on every page read and
//! every drag-and-drop, and a wrong answer either leaks a page or lets a user
//! detach a subtree from the workspace.
use super::*;

fn page(id: Uuid, parent_id: Option<Uuid>, children: Vec<Uuid>, title: &str) -> Page {
    Page {
        id,
        workspace_id: Uuid::nil(),
        parent_id,
        title: title.to_string(),
        icon: None,
        cover_image: None,
        blocks: Vec::new(),
        children,
        properties: HashMap::new(),
        permissions: PagePermissions {
            inherit_from_parent: true,
            public: false,
            public_edit: false,
            allowed_users: Vec::new(),
            allowed_roles: Vec::new(),
        },
        is_template: false,
        template_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Uuid::nil(),
        last_edited_by: Uuid::nil(),
    }
}

fn id(n: u8) -> Uuid {
    Uuid::from_u128(u128::from(n))
}

/// root(1) -> child(2) -> grandchild(3), plus a sibling(4) under root.
fn tree() -> HashMap<Uuid, Page> {
    let mut pages = HashMap::new();
    pages.insert(id(1), page(id(1), None, vec![id(2), id(4)], "Root"));
    pages.insert(
        id(2),
        page(id(2), Some(id(1)), vec![id(3)], "Child"),
    );
    pages.insert(id(3), page(id(3), Some(id(2)), Vec::new(), "Grandchild"));
    pages.insert(id(4), page(id(4), Some(id(1)), Vec::new(), "Sibling"));
    pages
}

#[test]
fn breadcrumbs_are_ordered_root_first() {
    let crumbs = build_breadcrumbs(id(3), &tree(), "Handbook", None, Uuid::nil());

    let titles: Vec<&str> = crumbs.path.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(titles, vec!["Root", "Child", "Grandchild"]);
    assert_eq!(crumbs.workspace_name, "Handbook");
}

#[test]
fn breadcrumbs_of_an_unknown_page_are_empty() {
    let crumbs = build_breadcrumbs(id(9), &tree(), "Handbook", None, Uuid::nil());

    assert!(crumbs.path.is_empty());
}

#[test]
fn depth_counts_the_page_itself() {
    let pages = tree();

    assert_eq!(get_page_depth(id(1), &pages), 1);
    assert_eq!(get_page_depth(id(2), &pages), 2);
    assert_eq!(get_page_depth(id(3), &pages), 3);
    assert_eq!(get_page_depth(id(9), &pages), 0);
}

#[test]
fn descendants_are_collected_recursively() {
    let pages = tree();

    assert_eq!(get_all_descendants(id(1), &pages), vec![id(2), id(3), id(4)]);
    assert_eq!(get_all_descendants(id(2), &pages), vec![id(3)]);
    assert!(get_all_descendants(id(3), &pages).is_empty());
}

#[test]
fn ancestors_are_listed_nearest_first() {
    let pages = tree();

    assert_eq!(get_all_ancestors(id(3), &pages), vec![id(2), id(1)]);
    assert!(get_all_ancestors(id(1), &pages).is_empty());
}

#[test]
fn is_descendant_of_follows_the_whole_chain() {
    let pages = tree();

    assert!(is_descendant_of(id(3), id(2), &pages));
    assert!(is_descendant_of(id(3), id(1), &pages));
    assert!(!is_descendant_of(id(4), id(2), &pages));
    assert!(!is_descendant_of(id(1), id(3), &pages));
}

#[test]
fn a_page_cannot_be_moved_into_itself() {
    let error = can_move_page(id(2), Some(id(2)), &tree()).expect_err("must be rejected");

    assert!(error.contains("itself"));
}

#[test]
fn a_page_cannot_be_moved_into_its_own_descendant() {
    let error = can_move_page(id(1), Some(id(3)), &tree()).expect_err("must be rejected");

    assert!(error.contains("descendant"));
}

#[test]
fn a_valid_move_and_a_move_to_the_root_are_allowed() {
    let pages = tree();

    assert!(can_move_page(id(3), Some(id(4)), &pages).is_ok());
    assert!(can_move_page(id(3), None, &pages).is_ok());
}

#[test]
fn owners_and_admins_may_do_anything() {
    let page = page(id(1), None, Vec::new(), "Root");

    for role in [WorkspaceRole::Owner, WorkspaceRole::Admin] {
        for permission in [
            PagePermissionType::View,
            PagePermissionType::Edit,
            PagePermissionType::Comment,
            PagePermissionType::Share,
            PagePermissionType::Delete,
        ] {
            assert!(
                check_page_permission(&page, Uuid::nil(), role, permission),
                "{role:?} must be allowed to {permission:?}"
            );
        }
    }
}

#[test]
fn editors_and_commenters_are_limited_to_their_own_level() {
    let page = page(id(1), None, Vec::new(), "Root");

    assert!(check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Editor,
        PagePermissionType::Edit
    ));
    assert!(!check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Editor,
        PagePermissionType::Share
    ));
    assert!(!check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Editor,
        PagePermissionType::Delete
    ));
    assert!(check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Commenter,
        PagePermissionType::Comment
    ));
    assert!(!check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Commenter,
        PagePermissionType::Edit
    ));
    assert!(check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Viewer,
        PagePermissionType::View
    ));
    assert!(!check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Viewer,
        PagePermissionType::Comment
    ));
}

#[test]
fn a_public_page_grants_view_and_only_public_edit_grants_edit() {
    let mut page = page(id(1), None, Vec::new(), "Public");
    page.permissions.public = true;

    assert!(check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Viewer,
        PagePermissionType::View
    ));
    assert!(!check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Viewer,
        PagePermissionType::Edit
    ));

    page.permissions.public_edit = true;
    assert!(check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Viewer,
        PagePermissionType::Edit
    ));
    assert!(!check_page_permission(
        &page,
        Uuid::nil(),
        WorkspaceRole::Viewer,
        PagePermissionType::Delete
    ));
}

#[test]
fn an_explicit_grant_beats_the_role_matrix() {
    let user = Uuid::from_u128(42);
    let mut page = page(id(1), None, Vec::new(), "Root");
    page.permissions.allowed_users.push(user);

    assert!(check_page_permission(
        &page,
        user,
        WorkspaceRole::Viewer,
        PagePermissionType::Delete
    ));

    let mut by_role = page(id(2), None, Vec::new(), "Role");
    by_role.permissions.allowed_roles.push(WorkspaceRole::Viewer);
    assert!(check_page_permission(
        &by_role,
        Uuid::nil(),
        WorkspaceRole::Viewer,
        PagePermissionType::Share
    ));
}
