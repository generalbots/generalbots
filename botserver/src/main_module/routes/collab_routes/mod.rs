// Cross-app collaboration API — threaded comments with @-mentions and emoji
// reactions, plus presence (viewing/typing) on any resource. Resources are
// addressed generically (resource_type + resource_id) so drive files, sheets,
// docs, tasks and calendar events all share one layer.
//
// All endpoints are authenticated (JWT via the platform auth middleware,
// which inserts `AuthenticatedUser` before handlers run).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use botcore::shared::state::AppState;
use crate::security::auth_api::types::AuthenticatedUser;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::PgConnection;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

// #1370 — this module used to be a 2204-line file. It is split by
// responsibility: helpers (identity/role/activity plumbing), dto (request and
// response payloads), comments, engagement (reactions, resolve, presence,
// activity), versions and permissions. Items that handlers share are
// re-exported here; the route table stays in this root.
mod helpers;
mod dto;
mod comments;
mod engagement;
mod versions;
mod permissions;

pub(crate) use helpers::*;
pub use dto::*;
pub use comments::*;
pub use engagement::*;
pub use versions::*;
pub use permissions::*;

pub fn configure_collab_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/activity",
            get(list_activity).post(record_activity_event),
        )
        .route(
            "/api/collab/comments",
            get(list_comments).post(create_comment),
        )
        .route("/api/collab/comments/:id", delete(delete_comment))
        .route(
            "/api/collab/comments/:id/resolve",
            post(resolve_comment),
        )
        .route(
            "/api/collab/comments/:id/reactions",
            post(toggle_reaction),
        )
        .route("/api/collab/comments/read", post(mark_comments_read))
        .route("/api/collab/comments/unread", get(unread_count))
        .route("/api/collab/mentions/me", get(my_mentions))
        .route(
            "/api/collab/permissions",
            get(list_permissions).post(grant_permission).delete(revoke_permission),
        )
        .route("/api/collab/permissions/transfer", post(transfer_ownership))
        .route("/api/collab/permissions/me", get(my_role))
        .route(
            "/api/collab/links",
            get(list_links).post(create_link),
        )
        .route("/api/collab/links/:token", delete(revoke_link))
        .route(
            "/api/collab/presence",
            get(list_presence).post(update_presence),
        )
        .route(
            "/api/collab/versions",
            get(list_versions).post(snapshot_version),
        )
        .route("/api/collab/versions/:id", get(get_version))
        .route("/api/collab/versions/:id/restore", post(restore_version))
        .route("/api/collab/versions/:id/name", post(name_version))
        .merge(super::collab_ops::configure_collab_ops_routes())
}
