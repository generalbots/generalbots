//! Default route permissions for authenticated users.
//!
//! The table used to be a single 517-line file. It is now composed per route
//! family — applications, Vibe, the remaining authenticated endpoints and the
//! HTMX UI fragments — and concatenated in the original order, so the resulting
//! permission list is unchanged.

mod app_routes;
mod rest_routes;
mod ui;
mod vibe_routes;

use super::RoutePermission;

pub(crate) fn authenticated_routes() -> Vec<RoutePermission> {
    let mut routes = Vec::new();
    routes.extend(app_routes::routes());
    routes.extend(vibe_routes::routes());
    routes.extend(rest_routes::routes());
    routes
}

pub(crate) fn ui_routes() -> Vec<RoutePermission> {
    ui::routes()
}
