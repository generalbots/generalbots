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
use diesel::sql_types::{Bool, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    warn!("collab error ({}): {}", status.as_u16(), msg);
    (status, Json(serde_json::json!({ "error": msg })))
}

fn collab_user_id(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.clone();
        }
    }
    if user.user_id.is_nil() {
        "default".to_string()
    } else {
        user.user_id.to_string()
    }
}

fn collab_user_name(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.split('@').next().unwrap_or(email).to_string();
        }
    }
    if !user.username.is_empty() {
        user.username.clone()
    } else {
        "User".to_string()
    }
}

/// Extract `@mention` tokens from a comment body. Tokens are `@` followed by
/// 2+ word characters; trailing punctuation is stripped.
fn extract_mentions(body: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    for token in body.split_whitespace() {
        if let Some(rest) = token.strip_prefix('@') {
            let cleaned: String = rest
                .trim_end_matches(|c: char| c.is_ascii_punctuation())
                .to_string();
            if cleaned.chars().count() >= 2 {
                mentions.push(cleaned);
            }
        }
    }
    mentions
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CommentQuery {
    pub resource_type: String,
    pub resource_id: String,
    /// When true, also match child resources (resource_type + ":" and
    /// resource_id + ":") so a document-level view aggregates anchored
    /// comments — e.g. every `sheet:cell` comment under its sheet.
    #[serde(default)]
    pub include_children: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentBody {
    pub resource_type: String,
    pub resource_id: String,
    pub body: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReactionBody {
    pub emoji: String,
}

#[derive(Debug, Deserialize)]
pub struct PresenceBody {
    pub resource_type: String,
    pub resource_id: String,
    pub typing: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ReactionItem {
    pub emoji: String,
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct CommentItem {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub author_id: String,
    pub author_name: String,
    pub parent_id: Option<String>,
    pub body: String,
    pub mentions: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub reactions: Vec<ReactionItem>,
    pub replies: Vec<CommentItem>,
}

#[derive(Debug, Serialize)]
pub struct PresenceItem {
    pub user_id: String,
    pub user_name: String,
    pub typing: bool,
    pub last_seen: String,
}

#[derive(QueryableByName)]
struct CommentRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    resource_type: String,
    #[diesel(sql_type = Text)]
    resource_id: String,
    #[diesel(sql_type = Text)]
    author_id: String,
    #[diesel(sql_type = Text)]
    author_name: String,
    #[diesel(sql_type = Nullable<Text>)]
    parent_id: Option<String>,
    #[diesel(sql_type = Text)]
    body: String,
    #[diesel(sql_type = Text)]
    mentions: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Timestamptz)]
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(QueryableByName)]
struct ReactionRow {
    #[diesel(sql_type = Text)]
    comment_id: String,
    #[diesel(sql_type = Text)]
    user_id: String,
    #[diesel(sql_type = Text)]
    emoji: String,
}

#[derive(QueryableByName)]
struct PresenceRow {
    #[diesel(sql_type = Text)]
    user_id: String,
    #[diesel(sql_type = Text)]
    user_name: String,
    #[diesel(sql_type = Bool)]
    typing: bool,
    #[diesel(sql_type = Timestamptz)]
    last_seen: chrono::DateTime<chrono::Utc>,
}

fn sanitize_resource(ty: &str, id: &str) -> bool {
    !ty.trim().is_empty()
        && ty.len() <= 64
        && !id.trim().is_empty()
        && id.len() <= 255
        && ty.chars().all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '-' || c == '_')
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/collab/comments?resource_type=&resource_id=` — threaded list
/// (top-level comments with inline replies), excluding soft-deleted rows.
pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
) -> Result<Json<Vec<CommentItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    // When children are not requested, the LIKE patterns bind as the empty
    // string (which matches nothing — resource_type/resource_id are never
    // empty), so the OR branch is inert and the query behaves like an exact
    // match. Binding the same four parameters in both cases keeps the two
    // diesel bind chains type-compatible.
    let ty_prefix = if params.include_children {
        format!("{}:%", params.resource_type)
    } else {
        String::new()
    };
    let id_prefix = if params.include_children {
        format!("{}:%", params.resource_id)
    } else {
        String::new()
    };

    let rows = diesel::sql_query(
        "SELECT id::text, resource_type, resource_id, author_id, author_name, \
                parent_id::text, body, mentions, created_at, updated_at \
         FROM collab_comments \
         WHERE deleted = FALSE \
           AND ((resource_type = $1 AND resource_id = $2) \
             OR (resource_type LIKE $3 AND resource_id LIKE $4)) \
         ORDER BY created_at ASC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Text, _>(&ty_prefix)
    .bind::<Text, _>(&id_prefix)
    .load::<CommentRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let reaction_rows = diesel::sql_query(
        "SELECT comment_id::text, user_id, emoji FROM collab_comment_reactions \
         WHERE comment_id IN (SELECT id FROM collab_comments \
                              WHERE deleted = FALSE \
                                AND ((resource_type = $1 AND resource_id = $2) \
                                  OR (resource_type LIKE $3 AND resource_id LIKE $4)))",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Text, _>(&ty_prefix)
    .bind::<Text, _>(&id_prefix)
    .load::<ReactionRow>(&mut conn)
    .unwrap_or_default();

    let mut reactions: std::collections::HashMap<String, Vec<ReactionItem>> =
        std::collections::HashMap::new();
    for r in reaction_rows {
        reactions
            .entry(r.comment_id)
            .or_default()
            .push(ReactionItem { emoji: r.emoji, user_id: r.user_id });
    }

    let mut top: Vec<CommentItem> = Vec::new();
    let mut replies: Vec<CommentItem> = Vec::new();
    for r in rows {
        let item = CommentItem {
            id: r.id.clone(),
            resource_type: r.resource_type,
            resource_id: r.resource_id,
            author_id: r.author_id,
            author_name: r.author_name,
            parent_id: r.parent_id.clone(),
            body: r.body,
            mentions: serde_json::from_str(&r.mentions).unwrap_or_default(),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
            reactions: reactions.remove(&r.id).unwrap_or_default(),
            replies: Vec::new(),
        };
        if r.parent_id.is_some() {
            replies.push(item);
        } else {
            top.push(item);
        }
    }

    let mut reply_map: std::collections::HashMap<String, Vec<CommentItem>> =
        std::collections::HashMap::new();
    for reply in replies {
        if let Some(parent) = reply.parent_id.clone() {
            reply_map.entry(parent).or_default().push(reply);
        }
    }
    for comment in &mut top {
        if let Some(children) = reply_map.remove(&comment.id) {
            comment.replies = children;
        }
    }

    Ok(Json(top))
}

/// `POST /api/collab/comments` — create a comment (or reply via parent_id).
/// `@mention` tokens are extracted and stored for notification/rendering.
pub async fn create_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateCommentBody>,
) -> Result<Json<CommentItem>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let body = req.body.trim().to_string();
    if body.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Comment body is required"));
    }
    if body.chars().count() > 8000 {
        return Err(err(StatusCode::BAD_REQUEST, "Comment too long (max 8000 chars)"));
    }

    let author_id = collab_user_id(&user);
    let author_name = collab_user_name(&user);
    let mentions = extract_mentions(&body);
    let mentions_json = serde_json::to_string(&mentions).unwrap_or_else(|_| "[]".to_string());
    let parent_id: Option<uuid::Uuid> = match req.parent_id.as_deref() {
        None | Some("") => None,
        Some(p) => match uuid::Uuid::parse_str(p) {
            Ok(id) => Some(id),
            Err(_) => return Err(err(StatusCode::BAD_REQUEST, "Invalid parent_id")),
        },
    };

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct CreatedRow {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let created = diesel::sql_query(
        "INSERT INTO collab_comments \
         (resource_type, resource_id, author_id, author_name, parent_id, body, mentions) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id::text, created_at",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&author_id)
    .bind::<Text, _>(&author_name)
    .bind::<Nullable<SqlUuid>, _>(parent_id)
    .bind::<Text, _>(&body)
    .bind::<Text, _>(&mentions_json)
    .get_result::<CreatedRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("insert failed: {e}")))?;

    info!("collab comment created by {author_id} on {}:{}", req.resource_type, req.resource_id);
    if !mentions.is_empty() {
        info!("collab mentions: {:?}", mentions);
    }

    Ok(Json(CommentItem {
        id: created.id,
        resource_type: req.resource_type,
        resource_id: req.resource_id,
        author_id,
        author_name,
        parent_id: req.parent_id,
        body,
        mentions,
        created_at: created.created_at.to_rfc3339(),
        updated_at: created.created_at.to_rfc3339(),
        reactions: Vec::new(),
        replies: Vec::new(),
    }))
}

/// `DELETE /api/collab/comments/:id` — soft-delete (author or admin only).
pub async fn delete_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let changed = if user.is_admin() || user.is_super_admin() {
        diesel::sql_query("UPDATE collab_comments SET deleted = TRUE WHERE id = $1")
            .bind::<SqlUuid, _>(id)
            .execute(&mut conn)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    } else {
        diesel::sql_query(
            "UPDATE collab_comments SET deleted = TRUE WHERE id = $1 AND author_id = $2",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    };

    if changed == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Comment not found"));
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

/// `POST /api/collab/comments/:id/reactions` — toggle an emoji reaction
/// (adds if absent, removes if the same user already reacted with it).
pub async fn toggle_reaction(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<ReactionBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let emoji = req.emoji.trim().to_string();
    if emoji.is_empty() || emoji.chars().count() > 8 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid emoji"));
    }
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let exists = diesel::sql_query(
        "SELECT 1 FROM collab_comment_reactions WHERE comment_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(&uid)
    .bind::<Text, _>(&emoji)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let added = if exists == 0 {
        diesel::sql_query(
            "INSERT INTO collab_comment_reactions (comment_id, user_id, emoji) VALUES ($1, $2, $3)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .bind::<Text, _>(&emoji)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;
        true
    } else {
        diesel::sql_query(
            "DELETE FROM collab_comment_reactions WHERE comment_id = $1 AND user_id = $2 AND emoji = $3",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .bind::<Text, _>(&emoji)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;
        false
    };

    Ok(Json(serde_json::json!({ "success": true, "added": added })))
}

/// `POST /api/collab/presence` — heartbeat with optional typing flag.
pub async fn update_presence(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<PresenceBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let uid = collab_user_id(&user);
    let name = collab_user_name(&user);
    let typing = req.typing.unwrap_or(false);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    diesel::sql_query(
        "INSERT INTO collab_presence (resource_type, resource_id, user_id, user_name, last_seen, typing) \
         VALUES ($1, $2, $3, $4, NOW(), $5) \
         ON CONFLICT (resource_type, resource_id, user_id) \
         DO UPDATE SET last_seen = NOW(), typing = $5, user_name = $4",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&uid)
    .bind::<Text, _>(&name)
    .bind::<Bool, _>(typing)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// `GET /api/collab/presence?resource_type=&resource_id=` — who is active
/// (heartbeat within the last 60 seconds) on the resource.
pub async fn list_presence(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
) -> Result<Json<Vec<PresenceItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let rows = diesel::sql_query(
        "SELECT user_id, user_name, typing, last_seen FROM collab_presence \
         WHERE resource_type = $1 AND resource_id = $2 \
           AND last_seen > NOW() - INTERVAL '60 seconds' \
         ORDER BY typing DESC, last_seen DESC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .load::<PresenceRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let items: Vec<PresenceItem> = rows
        .into_iter()
        .map(|r| PresenceItem {
            user_id: r.user_id,
            user_name: r.user_name,
            typing: r.typing,
            last_seen: r.last_seen.to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

pub fn configure_collab_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/collab/comments",
            get(list_comments).post(create_comment),
        )
        .route("/api/collab/comments/:id", delete(delete_comment))
        .route(
            "/api/collab/comments/:id/reactions",
            post(toggle_reaction),
        )
        .route(
            "/api/collab/presence",
            get(list_presence).post(update_presence),
        )
}
