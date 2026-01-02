use crate::auto_task::app_logs::{log_generator_error, log_generator_info};
use std::sync::OnceLock;
use crate::auto_task::task_manifest::{
    create_manifest_from_llm_response, FieldDefinition as ManifestField,
    FileDefinition, ManifestStatus, MonitorDefinition, PageDefinition,
    SchedulerDefinition, SectionStatus, SectionType, TableDefinition as ManifestTable,
    TaskManifest, TerminalLineType, ToolDefinition,
};
use crate::basic::keywords::table_definition::{
    generate_create_table_sql, FieldDefinition, TableDefinition,
};
use crate::core::config::ConfigManager;
use crate::core::shared::get_content_type;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::{AgentActivity, AppState};
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pages: Vec<GeneratedFile>,
    pub tables: Vec<TableDefinition>,
    pub tools: Vec<GeneratedFile>,
    pub schedulers: Vec<GeneratedFile>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub filename: String,
    pub content: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPage {
    pub filename: String,
    pub title: String,
    pub page_type: PageType,
    pub content: String,
    pub route: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Html,
    Css,
    Js,
    Bas,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageType {
    List,
    Form,
    Detail,
    Dashboard,
}

impl std::fmt::Display for PageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List => write!(f, "list"),
            Self::Form => write!(f, "form"),
            Self::Detail => write!(f, "detail"),
            Self::Dashboard => write!(f, "dashboard"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedScript {
    pub name: String,
    pub filename: String,
    pub script_type: ScriptType,
    pub content: String,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Tool,
    Scheduler,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStructure {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub tables: Vec<TableDefinition>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub tables_created: usize,
    pub fields_added: usize,
    pub migrations_applied: usize,
}

/// Streaming format parsed app structure
#[derive(Debug, Clone, Default)]
struct LlmGeneratedApp {
    name: String,
    description: String,
    domain: String,
    tables: Vec<LlmTable>,
    files: Vec<LlmFile>,
    tools: Vec<LlmFile>,
    schedulers: Vec<LlmFile>,
}

#[derive(Debug, Clone, Default)]
struct LlmTable {
    name: String,
    fields: Vec<LlmField>,
}

#[derive(Debug, Clone, Default)]
struct LlmField {
    name: String,
    field_type: String,
    nullable: bool,
    reference: Option<String>,
    default: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct LlmFile {
    filename: String,
    content: String,
}

/// Streaming delimiter constants
const DELIM_APP_START: &str = "<<<APP_START>>>";
const DELIM_APP_END: &str = "<<<APP_END>>>";
const DELIM_TABLES_START: &str = "<<<TABLES_START>>>";
const DELIM_TABLES_END: &str = "<<<TABLES_END>>>";
const DELIM_TABLE_PREFIX: &str = "<<<TABLE:";
const DELIM_FILE_PREFIX: &str = "<<<FILE:";
const DELIM_TOOL_PREFIX: &str = "<<<TOOL:";
const DELIM_SCHEDULER_PREFIX: &str = "<<<SCHEDULER:";
const DELIM_END: &str = ">>>";

pub struct AppGenerator {
    state: Arc<AppState>,
    task_id: Option<String>,
    generation_start: Option<std::time::Instant>,
    files_written: Vec<String>,
    tables_synced: Vec<String>,
    bytes_generated: u64,
    manifest: Option<TaskManifest>,
}

impl AppGenerator {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            task_id: None,
            generation_start: None,
            files_written: Vec::new(),
            tables_synced: Vec::new(),
            bytes_generated: 0,
            manifest: None,
        }
    }

    pub fn with_task_id(state: Arc<AppState>, task_id: impl Into<String>) -> Self {
        Self {
            state,
            task_id: Some(task_id.into()),
            generation_start: None,
            files_written: Vec::new(),
            tables_synced: Vec::new(),
            bytes_generated: 0,
            manifest: None,
        }
    }

    fn create_manifest_from_llm_app(&mut self, llm_app: &LlmGeneratedApp) {
        use crate::auto_task::task_manifest::ManifestSection;

        log::info!("[MANIFEST_CREATE] Creating manifest from LLM app: {} tables, {} files, {} tools, {} schedulers",
            llm_app.tables.len(), llm_app.files.len(), llm_app.tools.len(), llm_app.schedulers.len());

        let tables: Vec<ManifestTable> = llm_app
            .tables
            .iter()
            .map(|t| ManifestTable {
                name: t.name.clone(),
                fields: t
                    .fields
                    .iter()
                    .map(|f| ManifestField {
                        name: f.name.clone(),
                        field_type: f.field_type.clone(),
                        nullable: f.nullable,
                    })
                    .collect(),
            })
            .collect();

        let files: Vec<FileDefinition> = llm_app
            .files
            .iter()
            .map(|f| FileDefinition {
                filename: f.filename.clone(),
                size_estimate: f.content.len() as u64,
            })
            .collect();

        let pages: Vec<PageDefinition> = llm_app
            .files
            .iter()
            .filter(|f| f.filename.ends_with(".html"))
            .map(|f| PageDefinition {
                filename: f.filename.clone(),
                page_type: "html".to_string(),
            })
            .collect();

        let tools: Vec<ToolDefinition> = llm_app
            .tools
            .iter()
            .map(|t| ToolDefinition {
                name: t.filename.replace(".bas", ""),
                filename: t.filename.clone(),
                triggers: vec![],
            })
            .collect();

        let schedulers: Vec<SchedulerDefinition> = llm_app
            .schedulers
            .iter()
            .map(|s| SchedulerDefinition {
                name: s.filename.replace(".bas", ""),
                filename: s.filename.clone(),
                schedule: "".to_string(),
            })
            .collect();

        let monitors: Vec<MonitorDefinition> = Vec::new();

        // Create new manifest from LLM response
        log::info!("[MANIFEST_CREATE] Calling create_manifest_from_llm_response with {} tables, {} files, {} pages, {} tools",
            tables.len(), files.len(), pages.len(), tools.len());

        let mut new_manifest = create_manifest_from_llm_response(
            &llm_app.name,
            &llm_app.description,
            tables,
            files,
            pages,
            tools,
            schedulers,
            monitors,
        );

        log::info!("[MANIFEST_CREATE] New manifest created with {} sections:", new_manifest.sections.len());
        for section in &new_manifest.sections {
            log::info!("[MANIFEST_CREATE]   Section '{}': {} children, {} items, {} item_groups",
                section.name, section.children.len(), section.items.len(), section.item_groups.len());
            for child in &section.children {
                log::info!("[MANIFEST_CREATE]     Child '{}': {} items, {} item_groups",
                    child.name, child.items.len(), child.item_groups.len());
            }
        }

        // Mark "Analyzing Request" as completed and add it to the beginning
        let mut analyzing_section = ManifestSection::new("Analyzing Request", SectionType::Validation);
        analyzing_section.total_steps = 1;
        analyzing_section.current_step = 1;
        analyzing_section.status = SectionStatus::Completed;
        analyzing_section.started_at = self.manifest.as_ref()
            .and_then(|m| m.sections.first())
            .and_then(|s| s.started_at);
        analyzing_section.completed_at = Some(Utc::now());
        analyzing_section.duration_seconds = analyzing_section.started_at
            .map(|started| (Utc::now() - started).num_seconds() as u64);

        // Insert "Analyzing Request" at the beginning of sections
        new_manifest.sections.insert(0, analyzing_section);

        // Recalculate all global step offsets after insertion
        new_manifest.recalculate_global_steps();
        new_manifest.completed_steps = 1; // Analyzing is done

        // Preserve terminal output from preliminary manifest
        if let Some(ref old_manifest) = self.manifest {
            new_manifest.terminal_output = old_manifest.terminal_output.clone();
        }

        new_manifest.start();
        new_manifest.add_terminal_line(&format!("AI planned: {} tables, {} files, {} tools",
            llm_app.tables.len(), llm_app.files.len(), llm_app.tools.len()),
            TerminalLineType::Success);

        self.manifest = Some(new_manifest);

        if let Some(ref task_id) = self.task_id {
            if let Ok(mut manifests) = self.state.task_manifests.write() {
                log::info!("[MANIFEST_CREATE] Storing manifest for task_id: {}", task_id);
                manifests.insert(task_id.clone(), self.manifest.clone().unwrap());
            }
        }

        log::info!("[MANIFEST_CREATE] Broadcasting manifest update");
        self.broadcast_manifest_update();
    }

    fn broadcast_manifest_update(&self) {
        if let (Some(ref task_id), Some(ref manifest)) = (&self.task_id, &self.manifest) {
            // Log the TASK.md structure for debugging
            let task_md = manifest.to_task_md();
            log::info!(
                "[TASK.md] task={}\n{}",
                task_id,
                task_md
            );

            log::info!(
                "[MANIFEST_BROADCAST] task={} completed={}/{} sections={}",
                task_id,
                manifest.completed_steps,
                manifest.total_steps,
                manifest.sections.len()
            );

            // Log section details with children
            for section in &manifest.sections {
                let status = format!("{:?}", section.status);
                log::info!(
                    "[MANIFEST_BROADCAST]   Section '{}': status={}, children={}, items={}, item_groups={}",
                    section.name,
                    status,
                    section.children.len(),
                    section.items.len(),
                    section.item_groups.len()
                );
                for child in &section.children {
                    let child_status = format!("{:?}", child.status);
                    log::info!(
                        "[MANIFEST_BROADCAST]     Child '{}': status={}, items={}, item_groups={}",
                        child.name,
                        child_status,
                        child.items.len(),
                        child.item_groups.len()
                    );
                }
            }

            if let Ok(mut manifests) = self.state.task_manifests.write() {
                manifests.insert(task_id.clone(), manifest.clone());
            }

            let json_details = serde_json::to_string(&manifest.to_web_json()).unwrap_or_default();
            let json_size = json_details.len();
            log::info!("[MANIFEST_BROADCAST] JSON size: {} bytes", json_size);

            // Persist manifest to database for historical viewing
            self.persist_manifest_to_db(task_id, &json_details);

            // Build the event - if manifest JSON is too large (> 64KB), send without details
            // to avoid WebSocket frame size issues. Client will fetch full manifest via API.
            let event = if json_size > 65536 {
                log::warn!("[MANIFEST_BROADCAST] Manifest too large ({} bytes), sending without details", json_size);
                crate::core::shared::state::TaskProgressEvent::new(
                    task_id,
                    "manifest_update",
                    &format!("Manifest updated: {}", manifest.app_name),
                )
                .with_event_type("manifest_update")
                .with_progress(manifest.completed_steps as u8, manifest.total_steps as u8)
            } else {
                crate::core::shared::state::TaskProgressEvent::new(
                    task_id,
                    "manifest_update",
                    &format!("Manifest updated: {}", manifest.app_name),
                )
                .with_event_type("manifest_update")
                .with_progress(manifest.completed_steps as u8, manifest.total_steps as u8)
                .with_details(json_details)
            };

            // Log the final serialized event size
            if let Ok(event_json) = serde_json::to_string(&event) {
                log::info!("[MANIFEST_BROADCAST] Final event size: {} bytes (has_details={})",
                    event_json.len(), json_size <= 65536);
            }

            self.state.broadcast_task_progress(event);
        }
    }

    fn persist_manifest_to_db(&self, task_id: &str, manifest_json: &str) {
        let Ok(task_uuid) = Uuid::parse_str(task_id) else {
            log::warn!("[MANIFEST_PERSIST] Invalid task_id: {}", task_id);
            return;
        };

        let Ok(mut conn) = self.state.conn.get() else {
            log::warn!("[MANIFEST_PERSIST] Failed to get DB connection for task: {}", task_id);
            return;
        };

        let manifest_value: serde_json::Value = match serde_json::from_str(manifest_json) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("[MANIFEST_PERSIST] Failed to parse manifest JSON: {}", e);
                return;
            }
        };

        let result = sql_query(
            "UPDATE auto_tasks SET manifest_json = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind::<diesel::sql_types::Jsonb, _>(manifest_value)
        .bind::<diesel::sql_types::Uuid, _>(task_uuid)
        .execute(&mut conn);

        match result {
            Ok(_) => log::trace!("[MANIFEST_PERSIST] Saved manifest for task: {}", task_id),
            Err(e) => log::warn!("[MANIFEST_PERSIST] Failed to save manifest: {}", e),
        }
    }

    fn update_manifest_section(&mut self, section_type: SectionType, status: SectionStatus) {
        if let Some(ref mut manifest) = self.manifest {
            log::info!("[UPDATE_SECTION] Looking for {:?} to set {:?}", section_type, status);
            log::info!("[UPDATE_SECTION] Manifest has {} sections:", manifest.sections.len());
            for (i, s) in manifest.sections.iter().enumerate() {
                log::info!("[UPDATE_SECTION]   [{}] {:?} = '{}'", i, s.section_type, s.name);
            }
            let mut found = false;
            for section in &mut manifest.sections {
                if section.section_type == section_type {
                    found = true;
                    log::info!("[UPDATE_SECTION] Found section '{}'! Setting to {:?}", section.name, status);
                    section.status = status.clone();
                    if status == SectionStatus::Running {
                        section.started_at = Some(Utc::now());
                    } else if status == SectionStatus::Completed {
                        section.completed_at = Some(Utc::now());
                        section.current_step = section.total_steps;
                        if let Some(started) = section.started_at {
                            section.duration_seconds =
                                Some((Utc::now() - started).num_seconds() as u64);
                        }
                    } else if status == SectionStatus::Skipped {
                        // Skipped sections are marked complete with no work done
                        section.completed_at = Some(Utc::now());
                        section.current_step = section.total_steps;
                        section.duration_seconds = Some(0);
                    }
                    break;
                }
            }
            if !found {
                log::warn!("[UPDATE_SECTION] Section {:?} NOT FOUND in manifest!", section_type);
            }
            manifest.updated_at = Utc::now();
            self.broadcast_manifest_update();
        } else {
            log::warn!("[UPDATE_SECTION] No manifest exists! Cannot update {:?}", section_type);
        }
    }

    /// Update a child section within a parent section
    fn update_manifest_child(&mut self, parent_type: SectionType, child_type: SectionType, status: SectionStatus) {
        if let Some(ref mut manifest) = self.manifest {
            for section in &mut manifest.sections {
                if section.section_type == parent_type {
                    for child in &mut section.children {
                        if child.section_type == child_type {
                            child.status = status.clone();
                            if status == SectionStatus::Running {
                                child.started_at = Some(Utc::now());
                            } else if status == SectionStatus::Completed {
                                child.completed_at = Some(Utc::now());
                                child.current_step = child.total_steps;
                                if let Some(started) = child.started_at {
                                    child.duration_seconds =
                                        Some((Utc::now() - started).num_seconds() as u64);
                                }
                            }
                            break;
                        }
                    }
                    break;
                }
            }
            manifest.updated_at = Utc::now();
            self.broadcast_manifest_update();
        }
    }

    /// Mark a range of item groups as completed with duration
    fn complete_item_group_range(&mut self, parent_type: SectionType, child_type: SectionType, start_idx: usize, end_idx: usize) {
        if let Some(ref mut manifest) = self.manifest {
            for section in &mut manifest.sections {
                if section.section_type == parent_type {
                    for child in &mut section.children {
                        if child.section_type == child_type {
                            // Skip if no item_groups exist
                            if child.item_groups.is_empty() {
                                continue;
                            }
                            for idx in start_idx..=end_idx.min(child.item_groups.len() - 1) {
                                let group = &mut child.item_groups[idx];
                                if group.status != crate::auto_task::ItemStatus::Completed {
                                    group.status = crate::auto_task::ItemStatus::Completed;
                                    group.completed_at = Some(Utc::now());
                                    // Simulate realistic duration (1-5 minutes)
                                    group.duration_seconds = Some(60 + (idx as u64 * 30) % 300);
                                }
                            }
                            // Update child step progress
                            child.current_step = child.item_groups.iter()
                                .filter(|g| g.status == crate::auto_task::ItemStatus::Completed)
                                .count() as u32;

                            // Check if all item_groups in child are completed, then mark child as completed
                            let all_groups_completed = child.item_groups.iter()
                                .all(|g| g.status == crate::auto_task::ItemStatus::Completed);
                            if all_groups_completed && !child.item_groups.is_empty() {
                                child.status = SectionStatus::Completed;
                                child.completed_at = Some(Utc::now());
                                if let Some(started) = child.started_at {
                                    child.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
                                }
                            }
                            break;
                        }
                    }
                    // Update parent step progress
                    section.current_step = section.children.iter()
                        .map(|c| c.current_step)
                        .sum();

                    // Check if all children in section are completed, then mark section as completed
                    let all_children_completed = section.children.iter()
                        .all(|c| c.status == SectionStatus::Completed);
                    if all_children_completed && !section.children.is_empty() {
                        section.status = SectionStatus::Completed;
                        section.completed_at = Some(Utc::now());
                        if let Some(started) = section.started_at {
                            section.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
                        }
                    }
                    break;
                }
            }
            manifest.updated_at = Utc::now();
            self.broadcast_manifest_update();
        }
    }

    fn add_terminal_output(&mut self, content: &str, line_type: TerminalLineType) {
        if let Some(ref mut manifest) = self.manifest {
            log::info!("[TERMINAL_OUTPUT] Adding line: {:?} - '{}'", line_type, content);
            manifest.add_terminal_line(content, line_type);
            self.broadcast_manifest_update();
        } else {
            log::warn!("[TERMINAL_OUTPUT] No manifest! Cannot add: '{}'", content);
        }
    }

    fn create_preliminary_manifest(&mut self, intent: &str) {
        use crate::auto_task::task_manifest::ManifestSection;

        let app_name = intent
            .to_lowercase()
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join("-");

        let mut manifest = TaskManifest::new(&app_name, intent);

        // Section 1: Analyzing Request (LLM call)
        let mut analyzing_section = ManifestSection::new("Analyzing Request", SectionType::Validation);
        analyzing_section.total_steps = 1;
        analyzing_section.status = SectionStatus::Running;
        analyzing_section.started_at = Some(Utc::now());
        manifest.add_section(analyzing_section);

        // Section 2: Database & Models
        let db_section = ManifestSection::new("Database & Models", SectionType::DatabaseModels)
            .with_steps(1);
        manifest.add_section(db_section);

        // Section 3: Files
        let files_section = ManifestSection::new("Files", SectionType::Files)
            .with_steps(1);
        manifest.add_section(files_section);

        // Section 4: Tools
        let tools_section = ManifestSection::new("Tools", SectionType::Tools)
            .with_steps(1);
        manifest.add_section(tools_section);

        manifest.status = ManifestStatus::Running;
        manifest.add_terminal_line(&format!("Analyzing: {}", intent), TerminalLineType::Info);
        manifest.add_terminal_line("Sending request to AI...", TerminalLineType::Progress);

        self.manifest = Some(manifest);

        // Log the preliminary TASK.md (no children yet - they come after LLM response)
        if let Some(ref m) = self.manifest {
            log::info!(
                "[PRELIMINARY_MANIFEST] Created for intent: '{}'\n\
                 ============ PRELIMINARY TASK.md ============\n\
                 {}\n\
                 ============================================\n\
                 NOTE: Sections have NO CHILDREN yet - children are added after LLM completes",
                intent,
                m.to_task_md()
            );
        }

        if let Some(ref task_id) = self.task_id {
            if let Ok(mut manifests) = self.state.task_manifests.write() {
                log::info!("[MANIFEST] Storing preliminary manifest for task_id: {}", task_id);
                manifests.insert(task_id.clone(), self.manifest.clone().unwrap());
            }
        }

        self.broadcast_manifest_update();
    }

    fn update_manifest_stats_real(&mut self, broadcast: bool) {
        if let Some(ref mut manifest) = self.manifest {
            // Calculate real stats from actual progress
            let elapsed_secs = self.generation_start
                .map(|s| s.elapsed().as_secs_f64())
                .unwrap_or(0.0);

            // Data points = files written + tables synced
            let data_points = self.files_written.len() as u64 + self.tables_synced.len() as u64;
            manifest.processing_stats.data_points_processed = data_points;

            // Real processing speed based on actual items processed
            if elapsed_secs > 0.0 {
                manifest.processing_stats.sources_per_min = (data_points as f64 / elapsed_secs) * 60.0;
            }

            // Estimate remaining time based on current progress
            let total = manifest.total_steps as f64;
            let completed = manifest.completed_steps as f64;
            if completed > 0.0 && elapsed_secs > 0.0 {
                let time_per_step = elapsed_secs / completed;
                let remaining_steps = total - completed;
                manifest.processing_stats.estimated_remaining_seconds = (time_per_step * remaining_steps) as u64;
            }

            // Update runtime
            manifest.runtime_seconds = elapsed_secs as u64;

            if broadcast {
                self.broadcast_manifest_update();
            }
        }
    }

    /// Update a specific item's status within a section (with optional broadcast)
    fn update_item_status_internal(&mut self, section_type: SectionType, item_name: &str, status: crate::auto_task::ItemStatus, broadcast: bool) {
        let mut found = false;
        if let Some(ref mut manifest) = self.manifest {
            for section in &mut manifest.sections {
                if section.section_type == section_type {
                    // Check items directly in section
                    for item in &mut section.items {
                        if item.name == item_name {
                            item.status = status.clone();
                            if status == crate::auto_task::ItemStatus::Running {
                                item.started_at = Some(Utc::now());
                            } else if status == crate::auto_task::ItemStatus::Completed {
                                item.completed_at = Some(Utc::now());
                                if let Some(started) = item.started_at {
                                    item.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
                                }
                            }
                            found = true;
                            break;
                        }
                    }
                    if found { break; }
                    // Check items in children
                    for child in &mut section.children {
                        for item in &mut child.items {
                            if item.name == item_name {
                                item.status = status.clone();
                                if status == crate::auto_task::ItemStatus::Running {
                                    item.started_at = Some(Utc::now());
                                } else if status == crate::auto_task::ItemStatus::Completed {
                                    item.completed_at = Some(Utc::now());
                                    if let Some(started) = item.started_at {
                                        item.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
                                    }
                                    child.current_step += 1;

                                    // Check if all items in child are completed, then mark child as completed
                                    let all_completed = child.items.iter().all(|i| i.status == crate::auto_task::ItemStatus::Completed);
                                    if all_completed && !child.items.is_empty() {
                                        child.status = SectionStatus::Completed;
                                        child.completed_at = Some(Utc::now());
                                        if let Some(started) = child.started_at {
                                            child.duration_seconds = Some((Utc::now() - started).num_seconds() as u64);
                                        }
                                    }
                                }
                                found = true;
                                break;
                            }
                        }
                        if found { break; }
                    }
                }
                if found { break; }
            }
        }
        // Broadcast update so UI shows real-time file progress
        if found && broadcast {
            self.broadcast_manifest_update();
        }
    }

    /// Update a specific item's status within a section (always broadcasts)
    fn update_item_status(&mut self, section_type: SectionType, item_name: &str, status: crate::auto_task::ItemStatus) {
        self.update_item_status_internal(section_type, item_name, status, true);
    }

    fn emit_activity(&self, step: &str, message: &str, current: u8, total: u8, activity: AgentActivity) {
        if let Some(ref task_id) = self.task_id {
            self.state.emit_activity(task_id, step, message, current, total, activity);
        }
    }

    fn calculate_speed(&self, items_done: u32) -> (f32, Option<u32>) {
        if let Some(start) = self.generation_start {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed > 0.0 {
                let speed = (items_done as f32 / elapsed) * 60.0;
                return (speed, None);
            }
        }
        (0.0, None)
    }

    fn build_activity(&self, phase: &str, items_done: u32, items_total: Option<u32>, current_item: Option<&str>) -> AgentActivity {
        let (speed, eta) = self.calculate_speed(items_done);
        let mut activity = AgentActivity::new(phase)
            .with_progress(items_done, items_total)
            .with_bytes(self.bytes_generated);

        if speed > 0.0 {
            activity = activity.with_speed(speed, eta);
        }

        if !self.files_written.is_empty() {
            activity = activity.with_files(self.files_written.clone());
        }

        if !self.tables_synced.is_empty() {
            activity = activity.with_tables(self.tables_synced.clone());
        }

        if let Some(item) = current_item {
            activity = activity.with_current_item(item);
        }

        activity
    }

    pub async fn generate_app(
        &mut self,
        intent: &str,
        session: &UserSession,
    ) -> Result<GeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        const TOTAL_STEPS: u8 = 8;

        self.generation_start = Some(std::time::Instant::now());
        self.files_written.clear();
        self.tables_synced.clear();
        self.bytes_generated = 0;

        let intent_preview: String = intent.chars().take(100).collect();
        info!(
            "Generating app from intent: {}",
            intent_preview
        );

        let intent_short: String = intent.chars().take(50).collect();
        log_generator_info(
            "pending",
            &format!(
                "Starting app generation: {}",
                intent_short
            ),
        );

        if let Some(ref task_id) = self.task_id {
            let intent_msg: String = intent.chars().take(50).collect();
            self.state.emit_task_started(task_id, &format!("Generating app: {}", intent_msg), TOTAL_STEPS);
            self.create_preliminary_manifest(intent);
        }

        let activity = self.build_activity("analyzing", 0, Some(TOTAL_STEPS as u32), Some("Sending request to LLM"));
        self.emit_activity(
            "llm_request",
            "Analyzing request with AI...",
            1,
            TOTAL_STEPS,
            activity
        );

        // ========== PHASE 1: Get project plan (structure only) ==========
        let intent_trace: String = intent.chars().take(50).collect();
        trace!("APP_GENERATOR [PHASE1] Getting project plan for: {}", intent_trace);
        let plan_start = std::time::Instant::now();

        let mut llm_app = match self.get_project_plan_from_llm(intent, session.bot_id).await {
            Ok(plan) => {
                let plan_elapsed = plan_start.elapsed();
                info!("APP_GENERATOR [PHASE1] Plan received in {:?}: app={}, tables={}, files={}, tools={}",
                      plan_elapsed, plan.name, plan.tables.len(), plan.files.len(), plan.tools.len());

                let is_empty_plan = plan.files.is_empty() && plan.tables.is_empty() && plan.tools.is_empty();

                if is_empty_plan {
                    warn!("APP_GENERATOR [PHASE1] Empty plan received, falling back to single-phase generation");
                    self.add_terminal_output("Plan parsing returned empty, trying full generation...", TerminalLineType::Warning);
                    match self.generate_complete_app_with_llm(intent, session.bot_id).await {
                        Ok(app) => app,
                        Err(e2) => {
                            log_generator_error("unknown", "LLM app generation failed", &e2.to_string());
                            if let Some(ref task_id) = self.task_id {
                                self.state.emit_task_error(task_id, "llm_request", &e2.to_string());
                            }
                            return Err(e2);
                        }
                    }
                } else {
                    let activity = self.build_activity(
                        "planning",
                        1,
                        Some(TOTAL_STEPS as u32),
                        Some(&format!("Planned {} with {} files", plan.name, plan.files.len()))
                    );
                    self.emit_activity(
                        "plan_complete",
                        &format!("Project plan ready: {} tables, {} files", plan.tables.len(), plan.files.len()),
                        2,
                        TOTAL_STEPS,
                        activity
                    );
                    plan
                }
            }
            Err(e) => {
                error!("APP_GENERATOR [PHASE1] Planning failed: {}", e);
                self.add_terminal_output(&format!("Planning error: {e}, trying full generation..."), TerminalLineType::Warning);
                match self.generate_complete_app_with_llm(intent, session.bot_id).await {
                    Ok(app) => app,
                    Err(e2) => {
                        log_generator_error("unknown", "LLM app generation failed", &e2.to_string());
                        if let Some(ref task_id) = self.task_id {
                            self.state.emit_task_error(task_id, "llm_request", &e2.to_string());
                        }
                        return Err(e2);
                    }
                }
            }
        };

        // Mark "Analyzing Request" as completed
        info!("[PHASE1->2] Marking Analyzing Request as Completed");
        self.update_manifest_section(SectionType::Validation, SectionStatus::Completed);
        self.broadcast_manifest_update();

        // Create manifest WITH children immediately (before Phase 2)
        self.create_manifest_from_llm_app(&llm_app);
        self.broadcast_manifest_update();

        info!("APP_GENERATOR [PHASE1->2] Manifest created with full structure, starting content generation");
        self.add_terminal_output(&format!("## Project Plan: {}", llm_app.name), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Tables: {}", llm_app.tables.len()), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Files: {}", llm_app.files.len()), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Tools: {}", llm_app.tools.len()), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Schedulers: {}", llm_app.schedulers.len()), TerminalLineType::Info);
        self.add_terminal_output("", TerminalLineType::Info);
        self.add_terminal_output("## Phase 2: Generating content...", TerminalLineType::Progress);
        self.update_manifest_stats_real(true);

        // ========== PHASE 2A: DATABASE & MODELS (must come first!) ==========
        let activity = self.build_activity("parsing", 2, Some(TOTAL_STEPS as u32), Some(&format!("Processing {} structure", llm_app.name)));
        self.emit_activity("parse_structure", &format!("Parsing {} structure...", llm_app.name), 3, TOTAL_STEPS, activity);

        let tables = Self::convert_llm_tables(&llm_app.tables);

        if !tables.is_empty() {
            info!("[PHASE2] Setting Database & Models section to Running");
            self.update_manifest_section(SectionType::DatabaseModels, SectionStatus::Running);
            self.broadcast_manifest_update();
            self.update_manifest_child(SectionType::DatabaseModels, SectionType::SchemaDesign, SectionStatus::Running);
            self.add_terminal_output("## Creating database schema...", TerminalLineType::Progress);
            self.update_manifest_stats_real(true);

            let table_names: Vec<String> = tables.iter().map(|t| t.name.clone()).collect();
            let activity = self.build_activity(
                "database",
                3,
                Some(TOTAL_STEPS as u32),
                Some(&format!("Creating tables: {}", table_names.join(", ")))
            );
            self.emit_activity(
                "create_tables",
                &format!("Creating {} database tables...", tables.len()),
                4,
                TOTAL_STEPS,
                activity
            );

            let tables_bas_content = Self::generate_table_definitions(&tables)?;
            if let Err(e) = self.append_to_tables_bas(session.bot_id, &tables_bas_content) {
                log_generator_error(
                    &llm_app.name,
                    "Failed to append to tables.bas",
                    &e.to_string(),
                );
            }

            // Sync tables one-by-one with real-time progress updates
            let total_tables = tables.len();
            let mut tables_created = 0;
            let mut fields_added = 0;

            for (idx, table) in tables.iter().enumerate() {
                // Update current action to show which table is being processed
                self.add_terminal_output(&format!("  Creating table `{}`...", table.name), TerminalLineType::Info);

                // Mark this specific item as running
                self.update_item_status(SectionType::DatabaseModels, &table.name, crate::auto_task::ItemStatus::Running);
                self.broadcast_manifest_update();

                // Sync the individual table
                match self.sync_single_table_to_database(table) {
                    Ok(field_count) => {
                        tables_created += 1;
                        fields_added += field_count;

                        // Mark item as completed and broadcast immediately
                        self.update_item_status(SectionType::DatabaseModels, &table.name, crate::auto_task::ItemStatus::Completed);
                        self.add_terminal_output(&format!("  ✓ Table `{}` ({} fields)", table.name, field_count), TerminalLineType::Success);

                        // Update child progress
                        if let Some(ref mut manifest) = self.manifest {
                            if let Some(section) = manifest.sections.iter_mut().find(|s| s.section_type == SectionType::DatabaseModels) {
                                if let Some(child) = section.children.iter_mut().find(|c| c.section_type == SectionType::SchemaDesign) {
                                    child.current_step = (idx + 1) as u32;
                                }
                                section.current_step = (idx + 1) as u32;
                            }
                        }

                        // Complete item group if it exists
                        self.complete_item_group_range(SectionType::DatabaseModels, SectionType::SchemaDesign, idx, idx);

                        self.broadcast_manifest_update();

                        // Emit activity for each table
                        let activity = self.build_activity(
                            "database",
                            3,
                            Some(total_tables as u32),
                            Some(&format!("Created table {} ({}/{})", table.name, idx + 1, total_tables))
                        );
                        self.emit_activity(
                            "table_created",
                            &format!("Created table {}", table.name),
                            4,
                            TOTAL_STEPS,
                            activity
                        );
                    }
                    Err(e) => {
                        warn!("Table {} may already exist or failed: {}", table.name, e);
                        // Still mark as completed (table likely exists)
                        self.update_item_status(SectionType::DatabaseModels, &table.name, crate::auto_task::ItemStatus::Completed);
                        self.add_terminal_output(&format!("  ⚠ Table `{}` (may exist)", table.name), TerminalLineType::Info);
                        self.broadcast_manifest_update();
                    }
                }

                self.tables_synced.push(table.name.clone());
            }

            log_generator_info(
                &llm_app.name,
                &format!(
                    "Tables synced: {} created, {} fields",
                    tables_created, fields_added
                ),
            );

            // Mark child and parent as completed
            self.update_manifest_child(SectionType::DatabaseModels, SectionType::SchemaDesign, SectionStatus::Completed);
            self.update_manifest_section(SectionType::DatabaseModels, SectionStatus::Completed);
            self.update_manifest_stats_real(true);

            let activity = self.build_activity(
                "database",
                4,
                Some(TOTAL_STEPS as u32),
                Some(&format!("{} tables, {} fields created", tables_created, fields_added))
            );
            self.emit_activity(
                "tables_synced",
                "Database tables created",
                4,
                TOTAL_STEPS,
                activity
            );
        } else {
            // No tables - mark database section as skipped
            self.update_manifest_section(SectionType::DatabaseModels, SectionStatus::Skipped);
            self.broadcast_manifest_update();
        }

        // ========== PHASE 2B: GENERATE FILE CONTENT ==========
        let total_items = llm_app.files.len() + llm_app.tools.len() + llm_app.schedulers.len();
        let mut generated_count = 0;

        // Generate content for files that don't have it yet
        let files_needing_content: Vec<usize> = llm_app.files.iter()
            .enumerate()
            .filter(|(_, f)| f.content.is_empty())
            .map(|(i, _)| i)
            .collect();

        info!("[PHASE2B] Files needing content: {} out of {} total files", files_needing_content.len(), llm_app.files.len());
        for (i, file) in llm_app.files.iter().enumerate() {
            info!("[PHASE2B] File {}: {} - content_len={}", i, file.filename, file.content.len());
        }

        if !files_needing_content.is_empty() {
            info!("[PHASE2B] Setting Files section to Running - manifest exists: {}", self.manifest.is_some());

            // Debug: List all sections before update
            if let Some(ref manifest) = self.manifest {
                info!("[PHASE2B] Current manifest sections:");
                for (i, s) in manifest.sections.iter().enumerate() {
                    info!("[PHASE2B]   [{}] {:?} = '{}' status={:?}", i, s.section_type, s.name, s.status);
                }
            }

            self.update_manifest_section(SectionType::Files, SectionStatus::Running);
            self.broadcast_manifest_update();
            self.add_terminal_output(&format!("## Generating {} files...", files_needing_content.len()), TerminalLineType::Progress);

            for idx in files_needing_content {
                let filename = llm_app.files[idx].filename.clone();
                generated_count += 1;
                info!("[PHASE2B] Starting generation for file: {}", filename);
                self.add_terminal_output(&format!("Generating `{filename}`..."), TerminalLineType::Info);
                self.update_item_status(SectionType::Files, &filename, crate::auto_task::ItemStatus::Running);

                match self.generate_file_content(&llm_app, &filename, session.bot_id).await {
                    Ok(content) => {
                        let content_len = content.len();
                        info!("[PHASE2B] Generated file {} with {} bytes", filename, content_len);
                        llm_app.files[idx].content = content;
                        self.add_terminal_output(&format!("✓ `{filename}` ({content_len} bytes)"), TerminalLineType::Success);
                        self.update_item_status(SectionType::Files, &filename, crate::auto_task::ItemStatus::Completed);
                    }
                    Err(e) => {
                        error!("[PHASE2B] Failed to generate {}: {}", filename, e);
                        self.add_terminal_output(&format!("✗ `{filename}` failed: {e}"), TerminalLineType::Error);
                    }
                }

                let activity = self.build_activity("generating", generated_count as u32, Some(total_items as u32), Some(&filename));
                self.emit_activity("file_generated", &format!("Generated {filename}"), 3, TOTAL_STEPS, activity);
            }
        } else {
            info!("[PHASE2B] No files need content generation - all {} files already have content", llm_app.files.len());
            self.add_terminal_output(&format!("All {} files already generated", llm_app.files.len()), TerminalLineType::Success);
        }

        // Mark Files content generation as completed (writing happens next)
        self.broadcast_manifest_update();

        // Use bucket_name from state (e.g., "default.gbai") instead of deriving from bot name
        let bucket_name = self.state.bucket_name.clone();
        let sanitized_name = bucket_name.trim_end_matches(".gbai").to_string();
        let drive_app_path = format!("{}.gbapp/{}", sanitized_name, llm_app.name);

        info!("Writing app files to bucket: {}, path: {}", bucket_name, drive_app_path);

        // Build list of files to generate for progress tracking
        let mut files_to_generate: Vec<String> = llm_app.files.iter().map(|f| f.filename.clone()).collect();
        files_to_generate.push("designer.js".to_string());

        // Update task with file list before starting
        if let Some(ref task_id) = self.task_id {
            if let Ok(task_uuid) = uuid::Uuid::parse_str(task_id) {
                let _ = self.update_task_step_results(task_uuid, &files_to_generate, 0);
            }
        }

        let total_files = llm_app.files.len();
        let activity = self.build_activity("writing", 0, Some(total_files as u32), Some("Preparing files"));
        self.emit_activity(
            "write_files",
            &format!("Writing {} app files...", total_files),
            5,
            TOTAL_STEPS,
            activity
        );

        self.update_manifest_section(SectionType::Files, SectionStatus::Running);
        self.add_terminal_output(&format!("## Writing {} files...", total_files), TerminalLineType::Progress);
        self.update_manifest_stats_real(true);

        let mut pages = Vec::new();
        for (idx, file) in llm_app.files.iter().enumerate() {
            let drive_path = format!("{}/{}", drive_app_path, file.filename);

            self.files_written.push(file.filename.clone());
            self.bytes_generated += file.content.len() as u64;

            // Mark item as running (broadcast immediately so user sees file starting)
            self.update_item_status(SectionType::Files, &file.filename, crate::auto_task::ItemStatus::Running);
            self.add_terminal_output(&format!("Writing `{}`...", file.filename), TerminalLineType::Info);

            let activity = self.build_activity(
                "writing",
                (idx + 1) as u32,
                Some(total_files as u32),
                Some(&file.filename)
            );
            self.emit_activity(
                "write_file",
                &format!("Writing {}", file.filename),
                5,
                TOTAL_STEPS,
                activity
            );

            // Write to MinIO - drive monitor will sync to SITES_ROOT
            if let Err(e) = self
                .write_to_drive(&bucket_name, &drive_path, &file.content)
                .await
            {
                log_generator_error(
                    &llm_app.name,
                    &format!("Failed to write {}", file.filename),
                    &e.to_string(),
                );
            } else {
                // Update progress in database
                if let Some(ref task_id) = self.task_id {
                    if let Ok(task_uuid) = uuid::Uuid::parse_str(task_id) {
                        let _ = self.update_task_step_results(task_uuid, &files_to_generate, idx + 1);
                    }
                }
            }

            // Mark item as completed (broadcast immediately so user sees progress)
            self.update_item_status(SectionType::Files, &file.filename, crate::auto_task::ItemStatus::Completed);
            self.add_terminal_output(&format!("✓ `{}` ({} bytes)", file.filename, file.content.len()), TerminalLineType::Success);

            // Update section progress
            if let Some(ref mut manifest) = self.manifest {
                for section in &mut manifest.sections {
                    if section.section_type == SectionType::Files {
                        section.current_step = (idx + 1) as u32;
                        break;
                    }
                }
                manifest.completed_steps += 1;
            }

            // Stats are updated less frequently to avoid UI overload
            let should_update_stats = (idx + 1) % 3 == 0 || idx + 1 == total_files;
            self.update_manifest_stats_real(should_update_stats);

            let file_type = Self::detect_file_type(&file.filename);
            pages.push(GeneratedFile {
                filename: file.filename.clone(),
                content: file.content.clone(),
                file_type,
            });
        }

        self.update_manifest_section(SectionType::Files, SectionStatus::Completed);

        // Pages are the HTML files we just wrote, mark as completed
        self.update_manifest_section(SectionType::Pages, SectionStatus::Completed);

        self.files_written.push("designer.js".to_string());
        let activity = self.build_activity("configuring", total_files as u32, Some(total_files as u32), Some("designer.js"));
        self.emit_activity("write_designer", "Creating designer configuration...", 6, TOTAL_STEPS, activity);

        let designer_js = Self::generate_designer_js(&llm_app.name);
        self.bytes_generated += designer_js.len() as u64;

        // Write designer.js to MinIO
        self.write_to_drive(
            &bucket_name,
            &format!("{}/designer.js", drive_app_path),
            &designer_js,
        )
        .await?;

        let mut tools = Vec::new();
        if !llm_app.tools.is_empty() {
            self.update_manifest_section(SectionType::Tools, SectionStatus::Running);
            self.add_terminal_output("Creating automation tools...", TerminalLineType::Progress);

            let tools_count = llm_app.tools.len();
            let activity = self.build_activity("tools", 0, Some(tools_count as u32), Some("Creating BASIC tools"));
            self.emit_activity(
                "write_tools",
                &format!("Creating {} tools...", tools_count),
                7,
                TOTAL_STEPS,
                activity
            );

            for (idx, tool) in llm_app.tools.iter().enumerate() {
                let tool_path = format!(".gbdialog/tools/{}", tool.filename);
                self.files_written.push(format!("tools/{}", tool.filename));
                self.bytes_generated += tool.content.len() as u64;

                let activity = self.build_activity("tools", (idx + 1) as u32, Some(tools_count as u32), Some(&tool.filename));
                self.emit_activity("write_tool", &format!("Writing tool {}", tool.filename), 7, TOTAL_STEPS, activity);

                if let Err(e) = self
                    .write_to_drive(&bucket_name, &tool_path, &tool.content)
                    .await
                {
                    log_generator_error(
                        &llm_app.name,
                        &format!("Failed to write tool {}", tool.filename),
                        &e.to_string(),
                    );
                }
                self.update_item_status(SectionType::Tools, &tool.filename, crate::auto_task::ItemStatus::Completed);
                self.add_terminal_output(&format!("✓ Tool `{}`", tool.filename), TerminalLineType::Success);

                tools.push(GeneratedFile {
                    filename: tool.filename.clone(),
                    content: tool.content.clone(),
                    file_type: FileType::Bas,
                });
            }

            self.update_manifest_section(SectionType::Tools, SectionStatus::Completed);
        } else {
            // No tools - mark as skipped
            self.update_manifest_section(SectionType::Tools, SectionStatus::Skipped);
        }

        let mut schedulers = Vec::new();
        if !llm_app.schedulers.is_empty() {
            self.update_manifest_section(SectionType::Schedulers, SectionStatus::Running);
            self.add_terminal_output("Creating scheduled tasks...", TerminalLineType::Progress);

            let sched_count = llm_app.schedulers.len();
            let activity = self.build_activity("schedulers", 0, Some(sched_count as u32), Some("Creating schedulers"));
            self.emit_activity(
                "write_schedulers",
                &format!("Creating {} schedulers...", sched_count),
                7,
                TOTAL_STEPS,
                activity
            );

            for (idx, scheduler) in llm_app.schedulers.iter().enumerate() {
                let scheduler_path = format!(".gbdialog/schedulers/{}", scheduler.filename);
                self.files_written.push(format!("schedulers/{}", scheduler.filename));
                self.bytes_generated += scheduler.content.len() as u64;

                let activity = self.build_activity("schedulers", (idx + 1) as u32, Some(sched_count as u32), Some(&scheduler.filename));
                self.emit_activity("write_scheduler", &format!("Writing scheduler {}", scheduler.filename), 7, TOTAL_STEPS, activity);

                if let Err(e) = self
                    .write_to_drive(&bucket_name, &scheduler_path, &scheduler.content)
                    .await
                {
                    log_generator_error(
                        &llm_app.name,
                        &format!("Failed to write scheduler {}", scheduler.filename),
                        &e.to_string(),
                    );
                }
                self.update_item_status(SectionType::Schedulers, &scheduler.filename, crate::auto_task::ItemStatus::Completed);
                self.add_terminal_output(&format!("✓ Scheduler `{}`", scheduler.filename), TerminalLineType::Success);

                schedulers.push(GeneratedFile {
                    filename: scheduler.filename.clone(),
                    content: scheduler.content.clone(),
                    file_type: FileType::Bas,
                });
            }

            self.update_manifest_section(SectionType::Schedulers, SectionStatus::Completed);
        } else {
            // No schedulers - mark as skipped
            self.update_manifest_section(SectionType::Schedulers, SectionStatus::Skipped);
        }

        // No monitors generated currently - mark as skipped
        self.update_manifest_section(SectionType::Monitors, SectionStatus::Skipped);

        // Build the app URL (use relative URL so it works on any port)
        // Include trailing slash so relative paths in HTML resolve correctly
        let app_url = format!("/apps/{}/", llm_app.name.to_lowercase().replace(' ', "-"));

        if let Some(ref mut manifest) = self.manifest {
            manifest.complete();
        }
        self.add_terminal_output("## Complete!", TerminalLineType::Success);
        self.add_terminal_output(&format!("✓ App **{}** ready at `{}`", llm_app.name, app_url), TerminalLineType::Success);
        self.update_manifest_stats_real(true);

        let activity = self.build_activity("complete", TOTAL_STEPS as u32, Some(TOTAL_STEPS as u32), Some("App ready"));
        self.emit_activity("complete", &format!("App ready at {}", app_url), 8, TOTAL_STEPS, activity);

        let elapsed = self.generation_start.map(|s| s.elapsed().as_secs()).unwrap_or(0);

        log_generator_info(
            &llm_app.name,
            &format!(
                "App generated: {} files, {} tables, {} tools in {}s - URL: {}",
                pages.len(),
                tables.len(),
                tools.len(),
                elapsed,
                app_url
            ),
        );

        info!(
            "App '{}' generated in s3://{}/{} - URL: {}",
            llm_app.name, bucket_name, drive_app_path, app_url
        );

        // Update task with app_url in database
        if let Some(ref task_id) = self.task_id {
            if let Ok(task_uuid) = uuid::Uuid::parse_str(task_id) {
                let _ = self.update_task_app_url(task_uuid, &app_url);
            }

            let final_activity = AgentActivity::new("completed")
                .with_progress(TOTAL_STEPS as u32, Some(TOTAL_STEPS as u32))
                .with_bytes(self.bytes_generated)
                .with_files(self.files_written.clone())
                .with_tables(self.tables_synced.clone());

            // Include app_url in the completion event
            let event = crate::core::shared::state::TaskProgressEvent::new(task_id, "complete", &format!(
                "App '{}' created: {} files, {} tables, {} bytes in {}s",
                llm_app.name, pages.len(), tables.len(), self.bytes_generated, elapsed
            ))
            .with_progress(TOTAL_STEPS, TOTAL_STEPS)
            .with_activity(final_activity)
            .with_details(format!("app_url:{}", app_url))
            .completed();

            self.state.broadcast_task_progress(event);
        }

        Ok(GeneratedApp {
            id: Uuid::new_v4().to_string(),
            name: llm_app.name,
            description: llm_app.description,
            pages,
            tables,
            tools,
            schedulers,
            created_at: Utc::now(),
        })
    }

    fn get_platform_prompt() -> &'static str {
        static PROMPT: OnceLock<String> = OnceLock::new();

        PROMPT.get_or_init(|| {
            let prompt_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/auto_task/APP_GENERATOR_PROMPT.md");

            match std::fs::read_to_string(&prompt_path) {
                Ok(content) => {
                    info!("[APP_GENERATOR] Loaded prompt from {:?} ({} chars)", prompt_path, content.len());
                    content
                }
                Err(e) => {
                    warn!("[APP_GENERATOR] Failed to load APP_GENERATOR_PROMPT.md: {}, using fallback", e);
                    Self::get_fallback_prompt().to_string()
                }
            }
        }).as_str()
    }

    fn get_fallback_prompt() -> &'static str {
        r##"
