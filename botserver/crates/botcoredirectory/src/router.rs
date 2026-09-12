use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::groups;
use super::organizations;
use super::users;

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        // --- Organizations ---
        .route("/organizations/list", get(organizations::list_organizations))
        .route("/organizations/create", post(organizations::create_organization))
        .route("/organizations/:org_id", get(organizations::get_organization))
        // --- Users CRUD ---
        .route("/users/create", post(users::create_user))
        .route("/users/:user_id/update", put(users::update_user))
        .route("/users/:user_id/delete", delete(users::delete_user))
        .route("/users/list", get(users::list_users))
        .route("/users/search", get(users::list_users))
        .route("/users/:user_id/profile", get(users::get_user_profile))
        .route("/users/:user_id/profile/update", put(users::update_user))
        // --- Users organization ---
        .route(
            "/users/:user_id/organization",
            post(users::assign_organization),
        )
        .route(
            "/users/:user_id/organization/:org_id",
            delete(users::remove_from_organization),
        )
        .route(
            "/users/:user_id/organization/:org_id/roles",
            put(users::update_user_roles),
        )
        .route(
            "/users/:user_id/memberships",
            get(users::get_user_memberships),
        )
        // --- Users settings / permissions / roles ---
        .route("/users/:user_id/settings", get(users::get_user_profile))
        .route("/users/:user_id/permissions", get(users::get_user_permissions))
        .route("/users/:user_id/roles", get(users::get_user_memberships))
        .route("/users/:user_id/status", get(users::get_user_profile))
        .route("/users/:user_id/presence", get(users::get_user_presence))
        .route("/users/:user_id/activity", get(users::get_user_activity))
        // --- Users security ---
        .route(
            "/users/:user_id/security/2fa/enable",
            post(users::enable_2fa),
        )
        .route(
            "/users/:user_id/security/2fa/disable",
            post(users::disable_2fa),
        )
        .route(
            "/users/:user_id/security/devices",
            get(users::get_user_devices),
        )
        .route(
            "/users/:user_id/security/sessions",
            get(users::get_user_sessions),
        )
        // --- Users notifications ---
        .route(
            "/users/:user_id/notifications/preferences/update",
            put(users::update_notification_preferences),
        )
        // --- Groups CRUD ---
        .route("/groups/create", post(groups::create_group))
        .route("/groups/:group_id/update", put(groups::update_group))
        .route("/groups/:group_id/delete", delete(groups::delete_group))
        .route("/groups/list", get(groups::list_groups))
        .route("/groups/search", get(groups::list_groups))
        // --- Groups members ---
        .route("/groups/:group_id/members", get(groups::get_group_members))
        .route(
            "/groups/:group_id/members/add",
            post(groups::add_group_member),
        )
        .route(
            "/groups/:group_id/members/roles",
            post(groups::update_group_member_roles),
        )
        // --- Groups KBs ---
        .route("/groups/:group_id/kbs", get(groups::get_group_kbs))
        .route("/groups/:group_id/kbs/toggle/:kb_id", post(groups::toggle_group_kb))
        // --- Groups info ---
        .route("/groups/:group_id/settings", get(groups::get_group_settings))
        .route(
            "/groups/:group_id/permissions",
            get(groups::get_group_permissions),
        )
        .route(
            "/groups/:group_id/analytics",
            get(groups::get_group_analytics),
        )
        // --- Groups join requests ---
        .route(
            "/groups/:group_id/join/request",
            post(groups::request_join_group),
        )
        .route(
            "/groups/:group_id/join/approve",
            post(groups::approve_join_request),
        )
        .route(
            "/groups/:group_id/join/reject",
            post(groups::reject_join_request),
        )
        // --- Groups invites ---
        .route(
            "/groups/:group_id/invites/send",
            post(groups::send_group_invite),
        )
        .route(
            "/groups/:group_id/invites/list",
            get(groups::list_group_invites),
        )
}
