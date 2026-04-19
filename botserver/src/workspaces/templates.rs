use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{Block, Page, PagePermissions, WorkspaceIcon, WorkspaceSettings};

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
