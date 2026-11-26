use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

use super::auth::{AuthenticatedUser, Permission, PublicPathAllowed, Role};

// #1370 — this module used to be a 2219-line file. It is split by
// responsibility: types (config, decisions, wildcard matching), manager
// (RbacManager + its RbacService impl), middleware (HTTP middleware and
// extractors), routes_public/routes_user/routes_admin (the default route
// permission table, one function per documented section) and tests.
// The public surface is unchanged.
mod admin_group;
mod manager;
mod middleware;
mod routes_admin;
mod routes_public;
mod routes_user;
mod types;

#[cfg(test)]
mod admin_shaped_routes_tests;
#[cfg(test)]
mod cache_invalidation_tests;
#[cfg(test)]
mod ops_routes_tests;
#[cfg(test)]
mod routes_billing_tests;
// #2959 — handoff catalogue decisions live next to the route table without
// pushing `routes_admin.rs` past the 450-line limit.
#[cfg(test)]
#[path = "routes_handoff_tests.rs"]
mod routes_handoff_tests;
// #2964 — POS catalogue decisions live in their own file so the route table
// stays below the 450-line limit while the products/orders contract is locked.
#[cfg(test)]
#[path = "routes_pos_tests.rs"]
mod routes_pos_tests;
#[cfg(test)]
mod tests;

// `manager.rs` holds only the RbacManager impl blocks, so there is nothing to
// re-export from it.
pub use admin_group::{is_admin_group_name, ADMIN_GROUP_NAMES};
pub use middleware::*;
pub use types::*;

// The route table was a single 877-line function; it is now composed from one
// function per section. The order of the resulting permissions is unchanged.
pub fn build_default_route_permissions() -> Vec<RoutePermission> {
    let mut routes = Vec::new();
    routes.extend(routes_public::anonymous_routes());
    routes.extend(routes_user::authenticated_routes());
    routes.extend(routes_user::ui_routes());
    routes.extend(routes_admin::admin_routes());
    routes.extend(routes_admin::rbac_self_service_routes());
    routes.extend(routes_admin::super_admin_routes());
    routes
}
