use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

pub type DbPool = Arc<Pool<ConnectionManager<PgConnection>>>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, BoxError>> + Send>>;

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
    /// Drive object-store facade used by the BASIC-only pipeline (#754).
    fn file_ops(&self) -> Option<&dyn DriveOps>;
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
    /// Git source facade (reform #1501/#1505). `None` keeps the legacy Drive
    /// write path for bots that are not git-owned.
    fn source_ops(&self) -> Option<&dyn BotSourceOps> {
        None
    }

    /// BASIC reference handed to the intent compiler: the closed keyword
    /// catalog plus the syntax rules, rendered as prompt material.
    ///
    /// The catalog lives in the host (botserver's `basic::keywords`
    /// registration) because `botautotask` cannot depend on it — without this
    /// the compiler is asked for a program while knowing no keyword and emits
    /// the `TALK` stub instead. `None` degrades to the stub.
    fn basic_reference(&self) -> Option<String> {
        None
    }
}

/// Resolved bot identity used to build Drive buckets and DriveMonitor keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInfo {
    pub id: Uuid,
    pub name: String,
}

impl BotInfo {
    /// Drive bucket for the bot (MinIO layout: `{name}.gbai`).
    pub fn bucket_name(&self) -> String {
        format!("{}.gbai", self.name)
    }

    /// Drive folder inside the bucket (`{name}.gbdialog`).
    pub fn dialog_folder(&self) -> String {
        format!("{}.gbdialog", self.name)
    }
}

/// Persisted AutoTask row fragment used by the API pipeline (no full
/// agent-state machinery required to record a created task).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub bot_id: Uuid,
    pub session_id: Option<Uuid>,
    pub title: String,
    pub intent: String,
    pub status: String,
    pub mode: String,
    pub priority: String,
    pub plan_id: Option<String>,
    pub basic_program: Option<String>,
}

pub trait BotDatabaseOps: Send + Sync {
    fn create_table_in_bot_database(
        &self,
        bot_id: Uuid,
        sql: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Resolve a bot by its database identifier (name + Drive layout).
    fn resolve_bot(&self, bot_id: Uuid) -> Result<Option<BotInfo>, BoxError>;

    /// Persist an AutoTask metadata row created by the API pipeline
    /// (status, plan id, generated BASIC program) so it is observable in
    /// `/api/autotask/tasks`.
    fn persist_task(&self, task: &TaskRecord) -> Result<(), BoxError>;
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
    ) -> BoxFuture<()>;
}

/// Hard ceiling on one AutoTask LLM completion. A stalled upstream model (free
/// tiers do stall) must degrade into a reported failure instead of holding the
/// request and its HTTP client open — an intent classification once hung for
/// more than five minutes.
/// Generous, because the compile step asks a reasoning model for a whole BASIC
/// program: 90 s aborted real runs on Telegram with "llm call timed out"
/// (the model was still emitting reasoning tokens). The budget is a safety net
/// against a stalled provider, not a latency target.
pub const LLM_CALL_TIMEOUT_SECS: u64 = 300;

/// Drive a provider stream to completion under [`LLM_CALL_TIMEOUT_SECS`].
///
/// The producer runs concurrently with the collector: `generate_stream` fills
/// a bounded channel, so awaiting it *before* draining deadlocks once the
/// model emits more chunks than the channel holds (a reasoning model writing
/// a whole BASIC program sends hundreds) — the producer blocks on `send`, the
/// collector never reaches `recv`, and the timeout then abandons a turn that
/// was actually progressing. Chat never hits this because its pipeline loops
/// on `recv` while the producer is still running.
pub async fn collect_llm_stream(
    llm_ops: &dyn LlmProviderOps,
    prompt: &str,
    config: &serde_json::Value,
    model: &str,
    key: &str,
    system_prompt: Option<&str>,
) -> Result<String, BoxError> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let producer = llm_ops.generate_stream(prompt, config, tx, model, key, system_prompt);
    let collect = async {
        let mut response = String::new();
        while let Some(chunk) = rx.recv().await {
            response.push_str(&chunk);
        }
        response
    };
    // Polled together in this task (no spawn: callers may run off-runtime):
    // the producer drains because the collector consumes concurrently. The
    // timeout stays as the stall guard for a provider that stops emitting.
    let call = async { tokio::join!(producer, collect) };
    let (producer_result, response) = tokio::time::timeout(
        std::time::Duration::from_secs(LLM_CALL_TIMEOUT_SECS),
        call,
    )
    .await
    .map_err(|_| format!("llm call timed out after {LLM_CALL_TIMEOUT_SECS}s"))?;
    // A failed stream fails the collect even with partial output: feeding
    // truncated JSON to the parser would turn a provider outage into a
    // fabricated fallback plan — an honest error lets the caller report
    // and retry instead.
    producer_result?;
    Ok(response)
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
    /// Upload a file to MinIO/Drive. Bucket follows the Drive layout
    /// (`{bot}.gbai`), key is the object path inside it.
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), BoxError>;

    /// Read an object back (verification + rollback support).
    fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, BoxError>;
}

