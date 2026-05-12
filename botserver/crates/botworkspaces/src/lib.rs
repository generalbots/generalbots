use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<diesel::PgConnection>>;

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        settings -> Jsonb,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_members (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        role -> Varchar,
        invited_by -> Nullable<Uuid>,
        joined_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_pages (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        title -> Varchar,
        icon_type -> Nullable<Varchar>,
        icon_value -> Nullable<Varchar>,
        cover_image -> Nullable<Text>,
        content -> Jsonb,
        properties -> Jsonb,
        is_template -> Bool,
        template_id -> Nullable<Uuid>,
        is_public -> Bool,
        public_edit -> Bool,
        position -> Int4,
        created_by -> Uuid,
        last_edited_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_page_versions (id) {
        id -> Uuid,
        page_id -> Uuid,
        version_number -> Int4,
        title -> Varchar,
        content -> Jsonb,
        change_summary -> Nullable<Text>,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_comments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        page_id -> Uuid,
        block_id -> Nullable<Uuid>,
        parent_comment_id -> Nullable<Uuid>,
        author_id -> Uuid,
        content -> Text,
        resolved -> Bool,
        resolved_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    workspaces,
    workspace_members,
    workspace_pages,
    workspace_page_versions,
    workspace_comments,
);

pub type GetDefaultBotFn = fn(&mut diesel::PgConnection) -> (Uuid, String);

#[derive(Debug, Clone)]
pub struct WorkspacesState {
    pub pool: Arc<DbPool>,
    pub get_default_bot: GetDefaultBotFn,
}

fn get_bot_context(state: &WorkspacesState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.pool.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = (state.get_default_bot)(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = workspaces)]
pub struct DbWorkspace {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_type: Option<String>,
    pub icon_value: Option<String>,
    pub cover_image: Option<String>,
    pub settings: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = workspace_members)]
pub struct DbWorkspaceMember {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub invited_by: Option<Uuid>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = workspace_pages)]
pub struct DbWorkspacePage {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub icon_type: Option<String>,
    pub icon_value: Option<String>,
    pub cover_image: Option<String>,
    pub content: serde_json::Value,
    pub properties: serde_json::Value,
    pub is_template: bool,
    pub template_id: Option<Uuid>,
    pub is_public: bool,
    pub public_edit: bool,
    pub position: i32,
    pub created_by: Uuid,
    pub last_edited_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = workspace_page_versions)]
pub struct DbPageVersion {
    pub id: Uuid,
    pub page_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub content: serde_json::Value,
    pub change_summary: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = workspace_comments)]
