use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    Block, Page, PagePermissions, Workspace, WorkspaceIcon, WorkspaceSettings,
    blocks::{create_heading1, create_heading2, create_paragraph, create_checklist, create_callout, create_divider},
};

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

pub struct TemplateService {
    page_templates: Arc<RwLock<HashMap<Uuid, PageTemplate>>>,
    workspace_templates: Arc<RwLock<HashMap<Uuid, WorkspaceTemplate>>>,
}

impl TemplateService {
    pub fn new() -> Self {
        let service = Self {
            page_templates: Arc::new(RwLock::new(HashMap::new())),
            workspace_templates: Arc::new(RwLock::new(HashMap::new())),
        };

        tokio::spawn({
            let page_templates = service.page_templates.clone();
            let workspace_templates = service.workspace_templates.clone();
            async move {
                let system_user = Uuid::nil();
                let templates = create_system_page_templates(system_user);
                let mut pt = page_templates.write().await;
                for template in templates {
                    pt.insert(template.id, template);
                }

                let ws_templates = create_system_workspace_templates(system_user);
                let mut wt = workspace_templates.write().await;
                for template in ws_templates {
                    wt.insert(template.id, template);
                }
            }
        });

        service
    }

    pub async fn create_page_template(
        &self,
        name: &str,
        description: &str,
        blocks: Vec<Block>,
        category: TemplateCategory,
        created_by: Uuid,
        organization_id: Option<Uuid>,
        workspace_id: Option<Uuid>,
    ) -> PageTemplate {
        let now = Utc::now();
        let template = PageTemplate {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            icon: None,
            cover_image: None,
            blocks,
            properties: HashMap::new(),
            category,
            tags: Vec::new(),
            is_system: false,
            organization_id,
            workspace_id,
            created_by,
            created_at: now,
            updated_at: now,
            use_count: 0,
        };

        let mut templates = self.page_templates.write().await;
        templates.insert(template.id, template.clone());

        template
    }

    pub async fn get_page_template(&self, template_id: Uuid) -> Option<PageTemplate> {
        let templates = self.page_templates.read().await;
        templates.get(&template_id).cloned()
    }

    pub async fn list_page_templates(
        &self,
        organization_id: Option<Uuid>,
        workspace_id: Option<Uuid>,
        category: Option<TemplateCategory>,
    ) -> Vec<PageTemplate> {
        let templates = self.page_templates.read().await;
        templates
            .values()
            .filter(|t| {
                let org_match = t.is_system
                    || t.organization_id.is_none()
                    || t.organization_id == organization_id;
                let ws_match = t.workspace_id.is_none() || t.workspace_id == workspace_id;
                let cat_match = category.as_ref().map(|c| &t.category == c).unwrap_or(true);
                org_match && ws_match && cat_match
            })
            .cloned()
            .collect()
    }

    pub async fn update_page_template(
        &self,
        template_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        blocks: Option<Vec<Block>>,
        icon: Option<WorkspaceIcon>,
    ) -> Result<PageTemplate, TemplateError> {
        let mut templates = self.page_templates.write().await;
        let template = templates
            .get_mut(&template_id)
            .ok_or(TemplateError::TemplateNotFound)?;

        if template.is_system {
            return Err(TemplateError::CannotModifySystemTemplate);
        }

        if let Some(n) = name {
            template.name = n;
        }
        if let Some(d) = description {
            template.description = d;
        }
        if let Some(b) = blocks {
            template.blocks = b;
        }
        if icon.is_some() {
            template.icon = icon;
        }
        template.updated_at = Utc::now();

        Ok(template.clone())
    }

    pub async fn delete_page_template(&self, template_id: Uuid) -> Result<(), TemplateError> {
        let mut templates = self.page_templates.write().await;

        if let Some(template) = templates.get(&template_id) {
            if template.is_system {
                return Err(TemplateError::CannotModifySystemTemplate);
            }
        }

        templates
            .remove(&template_id)
            .ok_or(TemplateError::TemplateNotFound)?;

        Ok(())
    }

    pub async fn increment_template_usage(&self, template_id: Uuid) {
        let mut templates = self.page_templates.write().await;
        if let Some(template) = templates.get_mut(&template_id) {
            template.use_count += 1;
        }
    }

    pub async fn apply_page_template(
        &self,
        template_id: Uuid,
        workspace_id: Uuid,
        parent_id: Option<Uuid>,
        title: Option<String>,
        created_by: Uuid,
    ) -> Result<Page, TemplateError> {
        let template = self
            .get_page_template(template_id)
            .await
            .ok_or(TemplateError::TemplateNotFound)?;

        self.increment_template_usage(template_id).await;

        let now = Utc::now();
        let page = Page {
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
            template_id: Some(template_id),
            created_at: now,
            updated_at: now,
            created_by,
            last_edited_by: created_by,
        };

        Ok(page)
    }

