use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, BoxError>> + Send>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeProgressEvent {
    pub event_type: String,
    pub run_id: String,
    pub step: String,
    pub message: String,
    pub progress: u8,
    pub total_steps: u8,
    pub current_step: u8,
    pub timestamp: String,
}

impl VibeProgressEvent {
    pub fn started(run_id: impl Into<String>, message: impl Into<String>, total_steps: u8) -> Self {
        Self {
            event_type: "vibe_started".to_string(),
            run_id: run_id.into(),
            step: "init".to_string(),
            message: message.into(),
            progress: 0,
            total_steps,
            current_step: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VibeRunState {
    Pending,
    Running,
    AwaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for VibeRunState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::AwaitingApproval => write!(f, "awaiting_approval"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VibeUseCase {
    SoftwareDevelopment,
    CustomerSupport,
    FinancialAnalysis,
}

impl std::fmt::Display for VibeUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SoftwareDevelopment => write!(f, "software_development"),
            Self::CustomerSupport => write!(f, "customer_support"),
            Self::FinancialAnalysis => write!(f, "financial_analysis"),
        }
    }
}

impl VibeUseCase {
    pub fn default_system_prompt(&self) -> &'static str {
        match self {
            Self::SoftwareDevelopment => "Você é um agente de desenvolvimento de software. Analise requisitos, gere código, revise alterações e corrija defeitos com precisão.",
            Self::CustomerSupport => "Você é um agente de atendimento ao cliente. Resolva tickets, consulte dados CRM e forneça respostas contextualizadas com cortesia profissional.",
            Self::FinancialAnalysis => "Você é um agente de análise financeira. Agregue indicadores de sentimento, gere relatórios marcados e identifique tendências de mercado.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeRunConfig {
    pub use_case: VibeUseCase,
    pub auto_approve: bool,
    pub max_tool_calls: u32,
    pub timeout_seconds: u64,
    pub model: Option<String>,
}

impl Default for VibeRunConfig {
    fn default() -> Self {
        Self {
            use_case: VibeUseCase::SoftwareDevelopment,
            auto_approve: false,
            max_tool_calls: 50,
            timeout_seconds: 300,
            model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeRun {
    pub run_id: Uuid,
    pub bot_id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub state: VibeRunState,
    pub use_case: VibeUseCase,
    pub config: VibeRunConfig,
    pub intent: String,
    pub tool_calls: Vec<VibeToolCall>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

impl VibeRun {
    pub fn new(
        bot_id: Uuid,
        session_id: Uuid,
        user_id: Uuid,
        intent: String,
        config: VibeRunConfig,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            run_id: Uuid::new_v4(),
            bot_id,
            session_id,
            user_id,
            state: VibeRunState::Pending,
            use_case: config.use_case,
            config,
            intent,
            tool_calls: Vec::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
            error: None,
        }
    }

    pub fn transition(&mut self, new_state: VibeRunState) {
        if matches!(new_state, VibeRunState::Completed | VibeRunState::Failed | VibeRunState::Cancelled) {
            self.completed_at = Some(chrono::Utc::now());
        }
        self.state = new_state;
        self.updated_at = chrono::Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeContext {
    pub run_id: Uuid,
    pub system_prompt: String,
    pub conversation_history: Vec<ContextMessage>,
    pub kb_references: Vec<String>,
    pub user_preferences: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl VibeContext {
    pub fn new(run_id: Uuid, use_case: VibeUseCase) -> Self {
        Self {
            run_id,
            system_prompt: use_case.default_system_prompt().to_string(),
            conversation_history: Vec::new(),
            kb_references: Vec::new(),
            user_preferences: HashMap::new(),
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.conversation_history.push(ContextMessage {
            role: "user".to_string(),
            content,
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.conversation_history.push(ContextMessage {
            role: "assistant".to_string(),
            content,
            timestamp: chrono::Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeToolCall {
    pub call_id: Uuid,
    pub run_id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<VibeToolResult>,
    pub requires_approval: bool,
    pub approved: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl VibeToolCall {
    pub fn new(run_id: Uuid, tool_name: String, arguments: serde_json::Value, requires_approval: bool) -> Self {
        Self {
            call_id: Uuid::new_v4(),
            run_id,
            tool_name,
            arguments,
            result: None,
            requires_approval,
            approved: false,
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeToolResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeTelemetryEvent {
    pub event_id: Uuid,
    pub run_id: Uuid,
    pub event_type: VibeTelemetryEventType,
    pub tool_name: Option<String>,
    pub use_case: VibeUseCase,
    pub latency_ms: u64,
    pub tokens_used: Option<u32>,
    pub estimated_cost: f64,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VibeTelemetryEventType {
    RunStarted,
    RunCompleted,
    RunFailed,
    ToolCallStarted,
    ToolCallCompleted,
    ToolCallFailed,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalDenied,
}

pub trait VibeState: Send + Sync {
    fn db_pool(&self) -> &DbPool;
    fn broadcast_progress(&self, event: VibeProgressEvent);
    fn progress_sender(&self) -> Option<&broadcast::Sender<VibeProgressEvent>>;
    fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, VibeRun>>>;
}

pub trait VibeLlmOps: Send + Sync {
    fn generate(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        config: &serde_json::Value,
        model: &str,
        key: &str,
    ) -> BoxFuture<String>;
}

pub trait VibeConfigOps: Send + Sync {
    fn get_config(
        &self,
        bot_id: &Uuid,
        key: &str,
        default: Option<&str>,
    ) -> Result<String, BoxError>;

    fn set_config(
        &self,
        bot_id: &Uuid,
        key: &str,
        value: &str,
    ) -> Result<(), BoxError>;
}

pub const VIBE_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS vibe_runs (
    run_id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    state VARCHAR(50) NOT NULL DEFAULT 'pending',
    use_case VARCHAR(50) NOT NULL DEFAULT 'software_development',
    config JSONB NOT NULL DEFAULT '{}',
    intent TEXT NOT NULL,
    tool_calls JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    error TEXT
);

CREATE TABLE IF NOT EXISTS vibe_telemetry (
    event_id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES vibe_runs(run_id),
    event_type VARCHAR(50) NOT NULL,
    tool_name VARCHAR(200),
    use_case VARCHAR(50) NOT NULL,
    latency_ms BIGINT NOT NULL DEFAULT 0,
    tokens_used INTEGER,
    estimated_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    success BOOLEAN NOT NULL DEFAULT true,
    error TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_vibe_runs_bot_id ON vibe_runs(bot_id);
CREATE INDEX IF NOT EXISTS idx_vibe_runs_session_id ON vibe_runs(session_id);
CREATE INDEX IF NOT EXISTS idx_vibe_runs_state ON vibe_runs(state);
CREATE INDEX IF NOT EXISTS idx_vibe_runs_created_at ON vibe_runs(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_vibe_telemetry_run_id ON vibe_telemetry(run_id);
CREATE INDEX IF NOT EXISTS idx_vibe_telemetry_event_type ON vibe_telemetry(event_type);
CREATE INDEX IF NOT EXISTS idx_vibe_telemetry_timestamp ON vibe_telemetry(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_vibe_telemetry_use_case ON vibe_telemetry(use_case);
";