pub struct DbWorkspaceComment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub page_id: Uuid,
    pub block_id: Option<Uuid>,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub settings: WorkspaceSettings,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<WorkspaceMember>,
    pub root_pages: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIcon {
    pub icon_type: IconType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IconType {
    Emoji,
    Image,
    Lucide,
}

impl IconType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Emoji => "emoji",
            Self::Image => "image",
            Self::Lucide => "lucide",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "emoji" => Self::Emoji,
            "image" => Self::Image,
            "lucide" => Self::Lucide,
            _ => Self::Emoji,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub user_id: Uuid,
    pub role: WorkspaceRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceRole {
    Owner,
    Admin,
    Editor,
    Commenter,
    Viewer,
}

impl WorkspaceRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Commenter => "commenter",
            Self::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            "editor" => Self::Editor,
            "commenter" => Self::Commenter,
            "viewer" => Self::Viewer,
            _ => Self::Viewer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceSettings {
    #[serde(default)]
    pub default_page_width: PageWidth,
    #[serde(default)]
    pub allow_public_pages: bool,
    #[serde(default = "default_true")]
    pub enable_comments: bool,
    #[serde(default = "default_true")]
    pub enable_reactions: bool,
    #[serde(default = "default_true")]
    pub enable_gb_assist: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gb_bot_id: Option<Uuid>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PageWidth {
    Small,
    #[default]
    Normal,
    Wide,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub blocks: Vec<Block>,
    pub children: Vec<Uuid>,
    pub properties: HashMap<String, PropertyValue>,
    pub permissions: PagePermissions,
    pub is_template: bool,
    pub template_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PagePermissions {
    #[serde(default = "default_true")]
    pub inherit_from_parent: bool,
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub public_edit: bool,
    #[serde(default)]
    pub allowed_users: Vec<Uuid>,
    #[serde(default)]
    pub allowed_roles: Vec<WorkspaceRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub block_type: BlockType,
    pub content: BlockContent,
    pub properties: BlockProperties,
    pub children: Vec<Block>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    BulletedList,
    NumberedList,
    Checklist,
    Toggle,
    Quote,
    Callout,
    Divider,
    Table,
    Code,
    Image,
    Video,
    File,
    Embed,
    Bookmark,
    LinkToPage,
    SyncedBlock,
    TableOfContents,
    Breadcrumb,
    Equation,
    ColumnList,
    Column,
    GbComponent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockContent {
    Text { text: RichText },
    Media { url: String, caption: Option<String> },
    Table { rows: Vec<TableRow> },
    Code { code: String, language: Option<String> },
    Embed { url: String, embed_type: Option<String> },
    Callout { icon: Option<String>, text: RichText },
    Toggle { title: RichText, expanded: bool },
    Checklist { items: Vec<ChecklistItem> },
    GbComponent { component_type: String, config: serde_json::Value },
    Empty,
}

impl Default for BlockContent {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RichText {
    pub segments: Vec<TextSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    pub text: String,
    #[serde(default)]
    pub annotations: TextAnnotations,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<Mention>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextAnnotations {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default)]
    pub code: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub mention_type: MentionType,
    pub target_id: Uuid,
    pub display_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MentionType {
    User,
    Page,
    Date,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: Uuid,
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: RichText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: RichText,
    #[serde(default)]
    pub checked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default)]
    pub indent_level: u32,
    #[serde(default)]
    pub collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Date(DateTime<Utc>),
    Select(String),
    MultiSelect(Vec<String>),
    User(Uuid),
    Users(Vec<Uuid>),
    Url(String),
    Email(String),
    Phone(String),
    Relation(Vec<Uuid>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageVersion {
    pub id: Uuid,
    pub page_id: Uuid,
    pub version_number: i32,
    pub title: String,
    pub blocks: Vec<Block>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub change_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub page_id: Uuid,
    pub block_id: Option<Uuid>,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTreeNode {
    pub id: Uuid,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub children: Vec<PageTreeNode>,
    pub has_children: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSearchResult {
    pub page_id: Uuid,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub snippet: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: SlashCommandCategory,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SlashCommandCategory {
    GbAssist,
    General,
    Media,
    Embed,
    Advanced,
}

#[derive(Debug, Clone)]
pub enum WorkspacesError {
    WorkspaceNotFound,
    PageNotFound,
    BlockNotFound,
    CommentNotFound,
    VersionNotFound,
    MemberNotFound,
    MemberAlreadyExists,
    CannotRemoveLastOwner,
    PermissionDenied,
    InvalidOperation(String),
    DbError(String),
}

impl std::fmt::Display for WorkspacesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceNotFound => write!(f, "Workspace not found"),
            Self::PageNotFound => write!(f, "Page not found"),
            Self::BlockNotFound => write!(f, "Block not found"),
            Self::CommentNotFound => write!(f, "Comment not found"),
            Self::VersionNotFound => write!(f, "Version not found"),
            Self::MemberNotFound => write!(f, "Member not found"),
            Self::MemberAlreadyExists => write!(f, "Member already exists in workspace"),
            Self::CannotRemoveLastOwner => write!(f, "Cannot remove the last owner"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::InvalidOperation(e) => write!(f, "Invalid operation: {e}"),
            Self::DbError(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for WorkspacesError {}

impl IntoResponse for WorkspacesError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Self::WorkspaceNotFound
            | Self::PageNotFound
            | Self::BlockNotFound
            | Self::CommentNotFound
            | Self::VersionNotFound
            | Self::MemberNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::PermissionDenied => (StatusCode::FORBIDDEN, self.to_string()),
            Self::MemberAlreadyExists | Self::CannotRemoveLastOwner | Self::InvalidOperation(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            Self::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

fn db_to_workspace(db: DbWorkspace, members: Vec<WorkspaceMember>, root_pages: Vec<Uuid>) -> Workspace {
    let icon = match (&db.icon_type, &db.icon_value) {
        (Some(t), Some(v)) => Some(WorkspaceIcon {
            icon_type: IconType::from_str(t),
            value: v.clone(),
        }),
        _ => None,
    };
    let settings: WorkspaceSettings = serde_json::from_value(db.settings).unwrap_or_default();

    Workspace {
        id: db.id,
        org_id: db.org_id,
        name: db.name,
        description: db.description,
        icon,
        cover_image: db.cover_image,
        settings,
        created_by: db.created_by,
        created_at: db.created_at,
        updated_at: db.updated_at,
        members,
        root_pages,
    }
}

fn db_to_page(db: DbWorkspacePage, children: Vec<Uuid>) -> Page {
    let icon = match (&db.icon_type, &db.icon_value) {
        (Some(t), Some(v)) => Some(WorkspaceIcon {
            icon_type: IconType::from_str(t),
            value: v.clone(),
        }),
        _ => None,
    };
    let blocks: Vec<Block> = serde_json::from_value(db.content).unwrap_or_default();
    let properties: HashMap<String, PropertyValue> =
        serde_json::from_value(db.properties).unwrap_or_default();

    Page {
        id: db.id,
        workspace_id: db.workspace_id,
        parent_id: db.parent_id,
        title: db.title,
        icon,
        cover_image: db.cover_image,
        blocks,
        children,
        properties,
        permissions: PagePermissions {
            inherit_from_parent: true,
            public: db.is_public,
            public_edit: db.public_edit,
            allowed_users: vec![],
            allowed_roles: vec![],
        },
        is_template: db.is_template,
        template_id: db.template_id,
        created_at: db.created_at,
        updated_at: db.updated_at,
        created_by: db.created_by,
        last_edited_by: db.last_edited_by.unwrap_or(db.created_by),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<WorkspaceIcon>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePageRequest {
    pub title: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePageRequest {
    pub title: Option<String>,
    pub icon: Option<WorkspaceIcon>,
    pub blocks: Option<Vec<Block>>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: WorkspaceRole,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub block_id: Option<Uuid>,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

async fn list_workspaces(
    State(state): State<Arc<WorkspacesState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Workspace>>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = workspaces::table
        .filter(workspaces::org_id.eq(org_id))
        .filter(workspaces::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            workspaces::name
                .ilike(pattern.clone())
                .or(workspaces::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = q
        .order(workspaces::updated_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let mut result = Vec::with_capacity(db_workspaces.len());
    for ws in db_workspaces {
        let db_members: Vec<DbWorkspaceMember> = workspace_members::table
            .filter(workspace_members::workspace_id.eq(ws.id))
            .load(&mut conn)
            .unwrap_or_default();

        let members: Vec<WorkspaceMember> = db_members
            .into_iter()
            .map(|m| WorkspaceMember {
                user_id: m.user_id,
                role: WorkspaceRole::from_str(&m.role),
                joined_at: m.joined_at,
                invited_by: m.invited_by,
            })
            .collect();

        let root_pages: Vec<Uuid> = workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(ws.id))
            .filter(workspace_pages::parent_id.is_null())
            .select(workspace_pages::id)
            .order(workspace_pages::position.asc())
            .load(&mut conn)
            .unwrap_or_default();

        result.push(db_to_workspace(ws, members, root_pages));
    }

    Ok(Json(result))
}

async fn create_workspace(
    State(state): State<Arc<WorkspacesState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let user_id = Uuid::nil();

    let settings = WorkspaceSettings::default();
    let settings_json = serde_json::to_value(&settings).unwrap_or_else(|_| serde_json::json!({}));

    let db_workspace = DbWorkspace {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        icon_type: None,
        icon_value: None,
        cover_image: None,
        settings: settings_json,
        created_by: user_id,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(workspaces::table)
        .values(&db_workspace)
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let member = DbWorkspaceMember {
        id: Uuid::new_v4(),
        workspace_id: id,
        user_id,
        role: WorkspaceRole::Owner.as_str().to_string(),
        invited_by: None,
        joined_at: now,
    };

    diesel::insert_into(workspace_members::table)
        .values(&member)
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let members = vec![WorkspaceMember {
        user_id,
        role: WorkspaceRole::Owner,
        joined_at: now,
        invited_by: None,
    }];

    let workspace = db_to_workspace(db_workspace, members, vec![]);
    Ok(Json(workspace))
}

async fn get_workspace(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let db_workspace: DbWorkspace = workspaces::table
        .filter(workspaces::id.eq(workspace_id))
        .first(&mut conn)
        .map_err(|_| WorkspacesError::WorkspaceNotFound)?;

    let db_members: Vec<DbWorkspaceMember> = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .load(&mut conn)
        .unwrap_or_default();

    let members: Vec<WorkspaceMember> = db_members
        .into_iter()
        .map(|m| WorkspaceMember {
            user_id: m.user_id,
            role: WorkspaceRole::from_str(&m.role),
            joined_at: m.joined_at,
            invited_by: m.invited_by,
        })
        .collect();

    let root_pages: Vec<Uuid> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .filter(workspace_pages::parent_id.is_null())
        .select(workspace_pages::id)
        .order(workspace_pages::position.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let workspace = db_to_workspace(db_workspace, members, root_pages);
    Ok(Json(workspace))
}

async fn update_workspace(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let mut db_workspace: DbWorkspace = workspaces::table
        .filter(workspaces::id.eq(workspace_id))
        .first(&mut conn)
        .map_err(|_| WorkspacesError::WorkspaceNotFound)?;

    if let Some(name) = req.name {
        db_workspace.name = name;
    }
    if let Some(desc) = req.description {
        db_workspace.description = Some(desc);
    }
    if let Some(icon) = req.icon {
        db_workspace.icon_type = Some(icon.icon_type.as_str().to_string());
        db_workspace.icon_value = Some(icon.value);
    }
    db_workspace.updated_at = Utc::now();

    diesel::update(workspaces::table.filter(workspaces::id.eq(workspace_id)))
        .set(&db_workspace)
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let db_members: Vec<DbWorkspaceMember> = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .load(&mut conn)
        .unwrap_or_default();

    let members: Vec<WorkspaceMember> = db_members
        .into_iter()
        .map(|m| WorkspaceMember {
            user_id: m.user_id,
            role: WorkspaceRole::from_str(&m.role),
            joined_at: m.joined_at,
            invited_by: m.invited_by,
        })
        .collect();

    let root_pages: Vec<Uuid> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .filter(workspace_pages::parent_id.is_null())
        .select(workspace_pages::id)
        .order(workspace_pages::position.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let workspace = db_to_workspace(db_workspace, members, root_pages);
    Ok(Json(workspace))
}

async fn delete_workspace(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    diesel::delete(workspace_comments::table.filter(workspace_comments::workspace_id.eq(workspace_id)))
        .execute(&mut conn)
        .ok();

    let page_ids: Vec<Uuid> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .select(workspace_pages::id)
        .load(&mut conn)
        .unwrap_or_default();

    if !page_ids.is_empty() {
        diesel::delete(workspace_page_versions::table.filter(
            workspace_page_versions::page_id.eq_any(&page_ids),
        ))
        .execute(&mut conn)
        .ok();
    }

    diesel::delete(workspace_pages::table.filter(workspace_pages::workspace_id.eq(workspace_id)))
        .execute(&mut conn)
        .ok();

    diesel::delete(workspace_members::table.filter(workspace_members::workspace_id.eq(workspace_id)))
        .execute(&mut conn)
        .ok();

    let deleted = diesel::delete(workspaces::table.filter(workspaces::id.eq(workspace_id)))
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WorkspacesError::WorkspaceNotFound)
    }
}

async fn list_pages(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<PageTreeNode>>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let db_pages: Vec<DbWorkspacePage> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .order(workspace_pages::position.asc())
        .load(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    fn build_tree(pages: &[DbWorkspacePage], parent_id: Option<Uuid>) -> Vec<PageTreeNode> {
        pages
            .iter()
            .filter(|p| p.parent_id == parent_id)
            .map(|p| {
                let icon = match (&p.icon_type, &p.icon_value) {
                    (Some(t), Some(v)) => Some(WorkspaceIcon {
                        icon_type: IconType::from_str(t),
                        value: v.clone(),
                    }),
                    _ => None,
                };
                let children = build_tree(pages, Some(p.id));
                PageTreeNode {
                    id: p.id,
                    title: p.title.clone(),
                    icon,
                    has_children: !children.is_empty(),
                    children,
                }
            })
            .collect()
    }

    let tree = build_tree(&db_pages, None);
    Ok(Json(tree))
}

async fn create_page(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<Page>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let _: DbWorkspace = workspaces::table
        .filter(workspaces::id.eq(workspace_id))
        .first(&mut conn)
        .map_err(|_| WorkspacesError::WorkspaceNotFound)?;

    let now = Utc::now();
    let user_id = Uuid::nil();
    let id = Uuid::new_v4();

    let max_position: Option<i32> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .filter(workspace_pages::parent_id.is_not_distinct_from(req.parent_id))
        .select(diesel::dsl::max(workspace_pages::position))
        .first(&mut conn)
        .ok()
        .flatten();

    let db_page = DbWorkspacePage {
        id,
        workspace_id,
        parent_id: req.parent_id,
        title: req.title,
        icon_type: None,
        icon_value: None,
        cover_image: None,
        content: serde_json::json!([]),
        properties: serde_json::json!({}),
        is_template: false,
        template_id: None,
        is_public: false,
        public_edit: false,
        position: max_position.unwrap_or(0) + 1,
        created_by: user_id,
        last_edited_by: Some(user_id),
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(workspace_pages::table)
        .values(&db_page)
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let page = db_to_page(db_page, vec![]);
    Ok(Json(page))
}

async fn get_page(
    State(state): State<Arc<WorkspacesState>>,
    Path(page_id): Path<Uuid>,
) -> Result<Json<Page>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let db_page: DbWorkspacePage = workspace_pages::table
        .filter(workspace_pages::id.eq(page_id))
        .first(&mut conn)
        .map_err(|_| WorkspacesError::PageNotFound)?;

    let children: Vec<Uuid> = workspace_pages::table
        .filter(workspace_pages::parent_id.eq(page_id))
        .select(workspace_pages::id)
        .order(workspace_pages::position.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let page = db_to_page(db_page, children);
    Ok(Json(page))
}

async fn update_page(
    State(state): State<Arc<WorkspacesState>>,
    Path(page_id): Path<Uuid>,
    Json(req): Json<UpdatePageRequest>,
) -> Result<Json<Page>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let mut db_page: DbWorkspacePage = workspace_pages::table
        .filter(workspace_pages::id.eq(page_id))
        .first(&mut conn)
        .map_err(|_| WorkspacesError::PageNotFound)?;

    if let Some(title) = req.title {
        db_page.title = title;
    }
    if let Some(icon) = req.icon {
        db_page.icon_type = Some(icon.icon_type.as_str().to_string());
        db_page.icon_value = Some(icon.value);
    }
    if let Some(blocks) = req.blocks {
        db_page.content = serde_json::to_value(&blocks).unwrap_or_else(|_| serde_json::json!([]));
    }
    db_page.updated_at = Utc::now();
    db_page.last_edited_by = Some(Uuid::nil());

    diesel::update(workspace_pages::table.filter(workspace_pages::id.eq(page_id)))
        .set(&db_page)
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let children: Vec<Uuid> = workspace_pages::table
        .filter(workspace_pages::parent_id.eq(page_id))
        .select(workspace_pages::id)
        .order(workspace_pages::position.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let page = db_to_page(db_page, children);
    Ok(Json(page))
}

async fn delete_page(
    State(state): State<Arc<WorkspacesState>>,
    Path(page_id): Path<Uuid>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    diesel::delete(workspace_comments::table.filter(workspace_comments::page_id.eq(page_id)))
        .execute(&mut conn)
        .ok();

    diesel::delete(workspace_page_versions::table.filter(workspace_page_versions::page_id.eq(page_id)))
        .execute(&mut conn)
        .ok();

    diesel::delete(workspace_pages::table.filter(workspace_pages::parent_id.eq(page_id)))
        .execute(&mut conn)
        .ok();

    let deleted = diesel::delete(workspace_pages::table.filter(workspace_pages::id.eq(page_id)))
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WorkspacesError::PageNotFound)
    }
}

async fn add_member(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let existing: Option<DbWorkspaceMember> = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .filter(workspace_members::user_id.eq(req.user_id))
        .first(&mut conn)
        .optional()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    if existing.is_some() {
        return Err(WorkspacesError::MemberAlreadyExists);
    }

    let now = Utc::now();
    let member = DbWorkspaceMember {
        id: Uuid::new_v4(),
        workspace_id,
        user_id: req.user_id,
        role: req.role.as_str().to_string(),
        invited_by: Some(Uuid::nil()),
        joined_at: now,
    };

    diesel::insert_into(workspace_members::table)
        .values(&member)
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    Ok(StatusCode::CREATED)
}

async fn remove_member(
    State(state): State<Arc<WorkspacesState>>,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let owner_count: i64 = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .filter(workspace_members::role.eq("owner"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let member: Option<DbWorkspaceMember> = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .filter(workspace_members::user_id.eq(user_id))
        .first(&mut conn)
        .optional()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    if let Some(m) = member {
        if m.role == "owner" && owner_count <= 1 {
            return Err(WorkspacesError::CannotRemoveLastOwner);
        }
    } else {
        return Err(WorkspacesError::MemberNotFound);
    }

    diesel::delete(
        workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .filter(workspace_members::user_id.eq(user_id)),
    )
    .execute(&mut conn)
    .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn search_pages(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<PageSearchResult>>, WorkspacesError> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let query = params.q.unwrap_or_default();
    if query.is_empty() {
        return Ok(Json(vec![]));
    }

    let pattern = format!("%{query}%");
    let db_pages: Vec<DbWorkspacePage> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .filter(workspace_pages::title.ilike(&pattern))
        .order(workspace_pages::updated_at.desc())
        .limit(20)
        .load(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let results: Vec<PageSearchResult> = db_pages
        .into_iter()
        .map(|p| {
            let icon = match (&p.icon_type, &p.icon_value) {
                (Some(t), Some(v)) => Some(WorkspaceIcon {
                    icon_type: IconType::from_str(t),
                    value: v.clone(),
                }),
                _ => None,
            };
            PageSearchResult {
                page_id: p.id,
                title: p.title,
                icon,
                snippet: String::new(),
                updated_at: p.updated_at,
            }
        })
        .collect();

    Ok(Json(results))
}

async fn get_slash_commands_handler(
    State(_state): State<Arc<WorkspacesState>>,
) -> Json<Vec<SlashCommand>> {
    Json(vec![
        SlashCommand {
            id: "paragraph".to_string(),
            name: "Text".to_string(),
            description: "Plain text paragraph".to_string(),
            icon: "type".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["text".to_string(), "paragraph".to_string()],
        },
        SlashCommand {
            id: "heading1".to_string(),
            name: "Heading 1".to_string(),
            description: "Large section heading".to_string(),
            icon: "heading-1".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["h1".to_string(), "heading".to_string()],
        },
        SlashCommand {
            id: "heading2".to_string(),
            name: "Heading 2".to_string(),
            description: "Medium section heading".to_string(),
            icon: "heading-2".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["h2".to_string(), "heading".to_string()],
        },
        SlashCommand {
            id: "bulleted_list".to_string(),
            name: "Bulleted list".to_string(),
            description: "Create a bulleted list".to_string(),
            icon: "list".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["bullet".to_string(), "list".to_string()],
        },
        SlashCommand {
            id: "numbered_list".to_string(),
            name: "Numbered list".to_string(),
            description: "Create a numbered list".to_string(),
            icon: "list-ordered".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["number".to_string(), "list".to_string()],
        },
        SlashCommand {
            id: "checklist".to_string(),
            name: "Checklist".to_string(),
            description: "Create a checklist".to_string(),
            icon: "check-square".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["todo".to_string(), "checkbox".to_string()],
        },
        SlashCommand {
            id: "code".to_string(),
            name: "Code".to_string(),
            description: "Create a code block".to_string(),
            icon: "code".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["code".to_string(), "snippet".to_string()],
        },
        SlashCommand {
            id: "image".to_string(),
            name: "Image".to_string(),
            description: "Upload or embed an image".to_string(),
            icon: "image".to_string(),
            category: SlashCommandCategory::Media,
            keywords: vec!["image".to_string(), "picture".to_string()],
        },
    ])
}

pub fn configure_workspaces_routes() -> Router<Arc<WorkspacesState>> {
    Router::new()
        .route("/api/workspaces", get(list_workspaces).post(create_workspace))
        .route(
            "/api/workspaces/{workspace_id}",
            get(get_workspace).put(update_workspace).delete(delete_workspace),
        )
        .route(
            "/api/workspaces/{workspace_id}/pages",
            get(list_pages).post(create_page),
        )
        .route("/api/workspaces/{workspace_id}/members", post(add_member))
        .route(
            "/api/workspaces/{workspace_id}/members/{user_id}",
            delete(remove_member),
        )
        .route("/api/workspaces/{workspace_id}/search", get(search_pages))
        .route(
            "/api/pages/{page_id}",
            get(get_page).put(update_page).delete(delete_page),
        )
        .route("/api/workspaces/commands", get(get_slash_commands_handler))
}

pub struct BlockBuilder {
    block_type: BlockType,
    content: BlockContent,
    properties: BlockProperties,
    children: Vec<Block>,
    created_by: Uuid,
}

impl BlockBuilder {
    pub fn new(block_type: BlockType, created_by: Uuid) -> Self {
        Self {
            block_type,
            content: BlockContent::Empty,
            properties: BlockProperties::default(),
            children: Vec::new(),
            created_by,
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.content = BlockContent::Text {
            text: RichText {
                segments: vec![TextSegment {
                    text: text.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
        };
        self
    }

    pub fn with_rich_text(mut self, rich_text: RichText) -> Self {
        self.content = BlockContent::Text { text: rich_text };
        self
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.properties.color = Some(color.to_string());
        self
    }

    pub fn with_background(mut self, color: &str) -> Self {
        self.properties.background_color = Some(color.to_string());
        self
    }

    pub fn with_indent(mut self, level: u32) -> Self {
        self.properties.indent_level = level;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.properties.collapsed = collapsed;
        self
    }

    pub fn with_children(mut self, children: Vec<Block>) -> Self {
        self.children = children;
        self
    }

    pub fn build(self) -> Block {
        let now = Utc::now();
        Block {
            id: Uuid::new_v4(),
            block_type: self.block_type,
            content: self.content,
            properties: self.properties,
            children: self.children,
            created_at: now,
            updated_at: now,
            created_by: self.created_by,
        }
    }
}

pub fn create_paragraph(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Paragraph, created_by)
        .with_text(text)
        .build()
}

pub fn create_heading1(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Heading1, created_by)
        .with_text(text)
        .build()
}

pub fn create_heading2(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Heading2, created_by)
        .with_text(text)
        .build()
}

pub fn create_heading3(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Heading3, created_by)
        .with_text(text)
        .build()
}

pub fn create_bulleted_list_item(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::BulletedList, created_by)
        .with_text(text)
        .build()
}

pub fn create_numbered_list_item(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::NumberedList, created_by)
        .with_text(text)
        .build()
}

pub fn create_checklist(items: Vec<(&str, bool)>, created_by: Uuid) -> Block {
    let checklist_items: Vec<ChecklistItem> = items
        .into_iter()
        .map(|(text, checked)| ChecklistItem {
            id: Uuid::new_v4(),
            text: RichText {
                segments: vec![TextSegment {
                    text: text.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
            checked,
            assignee: None,
            due_date: None,
        })
        .collect();

    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Checklist,
        content: BlockContent::Checklist {
            items: checklist_items,
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_toggle(title: &str, expanded: bool, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Toggle,
        content: BlockContent::Toggle {
            title: RichText {
                segments: vec![TextSegment {
                    text: title.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
            expanded,
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_quote(text: &str, created_by: Uuid) -> Block {
    BlockBuilder::new(BlockType::Quote, created_by)
        .with_text(text)
        .build()
}

pub fn create_callout(icon: &str, text: &str, background: &str, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Callout,
        content: BlockContent::Callout {
            icon: Some(icon.to_string()),
            text: RichText {
                segments: vec![TextSegment {
                    text: text.to_string(),
                    annotations: TextAnnotations::default(),
                    link: None,
                    mention: None,
                }],
            },
        },
        properties: BlockProperties {
            background_color: Some(background.to_string()),
            ..Default::default()
        },
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_divider(created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Divider,
        content: BlockContent::Empty,
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_code(code: &str, language: &str, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Code,
        content: BlockContent::Code {
            code: code.to_string(),
            language: Some(language.to_string()),
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_image(url: &str, caption: Option<&str>, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Image,
        content: BlockContent::Media {
            url: url.to_string(),
            caption: caption.map(|s| s.to_string()),
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_video(url: &str, caption: Option<&str>, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Video,
        content: BlockContent::Media {
            url: url.to_string(),
            caption: caption.map(|s| s.to_string()),
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_embed(url: &str, embed_type: &str, created_by: Uuid) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Embed,
        content: BlockContent::Embed {
            url: url.to_string(),
            embed_type: Some(embed_type.to_string()),
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_table(rows: usize, cols: usize, created_by: Uuid) -> Block {
    let now = Utc::now();
    let table_rows: Vec<TableRow> = (0..rows)
        .map(|_| TableRow {
            id: Uuid::new_v4(),
            cells: (0..cols)
                .map(|_| TableCell {
                    content: RichText { segments: vec![] },
                    background_color: None,
                })
                .collect(),
        })
        .collect();

    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::Table,
        content: BlockContent::Table { rows: table_rows },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

pub fn create_gb_component(
    component_type: &str,
    config: serde_json::Value,
    created_by: Uuid,
) -> Block {
    let now = Utc::now();
    Block {
        id: Uuid::new_v4(),
        block_type: BlockType::GbComponent,
        content: BlockContent::GbComponent {
            component_type: component_type.to_string(),
            config,
        },
        properties: BlockProperties::default(),
        children: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockOperation {
    pub operation_type: BlockOperationType,
    pub block_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub position: Option<usize>,
    pub block: Option<Block>,
    pub properties: Option<BlockProperties>,
    pub content: Option<BlockContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockOperationType {
    Insert,
    Update,
    Delete,
    Move,
    Duplicate,
}

pub fn apply_block_operations(blocks: &mut Vec<Block>, operations: Vec<BlockOperation>) {
    for op in operations {
        match op.operation_type {
            BlockOperationType::Insert => {
                if let Some(block) = op.block {
                    let position = op.position.unwrap_or(blocks.len());
                    if position <= blocks.len() {
                        blocks.insert(position, block);
                    }
                }
            }
            BlockOperationType::Update => {
                if let Some(block_id) = op.block_id {
                    if let Some(block) = find_block_mut(blocks, block_id) {
                        if let Some(content) = op.content {
                            block.content = content;
                        }
                        if let Some(props) = op.properties {
                            block.properties = props;
                        }
                        block.updated_at = Utc::now();
                    }
                }
            }
            BlockOperationType::Delete => {
                if let Some(block_id) = op.block_id {
                    remove_block(blocks, block_id);
                }
            }
            BlockOperationType::Move => {
                if let Some(block_id) = op.block_id {
                    if let Some(position) = op.position {
                        if let Some(block) = remove_block(blocks, block_id) {
                            let insert_pos = position.min(blocks.len());
                            blocks.insert(insert_pos, block);
                        }
                    }
                }
            }
            BlockOperationType::Duplicate => {
                if let Some(block_id) = op.block_id {
                    if let Some(block) = find_block(blocks, block_id) {
                        let mut new_block = block.clone();
                        new_block.id = Uuid::new_v4();
                        new_block.created_at = Utc::now();
                        new_block.updated_at = Utc::now();
                        let position = op.position.unwrap_or(blocks.len());
                        blocks.insert(position.min(blocks.len()), new_block);
                    }
                }
            }
        }
    }
}

fn find_block(blocks: &[Block], block_id: Uuid) -> Option<&Block> {
    for block in blocks {
        if block.id == block_id {
            return Some(block);
        }
        if let Some(found) = find_block(&block.children, block_id) {
            return Some(found);
        }
    }
    None
}

fn find_block_mut(blocks: &mut [Block], block_id: Uuid) -> Option<&mut Block> {
    for block in blocks.iter_mut() {
        if block.id == block_id {
            return Some(block);
        }
        if let Some(found) = find_block_mut(&mut block.children, block_id) {
            return Some(found);
        }
    }
    None
}

fn remove_block(blocks: &mut Vec<Block>, block_id: Uuid) -> Option<Block> {
    if let Some(pos) = blocks.iter().position(|b| b.id == block_id) {
        return Some(blocks.remove(pos));
    }

    for block in blocks.iter_mut() {
        if let Some(removed) = remove_block(&mut block.children, block_id) {
            return Some(removed);
        }
    }

    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id: Uuid,
    pub page_id: Uuid,
    pub active_users: Vec<ActiveUser>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveUser {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub color: String,
    pub cursor_position: Option<CursorPosition>,
    pub selection: Option<TextSelection>,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub block_id: Uuid,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSelection {
    pub block_id: Uuid,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationEvent {
    pub event_type: CollaborationEventType,
    pub page_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: CollaborationPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationEventType {
    UserJoined,
    UserLeft,
    CursorMoved,
    SelectionChanged,
    BlockOperation,
    PageUpdated,
    CommentAdded,
    CommentResolved,
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CollaborationPayload {
    User(ActiveUser),
    Cursor(CursorPosition),
    Selection(TextSelection),
    Operation(BlockOperation),
    PageUpdate(PageUpdatePayload),
    Comment(CommentPayload),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageUpdatePayload {
    pub title: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentPayload {
    pub comment_id: Uuid,
    pub block_id: Option<Uuid>,
    pub content: String,
}

pub struct CollaborationManager {
    sessions: Arc<RwLock<HashMap<Uuid, CollaborationSession>>>,
    event_channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<CollaborationEvent>>>>,
    user_colors: Vec<String>,
}

impl CollaborationManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            event_channels: Arc::new(RwLock::new(HashMap::new())),
            user_colors: vec![
                "#E53935".to_string(),
                "#8E24AA".to_string(),
                "#3949AB".to_string(),
                "#039BE5".to_string(),
                "#00ACC1".to_string(),
                "#43A047".to_string(),
                "#7CB342".to_string(),
                "#FDD835".to_string(),
                "#FB8C00".to_string(),
                "#6D4C41".to_string(),
            ],
        }
    }

    pub async fn join_session(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        display_name: String,
        avatar_url: Option<String>,
    ) -> Result<
        (CollaborationSession, broadcast::Receiver<CollaborationEvent>),
        CollaborationError,
    > {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let mut channels = self.event_channels.write().await;

        let color = self.assign_color(&sessions, page_id);

        let active_user = ActiveUser {
            user_id,
            display_name,
            avatar_url,
            color,
            cursor_position: None,
            selection: None,
            joined_at: now,
            last_seen: now,
        };

        let session = sessions.entry(page_id).or_insert_with(|| CollaborationSession {
            id: Uuid::new_v4(),
            page_id,
            active_users: Vec::new(),
            created_at: now,
            last_activity: now,
        });

        if !session.active_users.iter().any(|u| u.user_id == user_id) {
            session.active_users.push(active_user.clone());
        }
        session.last_activity = now;

        let (tx, rx) = if let Some(existing_tx) = channels.get(&page_id) {
            (existing_tx.clone(), existing_tx.subscribe())
        } else {
            let (tx, rx) = broadcast::channel(256);
            channels.insert(page_id, tx.clone());
            (tx, rx)
        };

        let event = CollaborationEvent {
            event_type: CollaborationEventType::UserJoined,
            page_id,
            user_id,
            timestamp: now,
            payload: CollaborationPayload::User(active_user),
        };

        let _ = tx.send(event);

        Ok((session.clone(), rx))
    }

    pub async fn leave_session(
        &self,
        page_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            session.active_users.retain(|u| u.user_id != user_id);
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                let event = CollaborationEvent {
                    event_type: CollaborationEventType::UserLeft,
                    page_id,
                    user_id,
                    timestamp: now,
                    payload: CollaborationPayload::Empty,
                };
                let _ = tx.send(event);
            }

            if session.active_users.is_empty() {
                sessions.remove(&page_id);
            }
        }

        Ok(())
    }

    pub async fn update_cursor(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        cursor: CursorPosition,
    ) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            if let Some(user) = session.active_users.iter_mut().find(|u| u.user_id == user_id) {
                user.cursor_position = Some(cursor.clone());
                user.last_seen = now;
            }
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                let event = CollaborationEvent {
                    event_type: CollaborationEventType::CursorMoved,
                    page_id,
                    user_id,
                    timestamp: now,
                    payload: CollaborationPayload::Cursor(cursor),
                };
                let _ = tx.send(event);
            }
        }

        Ok(())
    }

    pub async fn update_selection(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        selection: Option<TextSelection>,
    ) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            if let Some(user) = session.active_users.iter_mut().find(|u| u.user_id == user_id) {
                user.selection = selection.clone();
                user.last_seen = now;
            }
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                if let Some(sel) = selection {
                    let event = CollaborationEvent {
                        event_type: CollaborationEventType::SelectionChanged,
                        page_id,
                        user_id,
                        timestamp: now,
                        payload: CollaborationPayload::Selection(sel),
                    };
                    let _ = tx.send(event);
                }
            }
        }

        Ok(())
    }

    pub async fn broadcast_operation(
        &self,
        page_id: Uuid,
        user_id: Uuid,
        operation: BlockOperation,
    ) -> Result<(), CollaborationError> {
        let now = Utc::now();

        let mut sessions = self.sessions.write().await;
        let channels = self.event_channels.read().await;

        if let Some(session) = sessions.get_mut(&page_id) {
            session.last_activity = now;

            if let Some(tx) = channels.get(&page_id) {
                let event = CollaborationEvent {
                    event_type: CollaborationEventType::BlockOperation,
                    page_id,
                    user_id,
                    timestamp: now,
                    payload: CollaborationPayload::Operation(operation),
                };
                let _ = tx.send(event);
            }
        }

        Ok(())
    }

    pub async fn get_session(&self, page_id: Uuid) -> Option<CollaborationSession> {
        let sessions = self.sessions.read().await;
        sessions.get(&page_id).cloned()
    }

    pub async fn get_active_users(&self, page_id: Uuid) -> Vec<ActiveUser> {
        let sessions = self.sessions.read().await;
        sessions
            .get(&page_id)
            .map(|s| s.active_users.clone())
            .unwrap_or_default()
    }

    pub async fn cleanup_stale_sessions(&self, timeout_seconds: i64) {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(timeout_seconds);

        let mut sessions = self.sessions.write().await;
        let mut channels = self.event_channels.write().await;

        let stale_pages: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, s)| s.last_activity < cutoff)
            .map(|(id, _)| *id)
            .collect();

        for page_id in stale_pages {
            sessions.remove(&page_id);
            channels.remove(&page_id);
        }

        for session in sessions.values_mut() {
            session.active_users.retain(|u| u.last_seen >= cutoff);
        }
    }

    fn assign_color(
        &self,
        sessions: &HashMap<Uuid, CollaborationSession>,
        page_id: Uuid,
    ) -> String {
        if let Some(session) = sessions.get(&page_id) {
            let used_colors: Vec<&String> = session.active_users.iter().map(|u| &u.color).collect();

            for color in &self.user_colors {
                if !used_colors.contains(&color) {
                    return color.clone();
                }
            }
        }

        self.user_colors[0].clone()
    }
}

impl Default for CollaborationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalTransform {
    pub base_version: u64,
    pub operations: Vec<TransformOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOperation {
    pub op_type: TransformOpType,
    pub path: Vec<usize>,
    pub value: Option<serde_json::Value>,
    pub old_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransformOpType {
    Insert,
    Delete,
    Replace,
    Move,
}

pub fn transform_operations(
    op1: &TransformOperation,
    op2: &TransformOperation,
) -> (TransformOperation, TransformOperation) {
    let mut transformed_op1 = op1.clone();
    let mut transformed_op2 = op2.clone();

    if op1.path.is_empty() || op2.path.is_empty() {
        return (transformed_op1, transformed_op2);
    }

    let common_prefix_len = op1
        .path
        .iter()
        .zip(op2.path.iter())
        .take_while(|(a, b)| a == b)
        .count();

    if common_prefix_len == 0 {
        return (transformed_op1, transformed_op2);
    }

    match (&op1.op_type, &op2.op_type) {
        (TransformOpType::Insert, TransformOpType::Insert) => {
            if op1.path <= op2.path {
                if let Some(idx) = transformed_op2.path.get_mut(common_prefix_len) {
                    *idx += 1;
                }
            } else if let Some(idx) = transformed_op1.path.get_mut(common_prefix_len) {
                *idx += 1;
            }
        }
        (TransformOpType::Delete, TransformOpType::Insert) => {
            if op1.path < op2.path {
                if let Some(idx) = transformed_op2.path.get_mut(common_prefix_len) {
                    *idx = idx.saturating_sub(1);
                }
            }
        }
        (TransformOpType::Insert, TransformOpType::Delete) => {
            if op2.path < op1.path {
                if let Some(idx) = transformed_op1.path.get_mut(common_prefix_len) {
                    *idx = idx.saturating_sub(1);
                }
            }
        }
        (TransformOpType::Delete, TransformOpType::Delete) => {
            if op1.path == op2.path {
                transformed_op2.op_type = TransformOpType::Replace;
                transformed_op2.value = None;
            }
        }
        _ => {}
    }

    (transformed_op1, transformed_op2)
}

#[derive(Debug, Clone)]
pub enum CollaborationError {
    SessionNotFound,
    UserNotInSession,
    BroadcastError(String),
    InvalidOperation(String),
}

impl std::fmt::Display for CollaborationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionNotFound => write!(f, "Collaboration session not found"),
            Self::UserNotInSession => write!(f, "User is not in the session"),
            Self::BroadcastError(e) => write!(f, "Broadcast error: {e}"),
            Self::InvalidOperation(e) => write!(f, "Invalid operation: {e}"),
        }
    }
}

impl std::error::Error for CollaborationError {}

pub async fn collaboration_cleanup_job(manager: Arc<CollaborationManager>, interval_seconds: u64) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));

    loop {
        ticker.tick().await;
        manager.cleanup_stale_sessions(300).await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub page_id: Uuid,
    pub users: Vec<PresenceUser>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUser {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub color: String,
    pub is_typing: bool,
    pub current_block: Option<Uuid>,
}

impl From<&ActiveUser> for PresenceUser {
    fn from(user: &ActiveUser) -> Self {
        Self {
            user_id: user.user_id,
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            color: user.color.clone(),
            is_typing: false,
            current_block: user.cursor_position.as_ref().map(|c| c.block_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub page_id: Uuid,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBreadcrumbs {
    pub workspace_id: Uuid,
    pub workspace_name: String,
    pub workspace_icon: Option<WorkspaceIcon>,
    pub path: Vec<Breadcrumb>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSummary {
    pub id: Uuid,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub parent_id: Option<Uuid>,
    pub has_children: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_edited_by: Uuid,
}

impl From<&Page> for PageSummary {
    fn from(page: &Page) -> Self {
        Self {
            id: page.id,
            title: page.title.clone(),
            icon: page.icon.clone(),
            parent_id: page.parent_id,
            has_children: !page.children.is_empty(),
            created_at: page.created_at,
            updated_at: page.updated_at,
            created_by: page.created_by,
            last_edited_by: page.last_edited_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesCreatePageRequest {
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub template_id: Option<Uuid>,
    pub properties: Option<HashMap<String, PropertyValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesUpdatePageRequest {
    pub title: Option<String>,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub properties: Option<HashMap<String, PropertyValue>>,
    pub permissions: Option<PagePermissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePageRequest {
    pub new_parent_id: Option<Uuid>,
    pub new_workspace_id: Option<Uuid>,
    pub position: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatePageRequest {
    pub new_parent_id: Option<Uuid>,
    pub new_workspace_id: Option<Uuid>,
    pub include_children: bool,
    pub new_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageExportOptions {
    pub format: ExportFormat,
    pub include_children: bool,
    pub include_images: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Markdown,
    Html,
    Pdf,
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageImportOptions {
    pub format: ImportFormat,
    pub parent_id: Option<Uuid>,
    pub workspace_id: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    Markdown,
    Html,
    Notion,
    Confluence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentPage {
    pub page_id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub workspace_name: String,
    pub accessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoritePage {
    pub page_id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub added_at: DateTime<Utc>,
}

pub fn build_breadcrumbs(
    page_id: Uuid,
    pages: &HashMap<Uuid, Page>,
    workspace_name: &str,
    workspace_icon: Option<WorkspaceIcon>,
    workspace_id: Uuid,
) -> PageBreadcrumbs {
    let mut path = Vec::new();
    let mut current_id = Some(page_id);

    while let Some(id) = current_id {
        if let Some(page) = pages.get(&id) {
            path.push(Breadcrumb {
                page_id: page.id,
                title: page.title.clone(),
                icon: page.icon.clone(),
            });
            current_id = page.parent_id;
        } else {
            break;
        }
    }

    path.reverse();

    PageBreadcrumbs {
        workspace_id,
        workspace_name: workspace_name.to_string(),
        workspace_icon,
        path,
    }
}

pub fn get_page_depth(page_id: Uuid, pages: &HashMap<Uuid, Page>) -> usize {
    let mut depth = 0;
    let mut current_id = Some(page_id);

    while let Some(id) = current_id {
        if let Some(page) = pages.get(&id) {
            depth += 1;
            current_id = page.parent_id;
        } else {
            break;
        }
    }

    depth
}

pub fn get_all_descendants(page_id: Uuid, pages: &HashMap<Uuid, Page>) -> Vec<Uuid> {
    let mut descendants = Vec::new();

    if let Some(page) = pages.get(&page_id) {
        for child_id in &page.children {
            descendants.push(*child_id);
            descendants.extend(get_all_descendants(*child_id, pages));
        }
    }

    descendants
}

pub fn get_all_ancestors(page_id: Uuid, pages: &HashMap<Uuid, Page>) -> Vec<Uuid> {
    let mut ancestors = Vec::new();
    let mut current_id = pages.get(&page_id).and_then(|p| p.parent_id);

    while let Some(id) = current_id {
        ancestors.push(id);
        current_id = pages.get(&id).and_then(|p| p.parent_id);
    }

    ancestors
}

pub fn is_descendant_of(page_id: Uuid, potential_ancestor: Uuid, pages: &HashMap<Uuid, Page>) -> bool {
    let ancestors = get_all_ancestors(page_id, pages);
    ancestors.contains(&potential_ancestor)
}

pub fn can_move_page(
    page_id: Uuid,
    new_parent_id: Option<Uuid>,
    pages: &HashMap<Uuid, Page>,
) -> Result<(), String> {
    if let Some(new_pid) = new_parent_id {
        if page_id == new_pid {
            return Err("Cannot move page into itself".to_string());
        }

        if is_descendant_of(new_pid, page_id, pages) {
            return Err("Cannot move page into its own descendant".to_string());
        }
    }

    Ok(())
}

pub fn check_page_permission(
    page: &Page,
    user_id: Uuid,
    user_role: WorkspaceRole,
    required_permission: PagePermissionType,
) -> bool {
    if page.permissions.public {
        match required_permission {
            PagePermissionType::View => return true,
            PagePermissionType::Edit => {
                if page.permissions.public_edit {
                    return true;
                }
            }
            _ => {}
        }
    }

    if page.permissions.allowed_users.contains(&user_id) {
        return true;
    }

    if page.permissions.allowed_roles.contains(&user_role) {
        return true;
    }

    match user_role {
        WorkspaceRole::Owner | WorkspaceRole::Admin => true,
        WorkspaceRole::Editor => matches!(
            required_permission,
            PagePermissionType::View | PagePermissionType::Edit | PagePermissionType::Comment
        ),
        WorkspaceRole::Commenter => matches!(
            required_permission,
            PagePermissionType::View | PagePermissionType::Comment
        ),
        WorkspaceRole::Viewer => matches!(required_permission, PagePermissionType::View),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagePermissionType {
    View,
    Edit,
    Comment,
    Share,
    Delete,
}

pub fn duplicate_page(
    page: &Page,
    new_parent_id: Option<Uuid>,
    new_workspace_id: Option<Uuid>,
    new_title: Option<String>,
    created_by: Uuid,
    pages: &HashMap<Uuid, Page>,
    include_children: bool,
) -> Vec<Page> {
    let mut duplicated_pages = Vec::new();
    let now = Utc::now();

    let new_page = Page {
        id: Uuid::new_v4(),
        workspace_id: new_workspace_id.unwrap_or(page.workspace_id),
        parent_id: new_parent_id,
        title: new_title.unwrap_or_else(|| format!("{} (Copy)", page.title)),
        icon: page.icon.clone(),
        cover_image: page.cover_image.clone(),
        blocks: page.blocks.clone(),
        children: Vec::new(),
        properties: page.properties.clone(),
        permissions: PagePermissions::default(),
        is_template: false,
        template_id: page.template_id,
        created_at: now,
        updated_at: now,
        created_by,
        last_edited_by: created_by,
    };

    let new_page_id = new_page.id;
    duplicated_pages.push(new_page);

    if include_children {
        for child_id in &page.children {
            if let Some(child_page) = pages.get(child_id) {
                let child_duplicates = duplicate_page(
                    child_page,
                    Some(new_page_id),
                    new_workspace_id,
                    None,
                    created_by,
                    pages,
                    true,
                );
                duplicated_pages.extend(child_duplicates);
            }
        }
    }

    duplicated_pages
}

pub fn sort_pages_by_title(pages: &mut [PageSummary]) {
    pages.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
}

pub fn sort_pages_by_updated(pages: &mut [PageSummary]) {
    pages.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
}

pub fn sort_pages_by_created(pages: &mut [PageSummary]) {
    pages.sort_by(|a, b| b.created_at.cmp(&a.created_at));
}

pub fn filter_pages_by_date_range(
    pages: Vec<PageSummary>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> Vec<PageSummary> {
    pages
        .into_iter()
        .filter(|p| {
            let after_start = start.map(|s| p.updated_at >= s).unwrap_or(true);
            let before_end = end.map(|e| p.updated_at <= e).unwrap_or(true);
            after_start && before_end
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStats {
    pub total_blocks: usize,
    pub total_words: usize,
    pub total_characters: usize,
    pub has_images: bool,
    pub has_tables: bool,
    pub has_code: bool,
    pub child_count: usize,
    pub comment_count: usize,
}

pub fn calculate_page_stats(page: &Page, comment_count: usize) -> PageStats {
    let mut stats = PageStats {
        total_blocks: 0,
        total_words: 0,
        total_characters: 0,
        has_images: false,
        has_tables: false,
        has_code: false,
        child_count: page.children.len(),
        comment_count,
    };

    count_blocks_stats(&page.blocks, &mut stats);

    stats
}

fn count_blocks_stats(blocks: &[Block], stats: &mut PageStats) {
    for block in blocks {
        stats.total_blocks += 1;

        match block.block_type {
            BlockType::Image => stats.has_images = true,
            BlockType::Table => stats.has_tables = true,
            BlockType::Code => stats.has_code = true,
            _ => {}
        }

        if let BlockContent::Text { text: rich_text } = &block.content {
            for segment in &rich_text.segments {
                stats.total_characters += segment.text.len();
                stats.total_words += segment.text.split_whitespace().count();
            }
        }

        count_blocks_stats(&block.children, stats);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub blocks: Vec<Block>,
    pub properties: HashMap<String, TemplateProperty>,
    pub category: TemplateCategory,
    pub tags: Vec<String>,
    pub is_system: bool,
    pub organization_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub use_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateProperty {
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: Option<serde_json::Value>,
    pub required: bool,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Text,
    Number,
    Date,
    Select,
    MultiSelect,
    Checkbox,
    Url,
    Email,
    Person,
    Files,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Meeting,
    Project,
    Documentation,
    Planning,
    Personal,
    Team,
    Marketing,
    Engineering,
    Sales,
    Hr,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub settings: WorkspaceSettings,
    pub page_templates: Vec<PageTemplateRef>,
    pub default_structure: Vec<PageStructure>,
    pub category: TemplateCategory,
    pub is_system: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTemplateRef {
    pub template_id: Uuid,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStructure {
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub template_id: Option<Uuid>,
    pub children: Vec<PageStructure>,
}

#[derive(Debug, Clone)]
pub enum TemplateError {
    TemplateNotFound,
    CannotModifySystemTemplate,
    InvalidTemplate(String),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemplateNotFound => write!(f, "Template not found"),
            Self::CannotModifySystemTemplate => write!(f, "Cannot modify system template"),
            Self::InvalidTemplate(e) => write!(f, "Invalid template: {e}"),
        }
    }
}

impl std::error::Error for TemplateError {}

pub fn clone_blocks_with_new_ids(blocks: &[Block], created_by: Uuid) -> Vec<Block> {
    let now = Utc::now();
    blocks
        .iter()
        .map(|block| {
            let mut new_block = block.clone();
            new_block.id = Uuid::new_v4();
            new_block.created_at = now;
            new_block.updated_at = now;
            new_block.created_by = created_by;
            new_block.children = clone_blocks_with_new_ids(&block.children, created_by);
            new_block
        })
        .collect()
}

pub fn apply_template_to_page(
    template: &PageTemplate,
    workspace_id: Uuid,
    parent_id: Option<Uuid>,
    title: Option<String>,
    created_by: Uuid,
) -> Page {
    let now = Utc::now();
    Page {
        id: Uuid::new_v4(),
        workspace_id,
        parent_id,
        title: title.unwrap_or_else(|| template.name.clone()),
        icon: template.icon.clone(),
        cover_image: template.cover_image.clone(),
        blocks: clone_blocks_with_new_ids(&template.blocks, created_by),
        children: Vec::new(),
        properties: HashMap::new(),
        permissions: PagePermissions::default(),
        is_template: false,
        template_id: Some(template.id),
        created_at: now,
        updated_at: now,
        created_by,
        last_edited_by: created_by,
    }
}

pub fn get_system_templates() -> Vec<PageTemplate> {
    let system_user = Uuid::nil();
    let now = Utc::now();

    vec![
        PageTemplate {
            id: Uuid::new_v4(),
            name: "Meeting Notes".to_string(),
            description: "Template for meeting notes with agenda and action items".to_string(),
            icon: None,
            cover_image: None,
            blocks: vec![],
            properties: HashMap::new(),
            category: TemplateCategory::Meeting,
            tags: vec!["meeting".to_string(), "notes".to_string()],
            is_system: true,
            organization_id: None,
            workspace_id: None,
            created_by: system_user,
            created_at: now,
            updated_at: now,
            use_count: 0,
        },
        PageTemplate {
            id: Uuid::new_v4(),
            name: "Project Brief".to_string(),
            description: "Template for project briefs and planning".to_string(),
            icon: None,
            cover_image: None,
            blocks: vec![],
            properties: HashMap::new(),
            category: TemplateCategory::Project,
            tags: vec!["project".to_string(), "planning".to_string()],
            is_system: true,
            organization_id: None,
            workspace_id: None,
            created_by: system_user,
            created_at: now,
            updated_at: now,
            use_count: 0,
        },
        PageTemplate {
            id: Uuid::new_v4(),
            name: "Documentation".to_string(),
            description: "Template for technical documentation".to_string(),
            icon: None,
            cover_image: None,
            blocks: vec![],
            properties: HashMap::new(),
            category: TemplateCategory::Documentation,
            tags: vec!["docs".to_string(), "technical".to_string()],
            is_system: true,
            organization_id: None,
            workspace_id: None,
            created_by: system_user,
            created_at: now,
            updated_at: now,
            use_count: 0,
        },
    ]
}

#[derive(Debug, Deserialize)]
pub struct UiListQuery {
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UiPageListQuery {
    pub parent_id: Option<Uuid>,
}

fn ui_get_bot_context(state: &WorkspacesState) -> (Uuid, Uuid) {
    get_bot_context(state)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn render_empty_state(icon: &str, title: &str, description: &str) -> String {
    format!(
        r##"<div class="empty-state">
<div class="empty-icon">{icon}</div>
<h3>{title}</h3>
<p>{description}</p>
</div>"##
    )
}

fn render_workspace_card(workspace: &DbWorkspace, member_count: i64, page_count: i64) -> String {
    let name = html_escape(&workspace.name);
    let description = workspace
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No description".to_string());
    let updated = workspace.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = workspace.id;
    let icon = workspace.icon_value.as_deref().unwrap_or("📁");

    format!(
        r##"<div class="workspace-card" data-id="{id}">
<div class="workspace-icon">{icon}</div>
<div class="workspace-info">
<h4 class="workspace-name">{name}</h4>
<p class="workspace-description">{description}</p>
<div class="workspace-meta">
<span class="workspace-members">{member_count} members</span>
<span class="workspace-pages">{page_count} pages</span>
<span class="workspace-updated">{updated}</span>
</div>
</div>
<div class="workspace-actions">
<button class="btn btn-sm btn-primary" hx-get="/api/ui/workspaces/{id}/pages" hx-target="#workspace-content" hx-swap="innerHTML">
Open
</button>
<button class="btn btn-sm btn-secondary" hx-get="/api/ui/workspaces/{id}/settings" hx-target="#modal-content" hx-swap="innerHTML">
Settings
</button>
<button class="btn btn-sm btn-danger" hx-delete="/api/workspaces/{id}" hx-confirm="Delete this workspace?" hx-swap="none">
Delete
</button>
</div>
</div>"##
    )
}

fn render_workspace_row(workspace: &DbWorkspace, member_count: i64, page_count: i64) -> String {
    let name = html_escape(&workspace.name);
    let description = workspace
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "-".to_string());
    let updated = workspace.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let id = workspace.id;
    let icon = workspace.icon_value.as_deref().unwrap_or("📁");

    format!(
        r##"<tr class="workspace-row" data-id="{id}">
<td class="workspace-icon">{icon}</td>
<td class="workspace-name">
<a href="#" hx-get="/api/ui/workspaces/{id}/pages" hx-target="#workspace-content" hx-swap="innerHTML">{name}</a>
</td>
<td class="workspace-description">{description}</td>
<td class="workspace-members">{member_count}</td>
<td class="workspace-pages">{page_count}</td>
<td class="workspace-updated">{updated}</td>
<td class="workspace-actions">
<button class="btn btn-xs btn-primary" hx-get="/api/ui/workspaces/{id}/pages" hx-target="#workspace-content">Open</button>
<button class="btn btn-xs btn-danger" hx-delete="/api/workspaces/{id}" hx-confirm="Delete?" hx-swap="none">Delete</button>
</td>
</tr>"##
    )
}

fn render_page_item(page: &DbWorkspacePage, child_count: i64) -> String {
    let title = html_escape(&page.title);
    let id = page.id;
    let workspace_id = page.workspace_id;
    let icon = page.icon_value.as_deref().unwrap_or("📄");
    let updated = page.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let has_children = if child_count > 0 {
        format!(
            r##"<button class="btn-expand" hx-get="/api/ui/workspaces/{workspace_id}/pages?parent_id={id}" hx-target="#children-{id}" hx-swap="innerHTML">
<span class="expand-icon">▶</span>
</button>"##
        )
    } else {
        r##"<span class="no-expand"></span>"##.to_string()
    };

    format!(
        r##"<div class="page-item" data-id="{id}">
<div class="page-row">
{has_children}
<span class="page-icon">{icon}</span>
<a class="page-title" href="#" hx-get="/api/ui/pages/{id}" hx-target="#page-content" hx-swap="innerHTML">{title}</a>
<span class="page-updated">{updated}</span>
<div class="page-actions">
<button class="btn btn-xs" hx-get="/api/ui/pages/{id}/edit" hx-target="#modal-content">Edit</button>
<button class="btn btn-xs btn-danger" hx-delete="/api/pages/{id}" hx-confirm="Delete?" hx-swap="none">Delete</button>
</div>
</div>
<div class="page-children" id="children-{id}"></div>
</div>"##
    )
}

pub async fn workspace_list(
    State(state): State<Arc<WorkspacesState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = ui_get_bot_context(&state);

    let mut q = workspaces::table
        .filter(workspaces::org_id.eq(org_id))
        .filter(workspaces::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            workspaces::name
                .ilike(pattern.clone())
                .or(workspaces::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = match q
        .order(workspaces::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load workspaces"));
        }
    };

    if db_workspaces.is_empty() {
        return Html(render_empty_state(
            "📁",
            "No Workspaces",
            "Create your first workspace to get started",
        ));
    }

    let mut rows = String::new();
    for workspace in &db_workspaces {
        let member_count: i64 = workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let page_count: i64 = workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        rows.push_str(&render_workspace_row(workspace, member_count, page_count));
    }

    Html(format!(
        r##"<table class="table workspace-table">
<thead>
<tr>
<th></th>
<th>Name</th>
<th>Description</th>
<th>Members</th>
<th>Pages</th>
<th>Updated</th>
<th>Actions</th>
</tr>
</thead>
<tbody>{rows}</tbody>
</table>"##
    ))
}

pub async fn workspace_cards(
    State(state): State<Arc<WorkspacesState>>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let (org_id, bot_id) = ui_get_bot_context(&state);

    let mut q = workspaces::table
        .filter(workspaces::org_id.eq(org_id))
        .filter(workspaces::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(search) = &query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            workspaces::name
                .ilike(pattern.clone())
                .or(workspaces::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = match q
        .order(workspaces::updated_at.desc())
        .limit(50)
        .load(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("⚠️", "Error", "Failed to load workspaces"));
        }
    };

    if db_workspaces.is_empty() {
        return Html(render_empty_state(
            "📁",
            "No Workspaces",
            "Create your first workspace to get started",
        ));
    }

    let mut cards = String::new();
    for workspace in &db_workspaces {
        let member_count: i64 = workspace_members::table
            .filter(workspace_members::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let page_count: i64 = workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        cards.push_str(&render_workspace_card(workspace, member_count, page_count));
    }

    Html(format!(r##"<div class="workspace-grid">{cards}</div>"##))
}

pub async fn workspace_count(State(state): State<Arc<WorkspacesState>>) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = ui_get_bot_context(&state);

    let count: i64 = workspaces::table
        .filter(workspaces::org_id.eq(org_id))
        .filter(workspaces::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

pub async fn workspace_detail(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let workspace: DbWorkspace = match workspaces::table
        .filter(workspaces::id.eq(workspace_id))
        .first(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Workspace not found"));
        }
    };

    let member_count: i64 = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let page_count: i64 = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let name = html_escape(&workspace.name);
    let description = workspace
        .description
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No description".to_string());
    let icon = workspace.icon_value.as_deref().unwrap_or("📁");
    let created = workspace.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated = workspace.updated_at.format("%Y-%m-%d %H:%M").to_string();

    Html(format!(
        r##"<div class="workspace-detail">
<div class="workspace-header">
<span class="workspace-icon-large">{icon}</span>
<div class="workspace-title">
<h2>{name}</h2>
<p class="workspace-description">{description}</p>
</div>
</div>
<div class="workspace-stats">
<div class="stat">
<span class="stat-label">Members</span>
<span class="stat-value">{member_count}</span>
</div>
<div class="stat">
<span class="stat-label">Pages</span>
<span class="stat-value">{page_count}</span>
</div>
</div>
<div class="workspace-dates">
<span>Created: {created}</span>
<span>Updated: {updated}</span>
</div>
<div class="workspace-actions">
<button class="btn btn-primary" hx-get="/api/ui/workspaces/{workspace_id}/pages" hx-target="#workspace-content" hx-swap="innerHTML">
View Pages
</button>
<button class="btn btn-secondary" hx-get="/api/ui/workspaces/{workspace_id}/members" hx-target="#workspace-content" hx-swap="innerHTML">
Manage Members
</button>
<button class="btn btn-secondary" hx-get="/api/ui/workspaces/{workspace_id}/settings" hx-target="#modal-content" hx-swap="innerHTML">
Settings
</button>
</div>
</div>"##
    ))
}

pub async fn ui_workspace_pages(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<UiPageListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let pages: Vec<DbWorkspacePage> = match query.parent_id {
        Some(parent_id) => workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace_id))
            .filter(workspace_pages::parent_id.eq(parent_id))
            .order(workspace_pages::position.asc())
            .load(&mut conn)
            .unwrap_or_default(),
        None => workspace_pages::table
            .filter(workspace_pages::workspace_id.eq(workspace_id))
            .filter(workspace_pages::parent_id.is_null())
            .order(workspace_pages::position.asc())
            .load(&mut conn)
            .unwrap_or_default(),
    };

    if pages.is_empty() && query.parent_id.is_none() {
        return Html(render_empty_state(
            "📄",
            "No Pages",
            "Create your first page to get started",
        ));
    }

    let mut items = String::new();
    for page in &pages {
        let child_count: i64 = workspace_pages::table
            .filter(workspace_pages::parent_id.eq(page.id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        items.push_str(&render_page_item(page, child_count));
    }

    if query.parent_id.is_some() {
        Html(items)
    } else {
        Html(format!(
            r##"<div class="workspace-pages-header">
<h3>Pages</h3>
<button class="btn btn-primary" hx-get="/api/ui/workspaces/{workspace_id}/pages/new" hx-target="#modal-content" hx-swap="innerHTML">
New Page
</button>
</div>
<div class="page-tree">{items}</div>"##
        ))
    }
}

pub async fn ui_workspace_members(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let members: Vec<DbWorkspaceMember> = workspace_members::table
        .filter(workspace_members::workspace_id.eq(workspace_id))
        .order(workspace_members::joined_at.asc())
        .load(&mut conn)
        .unwrap_or_default();

    if members.is_empty() {
        return Html(render_empty_state(
            "👥",
            "No Members",
            "This workspace has no members",
        ));
    }

    let mut rows = String::new();
    for member in &members {
        let user_id = member.user_id;
        let role = html_escape(&member.role);
        let joined = member.joined_at.format("%Y-%m-%d").to_string();
        let role_class = match role.as_str() {
            "owner" => "badge-primary",
            "admin" => "badge-warning",
            "editor" => "badge-info",
            _ => "badge-secondary",
        };

        rows.push_str(&format!(
            r##"<tr class="member-row" data-user-id="{user_id}">
<td class="member-user">{user_id}</td>
<td class="member-role"><span class="badge {role_class}">{role}</span></td>
<td class="member-joined">{joined}</td>
<td class="member-actions">
<button class="btn btn-xs btn-danger" hx-delete="/api/workspaces/{workspace_id}/members/{user_id}" hx-confirm="Remove member?" hx-swap="none">
Remove
</button>
</td>
</tr>"##
        ));
    }

    Html(format!(
        r##"<div class="workspace-members-header">
<h3>Members</h3>
<button class="btn btn-primary" hx-get="/api/ui/workspaces/{workspace_id}/members/add" hx-target="#modal-content" hx-swap="innerHTML">
Add Member
</button>
</div>
<table class="table members-table">
<thead>
<tr>
<th>User</th>
<th>Role</th>
<th>Joined</th>
<th>Actions</th>
</tr>
</thead>
<tbody>{rows}</tbody>
</table>"##
    ))
}

pub async fn page_detail(
    State(state): State<Arc<WorkspacesState>>,
    Path(page_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let page: DbWorkspacePage = match workspace_pages::table
        .filter(workspace_pages::id.eq(page_id))
        .first(&mut conn)
    {
        Ok(p) => p,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Page not found"));
        }
    };

    let title = html_escape(&page.title);
    let icon = page.icon_value.as_deref().unwrap_or("📄");
    let created = page.created_at.format("%Y-%m-%d %H:%M").to_string();
    let updated = page.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let workspace_id = page.workspace_id;

    let content_preview = if page.content.is_null() || page.content == serde_json::json!([]) {
        r##"<p class="text-muted">This page is empty. Click Edit to add content.</p>"##.to_string()
    } else {
        r##"<div class="page-blocks" id="page-blocks" hx-get="/api/ui/pages/{page_id}/blocks" hx-trigger="load" hx-swap="innerHTML"></div>"##
            .to_string()
            .replace("{page_id}", &page_id.to_string())
    };

    Html(format!(
        r##"<div class="page-detail">
<div class="page-header">
<div class="page-breadcrumb" hx-get="/api/ui/pages/{page_id}/breadcrumb" hx-trigger="load" hx-swap="innerHTML"></div>
<div class="page-title-row">
<span class="page-icon-large">{icon}</span>
<h2 class="page-title">{title}</h2>
</div>
</div>
<div class="page-meta">
<span>Created: {created}</span>
<span>Updated: {updated}</span>
</div>
<div class="page-actions">
<button class="btn btn-primary" hx-get="/api/ui/pages/{page_id}/edit" hx-target="#modal-content" hx-swap="innerHTML">
Edit
</button>
<button class="btn btn-secondary" hx-get="/api/ui/workspaces/{workspace_id}/pages/new?parent_id={page_id}" hx-target="#modal-content" hx-swap="innerHTML">
Add Subpage
</button>
<button class="btn btn-danger" hx-delete="/api/pages/{page_id}" hx-confirm="Delete this page?" hx-swap="none">
Delete
</button>
</div>
<div class="page-content">
{content_preview}
</div>
</div>"##
    ))
}

pub async fn new_workspace_form(State(_state): State<Arc<WorkspacesState>>) -> Html<String> {
    Html(
        r##"<div class="modal-header">
<h3>New Workspace</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="workspace-form" hx-post="/api/workspaces" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#workspace-list', 'refresh');">
<div class="form-group">
<label>Name</label>
<input type="text" name="name" placeholder="My Workspace" required />
</div>
<div class="form-group">
<label>Description</label>
<textarea name="description" rows="3" placeholder="Describe your workspace..."></textarea>
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Create Workspace</button>
</div>
</form>"##
            .to_string(),
    )
}

pub async fn new_page_form(
    State(_state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<UiPageListQuery>,
) -> Html<String> {
    let parent_input = match query.parent_id {
        Some(parent_id) => format!(r##"<input type="hidden" name="parent_id" value="{parent_id}" />"##),
        None => String::new(),
    };

    Html(format!(
        r##"<div class="modal-header">
<h3>New Page</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="page-form" hx-post="/api/workspaces/{workspace_id}/pages" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#page-tree', 'refresh');">
{parent_input}
<div class="form-group">
<label>Title</label>
<input type="text" name="title" placeholder="Page Title" required />
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Create Page</button>
</div>
</form>"##
    ))
}

pub async fn workspace_settings(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let workspace: DbWorkspace = match workspaces::table
        .filter(workspaces::id.eq(workspace_id))
        .first(&mut conn)
    {
        Ok(w) => w,
        Err(_) => {
            return Html(render_empty_state("❌", "Not Found", "Workspace not found"));
        }
    };

    let name = html_escape(&workspace.name);
    let description = workspace.description.as_deref().map(html_escape).unwrap_or_default();

    Html(format!(
        r##"<div class="modal-header">
<h3>Workspace Settings</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="workspace-settings-form" hx-put="/api/workspaces/{workspace_id}" hx-swap="none" hx-on::after-request="closeModal()">
<div class="form-group">
<label>Name</label>
<input type="text" name="name" value="{name}" required />
</div>
<div class="form-group">
<label>Description</label>
<textarea name="description" rows="3">{description}</textarea>
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Save Changes</button>
</div>
</form>"##
    ))
}

pub async fn add_member_form(
    State(_state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
) -> Html<String> {
    Html(format!(
        r##"<div class="modal-header">
<h3>Add Member</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="add-member-form" hx-post="/api/workspaces/{workspace_id}/members" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#members-table', 'refresh');">
<div class="form-group">
<label>User ID</label>
<input type="text" name="user_id" placeholder="User UUID" required />
</div>
<div class="form-group">
<label>Role</label>
<select name="role" required>
<option value="viewer">Viewer</option>
<option value="commenter">Commenter</option>
<option value="editor">Editor</option>
<option value="admin">Admin</option>
</select>
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Add Member</button>
</div>
</form>"##
    ))
}

pub async fn search_results(
    State(state): State<Arc<WorkspacesState>>,
    Path(workspace_id): Path<Uuid>,
    Query(query): Query<UiListQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(render_empty_state("⚠️", "Database Error", "Could not connect to database"));
    };

    let search_term = match &query.search {
        Some(s) if !s.is_empty() => s,
        _ => {
            return Html(render_empty_state("🔍", "Search", "Enter a search term"));
        }
    };

    let pattern = format!("%{search_term}%");
    let pages: Vec<DbWorkspacePage> = workspace_pages::table
        .filter(workspace_pages::workspace_id.eq(workspace_id))
        .filter(workspace_pages::title.ilike(&pattern))
        .order(workspace_pages::updated_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if pages.is_empty() {
        return Html(render_empty_state(
            "🔍",
            "No Results",
            "No pages match your search",
        ));
    }

    let mut items = String::new();
    for page in &pages {
        let title = html_escape(&page.title);
        let id = page.id;
        let icon = page.icon_value.as_deref().unwrap_or("📄");
        let updated = page.updated_at.format("%Y-%m-%d %H:%M").to_string();

        items.push_str(&format!(
            r##"<div class="search-result" data-id="{id}">
<span class="result-icon">{icon}</span>
<a class="result-title" href="#" hx-get="/api/ui/pages/{id}" hx-target="#page-content" hx-swap="innerHTML">{title}</a>
<span class="result-updated">{updated}</span>
</div>"##
        ));
    }

    Html(format!(
        r##"<div class="search-results">
<h4>Search Results ({count})</h4>
{items}
</div>"##,
        count = pages.len()
    ))
}

pub fn configure_workspaces_ui_routes() -> Router<Arc<WorkspacesState>> {
    Router::new()
        .route("/api/ui/workspaces", get(workspace_list))
        .route("/api/ui/workspaces/cards", get(workspace_cards))
        .route("/api/ui/workspaces/count", get(workspace_count))
        .route("/api/ui/workspaces/new", get(new_workspace_form))
        .route("/api/ui/workspaces/{workspace_id}", get(workspace_detail))
        .route("/api/ui/workspaces/{workspace_id}/pages", get(ui_workspace_pages))
        .route("/api/ui/workspaces/{workspace_id}/pages/new", get(new_page_form))
        .route("/api/ui/workspaces/{workspace_id}/members", get(ui_workspace_members))
        .route("/api/ui/workspaces/{workspace_id}/members/add", get(add_member_form))
        .route("/api/ui/workspaces/{workspace_id}/settings", get(workspace_settings))
        .route("/api/ui/workspaces/{workspace_id}/search", get(search_results))
        .route("/api/ui/pages/{page_id}", get(page_detail))
}