    pub async fn create_workspace_template(
        &self,
        name: &str,
        description: &str,
        settings: WorkspaceSettings,
        category: TemplateCategory,
        created_by: Uuid,
    ) -> WorkspaceTemplate {
        let now = Utc::now();
        let template = WorkspaceTemplate {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            icon: None,
            cover_image: None,
            settings,
            page_templates: Vec::new(),
            default_structure: Vec::new(),
            category,
            is_system: false,
            created_by,
            created_at: now,
            updated_at: now,
        };

        let mut templates = self.workspace_templates.write().await;
        templates.insert(template.id, template.clone());

        template
    }

    pub async fn get_workspace_template(&self, template_id: Uuid) -> Option<WorkspaceTemplate> {
        let templates = self.workspace_templates.read().await;
        templates.get(&template_id).cloned()
    }

    pub async fn list_workspace_templates(
        &self,
        category: Option<TemplateCategory>,
    ) -> Vec<WorkspaceTemplate> {
        let templates = self.workspace_templates.read().await;
        templates
            .values()
            .filter(|t| category.as_ref().map(|c| &t.category == c).unwrap_or(true))
            .cloned()
            .collect()
    }

    pub async fn apply_workspace_template(
        &self,
        template_id: Uuid,
        organization_id: Uuid,
        name: &str,
        created_by: Uuid,
    ) -> Result<(Workspace, Vec<Page>), TemplateError> {
        let template = self
            .get_workspace_template(template_id)
            .await
            .ok_or(TemplateError::TemplateNotFound)?;

        let now = Utc::now();
        let workspace = Workspace {
            id: Uuid::new_v4(),
            organization_id,
            name: name.to_string(),
            description: Some(template.description.clone()),
            icon: template.icon.clone(),
            cover_image: template.cover_image.clone(),
            members: vec![super::WorkspaceMember {
                user_id: created_by,
                role: super::WorkspaceRole::Owner,
                joined_at: now,
                invited_by: None,
            }],
            settings: template.settings.clone(),
            root_pages: Vec::new(),
            created_at: now,
            updated_at: now,
            created_by,
        };

        let pages = self
            .create_pages_from_structure(
                &template.default_structure,
                workspace.id,
                None,
                created_by,
            )
            .await;

        Ok((workspace, pages))
    }

