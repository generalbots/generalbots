
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Text, Uuid as DieselUuid};
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;
#[cfg(feature = "llm")]
use crate::core::config::ConfigManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModificationType {
    Style,
    Html,
    Database,
    Tool,
    Scheduler,
    Multiple,
    Unknown,
}

impl std::fmt::Display for ModificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Style => write!(f, "STYLE"),
            Self::Html => write!(f, "HTML"),
            Self::Database => write!(f, "DATABASE"),
            Self::Tool => write!(f, "TOOL"),
            Self::Scheduler => write!(f, "SCHEDULER"),
            Self::Multiple => write!(f, "MULTIPLE"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignerContext {
    pub current_app: Option<String>,
    pub current_page: Option<String>,
    pub current_element: Option<String>,
    pub available_tables: Vec<TableInfo>,
    pub available_tools: Vec<String>,
    pub available_schedulers: Vec<String>,
    pub recent_changes: Vec<ChangeRecord>,
    pub conversation_history: Vec<ConversationTurn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub fields: Vec<String>,
    pub record_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub id: String,
    pub change_type: ModificationType,
    pub description: String,
    pub file_path: String,
    pub original_content: String,
    pub new_content: String,
    pub timestamp: DateTime<Utc>,
    pub can_undo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificationRequest {
    pub instruction: String,
    pub context: DesignerContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModificationResult {
    pub success: bool,
    pub modification_type: ModificationType,
    pub message: String,
    pub changes: Vec<FileChange>,
    pub preview: Option<String>,
    pub requires_confirmation: bool,
    pub confirmation_message: Option<String>,
    pub can_undo: bool,
    pub change_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file_path: String,
    pub change_description: String,
    pub before_snippet: Option<String>,
    pub after_snippet: Option<String>,
    pub line_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalyzedModification {
    modification_type: ModificationType,
    target_file: String,
    changes: Vec<CodeChange>,
    requires_confirmation: bool,
    confirmation_reason: Option<String>,
    summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodeChange {
    change_type: String, // "replace", "insert", "delete", "append"
    target: String,      // CSS selector, line number, or marker
    content: String,
    context: Option<String>,
}

pub struct DesignerAI {
    state: Arc<AppState>,
}

impl DesignerAI {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn process_request(
        &self,
        request: &ModificationRequest,
        session: &UserSession,
    ) -> Result<ModificationResult, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Designer processing request: {}",
            &request.instruction[..request.instruction.len().min(100)]
        );

        // Analyze what the user wants to modify
        let analysis = self
            .analyze_modification(&request.instruction, &request.context, session.bot_id)
            .await?;

        trace!("Modification analysis: {:?}", analysis.modification_type);

        // Check if confirmation is needed (destructive operations)
        if analysis.requires_confirmation {
            return Ok(ModificationResult {
                success: false,
                modification_type: analysis.modification_type,
                message: analysis.summary.clone(),
                changes: analysis
                    .changes
                    .iter()
                    .map(|c| FileChange {
                        file_path: analysis.target_file.clone(),
                        change_description: c.content.clone(),
                        before_snippet: c.context.clone(),
                        after_snippet: Some(c.content.clone()),
                        line_number: None,
                    })
                    .collect(),
                preview: Some(Self::generate_preview(&analysis)),
                requires_confirmation: true,
                confirmation_message: analysis.confirmation_reason,
                can_undo: true,
                change_id: None,
                error: None,
            });
        }

        // Apply the modification
        self.apply_modification(&analysis, session)
    }

    pub fn apply_confirmed_modification(
        &self,
        change_id: &str,
        session: &UserSession,
    ) -> Result<ModificationResult, Box<dyn std::error::Error + Send + Sync>> {
        // Retrieve pending change from storage
        let pending = self.get_pending_change(change_id, session)?;

        match pending {
            Some(analysis) => self.apply_modification(&analysis, session),
            None => Ok(ModificationResult {
                success: false,
                modification_type: ModificationType::Unknown,
                message: "Pending change not found or expired".to_string(),
                changes: Vec::new(),
                preview: None,
                requires_confirmation: false,
                confirmation_message: None,
                can_undo: false,
                change_id: None,
                error: Some("Change not found".to_string()),
            }),
        }
    }

    pub fn undo_change(
        &self,
        change_id: &str,
        session: &UserSession,
    ) -> Result<ModificationResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Undoing change: {change_id}");

        let change_record = self.get_change_record(change_id, session)?;

        match change_record {
            Some(record) if record.can_undo => {
                // Restore original content
                self.write_file(session.bot_id, &record.file_path, &record.original_content)?;

                // Remove from change history
                self.remove_change_record(change_id, session)?;

                Ok(ModificationResult {
                    success: true,
                    modification_type: record.change_type,
                    message: format!("Undone: {}", record.description),
                    changes: vec![FileChange {
                        file_path: record.file_path,
                        change_description: "Restored to previous version".to_string(),
                        before_snippet: Some(record.new_content),
                        after_snippet: Some(record.original_content),
                        line_number: None,
                    }],
                    preview: None,
                    requires_confirmation: false,
                    confirmation_message: None,
                    can_undo: false,
                    change_id: Some(change_id.to_string()),
                    error: None,
                })
            }
            Some(_) => Ok(ModificationResult {
                success: false,
                modification_type: ModificationType::Unknown,
                message: "This change cannot be undone".to_string(),
                changes: Vec::new(),
                preview: None,
                requires_confirmation: false,
                confirmation_message: None,
                can_undo: false,
                change_id: None,
                error: Some("Change is not reversible".to_string()),
            }),
            None => Ok(ModificationResult {
                success: false,
                modification_type: ModificationType::Unknown,
                message: "Change not found".to_string(),
                changes: Vec::new(),
                preview: None,
                requires_confirmation: false,
                confirmation_message: None,
                can_undo: false,
                change_id: None,
                error: Some("Change record not found".to_string()),
            }),
        }
    }

    pub fn get_context(
        &self,
        session: &UserSession,
        current_app: Option<&str>,
        current_page: Option<&str>,
    ) -> Result<DesignerContext, Box<dyn std::error::Error + Send + Sync>> {
        let available_tables = self.get_available_tables(session)?;
        let available_tools = self.get_available_tools(session)?;
        let available_schedulers = self.get_available_schedulers(session)?;
        let recent_changes = self.get_recent_changes(session, 10)?;

        Ok(DesignerContext {
            current_app: current_app.map(String::from),
            current_page: current_page.map(String::from),
            current_element: None,
            available_tables,
            available_tools,
            available_schedulers,
            recent_changes,
            conversation_history: Vec::new(),
        })
    }

    async fn analyze_modification(
        &self,
        instruction: &str,
        context: &DesignerContext,
        bot_id: Uuid,
    ) -> Result<AnalyzedModification, Box<dyn std::error::Error + Send + Sync>> {
        let context_json = serde_json::to_string(context)?;

        let prompt = format!(
            r#"You are Designer, an AI assistant that modifies applications.

USER REQUEST: "{instruction}"

CURRENT CONTEXT:
{context_json}

Analyze the request and determine what modifications to make.

Response format (JSON only):
{{
    "modification_type": "STYLE|HTML|DATABASE|TOOL|SCHEDULER|MULTIPLE",
    "target_file": "path/to/file.ext",
    "changes": [
        {{
            "change_type": "replace|insert|delete|append",
            "target": "CSS selector, line marker, or element identifier",
            "content": "new content to add/replace",
            "context": "surrounding code for context"
        }}
    ],
    "requires_confirmation": true/false,
    "confirmation_reason": "why confirmation is needed (for destructive operations)",
    "summary": "Brief description of what will change"
}}

Guidelines:
- STYLE: Changes to CSS files (colors, layout, fonts, spacing)
- HTML: Changes to HTML structure (forms, buttons, elements)
- DATABASE: Adding fields to tables.bas or creating new tables
- TOOL: Creating/modifying .gbdialog/tools/*.bas files
- SCHEDULER: Creating/modifying .gbdialog/schedulers/*.bas files
- Require confirmation for: deletions, bulk changes, database schema changes
- Use the current_app and current_page context to determine which files to modify

Respond ONLY with valid JSON."#
        );

        let response = self.call_llm(&prompt, bot_id).await?;
        Self::parse_analysis_response(&response, instruction)
    }

    fn parse_analysis_response(
        response: &str,
        instruction: &str,
    ) -> Result<AnalyzedModification, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Deserialize)]
        struct LlmAnalysis {
            modification_type: String,
            target_file: String,
            changes: Vec<LlmChange>,
            requires_confirmation: Option<bool>,
            confirmation_reason: Option<String>,
            summary: String,
        }

        #[derive(Deserialize)]
        struct LlmChange {
            change_type: String,
            target: String,
            content: String,
            context: Option<String>,
        }

        match serde_json::from_str::<LlmAnalysis>(response) {
            Ok(analysis) => {
                let mod_type = match analysis.modification_type.to_uppercase().as_str() {
                    "STYLE" => ModificationType::Style,
                    "HTML" => ModificationType::Html,
                    "DATABASE" => ModificationType::Database,
                    "TOOL" => ModificationType::Tool,
                    "SCHEDULER" => ModificationType::Scheduler,
                    "MULTIPLE" => ModificationType::Multiple,
                    _ => ModificationType::Unknown,
                };

                Ok(AnalyzedModification {
                    modification_type: mod_type,
                    target_file: analysis.target_file,
                    changes: analysis
                        .changes
                        .into_iter()
                        .map(|c| CodeChange {
                            change_type: c.change_type,
                            target: c.target,
                            content: c.content,
                            context: c.context,
                        })
                        .collect(),
                    requires_confirmation: analysis.requires_confirmation.unwrap_or(false),
                    confirmation_reason: analysis.confirmation_reason,
                    summary: analysis.summary,
                })
            }
            Err(e) => {
                warn!("Failed to parse LLM analysis: {e}");
                Self::analyze_modification_heuristic(instruction)
            }
        }
    }

    fn analyze_modification_heuristic(
        instruction: &str,
    ) -> Result<AnalyzedModification, Box<dyn std::error::Error + Send + Sync>> {
        let lower = instruction.to_lowercase();

        let (mod_type, target_file) = if lower.contains("color")
            || lower.contains("background")
            || lower.contains("font")
            || lower.contains("style")
            || lower.contains("css")
        {
            (ModificationType::Style, "styles.css".to_string())
        } else if lower.contains("button")
            || lower.contains("form")
            || lower.contains("field")
            || lower.contains("input")
            || lower.contains("add")
        {
            (ModificationType::Html, "index.html".to_string())
        } else if lower.contains("table")
            || lower.contains("column")
            || lower.contains("database")
            || lower.contains("schema")
        {
            (ModificationType::Database, "tables.bas".to_string())
        } else if lower.contains("command")
            || lower.contains("trigger")
            || lower.contains("when i say")
        {
            (
                ModificationType::Tool,
                ".gbdialog/tools/new-tool.bas".to_string(),
            )
        } else if lower.contains("schedule")
            || lower.contains("every day")
            || lower.contains("daily")
            || lower.contains("weekly")
        {
            (
                ModificationType::Scheduler,
                ".gbdialog/schedulers/new-scheduler.bas".to_string(),
            )
        } else {
            (ModificationType::Unknown, "".to_string())
        };

        Ok(AnalyzedModification {
            modification_type: mod_type,
            target_file,
            changes: vec![CodeChange {
                change_type: "manual".to_string(),
                target: instruction.to_string(),
                content: "".to_string(),
                context: None,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            summary: format!("Process: {}", instruction),
        })
    }

    fn apply_modification(
        &self,
        analysis: &AnalyzedModification,
        session: &UserSession,
    ) -> Result<ModificationResult, Box<dyn std::error::Error + Send + Sync>> {
        let change_id = Uuid::new_v4().to_string();

        // Read original file content (for undo)
        let original_content = self
            .read_file(session.bot_id, &analysis.target_file)
            .unwrap_or_default();

        // Generate new content based on modification type
        let new_content = match analysis.modification_type {
            ModificationType::Style => {
                Self::apply_style_changes(&original_content, &analysis.changes)?
            }
            ModificationType::Html => {
                Self::apply_html_changes(&original_content, &analysis.changes)?
            }
            ModificationType::Database => {
                Self::apply_database_changes(&original_content, &analysis.changes)?
            }
            ModificationType::Tool => Self::generate_tool_file(&analysis.changes)?,
            ModificationType::Scheduler => {
                Self::generate_scheduler_file(&analysis.changes)?
            }
            ModificationType::Multiple => {
                Self::apply_multiple_changes()?
            }
            ModificationType::Unknown => {
                return Ok(ModificationResult {
                    success: false,
                    modification_type: ModificationType::Unknown,
                    message: "Could not understand the modification request".to_string(),
                    changes: Vec::new(),
                    preview: None,
                    requires_confirmation: false,
                    confirmation_message: None,
                    can_undo: false,
                    change_id: None,
                    error: Some("Unknown modification type".to_string()),
                });
            }
        };

        // Write the new content
        self.write_file(session.bot_id, &analysis.target_file, &new_content)?;

        // Store change record for undo
        let change_record = ChangeRecord {
            id: change_id.clone(),
            change_type: analysis.modification_type,
            description: analysis.summary.clone(),
            file_path: analysis.target_file.clone(),
            original_content,
            new_content,
            timestamp: Utc::now(),
            can_undo: true,
        };
        self.store_change_record(&change_record, session)?;

        Ok(ModificationResult {
            success: true,
            modification_type: analysis.modification_type,
            message: analysis.summary.clone(),
            changes: analysis
                .changes
                .iter()
                .map(|c| FileChange {
                    file_path: analysis.target_file.clone(),
                    change_description: c.content.clone(),
                    before_snippet: c.context.clone(),
                    after_snippet: Some(c.content.clone()),
                    line_number: None,
                })
                .collect(),
            preview: None,
            requires_confirmation: false,
            confirmation_message: None,
            can_undo: true,
            change_id: Some(change_id),
            error: None,
        })
    }

    fn apply_style_changes(
        original: &str,
        changes: &[CodeChange],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = original.to_string();

        for change in changes {
            match change.change_type.as_str() {
                "replace" => {
                    // Replace a CSS rule
                    let pattern = format!(r"{}[\s\S]*?\}}", regex::escape(&change.target));
                    if let Ok(re) = regex::Regex::new(&pattern) {
                        content = re.replace(&content, &change.content).to_string();
                    }
                }
                "append" => {
                    // Append new CSS rules
                    content.push_str("\n\n");
                    content.push_str(&change.content);
                }
                "insert" => {
                    // Insert before a target
                    if let Some(pos) = content.find(&change.target) {
                        content.insert_str(pos, &format!("{}\n\n", change.content));
                    }
                }
                _ => {
                    content.push('\n');
                    content.push_str(&change.content);
                }
            }
        }

        Ok(content)
    }

    fn apply_html_changes(
        original: &str,
        changes: &[CodeChange],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = original.to_string();

        for change in changes {
            match change.change_type.as_str() {
                "replace" => {
                    // Simple string replacement
                    content = content.replace(&change.target, &change.content);
                }
                "insert" => {
                    // Insert after a target element (e.g., after </form>)
                    if let Some(pos) = content.find(&change.target) {
                        let insert_pos = pos + change.target.len();
                        content.insert_str(insert_pos, &format!("\n{}", change.content));
                    }
                }
                "append" => {
                    // Append before </body> or at end
                    if let Some(pos) = content.find("</body>") {
                        content.insert_str(pos, &format!("{}\n", change.content));
                    } else {
                        content.push_str(&change.content);
                    }
                }
                "delete" => {
                    content = content.replace(&change.target, "");
                }
                _ => {}
            }
        }

        Ok(content)
    }

    fn apply_database_changes(
        original: &str,
        changes: &[CodeChange],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = original.to_string();

        for change in changes {
            match change.change_type.as_str() {
                "append_field" => {
                    // Add field to existing table
                    // Find "END TABLE" for the target table and insert before it
                    let end_marker = "END TABLE";
                    if let Some(table_pos) = content.find(&change.target) {
                        if let Some(end_pos) = content[table_pos..].find(end_marker) {
                            let insert_pos = table_pos + end_pos;
                            content.insert_str(insert_pos, &format!("    {}\n", change.content));
                        }
                    }
                }
                "append" => {
                    // Add new table definition
                    content.push_str("\n\n");
                    content.push_str(&change.content);
                }
                _ => {
                    content.push('\n');
                    content.push_str(&change.content);
                }
            }
        }

        // Sync schema to database
        Self::sync_schema_changes()?;

        Ok(content)
    }

    fn generate_tool_file(
        changes: &[CodeChange],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = String::new();
        let _ = write!(
            content,
            "' Tool generated by Designer\n' Created: {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M")
        );

        for change in changes {
            if !change.content.is_empty() {
                content.push_str(&change.content);
                content.push('\n');
            }
        }

        Ok(content)
    }

    fn generate_scheduler_file(
        changes: &[CodeChange],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = String::new();
        let _ = write!(
            content,
            "' Scheduler generated by Designer\n' Created: {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M")
        );

        for change in changes {
            if !change.content.is_empty() {
                content.push_str(&change.content);
                content.push('\n');
            }
        }

        Ok(content)
    }

    fn apply_multiple_changes() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("Multiple changes applied".to_string())
    }

    fn generate_preview(analysis: &AnalyzedModification) -> String {
        let mut preview = String::new();
        let _ = writeln!(preview, "File: {}\n\nChanges:", analysis.target_file);

        for (i, change) in analysis.changes.iter().enumerate() {
            let _ = writeln!(
                preview,
                "{}. {} at '{}'",
                i + 1,
                change.change_type,
                change.target
            );
            if !change.content.is_empty() {
                let _ = writeln!(
                    preview,
                    "   New content: {}",
                    &change.content[..change.content.len().min(100)]
                );
            }
        }

        preview
    }

    fn get_available_tables(
        &self,
        _session: &UserSession,
    ) -> Result<Vec<TableInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        #[derive(QueryableByName)]
        struct TableRow {
            #[diesel(sql_type = Text)]
            table_name: String,
        }

        let tables: Vec<TableRow> = sql_query(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = 'public'
             AND table_type = 'BASE TABLE'
             LIMIT 50",
        )
        .get_results(&mut conn)
        .unwrap_or_default();

        Ok(tables
            .into_iter()
            .map(|t| TableInfo {
                name: t.table_name,
                fields: Vec::new(), // Would need separate query
                record_count: None,
            })
            .collect())
    }

    fn get_available_tools(
        &self,
        session: &UserSession,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let site_path = self.get_site_path();
        let tools_path = format!("{}/{}.gbai/.gbdialog/tools", site_path, session.bot_id);

        let mut tools = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&tools_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.to_lowercase().ends_with(".bas") {
                        tools.push(name.to_string());
                    }
                }
            }
        }

        Ok(tools)
    }

    fn get_available_schedulers(
        &self,
        session: &UserSession,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let site_path = self.get_site_path();
        let schedulers_path = format!("{}/{}.gbai/.gbdialog/schedulers", site_path, session.bot_id);

        let mut schedulers = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&schedulers_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.to_lowercase().ends_with(".bas") {
                        schedulers.push(name.to_string());
                    }
                }
            }
        }

        Ok(schedulers)
    }

    fn get_recent_changes(
        &self,
        session: &UserSession,
        limit: usize,
    ) -> Result<Vec<ChangeRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        #[derive(QueryableByName)]
        struct ChangeRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            change_type: String,
            #[diesel(sql_type = Text)]
            description: String,
            #[diesel(sql_type = Text)]
            file_path: String,
            #[diesel(sql_type = Text)]
            original_content: String,
            #[diesel(sql_type = Text)]
            new_content: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<ChangeRow> = sql_query(
            "SELECT id, change_type, description, file_path, original_content, new_content, created_at
             FROM designer_changes
             WHERE bot_id = $1
             ORDER BY created_at DESC
             LIMIT $2",
        )
        .bind::<DieselUuid, _>(session.bot_id)
        .bind::<diesel::sql_types::Integer, _>(limit as i32)
        .get_results(&mut conn)
        .unwrap_or_default();

        Ok(rows
            .into_iter()
            .map(|r| ChangeRecord {
                id: r.id,
                change_type: match r.change_type.as_str() {
                    "STYLE" => ModificationType::Style,
                    "HTML" => ModificationType::Html,
                    "DATABASE" => ModificationType::Database,
                    "TOOL" => ModificationType::Tool,
                    "SCHEDULER" => ModificationType::Scheduler,
                    _ => ModificationType::Unknown,
                },
                description: r.description,
                file_path: r.file_path,
                original_content: r.original_content,
                new_content: r.new_content,
                timestamp: r.created_at,
                can_undo: true,
            })
            .collect())
    }

    fn get_site_path(&self) -> String {
        self.state
            .config
            .as_ref()
            .map(|c| c.site_path.clone())
            .unwrap_or_else(|| "./botserver-stack/sites".to_string())
    }

    fn read_file(
        &self,
        bot_id: Uuid,
        path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let site_path = self.get_site_path();
        let full_path = format!("{}/{}.gbai/{}", site_path, bot_id, path);

        match std::fs::read_to_string(&full_path) {
            Ok(content) => Ok(content),
            Err(e) => {
                trace!("Could not read file {}: {}", full_path, e);
                Err(Box::new(e))
            }
        }
    }

    fn write_file(
        &self,
        bot_id: Uuid,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let site_path = self.get_site_path();
        let full_path = format!("{}/{}.gbai/{}", site_path, bot_id, path);

        // Create directory if needed
        if let Some(dir) = std::path::Path::new(&full_path).parent() {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }

        std::fs::write(&full_path, content)?;
        info!("Designer wrote file: {}", full_path);

        Ok(())
    }

    fn sync_schema_changes() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This would trigger the TABLE keyword parser to sync
        // For now, just log
        info!("Schema changes need to be synced to database");
        Ok(())
    }

    fn store_change_record(
        &self,
        record: &ChangeRecord,
        session: &UserSession,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        sql_query(
            "INSERT INTO designer_changes
             (id, bot_id, change_type, description, file_path, original_content, new_content, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind::<DieselUuid, _>(Uuid::parse_str(&record.id)?)
        .bind::<DieselUuid, _>(session.bot_id)
        .bind::<Text, _>(record.change_type.to_string())
        .bind::<Text, _>(&record.description)
        .bind::<Text, _>(&record.file_path)
        .bind::<Text, _>(&record.original_content)
        .bind::<Text, _>(&record.new_content)
        .bind::<diesel::sql_types::Timestamptz, _>(record.timestamp)
        .execute(&mut conn)?;

        Ok(())
    }

    fn get_change_record(
        &self,
        change_id: &str,
        session: &UserSession,
    ) -> Result<Option<ChangeRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        #[derive(QueryableByName)]
        struct ChangeRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            change_type: String,
            #[diesel(sql_type = Text)]
            description: String,
            #[diesel(sql_type = Text)]
            file_path: String,
            #[diesel(sql_type = Text)]
            original_content: String,
            #[diesel(sql_type = Text)]
            new_content: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let result: Option<ChangeRow> = sql_query(
            "SELECT id, change_type, description, file_path, original_content, new_content, created_at
             FROM designer_changes
             WHERE id = $1 AND bot_id = $2",
        )
        .bind::<Text, _>(change_id)
        .bind::<DieselUuid, _>(session.bot_id)
        .get_result(&mut conn)
        .optional()?;

        Ok(result.map(|r| ChangeRecord {
            id: r.id,
            change_type: match r.change_type.as_str() {
                "STYLE" => ModificationType::Style,
                "HTML" => ModificationType::Html,
                "DATABASE" => ModificationType::Database,
                "TOOL" => ModificationType::Tool,
                "SCHEDULER" => ModificationType::Scheduler,
                _ => ModificationType::Unknown,
            },
            description: r.description,
            file_path: r.file_path,
            original_content: r.original_content,
            new_content: r.new_content,
            timestamp: r.created_at,
            can_undo: true,
        }))
    }

    fn remove_change_record(
        &self,
        change_id: &str,
        session: &UserSession,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        sql_query("DELETE FROM designer_changes WHERE id = $1 AND bot_id = $2")
            .bind::<Text, _>(change_id)
            .bind::<DieselUuid, _>(session.bot_id)
            .execute(&mut conn)?;

        Ok(())
    }

    fn get_pending_change(
        &self,
        change_id: &str,
        session: &UserSession,
    ) -> Result<Option<AnalyzedModification>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        #[derive(QueryableByName)]
        struct PendingRow {
            #[diesel(sql_type = Text)]
            analysis_json: String,
        }

        let result: Option<PendingRow> = sql_query(
            "SELECT analysis_json FROM designer_pending_changes
             WHERE id = $1 AND bot_id = $2 AND expires_at > NOW()",
        )
        .bind::<Text, _>(change_id)
        .bind::<DieselUuid, _>(session.bot_id)
        .get_result(&mut conn)
        .optional()?;

        match result {
            Some(row) => {
                let analysis: AnalyzedModification = serde_json::from_str(&row.analysis_json)?;
                Ok(Some(analysis))
            }
            None => Ok(None),
        }
    }

    async fn call_llm(
        &self,
        _prompt: &str,
        _bot_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        trace!("Designer calling LLM");

        #[cfg(feature = "llm")]
        {
            let prompt = _prompt;
            let bot_id = _bot_id;
            // Get model and key from bot configuration
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
                "temperature": 0.3,
                "max_tokens": 2000
            });
            let response = self
                .state
                .llm_provider
                .generate(prompt, &llm_config, &model, &key)
                .await?;
            Ok(response)
        }

        #[cfg(not(feature = "llm"))]
        {
            warn!("LLM feature not enabled for Designer");
            Ok("{}".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modification_type_display() {
        assert_eq!(ModificationType::Style.to_string(), "STYLE");
        assert_eq!(ModificationType::Html.to_string(), "HTML");
        assert_eq!(ModificationType::Database.to_string(), "DATABASE");
        assert_eq!(ModificationType::Tool.to_string(), "TOOL");
        assert_eq!(ModificationType::Scheduler.to_string(), "SCHEDULER");
    }
}
