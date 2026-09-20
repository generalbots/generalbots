//! `types::schema` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeToolCall {
    pub call_id: Uuid,
    pub run_id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<VibeToolResult>,
    pub requires_approval: bool,
    pub approved: bool,
    /// Server-internal orchestration marker (deploy pipeline only). It is
    /// never part of any request payload, so an agent tool call can neither
    /// set it nor spoof the privileges it carries (e.g. the
    /// `publish/project` production stamp). The executor injects the
    /// sanctioned internal arguments AFTER schema validation.
    #[serde(default)]
    pub internal: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub trait VibeState: Send + Sync {
    fn db_pool(&self) -> &DbPool;
    fn broadcast_progress(&self, event: VibeProgressEvent);
    fn progress_sender(&self) -> Option<&broadcast::Sender<VibeProgressEvent>>;
    fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, VibeRun>>>;
    fn run_signal_sender(&self) -> Option<&broadcast::Sender<VibeRunSignal>>;
    /// Per-bot LLM settings resolved from the bot's configuration via
    /// ConfigManager — sensitive keys from Vault per-bot paths, non-sensitive
    /// keys from Drive config.csv (bot_configuration table). `None` means
    /// "use environment".
    fn llm_config(&self, bot_id: &Uuid) -> Option<LlmConfig>;

    /// Per-bot LLM settings for a specific Vibe agent type
    /// (`reasoning` | `agentic` | `fast`, or any custom slot). Falls back to
    /// [`Self::llm_config`] when the agent slot is not configured.
    fn llm_config_for(&self, bot_id: &Uuid, agent: &str) -> Option<LlmConfig> {
        let _ = (bot_id, agent);
        self.llm_config(bot_id)
    }
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

CREATE TABLE IF NOT EXISTS vibe_email_outbox (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    recipient VARCHAR(320) NOT NULL,
    subject VARCHAR(500) NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'queued',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_vibe_email_outbox_status ON vibe_email_outbox(status);
CREATE INDEX IF NOT EXISTS idx_vibe_email_outbox_bot_id ON vibe_email_outbox(bot_id);

-- #816 — canvases/issues/sessions/teams were in-memory RwLock<Vec> and
-- vanished on restart. These tables are the write-through persistence layer.
CREATE TABLE IF NOT EXISTS vibe_canvases (
    canvas_id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    project TEXT,
    content JSONB NOT NULL DEFAULT '{}',
    share_token VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vibe_canvases_project ON vibe_canvases(project);
CREATE INDEX IF NOT EXISTS idx_vibe_canvases_share_token ON vibe_canvases(share_token);

CREATE TABLE IF NOT EXISTS vibe_issues (
    issue_id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    labels JSONB NOT NULL DEFAULT '[]',
    state VARCHAR(20) NOT NULL DEFAULT 'open',
    assignee TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vibe_issues_state ON vibe_issues(state);

CREATE TABLE IF NOT EXISTS vibe_sessions (
    session_id UUID PRIMARY KEY,
    parent_session_id UUID,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    intent TEXT NOT NULL DEFAULT '',
    use_case VARCHAR(50) NOT NULL DEFAULT 'software_development',
    budget_cents BIGINT NOT NULL DEFAULT 0,
    run JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vibe_sessions_bot_id ON vibe_sessions(bot_id);
CREATE INDEX IF NOT EXISTS idx_vibe_sessions_updated_at ON vibe_sessions(updated_at DESC);

CREATE TABLE IF NOT EXISTS vibe_teams (
    team_id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    objective TEXT NOT NULL DEFAULT '',
    members JSONB NOT NULL DEFAULT '[]',
    shared_tasks JSONB NOT NULL DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'running',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_vibe_teams_status ON vibe_teams(status);

CREATE TABLE IF NOT EXISTS vibe_skills (
    skill_id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    triggers JSONB NOT NULL DEFAULT '[]',
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vibe_skills_name ON vibe_skills(name);
";
