//! `types::run` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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

impl VibeRunState {
    /// True for the terminal states after which the run must never transition
    /// again. Guards against a late approve/cancel regressing a finished run
    /// back into a non-terminal state (stale "running" dock).
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeRunConfig {
    pub use_case: VibeUseCase,
    pub lang: String,
    pub auto_approve: bool,
    pub max_tool_calls: u32,
    pub timeout_seconds: u64,
    pub model: Option<String>,
    /// Per-bot LLM API key override (over env `LLM_KEY`).
    pub llm_key: Option<String>,
    /// Per-bot LLM endpoint override (over env `LLM_URL`).
    pub llm_url: Option<String>,
    /// Vibe agent slot this run uses: `reasoning` | `agentic` | `fast`.
    /// Resolves the per-agent LLM provider configured in Settings → Vibe.
    pub agent: Option<String>,
    pub budget_cents: u64,
    /// Vibe project this run operates on (uuid string); drives the deploy
    /// pipeline stage args and the agent's `project` workspace key.
    pub project_id: Option<String>,
    /// Project name as seen by the agent (workspace key, e.g. `calculator`).
    pub project_name: Option<String>,
    /// Pipeline the run executed: "deploy" for the production pipeline,
    /// otherwise the development agent loop. Persisted so the UI can tell
    /// a prod run from a dev run (e.g. skip auto-opening the dev browser).
    pub pipeline_mode: Option<String>,
}

impl Default for VibeRunConfig {
    fn default() -> Self {
        Self {
            use_case: VibeUseCase::SoftwareDevelopment,
            lang: "en".to_string(),
            auto_approve: false,
            max_tool_calls: 50,
            // 600s (the run-loop cap) so the LLM retry budget (~5.5min worst
            // case with a flaky provider) fits inside one run.
            timeout_seconds: 600,
            model: None,
            llm_key: None,
            llm_url: None,
            agent: None,
            budget_cents: 0,
            project_id: None,
            project_name: None,
            pipeline_mode: None,
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
        // Terminal states are absorbing: a finished run must never regress.
        // This guards against a late approve/cancel *and* a still-running
        // agent loop flipping a completed run back to "running" (stale dock).
        // Call sites that need to distinguish an already-finished run check
        // `is_terminal()` first; every other transition out of a terminal
        // state is simply ignored.
        if self.state.is_terminal() {
            return;
        }
        if new_state.is_terminal() {
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

impl VibeContext {
    pub fn new(run_id: Uuid) -> Self {
        Self {
            run_id,
            system_prompt: String::new(),
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
            internal: false,
            created_at: chrono::Utc::now(),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VibeRunSignal {
    Approved(Uuid),
    Cancelled(Uuid),
}
