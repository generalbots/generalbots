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

/// Append an audit-trail row for a resource. This is fire-and-forget from the
/// mutating handlers — an audit-log failure is logged but never fatal to the
/// primary write it trails.
fn record_activity(
    conn: &mut PgConnection,
    actor_id: &str,
    actor_name: &str,
    resource_type: &str,
    resource_id: &str,
    action: &str,
    payload: &serde_json::Value,
) {
    let payload_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
    let res = diesel::sql_query(
        "INSERT INTO collab_activity \
         (resource_type, resource_id, actor_id, actor_name, action, payload) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<Text, _>(resource_type)
    .bind::<Text, _>(resource_id)
    .bind::<Text, _>(actor_id)
    .bind::<Text, _>(actor_name)
    .bind::<Text, _>(action)
    .bind::<Text, _>(&payload_json)
    .execute(conn);
    if let Err(e) = res {
        warn!("collab activity insert failed: {e}");
    }
}

/// Resolve the (resource_type, resource_id) a comment belongs to, so mutating
/// handlers can write an audit trail against the parent resource.
fn comment_resource(conn: &mut PgConnection, id: uuid::Uuid) -> Option<(String, String)> {
    #[derive(QueryableByName)]
    struct ResRow {
        #[diesel(sql_type = Text)]
        resource_type: String,
        #[diesel(sql_type = Text)]
        resource_id: String,
    }
    diesel::sql_query("SELECT resource_type, resource_id FROM collab_comments WHERE id = $1")
        .bind::<SqlUuid, _>(id)
        .load::<ResRow>(conn)
        .ok()
        .and_then(|mut rows| rows.pop())
        .map(|r| (r.resource_type, r.resource_id))
}

/// SHA-256 hex digest of a snapshot's content, used to dedup unchanged saves
/// and shown as an integrity fingerprint in the version list.
fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
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
    pub resolved: bool,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
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
    #[diesel(sql_type = Bool)]
    resolved: bool,
    #[diesel(sql_type = Nullable<Text>)]
    resolved_by: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    resolved_at: Option<chrono::DateTime<chrono::Utc>>,
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

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    pub resource_type: String,
    pub resource_id: String,
    /// Max rows to return (1..200, default 50).
    #[serde(default)]
    pub limit: Option<i64>,
    /// Cursor: return only events strictly older than this RFC3339 timestamp.
    #[serde(default)]
    pub before: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordActivityBody {
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ActivityItem {
    pub id: String,
    pub actor_id: String,
    pub actor_name: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub created_at: String,
}

#[derive(QueryableByName)]
struct ActivityRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
    #[diesel(sql_type = Text)]
    actor_name: String,
    #[diesel(sql_type = Text)]
    action: String,
    #[diesel(sql_type = Text)]
    payload: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Version history types (#860)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VersionsQuery {
    pub resource_type: String,
    pub resource_id: String,
    /// Max versions to return (1..200, default 50).
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotBody {
    pub resource_type: String,
    pub resource_id: String,
    pub content: String,
    /// Optional milestone label (e.g. "v2 — approved").
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct NameBody {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct VersionItem {
    pub id: String,
    pub actor_id: String,
    pub actor_name: String,
    pub name: String,
    pub content_hash: String,
    pub size: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct VersionDetail {
    pub id: String,
    pub actor_id: String,
    pub actor_name: String,
    pub name: String,
    pub content: String,
    pub content_hash: String,
    pub size: i64,
    pub created_at: String,
}

#[derive(QueryableByName)]
struct VersionListRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
    #[diesel(sql_type = Text)]
    actor_name: String,
    #[diesel(sql_type = Text)]
    content_hash: String,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = BigInt)]
    size: i64,
    #[diesel(sql_type = Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(QueryableByName)]
struct VersionDetailRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    actor_id: String,
    #[diesel(sql_type = Text)]
    actor_name: String,
    #[diesel(sql_type = Text)]
    content: String,
    #[diesel(sql_type = Text)]
    content_hash: String,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(QueryableByName)]
struct RestoreRow {
    #[diesel(sql_type = Text)]
    resource_type: String,
    #[diesel(sql_type = Text)]
    resource_id: String,
    #[diesel(sql_type = Text)]
    content: String,
    #[diesel(sql_type = Text)]
    content_hash: String,
    #[diesel(sql_type = Text)]
    name: String,
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
                parent_id::text, body, mentions, created_at, updated_at, \
                resolved, resolved_by, resolved_at \
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
            resolved: r.resolved,
            resolved_by: r.resolved_by,
            resolved_at: r.resolved_at.map(|d| d.to_rfc3339()),
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

    record_activity(
        &mut conn,
        &author_id,
        &author_name,
        &req.resource_type,
        &req.resource_id,
        "comment",
        &serde_json::json!({ "body_len": body.chars().count(), "reply": req.parent_id.is_some() }),
    );

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
        resolved: false,
        resolved_by: None,
        resolved_at: None,
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

    if let Some((resource_type, resource_id)) = comment_resource(&mut conn, id) {
        record_activity(
            &mut conn,
            &uid,
            &collab_user_name(&user),
            &resource_type,
            &resource_id,
            "delete",
            &serde_json::json!({ "comment_id": id.to_string() }),
        );
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

    if let Some((resource_type, resource_id)) = comment_resource(&mut conn, id) {
        record_activity(
            &mut conn,
            &uid,
            &collab_user_name(&user),
            &resource_type,
            &resource_id,
            "reaction",
            &serde_json::json!({ "comment_id": id.to_string(), "emoji": emoji, "added": added }),
        );
    }

    Ok(Json(serde_json::json!({ "success": true, "added": added })))
}

// ---------------------------------------------------------------------------
// Resolve / read tracking (#863)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ResolveBody {
    pub resolved: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReadBody {
    pub resource_type: String,
    pub resource_id: String,
}

/// `POST /api/collab/comments/:id/resolve` — resolve or reopen a thread.
/// The comment author (or an admin) toggles the resolved state; reopening
/// clears the resolved_by/resolved_at audit fields.
pub async fn resolve_comment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<ResolveBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let resolved_by: Option<String> = if req.resolved { Some(uid.clone()) } else { None };
    let resolved_at: Option<chrono::DateTime<chrono::Utc>> =
        if req.resolved { Some(chrono::Utc::now()) } else { None };

    let changed = if user.is_admin() || user.is_super_admin() {
        diesel::sql_query(
            "UPDATE collab_comments \
             SET resolved = $1, resolved_by = $2, resolved_at = $3, updated_at = NOW() \
             WHERE id = $4",
        )
        .bind::<Bool, _>(req.resolved)
        .bind::<Nullable<Text>, _>(resolved_by)
        .bind::<Nullable<Timestamptz>, _>(resolved_at)
        .bind::<SqlUuid, _>(id)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    } else {
        diesel::sql_query(
            "UPDATE collab_comments \
             SET resolved = $1, resolved_by = $2, resolved_at = $3, updated_at = NOW() \
             WHERE id = $4 AND author_id = $5",
        )
        .bind::<Bool, _>(req.resolved)
        .bind::<Nullable<Text>, _>(resolved_by)
        .bind::<Nullable<Timestamptz>, _>(resolved_at)
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(&uid)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    };

    if changed == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Comment not found or not yours to resolve"));
    }
    info!("collab comment {} set resolved={} by {uid}", id, req.resolved);

    let action = if req.resolved { "resolve" } else { "reopen" };
    if let Some((resource_type, resource_id)) = comment_resource(&mut conn, id) {
        record_activity(
            &mut conn,
            &uid,
            &collab_user_name(&user),
            &resource_type,
            &resource_id,
            action,
            &serde_json::json!({ "comment_id": id.to_string() }),
        );
    }

    Ok(Json(serde_json::json!({ "success": true, "resolved": req.resolved })))
}

/// `POST /api/collab/comments/read` — mark a resource's comments as read up to
/// now, so the unread badge resets when the user opens the panel.
pub async fn mark_comments_read(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ReadBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let uid = collab_user_id(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    diesel::sql_query(
        "INSERT INTO collab_comment_reads (resource_type, resource_id, user_id, last_read_at) \
         VALUES ($1, $2, $3, NOW()) \
         ON CONFLICT (resource_type, resource_id, user_id) DO UPDATE SET last_read_at = NOW()",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&uid)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// `GET /api/collab/comments/unread?resource_type=&resource_id=&include_children=`
/// — count of non-deleted comments (excluding the reader's own) created since
/// the reader last marked the resource read.
pub async fn unread_count(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<CommentQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let uid = collab_user_id(&user);
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
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let row = diesel::sql_query(
        "SELECT COUNT(*)::bigint AS count FROM collab_comments c \
         WHERE c.deleted = FALSE AND c.author_id <> $5 \
           AND ((c.resource_type = $1 AND c.resource_id = $2) \
             OR (c.resource_type LIKE $3 AND c.resource_id LIKE $4)) \
           AND c.created_at > COALESCE( \
                 (SELECT last_read_at FROM collab_comment_reads \
                  WHERE resource_type = $1 AND resource_id = $2 AND user_id = $5), \
                 TIMESTAMPTZ 'epoch')",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Text, _>(&ty_prefix)
    .bind::<Text, _>(&id_prefix)
    .bind::<Text, _>(&uid)
    .get_result::<CountRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    Ok(Json(serde_json::json!({ "count": row.count })))
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

/// `GET /api/activity?resource_type=&resource_id=&limit=&before=` — audit
/// timeline for a resource, newest first, cursor-paginated on `created_at`.
pub async fn list_activity(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<ActivityQuery>,
) -> Result<Json<Vec<ActivityItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let before = params
        .before
        .as_deref()
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|d| d.with_timezone(&chrono::Utc))
        })
        .transpose()
        .map_err(|_| err(StatusCode::BAD_REQUEST, "Invalid `before` cursor"))?;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    // A NULL `before` means "start from the newest"; the single 4-bind query
    // keeps both cases type-compatible with diesel.
    let rows = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, action, payload, created_at \
         FROM collab_activity \
         WHERE resource_type = $1 AND resource_id = $2 \
           AND ($3::timestamptz IS NULL OR created_at < $3) \
         ORDER BY created_at DESC LIMIT $4",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<Nullable<Timestamptz>, _>(before)
    .bind::<BigInt, _>(limit)
    .load::<ActivityRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let items: Vec<ActivityItem> = rows
        .into_iter()
        .map(|r| ActivityItem {
            id: r.id,
            actor_id: r.actor_id,
            actor_name: r.actor_name,
            action: r.action,
            payload: serde_json::from_str(&r.payload).unwrap_or(serde_json::Value::Null),
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

/// `POST /api/activity` — record an audit event from a frontend mutation
/// (edit/share/restore/transfer). The actor is always resolved server-side;
/// `action` is restricted to a small allow-list.
pub async fn record_activity_event(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<RecordActivityBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    const ALLOWED: &[&str] = &[
        "create", "edit", "comment", "delete", "resolve", "reopen",
        "reaction", "share", "restore", "transfer",
    ];
    if !ALLOWED.contains(&req.action.as_str()) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid action"));
    }

    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    record_activity(
        &mut conn,
        &actor_id,
        &actor_name,
        &req.resource_type,
        &req.resource_id,
        &req.action,
        &req.payload,
    );

    Ok(Json(serde_json::json!({ "success": true })))
}

/// `POST /api/collab/versions` — snapshot a document/presentation. Unchanged
/// content (same SHA-256 as the latest snapshot) is deduped into a no-op.
pub async fn snapshot_version(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<SnapshotBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&req.resource_type, &req.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    if req.content.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Content is required"));
    }
    if req.content.len() > 2_000_000 {
        return Err(err(StatusCode::BAD_REQUEST, "Content too large"));
    }
    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let content_hash = sha256_hex(&req.content);
    let name = req.name.trim().to_string();
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct LatestHash {
        #[diesel(sql_type = Text)]
        content_hash: String,
    }
    let latest_hash = diesel::sql_query(
        "SELECT content_hash FROM collab_versions \
         WHERE resource_type = $1 AND resource_id = $2 \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .load::<LatestHash>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?
    .into_iter()
    .next();

    if let Some(latest) = latest_hash {
        if latest.content_hash == content_hash {
            return Ok(Json(serde_json::json!({ "skipped": true })));
        }
    }

    #[derive(QueryableByName)]
    struct CreatedVersion {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let created = diesel::sql_query(
        "INSERT INTO collab_versions \
         (resource_type, resource_id, actor_id, actor_name, content, content_hash, name) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id::text, created_at",
    )
    .bind::<Text, _>(&req.resource_type)
    .bind::<Text, _>(&req.resource_id)
    .bind::<Text, _>(&actor_id)
    .bind::<Text, _>(&actor_name)
    .bind::<Text, _>(&req.content)
    .bind::<Text, _>(&content_hash)
    .bind::<Text, _>(&name)
    .get_result::<CreatedVersion>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("insert failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "id": created.id,
        "actor_id": actor_id,
        "actor_name": actor_name,
        "name": name,
        "content_hash": content_hash,
        "size": req.content.len(),
        "created_at": created.created_at.to_rfc3339(),
    })))
}

/// `GET /api/collab/versions?resource_type=&resource_id=&limit=` — version
/// metadata (no content), newest first.
pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<VersionsQuery>,
) -> Result<Json<Vec<VersionItem>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let rows = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, content_hash, name, \
                octet_length(content)::bigint AS size, created_at \
         FROM collab_versions \
         WHERE resource_type = $1 AND resource_id = $2 \
         ORDER BY created_at DESC LIMIT $3",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<BigInt, _>(limit)
    .load::<VersionListRow>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    let items: Vec<VersionItem> = rows
        .into_iter()
        .map(|r| VersionItem {
            id: r.id,
            actor_id: r.actor_id,
            actor_name: r.actor_name,
            name: r.name,
            content_hash: r.content_hash,
            size: r.size,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

/// `GET /api/collab/versions/:id` — full content of a single snapshot.
pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<VersionDetail>, (StatusCode, Json<serde_json::Value>)> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let row = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, content, content_hash, name, created_at \
         FROM collab_versions WHERE id = $1",
    )
    .bind::<SqlUuid, _>(id)
    .get_result::<VersionDetailRow>(&mut conn)
    .map_err(|e| err(StatusCode::NOT_FOUND, &format!("version: {e}")))?;

    Ok(Json(VersionDetail {
        id: row.id,
        actor_id: row.actor_id,
        actor_name: row.actor_name,
        name: row.name,
        size: row.content.len() as i64,
        content: row.content,
        content_hash: row.content_hash,
        created_at: row.created_at.to_rfc3339(),
    }))
}

/// `POST /api/collab/versions/:id/restore` — create a NEW current version from
/// an older snapshot's content. History is append-only; nothing is destroyed.
pub async fn restore_version(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<VersionDetail>, (StatusCode, Json<serde_json::Value>)> {
    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let old = diesel::sql_query(
        "SELECT resource_type, resource_id, content, content_hash, name \
         FROM collab_versions WHERE id = $1",
    )
    .bind::<SqlUuid, _>(id)
    .get_result::<RestoreRow>(&mut conn)
    .map_err(|e| err(StatusCode::NOT_FOUND, &format!("version: {e}")))?;

    #[derive(QueryableByName)]
    struct CreatedVersion {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let created = diesel::sql_query(
        "INSERT INTO collab_versions \
         (resource_type, resource_id, actor_id, actor_name, content, content_hash, name) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id::text, created_at",
    )
    .bind::<Text, _>(&old.resource_type)
    .bind::<Text, _>(&old.resource_id)
    .bind::<Text, _>(&actor_id)
    .bind::<Text, _>(&actor_name)
    .bind::<Text, _>(&old.content)
    .bind::<Text, _>(&old.content_hash)
    .bind::<Text, _>(&old.name)
    .get_result::<CreatedVersion>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("insert failed: {e}")))?;

    record_activity(
        &mut conn,
        &actor_id,
        &actor_name,
        &old.resource_type,
        &old.resource_id,
        "restore",
        &serde_json::json!({ "from_version": id.to_string() }),
    );

    Ok(Json(VersionDetail {
        id: created.id,
        actor_id,
        actor_name,
        name: old.name,
        size: old.content.len() as i64,
        content: old.content,
        content_hash: old.content_hash,
        created_at: created.created_at.to_rfc3339(),
    }))
}

/// `POST /api/collab/versions/:id/name` — name/rename a milestone version.
pub async fn name_version(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<NameBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let name = req.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 255 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid name"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let changed = diesel::sql_query("UPDATE collab_versions SET name = $1 WHERE id = $2")
        .bind::<Text, _>(&name)
        .bind::<SqlUuid, _>(id)
        .execute(&mut conn)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {e}")))?;

    if changed == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Version not found"));
    }
    info!("version {id} named by {}", collab_user_id(&user));
    Ok(Json(serde_json::json!({ "success": true, "name": name })))
}

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
}
