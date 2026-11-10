//! #2069 — the workspace domain vocabulary (roles, icons, widths, settings) and
//! the error→HTTP mapping. Persisted as text, so the labels are a contract.
use super::*;

#[test]
fn workspace_roles_round_trip_through_their_labels() {
    for role in [
        WorkspaceRole::Owner,
        WorkspaceRole::Admin,
        WorkspaceRole::Editor,
        WorkspaceRole::Commenter,
        WorkspaceRole::Viewer,
    ] {
        assert_eq!(WorkspaceRole::from_label(role.as_str()), role);
        let json = serde_json::to_string(&role).expect("serialize");
        assert_eq!(json, format!("\"{}\"", role.as_str()));
        assert_eq!(serde_json::from_str::<WorkspaceRole>(&json).expect("deserialize"), role);
    }
}

#[test]
fn unknown_labels_fall_back_to_the_least_privileged_role() {
    assert_eq!(WorkspaceRole::from_label(""), WorkspaceRole::Viewer);
    assert_eq!(WorkspaceRole::from_label("OWNER"), WorkspaceRole::Viewer);
    assert_eq!(WorkspaceRole::from_label("superuser"), WorkspaceRole::Viewer);
    assert_eq!(IconType::from_label("gif"), IconType::Emoji);
}

#[test]
fn icon_types_round_trip() {
    for icon in [IconType::Emoji, IconType::Image, IconType::Lucide] {
        assert_eq!(IconType::from_label(icon.as_str()), icon);
        assert_eq!(serde_json::to_string(&icon).expect("serialize"), format!("\"{}\"", icon.as_str()));
    }
}

#[test]
fn page_width_defaults_to_normal_and_serializes_snake_case() {
    assert_eq!(PageWidth::default(), PageWidth::Normal);
    assert_eq!(serde_json::to_string(&PageWidth::Small).expect("serialize"), "\"small\"");
    assert_eq!(serde_json::to_string(&PageWidth::Full).expect("serialize"), "\"full\"");
}

#[test]
fn workspace_settings_defaults_match_an_empty_json_object() {
    let settings: WorkspaceSettings = serde_json::from_str("{}").expect("deserialize");
    assert_eq!(settings.default_page_width, PageWidth::Normal);
    assert!(!settings.allow_public_pages);
    assert!(settings.enable_comments);
    assert!(settings.enable_reactions);
    assert!(settings.enable_gb_assist);
    assert_eq!(settings.gb_bot_id, None);
}

#[test]
fn page_permissions_inherit_by_default_when_deserialized() {
    let permissions: PagePermissions = serde_json::from_str("{}").expect("deserialize");
    assert!(permissions.inherit_from_parent);
    assert!(!permissions.public);
    assert!(!permissions.public_edit);
    assert!(permissions.allowed_users.is_empty());
    assert!(permissions.allowed_roles.is_empty());

    let shared: PagePermissions =
        serde_json::from_str(r#"{"public":true,"allowed_roles":["editor"]}"#).expect("deserialize");
    assert!(shared.public);
    assert_eq!(shared.allowed_roles, vec![WorkspaceRole::Editor]);
}

#[test]
fn every_error_has_a_message_and_the_documented_status() {
    use axum::response::IntoResponse;

    let cases: Vec<(WorkspacesError, StatusCode, &str)> = vec![
        (WorkspacesError::WorkspaceNotFound, StatusCode::NOT_FOUND, "Workspace not found"),
        (WorkspacesError::PageNotFound, StatusCode::NOT_FOUND, "Page not found"),
        (WorkspacesError::BlockNotFound, StatusCode::NOT_FOUND, "Block not found"),
        (WorkspacesError::CommentNotFound, StatusCode::NOT_FOUND, "Comment not found"),
        (WorkspacesError::VersionNotFound, StatusCode::NOT_FOUND, "Version not found"),
        (WorkspacesError::MemberNotFound, StatusCode::NOT_FOUND, "Member not found"),
        (WorkspacesError::PermissionDenied, StatusCode::FORBIDDEN, "Permission denied"),
        (
            WorkspacesError::MemberAlreadyExists,
            StatusCode::BAD_REQUEST,
            "Member already exists in workspace",
        ),
        (
            WorkspacesError::CannotRemoveLastOwner,
            StatusCode::BAD_REQUEST,
            "Cannot remove the last owner",
        ),
        (
            WorkspacesError::InvalidOperation("bad block".to_string()),
            StatusCode::BAD_REQUEST,
            "Invalid operation: bad block",
        ),
        (
            // `Display` keeps the driver detail for logs; the HTTP body is
            // sanitized (see the test below).
            WorkspacesError::DbError("connection reset".to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error: connection reset",
        ),
    ];

    for (error, status, message) in cases {
        assert_eq!(error.to_string(), message);
        assert_eq!(error.into_response().status(), status, "{message}");
    }
}

#[tokio::test]
async fn database_errors_never_leak_the_driver_message_in_the_http_body() {
    use axum::response::IntoResponse;

    let error = WorkspacesError::DbError("password authentication failed for user gb".to_string());
    // `Display` is the log line: it keeps the detail on purpose.
    assert!(error.to_string().contains("password authentication"));

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .expect("body");
    let body = String::from_utf8_lossy(&bytes);
    assert!(!body.contains("password authentication"), "{body}");
    assert!(body.contains("Database error"), "{body}");
}

#[test]
fn template_errors_carry_their_own_messages() {
    assert_eq!(TemplateError::TemplateNotFound.to_string(), "Template not found");
    assert_eq!(
        TemplateError::CannotModifySystemTemplate.to_string(),
        "Cannot modify system template"
    );
    assert_eq!(
        TemplateError::InvalidTemplate("empty".to_string()).to_string(),
        "Invalid template: empty"
    );
}

#[test]
fn collaboration_errors_carry_their_own_messages() {
    assert_eq!(
        CollaborationError::SessionNotFound.to_string(),
        "Collaboration session not found"
    );
    assert_eq!(
        CollaborationError::UserNotInSession.to_string(),
        "User is not in the session"
    );
    assert_eq!(
        CollaborationError::BroadcastError("closed".to_string()).to_string(),
        "Broadcast error: closed"
    );
    assert_eq!(
        CollaborationError::InvalidOperation("stale".to_string()).to_string(),
        "Invalid operation: stale"
    );
}