    async fn create_pages_from_structure(
        &self,
        structure: &[PageStructure],
        workspace_id: Uuid,
        parent_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Vec<Page> {
        let mut pages = Vec::new();

        for item in structure {
            let page = if let Some(template_id) = item.template_id {
                self.apply_page_template(
                    template_id,
                    workspace_id,
                    parent_id,
                    Some(item.title.clone()),
                    created_by,
                )
                .await
                .ok()
            } else {
                let now = Utc::now();
                Some(Page {
                    id: Uuid::new_v4(),
                    workspace_id,
                    parent_id,
                    title: item.title.clone(),
                    icon: item.icon.clone(),
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
                })
            };

            if let Some(page) = page {
                let page_id = page.id;
                pages.push(page);

                if !item.children.is_empty() {
                    let child_pages = Box::pin(self.create_pages_from_structure(
                        &item.children,
                        workspace_id,
                        Some(page_id),
                        created_by,
                    ))
                    .await;
                    pages.extend(child_pages);
                }
            }
        }

        pages
    }
}

impl Default for TemplateService {
    fn default() -> Self {
        Self::new()
    }
}

fn clone_blocks_with_new_ids(blocks: &[Block], created_by: Uuid) -> Vec<Block> {
    let now = Utc::now();
    blocks
        .iter()
        .map(|block| Block {
            id: Uuid::new_v4(),
            block_type: block.block_type.clone(),
            content: block.content.clone(),
            properties: block.properties.clone(),
            children: clone_blocks_with_new_ids(&block.children, created_by),
            created_at: now,
            updated_at: now,
            created_by,
        })
        .collect()
}

fn create_system_page_templates(system_user: Uuid) -> Vec<PageTemplate> {
    let now = Utc::now();

    vec![
        PageTemplate {
            id: Uuid::new_v4(),
            name: "Meeting Notes".to_string(),
            description: "Template for capturing meeting notes with agenda, attendees, and action items".to_string(),
            icon: Some(WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: "📝".to_string(),
            }),
            cover_image: None,
            blocks: vec![
                create_heading2("Attendees", system_user),
                create_paragraph("@mention attendees here", system_user),
                create_divider(system_user),
                create_heading2("Agenda", system_user),
                create_checklist(vec![
                    ("Topic 1", false),
                    ("Topic 2", false),
                    ("Topic 3", false),
                ], system_user),
                create_divider(system_user),
                create_heading2("Discussion Notes", system_user),
                create_paragraph("", system_user),
                create_divider(system_user),
                create_heading2("Action Items", system_user),
                create_checklist(vec![
                    ("Action item 1", false),
                    ("Action item 2", false),
                ], system_user),
            ],
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
            name: "Project Overview".to_string(),
            description: "Template for project documentation with goals, timeline, and team".to_string(),
            icon: Some(WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: "🚀".to_string(),
            }),
            cover_image: None,
            blocks: vec![
                create_callout("🎯", "Project Goal: Define your project goal here", "#E3F2FD", system_user),
                create_divider(system_user),
                create_heading2("Overview", system_user),
                create_paragraph("Describe the project and its objectives.", system_user),
                create_divider(system_user),
                create_heading2("Team", system_user),
                create_paragraph("@mention team members and their roles", system_user),
                create_divider(system_user),
                create_heading2("Timeline", system_user),
                create_paragraph("Key milestones and deadlines", system_user),
                create_divider(system_user),
                create_heading2("Resources", system_user),
                create_paragraph("Links to relevant documents and tools", system_user),
            ],
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
            name: "Technical Specification".to_string(),
            description: "Template for technical documentation and specifications".to_string(),
            icon: Some(WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: "📋".to_string(),
            }),
            cover_image: None,
            blocks: vec![
                create_heading1("Technical Specification", system_user),
                create_callout("📌", "Status: Draft", "#FFF3E0", system_user),
                create_divider(system_user),
                create_heading2("Summary", system_user),
                create_paragraph("Brief overview of the technical solution.", system_user),
                create_divider(system_user),
                create_heading2("Requirements", system_user),
                create_paragraph("List functional and non-functional requirements.", system_user),
                create_divider(system_user),
                create_heading2("Architecture", system_user),
                create_paragraph("Describe the system architecture.", system_user),
                create_divider(system_user),
                create_heading2("Implementation Details", system_user),
                create_paragraph("Technical implementation details.", system_user),
                create_divider(system_user),
                create_heading2("Testing Strategy", system_user),
                create_paragraph("How the solution will be tested.", system_user),
                create_divider(system_user),
                create_heading2("Open Questions", system_user),
                create_checklist(vec![
                    ("Question 1?", false),
                    ("Question 2?", false),
                ], system_user),
            ],
            properties: HashMap::new(),
            category: TemplateCategory::Engineering,
            tags: vec!["technical".to_string(), "spec".to_string(), "engineering".to_string()],
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
            name: "Weekly Status Update".to_string(),
            description: "Template for weekly team status updates".to_string(),
            icon: Some(WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: "📊".to_string(),
            }),
            cover_image: None,
            blocks: vec![
                create_heading2("Completed This Week", system_user),
                create_checklist(vec![
                    ("Task 1", true),
                    ("Task 2", true),
                ], system_user),
                create_divider(system_user),
                create_heading2("In Progress", system_user),
                create_checklist(vec![
                    ("Task 3", false),
                    ("Task 4", false),
                ], system_user),
                create_divider(system_user),
                create_heading2("Planned for Next Week", system_user),
                create_checklist(vec![
                    ("Task 5", false),
                    ("Task 6", false),
                ], system_user),
                create_divider(system_user),
                create_heading2("Blockers", system_user),
                create_callout("⚠️", "List any blockers or issues", "#FFEBEE", system_user),
            ],
            properties: HashMap::new(),
            category: TemplateCategory::Team,
            tags: vec!["status".to_string(), "weekly".to_string(), "update".to_string()],
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
            name: "Blank Page".to_string(),
            description: "Start with a blank page".to_string(),
            icon: Some(WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: "📄".to_string(),
            }),
            cover_image: None,
            blocks: vec![
                create_paragraph("", system_user),
            ],
            properties: HashMap::new(),
            category: TemplateCategory::Personal,
            tags: vec!["blank".to_string(), "empty".to_string()],
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

fn create_system_workspace_templates(system_user: Uuid) -> Vec<WorkspaceTemplate> {
    let now = Utc::now();

    vec![
        WorkspaceTemplate {
            id: Uuid::new_v4(),
            name: "Team Workspace".to_string(),
            description: "A workspace for team collaboration with common sections".to_string(),
            icon: Some(WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: "👥".to_string(),
            }),
            cover_image: None,
            settings: WorkspaceSettings::default(),
            page_templates: Vec::new(),
            default_structure: vec![
                PageStructure {
                    title: "Welcome".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "👋".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Meetings".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "📅".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Projects".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "📁".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Documentation".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "📚".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
            ],
            category: TemplateCategory::Team,
            is_system: true,
            created_by: system_user,
            created_at: now,
            updated_at: now,
        },
        WorkspaceTemplate {
            id: Uuid::new_v4(),
            name: "Project Workspace".to_string(),
            description: "A workspace organized for project management".to_string(),
            icon: Some(WorkspaceIcon {
                icon_type: super::IconType::Emoji,
                value: "🎯".to_string(),
            }),
            cover_image: None,
            settings: WorkspaceSettings::default(),
            page_templates: Vec::new(),
            default_structure: vec![
                PageStructure {
                    title: "Project Overview".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "🚀".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Requirements".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "📋".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Design".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "🎨".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Development".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "💻".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Testing".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "🧪".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
                PageStructure {
                    title: "Launch".to_string(),
                    icon: Some(WorkspaceIcon {
                        icon_type: super::IconType::Emoji,
                        value: "🎉".to_string(),
                    }),
                    template_id: None,
                    children: Vec::new(),
                },
            ],
            category: TemplateCategory::Project,
            is_system: true,
            created_by: system_user,
            created_at: now,
            updated_at: now,
        },
    ]
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
