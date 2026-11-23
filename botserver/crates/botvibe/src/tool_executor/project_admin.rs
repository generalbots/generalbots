//! #2751 — shared acting-user stamping for project-admin tools.
//!
//! Every execution path (agent loop, direct REST call, deploy pipeline and
//! stored-run replay) stamps `on_behalf_of_user` from the authenticated
//! principal before a project-mutating tool runs. The tool handlers use that
//! field for their project-Admin RBAC check. A client-supplied value is always
//! overwritten here, so it can never be used to impersonate an admin.

use serde_json::Value;
use uuid::Uuid;

/// Tools that mutate a project's deployment surface and therefore require
/// project-Admin RBAC. Each one must declare `on_behalf_of_user` in its schema,
/// otherwise argument validation rejects the server-injected key.
pub(crate) const PROJECT_ADMIN_TOOLS: &[&str] = &[
    "publish/project",
    "domain/bind",
    "domain/security",
    "domain/tls",
    "domain/unbind",
];

pub(crate) fn is_project_admin_tool(tool_name: &str) -> bool {
    PROJECT_ADMIN_TOOLS.contains(&tool_name)
}

/// Stamp the trusted acting user id into a project-admin tool's arguments.
/// Non-admin tools are left untouched. Callers must pass the authenticated
/// user id — never a value taken from the tool payload.
pub(crate) fn stamp_acting_user(tool_name: &str, args: &mut Value, user_id: Uuid) {
    if !is_project_admin_tool(tool_name) {
        return;
    }
    if let Some(obj) = args.as_object_mut() {
        obj.insert(
            "on_behalf_of_user".to_string(),
            Value::String(user_id.to_string()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_overwrites_a_client_supplied_acting_user() {
        let real = Uuid::new_v4();
        let forged = Uuid::new_v4();
        let mut args = serde_json::json!({
            "on_behalf_of_user": forged.to_string(),
            "domain": "shop.example.com",
        });

        stamp_acting_user("domain/bind", &mut args, real);

        assert_eq!(
            args.get("on_behalf_of_user").and_then(Value::as_str),
            Some(real.to_string().as_str())
        );
        assert!(is_project_admin_tool("domain/bind"));
        assert!(is_project_admin_tool("publish/project"));
        assert!(!is_project_admin_tool("domain/verify"));
    }

    #[test]
    fn stamp_leaves_non_admin_tools_untouched() {
        let user = Uuid::new_v4();
        let mut args = serde_json::json!({ "domain": "shop.example.com" });
        stamp_acting_user("domain/verify", &mut args, user);
        assert_eq!(args, serde_json::json!({ "domain": "shop.example.com" }));
    }
}
