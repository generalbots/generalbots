use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;

pub mod blocks;
pub mod pages;
pub mod collaboration;
pub mod templates;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub members: Vec<WorkspaceMember>,
    pub settings: WorkspaceSettings,
    pub root_pages: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub default_page_width: PageWidth,
    pub allow_public_pages: bool,
    pub enable_comments: bool,
    pub enable_reactions: bool,
    pub enable_gb_assist: bool,
    pub gb_bot_id: Option<Uuid>,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            default_page_width: PageWidth::Normal,
            allow_public_pages: false,
            enable_comments: true,
            enable_reactions: true,
            enable_gb_assist: true,
            gb_bot_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageWidth {
    Small,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagePermissions {
    pub inherit_from_parent: bool,
    pub public: bool,
    pub public_edit: bool,
    pub allowed_users: Vec<Uuid>,
    pub allowed_roles: Vec<WorkspaceRole>,
}

impl Default for PagePermissions {
    fn default() -> Self {
        Self {
            inherit_from_parent: true,
            public: false,
            public_edit: false,
            allowed_users: Vec::new(),
            allowed_roles: Vec::new(),
        }
    }
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
#[serde(untagged)]
pub enum BlockContent {
    Text(RichText),
    Media(MediaContent),
    Table(TableContent),
    Code(CodeContent),
    Embed(EmbedContent),
    Callout(CalloutContent),
    Toggle(ToggleContent),
    Checklist(ChecklistContent),
    GbComponent(GbComponentContent),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichText {
    pub segments: Vec<TextSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    pub text: String,
    pub annotations: TextAnnotations,
    pub link: Option<String>,
    pub mention: Option<Mention>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextAnnotations {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub color: Option<String>,
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
pub struct MediaContent {
    pub url: String,
    pub caption: Option<RichText>,
    pub alt_text: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableContent {
    pub rows: Vec<TableRow>,
    pub has_header_row: bool,
    pub has_header_column: bool,
    pub column_widths: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: Uuid,
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: RichText,
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContent {
    pub code: String,
    pub language: String,
    pub caption: Option<RichText>,
    pub wrap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedContent {
    pub url: String,
    pub embed_type: EmbedType,
    pub caption: Option<RichText>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmbedType {
    Youtube,
    Vimeo,
    Figma,
    GoogleDrive,
    GoogleMaps,
    Twitter,
    Github,
    Codepen,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutContent {
    pub icon: WorkspaceIcon,
    pub text: RichText,
    pub background_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleContent {
    pub title: RichText,
    pub expanded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistContent {
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub text: RichText,
    pub checked: bool,
    pub assignee: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GbComponentContent {
    pub component_type: GbComponentType,
    pub bot_id: Option<Uuid>,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GbComponentType {
    AskGb,
    SummarizePage,
    CreateContent,
    TranslateBlock,
    FormEmbed,
    DataTable,
    Chart,
    KbSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockProperties {
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub indent_level: u8,
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

pub struct WorkspacesService {
    workspaces: Arc<RwLock<HashMap<Uuid, Workspace>>>,
    pages: Arc<RwLock<HashMap<Uuid, Page>>>,
    page_versions: Arc<RwLock<HashMap<Uuid, Vec<PageVersion>>>>,
    comments: Arc<RwLock<HashMap<Uuid, Vec<Comment>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageVersion {
    pub id: Uuid,
    pub page_id: Uuid,
    pub version_number: u32,
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
    pub content: RichText,
    pub resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub reactions: Vec<Reaction>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl WorkspacesService {
    pub fn new() -> Self {
        Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            pages: Arc::new(RwLock::new(HashMap::new())),
            page_versions: Arc::new(RwLock::new(HashMap::new())),
            comments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_workspace(
        &self,
        organization_id: Uuid,
        name: &str,
        created_by: Uuid,
    ) -> Result<Workspace, WorkspacesError> {
        let now = Utc::now();
        let workspace = Workspace {
            id: Uuid::new_v4(),
            organization_id,
            name: name.to_string(),
            description: None,
            icon: None,
            cover_image: None,
            members: vec![WorkspaceMember {
                user_id: created_by,
                role: WorkspaceRole::Owner,
                joined_at: now,
                invited_by: None,
            }],
            settings: WorkspaceSettings::default(),
            root_pages: Vec::new(),
            created_at: now,
            updated_at: now,
            created_by,
        };

        let mut workspaces = self.workspaces.write().await;
        workspaces.insert(workspace.id, workspace.clone());

        Ok(workspace)
    }

    pub async fn get_workspace(&self, workspace_id: Uuid) -> Option<Workspace> {
        let workspaces = self.workspaces.read().await;
        workspaces.get(&workspace_id).cloned()
    }

    pub async fn list_workspaces(&self, organization_id: Uuid) -> Vec<Workspace> {
        let workspaces = self.workspaces.read().await;
        workspaces
            .values()
            .filter(|w| w.organization_id == organization_id)
            .cloned()
            .collect()
    }

    pub async fn list_user_workspaces(&self, user_id: Uuid) -> Vec<Workspace> {
        let workspaces = self.workspaces.read().await;
        workspaces
            .values()
            .filter(|w| w.members.iter().any(|m| m.user_id == user_id))
            .cloned()
            .collect()
    }

    pub async fn update_workspace(
        &self,
        workspace_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        icon: Option<WorkspaceIcon>,
    ) -> Result<Workspace, WorkspacesError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces
            .get_mut(&workspace_id)
            .ok_or(WorkspacesError::WorkspaceNotFound)?;

        if let Some(n) = name {
            workspace.name = n;
        }
        if description.is_some() {
            workspace.description = description;
        }
        if icon.is_some() {
            workspace.icon = icon;
        }
        workspace.updated_at = Utc::now();

        Ok(workspace.clone())
    }

    pub async fn delete_workspace(&self, workspace_id: Uuid) -> Result<(), WorkspacesError> {
        let mut workspaces = self.workspaces.write().await;
        workspaces
            .remove(&workspace_id)
            .ok_or(WorkspacesError::WorkspaceNotFound)?;

        let mut pages = self.pages.write().await;
        pages.retain(|_, p| p.workspace_id != workspace_id);

        Ok(())
    }

    pub async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: WorkspaceRole,
        invited_by: Uuid,
    ) -> Result<(), WorkspacesError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces
            .get_mut(&workspace_id)
            .ok_or(WorkspacesError::WorkspaceNotFound)?;

        if workspace.members.iter().any(|m| m.user_id == user_id) {
            return Err(WorkspacesError::MemberAlreadyExists);
        }

        workspace.members.push(WorkspaceMember {
            user_id,
            role,
            joined_at: Utc::now(),
            invited_by: Some(invited_by),
        });
        workspace.updated_at = Utc::now();

        Ok(())
    }

    pub async fn remove_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), WorkspacesError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces
            .get_mut(&workspace_id)
            .ok_or(WorkspacesError::WorkspaceNotFound)?;

        let owner_count = workspace
            .members
            .iter()
            .filter(|m| m.role == WorkspaceRole::Owner)
            .count();

        if let Some(member) = workspace.members.iter().find(|m| m.user_id == user_id) {
            if member.role == WorkspaceRole::Owner && owner_count <= 1 {
                return Err(WorkspacesError::CannotRemoveLastOwner);
            }
        }

        workspace.members.retain(|m| m.user_id != user_id);
        workspace.updated_at = Utc::now();

        Ok(())
    }

    pub async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        new_role: WorkspaceRole,
    ) -> Result<(), WorkspacesError> {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces
            .get_mut(&workspace_id)
            .ok_or(WorkspacesError::WorkspaceNotFound)?;

        let member = workspace
            .members
            .iter_mut()
            .find(|m| m.user_id == user_id)
            .ok_or(WorkspacesError::MemberNotFound)?;

        member.role = new_role;
        workspace.updated_at = Utc::now();

        Ok(())
    }

    pub async fn create_page(
        &self,
        workspace_id: Uuid,
        parent_id: Option<Uuid>,
        title: &str,
        created_by: Uuid,
    ) -> Result<Page, WorkspacesError> {
        let workspaces = self.workspaces.read().await;
        if !workspaces.contains_key(&workspace_id) {
            return Err(WorkspacesError::WorkspaceNotFound);
        }
        drop(workspaces);

        let now = Utc::now();
        let page = Page {
            id: Uuid::new_v4(),
            workspace_id,
            parent_id,
            title: title.to_string(),
            icon: None,
            cover_image: None,
            blocks: Vec::new(),
            children: Vec::new(),
            properties: HashMap::new(),
            permissions: PagePermissions::default(),
            is_template: false,
            template_id: None,
            created_at: now,
            updated_at: now,
            created_by,
            last_edited_by: created_by,
        };

        let mut pages = self.pages.write().await;
        pages.insert(page.id, page.clone());
        drop(pages);

        if let Some(pid) = parent_id {
            let mut pages = self.pages.write().await;
            if let Some(parent) = pages.get_mut(&pid) {
                parent.children.push(page.id);
                parent.updated_at = Utc::now();
            }
        } else {
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&workspace_id) {
                workspace.root_pages.push(page.id);
                workspace.updated_at = Utc::now();
            }
        }

        Ok(page)
    }

    pub async fn get_page(&self, page_id: Uuid) -> Option<Page> {
        let pages = self.pages.read().await;
        pages.get(&page_id).cloned()
    }

    pub async fn get_page_tree(&self, workspace_id: Uuid) -> Vec<PageTreeNode> {
        let workspaces = self.workspaces.read().await;
        let workspace = match workspaces.get(&workspace_id) {
            Some(w) => w,
            None => return Vec::new(),
        };

        let pages = self.pages.read().await;
        let mut tree = Vec::new();

        for page_id in &workspace.root_pages {
            if let Some(node) = self.build_page_tree_node(*page_id, &pages) {
                tree.push(node);
            }
        }

        tree
    }

    fn build_page_tree_node(
        &self,
        page_id: Uuid,
        pages: &HashMap<Uuid, Page>,
    ) -> Option<PageTreeNode> {
        let page = pages.get(&page_id)?;

        let children: Vec<PageTreeNode> = page
            .children
            .iter()
            .filter_map(|child_id| self.build_page_tree_node(*child_id, pages))
            .collect();

        Some(PageTreeNode {
            id: page.id,
            title: page.title.clone(),
            icon: page.icon.clone(),
            children,
            has_children: !page.children.is_empty(),
        })
    }

    pub async fn update_page(
        &self,
        page_id: Uuid,
        title: Option<String>,
        icon: Option<WorkspaceIcon>,
        cover_image: Option<String>,
        edited_by: Uuid,
    ) -> Result<Page, WorkspacesError> {
        let mut pages = self.pages.write().await;
        let page = pages
            .get_mut(&page_id)
            .ok_or(WorkspacesError::PageNotFound)?;

        if let Some(t) = title {
            page.title = t;
        }
        if icon.is_some() {
            page.icon = icon;
        }
        if cover_image.is_some() {
            page.cover_image = cover_image;
        }
        page.updated_at = Utc::now();
        page.last_edited_by = edited_by;

        Ok(page.clone())
    }

    pub async fn update_page_blocks(
        &self,
        page_id: Uuid,
        blocks: Vec<Block>,
        edited_by: Uuid,
    ) -> Result<Page, WorkspacesError> {
        let old_page = {
            let pages = self.pages.read().await;
            pages.get(&page_id).cloned()
        };

        if let Some(old) = old_page {
            self.save_page_version(&old).await;
        }

        let mut pages = self.pages.write().await;
        let page = pages
            .get_mut(&page_id)
            .ok_or(WorkspacesError::PageNotFound)?;

        page.blocks = blocks;
        page.updated_at = Utc::now();
        page.last_edited_by = edited_by;

        Ok(page.clone())
    }

    async fn save_page_version(&self, page: &Page) {
        let mut versions = self.page_versions.write().await;
        let page_versions = versions.entry(page.id).or_default();

        let version_number = page_versions.len() as u32 + 1;
        let version = PageVersion {
            id: Uuid::new_v4(),
            page_id: page.id,
            version_number,
            title: page.title.clone(),
            blocks: page.blocks.clone(),
            created_at: Utc::now(),
            created_by: page.last_edited_by,
            change_summary: None,
        };

        page_versions.push(version);

        if page_versions.len() > 100 {
            page_versions.remove(0);
        }
    }

    pub async fn get_page_versions(&self, page_id: Uuid) -> Vec<PageVersion> {
        let versions = self.page_versions.read().await;
        versions.get(&page_id).cloned().unwrap_or_default()
    }

    pub async fn restore_page_version(
        &self,
        page_id: Uuid,
        version_id: Uuid,
        restored_by: Uuid,
    ) -> Result<Page, WorkspacesError> {
        let version = {
            let versions = self.page_versions.read().await;
            versions
                .get(&page_id)
                .and_then(|v| v.iter().find(|pv| pv.id == version_id).cloned())
        };

        let version = version.ok_or(WorkspacesError::VersionNotFound)?;

        self.update_page_blocks(page_id, version.blocks, restored_by)
            .await
    }

    pub async fn delete_page(&self, page_id: Uuid) -> Result<(), WorkspacesError> {
        let page = {
            let pages = self.pages.read().await;
            pages.get(&page_id).cloned()
        };

        let page = page.ok_or(WorkspacesError::PageNotFound)?;

        for child_id in &page.children {
            let _ = Box::pin(self.delete_page(*child_id)).await;
        }

        let mut pages = self.pages.write().await;
        pages.remove(&page_id);
        drop(pages);

        if let Some(parent_id) = page.parent_id {
            let mut pages = self.pages.write().await;
            if let Some(parent) = pages.get_mut(&parent_id) {
                parent.children.retain(|id| *id != page_id);
            }
        } else {
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&page.workspace_id) {
                workspace.root_pages.retain(|id| *id != page_id);
            }
        }

        let mut versions = self.page_versions.write().await;
        versions.remove(&page_id);

        let mut comments = self.comments.write().await;
        comments.remove(&page_id);

        Ok(())
    }

    pub async fn move_page(
        &self,
        page_id: Uuid,
        new_parent_id: Option<Uuid>,
        new_workspace_id: Option<Uuid>,
    ) -> Result<Page, WorkspacesError> {
        let mut pages = self.pages.write().await;
        let page = pages
            .get_mut(&page_id)
            .ok_or(WorkspacesError::PageNotFound)?;

        let old_parent_id = page.parent_id;
        let old_workspace_id = page.workspace_id;

        page.parent_id = new_parent_id;
        if let Some(ws_id) = new_workspace_id {
            page.workspace_id = ws_id;
        }
        page.updated_at = Utc::now();

        let page_clone = page.clone();
        drop(pages);

        if let Some(old_pid) = old_parent_id {
            let mut pages = self.pages.write().await;
            if let Some(old_parent) = pages.get_mut(&old_pid) {
                old_parent.children.retain(|id| *id != page_id);
            }
        } else {
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&old_workspace_id) {
                workspace.root_pages.retain(|id| *id != page_id);
            }
        }

        if let Some(new_pid) = new_parent_id {
            let mut pages = self.pages.write().await;
            if let Some(new_parent) = pages.get_mut(&new_pid) {
                if !new_parent.children.contains(&page_id) {
                    new_parent.children.push(page_id);
                }
            }
        } else {
            let ws_id = new_workspace_id.unwrap_or(old_workspace_id);
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&ws_id) {
                if !workspace.root_pages.contains(&page_id) {
                    workspace.root_pages.push(page_id);
                }
            }
        }

        Ok(page_clone)
    }

    pub async fn add_comment(
        &self,
        page_id: Uuid,
        block_id: Option<Uuid>,
        author_id: Uuid,
        content: RichText,
        parent_comment_id: Option<Uuid>,
    ) -> Result<Comment, WorkspacesError> {
        let pages = self.pages.read().await;
        if !pages.contains_key(&page_id) {
            return Err(WorkspacesError::PageNotFound);
        }
        drop(pages);

        let now = Utc::now();
        let comment = Comment {
            id: Uuid::new_v4(),
            page_id,
            block_id,
            parent_comment_id,
            author_id,
            content,
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            reactions: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        let mut comments = self.comments.write().await;
        comments.entry(page_id).or_default().push(comment.clone());

        Ok(comment)
    }

    pub async fn get_page_comments(&self, page_id: Uuid) -> Vec<Comment> {
        let comments = self.comments.read().await;
        comments.get(&page_id).cloned().unwrap_or_default()
    }

    pub async fn resolve_comment(
        &self,
        page_id: Uuid,
        comment_id: Uuid,
        resolved_by: Uuid,
    ) -> Result<Comment, WorkspacesError> {
        let mut comments = self.comments.write().await;
        let page_comments = comments
            .get_mut(&page_id)
            .ok_or(WorkspacesError::CommentNotFound)?;

        let comment = page_comments
            .iter_mut()
            .find(|c| c.id == comment_id)
            .ok_or(WorkspacesError::CommentNotFound)?;

        comment.resolved = true;
        comment.resolved_by = Some(resolved_by);
        comment.resolved_at = Some(Utc::now());
        comment.updated_at = Utc::now();

        Ok(comment.clone())
    }

    pub async fn add_reaction(
        &self,
        page_id: Uuid,
        comment_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<(), WorkspacesError> {
        let mut comments = self.comments.write().await;
        let page_comments = comments
            .get_mut(&page_id)
            .ok_or(WorkspacesError::CommentNotFound)?;

        let comment = page_comments
            .iter_mut()
            .find(|c| c.id == comment_id)
            .ok_or(WorkspacesError::CommentNotFound)?;

        if comment.reactions.iter().any(|r| r.user_id == user_id && r.emoji == emoji) {
            return Ok(());
        }

        comment.reactions.push(Reaction {
            emoji: emoji.to_string(),
            user_id,
            created_at: Utc::now(),
        });

        Ok(())
    }

    pub async fn search_pages(&self, workspace_id: Uuid, query: &str) -> Vec<PageSearchResult> {
        let pages = self.pages.read().await;
        let query_lower = query.to_lowercase();

        pages
            .values()
            .filter(|p| p.workspace_id == workspace_id)
            .filter(|p| {
                p.title.to_lowercase().contains(&query_lower)
                    || self.blocks_contain_text(&p.blocks, &query_lower)
            })
            .map(|p| PageSearchResult {
                page_id: p.id,
                title: p.title.clone(),
                icon: p.icon.clone(),
                snippet: self.extract_snippet(&p.blocks, &query_lower),
                updated_at: p.updated_at,
            })
            .collect()
    }

    fn blocks_contain_text(&self, blocks: &[Block], query: &str) -> bool {
        for block in blocks {
            if let BlockContent::Text(rich_text) = &block.content {
                for segment in &rich_text.segments {
                    if segment.text.to_lowercase().contains(query) {
                        return true;
                    }
                }
            }
            if self.blocks_contain_text(&block.children, query) {
                return true;
            }
        }
        false
    }

    fn extract_snippet(&self, blocks: &[Block], query: &str) -> Option<String> {
        for block in blocks {
            if let BlockContent::Text(rich_text) = &block.content {
                let full_text: String = rich_text.segments.iter().map(|s| s.text.as_str()).collect();
                if full_text.to_lowercase().contains(query) {
                    let max_len = 150;
                    if full_text.len() <= max_len {
                        return Some(full_text);
                    }
                    return Some(format!("{}...", &full_text[..max_len]));
                }
            }
        }
        None
    }
}

impl Default for WorkspacesService {
    fn default() -> Self {
        Self::new()
    }
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
    pub snippet: Option<String>,
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

pub fn get_slash_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand {
            id: "ask_gb".to_string(),
            name: "Ask General Bots".to_string(),
            description: "Ask GB to answer a question using your knowledge base".to_string(),
            icon: "bot".to_string(),
            category: SlashCommandCategory::GbAssist,
            keywords: vec!["ai".to_string(), "ask".to_string(), "question".to_string()],
        },
        SlashCommand {
            id: "create_content".to_string(),
            name: "Create page content".to_string(),
            description: "Use GB to generate content for this page".to_string(),
            icon: "sparkles".to_string(),
            category: SlashCommandCategory::GbAssist,
            keywords: vec!["generate".to_string(), "write".to_string(), "create".to_string()],
        },
        SlashCommand {
            id: "summarize".to_string(),
            name: "Summarize page".to_string(),
            description: "Generate a summary of this page using GB".to_string(),
            icon: "file-text".to_string(),
            category: SlashCommandCategory::GbAssist,
            keywords: vec!["summary".to_string(), "tldr".to_string()],
        },
        SlashCommand {
            id: "translate".to_string(),
            name: "Translate block".to_string(),
            description: "Translate selected content to another language".to_string(),
            icon: "languages".to_string(),
            category: SlashCommandCategory::GbAssist,
            keywords: vec!["language".to_string(), "translate".to_string()],
        },
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
            keywords: vec!["h1".to_string(), "heading".to_string(), "title".to_string()],
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
            id: "heading3".to_string(),
            name: "Heading 3".to_string(),
            description: "Small section heading".to_string(),
            icon: "heading-3".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["h3".to_string(), "heading".to_string()],
        },
        SlashCommand {
            id: "bulleted_list".to_string(),
            name: "Bulleted list".to_string(),
            description: "Create a bulleted list".to_string(),
            icon: "list".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["bullet".to_string(), "list".to_string(), "ul".to_string()],
        },
        SlashCommand {
            id: "numbered_list".to_string(),
            name: "Numbered list".to_string(),
            description: "Create a numbered list".to_string(),
            icon: "list-ordered".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["number".to_string(), "list".to_string(), "ol".to_string()],
        },
        SlashCommand {
            id: "checklist".to_string(),
            name: "Checklist".to_string(),
            description: "Create a checklist with checkboxes".to_string(),
            icon: "check-square".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["todo".to_string(), "checkbox".to_string(), "task".to_string()],
        },
        SlashCommand {
            id: "toggle".to_string(),
            name: "Toggle".to_string(),
            description: "Create a collapsible toggle block".to_string(),
            icon: "chevron-right".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["collapse".to_string(), "expand".to_string(), "toggle".to_string()],
        },
        SlashCommand {
            id: "table".to_string(),
            name: "Table".to_string(),
            description: "Create a table".to_string(),
            icon: "table".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["table".to_string(), "grid".to_string()],
        },
        SlashCommand {
            id: "divider".to_string(),
            name: "Divider".to_string(),
            description: "Create a horizontal divider".to_string(),
            icon: "minus".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["hr".to_string(), "line".to_string(), "separator".to_string()],
        },
        SlashCommand {
            id: "quote".to_string(),
            name: "Quote".to_string(),
            description: "Create a quote block".to_string(),
            icon: "quote".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["blockquote".to_string(), "quote".to_string()],
        },
        SlashCommand {
            id: "callout".to_string(),
            name: "Callout".to_string(),
            description: "Create a callout block with icon".to_string(),
            icon: "alert-circle".to_string(),
            category: SlashCommandCategory::General,
            keywords: vec!["callout".to_string(), "note".to_string(), "tip".to_string()],
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
            keywords: vec!["image".to_string(), "picture".to_string(), "photo".to_string()],
        },
        SlashCommand {
            id: "video".to_string(),
            name: "Video".to_string(),
            description: "Embed a video".to_string(),
            icon: "video".to_string(),
            category: SlashCommandCategory::Media,
            keywords: vec!["video".to_string(), "youtube".to_string()],
        },
        SlashCommand {
            id: "file".to_string(),
            name: "File".to_string(),
            description: "Upload a file".to_string(),
            icon: "file".to_string(),
            category: SlashCommandCategory::Media,
            keywords: vec!["file".to_string(), "upload".to_string(), "attachment".to_string()],
        },
        SlashCommand {
            id: "embed".to_string(),
            name: "Embed".to_string(),
            description: "Embed external content".to_string(),
            icon: "globe".to_string(),
            category: SlashCommandCategory::Embed,
            keywords: vec!["embed".to_string(), "iframe".to_string()],
        },
        SlashCommand {
            id: "link_to_page".to_string(),
            name: "Link to page".to_string(),
            description: "Create a link to another page".to_string(),
            icon: "link".to_string(),
            category: SlashCommandCategory::Advanced,
            keywords: vec!["link".to_string(), "page".to_string()],
        },
        SlashCommand {
            id: "toc".to_string(),
            name: "Table of contents".to_string(),
            description: "Generate a table of contents".to_string(),
            icon: "list-tree".to_string(),
            category: SlashCommandCategory::Advanced,
            keywords: vec!["toc".to_string(), "contents".to_string()],
        },
    ]
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
        }
    }
}

impl std::error::Error for WorkspacesError {}

impl IntoResponse for WorkspacesError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Self::WorkspaceNotFound | Self::PageNotFound | Self::BlockNotFound
            | Self::CommentNotFound | Self::VersionNotFound | Self::MemberNotFound => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Self::PermissionDenied => (StatusCode::FORBIDDEN, self.to_string()),
            Self::MemberAlreadyExists | Self::CannotRemoveLastOwner | Self::InvalidOperation(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
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
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: WorkspaceRole,
}

async fn list_workspaces(
    State(_state): State<Arc<AppState>>,
) -> Json<Vec<Workspace>> {
    let service = WorkspacesService::new();
    let org_id = Uuid::nil();
    let workspaces = service.list_workspaces(org_id).await;
    Json(workspaces)
}

async fn create_workspace(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let service = WorkspacesService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let workspace = service.create_workspace(org_id, &req.name, user_id).await?;
    Ok(Json(workspace))
}

async fn get_workspace(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let service = WorkspacesService::new();
    let workspace = service.get_workspace(workspace_id).await.ok_or(WorkspacesError::WorkspaceNotFound)?;
    Ok(Json(workspace))
}

async fn update_workspace(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>, WorkspacesError> {
    let service = WorkspacesService::new();
    let workspace = service.update_workspace(workspace_id, req.name, req.description, req.icon).await?;
    Ok(Json(workspace))
}

async fn delete_workspace(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<StatusCode, WorkspacesError> {
    let service = WorkspacesService::new();
    service.delete_workspace(workspace_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_pages(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Json<Vec<PageTreeNode>> {
    let service = WorkspacesService::new();
    let pages = service.get_page_tree(workspace_id).await;
    Json(pages)
}

async fn create_page(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<Page>, WorkspacesError> {
    let service = WorkspacesService::new();
    let user_id = Uuid::nil();
    let page = service.create_page(workspace_id, req.parent_id, &req.title, user_id).await?;
    Ok(Json(page))
}

async fn get_page(
    State(_state): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
) -> Result<Json<Page>, WorkspacesError> {
    let service = WorkspacesService::new();
    let page = service.get_page(page_id).await.ok_or(WorkspacesError::PageNotFound)?;
    Ok(Json(page))
}

async fn update_page(
    State(_state): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
    Json(req): Json<UpdatePageRequest>,
) -> Result<Json<Page>, WorkspacesError> {
    let service = WorkspacesService::new();
    let user_id = Uuid::nil();
    let page = service.update_page(page_id, req.title, req.icon, None, user_id).await?;
    Ok(Json(page))
}

async fn delete_page(
    State(_state): State<Arc<AppState>>,
    Path(page_id): Path<Uuid>,
) -> Result<StatusCode, WorkspacesError> {
    let service = WorkspacesService::new();
    service.delete_page(page_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_member(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<StatusCode, WorkspacesError> {
    let service = WorkspacesService::new();
    let inviter_id = Uuid::nil();
    service.add_member(workspace_id, req.user_id, req.role, inviter_id).await?;
    Ok(StatusCode::CREATED)
}

async fn remove_member(
    State(_state): State<Arc<AppState>>,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, WorkspacesError> {
    let service = WorkspacesService::new();
    service.remove_member(workspace_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn search_pages(
    State(_state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Json<Vec<PageSearchResult>> {
    let service = WorkspacesService::new();
    let query = params.get("q").cloned().unwrap_or_default();
    let results = service.search_pages(workspace_id, &query).await;
    Json(results)
}

async fn get_slash_commands_handler(
    State(_state): State<Arc<AppState>>,
) -> Json<Vec<SlashCommand>> {
    Json(get_slash_commands())
}

pub fn configure_workspaces_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/workspaces", get(list_workspaces).post(create_workspace))
        .route(
            "/api/workspaces/:workspace_id",
            get(get_workspace).put(update_workspace).delete(delete_workspace),
        )
        .route(
            "/api/workspaces/:workspace_id/pages",
            get(list_pages).post(create_page),
        )
        .route(
            "/api/workspaces/:workspace_id/members",
            post(add_member),
        )
        .route(
            "/api/workspaces/:workspace_id/members/:user_id",
            delete(remove_member),
        )
        .route(
            "/api/workspaces/:workspace_id/search",
            get(search_pages),
        )
        .route("/api/pages/:page_id", get(get_page).put(update_page).delete(delete_page))
        .route("/api/workspaces/commands", get(get_slash_commands_handler))
}
