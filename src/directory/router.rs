
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::shared::state::AppState;

use super::groups;
use super::users;



pub fn configure() -> Router<Arc<AppState>> {
    Router::new()



        .route("/users/create", post(users::create_user))
        .route("/users/{user_id}/update", put(users::update_user))
        .route("/users/{user_id}/delete", delete(users::delete_user))
        .route("/users/list", get(users::list_users))
        .route("/users/search", get(users::list_users))
        .route("/users/{user_id}/profile", get(users::get_user_profile))
        .route("/users/{user_id}/profile/update", put(users::update_user))
        .route("/users/{user_id}/settings", get(users::get_user_profile))
        .route("/users/{user_id}/permissions", get(users::get_user_profile))
        .route("/users/{user_id}/roles", get(users::get_user_profile))
        .route("/users/{user_id}/status", get(users::get_user_profile))
        .route("/users/{user_id}/presence", get(users::get_user_profile))
        .route("/users/{user_id}/activity", get(users::get_user_profile))
        .route(
            "/users/{user_id}/security/2fa/enable",
            post(users::get_user_profile),
        )
        .route(
            "/users/{user_id}/security/2fa/disable",
            post(users::get_user_profile),
        )
        .route(
            "/users/:user_id/security/devices",
            get(users::get_user_profile),
        )
        .route(
            "/users/:user_id/security/sessions",
            get(users::get_user_profile),
        )
        .route(
            "/users/{user_id}/notifications/preferences/update",
            get(users::get_user_profile),
        )



        .route("/groups/create", post(groups::create_group))
        .route("/groups/{group_id}/update", put(groups::update_group))
        .route("/groups/{group_id}/delete", delete(groups::delete_group))
        .route("/groups/list", get(groups::list_groups))
        .route("/groups/search", get(groups::list_groups))
        .route("/groups/{group_id}/members", get(groups::get_group_members))
        .route(
            "/groups/{group_id}/members/add",
            post(groups::add_group_member),
        )
        .route(
            "/groups/{group_id}/members/roles",
            post(groups::remove_group_member),
        )
        .route(
            "/groups/{group_id}/permissions",
            get(groups::get_group_members),
        )
        .route(
            "/groups/{group_id}/settings",
            get(groups::get_group_members),
        )
        .route(
            "/groups/{group_id}/analytics",
            get(groups::get_group_members),
        )
        .route(
            "/groups/:group_id/join/request",
            post(groups::add_group_member),
        )
        .route(
            "/groups/:group_id/join/approve",
            post(groups::add_group_member),
        )
        .route(
            "/groups/:group_id/join/reject",
            post(groups::remove_group_member),
        )
        .route(
            "/groups/:group_id/invites/send",
            post(groups::add_group_member),
        )
        .route(
            "/groups/:group_id/invites/list",
            get(groups::get_group_members),
        )
}
