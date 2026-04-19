use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::core::shared::schema::workspaces::{
    workspace_comments, workspace_members, workspace_page_versions, workspace_pages,
    workspaces as workspaces_table,
};
use crate::core::shared::state::AppState;

pub mod blocks;
pub mod collaboration;
pub mod pages;
pub mod templates;
pub mod ui;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = workspaces_table)]
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
    fn as_str(&self) -> &'static str {
        match self {
            Self::Emoji => "emoji",
            Self::Image => "image",
            Self::Lucide => "lucide",
        }
    }

    fn from_str(s: &str) -> Self {
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
    fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Commenter => "commenter",
            Self::Viewer => "viewer",
        }
    }

    fn from_str(s: &str) -> Self {
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

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

fn db_to_workspace(db: DbWorkspace, members: Vec<WorkspaceMember>, root_pages: Vec<Uuid>) -> Workspace {
    let icon = match (&db.icon_type, &db.icon_value) {
        (Some(t), Some(v)) => Some(WorkspaceIcon {
            icon_type: IconType::from_str(t),
            value: v.clone(),
        }),
        _ => None,
    };
    let settings: WorkspaceSettings =
        serde_json::from_value(db.settings).unwrap_or_default();

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
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Workspace>>, WorkspacesError> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = workspaces_table::table
        .filter(workspaces_table::org_id.eq(org_id))
        .filter(workspaces_table::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            workspaces_table::name
                .ilike(pattern.clone())
                .or(workspaces_table::description.ilike(pattern)),
        );
    }

    let db_workspaces: Vec<DbWorkspace> = q
        .order(workspaces_table::updated_at.desc())
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let mut conn = state
        .conn
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

    diesel::insert_into(workspaces_table::table)
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
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let db_workspace: DbWorkspace = workspaces_table::table
        .filter(workspaces_table::id.eq(workspace_id))
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
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let mut db_workspace: DbWorkspace = workspaces_table::table
        .filter(workspaces_table::id.eq(workspace_id))
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

    diesel::update(workspaces_table::table.filter(workspaces_table::id.eq(workspace_id)))
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
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .conn
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
            workspace_page_versions::page_id.eq_any(&page_ids)
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

    let deleted = diesel::delete(workspaces_table::table.filter(workspaces_table::id.eq(workspace_id)))
        .execute(&mut conn)
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WorkspacesError::WorkspaceNotFound)
    }
}

async fn list_pages(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<PageTreeNode>>, WorkspacesError> {
    let mut conn = state
        .conn
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
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<Page>, WorkspacesError> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| WorkspacesError::DbError(e.to_string()))?;

    let _: DbWorkspace = workspaces_table::table
        .filter(workspaces_table::id.eq(workspace_id))
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
    State(state): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
) -> Result<Json<Page>, WorkspacesError> {
    let mut conn = state
        .conn
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
    State(state): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
    Json(req): Json<UpdatePageRequest>,
) -> Result<Json<Page>, WorkspacesError> {
    let mut conn = state
        .conn
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
    State(state): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .conn
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
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .conn
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
    State(state): State<Arc<AppState>>,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, WorkspacesError> {
    let mut conn = state
        .conn
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
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<PageSearchResult>>, WorkspacesError> {
    let mut conn = state
        .conn
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
    State(_state): State<Arc<AppState>>,
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

pub fn configure_workspaces_routes() -> Router<Arc<AppState>> {
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