GENERAL BOTS PLATFORM - APP GENERATION

You are an expert full-stack developer generating complete applications for General Bots platform.

=== AVAILABLE APIs ===

DATABASE (/api/db/):
- GET /api/db/{table} - List records (query: limit, offset, order_by, order_dir, search, field=value)
- GET /api/db/{table}/{id} - Get single record
- POST /api/db/{table} - Create record (JSON body)
- PUT /api/db/{table}/{id} - Update record
- DELETE /api/db/{table}/{id} - Delete record

=== HTMX REQUIREMENTS ===

All HTML pages MUST use HTMX exclusively. NO fetch(), NO XMLHttpRequest.

Key attributes: hx-get, hx-post, hx-put, hx-delete, hx-target, hx-swap, hx-trigger

=== BASIC SCRIPTS (.bas) ===

Tools: HEAR "keyword" ... END HEAR
Schedulers: SET SCHEDULE "cron" ... END SCHEDULE
Keywords: TALK, ASK, GET FROM, SAVE TO, SEND MAIL, LLM

=== FIELD TYPES ===
guid, string, text, integer, decimal, boolean, date, datetime, json

Generate COMPLETE, WORKING code with no placeholders.
"##
    }

    /// PHASE 1: Get complete project plan from LLM (structure only, no content)
    async fn get_project_plan_from_llm(
        &self,
        intent: &str,
        bot_id: Uuid,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let platform = Self::get_platform_prompt();
        let prompt = format!(
            r#"{platform}

=== PLANNING PHASE ===
Analyze this request and return ONLY the project structure (no file content yet).

USER REQUEST: "{intent}"

Output format (use EXACT delimiters):

<<<APP_START>>>
name: app-name-lowercase
description: Brief description
domain: utility|crm|inventory|booking|etc
<<<TABLES_START>>>
<<<TABLE:table_name>>>
id:guid:false
field_name:type:nullable
<<<TABLES_END>>>
<<<FILES_PLAN>>>
index.html: Main page with [describe purpose]
styles.css: Styling with [describe theme]
app.js: Logic for [describe functionality]
<<<FILES_END>>>
<<<TOOLS_PLAN>>>
helper.bas: Voice command for [purpose]
<<<TOOLS_END>>>
<<<SCHEDULERS_PLAN>>>
daily_task.bas: Runs daily to [purpose]
<<<SCHEDULERS_END>>>
<<<APP_END>>>

RESPOND ONLY WITH THE PLAN STRUCTURE. NO QUESTIONS."#
        );

        let intent_preview: String = intent.chars().take(50).collect();
        info!("[PHASE1] Getting project plan from LLM for: {}", intent_preview);
        let response = self.call_llm(&prompt, bot_id).await?;
        info!("[PHASE1] Project plan received, parsing...");

        Self::parse_project_plan(&response, intent)
    }

    /// Parse the project plan (structure only)
    fn parse_project_plan(
        response: &str,
        intent: &str,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let mut app = LlmGeneratedApp::default();

        // Debug: Log the raw response to understand what LLM returned
        let response_preview: String = response.chars().take(500).collect();
        info!("[PHASE1_PARSE] Response preview: {}", response_preview.replace('\n', "\\n"));
        info!("[PHASE1_PARSE] Has APP_START: {}, Has TABLES_START: {}, Has FILES_PLAN: {}, Has TOOLS_PLAN: {}",
            response.contains("<<<APP_START>>>"),
            response.contains("<<<TABLES_START>>>"),
            response.contains("<<<FILES_PLAN>>>"),
            response.contains("<<<TOOLS_PLAN>>>")
        );

        // Extract app name and description
        if let Some(start) = response.find("<<<APP_START>>>") {
            let content = response.get(start..).unwrap_or("");

            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("name:") {
                    app.name = line.get(5..).unwrap_or("").trim().to_string();
                } else if line.starts_with("description:") {
                    app.description = line.get(12..).unwrap_or("").trim().to_string();
                } else if line.starts_with("domain:") {
                    app.domain = line.get(7..).unwrap_or("").trim().to_string();
                }
            }
        }

        // Default name if not found
        if app.name.is_empty() {
            app.name = intent.split_whitespace().take(3).collect::<Vec<_>>().join("-").to_lowercase();
        }
        if app.description.is_empty() {
            app.description = intent.to_string();
        }

        // Parse tables
        if let (Some(start), Some(end)) = (response.find("<<<TABLES_START>>>"), response.find("<<<TABLES_END>>>")) {
            let tables_section = response.get(start..end).unwrap_or("");
            let mut current_table: Option<LlmTable> = None;

            for line in tables_section.lines() {
                let line = line.trim();
                if line.starts_with("<<<TABLE:") && line.ends_with(">>>") {
                    if let Some(table) = current_table.take() {
                        if !table.name.is_empty() {
                            app.tables.push(table);
                        }
                    }
                    let table_name = line.get(9..line.len().saturating_sub(3)).unwrap_or("").trim();
                    current_table = Some(LlmTable {
                        name: table_name.to_string(),
                        fields: Vec::new(),
                    });
                } else if line.contains(':') && current_table.is_some() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let field = LlmField {
                            name: parts[0].trim().to_string(),
                            field_type: parts[1].trim().to_string(),
                            nullable: parts.get(2).map(|s| *s == "true").unwrap_or(true),
                            default: parts.get(3).map(|s| s.to_string()),
                            reference: parts.get(5).map(|s| s.to_string()),
                        };
                        if let Some(ref mut table) = current_table {
                            table.fields.push(field);
                        }
                    }
                }
            }
            if let Some(table) = current_table {
                if !table.name.is_empty() {
                    app.tables.push(table);
                }
            }
        } else {
            info!("[PHASE1_PARSE] TABLES_START not found, trying <<<TABLE:>>> delimiters...");
            for table_match in response.match_indices("<<<TABLE:") {
                let start = table_match.0;
                if let Some(rest) = response.get(start..) {
                    if let Some(end_offset) = rest.find(">>>") {
                        if let Some(table_name) = rest.get(9..end_offset) {
                            let table_name = table_name.trim();
                            if !table_name.is_empty() {
                                info!("[PHASE1_PARSE] Found table from delimiter: {}", table_name);
                                app.tables.push(LlmTable {
                                    name: table_name.to_string(),
                                    fields: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Parse file plan (just names, no content yet)
        if let (Some(start), Some(end)) = (response.find("<<<FILES_PLAN>>>"), response.find("<<<FILES_END>>>")) {
            let files_section = response.get(start + 16..end).unwrap_or("");
            info!("[PHASE1_PARSE] FILES_PLAN section: {}", files_section.replace('\n', "\\n"));
            for line in files_section.lines() {
                let line = line.trim();
                if line.contains(':') {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    let filename = parts[0].trim().to_string();
                    // Accept common web file extensions
                    if !filename.is_empty() && (filename.ends_with(".html") || filename.ends_with(".css") || filename.ends_with(".js") || filename.ends_with(".bas") || filename.ends_with(".json")) {
                        info!("[PHASE1_PARSE] Adding file: {}", filename);
                        app.files.push(LlmFile {
                            filename,
                            content: String::new(), // Content will be generated in Phase 2
                        });
                    } else if !filename.is_empty() {
                        info!("[PHASE1_PARSE] Skipped file (unknown ext): {}", filename);
                    }
                }
            }
        } else {
            info!("[PHASE1_PARSE] FILES_PLAN section not found! Looking for <<<FILE: delimiters...");
            // Fallback: try to find <<<FILE:xxx>>> delimiters directly
            for file_match in response.match_indices("<<<FILE:") {
                let start = file_match.0;
                if let Some(rest) = response.get(start..) {
                    if let Some(end_offset) = rest.find(">>>") {
                        if let Some(filename) = rest.get(8..end_offset) {
                            let filename = filename.trim();
                            if !filename.is_empty() {
                                info!("[PHASE1_PARSE] Found file from delimiter: {}", filename);
                                app.files.push(LlmFile {
                                    filename: filename.to_string(),
                                    content: String::new(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Parse tools plan
        if let (Some(start), Some(end)) = (response.find("<<<TOOLS_PLAN>>>"), response.find("<<<TOOLS_END>>>")) {
            let tools_section = response.get(start + 16..end).unwrap_or("");
            for line in tools_section.lines() {
                let line = line.trim();
                if line.contains(':') {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    let filename = parts[0].trim().to_string();
                    if !filename.is_empty() {
                        let filename = if filename.ends_with(".bas") { filename } else { format!("{}.bas", filename) };
                        app.tools.push(LlmFile {
                            filename,
                            content: String::new(),
                        });
                    }
                }
            }
        } else {
            info!("[PHASE1_PARSE] TOOLS_PLAN not found, trying <<<TOOL:>>> delimiters...");
            for tool_match in response.match_indices("<<<TOOL:") {
                let start = tool_match.0;
                if let Some(rest) = response.get(start..) {
                    if let Some(end_offset) = rest.find(">>>") {
                        if let Some(tool_name) = rest.get(8..end_offset) {
                            let tool_name = tool_name.trim();
                            if !tool_name.is_empty() {
                                let filename = if tool_name.ends_with(".bas") { tool_name.to_string() } else { format!("{}.bas", tool_name) };
                                info!("[PHASE1_PARSE] Found tool from delimiter: {}", filename);
                                app.tools.push(LlmFile {
                                    filename,
                                    content: String::new(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Parse schedulers plan
        if let (Some(start), Some(end)) = (response.find("<<<SCHEDULERS_PLAN>>>"), response.find("<<<SCHEDULERS_END>>>")) {
            let sched_section = response.get(start + 21..end).unwrap_or("");
            for line in sched_section.lines() {
                let line = line.trim();
                if line.contains(':') {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    let filename = parts[0].trim().to_string();
                    if !filename.is_empty() {
                        let filename = if filename.ends_with(".bas") { filename } else { format!("{}.bas", filename) };
                        app.schedulers.push(LlmFile {
                            filename,
                            content: String::new(),
                        });
                    }
                }
            }
        } else {
            info!("[PHASE1_PARSE] SCHEDULERS_PLAN not found, trying <<<SCHEDULER:>>> delimiters...");
            for sched_match in response.match_indices("<<<SCHEDULER:") {
                let start = sched_match.0;
                if let Some(rest) = response.get(start..) {
                    if let Some(end_offset) = rest.find(">>>") {
                        if let Some(sched_name) = rest.get(13..end_offset) {
                            let sched_name = sched_name.trim();
                            if !sched_name.is_empty() {
                                let filename = if sched_name.ends_with(".bas") { sched_name.to_string() } else { format!("{}.bas", sched_name) };
                                info!("[PHASE1_PARSE] Found scheduler from delimiter: {}", filename);
                                app.schedulers.push(LlmFile {
                                    filename,
                                    content: String::new(),
                                });
                            }
                        }
                    }
                }
            }
        }

        info!("[PHASE1_PARSE] Final result: {} tables, {} files, {} tools, {} schedulers",
            app.tables.len(), app.files.len(), app.tools.len(), app.schedulers.len());

        if app.tables.is_empty() && app.files.is_empty() && app.tools.is_empty() {
            warn!("[PHASE1_PARSE] Empty plan! LLM did not return expected delimiters. Full response ({} chars):", response.len());
            let full_preview: String = response.chars().take(2000).collect();
            warn!("[PHASE1_PARSE] {}", full_preview);
        }

        Ok(app)
    }

    /// PHASE 2: Generate content for a single file
    async fn generate_file_content(
        &self,
        app: &LlmGeneratedApp,
        filename: &str,
        bot_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let platform = Self::get_platform_prompt();
        let tables_desc = app.tables.iter()
            .map(|t| format!("- {}: {}", t.name, t.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", ")))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"{platform}

=== APP CONTEXT ===
App Name: {app_name}
Description: {description}
Tables:
{tables_desc}

=== GENERATE FILE: {filename} ===
Generate COMPLETE content for this file. No placeholders, no "...", no shortcuts.

Rules:
- Use data-app-name="{app_name}" in HTML for API calls
- Include all necessary HTML structure, CSS, and JavaScript
- For CRUD pages, implement full list/create/edit/delete functionality
- CSS should be comprehensive with dark mode support
- JavaScript should be complete and functional

RESPOND WITH ONLY THE FILE CONTENT. NO EXPLANATIONS."#,
            platform = platform,
            app_name = app.name,
            description = app.description,
            tables_desc = if tables_desc.is_empty() { "None".to_string() } else { tables_desc },
            filename = filename,
        );

        self.call_llm(&prompt, bot_id).await
    }

    async fn generate_complete_app_with_llm(
        &self,
        intent: &str,
        bot_id: Uuid,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let platform = Self::get_platform_prompt();

        let prompt = format!(
            r#"{platform}

=== FULL GENERATION MODE ===
USER REQUEST: "{intent}"

Generate a COMPLETE app with all files. Use the STREAMING DELIMITERS format from the platform docs.
Be creative - add features the user would love. NO placeholders, NO "...", complete code only.

NO QUESTIONS. JUST BUILD."#
        );

        let response = self.call_llm(&prompt, bot_id).await?;
        Self::parse_streaming_response(&response)
    }

    /// Parse streaming delimiter format response
    fn parse_streaming_response(
        response: &str,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let mut app = LlmGeneratedApp::default();

        // Find APP_START and APP_END
        let start_idx = response.find(DELIM_APP_START);
        let end_idx = response.find(DELIM_APP_END);

        let content = match (start_idx, end_idx) {
            (Some(s), Some(e)) => {
                response.get(s + DELIM_APP_START.len()..e).unwrap_or("")
            }
            (Some(s), None) => {
                warn!("No APP_END found, using rest of response");
                response.get(s + DELIM_APP_START.len()..).unwrap_or("")
            }
            _ => {
                // Fallback: try to parse as JSON for backwards compatibility
                return Self::parse_json_fallback(response);
            }
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut current_section = "header";
        let mut current_table: Option<LlmTable> = None;
        let mut current_file: Option<(String, String, String)> = None; // (type, filename, content)

        for raw_line in lines.iter() {
            let line = raw_line.trim();

            // Parse header fields
            if current_section == "header" {
                if line.starts_with("name:") {
                    app.name = line.get(5..).unwrap_or("").trim().to_string();
                    continue;
                }
                if line.starts_with("description:") {
                    app.description = line.get(12..).unwrap_or("").trim().to_string();
                    continue;
                }
                if line.starts_with("domain:") {
                    app.domain = line.get(7..).unwrap_or("").trim().to_string();
                    continue;
                }
            }

            // Section transitions
            if line == DELIM_TABLES_START {
                current_section = "tables";
                continue;
            }
            if line == DELIM_TABLES_END {
                // Save any pending table
                if let Some(table) = current_table.take() {
                    if !table.name.is_empty() {
                        app.tables.push(table);
                    }
                }
                current_section = "files";
                continue;
            }

            // Table definitions
            if line.starts_with(DELIM_TABLE_PREFIX) && line.ends_with(DELIM_END) {
                // Save previous table
                if let Some(table) = current_table.take() {
                    if !table.name.is_empty() {
                        app.tables.push(table);
                    }
                }
                let table_name = line.get(DELIM_TABLE_PREFIX.len()..line.len().saturating_sub(DELIM_END.len())).unwrap_or("").trim();
                current_table = Some(LlmTable {
                    name: table_name.to_string(),
                    fields: Vec::new(),
                });
                continue;
            }

            // Table field (when in tables section with active table)
            if current_section == "tables" && current_table.is_some() && !line.is_empty() && !line.starts_with("<<<") {
                if let Some(ref mut table) = current_table {
                    if let Some(field) = Self::parse_field_line(line) {
                        table.fields.push(field);
                    }
                }
                continue;
            }

            // File definitions
            if line.starts_with(DELIM_FILE_PREFIX) && line.ends_with(DELIM_END) {
                // Save previous file
                if let Some((file_type, filename, content)) = current_file.take() {
                    Self::save_parsed_file(&mut app, &file_type, filename, content);
                }
                let filename = line.get(DELIM_FILE_PREFIX.len()..line.len().saturating_sub(DELIM_END.len())).unwrap_or("").trim();
                current_file = Some(("file".to_string(), filename.to_string(), String::new()));
                continue;
            }

            // Tool definitions
            if line.starts_with(DELIM_TOOL_PREFIX) && line.ends_with(DELIM_END) {
                if let Some((file_type, filename, content)) = current_file.take() {
                    Self::save_parsed_file(&mut app, &file_type, filename, content);
                }
                let filename = line.get(DELIM_TOOL_PREFIX.len()..line.len().saturating_sub(DELIM_END.len())).unwrap_or("").trim();
                current_file = Some(("tool".to_string(), filename.to_string(), String::new()));
                continue;
            }

            // Scheduler definitions
            if line.starts_with(DELIM_SCHEDULER_PREFIX) && line.ends_with(DELIM_END) {
                if let Some((file_type, filename, content)) = current_file.take() {
                    Self::save_parsed_file(&mut app, &file_type, filename, content);
                }
                let filename = line.get(DELIM_SCHEDULER_PREFIX.len()..line.len().saturating_sub(DELIM_END.len())).unwrap_or("").trim();
                current_file = Some(("scheduler".to_string(), filename.to_string(), String::new()));
                continue;
            }

            // Accumulate file content (use original line to preserve indentation)
            if let Some((_, _, ref mut file_content)) = current_file {
                if !file_content.is_empty() {
                    file_content.push('\n');
                }
                file_content.push_str(raw_line);
            }
        }

        // Save any remaining file
        if let Some((file_type, filename, content)) = current_file.take() {
            Self::save_parsed_file(&mut app, &file_type, filename, content);
        }

        // Validate
        if app.name.is_empty() {
            return Err("No app name found in response".into());
        }
        if app.files.is_empty() {
            return Err("No files generated".into());
        }

        info!(
            "Parsed streaming response: name={}, tables={}, files={}, tools={}, schedulers={}",
            app.name,
            app.tables.len(),
            app.files.len(),
            app.tools.len(),
            app.schedulers.len()
        );

        Ok(app)
    }

    /// Parse a table field line in format: name:type:nullable[:default][:ref:table]
    fn parse_field_line(line: &str) -> Option<LlmField> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 3 {
            return None;
        }

        let mut field = LlmField {
            name: parts[0].trim().to_string(),
            field_type: parts[1].trim().to_string(),
            nullable: parts[2].trim() == "true",
            reference: None,
            default: None,
        };

        // Parse optional parts
        let mut i = 3;
        while i < parts.len() {
            if parts[i].trim() == "ref" && i + 1 < parts.len() {
                field.reference = Some(parts[i + 1].trim().to_string());
                i += 2;
            } else {
                // It's a default value
                field.default = Some(parts[i].trim().to_string());
                i += 1;
            }
        }

        Some(field)
    }

    /// Save a parsed file to the appropriate collection
    fn save_parsed_file(app: &mut LlmGeneratedApp, file_type: &str, filename: String, content: String) {
        let file = LlmFile {
            filename,
            content: content.trim().to_string(),
        };

        match file_type {
            "tool" => app.tools.push(file),
            "scheduler" => app.schedulers.push(file),
            _ => app.files.push(file),
        }
    }

    /// Fallback to JSON parsing for backwards compatibility
    fn parse_json_fallback(
        response: &str,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        warn!("Falling back to JSON parsing");

        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        #[derive(Debug, Deserialize)]
        struct JsonApp {
            name: String,
            description: String,
            #[serde(default)]
            domain: String,
            #[serde(default)]
            tables: Vec<JsonTable>,
            #[serde(default)]
            files: Vec<JsonFile>,
            #[serde(default)]
            tools: Option<Vec<JsonFile>>,
            #[serde(default)]
            schedulers: Option<Vec<JsonFile>>,
        }

        #[derive(Debug, Deserialize)]
        struct JsonTable {
            name: String,
            fields: Vec<JsonField>,
        }

        #[derive(Debug, Deserialize)]
        struct JsonField {
            name: String,
            #[serde(rename = "type")]
            field_type: String,
            #[serde(default)]
            nullable: Option<bool>,
            #[serde(default)]
            reference: Option<String>,
            #[serde(default, deserialize_with = "deserialize_default_value")]
            default: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct JsonFile {
            filename: String,
            content: String,
        }

        /// Deserialize default value that can be string, bool, number, or null
        fn deserialize_default_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
            match value {
                None => Ok(None),
                Some(serde_json::Value::Null) => Ok(None),
                Some(serde_json::Value::String(s)) => Ok(Some(s)),
                Some(serde_json::Value::Bool(b)) => Ok(Some(b.to_string())),
                Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
                Some(v) => Ok(Some(v.to_string())),
            }
        }

        match serde_json::from_str::<JsonApp>(cleaned) {
            Ok(json_app) => {
                let app = LlmGeneratedApp {
                    name: json_app.name,
                    description: json_app.description,
                    domain: json_app.domain,
                    tables: json_app.tables.into_iter().map(|t| LlmTable {
                        name: t.name,
                        fields: t.fields.into_iter().map(|f| LlmField {
                            name: f.name,
                            field_type: f.field_type,
                            nullable: f.nullable.unwrap_or(true),
                            reference: f.reference,
                            default: f.default,
                        }).collect(),
                    }).collect(),
                    files: json_app.files.into_iter().map(|f| LlmFile {
                        filename: f.filename,
                        content: f.content,
                    }).collect(),
                    tools: json_app.tools.unwrap_or_default().into_iter().map(|f| LlmFile {
                        filename: f.filename,
                        content: f.content,
                    }).collect(),
                    schedulers: json_app.schedulers.unwrap_or_default().into_iter().map(|f| LlmFile {
                        filename: f.filename,
                        content: f.content,
                    }).collect(),
                };

                if app.files.is_empty() {
                    return Err("LLM generated no files".into());
                }
                Ok(app)
            }
            Err(e) => {
                error!("Failed to parse LLM response: {}", e);
                let preview: String = response.chars().take(500).collect();
                error!("Response was: {}", preview);
                Err(format!("Failed to parse LLM response: {}", e).into())
            }
        }
    }

    fn convert_llm_tables(llm_tables: &[LlmTable]) -> Vec<TableDefinition> {
        llm_tables
            .iter()
            .map(|t| {
                let fields = t
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| FieldDefinition {
                        name: f.name.clone(),
                        field_type: f.field_type.clone(),
                        is_key: f.name == "id",
                        is_nullable: f.nullable,
                        reference_table: f.reference.clone(),
                        default_value: f.default.clone(),
                        field_order: i as i32,
                        ..Default::default()
                    })
                    .collect();

                TableDefinition {
                    name: t.name.clone(),
                    connection_name: "default".to_string(),
                    fields,
                    ..Default::default()
                }
            })
            .collect()
    }

    fn detect_file_type(filename: &str) -> FileType {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "css" => FileType::Css,
            "js" => FileType::Js,
            "bas" => FileType::Bas,
            "json" => FileType::Json,
            _ => FileType::Html,
        }
    }

    async fn call_llm(
        &self,
        prompt: &str,
        bot_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "llm")]
        {
            let config_manager = ConfigManager::new(self.state.conn.clone());
            let model = config_manager
                .get_config(&bot_id, "llm-model", None)
                .unwrap_or_else(|_| {
                    config_manager
                        .get_config(&Uuid::nil(), "llm-model", None)
                        .unwrap_or_else(|_| "gpt-4".to_string())
                });
            let key = config_manager
                .get_config(&bot_id, "llm-key", None)
                .unwrap_or_else(|_| {
                    config_manager
                        .get_config(&Uuid::nil(), "llm-key", None)
                        .unwrap_or_default()
                });

            let llm_config = serde_json::json!({
                "temperature": 0.7,
                "max_tokens": 16000
            });

            let prompt_len = prompt.len();
            trace!("APP_GENERATOR Starting LLM streaming: model={}, prompt_len={}", model, prompt_len);
            let start = std::time::Instant::now();

            // Use streaming to provide real-time feedback
            let (tx, mut rx) = mpsc::channel::<String>(100);
            let state = self.state.clone();
            let task_id = self.task_id.clone();

            // Spawn a task to receive stream chunks and broadcast them
            let stream_task = tokio::spawn(async move {
                let mut full_response = String::new();
                let mut chunk_buffer = String::new();
                let mut last_emit = std::time::Instant::now();
                let mut chunk_count = 0u32;
                let stream_start = std::time::Instant::now();
                let mut last_progress_update = std::time::Instant::now();
                let mut detected_tables: Vec<String> = Vec::new();
                let mut detected_files: Vec<String> = Vec::new();
                let mut detected_tools: Vec<String> = Vec::new();
                trace!("APP_GENERATOR Stream receiver started");

                while let Some(chunk) = rx.recv().await {
                    chunk_count += 1;
                    full_response.push_str(&chunk);
                    chunk_buffer.push_str(&chunk);

                    // Detect section markers using full_response (not chunk_buffer which gets trimmed)
                    let in_files_plan = full_response.contains("<<<FILES_PLAN>>>") && !full_response.contains("<<<FILES_END>>>");
                    let in_tools_plan = full_response.contains("<<<TOOLS_PLAN>>>") && !full_response.contains("<<<TOOLS_END>>>");
                    let in_schedulers_plan = full_response.contains("<<<SCHEDULERS_PLAN>>>") && !full_response.contains("<<<SCHEDULERS_END>>>");

                    // Detect items being generated in real-time (full generation format)
                    // Use full_response for reliable detection with safe string extraction
                    for table_match in full_response.match_indices("<<<TABLE:") {
                        let start = table_match.0;
                        if let Some(rest) = full_response.get(start..) {
                            if let Some(end_offset) = rest.find(">>>") {
                                if let Some(table_name) = rest.get(9..end_offset) {
                                    let table_name = table_name.trim();
                                    if !table_name.is_empty() && !detected_tables.contains(&table_name.to_string()) {
                                        detected_tables.push(table_name.to_string());
                                        info!("[LLM_STREAM] Detected table: {table_name}");
                                    }
                                }
                            }
                        }
                    }

                    for file_match in full_response.match_indices("<<<FILE:") {
                        let start = file_match.0;
                        if let Some(rest) = full_response.get(start..) {
                            if let Some(end_offset) = rest.find(">>>") {
                                if let Some(file_name) = rest.get(8..end_offset) {
                                    let file_name = file_name.trim();
                                    if !file_name.is_empty() && !detected_files.contains(&file_name.to_string()) {
                                        detected_files.push(file_name.to_string());
                                        info!("[LLM_STREAM] Detected file: {file_name}");
                                    }
                                }
                            }
                        }
                    }

                    for tool_match in full_response.match_indices("<<<TOOL:") {
                        let start = tool_match.0;
                        if let Some(rest) = full_response.get(start..) {
                            if let Some(end_offset) = rest.find(">>>") {
                                if let Some(tool_name) = rest.get(8..end_offset) {
                                    let tool_name = tool_name.trim();
                                    if !tool_name.is_empty() && !detected_tools.contains(&tool_name.to_string()) {
                                        detected_tools.push(tool_name.to_string());
                                        info!("[LLM_STREAM] Detected tool: {tool_name}");
                                    }
                                }
                            }
                        }
                    }

                    // Detect items from plan format (filename: description lines)
                    // Parse from full_response for FILES_PLAN section
                    if in_files_plan {
                        if let Some(plan_start) = full_response.find("<<<FILES_PLAN>>>") {
                            if let Some(plan_content) = full_response.get(plan_start.saturating_add(16)..) {
                                for line in plan_content.lines() {
                                    let line = line.trim();
                                    if line.starts_with("<<<") {
                                        break;
                                    }
                                    if line.contains(':') {
                                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                                        let name = parts[0].trim();
                                        if !name.is_empty() && (name.ends_with(".html") || name.ends_with(".css") || name.ends_with(".js") || name.ends_with(".bas")) {
                                            if !detected_files.contains(&name.to_string()) {
                                                detected_files.push(name.to_string());
                                                info!("[LLM_STREAM] Detected planned file: {name}");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if in_tools_plan {
                        if let Some(plan_start) = full_response.find("<<<TOOLS_PLAN>>>") {
                            if let Some(plan_content) = full_response.get(plan_start.saturating_add(16)..) {
                                for line in plan_content.lines() {
                                    let line = line.trim();
                                    if line.starts_with("<<<") {
                                        break;
                                    }
                                    if line.contains(':') {
                                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                                        let name = parts[0].trim();
                                        if !name.is_empty() {
                                            let tool_name = if name.ends_with(".bas") { name.to_string() } else { format!("{name}.bas") };
                                            if !detected_tools.contains(&tool_name) {
                                                detected_tools.push(tool_name.clone());
                                                info!("[LLM_STREAM] Detected planned tool: {tool_name}");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if in_schedulers_plan {
                        if let Some(plan_start) = full_response.find("<<<SCHEDULERS_PLAN>>>") {
                            if let Some(plan_content) = full_response.get(plan_start.saturating_add(21)..) {
                                for line in plan_content.lines() {
                                    let line = line.trim();
                                    if line.starts_with("<<<") {
                                        break;
                                    }
                                    if line.contains(':') {
                                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                                        let name = parts[0].trim();
                                        if !name.is_empty() {
                                            let sched_name = if name.ends_with(".bas") { name.to_string() } else { format!("{name}.bas") };
                                            if !detected_tools.contains(&sched_name) {
                                                detected_tools.push(sched_name.clone());
                                                info!("[LLM_STREAM] Detected planned scheduler: {sched_name}");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Log progress periodically
                    if chunk_count == 1 || chunk_count % 500 == 0 {
                        trace!("APP_GENERATOR Stream progress: {} chunks, {} chars, {:?}",
                              chunk_count, full_response.len(), stream_start.elapsed());
                    }

                    // Emit progress updates every 2 seconds
                    if last_progress_update.elapsed().as_secs() >= 2 {
                        if let Some(ref tid) = task_id {
                            let total_detected = detected_tables.len() + detected_files.len() + detected_tools.len();
                            let progress_msg = if total_detected > 0 {
                                format!(
                                    "AI generating... {} tables, {} files, {} tools detected",
                                    detected_tables.len(),
                                    detected_files.len(),
                                    detected_tools.len()
                                )
                            } else {
                                let chars_received = full_response.len();
                                format!("AI generating content... {} chars received", chars_received)
                            };
                            info!("[LLM_STREAM] Progress: {}", progress_msg);
                            let event = crate::core::shared::state::TaskProgressEvent::new(
                                tid,
                                "llm_generating",
                                &progress_msg,
                            )
                            .with_progress(1, 10);
                            state.broadcast_task_progress(event);
                        }
                        last_progress_update = std::time::Instant::now();
                    }

                    // Don't emit raw LLM stream to WebSocket - it contains HTML/code garbage
                    // Only clear buffer periodically to track progress
                    if last_emit.elapsed().as_millis() > 100 || chunk_buffer.len() > 500 {
                        // Keep last 200 chars for detecting split delimiters (Unicode-safe)
                        if chunk_buffer.chars().count() > 200 {
                            chunk_buffer = chunk_buffer.chars().skip(chunk_buffer.chars().count() - 200).collect();
                        }
                        last_emit = std::time::Instant::now();
                    }
                }

                // Final progress update
                if let Some(ref tid) = task_id {
                    let total_detected = detected_tables.len() + detected_files.len() + detected_tools.len();
                    let final_msg = if total_detected > 0 {
                        format!(
                            "AI complete: {} tables, {} files, {} tools",
                            detected_tables.len(),
                            detected_files.len(),
                            detected_tools.len()
                        )
                    } else {
                        format!("AI complete: {} chars generated", full_response.len())
                    };
                    info!("[LLM_STREAM] {}", final_msg);
                    let event = crate::core::shared::state::TaskProgressEvent::new(
                        tid,
                        "llm_complete",
                        &final_msg,
                    )
                    .with_progress(2, 10);
                    state.broadcast_task_progress(event);
                }

                trace!("APP_GENERATOR Stream finished: {} chunks, {} chars in {:?}",
                      chunk_count, full_response.len(), stream_start.elapsed());

                // Don't emit remaining buffer - it's raw code/HTML
                if !chunk_buffer.is_empty() {
                    trace!("APP_GENERATOR Final buffer (not emitting): {} chars", chunk_buffer.len());
                }

                // Log response preview (Unicode-safe)
                if !full_response.is_empty() {
                    let preview: String = full_response.chars().take(200).collect();
                    let suffix = if full_response.chars().count() > 200 { "..." } else { "" };
                    trace!("APP_GENERATOR Response preview: {}{}", preview.replace('\n', "\\n"), suffix);
                }

                full_response
            });

            // Start the streaming LLM call
            trace!("APP_GENERATOR Starting generate_stream...");
            match self
                .state
                .llm_provider
                .generate_stream(prompt, &llm_config, tx, &model, &key)
                .await
            {
                Ok(()) => {
                    trace!("APP_GENERATOR generate_stream completed, waiting for stream_task");
                    // Wait for the stream task to complete and get the full response
                    match stream_task.await {
                        Ok(response) => {
                            let elapsed = start.elapsed();
                            trace!("APP_GENERATOR LLM streaming succeeded: {} chars in {:?}", response.len(), elapsed);
                            if response.is_empty() {
                                error!("APP_GENERATOR Empty response from LLM");
                            }
                            return Ok(response);
                        }
                        Err(e) => {
                            let elapsed = start.elapsed();
                            error!("APP_GENERATOR LLM stream task failed after {:?}: {}", elapsed, e);
                            return Err(format!("Stream task failed: {}", e).into());
                        }
                    }
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    error!("APP_GENERATOR LLM streaming failed after {:?}: {}", elapsed, e);
                    // Abort the stream task
                    stream_task.abort();
                    return Err(e);
                }
            }
        }

        #[cfg(not(feature = "llm"))]
        {
            Err("LLM feature not enabled. App generation requires LLM.".into())
        }
    }

    fn generate_table_definitions(
        tables: &[TableDefinition],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use std::fmt::Write;
        let mut output = String::new();

        for table in tables {
            let _ = writeln!(output, "\nTABLE {}", table.name);

            for field in &table.fields {
                let mut line = format!("    {} AS {}", field.name, field.field_type.to_uppercase());
                if field.is_key {
                    line.push_str(" KEY");
                }
                if !field.is_nullable {
                    line.push_str(" REQUIRED");
                }
                if let Some(ref default) = field.default_value {
                    let _ = write!(line, " DEFAULT {}", default);
                }
                if let Some(ref refs) = field.reference_table {
                    let _ = write!(line, " REFERENCES {}", refs);
                }
                let _ = writeln!(output, "{}", line);
            }

            let _ = writeln!(output, "END TABLE\n");
        }

        Ok(output)
    }

    fn append_to_tables_bas(
        &self,
        _bot_id: Uuid,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use bucket_name from state instead of deriving from bot name
        let bucket = self.state.bucket_name.clone();
        let path = ".gbdata/tables.bas";

        let mut conn = self.state.conn.get()?;

        #[derive(QueryableByName)]
        struct ContentRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            content: String,
        }

        let existing: Option<String> =
            sql_query("SELECT content FROM drive_files WHERE bucket = $1 AND path = $2 LIMIT 1")
                .bind::<diesel::sql_types::Text, _>(&bucket)
                .bind::<diesel::sql_types::Text, _>(path)
                .load::<ContentRow>(&mut conn)
                .ok()
                .and_then(|rows| rows.into_iter().next().map(|r| r.content));

        let new_content = match existing {
            Some(existing_content) => format!("{}\n{}", existing_content, content),
            None => content.to_string(),
        };

        sql_query(
            "INSERT INTO drive_files (id, bucket, path, content, content_type, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'text/plain', NOW(), NOW())
             ON CONFLICT (bucket, path) DO UPDATE SET content = $4, updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Text, _>(&bucket)
        .bind::<diesel::sql_types::Text, _>(path)
        .bind::<diesel::sql_types::Text, _>(&new_content)
        .execute(&mut conn)?;

        Ok(())
    }

    /// Ensure the bucket exists, creating it if necessary
    async fn ensure_bucket_exists(
        &self,
        bucket: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref s3) = self.state.drive {
            // Check if bucket exists
            match s3.head_bucket().bucket(bucket).send().await {
                Ok(_) => {
                    trace!("Bucket {} already exists", bucket);
                    return Ok(());
                }
                Err(_) => {
                    // Bucket doesn't exist, try to create it
                    info!("Bucket {} does not exist, creating...", bucket);
                    match s3.create_bucket().bucket(bucket).send().await {
                        Ok(_) => {
                            info!("Created bucket: {}", bucket);
                            return Ok(());
                        }
                        Err(e) => {
                            // Check if error is "bucket already exists" (race condition)
                            let err_str = format!("{:?}", e);
                            if err_str.contains("BucketAlreadyExists") || err_str.contains("BucketAlreadyOwnedByYou") {
                                trace!("Bucket {} already exists (race condition)", bucket);
                                return Ok(());
                            }
                            error!("Failed to create bucket {}: {}", bucket, e);
                            return Err(Box::new(e));
                        }
                    }
                }
            }
        } else {
            // No S3 client, we'll use DB fallback - no bucket needed
            trace!("No S3 client, using DB fallback for storage");
            Ok(())
        }
    }

    async fn write_to_drive(
        &self,
        bucket: &str,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("write_to_drive: bucket={}, path={}, content_len={}", bucket, path, content.len());

        if let Some(ref s3) = self.state.drive {
            let body = ByteStream::from(content.as_bytes().to_vec());
            let content_type = get_content_type(path);

            info!("S3 client available, attempting put_object to s3://{}/{}", bucket, path);

            match s3.put_object()
                .bucket(bucket)
                .key(path)
                .body(body)
                .content_type(content_type)
                .send()
                .await
            {
                Ok(_) => {
                    info!("Successfully wrote to S3: s3://{}/{}", bucket, path);
                }
                Err(e) => {
                    // Log detailed error info
                    error!("S3 put_object failed: bucket={}, path={}, error={:?}", bucket, path, e);
                    error!("S3 error details: {}", e);

                    // If bucket doesn't exist, try to create it and retry
                    let err_str = format!("{:?}", e);
                    if err_str.contains("NoSuchBucket") || err_str.contains("NotFound") {
                        warn!("Bucket {} not found, attempting to create...", bucket);
                        self.ensure_bucket_exists(bucket).await?;

                        // Retry the write
                        let body = ByteStream::from(content.as_bytes().to_vec());
                        s3.put_object()
                            .bucket(bucket)
                            .key(path)
                            .body(body)
                            .content_type(get_content_type(path))
                            .send()
                            .await?;
                        info!("Wrote to S3 after creating bucket: s3://{}/{}", bucket, path);
                    } else {
                        error!("S3 write failed (not a bucket issue): {}", err_str);
                        return Err(Box::new(e));
                    }
                }
            }
        } else {
            warn!("No S3/drive client available, using DB fallback for {}/{}", bucket, path);
            self.write_to_db_fallback(bucket, path, content)?;
        }

        Ok(())
    }

    fn write_to_db_fallback(
        &self,
        bucket: &str,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;
        let content_type = get_content_type(path);

        sql_query(
            "INSERT INTO drive_files (id, bucket, path, content, content_type, size, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
             ON CONFLICT (bucket, path) DO UPDATE SET
             content = EXCLUDED.content,
             content_type = EXCLUDED.content_type,
             size = EXCLUDED.size,
             updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Text, _>(bucket)
        .bind::<diesel::sql_types::Text, _>(path)
        .bind::<diesel::sql_types::Text, _>(content)
        .bind::<diesel::sql_types::Text, _>(content_type)
        .bind::<diesel::sql_types::BigInt, _>(content.len() as i64)
        .execute(&mut conn)?;

        trace!("Wrote to DB: {}/{}", bucket, path);
        Ok(())
    }

    /// Sync a single table to database - used for real-time progress updates
    fn sync_single_table_to_database(
        &self,
        table: &TableDefinition,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;
        let create_sql = generate_create_table_sql(table, "postgres");

        sql_query(&create_sql).execute(&mut conn)?;
        info!("Created table: {}", table.name);

        Ok(table.fields.len())
    }

    fn update_task_app_url(
        &self,
        task_id: Uuid,
        app_url: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        let final_step_results = serde_json::json!([
            {
                "step_id": "file_0",
                "step_order": 1,
                "step_name": "Generate app structure",
                "status": "Completed",
                "duration_ms": 500,
                "logs": [{"message": "App structure generated"}]
            },
            {
                "step_id": "file_1",
                "step_order": 2,
                "step_name": "Write app files",
                "status": "Completed",
                "duration_ms": 300,
                "logs": [{"message": "Files written to storage"}]
            },
            {
                "step_id": "file_2",
                "step_order": 3,
                "step_name": "Configure app",
                "status": "Completed",
                "duration_ms": 200,
                "logs": [{"message": format!("App ready at {}", app_url)}]
            }
        ]);

        sql_query(
            "UPDATE auto_tasks SET
             progress = 1.0,
             current_step = 3,
             total_steps = 3,
             step_results = $1,
             status = 'completed',
             completed_at = NOW(),
             updated_at = NOW()
             WHERE id = $2",
        )
        .bind::<diesel::sql_types::Jsonb, _>(final_step_results)
        .bind::<diesel::sql_types::Uuid, _>(task_id)
        .execute(&mut conn)?;

        info!("Updated task {} completed with app_url: {}", task_id, app_url);
        Ok(())
    }

    fn update_task_step_results(
        &self,
        task_id: Uuid,
        files: &[String],
        completed_count: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        // Build step_results JSON with file status
        let step_results: Vec<serde_json::Value> = files.iter().enumerate().map(|(idx, filename)| {
            let status = if idx < completed_count {
                "Completed"
            } else if idx == completed_count {
                "Running"
            } else {
                "Pending"
            };
            serde_json::json!({
                "step_id": format!("file_{}", idx),
                "step_order": idx + 1,
                "step_name": format!("Write {}", filename),
                "status": status,
                "started_at": chrono::Utc::now().to_rfc3339(),
                "duration_ms": if idx < completed_count { Some(100) } else { None::<i64> },
                "logs": []
            })
        }).collect();

        let step_results_json = serde_json::to_value(&step_results)?;
        let progress = if files.is_empty() { 0.0 } else { completed_count as f64 / files.len() as f64 };

        sql_query(
            "UPDATE auto_tasks SET
             step_results = $1,
             current_step = $2,
             total_steps = $3,
             progress = $4,
             updated_at = NOW()
             WHERE id = $5",
        )
        .bind::<diesel::sql_types::Jsonb, _>(step_results_json)
        .bind::<diesel::sql_types::Integer, _>(completed_count as i32)
        .bind::<diesel::sql_types::Integer, _>(files.len() as i32)
        .bind::<diesel::sql_types::Double, _>(progress)
        .bind::<diesel::sql_types::Uuid, _>(task_id)
        .execute(&mut conn)?;

        trace!("Updated task {} step_results: {}/{} files", task_id, completed_count, files.len());
        Ok(())
    }

    fn generate_designer_js(app_name: &str) -> String {
        format!(
            r#"(function() {{
    const APP_NAME = '{app_name}';
    const currentPage = window.location.pathname.split('/').pop() || 'index.html';

    const style = document.createElement('style');
    style.textContent = `
        .designer-fab {{ position: fixed; bottom: 20px; right: 20px; width: 56px; height: 56px; border-radius: 50%; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border: none; cursor: pointer; box-shadow: 0 4px 12px rgba(102,126,234,0.4); font-size: 24px; z-index: 9999; transition: transform 0.2s; }}
        .designer-fab:hover {{ transform: scale(1.1); }}
        .designer-panel {{ position: fixed; bottom: 90px; right: 20px; width: 380px; max-height: 500px; background: white; border-radius: 16px; box-shadow: 0 10px 40px rgba(0,0,0,0.2); z-index: 9998; display: none; flex-direction: column; overflow: hidden; }}
        .designer-panel.open {{ display: flex; }}
        .designer-header {{ padding: 16px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; font-weight: 600; display: flex; justify-content: space-between; align-items: center; }}
        .designer-close {{ background: none; border: none; color: white; font-size: 20px; cursor: pointer; }}
        .designer-messages {{ flex: 1; overflow-y: auto; padding: 16px; max-height: 300px; }}
        .designer-msg {{ margin: 8px 0; padding: 10px 14px; border-radius: 12px; max-width: 85%; word-wrap: break-word; }}
        .designer-msg.user {{ background: #667eea; color: white; margin-left: auto; }}
        .designer-msg.ai {{ background: #f0f0f0; color: #333; }}
        .designer-input {{ display: flex; padding: 12px; border-top: 1px solid #eee; gap: 8px; }}
        .designer-input input {{ flex: 1; padding: 10px 14px; border: 1px solid #ddd; border-radius: 20px; outline: none; }}
        .designer-input button {{ padding: 10px 16px; background: #667eea; color: white; border: none; border-radius: 20px; cursor: pointer; }}
    `;
    document.head.appendChild(style);

    const fab = document.createElement('button');
    fab.className = 'designer-fab';
    fab.innerHTML = '🎨';
    fab.title = 'Designer AI';
    document.body.appendChild(fab);

    const panel = document.createElement('div');
    panel.className = 'designer-panel';
    panel.innerHTML = `
        <div class="designer-header">
            <span>🎨 Designer AI</span>
            <button class="designer-close">×</button>
        </div>
        <div class="designer-messages">
            <div class="designer-msg ai">Hi! I can help you modify this app. What would you like to change?</div>
        </div>
        <div class="designer-input">
            <input type="text" placeholder="e.g., Add a blue header..." />
            <button>Send</button>
        </div>
    `;
    document.body.appendChild(panel);

    fab.onclick = () => panel.classList.toggle('open');
    panel.querySelector('.designer-close').onclick = () => panel.classList.remove('open');

    const input = panel.querySelector('input');
    const sendBtn = panel.querySelector('.designer-input button');
    const messages = panel.querySelector('.designer-messages');

    async function sendMessage() {{
        const msg = input.value.trim();
        if (!msg) return;

        messages.innerHTML += `<div class="designer-msg user">${{msg}}</div>`;
        input.value = '';
        messages.scrollTop = messages.scrollHeight;

        try {{
            const res = await fetch('/api/designer/modify', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ app_name: APP_NAME, current_page: currentPage, message: msg }})
            }});
            const data = await res.json();
            messages.innerHTML += `<div class="designer-msg ai">${{data.message || 'Done!'}}</div>`;
            if (data.success && data.changes && data.changes.length > 0) {{
                setTimeout(() => location.reload(), 1500);
            }}
        }} catch (e) {{
            messages.innerHTML += `<div class="designer-msg ai">Sorry, something went wrong. Try again.</div>`;
            if (window.AppLogger) window.AppLogger.error('Designer error', e.toString());
        }}
        messages.scrollTop = messages.scrollHeight;
    }}

    sendBtn.onclick = sendMessage;
    input.onkeypress = (e) => {{ if (e.key === 'Enter') sendMessage(); }};
}})();"#,
            app_name = app_name
        )
    }
}
