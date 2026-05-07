use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

pub type DbPool = Arc<Pool<ConnectionManager<PgConnection>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub context_data: serde_json::Value,
    pub current_tool: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskProgressEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub task_id: String,
    pub step: String,
    pub message: String,
    pub progress: u8,
    pub total_steps: u8,
    pub current_step: u8,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<AgentActivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl TaskProgressEvent {
    pub fn new(
        task_id: impl Into<String>,
        step: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            event_type: "task_progress".to_string(),
            task_id: task_id.into(),
            step: step.into(),
            message: message.into(),
            progress: 0,
            total_steps: 0,
            current_step: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: None,
            error: None,
            activity: None,
            text: None,
        }
    }

    #[must_use]
    pub fn with_progress(mut self, current: u8, total: u8) -> Self {
        self.current_step = current;
        self.total_steps = total;
        self.progress = if total > 0 {
            ((current as u16 * 100) / total as u16) as u8
        } else {
            0
        };
        self
    }

    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    #[must_use]
    pub fn with_activity(mut self, activity: AgentActivity) -> Self {
        self.activity = Some(activity);
        self
    }

    #[must_use]
    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = event_type.into();
        self
    }

    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.event_type = "task_error".to_string();
        self.error = Some(error.into());
        self
    }

    #[must_use]
    pub fn completed(mut self) -> Self {
        self.event_type = "task_completed".to_string();
        self.progress = 100;
        self
    }

    pub fn started(
        task_id: impl Into<String>,
        message: impl Into<String>,
        total_steps: u8,
    ) -> Self {
        Self {
            event_type: "task_started".to_string(),
            task_id: task_id.into(),
            step: "init".to_string(),
            message: message.into(),
            progress: 0,
            total_steps,
            current_step: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: None,
            error: None,
            activity: None,
            text: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentActivity {
    pub phase: String,
    pub items_processed: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_per_min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_processed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_created: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tables_created: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_lines: Option<Vec<String>>,
}

impl AgentActivity {
    pub fn new(phase: impl Into<String>) -> Self {
        Self {
            phase: phase.into(),
            items_processed: 0,
            items_total: None,
            speed_per_min: None,
            eta_seconds: None,
            current_item: None,
            bytes_processed: None,
            tokens_used: None,
            files_created: None,
            tables_created: None,
            log_lines: None,
        }
    }

    #[must_use]
    pub fn with_progress(mut self, processed: u32, total: Option<u32>) -> Self {
        self.items_processed = processed;
        self.items_total = total;
        self
    }

    #[must_use]
    pub fn with_speed(mut self, speed: f32, eta: Option<u32>) -> Self {
        self.speed_per_min = Some(speed);
        self.eta_seconds = eta;
        self
    }

    #[must_use]
    pub fn with_bytes(mut self, bytes: u64) -> Self {
        self.bytes_processed = Some(bytes);
        self
    }

    #[must_use]
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files_created = Some(files);
        self
    }

    #[must_use]
    pub fn with_tables(mut self, tables: Vec<String>) -> Self {
        self.tables_created = Some(tables);
        self
    }

    #[must_use]
    pub fn with_current_item(mut self, item: impl Into<String>) -> Self {
        self.current_item = Some(item.into());
        self
    }

    #[must_use]
    pub fn with_log_lines(mut self, lines: Vec<String>) -> Self {
        self.log_lines = Some(lines);
        self
    }

    #[must_use]
    pub fn with_tokens(mut self, tokens: u32) -> Self {
        self.tokens_used = Some(tokens);
        self
    }
}

pub trait AutoTaskState: Send + Sync {
    fn db_pool(&self) -> &DbPool;
    fn bucket_name(&self) -> &str;
    fn broadcast_task_progress(&self, event: TaskProgressEvent);
    fn emit_activity(
        &self,
        task_id: &str,
        step: &str,
        message: &str,
        current: u8,
        total: u8,
        activity: AgentActivity,
    );
    fn emit_task_started(&self, task_id: &str, message: &str, total_steps: u8);
    fn emit_task_error(&self, task_id: &str, step: &str, error: &str);
    fn task_manifests(&self) -> &Arc<RwLock<HashMap<String, crate::TaskManifest>>>;
    fn task_progress_broadcast(&self) -> Option<&broadcast::Sender<TaskProgressEvent>>;
}

pub trait BotDatabaseOps: Send + Sync {
    fn create_table_in_bot_database(
        &self,
        bot_id: Uuid,
        sql: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub trait LlmProviderOps: Send + Sync {
    fn generate_stream(
        &self,
        prompt: &str,
        config: &serde_json::Value,
        tx: tokio::sync::mpsc::Sender<String>,
        model: &str,
        key: &str,
        system_prompt: Option<&str>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
    >;
}

pub trait ConfigOps: Send + Sync {
    fn get_config(
        &self,
        bot_id: &Uuid,
        key: &str,
        default: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    fn set_config(
        &self,
        bot_id: &Uuid,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub trait DriveOps: Send + Sync {
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
    >;
}

pub trait ScriptRunner: Send + Sync {
    fn run_script(
        &self,
        script_name: &str,
        bot_id: Uuid,
        session: &UserSession,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub fn get_content_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "md" => "text/markdown",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "bas" => "text/plain",
        _ => "application/octet-stream",
    }
}

pub fn generate_create_table_sql(table: &crate::TableDefinition, driver: &str) -> String {
    if driver != "postgres" {
        return String::new();
    }

    let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", table.name);

    let mut field_lines = Vec::new();
    for field in &table.fields {
        let mut line = format!("  {}", field.name);

        let pg_type = match field.field_type.to_lowercase().as_str() {
            "guid" | "uuid" => "UUID".to_string(),
            "string" | "varchar" => "VARCHAR(255)".to_string(),
            "text" => "TEXT".to_string(),
            "integer" | "int" => "INTEGER".to_string(),
            "decimal" | "numeric" | "float" | "double" => "DECIMAL(10,2)".to_string(),
            "boolean" | "bool" => "BOOLEAN".to_string(),
            "date" => "DATE".to_string(),
            "datetime" | "timestamp" => "TIMESTAMPTZ".to_string(),
            "json" | "jsonb" => "JSONB".to_string(),
            "bigint" => "BIGINT".to_string(),
            "serial" | "autoincrement" => "SERIAL".to_string(),
            other => other.to_string(),
        };
        line.push_str(&format!(" {}", pg_type));

        if field.is_key {
            line.push_str(" PRIMARY KEY");
            if field.field_type.to_lowercase() == "guid" || field.field_type.to_lowercase() == "uuid" {
                line.push_str(" DEFAULT gen_random_uuid()");
            }
        }

        if !field.is_nullable && !field.is_key {
            line.push_str(" NOT NULL");
        }

        if let Some(ref default) = field.default_value {
            if !field.is_key {
                line.push_str(&format!(" DEFAULT {}", default));
            }
        }

        if let Some(ref _refs) = field.reference_table {
            // References handled via foreign key constraints separately
        }

        field_lines.push(line);
    }

    sql.push_str(&field_lines.join(",\n"));
    sql.push_str("\n)");
    sql
}