/// Reform #1501/#1505 — persistence of AutoTask-generated bot sources into the
/// bot's git repository (ALM). For a git-owned bot the repository is the
/// canonical `.gbdialog`; a Drive-only write is overwritten by the next
/// git-pull monitor tick and leaves no artifact of the task behind.
///
/// Implemented by the host crate; `None` for bots without a git project, in
/// which case the legacy Drive path is used.
/// One source file of a bot's `.gbdialog`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub name: String,
    pub size: u64,
}

/// A bot's source files plus the vibe project that owns the repository, so a
/// caller can open them in the suite editor (project workspace + Source
/// Control) instead of guessing a path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceListing {
    pub project_id: Option<Uuid>,
    pub files: Vec<SourceFile>,
}

pub trait BotSourceOps: Send + Sync {
    /// Write `files` — `(path relative to .gbdialog, content)` — into the bot's
    /// source repository, commit and push them, returning the dialog-root file
    /// names written.
    fn write_sources(
        &self,
        bot_id: Uuid,
        files: &[(String, String)],
        message: &str,
    ) -> Result<Vec<String>, BoxError>;

    /// Merge the `BEGIN TABLE … END TABLE` blocks of `tables_bas` into the bot's
    /// `.gbdialog/tables.bas`, returning the table names appended (empty when
    /// the schema already declared them).
    fn merge_tables(&self, bot_id: Uuid, tables_bas: &str) -> Result<Vec<String>, BoxError>;

    /// Write the bot's `.gbot` configuration — `PROMPT-{CHANNEL}.md`, styles,
    /// `config.csv` — into the repository, commit and push it. The runtime reads
    /// these files from the work layout; without them the bot answers with the
    /// generic fallback prompt and its tools (e.g. `classify_media`) are never
    /// called, so a shipped template must deliver them with its tool.
    fn write_bot_config(
        &self,
        bot_id: Uuid,
        files: &[(String, String)],
        message: &str,
    ) -> Result<Vec<String>, BoxError>;

    /// Read one source file back from the bot's repository, so the editor can
    /// show the current committed content. `None` when the file is absent.
    fn read_source(&self, bot_id: Uuid, name: &str) -> Result<Option<String>, BoxError>;

    /// List the bot's `.gbdialog` sources with the owning project id.
    fn list_sources(&self, bot_id: Uuid) -> Result<SourceListing, BoxError>;
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

#[cfg(test)]
mod tests {
    use super::{collect_llm_stream, BoxError, BoxFuture, LlmProviderOps};

    /// Emits `chunks` messages before optionally failing — enough chunks to
    /// overflow the bounded channel `collect_llm_stream` builds (cap 100).
    struct ManyChunkLlm {
        chunks: usize,
        fail_after: bool,
    }

    impl LlmProviderOps for ManyChunkLlm {
        fn generate_stream(
            &self,
            _prompt: &str,
            _config: &serde_json::Value,
            tx: tokio::sync::mpsc::Sender<String>,
            _model: &str,
            _key: &str,
            _system_prompt: Option<&str>,
        ) -> BoxFuture<()> {
            let chunks = self.chunks;
            let fail_after = self.fail_after;
            Box::pin(async move {
                for i in 0..chunks {
                    if tx.send(format!("chunk-{i}")).await.is_err() {
                        return Ok(()); // collector dropped
                    }
                }
                if fail_after {
                    return Err(BoxError::from("producer failed after streaming"));
                }
                Ok(())
            })
        }
    }

    #[test]
    fn collects_more_chunks_than_the_channel_holds() {
        // The deadlock regression: a reasoning model writing a whole BASIC
        // program emits hundreds of chunks. A collector that awaits the
        // producer before draining blocks on `send` #101, times out after
        // LLM_CALL_TIMEOUT_SECS, and the AutoTask turn dies with no artifact
        // and no reply (prod, 2026-09-29).
        let adapter = ManyChunkLlm { chunks: 250, fail_after: false };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let response = rt.block_on(async {
            collect_llm_stream(&adapter, "p", &serde_json::json!({}), "m", "k", None)
                .await
                .expect("collect must succeed under the timeout")
        });
        assert_eq!(response.matches("chunk-").count(), 250);
        assert!(response.ends_with("chunk-249"));
    }

    #[test]
    fn producer_error_fails_the_collect_even_with_partial_output() {
        // A gateway drop mid-stream must surface as an error (the caller
        // reports and can retry), not as the partial text — the compiler's
        // parse fallback would otherwise fabricate a plan from truncated
        // output and claim success.
        let adapter = ManyChunkLlm { chunks: 5, fail_after: true };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let result = rt.block_on(async {
            collect_llm_stream(&adapter, "p", &serde_json::json!({}), "m", "k", None).await
        });
        let err = result.expect_err("producer error must propagate");
        assert!(err.to_string().contains("producer failed after streaming"));
    }
}
