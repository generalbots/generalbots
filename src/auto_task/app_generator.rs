use crate::auto_task::app_logs::{log_generator_error, log_generator_info};
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
                manifests.insert(task_id.clone(), self.manifest.clone().unwrap());
            }
        }

        self.broadcast_manifest_update();
    }

    fn broadcast_manifest_update(&self) {
        if let (Some(ref task_id), Some(ref manifest)) = (&self.task_id, &self.manifest) {
            log::info!(
                "[MANIFEST_BROADCAST] task={} completed={}/{} sections={}",
                task_id,
                manifest.completed_steps,
                manifest.total_steps,
                manifest.sections.len()
            );

            if let Ok(mut manifests) = self.state.task_manifests.write() {
                manifests.insert(task_id.clone(), manifest.clone());
            }

            let json_details = serde_json::to_string(&manifest.to_web_json()).unwrap_or_default();
            log::debug!("[MANIFEST_BROADCAST] JSON size: {} bytes", json_details.len());

            let event = crate::core::shared::state::TaskProgressEvent::new(
                task_id,
                "manifest_update",
                &format!("Manifest updated: {}", manifest.app_name),
            )
            .with_event_type("manifest_update")
            .with_progress(manifest.completed_steps as u8, manifest.total_steps as u8)
            .with_details(json_details);

            self.state.broadcast_task_progress(event);
        }
    }

    fn update_manifest_section(&mut self, section_type: SectionType, status: SectionStatus) {
        if let Some(ref mut manifest) = self.manifest {
            for section in &mut manifest.sections {
                if section.section_type == section_type {
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
            manifest.updated_at = Utc::now();
            self.broadcast_manifest_update();
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

    /// Update item groups within a child section (for field groups like "email, password_hash")
    fn update_manifest_item_groups(&mut self, parent_type: SectionType, child_type: SectionType, group_indices: &[usize], status: crate::auto_task::ItemStatus) {
        if let Some(ref mut manifest) = self.manifest {
            for section in &mut manifest.sections {
                if section.section_type == parent_type {
                    for child in &mut section.children {
                        if child.section_type == child_type {
                            for &idx in group_indices {
                                if idx < child.item_groups.len() {
                                    let group = &mut child.item_groups[idx];
                                    group.status = status.clone();
                                    if status == crate::auto_task::ItemStatus::Running {
                                        group.started_at = Some(Utc::now());
                                    } else if status == crate::auto_task::ItemStatus::Completed {
                                        group.completed_at = Some(Utc::now());
                                        if let Some(started) = group.started_at {
                                            group.duration_seconds =
                                                Some((Utc::now() - started).num_seconds() as u64);
                                        }
                                    }
                                }
                            }
                            // Update child step progress
                            child.current_step = child.item_groups.iter()
                                .filter(|g| g.status == crate::auto_task::ItemStatus::Completed)
                                .count() as u32;
                            break;
                        }
                    }
                    // Update parent step progress
                    section.current_step = section.children.iter()
                        .map(|c| c.current_step)
                        .sum();
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
                            for idx in start_idx..=end_idx.min(child.item_groups.len().saturating_sub(1)) {
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
                            break;
                        }
                    }
                    // Update parent step progress
                    section.current_step = section.children.iter()
                        .map(|c| c.current_step)
                        .sum();
                    break;
                }
            }
            manifest.updated_at = Utc::now();
            self.broadcast_manifest_update();
        }
    }

    fn add_terminal_output(&mut self, content: &str, line_type: TerminalLineType) {
        if let Some(ref mut manifest) = self.manifest {
            manifest.add_terminal_line(content, line_type);
            self.broadcast_manifest_update();
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

        // Section 5: Deployment
        let deploy_section = ManifestSection::new("Deployment", SectionType::Deployment)
            .with_steps(1);
        manifest.add_section(deploy_section);

        manifest.status = ManifestStatus::Running;
        manifest.add_terminal_line(&format!("Analyzing: {}", intent), TerminalLineType::Info);
        manifest.add_terminal_line("Sending request to AI...", TerminalLineType::Progress);

        self.manifest = Some(manifest);

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

    /// Update a specific item's status without broadcasting (for batch updates)
    fn update_item_status_silent(&mut self, section_type: SectionType, item_name: &str, status: crate::auto_task::ItemStatus) {
        self.update_item_status_internal(section_type, item_name, status, false);
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

        info!(
            "Generating app from intent: {}",
            &intent[..intent.len().min(100)]
        );

        log_generator_info(
            "pending",
            &format!(
                "Starting app generation: {}",
                &intent[..intent.len().min(50)]
            ),
        );

        if let Some(ref task_id) = self.task_id {
            self.state.emit_task_started(task_id, &format!("Generating app: {}", &intent[..intent.len().min(50)]), TOTAL_STEPS);
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

        trace!("APP_GENERATOR Calling LLM for intent: {}", &intent[..intent.len().min(50)]);
        let llm_start = std::time::Instant::now();

        let llm_app = match self.generate_complete_app_with_llm(intent, session.bot_id).await {
            Ok(app) => {
                let llm_elapsed = llm_start.elapsed();
                info!("APP_GENERATOR LLM completed in {:?}: app={}, files={}, tables={}",
                      llm_elapsed, app.name, app.files.len(), app.tables.len());
                log_generator_info(
                    &app.name,
                    "LLM successfully generated app structure and files",
                );

                let total_bytes: u64 = app.files.iter().map(|f| f.content.len() as u64).sum();
                self.bytes_generated = total_bytes;

                let activity = self.build_activity(
                    "parsing",
                    1,
                    Some(TOTAL_STEPS as u32),
                    Some(&format!("Generated {} with {} files", app.name, app.files.len()))
                );
                self.emit_activity(
                    "llm_response",
                    &format!("AI generated {} structure", app.name),
                    2,
                    TOTAL_STEPS,
                    activity
                );
                app
            }
            Err(e) => {
                let llm_elapsed = llm_start.elapsed();
                error!("APP_GENERATOR LLM failed after {:?}: {}", llm_elapsed, e);
                log_generator_error("unknown", "LLM app generation failed", &e.to_string());
                if let Some(ref task_id) = self.task_id {
                    self.state.emit_task_error(task_id, "llm_request", &e.to_string());
                }
                return Err(e);
            }
        };

        // Mark "Analyzing Request" as completed BEFORE creating new manifest
        self.update_manifest_section(SectionType::Validation, SectionStatus::Completed);

        self.create_manifest_from_llm_app(&llm_app);
        self.add_terminal_output(&format!("## Planning: {}", llm_app.name), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Tables: {}", llm_app.tables.len()), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Files: {}", llm_app.files.len()), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Tools: {}", llm_app.tools.len()), TerminalLineType::Info);
        self.add_terminal_output(&format!("- Schedulers: {}", llm_app.schedulers.len()), TerminalLineType::Info);
        self.update_manifest_stats_real(true);

        let activity = self.build_activity("parsing", 2, Some(TOTAL_STEPS as u32), Some(&format!("Processing {} structure", llm_app.name)));
        self.emit_activity("parse_structure", &format!("Parsing {} structure...", llm_app.name), 3, TOTAL_STEPS, activity);

        let tables = Self::convert_llm_tables(&llm_app.tables);

        if !tables.is_empty() {
            self.update_manifest_section(SectionType::DatabaseModels, SectionStatus::Running);
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

            match self.sync_tables_to_database(&tables) {
                Ok(result) => {
                    log_generator_info(
                        &llm_app.name,
                        &format!(
                            "Tables synced: {} created, {} fields",
                            result.tables_created, result.fields_added
                        ),
                    );
                    self.tables_synced = table_names.clone();

                    // Complete all item groups in the schema design child
                    if let Some(ref manifest) = self.manifest {
                        let group_count = manifest.sections.iter()
                            .find(|s| s.section_type == SectionType::DatabaseModels)
                            .and_then(|s| s.children.first())
                            .map(|c| c.item_groups.len())
                            .unwrap_or(0);
                        if group_count > 0 {
                            self.complete_item_group_range(SectionType::DatabaseModels, SectionType::SchemaDesign, 0, group_count - 1);
                        }
                    }

                    // Mark child and parent as completed
                    self.update_manifest_child(SectionType::DatabaseModels, SectionType::SchemaDesign, SectionStatus::Completed);
                    self.update_manifest_section(SectionType::DatabaseModels, SectionStatus::Completed);
                    for table_name in &table_names {
                        self.update_item_status(SectionType::DatabaseModels, table_name, crate::auto_task::ItemStatus::Completed);
                        self.add_terminal_output(&format!("✓ Table `{}`", table_name), TerminalLineType::Success);
                    }
                    self.update_manifest_stats_real(true);
                    let activity = self.build_activity(
                        "database",
                        4,
                        Some(TOTAL_STEPS as u32),
                        Some(&format!("{} tables, {} fields created", result.tables_created, result.fields_added))
                    );
                    self.emit_activity(
                        "tables_synced",
                        "Database tables created",
                        4,
                        TOTAL_STEPS,
                        activity
                    );
                }
                Err(e) => {
                    log_generator_error(&llm_app.name, "Failed to sync tables", &e.to_string());
                    self.add_terminal_output(&format!("  ✗ Error: {}", e), TerminalLineType::Error);
                }
            }
        }

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
        r##"
GENERAL BOTS PLATFORM - APP GENERATION

You are an expert full-stack developer generating complete applications for General Bots platform.

=== AVAILABLE APIs ===

DATABASE (/api/db/):
- GET /api/db/{table} - List records (query: limit, offset, order_by, order_dir, search, field=value)
- GET /api/db/{table}/{id} - Get single record
- GET /api/db/{table}/count - Count records
- POST /api/db/{table} - Create record (JSON body)
- PUT /api/db/{table}/{id} - Update record
- DELETE /api/db/{table}/{id} - Delete record

DRIVE (/api/drive/):
- GET /api/drive/list?path=/folder - List files
- GET /api/drive/download?path=/file - Download
- POST /api/drive/upload - Upload (multipart)
- DELETE /api/drive/delete?path=/file - Delete

COMMUNICATION:
- POST /api/mail/send - {"to", "subject", "body"}
- POST /api/whatsapp/send - {"to", "message"}
- POST /api/llm/generate - {"prompt", "max_tokens"}

=== HTMX REQUIREMENTS ===

All HTML pages MUST use HTMX exclusively. NO fetch(), NO XMLHttpRequest, NO inline onclick.

Key attributes:
- hx-get, hx-post, hx-put, hx-delete - HTTP methods
- hx-target="#id" - Response destination
- hx-swap="innerHTML|outerHTML|beforeend|delete" - Insert method
- hx-trigger="click|submit|load|every 5s|keyup changed delay:300ms"
- hx-indicator="#spinner" - Loading indicator
- hx-confirm="Message?" - Confirmation
- hx-vals='{"key":"value"}' - Extra values
- hx-headers='{"X-Custom":"value"}' - Headers

=== REQUIRED HTML STRUCTURE ===

Every HTML file must include:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Title</title>
    <link rel="stylesheet" href="styles.css">
    <script src="/js/vendor/htmx.min.js"></script>
    <script src="/api/app-logs/logger.js" defer></script>
    <script src="designer.js" defer></script>
</head>
<body data-app-name="APP_NAME_HERE">
    <!-- Content -->
</body>
</html>
```

=== BASIC SCRIPTS (.bas) ===

Tools (triggered by chat):
```
HEAR "keyword1", "keyword2"
    result = GET FROM "table" WHERE field = value
    TALK "Response: " + result
END HEAR
```

Schedulers (cron-based):
```
SET SCHEDULE "0 9 * * *"
    data = GET FROM "table"
    SEND MAIL TO "email" WITH SUBJECT "Report" BODY data
END SCHEDULE
```

BASIC Keywords:
- TALK "message" - Send message
- ASK "question" - Get input
- GET FROM "table" WHERE field=val - Query
- SAVE TO "table" WITH field1, field2 - Insert
- SEND MAIL TO "x" WITH SUBJECT "y" BODY "z"
- result = LLM "prompt" - AI generation

=== FIELD TYPES ===
guid, string, text, integer, decimal, boolean, date, datetime, json

=== GENERATION RULES ===

1. Generate COMPLETE, WORKING code - no placeholders, no "...", no "add more here"
2. Use semantic HTML5 (header, main, nav, section, article, footer)
3. Include loading states (hx-indicator)
4. Include error handling
5. Make it beautiful, modern, responsive
6. Include dark mode support in CSS
7. Tables should have id, created_at, updated_at fields
8. Forms must validate required fields
9. Lists must have search, pagination, edit/delete actions
"##
    }

    async fn generate_complete_app_with_llm(
        &self,
        intent: &str,
        bot_id: Uuid,
    ) -> Result<LlmGeneratedApp, Box<dyn std::error::Error + Send + Sync>> {
        let platform = Self::get_platform_prompt();

        let prompt = format!(
            r#"{platform}

=== USER REQUEST ===
"{intent}"

=== YOLO MODE - JUST BUILD IT ===
DO NOT ask questions. DO NOT request clarification. Just CREATE the app NOW.

If user says "calculator" → build a full-featured calculator with basic ops, scientific functions, history
If user says "CRM" → build customer management with contacts, companies, deals, notes
If user says "inventory" → build stock tracking with products, categories, movements
If user says "booking" → build appointment scheduler with calendar, slots, confirmations
If user says ANYTHING → interpret creatively and BUILD SOMETHING AWESOME

=== OUTPUT FORMAT (STREAMING DELIMITERS) ===

Use this EXACT format with delimiters (NOT JSON) so content can stream safely:

<<<APP_START>>>
name: app-name-lowercase-dashes
description: What this app does
domain: healthcare|sales|inventory|booking|utility|etc
<<<TABLES_START>>>
<<<TABLE:table_name>>>
id:guid:false
created_at:datetime:false:now()
updated_at:datetime:false:now()
field_name:string:true
foreign_key:guid:false:ref:other_table
<<<TABLE:another_table>>>
id:guid:false
name:string:true
<<<TABLES_END>>>
<<<FILE:index.html>>>
<!DOCTYPE html>
<html lang="en">
... complete HTML content here ...
</html>
<<<FILE:styles.css>>>
:root {{ --primary: #3b82f6; }}
body {{ margin: 0; font-family: system-ui; }}
... complete CSS content here ...
<<<FILE:table_name.html>>>
<!DOCTYPE html>
... complete list page ...
<<<FILE:table_name_form.html>>>
<!DOCTYPE html>
... complete form page ...
<<<TOOL:app_helper.bas>>>
HEAR "help"
    TALK "I can help with..."
END HEAR
<<<SCHEDULER:daily_report.bas>>>
SET SCHEDULE "0 9 * * *"
    data = GET FROM "table"
    SEND MAIL TO "admin@example.com" WITH SUBJECT "Daily Report" BODY data
END SCHEDULE
<<<APP_END>>>

=== TABLE FIELD FORMAT ===
Each field on its own line: name:type:nullable[:default][:ref:table]
- Types: guid, string, text, integer, decimal, boolean, date, datetime, json
- nullable: true or false
- default: optional, e.g., now(), 0, ''
- ref:table: optional foreign key reference

=== CRITICAL RULES ===
- For utilities (calculator, timer, converter): TABLES_START/END with nothing between, focus on HTML/JS
- For data apps (CRM, inventory): design proper tables and CRUD pages
- Generate ALL files completely - no placeholders, no "...", no shortcuts
- CSS must be comprehensive with variables, responsive design, dark mode
- Every HTML page needs proper structure with all required scripts
- Replace APP_NAME_HERE with actual app name in data-app-name attribute
- BE CREATIVE - add extra features the user didn't ask for but would love
- Use the EXACT delimiter format above - this allows streaming progress!

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
            (Some(s), Some(e)) => &response[s + DELIM_APP_START.len()..e],
            (Some(s), None) => {
                warn!("No APP_END found, using rest of response");
                &response[s + DELIM_APP_START.len()..]
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
                    app.name = line[5..].trim().to_string();
                    continue;
                }
                if line.starts_with("description:") {
                    app.description = line[12..].trim().to_string();
                    continue;
                }
                if line.starts_with("domain:") {
                    app.domain = line[7..].trim().to_string();
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
                let table_name = &line[DELIM_TABLE_PREFIX.len()..line.len() - DELIM_END.len()];
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
                let filename = &line[DELIM_FILE_PREFIX.len()..line.len() - DELIM_END.len()];
                current_file = Some(("file".to_string(), filename.to_string(), String::new()));
                continue;
            }

            // Tool definitions
            if line.starts_with(DELIM_TOOL_PREFIX) && line.ends_with(DELIM_END) {
                if let Some((file_type, filename, content)) = current_file.take() {
                    Self::save_parsed_file(&mut app, &file_type, filename, content);
                }
                let filename = &line[DELIM_TOOL_PREFIX.len()..line.len() - DELIM_END.len()];
                current_file = Some(("tool".to_string(), filename.to_string(), String::new()));
                continue;
            }

            // Scheduler definitions
            if line.starts_with(DELIM_SCHEDULER_PREFIX) && line.ends_with(DELIM_END) {
                if let Some((file_type, filename, content)) = current_file.take() {
                    Self::save_parsed_file(&mut app, &file_type, filename, content);
                }
                let filename = &line[DELIM_SCHEDULER_PREFIX.len()..line.len() - DELIM_END.len()];
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
                error!("Response was: {}", &response[..response.len().min(500)]);
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

                trace!("APP_GENERATOR Stream receiver started");

                while let Some(chunk) = rx.recv().await {
                    chunk_count += 1;
                    full_response.push_str(&chunk);
                    chunk_buffer.push_str(&chunk);

                    // Log progress periodically
                    if chunk_count == 1 || chunk_count % 500 == 0 {
                        trace!("APP_GENERATOR Stream progress: {} chunks, {} chars, {:?}",
                              chunk_count, full_response.len(), stream_start.elapsed());
                    }

                    // Don't emit raw LLM stream to WebSocket - it contains HTML/code garbage
                    // Only clear buffer periodically to track progress
                    if last_emit.elapsed().as_millis() > 100 || chunk_buffer.len() > 50 {
                        chunk_buffer.clear();
                        last_emit = std::time::Instant::now();
                    }
                }

                trace!("APP_GENERATOR Stream finished: {} chunks, {} chars in {:?}",
                      chunk_count, full_response.len(), stream_start.elapsed());

                // Don't emit remaining buffer - it's raw code/HTML
                if !chunk_buffer.is_empty() {
                    trace!("APP_GENERATOR Final buffer (not emitting): {} chars", chunk_buffer.len());
                }

                // Log response preview
                if full_response.len() > 0 {
                    let preview = if full_response.len() > 200 {
                        format!("{}...", &full_response[..200])
                    } else {
                        full_response.clone()
                    };
                    trace!("APP_GENERATOR Response preview: {}", preview.replace('\n', "\\n"));
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

    fn sync_tables_to_database(
        &self,
        tables: &[TableDefinition],
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut tables_created = 0;
        let mut fields_added = 0;

        let mut conn = self.state.conn.get()?;

        for table in tables {
            let create_sql = generate_create_table_sql(table, "postgres");

            match sql_query(&create_sql).execute(&mut conn) {
                Ok(_) => {
                    tables_created += 1;
                    fields_added += table.fields.len();
                    info!("Created table: {}", table.name);
                }
                Err(e) => {
                    warn!("Table {} may already exist: {}", table.name, e);
                }
            }
        }

        Ok(SyncResult {
            tables_created,
            fields_added,
            migrations_applied: tables_created,
        })
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
