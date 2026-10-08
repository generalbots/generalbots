use super::*;

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
pub(crate) struct CommentRow {
    #[diesel(sql_type = Text)]
    pub(crate) id: String,
    #[diesel(sql_type = Text)]
    pub(crate) resource_type: String,
    #[diesel(sql_type = Text)]
    pub(crate) resource_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) author_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) author_name: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) parent_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub(crate) body: String,
    #[diesel(sql_type = Text)]
    pub(crate) mentions: String,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) updated_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Bool)]
    pub(crate) resolved: bool,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) resolved_by: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub(crate) resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(QueryableByName)]
pub(crate) struct ReactionRow {
    #[diesel(sql_type = Text)]
    pub(crate) comment_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) user_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) emoji: String,
}

#[derive(QueryableByName)]
pub(crate) struct PresenceRow {
    #[diesel(sql_type = Text)]
    pub(crate) user_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) user_name: String,
    #[diesel(sql_type = Bool)]
    pub(crate) typing: bool,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) last_seen: chrono::DateTime<chrono::Utc>,
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
pub(crate) struct ActivityRow {
    #[diesel(sql_type = Text)]
    pub(crate) id: String,
    #[diesel(sql_type = Text)]
    pub(crate) actor_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) actor_name: String,
    #[diesel(sql_type = Text)]
    pub(crate) action: String,
    #[diesel(sql_type = Text)]
    pub(crate) payload: String,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Version history types (#860)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MentionsQuery {
    /// Max comments to return (1..100, default 20).
    #[serde(default)]
    pub limit: Option<i64>,
}

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
pub(crate) struct VersionListRow {
    #[diesel(sql_type = Text)]
    pub(crate) id: String,
    #[diesel(sql_type = Text)]
    pub(crate) actor_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) actor_name: String,
    #[diesel(sql_type = Text)]
    pub(crate) content_hash: String,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = BigInt)]
    pub(crate) size: i64,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(QueryableByName)]
pub(crate) struct VersionDetailRow {
    #[diesel(sql_type = Text)]
    pub(crate) id: String,
    #[diesel(sql_type = Text)]
    pub(crate) actor_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) actor_name: String,
    #[diesel(sql_type = Text)]
    pub(crate) content: String,
    #[diesel(sql_type = Text)]
    pub(crate) content_hash: String,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(QueryableByName)]
pub(crate) struct RestoreRow {
    #[diesel(sql_type = Text)]
    pub(crate) resource_type: String,
    #[diesel(sql_type = Text)]
    pub(crate) resource_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) content: String,
    #[diesel(sql_type = Text)]
    pub(crate) content_hash: String,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
}

// ---------------------------------------------------------------------------
// Resource sharing types (#861)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PermissionsQuery {
    pub resource_type: String,
    pub resource_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantBody {
    pub resource_type: String,
    pub resource_id: String,
    /// 'user' | 'group' | 'domain'
    pub grantee_type: String,
    /// email, group id, or domain (e.g. "corp.com" or "@corp.com")
    pub grantee_id: String,
    /// 'viewer' | 'commenter' | 'editor'
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct RevokeBody {
    pub resource_type: String,
    pub resource_id: String,
    pub grantee_type: String,
    pub grantee_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferBody {
    pub resource_type: String,
    pub resource_id: String,
    /// email (or user id) of the new owner
    pub new_owner_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LinkBody {
    pub resource_type: String,
    pub resource_id: String,
    /// 'viewer' | 'commenter' | 'editor'
    pub role: String,
    #[serde(default)]
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PermissionItem {
    pub grantee_type: String,
    pub grantee_id: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct LinkItem {
    pub token: String,
    pub role: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(QueryableByName)]
pub(crate) struct PermissionRow {
    #[diesel(sql_type = Text)]
    pub(crate) grantee_type: String,
    #[diesel(sql_type = Text)]
    pub(crate) grantee_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) role: String,
}

#[derive(QueryableByName)]
pub(crate) struct LinkRow {
    #[diesel(sql_type = Text)]
    pub(crate) token: String,
    #[diesel(sql_type = Text)]
    pub(crate) role: String,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub(crate) expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = Timestamptz)]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

pub(crate) fn valid_editable_role(role: &str) -> bool {
    matches!(role, "viewer" | "commenter" | "editor")
}

pub(crate) fn valid_grantee_type(t: &str) -> bool {
    matches!(t, "user" | "group" | "domain")
}

pub(crate) fn sanitize_resource(ty: &str, id: &str) -> bool {
    !ty.trim().is_empty()
        && ty.len() <= 64
        && !id.trim().is_empty()
        && id.len() <= 255
        && ty.chars().all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '-' || c == '_')
}
