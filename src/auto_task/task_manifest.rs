use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskManifest {
    pub id: String,
    pub app_name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: ManifestStatus,
    pub current_status: CurrentStatus,
    pub sections: Vec<ManifestSection>,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub runtime_seconds: u64,
    pub estimated_seconds: u64,
    pub terminal_output: Vec<TerminalLine>,
    pub processing_stats: ProcessingStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurrentStatus {
    pub title: String,
    pub current_action: Option<String>,
    pub decision_point: Option<DecisionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    pub step_current: u32,
    pub step_total: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ManifestStatus {
    #[default]
    Planning,
    Ready,
    Running,
    Paused,
    Completed,
    Failed,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSection {
    pub id: String,
    pub name: String,
    pub section_type: SectionType,
    pub status: SectionStatus,
    pub current_step: u32,
    pub total_steps: u32,
    pub global_step_start: u32,
    pub duration_seconds: Option<u64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub items: Vec<ManifestItem>,
    pub item_groups: Vec<ItemGroup>,
    pub children: Vec<ManifestSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SectionType {
    DatabaseModels,
    SchemaDesign,
    Tables,
    Files,
    Pages,
    Tools,
    Schedulers,
    Monitors,
    Validation,
    Deployment,
}

impl std::fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseModels => write!(f, "Database & Models"),
            Self::SchemaDesign => write!(f, "Database Schema Design"),
            Self::Tables => write!(f, "Tables"),
            Self::Files => write!(f, "Files"),
            Self::Pages => write!(f, "Pages"),
            Self::Tools => write!(f, "Tools"),
            Self::Schedulers => write!(f, "Schedulers"),
            Self::Monitors => write!(f, "Monitors"),
            Self::Validation => write!(f, "Validation"),
            Self::Deployment => write!(f, "Deployment"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum SectionStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub status: ItemStatus,
    pub details: Option<String>,
    pub duration_seconds: Option<u64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Grouped items displayed as a single row (e.g., "email, password_hash, email_verified")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemGroup {
    pub id: String,
    pub items: Vec<String>,
    pub status: ItemStatus,
    pub duration_seconds: Option<u64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ItemGroup {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            items,
            status: ItemStatus::Pending,
            duration_seconds: None,
            started_at: None,
            completed_at: None,
        }
    }

    pub fn display_name(&self) -> String {
        self.items.join(", ")
    }

    pub fn start(&mut self) {
        self.status = ItemStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self) {
        self.status = ItemStatus::Completed;
        self.completed_at = Some(Utc::now());
        if let Some(started) = self.started_at {
            self.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    Table,
    Field,
    Index,
    File,
    Page,
    Tool,
    Scheduler,
    Monitor,
    Config,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ItemStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLine {
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub line_type: TerminalLineType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerminalLineType {
    Info,
    Progress,
    Success,
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingStats {
    pub data_points_processed: u64,
    pub processing_speed: f64,
    pub sources_per_min: f64,
    pub estimated_remaining_seconds: u64,
}

impl TaskManifest {
    pub fn new(app_name: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            app_name: app_name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: ManifestStatus::Planning,
            current_status: CurrentStatus {
                title: description.to_string(),
                current_action: None,
                decision_point: None,
            },
            sections: Vec::new(),
            total_steps: 0,
            completed_steps: 0,
            runtime_seconds: 0,
            estimated_seconds: 0,
            terminal_output: Vec::new(),
            processing_stats: ProcessingStats::default(),
        }
    }

    pub fn set_current_action(&mut self, action: &str) {
        self.current_status.current_action = Some(action.to_string());
        self.updated_at = Utc::now();
    }

    pub fn set_decision_point(&mut self, current: u32, total: u32, message: &str) {
        self.current_status.decision_point = Some(DecisionPoint {
            step_current: current,
            step_total: total,
            message: message.to_string(),
        });
        self.updated_at = Utc::now();
    }

    pub fn add_section(&mut self, mut section: ManifestSection) {
        // Set global step start for this section
        section.global_step_start = self.total_steps;

        // Update global step starts for children
        let mut child_offset = self.total_steps;
        for child in &mut section.children {
            child.global_step_start = child_offset;
            child_offset += child.total_steps;
        }

        self.total_steps += section.total_steps;
        for child in &section.children {
            self.total_steps += child.total_steps;
        }
        self.sections.push(section);
        self.updated_at = Utc::now();
    }

    pub fn start(&mut self) {
        self.status = ManifestStatus::Running;
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self) {
        self.status = ManifestStatus::Completed;
        self.completed_steps = self.total_steps;
        self.updated_at = Utc::now();
    }

    /// Recalculate global_step_start for all sections after modifications
    pub fn recalculate_global_steps(&mut self) {
        let mut offset = 0u32;
        for section in &mut self.sections {
            section.global_step_start = offset;

            // Update children's global step starts
            let mut child_offset = offset;
            for child in &mut section.children {
                child.global_step_start = child_offset;
                child_offset += child.total_steps;
            }

            // Add this section's steps (including children)
            offset += section.total_steps;
            for child in &section.children {
                offset += child.total_steps;
            }
        }

        // Recalculate total
        self.total_steps = offset;
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = ManifestStatus::Failed;
        self.updated_at = Utc::now();
    }

    pub fn add_terminal_line(&mut self, content: &str, line_type: TerminalLineType) {
        self.terminal_output.push(TerminalLine {
            timestamp: Utc::now(),
            content: content.to_string(),
            line_type,
        });
        self.updated_at = Utc::now();
    }

    pub fn update_section_status(&mut self, section_id: &str, status: SectionStatus) {
        for section in &mut self.sections {
            if section.id == section_id {
                section.status = status.clone();
                if status == SectionStatus::Completed {
                    section.completed_at = Some(Utc::now());
                    self.completed_steps += section.total_steps;
                }
                break;
            }
            for child in &mut section.children {
                if child.id == section_id {
                    child.status = status.clone();
                    if status == SectionStatus::Completed {
                        child.completed_at = Some(Utc::now());
                        self.completed_steps += child.total_steps;
                    }
                    break;
                }
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn update_item_status(&mut self, section_id: &str, item_id: &str, status: ItemStatus) {
        for section in &mut self.sections {
            if section.id == section_id {
                for item in &mut section.items {
                    if item.id == item_id {
                        item.status = status;
                        if status == ItemStatus::Completed {
                            item.completed_at = Some(Utc::now());
                        }
                        return;
                    }
                }
            }
            for child in &mut section.children {
                if child.id == section_id {
                    for item in &mut child.items {
                        if item.id == item_id {
                            item.status = status;
                            if status == ItemStatus::Completed {
                                item.completed_at = Some(Utc::now());
                            }
                            return;
                        }
                    }
                }
            }
        }
        self.updated_at = Utc::now();
    }

    pub fn update_processing_stats(&mut self, stats: ProcessingStats) {
        self.processing_stats = stats;
        self.updated_at = Utc::now();
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        (self.completed_steps as f64 / self.total_steps as f64) * 100.0
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# TASK.md - {}\n\n", self.app_name));
        md.push_str(&format!("**Description:** {}\n\n", self.description));
        md.push_str(&format!("**Status:** {:?}\n", self.status));
        md.push_str(&format!(
            "**Progress:** {}/{} steps ({}%)\n\n",
            self.completed_steps,
            self.total_steps,
            self.progress_percentage() as u32
        ));

        md.push_str("## Artifacts\n\n");

        for section in &self.sections {
            md.push_str(&format!(
                "### {} - {:?}\n",
                section.name, section.status
            ));
            md.push_str(&format!(
                "- Steps: {}/{}\n",
                section.current_step, section.total_steps
            ));

            if !section.items.is_empty() {
                md.push_str("- Items:\n");
                for item in &section.items {
                    md.push_str(&format!(
                        "  - {} ({:?}): {:?}\n",
                        item.name, item.item_type, item.status
                    ));
                }
            }

            for child in &section.children {
                md.push_str(&format!(
                    "  #### {} - {:?}\n",
                    child.name, child.status
                ));
                md.push_str(&format!(
                    "  - Steps: {}/{}\n",
                    child.current_step, child.total_steps
                ));

                if !child.items.is_empty() {
                    md.push_str("  - Items:\n");
                    for item in &child.items {
                        md.push_str(&format!(
                            "    - {} ({:?}): {:?}\n",
                            item.name, item.item_type, item.status
                        ));
                    }
                }
            }

            md.push('\n');
        }

        md
    }

    pub fn to_web_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "app_name": self.app_name,
            "description": self.description,
            "status": {
                "title": self.current_status.title,
                "runtime_display": format_duration(self.runtime_seconds),
                "estimated_display": format_duration(self.estimated_seconds),
                "current_action": self.current_status.current_action,
                "decision_point": self.current_status.decision_point.as_ref().map(|dp| serde_json::json!({
                    "step_current": dp.step_current,
                    "step_total": dp.step_total,
                    "message": dp.message
                }))
            },
            "progress": {
                "current": self.completed_steps,
                "total": self.total_steps,
                "percentage": self.progress_percentage()
            },
            "sections": self.sections.iter().map(section_to_web_json).collect::<Vec<_>>(),
            "terminal": {
                "lines": self.terminal_output.iter().map(|l| serde_json::json!({
                    "content": l.content,
                    "type": format!("{:?}", l.line_type).to_lowercase(),
                    "timestamp": l.timestamp.to_rfc3339()
                })).collect::<Vec<_>>(),
                "stats": {
                    "processed": self.processing_stats.data_points_processed,
                    "speed": format!("{:.1} sources/min", self.processing_stats.sources_per_min),
                    "estimated_completion": format_duration(self.processing_stats.estimated_remaining_seconds)
                }
            }
        })
    }

    pub fn to_task_md(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# TASK.md - {}\n\n", self.app_name));
        md.push_str("## STATUS\n");
        md.push_str(&format!("- {}\n", self.current_status.title));
        if let Some(ref action) = self.current_status.current_action {
            md.push_str(&format!("  - [>] {}\n", action));
        }
        if let Some(ref dp) = self.current_status.decision_point {
            md.push_str(&format!("  - [ ] Decision Point (Step {}/{}) - {}\n", dp.step_current, dp.step_total, dp.message));
        }
        md.push_str("\n## PROGRESS LOG\n");
        for section in &self.sections {
            let checkbox = match section.status {
                SectionStatus::Completed => "[x]",
                SectionStatus::Running => "[>]",
                _ => "[ ]",
            };
            let global_step = section.global_step_start + section.current_step;
            md.push_str(&format!("- {} {} (Step {}/{})\n", checkbox, section.name, global_step, self.total_steps));
            for child in &section.children {
                let child_checkbox = match child.status {
                    SectionStatus::Completed => "[x]",
                    SectionStatus::Running => "[>]",
                    _ => "[ ]",
                };
                md.push_str(&format!("  - {} {} (Step {}/{})\n", child_checkbox, child.name, child.current_step, child.total_steps));

                // Render item groups first
                for group in &child.item_groups {
                    let group_checkbox = match group.status {
                        ItemStatus::Completed => "[x]",
                        ItemStatus::Running => "[>]",
                        _ => "[ ]",
                    };
                    let duration = group.duration_seconds.map(|s| format!(" - Duration: {} min", s / 60)).unwrap_or_default();
                    md.push_str(&format!("    - {} {}{}\n", group_checkbox, group.display_name(), duration));
                }

                // Then individual items
                for item in &child.items {
                    let item_checkbox = match item.status {
                        ItemStatus::Completed => "[x]",
                        ItemStatus::Running => "[>]",
                        _ => "[ ]",
                    };
                    let duration = item.duration_seconds.map(|s| format!(" - Duration: {}s", s)).unwrap_or_default();
                    md.push_str(&format!("    - {} {}{}\n", item_checkbox, item.name, duration));
                }
            }

            // Render section-level item groups
            for group in &section.item_groups {
                let group_checkbox = match group.status {
                    ItemStatus::Completed => "[x]",
                    ItemStatus::Running => "[>]",
                    _ => "[ ]",
                };
                let duration = group.duration_seconds.map(|s| format!(" - Duration: {} min", s / 60)).unwrap_or_default();
                md.push_str(&format!("  - {} {}{}\n", group_checkbox, group.display_name(), duration));
            }

            for item in &section.items {
                let item_checkbox = match item.status {
                    ItemStatus::Completed => "[x]",
                    ItemStatus::Running => "[>]",
                    _ => "[ ]",
                };
                md.push_str(&format!("  - {} {}\n", item_checkbox, item.name));
            }
        }
        md
    }

}

impl ManifestSection {
    pub fn new(name: &str, section_type: SectionType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            section_type,
            status: SectionStatus::Pending,
            current_step: 0,
            total_steps: 0,
            global_step_start: 0,
            duration_seconds: None,
            started_at: None,
            completed_at: None,
            items: Vec::new(),
            item_groups: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_steps(mut self, total: u32) -> Self {
        self.total_steps = total;
        self
    }

    pub fn add_item(&mut self, item: ManifestItem) {
        self.total_steps += 1;
        self.items.push(item);
    }

    pub fn add_item_group(&mut self, group: ItemGroup) {
        self.total_steps += 1;
        self.item_groups.push(group);
    }

    pub fn add_child(&mut self, child: ManifestSection) {
        self.total_steps += child.total_steps;
        self.children.push(child);
    }

    pub fn start(&mut self) {
        self.status = SectionStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self) {
        self.status = SectionStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.current_step = self.total_steps;
        if let Some(started) = self.started_at {
            self.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
        }
    }

    pub fn increment_step(&mut self) {
        self.current_step += 1;
    }
}

impl ManifestItem {
    pub fn new(name: &str, item_type: ItemType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            item_type,
            status: ItemStatus::Pending,
            details: None,
            duration_seconds: None,
            started_at: None,
            completed_at: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    pub fn start(&mut self) {
        self.status = ItemStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self) {
        self.status = ItemStatus::Completed;
        self.completed_at = Some(Utc::now());
        if let Some(started) = self.started_at {
            self.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
        }
    }
}

fn section_to_web_json(section: &ManifestSection) -> serde_json::Value {
    let checkbox = match section.status {
        SectionStatus::Completed => "[x]",
        SectionStatus::Running => "[>]",
        _ => "[ ]",
    };

    // Calculate global step display (e.g., "Step 24/60")
    let global_current = section.global_step_start + section.current_step;

    serde_json::json!({
        "id": section.id,
        "name": section.name,
        "checkbox": checkbox,
        "type": format!("{:?}", section.section_type),
        "status": format!("{:?}", section.status),
        "progress": {
            "current": section.current_step,
            "total": section.total_steps,
            "display": format!("Step {}/{}", section.current_step, section.total_steps),
            "global_current": global_current,
            "global_start": section.global_step_start
        },
        "duration": section.duration_seconds.map(format_duration),
        "duration_seconds": section.duration_seconds,
        "items": section.items.iter().map(|i| {
            let item_checkbox = match i.status {
                ItemStatus::Completed => "[x]",
                ItemStatus::Running => "[>]",
                _ => "[ ]",
            };
            serde_json::json!({
                "id": i.id,
                "name": i.name,
                "checkbox": item_checkbox,
                "type": format!("{:?}", i.item_type),
                "status": format!("{:?}", i.status),
                "details": i.details,
                "duration": i.duration_seconds.map(format_duration),
                "duration_seconds": i.duration_seconds
            })
        }).collect::<Vec<_>>(),
        "item_groups": section.item_groups.iter().map(|g| {
            let group_checkbox = match g.status {
                ItemStatus::Completed => "[x]",
                ItemStatus::Running => "[>]",
                _ => "[ ]",
            };
            serde_json::json!({
                "id": g.id,
                "name": g.display_name(),
                "items": g.items,
                "checkbox": group_checkbox,
                "status": format!("{:?}", g.status),
                "duration": g.duration_seconds.map(format_duration),
                "duration_seconds": g.duration_seconds
            })
        }).collect::<Vec<_>>(),
        "children": section.children.iter().map(section_to_web_json).collect::<Vec<_>>()
    })
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{} sec", seconds)
    } else if seconds < 3600 {
        format!("{} min", seconds / 60)
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{} hr {} min", hours, mins)
    }
}

pub struct ManifestBuilder {
    manifest: TaskManifest,
}

impl ManifestBuilder {
    pub fn new(app_name: &str, description: &str) -> Self {
        Self {
            manifest: TaskManifest::new(app_name, description),
        }
    }

    pub fn with_tables(mut self, tables: Vec<TableDefinition>) -> Self {
        if tables.is_empty() {
            return self;
        }

        let mut db_section = ManifestSection::new("Database & Models", SectionType::DatabaseModels);

        let mut schema_section =
            ManifestSection::new("Database Schema Design", SectionType::SchemaDesign);

        // Each table becomes an item in the schema section
        for table in &tables {
            let field_count = table.fields.len();
            let field_names: Vec<String> = table.fields.iter().take(4).map(|f| f.name.clone()).collect();
            let fields_preview = if field_count > 4 {
                format!("{}, +{} more", field_names.join(", "), field_count - 4)
            } else {
                field_names.join(", ")
            };

            let mut item = ManifestItem::new(&table.name, ItemType::Table);
            item.details = Some(format!("{} fields: {}", field_count, fields_preview));
            schema_section.add_item(item);
        }

        db_section.add_child(schema_section);
        self.manifest.add_section(db_section);
        self
    }

    pub fn with_files(mut self, files: Vec<FileDefinition>) -> Self {
        if files.is_empty() {
            return self;
        }

        let mut files_section = ManifestSection::new("Files", SectionType::Files);

        // Group files by type for better organization
        let html_files: Vec<_> = files.iter().filter(|f| f.filename.ends_with(".html")).collect();
        let css_files: Vec<_> = files.iter().filter(|f| f.filename.ends_with(".css")).collect();
        let js_files: Vec<_> = files.iter().filter(|f| f.filename.ends_with(".js")).collect();

        // Create child section for HTML pages
        if !html_files.is_empty() {
            let mut pages_child = ManifestSection::new("HTML Pages", SectionType::Pages);
            for file in &html_files {
                let item = ManifestItem::new(&file.filename, ItemType::Page);
                pages_child.add_item(item);
            }
            files_section.add_child(pages_child);
        }

        // Create child section for styles
        if !css_files.is_empty() {
            let mut styles_child = ManifestSection::new("Stylesheets", SectionType::Files);
            for file in &css_files {
                let item = ManifestItem::new(&file.filename, ItemType::File);
                styles_child.add_item(item);
            }
            files_section.add_child(styles_child);
        }

        // Create child section for scripts
        if !js_files.is_empty() {
            let mut scripts_child = ManifestSection::new("Scripts", SectionType::Files);
            for file in &js_files {
                let item = ManifestItem::new(&file.filename, ItemType::File);
                scripts_child.add_item(item);
            }
            files_section.add_child(scripts_child);
        }

        self.manifest.add_section(files_section);
        self
    }

    pub fn with_pages(self, _pages: Vec<PageDefinition>) -> Self {
        // Pages are now included in Files section as HTML Pages child
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        if tools.is_empty() {
            return self;
        }

        let mut tools_section = ManifestSection::new("Tools & Automation", SectionType::Tools);

        let mut automation_child = ManifestSection::new("BASIC Scripts", SectionType::Tools);
        for tool in tools {
            let item = ManifestItem::new(&tool.name, ItemType::Tool)
                .with_details(&tool.filename);
            automation_child.add_item(item);
        }
        tools_section.add_child(automation_child);

        self.manifest.add_section(tools_section);
        self
    }

    pub fn with_schedulers(mut self, schedulers: Vec<SchedulerDefinition>) -> Self {
        if schedulers.is_empty() {
            return self;
        }

        let mut sched_section = ManifestSection::new("Scheduled Tasks", SectionType::Schedulers);

        let mut cron_child = ManifestSection::new("Cron Jobs", SectionType::Schedulers);
        for scheduler in schedulers {
            let item = ManifestItem::new(&scheduler.name, ItemType::Scheduler)
                .with_details(&scheduler.schedule);
            cron_child.add_item(item);
        }
        sched_section.add_child(cron_child);

        self.manifest.add_section(sched_section);
        self
    }

    pub fn with_monitors(mut self, monitors: Vec<MonitorDefinition>) -> Self {
        if monitors.is_empty() {
            return self;
        }

        let mut mon_section = ManifestSection::new("Monitoring", SectionType::Monitors);

        let mut alerts_child = ManifestSection::new("Alert Rules", SectionType::Monitors);
        for monitor in monitors {
            let item =
                ManifestItem::new(&monitor.name, ItemType::Monitor).with_details(&monitor.target);
            alerts_child.add_item(item);
        }
        mon_section.add_child(alerts_child);

        self.manifest.add_section(mon_section);
        self
    }

    pub fn with_estimated_time(mut self, seconds: u64) -> Self {
        self.manifest.estimated_seconds = seconds;
        self
    }

    pub fn build(mut self) -> TaskManifest {
        self.manifest.status = ManifestStatus::Ready;
        self.manifest
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDefinition {
    pub filename: String,
    pub size_estimate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDefinition {
    pub filename: String,
    pub page_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub filename: String,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerDefinition {
    pub name: String,
    pub filename: String,
    pub schedule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorDefinition {
    pub name: String,
    pub target: String,
}

pub fn create_manifest_from_llm_response(
    app_name: &str,
    description: &str,
    tables: Vec<TableDefinition>,
    files: Vec<FileDefinition>,
    pages: Vec<PageDefinition>,
    tools: Vec<ToolDefinition>,
    schedulers: Vec<SchedulerDefinition>,
    monitors: Vec<MonitorDefinition>,
) -> TaskManifest {
    let estimated_time = estimate_generation_time(&tables, &files, &tools, &schedulers);

    ManifestBuilder::new(app_name, description)
        .with_tables(tables)
        .with_files(files)
        .with_pages(pages)
        .with_tools(tools)
        .with_schedulers(schedulers)
        .with_monitors(monitors)
        .with_estimated_time(estimated_time)
        .build()
}

fn estimate_generation_time(
    tables: &[TableDefinition],
    files: &[FileDefinition],
    tools: &[ToolDefinition],
    schedulers: &[SchedulerDefinition],
) -> u64 {
    let table_time: u64 = tables.iter().map(|t| 5 + t.fields.len() as u64).sum();
    let file_time: u64 = files.len() as u64 * 3;
    let tool_time: u64 = tools.len() as u64 * 10;
    let sched_time: u64 = schedulers.len() as u64 * 5;

    table_time + file_time + tool_time + sched_time + 30
}
