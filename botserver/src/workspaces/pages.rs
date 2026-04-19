use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{Block, Page, PagePermissions, PropertyValue, WorkspaceIcon, WorkspaceRole};

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
pub struct CreatePageRequest {
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub icon: Option<WorkspaceIcon>,
    pub cover_image: Option<String>,
    pub template_id: Option<Uuid>,
    pub properties: Option<HashMap<String, PropertyValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePageRequest {
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
    use super::{BlockContent, BlockType};

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
