use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuItemType {
    App,
    Section,
    Divider,
    Link,
    SubMenu,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppType {
    Docs,
    Paper,
    Forms,
    Sites,
    Drive,
    Sheet,
    Slides,
    Calendar,
    Meet,
    Tasks,
    Mail,
    Chat,
    Analytics,
    Monitoring,
    Settings,
    Admin,
}

impl AppType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Paper => "paper",
            Self::Forms => "forms",
            Self::Sites => "sites",
            Self::Drive => "drive",
            Self::Sheet => "sheet",
            Self::Slides => "slides",
            Self::Calendar => "calendar",
            Self::Meet => "meet",
            Self::Tasks => "tasks",
            Self::Mail => "mail",
            Self::Chat => "chat",
            Self::Analytics => "analytics",
            Self::Monitoring => "monitoring",
            Self::Settings => "settings",
            Self::Admin => "admin",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Docs => "Docs",
            Self::Paper => "Paper",
            Self::Forms => "Forms",
            Self::Sites => "Sites",
            Self::Drive => "Drive",
            Self::Sheet => "Sheet",
            Self::Slides => "Slides",
            Self::Calendar => "Calendar",
            Self::Meet => "Meet",
            Self::Tasks => "Tasks",
            Self::Mail => "Mail",
            Self::Chat => "Chat",
            Self::Analytics => "Analytics",
            Self::Monitoring => "Monitoring",
            Self::Settings => "Settings",
            Self::Admin => "Admin",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Docs => "📝",
            Self::Paper => "📓",
            Self::Forms => "📋",
            Self::Sites => "🌐",
            Self::Drive => "💾",
            Self::Sheet => "📊",
            Self::Slides => "📽️",
            Self::Calendar => "📅",
            Self::Meet => "📹",
            Self::Tasks => "✅",
            Self::Mail => "✉️",
            Self::Chat => "💬",
            Self::Analytics => "📈",
            Self::Monitoring => "🔍",
            Self::Settings => "⚙️",
            Self::Admin => "🛡️",
        }
    }

    pub fn route(&self) -> &'static str {
        match self {
            Self::Docs => "/docs",
            Self::Paper => "/paper",
            Self::Forms => "/forms",
            Self::Sites => "/sites",
            Self::Drive => "/drive",
            Self::Sheet => "/sheet",
            Self::Slides => "/slides",
            Self::Calendar => "/calendar",
            Self::Meet => "/meet",
            Self::Tasks => "/tasks",
            Self::Mail => "/mail",
            Self::Chat => "/chat",
            Self::Analytics => "/analytics",
            Self::Monitoring => "/monitoring",
            Self::Settings => "/settings",
            Self::Admin => "/admin",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Docs => "Full-featured document editor with rich text formatting",
            Self::Paper => "Quick notes and lightweight text editor",
            Self::Forms => "Create surveys, quizzes, and data collection forms",
            Self::Sites => "Build and publish websites",
            Self::Drive => "File storage and management",
            Self::Sheet => "Spreadsheets and data analysis",
            Self::Slides => "Presentations and slideshows",
            Self::Calendar => "Schedule and event management",
            Self::Meet => "Video conferencing and meetings",
            Self::Tasks => "Task and project management",
            Self::Mail => "Email client",
            Self::Chat => "Instant messaging and conversations",
            Self::Analytics => "Data analytics and reporting",
            Self::Monitoring => "System monitoring and health",
            Self::Settings => "Application settings",
            Self::Admin => "Administration panel",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub item_type: MenuItemType,
    pub label: String,
    pub icon: Option<String>,
    pub route: Option<String>,
    pub app_type: Option<AppType>,
    pub children: Vec<MenuItem>,
    pub badge: Option<String>,
    pub tooltip: Option<String>,
    pub visible: bool,
    pub enabled: bool,
    pub order: i32,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MenuItem {
    pub fn app(app_type: AppType, order: i32) -> Self {
        Self {
            id: app_type.as_str().to_string(),
            item_type: MenuItemType::App,
            label: app_type.display_name().to_string(),
            icon: Some(app_type.icon().to_string()),
            route: Some(app_type.route().to_string()),
            app_type: Some(app_type),
            children: Vec::new(),
            badge: None,
            tooltip: None,
            visible: true,
            enabled: true,
            order,
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn section(id: &str, label: &str, order: i32) -> Self {
        Self {
            id: id.to_string(),
            item_type: MenuItemType::Section,
            label: label.to_string(),
            icon: None,
            route: None,
            app_type: None,
            children: Vec::new(),
            badge: None,
            tooltip: None,
            visible: true,
            enabled: true,
            order,
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn divider(order: i32) -> Self {
        Self {
            id: format!("divider_{order}"),
            item_type: MenuItemType::Divider,
            label: String::new(),
            icon: None,
            route: None,
            app_type: None,
            children: Vec::new(),
            badge: None,
            tooltip: None,
            visible: true,
            enabled: true,
            order,
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<MenuItem>) -> Self {
        self.children = children;
        self
    }

    pub fn with_badge(mut self, badge: &str) -> Self {
        self.badge = Some(badge.to_string());
        self
    }

    pub fn with_tooltip(mut self, tooltip: &str) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<&str>) -> Self {
        self.permissions = permissions.into_iter().map(String::from).collect();
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuSection {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub items: Vec<MenuItem>,
    pub collapsed: bool,
    pub order: i32,
}

impl MenuSection {
    pub fn new(id: &str, label: &str, order: i32) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            items: Vec::new(),
            collapsed: false,
            order,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self
    }

    pub fn collapsed(mut self) -> Self {
        self.collapsed = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuConfig {
    pub sections: Vec<MenuSection>,
    pub footer_items: Vec<MenuItem>,
    pub user_menu_items: Vec<MenuItem>,
    pub quick_actions: Vec<MenuItem>,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuConfig {
    pub fn new() -> Self {
        let productivity_section = MenuSection::new("productivity", "Productivity", 1)
            .with_icon("⚡")
            .with_items(vec![
                MenuItem::app(AppType::Chat, 1),
                MenuItem::app(AppType::Mail, 2),
                MenuItem::app(AppType::Calendar, 3),
                MenuItem::app(AppType::Tasks, 4),
                MenuItem::app(AppType::Meet, 5),
            ]);

        let library_section = MenuSection::new("library", "Library", 2)
            .with_icon("📚")
            .with_items(vec![
                MenuItem::app(AppType::Drive, 1),
                MenuItem::app(AppType::Docs, 2)
                    .with_tooltip("Full-featured document editor with rich text formatting, collaboration, and export options"),
                MenuItem::app(AppType::Paper, 3)
                    .with_tooltip("Quick notes and lightweight text editor for fast capture"),
                MenuItem::app(AppType::Sheet, 4),
                MenuItem::app(AppType::Slides, 5),
                MenuItem::app(AppType::Forms, 6)
                    .with_tooltip("Create surveys, quizzes, and data collection forms"),
                MenuItem::app(AppType::Sites, 7)
                    .with_tooltip("Build and publish websites"),
            ]);

        let insights_section = MenuSection::new("insights", "Insights", 3)
            .with_icon("📊")
            .with_items(vec![
                MenuItem::app(AppType::Analytics, 1),
                MenuItem::app(AppType::Monitoring, 2),
            ]);

        let admin_section = MenuSection::new("admin", "Administration", 4)
            .with_icon("🛡️")
            .with_items(vec![
                MenuItem::app(AppType::Settings, 1),
                MenuItem::app(AppType::Admin, 2)
                    .with_permissions(vec!["admin", "owner"]),
            ]);

        let footer_items = vec![
            MenuItem {
                id: "help".to_string(),
                item_type: MenuItemType::Link,
                label: "Help & Support".to_string(),
                icon: Some("❓".to_string()),
                route: Some("/help".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 1,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem {
                id: "feedback".to_string(),
                item_type: MenuItemType::Link,
                label: "Send Feedback".to_string(),
                icon: Some("💭".to_string()),
                route: Some("/feedback".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 2,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
        ];

        let user_menu_items = vec![
            MenuItem {
                id: "profile".to_string(),
                item_type: MenuItemType::Link,
                label: "Profile".to_string(),
                icon: Some("👤".to_string()),
                route: Some("/profile".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 1,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem {
                id: "preferences".to_string(),
                item_type: MenuItemType::Link,
                label: "Preferences".to_string(),
                icon: Some("⚙️".to_string()),
                route: Some("/preferences".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 2,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem::divider(3),
            MenuItem {
                id: "logout".to_string(),
                item_type: MenuItemType::Link,
                label: "Sign Out".to_string(),
                icon: Some("🚪".to_string()),
                route: Some("/logout".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 4,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
        ];

        let quick_actions = vec![
            MenuItem {
                id: "new_doc".to_string(),
                item_type: MenuItemType::Link,
                label: "New Document".to_string(),
                icon: Some("📝".to_string()),
                route: Some("/docs/new".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: Some("Create a new full document".to_string()),
                visible: true,
                enabled: true,
                order: 1,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem {
                id: "new_note".to_string(),
                item_type: MenuItemType::Link,
                label: "New Note".to_string(),
                icon: Some("📓".to_string()),
                route: Some("/paper/new".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: Some("Create a quick note".to_string()),
                visible: true,
                enabled: true,
                order: 2,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem {
                id: "new_sheet".to_string(),
                item_type: MenuItemType::Link,
                label: "New Spreadsheet".to_string(),
                icon: Some("📊".to_string()),
                route: Some("/sheet/new".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 3,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem {
                id: "new_form".to_string(),
                item_type: MenuItemType::Link,
                label: "New Form".to_string(),
                icon: Some("📋".to_string()),
                route: Some("/forms/new".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 4,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem {
                id: "new_site".to_string(),
                item_type: MenuItemType::Link,
                label: "New Site".to_string(),
                icon: Some("🌐".to_string()),
                route: Some("/sites/new".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 5,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
            MenuItem {
                id: "schedule_meeting".to_string(),
                item_type: MenuItemType::Link,
                label: "Schedule Meeting".to_string(),
                icon: Some("📹".to_string()),
                route: Some("/meet/schedule".to_string()),
                app_type: None,
                children: Vec::new(),
                badge: None,
                tooltip: None,
                visible: true,
                enabled: true,
                order: 6,
                permissions: Vec::new(),
                metadata: HashMap::new(),
            },
        ];

        Self {
            sections: vec![
                productivity_section,
                library_section,
                insights_section,
                admin_section,
            ],
            footer_items,
            user_menu_items,
            quick_actions,
        }
    }

    pub fn get_all_items(&self) -> Vec<&MenuItem> {
        let mut items = Vec::new();

        for section in &self.sections {
            for item in &section.items {
                items.push(item);
                for child in &item.children {
                    items.push(child);
                }
            }
        }

        for item in &self.footer_items {
            items.push(item);
        }

        items
    }

    pub fn get_item_by_id(&self, id: &str) -> Option<&MenuItem> {
        self.get_all_items().into_iter().find(|item| item.id == id)
    }

    pub fn get_section_by_id(&self, id: &str) -> Option<&MenuSection> {
        self.sections.iter().find(|section| section.id == id)
    }

    pub fn filter_by_permissions(&self, user_permissions: &[String]) -> Self {
        let mut filtered = self.clone();

        for section in &mut filtered.sections {
            section.items.retain(|item| {
                if item.permissions.is_empty() {
                    return true;
                }
                item.permissions.iter().any(|p| user_permissions.contains(p))
            });
        }

        filtered.sections.retain(|section| !section.items.is_empty());

        filtered
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

pub fn get_docs_paper_clarification() -> HashMap<&'static str, &'static str> {
    let mut clarification = HashMap::new();

    clarification.insert(
        "docs_purpose",
        "Docs is a full-featured document editor similar to Google Docs or Microsoft Word. \
         It supports rich text formatting, images, tables, collaboration, comments, \
         version history, and export to multiple formats (PDF, DOCX, HTML)."
    );

    clarification.insert(
        "paper_purpose",
        "Paper is a lightweight notes application similar to Apple Notes or Google Keep. \
         It's designed for quick capture, simple formatting, and fast access to notes. \
         Perfect for meeting notes, quick thoughts, and simple text."
    );

    clarification.insert(
        "when_to_use_docs",
        "Use Docs when you need: rich formatting, collaboration features, \
         document templates, export capabilities, version history, or formal documents."
    );

    clarification.insert(
        "when_to_use_paper",
        "Use Paper when you need: quick note-taking, simple lists, \
         fast capture without formatting overhead, or lightweight text storage."
    );

    clarification.insert(
        "key_differences",
        "Docs = Full document editor (Word-like), Paper = Quick notes (Notes-like). \
         Different tools for different purposes - both available in the Library section."
    );

    clarification
}

pub fn get_library_section_info() -> HashMap<&'static str, &'static str> {
    let mut info = HashMap::new();

    info.insert(
        "name",
        "Library"
    );

    info.insert(
        "description",
        "The Library section contains all content creation and storage apps. \
         This includes Drive (file storage), Docs (full documents), Paper (quick notes), \
         Sheet (spreadsheets), Slides (presentations), Forms (surveys/forms), \
         and Sites (website builder)."
    );

    info.insert(
        "previous_name",
        "Sources"
    );

    info.insert(
        "rename_reason",
        "Renamed from 'Sources' to 'Library' for better clarity. \
         Library better represents a collection of content and documents, \
         while 'Sources' was ambiguous and could be confused with data sources or integrations."
    );

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_menu_config() {
        let config = MenuConfig::default();
        assert!(!config.sections.is_empty());
        assert!(!config.footer_items.is_empty());
        assert!(!config.user_menu_items.is_empty());
    }

    #[test]
    fn test_library_section_exists() {
        let config = MenuConfig::default();
        let library = config.get_section_by_id("library");
        assert!(library.is_some());
        assert_eq!(library.map(|s| s.label.as_str()), Some("Library"));
    }

    #[test]
    fn test_docs_and_paper_in_library() {
        let config = MenuConfig::default();
        let library = config.get_section_by_id("library").unwrap();

        let has_docs = library.items.iter().any(|i| i.id == "docs");
        let has_paper = library.items.iter().any(|i| i.id == "paper");
        let has_forms = library.items.iter().any(|i| i.id == "forms");
        let has_sites = library.items.iter().any(|i| i.id == "sites");

        assert!(has_docs, "Library should contain Docs");
        assert!(has_paper, "Library should contain Paper");
        assert!(has_forms, "Library should contain Forms");
        assert!(has_sites, "Library should contain Sites");
    }

    #[test]
    fn test_app_type_properties() {
        assert_eq!(AppType::Docs.as_str(), "docs");
        assert_eq!(AppType::Paper.as_str(), "paper");
        assert_eq!(AppType::Docs.route(), "/docs");
        assert_eq!(AppType::Paper.route(), "/paper");
    }

    #[test]
    fn test_permission_filtering() {
        let config = MenuConfig::default();
        let user_permissions = vec!["member".to_string()];
        let filtered = config.filter_by_permissions(&user_permissions);

        for section in &filtered.sections {
            for item in &section.items {
                if !item.permissions.is_empty() {
                    assert!(item.permissions.iter().any(|p| user_permissions.contains(p)));
                }
            }
        }
    }

    #[test]
    fn test_json_serialization() {
        let config = MenuConfig::default();
        let json = config.to_json().unwrap();
        let deserialized = MenuConfig::from_json(&json).unwrap();
        assert_eq!(config.sections.len(), deserialized.sections.len());
    }

    #[test]
    fn test_docs_paper_clarification() {
        let clarification = get_docs_paper_clarification();
        assert!(clarification.contains_key("docs_purpose"));
        assert!(clarification.contains_key("paper_purpose"));
        assert!(clarification.contains_key("key_differences"));
    }

    #[test]
    fn test_library_section_info() {
        let info = get_library_section_info();
        assert_eq!(info.get("name"), Some(&"Library"));
        assert_eq!(info.get("previous_name"), Some(&"Sources"));
    }
}
