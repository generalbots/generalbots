-- LLM Feature Tables (Observability, Costs, Episodic Memory)

-- Conversation Cost Tracking
CREATE TABLE IF NOT EXISTS conversation_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model_used VARCHAR(100),
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cost_usd DECIMAL(10, 6) NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conv_costs_session ON conversation_costs(session_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_user ON conversation_costs(user_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_bot ON conversation_costs(bot_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_time ON conversation_costs(timestamp);

-- LLM Metrics
CREATE TABLE IF NOT EXISTS llm_metrics (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model VARCHAR(200) NOT NULL,
    request_type VARCHAR(50) NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    latency_ms BIGINT NOT NULL DEFAULT 0,
    ttft_ms BIGINT,
    cached BOOLEAN NOT NULL DEFAULT false,
    success BOOLEAN NOT NULL DEFAULT true,
    error TEXT,
    estimated_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_llm_metrics_bot_id ON llm_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_session_id ON llm_metrics(session_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_timestamp ON llm_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_model ON llm_metrics(model);

-- LLM Metrics Hourly
CREATE TABLE IF NOT EXISTS llm_metrics_hourly (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    hour TIMESTAMP WITH TIME ZONE NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    successful_requests BIGINT NOT NULL DEFAULT 0,
    failed_requests BIGINT NOT NULL DEFAULT 0,
    cache_hits BIGINT NOT NULL DEFAULT 0,
    cache_misses BIGINT NOT NULL DEFAULT 0,
    total_input_tokens BIGINT NOT NULL DEFAULT 0,
    total_output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p50_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p95_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p99_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_latency_ms BIGINT NOT NULL DEFAULT 0,
    min_latency_ms BIGINT NOT NULL DEFAULT 0,
    requests_by_model JSONB NOT NULL DEFAULT '{}',
    tokens_by_model JSONB NOT NULL DEFAULT '{}',
    cost_by_model JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, hour)
);

CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_bot_id ON llm_metrics_hourly(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_hour ON llm_metrics_hourly(hour DESC);

-- LLM Budget
CREATE TABLE IF NOT EXISTS llm_budget (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL UNIQUE,
    daily_limit DOUBLE PRECISION NOT NULL DEFAULT 100,
    monthly_limit DOUBLE PRECISION NOT NULL DEFAULT 2000,
    alert_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.8,
    daily_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    monthly_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    daily_reset_date DATE NOT NULL DEFAULT CURRENT_DATE,
    monthly_reset_date DATE NOT NULL DEFAULT DATE_TRUNC('month', CURRENT_DATE)::DATE,
    daily_alert_sent BOOLEAN NOT NULL DEFAULT false,
    monthly_alert_sent BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- LLM Traces
CREATE TABLE IF NOT EXISTS llm_traces (
    id UUID PRIMARY KEY,
    parent_id UUID,
    trace_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    component VARCHAR(100) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    duration_ms BIGINT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    attributes JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'in_progress',
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_llm_traces_trace_id ON llm_traces(trace_id);
CREATE INDEX IF NOT EXISTS idx_llm_traces_start_time ON llm_traces(start_time DESC);
CREATE INDEX IF NOT EXISTS idx_llm_traces_component ON llm_traces(component);

-- Episodic Memories
CREATE TABLE IF NOT EXISTS episodic_memories (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    session_id UUID,
    summary TEXT NOT NULL,
    key_topics JSONB DEFAULT '[]',
    decisions JSONB DEFAULT '[]',
    action_items JSONB DEFAULT '[]',
    message_count INTEGER NOT NULL DEFAULT 0,
    start_timestamp TIMESTAMPTZ NOT NULL,
    end_timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_episodic_bot ON episodic_memories(bot_id);
CREATE INDEX IF NOT EXISTS idx_episodic_user ON episodic_memories(user_id);
CREATE INDEX IF NOT EXISTS idx_episodic_session ON episodic_memories(session_id);
CREATE INDEX IF NOT EXISTS idx_episodic_time ON episodic_memories(bot_id, user_id, created_at);

-- Bot Reflections
CREATE TABLE IF NOT EXISTS bot_reflections (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    reflection_type TEXT NOT NULL,
    score FLOAT NOT NULL DEFAULT 0.0,
    insights TEXT NOT NULL DEFAULT '[]',
    improvements TEXT NOT NULL DEFAULT '[]',
    positive_patterns TEXT NOT NULL DEFAULT '[]',
    concerns TEXT NOT NULL DEFAULT '[]',
    raw_response TEXT NOT NULL DEFAULT '',
    messages_analyzed INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bot_reflections_bot ON bot_reflections(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_reflections_session ON bot_reflections(session_id);
CREATE INDEX IF NOT EXISTS idx_bot_reflections_time ON bot_reflections(bot_id, created_at);

-- Triggers for Updated At (e.g. Budget)
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_llm_budget_updated_at ON llm_budget;
CREATE TRIGGER update_llm_budget_updated_at
    BEFORE UPDATE ON llm_budget
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
